"""Which sources of a runtime/test program are newer than a built copy of it.

make_game.py (vk_pipeline_check) and first_light.py (vk_harness) take a program already in runtime/build when there
is one. After a pull that changed the program, that copy is from the old sources and would run silently. The
sources are runtime/test/<name>.cpp and the headers it pulls in with #include "..." (followed through other
headers): a change to any of them recompiles and relinks the program, so a current build is never older than them.
runtime/CMakeLists.txt isn't one: a change there that doesn't concern the program leaves it as it was.
"""
import os
import re

_INCLUDE = re.compile(r'^\s*#\s*include\s*"([^"]+)"', re.MULTILINE)


def program_sources(root, name):
    """The files program `name` of runtime/test is built from, as far as they exist in this checkout."""
    runtime = os.path.join(root, "runtime")
    pending = [os.path.join(runtime, "test", name + ".cpp")]
    found = []
    seen = set()
    while pending:
        path = os.path.normpath(pending.pop())
        if path in seen or not os.path.isfile(path):
            continue
        seen.add(path)
        found.append(path)
        try:
            with open(path, encoding="utf-8", errors="replace") as f:
                text = f.read()
        except OSError:
            continue
        pending.extend(os.path.join(os.path.dirname(path), inc) for inc in _INCLUDE.findall(text))
    return found


def newer_sources(exe, root, name):
    """The sources of program `name` changed after `exe` was built, relative to `root`; empty when `exe` is current
    or isn't there to compare."""
    try:
        built = os.path.getmtime(exe)
    except OSError:
        return []
    newer = []
    for path in program_sources(root, name):
        try:
            if os.path.getmtime(path) > built:
                newer.append(os.path.relpath(path, root).replace(os.sep, "/"))
        except OSError:
            pass
    return newer
