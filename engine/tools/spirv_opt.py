"""Sound basic-block SPIR-V peephole optimizer for tools/rdna2spirv.py output (#884).

Targets the main source of SPIR-V size and driver compile-time bloat in translated
RDNA2 shaders: per-instruction Function-scope register array OpAccessChain / OpLoad /
OpStore traffic, repeated OpLoad %bool %active guards, OpSelect on constant booleans,
OpCopyObject wrappers, dead local stores, and dead SSA temporaries.

Every transform is strictly sound:
  - Only Function-scope OpVariables that do not escape (never passed to OpFunctionCall,
    OpAtomic*, OpCopyMemory, etc.) are tracked.
  - Dynamic OpAccessChain indices (e.g. v_movrels / s_movrels) are detected per variable:
    dynamic stores invalidate all cached slots of that variable in the current block;
    dynamic loads prevent killing earlier stores to that variable; variables with any
    dynamic index are excluded from whole-function dead-slot elimination.
  - All value forwarding and OpAccessChain CSE are strictly intra-block (except entry
    block variable initializers), preserving SSA dominance for every use including OpPhi.
  - Pure DCE uses an explicit allowlist of side-effect-free opcodes and drops any
    OpDecorate / OpName lines that reference deleted SSA IDs.

As a module:
    from spirv_opt import optimize
    opt_spvasm = optimize(spvasm)

As a CLI:
    python3 -I tools/spirv_opt.py input.spvasm -o output.spvasm
"""
import argparse
import sys

_PURE_OPS = frozenset((
    "OpAccessChain",
    "OpCopyObject",
    "OpSelect",
    "OpBitcast",
    "OpIAdd", "OpISub", "OpIMul", "OpUDiv", "OpSDiv", "OpUMod", "OpSMod", "OpSRem",
    "OpFAdd", "OpFSub", "OpFMul", "OpFDiv", "OpFNegate", "OpFmaKHR",
    "OpBitwiseOr", "OpBitwiseAnd", "OpBitwiseXor", "OpNot",
    "OpShiftLeftLogical", "OpShiftRightLogical", "OpShiftRightArithmetic",
    "OpBitFieldUExtract", "OpBitFieldSExtract", "OpBitFieldInsert",
    "OpBitReverse", "OpBitCount",
    "OpIEqual", "OpINotEqual",
    "OpUGreaterThan", "OpUGreaterThanEqual", "OpULessThan", "OpULessThanEqual",
    "OpSGreaterThan", "OpSGreaterThanEqual", "OpSLessThan", "OpSLessThanEqual",
    "OpFOrdEqual", "OpFOrdNotEqual",
    "OpFOrdGreaterThan", "OpFOrdGreaterThanEqual", "OpFOrdLessThan", "OpFOrdLessThanEqual",
    "OpFUnordEqual", "OpFUnordNotEqual",
    "OpFUnordGreaterThan", "OpFUnordGreaterThanEqual", "OpFUnordLessThan", "OpFUnordLessThanEqual",
    "OpIsNan", "OpIsInf",
    "OpLogicalAnd", "OpLogicalOr", "OpLogicalNot", "OpLogicalEqual", "OpLogicalNotEqual",
    "OpCompositeConstruct", "OpCompositeExtract", "OpCompositeInsert",
    "OpVectorShuffle", "OpVectorExtractDynamic", "OpVectorTimesScalar",
    "OpConvertFToU", "OpConvertFToS", "OpConvertSToF", "OpConvertUToF",
    "OpUConvert", "OpSConvert", "OpFConvert", "OpQuantizeToF16",
    "OpIAddCarry", "OpISubBorrow", "OpUMulExtended", "OpSMulExtended",
))

_PURE_GLSL_EXT = frozenset((
    "FAbs", "SAbs", "FSign", "SSign", "Floor", "Trunc", "Round", "RoundEven",
    "Ceil", "Fract", "Sin", "Cos", "Tan", "Asin", "Acos", "Atan",
    "Sinh", "Cosh", "Tanh", "Pow", "Exp", "Log", "Exp2", "Log2",
    "Sqrt", "InverseSqrt",
    "FMin", "UMin", "SMin", "FMax", "UMax", "SMax",
    "FClamp", "UClamp", "SClamp", "NMin", "NMax", "NClamp",
    "FMix", "Step", "SmoothStep", "Fma",
    "PackHalf2x16", "UnpackHalf2x16",
    "PackSnorm4x8", "PackUnorm4x8", "PackSnorm2x16", "PackUnorm2x16",
    "UnpackSnorm4x8", "UnpackUnorm4x8", "UnpackSnorm2x16", "UnpackUnorm2x16",
    "FindILsb", "FindSMsb", "FindUMsb", "Ldexp", "FrexpStruct", "ModfStruct",
))

_TERMINATORS = frozenset((
    "OpBranch", "OpBranchConditional", "OpSwitch", "OpReturn", "OpReturnValue",
    "OpKill", "OpTerminateInvocation", "OpUnreachable",
))


def _parse_line(line: str):
    """Return (res_id_or_None, opname, args_list) for one SPIR-V assembly line, or None if blank/comment."""
    if '"' not in line:
        semi = line.find(";")
        toks = (line[:semi] if semi >= 0 else line).split()
    else:
        # Lines with quotes only appear in module header (OpExtension, OpEntryPoint, OpName, OpString).
        return None
    if not toks:
        return None
    if len(toks) >= 3 and toks[1] == "=":
        return toks[0], toks[2], toks[3:]
    return None, toks[0], toks[1:]


def _format_inst(res: str | None, op: str, args: list[str]) -> str:
    if res is not None:
        return f"         {res} = {op} {' '.join(args)}" if args else f"         {res} = {op}"
    return f"         {op} {' '.join(args)}" if args else f"         {op}"


def _optimize_function(body_lines: list[str], const_ids: set[str], const_true: set[str],
                       const_false: set[str], const_null_elem_zero: dict[str, str],
                       push_const_vars: set[str], glsl_sets: set[str],
                       decorated_ids: set[str], max_passes: int = 3) -> tuple[list[str], set[str]]:
    """Optimize one OpFunction ... OpFunctionEnd slice. Returns (new_lines, deleted_ids)."""
    # Parse all lines inside the function into mutable [res, op, args, raw_line] entries.
    # Indices 0 is OpFunction, -1 is OpFunctionEnd; keep raw text for anything unparsed.
    insts: list[list] = []
    for raw in body_lines:
        parsed = _parse_line(raw)
        if parsed is None:
            insts.append([None, None, None, raw])
        else:
            res, op, args = parsed
            insts.append([res, op, args, raw])

    deleted_ids: set[str] = set()

    for _pass in range(max_passes):
        changed = False

        # 1. Discover Function-scope variables, pointer-to-slot mappings, escapes, dynamic indices,
        #    and which labels are branch targets.
        func_vars: dict[str, str | None] = {}       # var_id -> initializer_id or None
        ptr_root: dict[str, str] = {}               # ptr_id -> root var_id (Function or PushConstant)
        ptr_slot: dict[str, tuple | None] = {}      # ptr_id -> (var_id, *const_indices) or None (dynamic)
        escaped: set[str] = set()
        dyn_indexed: set[str] = set()
        branch_targets: set[str] = set()

        for res, op, args, _ in insts:
            if op is None:
                continue
            if op == "OpVariable" and len(args) >= 2 and args[1] == "Function" and res:
                init = args[2] if len(args) >= 3 else None
                func_vars[res] = init
                ptr_root[res] = res
                ptr_slot[res] = (res,)
            elif op == "OpAccessChain" and res and len(args) >= 2:
                base = args[1]
                idxs = args[2:]
                if base in ptr_root:
                    root = ptr_root[base]
                    ptr_root[res] = root
                    base_slot = ptr_slot.get(base)
                    if base_slot is not None and all(i in const_ids for i in idxs):
                        ptr_slot[res] = base_slot + tuple(idxs)
                    else:
                        ptr_slot[res] = None
                        if root in func_vars:
                            dyn_indexed.add(root)
                elif base in push_const_vars:
                    ptr_root[res] = base
            elif op in ("OpBranch", "OpBranchConditional", "OpSwitch"):
                for tok in args:
                    if tok.startswith("%"):
                        branch_targets.add(tok)
            elif op == "OpPhi":
                for k in range(2, len(args), 2):
                    branch_targets.add(args[k])

        # Mark any Function-scope variable whose pointer is used by anything other than
        # OpLoad, OpStore (as pointer), or OpAccessChain (as base).
        for res, op, args, _ in insts:
            if op is None:
                continue
            if op == "OpLoad":
                # args = [type, ptr, ...mem_access]
                for tok in args[2:]:
                    if tok in ptr_root and ptr_root[tok] in func_vars:
                        escaped.add(ptr_root[tok])
            elif op == "OpStore":
                # args = [ptr, val, ...mem_access]; storing a Function pointer as a value escapes it
                for tok in args[1:]:
                    if tok in ptr_root and ptr_root[tok] in func_vars:
                        escaped.add(ptr_root[tok])
            elif op == "OpAccessChain":
                # args = [type, base, idx0, ...]; using a Function pointer as an index escapes it
                for tok in args[2:]:
                    if tok in ptr_root and ptr_root[tok] in func_vars:
                        escaped.add(ptr_root[tok])
            elif op != "OpVariable":
                for tok in args:
                    if tok in ptr_root and ptr_root[tok] in func_vars:
                        escaped.add(ptr_root[tok])

        trackable = {v for v in func_vars if v not in escaped}

        # 2. Forward intra-block pass:
        #    - Resolve SSA aliases (union-find via alias dict)
        #    - OpCopyObject / OpSelect(const) / OpLogical* algebraic folding
        #    - Local OpAccessChain CSE
        #    - Local store-to-load forwarding & redundant/overwritten store elimination
        alias: dict[str, str] = {}

        def resolve(tok: str) -> str:
            cur = tok
            while cur in alias:
                cur = alias[cur]
            if cur != tok:
                alias[tok] = cur
            return cur

        dead_idx: set[int] = set()
        first_label_seen = False
        in_entry_block = False
        slot_val: dict[tuple, str] = {}
        last_store: dict[tuple, int] = {}
        ac_cse: dict[tuple, str] = {}
        val_cse: dict[tuple, str] = {}
        id_type: dict[str, str] = {}            # res_id -> result_type_id
        bitcast_of: dict[str, tuple[str, str]] = {}  # res_id -> (src_id, src_type_id)
        entry_null_zero: dict[str, str] = {}  # var_id -> const_zero_id for untouched zero-init arrays in entry

        for idx, entry in enumerate(insts):
            res, op, args, _ = entry
            if op is None:
                continue

            if op == "OpLabel":
                slot_val.clear()
                last_store.clear()
                ac_cse.clear()
                val_cse.clear()
                entry_null_zero.clear()
                if not first_label_seen:
                    first_label_seen = True
                    in_entry_block = res not in branch_targets
                    if in_entry_block:
                        for v in trackable:
                            init = func_vars[v]
                            if init is None:
                                continue
                            if init in const_null_elem_zero:
                                entry_null_zero[v] = const_null_elem_zero[init]
                            elif init in const_ids:
                                slot_val[(v,)] = init
                else:
                    in_entry_block = False
                continue

            # Rewrite input operands through alias (never rewrite type arg[0] for typed ops,
            # though types are never aliased; for OpPhi, labels are never aliased either).
            if alias and args:
                new_args = [resolve(t) if t.startswith("%") else t for t in args]
                if new_args != args:
                    entry[2] = new_args
                    args = new_args
                    changed = True

            # Record result type for typed instructions (used by round-trip OpBitcast cancellation).
            if res is not None and args and op != "OpLabel":
                id_type[res] = args[0]

            # Algebraic / copy / round-trip bitcast simplifications (only when result has no decorations)
            if res is not None and res not in decorated_ids:
                if op == "OpCopyObject" and len(args) == 2:
                    alias[res] = args[1]
                    dead_idx.add(idx)
                    deleted_ids.add(res)
                    changed = True
                    continue
                if op == "OpBitcast" and len(args) == 2:
                    dst_ty, src = args[0], args[1]
                    prev_bc = bitcast_of.get(src)
                    if prev_bc is not None and prev_bc[1] == dst_ty:
                        alias[res] = prev_bc[0]
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    src_ty = id_type.get(src)
                    if src_ty is not None:
                        bitcast_of[res] = (src, src_ty)
                if op == "OpSelect" and len(args) == 4:
                    cond, x, y = args[1], args[2], args[3]
                    if cond in const_true or x == y:
                        alias[res] = x
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if cond in const_false:
                        alias[res] = y
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                if op == "OpLogicalNot" and len(args) == 2:
                    if args[1] in const_true and const_false:
                        alias[res] = next(iter(const_false))
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if args[1] in const_false and const_true:
                        alias[res] = next(iter(const_true))
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                if op == "OpLogicalAnd" and len(args) == 3:
                    a1, a2 = args[1], args[2]
                    if a1 in const_true:
                        alias[res] = a2
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if a2 in const_true:
                        alias[res] = a1
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if a1 in const_false:
                        alias[res] = a1
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if a2 in const_false:
                        alias[res] = a2
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                if op == "OpLogicalOr" and len(args) == 3:
                    a1, a2 = args[1], args[2]
                    if a1 in const_false:
                        alias[res] = a2
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if a2 in const_false:
                        alias[res] = a1
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if a1 in const_true:
                        alias[res] = a1
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue
                    if a2 in const_true:
                        alias[res] = a2
                        dead_idx.add(idx)
                        deleted_ids.add(res)
                        changed = True
                        continue

            # Local OpAccessChain CSE
            if op == "OpAccessChain" and res is not None and res not in decorated_ids:
                key = tuple(args)
                prev = ac_cse.get(key)
                if prev is not None:
                    alias[res] = prev
                    dead_idx.add(idx)
                    deleted_ids.add(res)
                    changed = True
                    continue
                ac_cse[key] = res
                continue

            # Intra-block pure-value CSE (undecorated pure ALU and pure GLSL.std.450 ops)
            if res is not None and res not in decorated_ids and (
                op in _PURE_OPS or (op == "OpExtInst" and len(args) >= 3 and args[1] in glsl_sets and args[2] in _PURE_GLSL_EXT)
            ):
                key = (op, tuple(args))
                prev = val_cse.get(key)
                if prev is not None:
                    alias[res] = prev
                    dead_idx.add(idx)
                    deleted_ids.add(res)
                    changed = True
                    continue
                val_cse[key] = res
                continue

            # Function-scope OpLoad forwarding & CSE
            if op == "OpLoad" and res is not None and len(args) >= 2:
                ptr = args[1]
                volatile = any("Volatile" in m for m in args[2:])
                root = ptr_root.get(ptr)
                if not volatile and root in trackable:
                    slot = ptr_slot.get(ptr)
                    if slot is not None:
                        cached = slot_val.get(slot)
                        if cached is None and in_entry_block and root in entry_null_zero and len(slot) == 2:
                            cached = entry_null_zero[root]
                        if cached is not None and res not in decorated_ids:
                            alias[res] = cached
                            dead_idx.add(idx)
                            deleted_ids.add(res)
                            changed = True
                            continue
                        slot_val[slot] = res
                        last_store.pop(slot, None)
                    else:
                        # Dynamic load from root: may read any slot of root, so protect all pending stores to root.
                        for s in [k for k in last_store if k[0] == root]:
                            last_store.pop(s, None)
                continue

            # Function-scope OpStore tracking & redundant/overwritten store elimination
            if op == "OpStore" and len(args) >= 2:
                ptr, val = args[0], args[1]
                volatile = any("Volatile" in m for m in args[2:])
                root = ptr_root.get(ptr)
                if not volatile and root in trackable:
                    slot = ptr_slot.get(ptr)
                    if slot is not None:
                        cur_val = slot_val.get(slot)
                        if cur_val is None and in_entry_block and root in entry_null_zero and len(slot) == 2:
                            cur_val = entry_null_zero[root]
                        if cur_val == val:
                            # Storing the exact value already held by this slot is a no-op.
                            dead_idx.add(idx)
                            changed = True
                            continue
                        prev_st = last_store.get(slot)
                        if prev_st is not None:
                            dead_idx.add(prev_st)
                            changed = True
                        slot_val[slot] = val
                        last_store[slot] = idx
                    else:
                        # Dynamic store into root: invalidate all cached slots and entry-zero assumption for root.
                        entry_null_zero.pop(root, None)
                        for s in [k for k in slot_val if k[0] == root]:
                            slot_val.pop(s, None)
                        for s in [k for k in last_store if k[0] == root]:
                            last_store.pop(s, None)
                continue

        if dead_idx:
            insts = [e for i, e in enumerate(insts) if i not in dead_idx]

        # 3. Whole-function dead-slot store elimination for non-escaped, statically-indexed Function vars:
        #    if a slot (var, ...) has zero remaining OpLoads anywhere in the function, every OpStore to it is dead.
        static_trackable = {v for v in trackable if v not in dyn_indexed}
        if static_trackable:
            loaded_slots: set[tuple] = set()
            for res, op, args, _ in insts:
                if op == "OpLoad" and len(args) >= 2:
                    slot = ptr_slot.get(args[1])
                    if slot is not None and slot[0] in static_trackable:
                        loaded_slots.add(slot)
            dead_stores: set[int] = set()
            for idx, (res, op, args, _) in enumerate(insts):
                if op == "OpStore" and len(args) >= 2:
                    if any("Volatile" in m for m in args[2:]):
                        continue
                    slot = ptr_slot.get(args[0])
                    if slot is not None and slot[0] in static_trackable and slot not in loaded_slots:
                        dead_stores.add(idx)
            if dead_stores:
                insts = [e for i, e in enumerate(insts) if i not in dead_stores]
                changed = True

        # 4. Backward pure Dead-Code Elimination (DCE) inside the function.
        use_count: dict[str, int] = {}
        for res, op, args, _ in insts:
            if op is None or not args:
                continue
            for tok in args:
                if tok.startswith("%"):
                    use_count[tok] = use_count.get(tok, 0) + 1

        def is_pure_dce_candidate(op: str, args: list[str]) -> bool:
            if op in _PURE_OPS:
                return True
            if op == "OpExtInst" and len(args) >= 3 and args[1] in glsl_sets and args[2] in _PURE_GLSL_EXT:
                return True
            if op == "OpLoad" and len(args) >= 2 and not any("Volatile" in m for m in args[2:]):
                root = ptr_root.get(args[1])
                return root in trackable or root in push_const_vars
            return False

        dce_dead: set[int] = set()
        for idx in range(len(insts) - 1, -1, -1):
            res, op, args, _ = insts[idx]
            if res is None or op is None:
                continue
            if use_count.get(res, 0) == 0 and is_pure_dce_candidate(op, args):
                dce_dead.add(idx)
                deleted_ids.add(res)
                for tok in args:
                    if tok.startswith("%"):
                        use_count[tok] = use_count.get(tok, 0) - 1

        if dce_dead:
            insts = [e for i, e in enumerate(insts) if i not in dce_dead]
            changed = True

        if not changed:
            break

    out_lines = []
    for res, op, args, raw in insts:
        if op is None or op in ("OpFunction", "OpFunctionEnd", "OpLabel", "OpVariable"):
            out_lines.append(raw)
        else:
            out_lines.append(_format_inst(res, op, args))
    return out_lines, deleted_ids


def optimize(spvasm: str, max_passes: int = 3) -> str:
    """Apply sound basic-block peephole optimizations to a SPIR-V assembly module."""
    lines = spvasm.splitlines()

    # Scan module header for constants, constant types, PushConstant globals, GLSL ext sets, decorations.
    const_ids: set[str] = set()
    const_true: set[str] = set()
    const_false: set[str] = set()
    array_elem_type: dict[str, str] = {}     # arr_type_id -> elem_type_id
    zero_const_of_type: dict[str, str] = {}  # scalar_type_id -> zero_const_id
    const_null_type: dict[str, str] = {}     # null_const_id -> array_type_id
    push_const_ptr_types: set[str] = set()
    push_const_vars: set[str] = set()
    glsl_sets: set[str] = set()
    decorated_ids: set[str] = set()

    # Split into regions: header/inter-function lines and OpFunction ... OpFunctionEnd slices.
    func_ranges: list[tuple[int, int]] = []
    fn_start = None
    for i, raw in enumerate(lines):
        if fn_start is None:
            if "OpExtInstImport" in raw and '"GLSL.std.450"' in raw:
                lhs = raw.split("=", 1)[0].strip()
                if lhs.startswith("%"):
                    glsl_sets.add(lhs)
                continue
            parsed = _parse_line(raw)
            if parsed is None:
                continue
            res, op, args = parsed
            if op == "OpFunction":
                fn_start = i
                continue
            if op == "OpDecorate" and args:
                decorated_ids.add(args[0])
            elif op == "OpConstant" and res and len(args) >= 2:
                const_ids.add(res)
                if args[1] in ("0", "0x0", "0.0"):
                    zero_const_of_type.setdefault(args[0], res)
            elif op == "OpConstantTrue" and res:
                const_ids.add(res)
                const_true.add(res)
            elif op == "OpConstantFalse" and res:
                const_ids.add(res)
                const_false.add(res)
            elif op == "OpConstantComposite" and res:
                const_ids.add(res)
            elif op == "OpConstantNull" and res and args:
                const_ids.add(res)
                const_null_type[res] = args[0]
            elif op == "OpTypeArray" and res and args:
                array_elem_type[res] = args[0]
            elif op == "OpTypePointer" and res and len(args) >= 2 and args[0] == "PushConstant":
                push_const_ptr_types.add(res)
            elif op == "OpVariable" and res and len(args) >= 2 and args[1] == "PushConstant":
                push_const_vars.add(res)
        else:
            parsed = _parse_line(raw)
            if parsed is not None and parsed[1] == "OpFunctionEnd":
                func_ranges.append((fn_start, i + 1))
                fn_start = None

    if not func_ranges:
        return spvasm

    # Map OpConstantNull array IDs -> zero constant ID of their element type (e.g. %null_v -> %c0).
    const_null_elem_zero: dict[str, str] = {}
    for null_id, arr_ty in const_null_type.items():
        elem_ty = array_elem_type.get(arr_ty)
        if elem_ty and elem_ty in zero_const_of_type:
            const_null_elem_zero[null_id] = zero_const_of_type[elem_ty]

    all_deleted: set[str] = set()
    opt_funcs: dict[int, list[str]] = {}
    for start, end in func_ranges:
        new_body, deleted = _optimize_function(
            lines[start:end], const_ids, const_true, const_false,
            const_null_elem_zero, push_const_vars, glsl_sets, decorated_ids,
            max_passes=max_passes,
        )
        opt_funcs[start] = new_body
        all_deleted.update(deleted)

    # Reassemble module, dropping OpDecorate / OpName lines that target deleted SSA IDs.
    out: list[str] = []
    func_map = {start: (end, opt_funcs[start]) for start, end in func_ranges}
    i = 0
    n = len(lines)
    while i < n:
        if i in func_map:
            end, new_body = func_map[i]
            out.extend(new_body)
            i = end
            continue
        raw = lines[i]
        if all_deleted:
            stripped = raw.lstrip()
            if stripped.startswith(("OpDecorate ", "OpName ")):
                toks = stripped.split(None, 2)
                if len(toks) >= 2 and toks[1] in all_deleted:
                    i += 1
                    continue
        out.append(raw)
        i += 1
    return "\n".join(out) + ("\n" if spvasm.endswith("\n") else "")


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="Sound basic-block SPIR-V peephole optimizer.")
    ap.add_argument("src", help="input .spvasm file")
    ap.add_argument("-o", "--out", required=True, help="output .spvasm file")
    ap.add_argument("--passes", type=int, default=3, help="max optimization passes (default: 3)")
    args = ap.parse_args(argv)
    with open(args.src, encoding="utf-8") as f:
        text = f.read()
    opt = optimize(text, max_passes=args.passes)
    with open(args.out, "w", encoding="utf-8", newline="\n") as f:
        f.write(opt)
    return 0


if __name__ == "__main__":
    sys.exit(main())
