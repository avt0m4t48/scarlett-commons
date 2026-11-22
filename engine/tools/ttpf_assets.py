"""Complete the rpf9dump texture-pool manifest during the assets build."""
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import zlib

# `python -I` implies `-P`, so the script's directory is not on sys.path; the sibling modules are next to it.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import bcpack
from file_mode import published_mode
import ttpf_mips as ttpf


def load_profile(work):
    path = Path(work, "bcpack-calibration.json")
    try:
        raw = path.read_bytes()
    except FileNotFoundError:
        return "v1-strict", None
    profile = json.loads(raw)
    if not isinstance(profile, dict) or profile.get("version") != 1:
        raise ValueError(f"Unsupported calibration version in {path}")
    try:
        from calibrated_texture import native
        native()
    except (OSError, ImportError) as exc:
        raise RuntimeError("Calibrated texture export requires NumPy and the release build of "
                           "tools/bcpack_calibration (see its README.md)") from exc
    return "v1-" + hashlib.sha256(raw).hexdigest(), profile


def publish(path, data):
    f = tempfile.NamedTemporaryFile(dir=path.parent, delete=False)
    tmp = Path(f.name)
    try:
        with f:
            f.write(data)
        if os.name != "nt":
            tmp.chmod(published_mode(path))
        os.replace(tmp, path)
    finally:
        tmp.unlink(missing_ok=True)


def file_identity(path):
    """(device, inode) of an existing file, or None: two names with one identity are one file on disk."""
    try:
        st = os.stat(path)
    except OSError:
        return None
    return (st.st_dev, st.st_ino) if st.st_ino else None


def remove_replaced(directory, replaced, rows):
    """Remove the files of `replaced` (names the rows had before they were switched) from `directory`; returns
    how many could not be removed.

    Only a .dds file by its bare name in this folder is removed, never manifest.json, and never a file a row still
    names: by the same name, by a name that differs in letter case only, or by any other spelling of the same file
    on disk (on Windows and macOS that can be the file the pass has just published). Each file is looked at once:
    one stat per named row and one per replaced name, not one per pair."""
    names = [row["File"] for row in rows if isinstance(row.get("File"), str)]
    named = {os.path.normcase(name) for name in names}
    on_disk = {file_identity(directory / name) for name in names} - {None}
    left = 0
    for name in sorted(replaced):
        if Path(name).name != name or not name.lower().endswith(".dds") or os.path.normcase(name) in named:
            continue
        if file_identity(directory / name) in on_disk:
            continue
        try:
            (directory / name).unlink(missing_ok=True)
        except OSError:
            left += 1
    return left


def candidate(row, profile):
    if row.get("DxgiFormat") not in ttpf.BC_FORMATS:
        return False
    if row.get("BCPackDecoder"):
        return profile is not None or row["BCPackDecoder"].endswith("-strict")
    return (row["File"] is None or row.get("BaseMipLevel", 0) > 0
            or row.get("Mips", 0) < row.get("SourceMips", 0))


def complete_pool(source, directory, manifest, settings):
    """Publish full chains next to ordinary DDS files, then switch the catalog to them."""
    stamp, profile = settings
    if manifest.get("BCPackPass") == stamp:
        return 0, 0
    rows = [row for row in manifest["Textures"] if candidate(row, profile)]
    if not rows:
        return 0, 0
    directory = Path(directory)
    written = blocked = 0
    superseded = set()   # the files the rows named before they were switched to a full chain
    with closing(ttpf.Pool(source)) as pool:
        decoder = None
        if profile is not None:
            from calibrated_texture import LocalDecoder
            decoder = LocalDecoder(pool, profile)
        try:
            for row in rows:
                index = row["Index"]
                desc = pool.descriptor(index)
                base = desc["base"]
                if not pool.chunks or base < pool.chunks[0][0] or base >= pool.virtual_size:
                    continue
                span = pool.span(index, base)
                if not span:
                    continue
                try:
                    if not pool.codecs(base, span) & bcpack.SUPPORTED_REBUILD_CODECS:
                        continue
                    if decoder and desc["dimension"] == 1 and desc["format"] in decoder.formats:
                        method = stamp
                        blob = decoder.dds(index)
                    else:
                        method = "v1-strict"
                        analysis = ttpf.analyze(pool, index)
                        blob = ttpf.texture_dds(pool, analysis)
                except (ValueError, KeyError, IndexError, ttpf.BcpackRequired,
                        bcpack.BcpackUnsupportedCodec, zlib.error) as exc:
                    row["BCPackLimitation"] = str(exc)
                    blocked += 1
                    continue
                # A distinct name keeps an existing lower-mip DDS usable until the manifest switches.
                name = f"{index:06d}_bcpack_{method[:19]}.dds"
                publish(directory / name, blob)
                if isinstance(row.get("File"), str) and row["File"] != name:
                    superseded.add(row["File"])
                row.update(File=name, Mips=desc["mips"], BaseMipLevel=0, Limitation=None, BCPackDecoder=method)
                row.pop("BCPackLimitation", None)
                written += 1
        finally:
            if decoder:
                decoder.read_chunk.cache_clear()
    manifest["BCPackPass"] = stamp
    publish(directory / "manifest.json", json.dumps(manifest, indent=2).encode("utf-8"))
    # Only now that the manifest names the full chains: the files they replace would otherwise stay in the folder,
    # named by no row, so every completed texture would be stored twice. A file another row still names is kept,
    # and so is one that can't be removed (in use): the export is complete either way.
    left = remove_replaced(directory, superseded, manifest["Textures"]) if superseded else 0
    if left:
        print(f"assets: {directory.name}: {left} replaced DDS file(s) could not be removed (in use?) and stay next "
              "to the full-chain files", file=sys.stderr)
    return written, blocked
