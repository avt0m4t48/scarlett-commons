"""Find texture headers in carved CPU-segment blobs and pair them with their GPU blob.

Texture descriptor layout (qwords, found in GTA VI's resource CPU segments):
  q[k-2]  packed u64 (unknown: size/flags)
  q[k-1]  0x980                     (descriptor type)
  q[k]    0x60000000 + offset       (GPU data pointer into the resource's GPU segment)
  q[k+4]  u16 width, u16 height, u16 depth/array, u8 ?, u8 DXGI format
  q[k+5]  tiling / flags (0x0001 = AMD SW_4K_S verified on an R16 texture)
The GPU segment is the carved blob that follows the CPU blob.

Usage:
    python -I tex_headers.py <carve dir> [--archives a,b] [--out textures_index.json]
"""
import argparse
import json
import os
import sys
from collections import Counter

import numpy as np

DXGI = {28: "R8G8B8A8_UNORM", 29: "R8G8B8A8_UNORM_SRGB", 56: "R16_UNORM", 61: "R8_UNORM", 71: "BC1_UNORM",
        72: "BC1_UNORM_SRGB", 74: "BC2_UNORM", 77: "BC3_UNORM", 78: "BC3_UNORM_SRGB", 80: "BC4_UNORM",
        81: "BC4_SNORM", 83: "BC5_UNORM", 84: "BC5_SNORM", 87: "B8G8R8A8_UNORM", 91: "B8G8R8A8_UNORM_SRGB",
        95: "BC6H_UF16", 96: "BC6H_SF16", 98: "BC7_UNORM", 99: "BC7_UNORM_SRGB", 10: "R16G16B16A16_FLOAT",
        2: "R32G32B32A32_FLOAT", 41: "R32_FLOAT", 54: "R16_FLOAT", 34: "R16G16_FLOAT", 24: "R10G10B10A2_UNORM",
        26: "R11G11B10_FLOAT", 49: "R8G8_UNORM"}


def scan_archive(carve, arch):
    files = sorted(json.load(open(os.path.join(carve, arch, "index.json"), encoding="utf-8"))["files"],
                   key=lambda r: r["off"])
    out = []
    for i in range(len(files) - 1):
        v, g = files[i], files[i + 1]
        # CPU segment: starts with a zero qword but isn't zero-padded GPU data. The GPU segment is whatever
        # blob follows it (texture data often starts directly with BC blocks, not a zero header).
        if not (v["head"][:16] == "0" * 16 and not v["head"].startswith("0" * 32)):
            continue
        if g["head"].startswith("42400000") or g.get("type") == "bink2":
            continue
        # The GPU segment can span several consecutive blobs: everything up to the next CPU/header blob.
        chain = []
        for j in range(i + 1, min(len(files), i + 257)):
            r = files[j]
            if r["head"].startswith("42400000") or r.get("type") == "bink2" or (
                    r["head"][:16] == "0" * 16 and not r["head"].startswith("0" * 32) and chain):
                break
            chain.append([r["off"], r["size"]])
        with open(os.path.join(carve, arch, f"{v['off']:012x}.bin"), "rb") as f:
            V = f.read()
        q = np.frombuffer(V[: len(V) // 8 * 8], "<u8")
        # Any GPU pointer whose header (4 qwords later) decodes to sane dimensions and a known DXGI format.
        for k in np.nonzero(((q >> 28) == 6) & ((q >> 32) == 0))[0]:
            k = int(k)
            if k < 2 or k + 5 >= len(q):
                continue
            d = int(q[k + 4]).to_bytes(8, "little")
            w, h, depth = int.from_bytes(d[0:2], "little"), int.from_bytes(d[2:4], "little"), int.from_bytes(d[4:6], "little")
            fmt = d[7]
            if fmt not in DXGI or not (4 <= w <= 16384 and 4 <= h <= 16384 and 1 <= depth <= 2048):
                continue
            # Real texture data is 256-byte aligned (4 KB in every verified case); unaligned hits are other structs.
            if (int(q[k]) & 0xFF) != 0:
                continue
            out.append({"archive": arch, "cpu": v["off"], "gpu": g["off"], "gpu_size": g["size"], "gpu_chain": chain,
                        "data_off": int(q[k]) & 0x0FFFFFFF, "w": w, "h": h, "depth": depth, "b6": d[6], "fmt": fmt,
                        "fmt_name": DXGI.get(fmt, f"dxgi{fmt}"), "flags": int(q[k + 5]), "packed": int(q[k - 2]),
                        "dtype": int(q[k - 1]), "mips": (int(q[k + 5]) >> 16) & 0xFF, "q_index": k})
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("carve")
    ap.add_argument("--archives")
    ap.add_argument("--out")
    a = ap.parse_args()
    archs = a.archives.split(",") if a.archives else sorted(
        d for d in os.listdir(a.carve) if os.path.isfile(os.path.join(a.carve, d, "index.json")))
    allt = []
    for arch in archs:
        t = scan_archive(a.carve, arch)
        print(f"{arch}: {len(t)} texture headers", flush=True)
        allt += t
    path = a.out or os.path.join(a.carve, "textures_index.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(allt, f)
    print(f"{len(allt)} textures -> {path}")
    print("formats:", Counter(t["fmt_name"] for t in allt).most_common(12))
    print("flags (low 16 bits):", Counter(hex(t["flags"] & 0xFFFF) for t in allt).most_common(10))
    print("byte6:", Counter(t["b6"] for t in allt).most_common(8))


if __name__ == "__main__":
    sys.exit(main())
