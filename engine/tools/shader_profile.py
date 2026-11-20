"""Profile the disassembled shader corpus: which RDNA instructions the translator must support.

Usage:
    python -I shader_profile.py <disasm dir> [--out profile.json]

Counts each mnemonic (number of shaders using it and total occurrences), groups them by encoding family, and
lists the dwords LLVM couldn't decode (likely Xbox-specific instructions) with how many shaders contain them.
"""
import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict

FAMILIES = [("s_buffer_", "smem"), ("s_load", "smem"), ("s_", "scalar"), ("v_", "vector"), ("ds_", "lds"),
            ("buffer_", "buffer"), ("tbuffer_", "buffer"), ("image_", "image"), ("global_", "global"),
            ("flat_", "flat"), ("scratch_", "scratch"), ("exp", "export")]


def family(m):
    for p, f in FAMILIES:
        if m.startswith(p):
            return f
    return "other"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("src")
    ap.add_argument("--out")
    a = ap.parse_args()
    occ, users = Counter(), Counter()
    invalid_users = Counter()
    per_stage = defaultdict(Counter)
    n = 0
    for name in os.listdir(a.src):
        if not name.endswith(".s"):
            continue
        n += 1
        stage = name[:2].lower() if name[2:3] == "_" else "?"
        seen = set()
        with open(os.path.join(a.src, name), encoding="utf-8", errors="replace") as f:
            text = f.read()
        for line in text.splitlines():
            if not line.startswith("\t"):
                continue
            m = line.split()[0]
            occ[m] += 1
            seen.add(m)
        for m in seen:
            users[m] += 1
            per_stage[stage][m] += 1
        for w in set(re.findall(r"<stdin>:\d+:\d+: warning: invalid instruction encoding\n; (0x[0-9a-f]{2}(?: 0x[0-9a-f]{2}){3})", text)):
            invalid_users[w] += 1
    fam = defaultdict(lambda: [0, 0])
    for m, c in occ.items():
        fam[family(m)][0] += 1
        fam[family(m)][1] += c
    out = {"shaders": n, "mnemonics": len(occ),
           "families": {k: {"mnemonics": v[0], "occurrences": v[1]} for k, v in sorted(fam.items())},
           "by_mnemonic": [{"m": m, "shaders": users[m], "count": occ[m]} for m, _ in users.most_common()]}
    path = a.out or os.path.join(a.src, "..", "profile.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(out, f, indent=1)
    print(f"{n} shaders, {len(occ)} distinct mnemonics -> {os.path.abspath(path)}")
    for k, v in sorted(fam.items(), key=lambda kv: -kv[1][1]):
        print(f"  {k:8} {v[0]:4} mnemonics {v[1]:>12,} uses")
    cum, total = 0, sum(occ.values())
    for i, (m, c) in enumerate(occ.most_common(), 1):
        cum += c
        if cum / total >= 0.99:
            print(f"  {i} mnemonics cover 99% of all instructions")
            break
    print("  most widespread:", ", ".join(f"{m} ({users[m]})" for m, _ in users.most_common(12)))


if __name__ == "__main__":
    sys.exit(main())
