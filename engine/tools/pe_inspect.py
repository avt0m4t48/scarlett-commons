"""Inspect a PE image (exe or dll) for porting work.

Usage:
    python -I pe_inspect.py <image> [--game-dir DIR] [--out DIR]

Writes <out>/<name>.json and <out>/<name>.md. Reads the file only; never runs it.
"""
import argparse
import json
import math
import os
import sys
from collections import Counter

import pefile

SYSTEM32 = os.path.join(os.environ.get("SystemRoot", r"C:\Windows"), "System32")

MACHINES = {0x8664: "x86-64", 0x14C: "x86", 0xAA64: "ARM64"}
SUBSYSTEMS = {2: "GUI", 3: "Console", 14: "Xbox"}


def entropy(data: bytes) -> float:
    if not data:
        return 0.0
    n = len(data)
    return -sum(c / n * math.log2(c / n) for c in Counter(data).values())


_apiset_cache: dict[str, bool] = {}


def _apiset_resolves(name: str) -> bool:
    """True if this Windows' API-set schema maps the contract to a host DLL (ntdll query; nothing is loaded)."""
    if name not in _apiset_cache:
        import ctypes
        from ctypes import wintypes

        class UNICODE_STRING(ctypes.Structure):
            _fields_ = [("Length", wintypes.USHORT), ("MaximumLength", wintypes.USHORT), ("Buffer", wintypes.LPWSTR)]

        query = ctypes.WinDLL("ntdll").ApiSetQueryApiSetPresence
        query.argtypes = [ctypes.POINTER(UNICODE_STRING), ctypes.POINTER(ctypes.c_ubyte)]
        query.restype = ctypes.c_long
        base = name[:-4] if name.lower().endswith(".dll") else name
        buf = ctypes.create_unicode_buffer(base)
        us = UNICODE_STRING(len(base) * 2, (len(base) + 1) * 2, ctypes.cast(buf, wintypes.LPWSTR))
        present = ctypes.c_ubyte(0)
        status = query(ctypes.byref(us), ctypes.byref(present))
        _apiset_cache[name] = status == 0 and bool(present.value)
    return _apiset_cache[name]


def classify_dll(name: str, game_dir: str | None) -> str:
    """bundled = ships beside the game, system = present on this Windows, shim = must be reimplemented."""
    if game_dir and os.path.isfile(os.path.join(game_dir, name)):
        return "bundled"
    low = name.lower()
    if low.startswith("api-ms-win-") or low.startswith("ext-ms-win-"):
        # API sets resolve through the OS schema: ask this Windows whether it knows the contract.
        return "system" if _apiset_resolves(name) else "shim"
    if os.path.isfile(os.path.join(SYSTEM32, name)):
        return "system"
    return "shim"


def inspect(path: str, game_dir: str | None) -> dict:
    pe = pefile.PE(path, fast_load=True)
    pe.parse_data_directories(directories=[
        pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_IMPORT"],
        pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_DELAY_IMPORT"],
        pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_EXPORT"],
        pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_TLS"],
        pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_EXCEPTION"],
        pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_BASERELOC"],
    ])
    fh, oh = pe.FILE_HEADER, pe.OPTIONAL_HEADER

    sections = []
    for s in pe.sections:
        data = s.get_data()
        e = entropy(data)
        sections.append({
            "name": s.Name.rstrip(b"\0").decode(errors="replace"),
            "rva": s.VirtualAddress, "vsize": s.Misc_VirtualSize, "rawsize": s.SizeOfRawData,
            "exec": bool(s.Characteristics & 0x20000000),
            "write": bool(s.Characteristics & 0x80000000),
            "entropy": round(e, 3),
            "looks_encrypted": e > 7.9 and len(data) > 4096,
        })

    imports = []
    for kind, attr in (("normal", "DIRECTORY_ENTRY_IMPORT"), ("delay", "DIRECTORY_ENTRY_DELAY_IMPORT")):
        for entry in getattr(pe, attr, []):
            dll = entry.dll.decode(errors="replace")
            funcs = [(i.name.decode(errors="replace") if i.name else f"#{i.ordinal}") for i in entry.imports]
            imports.append({"dll": dll, "kind": kind, "where": classify_dll(dll, game_dir), "functions": funcs})

    exports = []
    if hasattr(pe, "DIRECTORY_ENTRY_EXPORT"):
        exports = [(e.name.decode(errors="replace") if e.name else f"#{e.ordinal}") for e in pe.DIRECTORY_ENTRY_EXPORT.symbols]

    pdata = len(getattr(pe, "DIRECTORY_ENTRY_EXCEPTION", []))
    tls_callbacks = 0
    if hasattr(pe, "DIRECTORY_ENTRY_TLS"):
        cb = pe.DIRECTORY_ENTRY_TLS.struct.AddressOfCallBacks
        if cb:
            rva = cb - oh.ImageBase
            while True:
                try:
                    ptr = pe.get_qword_at_rva(rva)
                except Exception:
                    break
                if not ptr:
                    break
                tls_callbacks += 1
                rva += 8

    return {
        "file": os.path.basename(path),
        "size": os.path.getsize(path),
        "machine": MACHINES.get(fh.Machine, hex(fh.Machine)),
        "timestamp": fh.TimeDateStamp,
        "is_dll": bool(fh.Characteristics & 0x2000),
        "subsystem": SUBSYSTEMS.get(oh.Subsystem, oh.Subsystem),
        "image_base": hex(oh.ImageBase),
        "size_of_image": oh.SizeOfImage,
        "entry_rva": hex(oh.AddressOfEntryPoint),
        "has_relocs": hasattr(pe, "DIRECTORY_ENTRY_BASERELOC"),
        "dynamic_base": bool(oh.DllCharacteristics & 0x40),
        "sections": sections,
        "imports": imports,
        "export_count": len(exports),
        "exports": exports,
        "pdata_functions": pdata,
        "tls_callbacks": tls_callbacks,
    }


def to_markdown(r: dict) -> str:
    out = [f"# {r['file']}", "",
           f"- {r['machine']}, {'DLL' if r['is_dll'] else 'EXE'}, subsystem {r['subsystem']}, {r['size']:,} bytes",
           f"- image base {r['image_base']}, size {r['size_of_image']:,}, entry RVA {r['entry_rva']}",
           f"- relocations: {'yes' if r['has_relocs'] else 'NO (must load at preferred base)'}; "
           f"ASLR flag: {r['dynamic_base']}",
           f"- .pdata functions: {r['pdata_functions']:,}; TLS callbacks: {r['tls_callbacks']}; exports: {r['export_count']}",
           "", "## Sections", "", "| name | rva | vsize | exec | write | entropy | |", "|---|---|---|---|---|---|---|"]
    for s in r["sections"]:
        flag = "**encrypted?**" if s["looks_encrypted"] else ""
        out.append(f"| {s['name']} | {s['rva']:#x} | {s['vsize']:,} | {s['exec']} | {s['write']} | {s['entropy']} | {flag} |")
    out += ["", "## Imports", ""]
    by_where = Counter()
    for imp in r["imports"]:
        by_where[imp["where"]] += len(imp["functions"])
    out.append(", ".join(f"{k}: {v}" for k, v in sorted(by_where.items())) or "none")
    out += ["", "| dll | kind | resolves | functions |", "|---|---|---|---|"]
    for imp in sorted(r["imports"], key=lambda i: (i["where"] != "shim", i["dll"].lower())):
        out.append(f"| {imp['dll']} | {imp['kind']} | {imp['where']} | {len(imp['functions'])} |")
    shims = [i for i in r["imports"] if i["where"] in ("shim", "apiset")]
    if shims:
        out += ["", "## Needs checking or a shim", ""]
        for imp in shims:
            out.append(f"### {imp['dll']} ({imp['where']})")
            out.append("")
            out.append(", ".join(f"`{f}`" for f in imp["functions"]))
            out.append("")
    return "\n".join(out) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("image", nargs="+")
    ap.add_argument("--game-dir")
    ap.add_argument("--out", default=os.path.join(os.path.dirname(__file__), "..", "reports", "pe"))
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    rc = 0
    for img in a.image:
        try:
            r = inspect(img, a.game_dir or os.path.dirname(os.path.abspath(img)))
        except pefile.PEFormatError as e:
            print(f"{img}: not a PE image ({e})", file=sys.stderr)
            rc = 1
            continue
        stem = os.path.join(a.out, os.path.splitext(r["file"])[0])
        with open(stem + ".json", "w", encoding="utf-8") as f:
            json.dump(r, f, indent=1)
        with open(stem + ".md", "w", encoding="utf-8") as f:
            f.write(to_markdown(r))
        enc = [s["name"] for s in r["sections"] if s["looks_encrypted"]]
        shim = sum(len(i["functions"]) for i in r["imports"] if i["where"] == "shim")
        print(f"{r['file']:<40} {r['machine']:<7} subsys={r['subsystem']!s:<7} pdata={r['pdata_functions']:<7} "
              f"shim-imports={shim:<5} encrypted={','.join(enc) or '-'}")
    return rc


if __name__ == "__main__":
    sys.exit(main())
