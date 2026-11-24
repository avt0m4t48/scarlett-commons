# Engine

The static recompiler and runtime: an RDNA2 → SPIR-V shader translator (`tools/rdna2spirv.py`), a PE loader and
console API shims with a D3D12 → Vulkan layer (`runtime/`), and their tests (`tests/`, `runtime/test/`).

## Where it comes from

Imported from [ubiquitarianism/leonida](https://github.com/ubiquitarianism/leonida) (GPL-3.0-or-later), the continuation of
the Leonida project, **with its full history and authorship**: `git log -- engine/` shows every original commit
and author. Issue and pull request numbers in that history (`#1234`) refer to the original repositories, not to
this one.

Imported: `tools/`, `runtime/`, `tests/`, `scripts/`, and the compatibly licensed `third_party/` code
(bcpack-rs: MIT or Apache-2.0; xtaskqueue: MIT, from microsoft/libHttpClient; xma: only the notice and licence text of an external Cargo dependency). Not imported: the title-specific build pipeline
(`make_game.py`, `game/`), mods, the viewer, and the `rpf9-explorer` submodule, which has no licence. Every version
of every imported file, across the whole history, passed this repository's game-data guard before it was
published here.

## Titles

The engine knows nothing it ships about any title's content: it runs a title's own code from the user's own copy
on disk. It was developed against GTA VI (Xbox Series build 1.0.2.1); the profile that builds a PC copy of that
title from your own files (file list and hashes, build stages, mods) is maintained in the Leonida project, not
here. Results on a title are reported by build hash (see [docs/title-runs.md](../docs/title-runs.md)).

## Building and testing

See `runtime/README.md` and the test scripts under `scripts/`. The engine's own test suites need their original
toolchain (CMake, a C++20 compiler, the Vulkan SDK, Python 3.10+ with pytest for `tests/`); they are not yet part
of this repository's CI.
