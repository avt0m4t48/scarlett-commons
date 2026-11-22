"""Report how a set of PE imports maps onto the runtime's shim layer.

Usage:
    python -I import_inventory.py <imports.txt> [<imports.txt> ...]
    python -I import_inventory.py --pe <image.exe> [--pe <image.dll>]

An imports file lists one ``dll!function`` per line; the DLL part is case-insensitive, blank lines and ``#``
comments are ignored. ``--pe`` reads the import directory of a PE32+ image directly (standard library only);
ordinal-only imports report as ``dll!#ordinal``.

Each import gets the return class runtime/shims/registry.cpp's classify() gives it — the same table that decides
what an unimplemented thunk returns under LEONIDA_UNIMPLEMENTED=fail — and is checked against the SHIM_TABLE
registrations in runtime/shims/*.cpp: a registered ``dll!name`` has a real implementation; anything else binds to
the unimplemented-import thunk (or to the host Windows DLL, for system modules).
"""
from __future__ import annotations

import argparse
import glob
import os
import re
import struct
import sys

HRESULT = "hresult"
OTHER = "other"

# Ported from runtime/shims/registry.cpp: the xgameruntime families classify() treats as HRESULT-returning
# (kXFamilies) and the members the GDK reference shows as void/bool/int (kNotHResult).
NOT_HRESULT = frozenset({
    "XAsyncCancel", "XAsyncComplete",
    "XTaskQueueCloseHandle", "XTaskQueueDispatch", "XTaskQueueGetCurrentProcessTaskQueue",
    "XTaskQueueSetCurrentProcessTaskQueue", "XTaskQueueUnregisterMonitor", "XTaskQueueUnregisterWaiter",
    "XUserCloseHandle", "XUserCloseSignOutDeferralHandle", "XUserCompare", "XUserIsStoreUser",
    "XUserUnregisterForChangeEvent", "XUserUnregisterForDefaultAudioEndpointUtf16Changed",
    "XUserUnregisterForDeviceAssociationChanged",
    "XStoreCloseContextHandle", "XStoreCloseLicenseHandle", "XStoreCloseProductsQueryHandle",
    "XStoreIsLicenseValid", "XStoreProductsQueryHasMorePages", "XStoreUnregisterGameLicenseChanged",
    "XStoreUnregisterPackageLicenseLost",
    "XGameSaveCloseContainer", "XGameSaveCloseProvider", "XGameSaveCloseUpdate",
    "XPackageCloseInstallationMonitorHandle", "XPackageCloseMountHandle", "XPackageGetInstallationProgress",
    "XPackageIsPackagedProcess", "XPackageUninstallPackage", "XPackageUnregisterInstallationProgressChanged",
    "XPackageUnregisterPackageInstalled", "XPackageUpdateInstallationMonitor",
})
X_FAMILIES = ("XAsync", "XTaskQueue", "XUser", "XStore", "XGameSave", "XPackage", "XGameUi")


def classify(dll: str, name: str) -> str:
    """"hresult" or "other", the same split registry.cpp's classify() makes. `dll` is case-insensitive;
    `name` must match exactly, as the registry's string comparisons do."""
    dll = dll.lower()
    if dll in ("d3d12.dll", "d3d12_xs.dll"):
        if (name.startswith(("D3D12Create", "D3D12Serialize"))
                or name in ("D3D12GetDebugInterface", "D3D12EnableExperimentalFeatures",
                            "D3D12XboxCreateDevice", "D3D12XboxCreateSyntheticDevice")):
            return HRESULT
    elif dll == "dxgi.dll":
        if name.startswith("CreateDXGIFactory") or name in ("DXGIGetDebugInterface1",
                                                           "DXGIDeclareAdapterRemovalSupport"):
            return HRESULT
    elif dll in ("xaudio2_9.dll", "xaudio2_9redist.dll"):
        if name == "XAudio2Create":
            return HRESULT
    elif dll == "gameinput.dll":
        if name == "GameInputCreate":
            return HRESULT
    elif dll in ("dstorage.dll", "dstorage_xs.dll"):
        if name == "DStorageGetFactory":
            return HRESULT
    elif dll == "xgameruntime.dll":
        if name in ("XGameRuntimeInitialize", "XGameEventWrite"):
            return HRESULT
        if name in NOT_HRESULT:
            return OTHER
        if name.startswith(X_FAMILIES):
            return HRESULT
    return OTHER


def parse_import_lines(lines) -> tuple[list[tuple[str, str]], int]:
    """(dll, name) pairs from `dll!function` lines, and the count of lines skipped as malformed."""
    imports, bad = [], 0
    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        dll, bang, name = line.partition("!")
        if not bang or not dll.strip() or not name.strip():
            bad += 1
            continue
        imports.append((dll.strip().lower(), name.strip()))
    return imports, bad


class PeFormatError(ValueError):
    """The file is not a PE32+ image this parser can read."""


MAX_ENTRIES = 65536  # a table this long is not a table; stops a malformed image from looping forever


def _cstring(data: bytes, offset: int, limit: int = 4097) -> str | None:
    end = data.find(b"\0", offset, min(len(data), offset + limit))
    if end < 0:
        return None
    return data[offset:end].decode(errors="replace")


def pe_imports(path: str) -> list[tuple[str, str]]:
    """(dll, name) pairs from a PE32+ image's import directory. Ordinal-only imports come back as '#ordinal'."""
    with open(path, "rb") as f:
        data = f.read()
    if len(data) < 0x40 or data[:2] != b"MZ":
        raise PeFormatError("no MZ header")
    nt = struct.unpack_from("<I", data, 0x3C)[0]
    if nt + 24 > len(data) or data[nt:nt + 4] != b"PE\0\0":
        raise PeFormatError("no PE signature")
    (_machine, section_count, _ts, _symtab, _syms, opt_size, _chars) = struct.unpack_from(
        "<HHIIIHH", data, nt + 4)
    opt = nt + 24
    if opt + opt_size > len(data) or opt_size < 116:
        raise PeFormatError("truncated optional header")
    magic = struct.unpack_from("<H", data, opt)[0]
    if magic != 0x20B:
        raise PeFormatError(f"not a PE32+ image (magic 0x{magic:x})")
    size_of_headers = struct.unpack_from("<I", data, opt + 60)[0]
    dir_count = struct.unpack_from("<I", data, opt + 108)[0]

    sections = []
    table = opt + opt_size
    for i in range(section_count):
        at = table + i * 40
        if at + 40 > len(data):
            raise PeFormatError("truncated section table")
        vsize, rva, rawsize, rawptr = struct.unpack_from("<IIII", data, at + 8)
        sections.append((vsize, rva, rawsize, rawptr))

    def offset(rva: int) -> int | None:
        """File offset of an RVA, or None when no file bytes back it."""
        if rva < size_of_headers:
            return rva if rva < len(data) else None
        for vsize, srva, rawsize, rawptr in sections:
            span = vsize or rawsize
            if srva <= rva < srva + span:
                if rva - srva >= min(rawsize, span):
                    return None
                at = rawptr + rva - srva
                return at if at < len(data) else None
        return None

    def string_at(rva: int) -> str | None:
        at = offset(rva)
        return None if at is None else _cstring(data, at)

    if dir_count <= 1 or opt + 120 > len(data):
        return []
    imp_rva, _imp_size = struct.unpack_from("<II", data, opt + 112 + 8)  # data directory entry 1
    base = offset(imp_rva) if imp_rva else None
    if base is None:
        return []

    out = []
    for i in range(MAX_ENTRIES):
        at = base + i * 20  # IMAGE_IMPORT_DESCRIPTOR
        if at + 20 > len(data):
            break
        oft, _stamp, _forwarder, name_rva, iat_rva = struct.unpack_from("<IIIII", data, at)
        if not (oft or name_rva or iat_rva):
            break
        dll = string_at(name_rva)
        if dll is None:
            continue
        lookup = oft if oft else iat_rva
        for j in range(MAX_ENTRIES):
            entry = offset(lookup + j * 8)
            if entry is None or entry + 8 > len(data):
                break
            value = struct.unpack_from("<Q", data, entry)[0]
            if not value:
                break
            if value & (1 << 63):
                out.append((dll.lower(), f"#{value & 0xFFFF}"))
                continue
            name = string_at((value & 0x7FFFFFFF) + 2)  # skip the 2-byte hint
            out.append((dll.lower(), name if name is not None else "?"))
    return out


# ---- SHIM_TABLE cross-reference ------------------------------------------------------------------------------

STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')
ENTRY_GROUP = re.compile(r"\{([^{}]*)\}")
TABLE_RE = re.compile(r"\bSHIM_TABLE\s*\(")
MACRO_CALL = re.compile(r"\b([A-Za-z_]\w*)\s*\(")
# Only function-like macros with one parameter: the shim files build their entry lists from helpers like
# FILE_API(dll) and XGR(name). SHIM_TABLE(id, ...) itself takes two and is matched by TABLE_RE instead.
DEFINE_RE = re.compile(
    r"(?m)^\s*#\s*define\s+([A-Za-z_]\w*)\s*\(\s*([A-Za-z_]\w*)\s*\)[ \t]*((?:[^\n]*\\[ \t]*\r?\n)*[^\n]*)")


def _table_arguments(text: str) -> list[str]:
    """The argument list of each SHIM_TABLE(...) call in a shim source, comments removed."""
    args = []
    pos = 0
    while True:
        m = TABLE_RE.search(text, pos)
        if not m:
            return args
        i = m.end()  # just past '('
        depth, in_str = 1, False
        out = []
        while i < len(text):
            c = text[i]
            if in_str:
                out.append(c)
                if c == "\\" and i + 1 < len(text):
                    out.append(text[i + 1])
                    i += 2
                    continue
                if c == '"':
                    in_str = False
            elif c == '"':
                in_str = True
                out.append(c)
            elif text.startswith("//", i):
                end = text.find("\n", i)
                i = len(text) if end < 0 else end
                continue
            elif text.startswith("/*", i):
                end = text.find("*/", i + 2)
                i = len(text) if end < 0 else end + 2
                continue
            elif c == "(":
                depth += 1
                out.append(c)
            elif c == ")":
                depth -= 1
                if depth == 0:
                    break
                out.append(c)
            else:
                out.append(c)
            i += 1
        args.append("".join(out))
        pos = i + 1


def _expand_macros(text: str, macros: dict[str, tuple[str, str]], limit: int = 16) -> str:
    """SHIM_TABLE argument text with the file's one-parameter entry macros applied."""
    for _ in range(limit):
        found, out, pos = False, [], 0
        for m in MACRO_CALL.finditer(text):
            if m.group(1) not in macros:
                continue
            open_paren = m.end() - 1
            close = _balanced(text, open_paren)
            if close < 0:
                continue
            param, body = macros[m.group(1)]
            arg = text[open_paren + 1:close].strip()
            expanded = body.replace("#" + param, '"' + arg.strip('"').strip() + '"')
            expanded = re.sub(r"\b" + re.escape(param) + r"\b", arg, expanded)
            out.append(text[pos:m.start()])
            out.append(expanded)
            pos = close + 1
            found = True
        if not found:
            return text
        text = "".join(out) + text[pos:]
    return text


def _balanced(text: str, start: int) -> int:
    """Index of the ')' matching the '(' at `start`, or -1."""
    depth, in_str = 0, False
    i = start
    while i < len(text):
        c = text[i]
        if in_str:
            if c == "\\":
                i += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def shim_table_entries(shims_dir: str) -> set[str]:
    """The "dll!name" keys the SHIM_TABLE registrations in shims_dir's .cpp files implement."""
    entries = set()
    for path in sorted(glob.glob(os.path.join(shims_dir, "*.cpp"))):
        with open(path, encoding="utf-8", errors="replace") as f:
            text = f.read()
        macros = {}
        for name, param, body in DEFINE_RE.findall(text):
            macros[name] = (param, re.sub(r"\\[ \t]*\r?\n", " ", body))
        for args in _table_arguments(text):
            body = args.split(",", 1)[1] if "," in args else ""
            for group in ENTRY_GROUP.findall(_expand_macros(body, macros)):
                strings = STRING.findall(group)
                if len(strings) >= 2:
                    entries.add(strings[0].lower() + "!" + strings[1])
    return entries


def summarize(imports: list[tuple[str, str]], implemented: set[str]):
    """(rows, per_dll): rows are (dll, name, class, implemented) sorted and deduplicated; per_dll maps a dll to
    [imports, hresult, other, implemented]."""
    rows = []
    for dll, name in sorted(set(imports)):
        dll = dll.lower()
        cls = classify(dll, name)
        rows.append((dll, name, cls, f"{dll}!{name}" in implemented))
    per_dll: dict[str, list[int]] = {}
    for dll, _name, cls, impl in rows:
        c = per_dll.setdefault(dll, [0, 0, 0, 0])
        c[0] += 1
        c[1] += cls == HRESULT
        c[2] += cls == OTHER
        c[3] += impl
    return rows, per_dll


def format_report(imports: list[tuple[str, str]], implemented: set[str]) -> str:
    """The per-import and per-DLL report for (dll, name) pairs against the shim table."""
    rows, per_dll = summarize(imports, implemented)

    out = []
    width = max((len(d) + len(n) + 1 for d, n, _c, _i in rows), default=6)
    for dll, name, cls, impl in rows:
        out.append(f"{dll + '!' + name:<{width}}  {cls:<7}  "
                   f"{'implemented' if impl else 'unimplemented thunk'}")
    out.append("")

    head = ("dll", "imports", "hresult", "other", "implemented", "unimplemented thunk")
    dw = max([len(d) for d in per_dll] + [len(head[0]), len("total")])
    out.append(f"{head[0]:<{dw}}  {head[1]:>7}  {head[2]:>7}  {head[3]:>5}  {head[4]:>11}  {head[5]:>19}")
    totals = [0, 0, 0, 0]
    for dll in sorted(per_dll):
        imports_n, hresult, other, impl = per_dll[dll]
        totals = [a + b for a, b in zip(totals, per_dll[dll])]
        out.append(f"{dll:<{dw}}  {imports_n:>7}  {hresult:>7}  {other:>5}  {impl:>11}  "
                   f"{imports_n - impl:>19}")
    out.append(f"{'total':<{dw}}  {totals[0]:>7}  {totals[1]:>7}  {totals[2]:>5}  "
               f"{totals[3]:>11}  {totals[0] - totals[3]:>19}")
    return "\n".join(out) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="import_inventory.py",
                                 description="Report how PE imports map onto the runtime's shim layer.")
    ap.add_argument("imports", nargs="*", metavar="LIST",
                    help="text file(s) of dll!function lines")
    ap.add_argument("--pe", action="append", default=[], metavar="IMAGE",
                    help="read imports from a PE32+ image")
    ap.add_argument("--shims", metavar="DIR",
                    default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "runtime", "shims"),
                    help="directory of shim .cpp files to cross-reference")
    a = ap.parse_args(argv)
    if not a.imports and not a.pe:
        print("no inputs: give an imports file or --pe IMAGE", file=sys.stderr)
        return 1

    imports = []
    rc = 0
    for path in a.imports:
        try:
            with open(path, encoding="utf-8", errors="replace") as f:
                parsed, bad = parse_import_lines(f)
        except OSError as e:
            print(f"{path}: cannot read ({e.strerror or e})", file=sys.stderr)
            rc = 1
            continue
        if bad:
            print(f"{path}: {bad} line(s) without dll!name skipped", file=sys.stderr)
        imports.extend(parsed)
    for path in a.pe:
        try:
            imports.extend(pe_imports(path))
        except (PeFormatError, OSError) as e:
            print(f"{path}: not a readable PE32+ image ({e})", file=sys.stderr)
            rc = 1

    implemented = shim_table_entries(a.shims)
    print(format_report(imports, implemented), end="")
    return rc


if __name__ == "__main__":
    try:  # a redirected stdout is strict ANSI code page under -I; write '?' for what it cannot encode
        sys.stdout.reconfigure(errors="replace")
    except (AttributeError, ValueError, OSError):
        pass
    sys.exit(main())
