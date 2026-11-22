"""Write a one-line Windows batch file (a fake tool for a test) that cmd.exe can run from any user folder.

cmd.exe decodes a batch file with the console's OEM code page, not UTF-8, so a path with non-ASCII letters (a
user folder named in Cyrillic, say) written as UTF-8 becomes "The system cannot find the path specified".
Standard library only.
"""


def batch_path(path):
    """A spelling of path that cmd.exe reads correctly from a batch file: as is when ASCII, else its 8.3 short name
    (ASCII). Falls back to the long path where the volume has 8.3 names turned off."""
    if path.isascii():
        return path
    import ctypes
    buf = ctypes.create_unicode_buffer(32768)
    n = ctypes.windll.kernel32.GetShortPathNameW(path, buf, len(buf))
    return buf.value if 0 < n < len(buf) else path


def write_python_cmd(cmd_path, python, script):
    """Write cmd_path as a batch file that runs `python script` with the batch file's arguments."""
    # "oem" for a path that keeps non-ASCII letters (no short name): the code page cmd.exe reads it with.
    with open(cmd_path, "w", encoding="oem") as f:
        f.write(f'@"{batch_path(python)}" "{batch_path(script)}" %*\n')
    return cmd_path
