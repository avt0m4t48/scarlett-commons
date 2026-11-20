"""Disassemble the native RDNA2 code in unpacked FSSM shader blobs with LLVM's AMDGPU disassembler.

FSSM header (little-endian qwords at 0x00..): 'FSSM' 01 01, blob size, ..., then a table where the
code range appears as (offset, size) = qword[6], qword[7] in every blob checked so far (e.g. 0x2f8, 0x100).
The code ends with s_endpgm (0xBF810000). If the header guess doesn't hold for a blob, the tool falls back to
scanning back from the first s_endpgm to the start that decodes with no invalid instructions.

Usage:
    python -I shader_disasm.py <fssm dir or file> [--out DIR] [--mcpu gfx1013] [--limit N]
Writes <out>/<name>.s and prints a summary (decoded cleanly / with invalid words / failed).
"""
import argparse
import os
import struct
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

LLVM_MC = os.environ.get("LLVM_MC", "llvm-mc")
ENDPGM = b"\x00\x00\x81\xbf"


def disasm(code: bytes, mcpu: str):
    text = " ".join(f"0x{b:02x}" for b in code)
    p = subprocess.run([LLVM_MC, "--disassemble", "-triple=amdgcn--amdpal", f"-mcpu={mcpu}", "-show-encoding"],
                       input=text, capture_output=True, text=True,
                       creationflags=getattr(subprocess, "BELOW_NORMAL_PRIORITY_CLASS", 0))
    bad = p.stderr.count("invalid instruction encoding")
    return p.stdout, bad, p.stderr


def code_range(b: bytes):
    """The code section offset is the u32 at 0xC0; instructions start 0x28 bytes into it and run to the last
    4-byte-aligned s_endpgm in the blob (some blobs hold several programs). Found by correlating against
    clean decodes of 154 random shaders."""
    off = struct.unpack_from("<I", b, 0xC0)[0]
    start = off + 0x28
    if 0 < off and start < len(b):
        last = b.rfind(ENDPGM, start)
        while last >= 0 and (last - start) % 4:
            last = b.rfind(ENDPGM, start, last)
        if last >= 0:
            return start, last + 4, "header"
    end = b.find(ENDPGM)
    return None, end + 4 if end >= 0 else None, "scan"


def one(path, out, mcpu):
    b = open(path, "rb").read()
    start, end, how = code_range(b)
    if end is None:
        return "no s_endpgm"
    if start is None:
        # Walk back in 4-byte steps from the end and keep the longest clean decode.
        start, best = end - 4, None
        for s in range(end - 4, max(0x100, end - 0x4000) - 1, -4):
            _, bad, _ = disasm(b[s:end], mcpu)
            if bad == 0:
                best = s
        if best is None:
            return "scan failed"
        start = best
    asm, bad, err = disasm(b[start:end], mcpu)
    name = os.path.splitext(os.path.basename(path))[0]
    with open(os.path.join(out, name + ".s"), "w", encoding="utf-8") as f:
        f.write(f"; {name}  code 0x{start:x}..0x{end:x} ({how}), {mcpu}, {bad} invalid words\n")
        f.write(asm)
        if err.strip():
            f.write("\n; --- llvm-mc messages ---\n; " + err.strip().replace("\n", "\n; ") + "\n")
    return "clean" if bad == 0 else "invalid words"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("src")
    ap.add_argument("--out")
    ap.add_argument("--mcpu", default="gfx1013")
    ap.add_argument("--limit", type=int)
    ap.add_argument("--jobs", type=int, default=max(1, (os.cpu_count() or 2) // 2))
    a = ap.parse_args()
    files = [a.src] if os.path.isfile(a.src) else sorted(
        os.path.join(a.src, f) for f in os.listdir(a.src) if f.endswith(".fssm"))
    files = files[: a.limit] if a.limit else files
    out = a.out or os.path.join(os.path.dirname(files[0]), "..", "disasm")
    os.makedirs(out, exist_ok=True)
    from collections import Counter
    with ThreadPoolExecutor(a.jobs) as ex:
        res = Counter(ex.map(lambda p: one(p, out, a.mcpu), files))
    print(f"{len(files)} shaders -> {os.path.abspath(out)}: {dict(res)}")


if __name__ == "__main__":
    sys.exit(main())
