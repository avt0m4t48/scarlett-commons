"""SHA-256 manifest of a game folder, so later copies (and the exe, when it arrives) can be checked.

Usage:
    python -I manifest.py create <game_dir> [--out FILE]
    python -I manifest.py verify <game_dir> [--manifest FILE]

On Windows it lowers its own priority to BelowNormal so the PC stays usable.
"""
import argparse
import hashlib
import json
import os
import sys
import time

CHUNK = 8 * 1024 * 1024
DEFAULT = os.path.join(os.path.dirname(__file__), "..", "reports", "manifest.json")


def below_normal() -> None:
    if os.name == "nt":
        import ctypes
        k32 = ctypes.windll.kernel32
        k32.SetPriorityClass(k32.GetCurrentProcess(), 0x00004000)  # BELOW_NORMAL_PRIORITY_CLASS


def sha256(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb", buffering=0) as f:
        while chunk := f.read(CHUNK):
            h.update(chunk)
    return h.hexdigest()


def walk(game_dir: str):
    for root, _, files in os.walk(game_dir):
        for name in sorted(files):
            p = os.path.join(root, name)
            yield os.path.relpath(p, game_dir).replace("\\", "/"), p


def create(game_dir: str, out: str) -> int:
    entries = {}
    total = sum(os.path.getsize(p) for _, p in walk(game_dir))
    done, t0 = 0, time.time()
    for rel, p in walk(game_dir):
        size = os.path.getsize(p)
        entries[rel] = {"size": size, "sha256": sha256(p)}
        done += size
        rate = done / max(time.time() - t0, 1e-6) / 2**20
        print(f"[{done / total:6.1%}] {rate:7.0f} MiB/s  {rel}", flush=True)
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    with open(out, "w", encoding="utf-8") as f:
        json.dump({"created": time.strftime("%Y-%m-%d %H:%M:%S"), "files": entries}, f, indent=1)
    print(f"{len(entries)} files, {total / 2**30:.1f} GiB -> {out}")
    return 0


def verify(game_dir: str, manifest: str) -> int:
    with open(manifest, encoding="utf-8") as f:
        want = json.load(f)["files"]
    have = dict(walk(game_dir))
    bad = 0
    for rel, info in want.items():
        p = have.get(rel)
        if p is None:
            print(f"MISSING   {rel}")
            bad += 1
        elif os.path.getsize(p) != info["size"]:
            print(f"SIZE      {rel}")
            bad += 1
        elif sha256(p) != info["sha256"]:
            print(f"CHANGED   {rel}")
            bad += 1
    for rel in sorted(set(have) - set(want)):
        print(f"NEW       {rel}")
    print("OK" if not bad else f"{bad} problem(s)")
    return 1 if bad else 0


def main() -> int:
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("create"); c.add_argument("game_dir"); c.add_argument("--out", default=DEFAULT)
    v = sub.add_parser("verify"); v.add_argument("game_dir"); v.add_argument("--manifest", default=DEFAULT)
    a = ap.parse_args()
    below_normal()
    return create(a.game_dir, a.out) if a.cmd == "create" else verify(a.game_dir, a.manifest)


if __name__ == "__main__":
    sys.exit(main())
