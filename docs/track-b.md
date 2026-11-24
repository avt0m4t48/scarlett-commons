# Track B: recompiling the title's functions

Track A (the VIctory runtime in `victory/`) runs the title's own x86-64 code natively. Track B replaces chosen
functions of that code with our own recompiled versions, one function at a time, each proven equal to the original
before it is switched on. Both run in one process: `cargo commons run <exe> --execute` (from `victory/`) maps the
title, installs the build's track B functions (`recomp/`), and runs. What a manifest does not list stays native.

Why one function at a time: the code already runs on a PC, so recompiling all of it gains nothing by itself. A
replaced function is worth having when it fixes a crash or a stop, is hot and can be made faster, is a point mods
need, or brings a port to another CPU (the arm64 groundwork in `engine/tools/x64_recomp.py`) closer.

## The steps for one function

| Step | What | Where | Task |
|---|---|---|---|
| 1. Find functions | Function boundaries and names for the build, exported once (headless Ghidra), cross-checked | `recomp` tools | #59, #60, #61 |
| 2. Choose | Crash and stop sites from title-run reports, the hottest functions, mod points | title-run reports | #135 |
| 3. Record inputs | Calls to the function under track A: arguments, the memory it reads, what it returns and writes | runtime hook | #68 |
| 4. Translate | Decompile to C or Rust and fix it up by hand (layer 3), or lift it automatically (layer 4, remill) behind one register-state wrapper | `recomp/src/` | #134, #64, #65 |
| 5. Prove | The differential runner calls the original and ours on every recorded input and compares results and memory effects | `recomp` tests | #69 |
| 6. Switch on | Add it to `REGISTRY` and to `recomp/manifests/<build>.toml` as `testing`, run the title with `COMMONS_ONLY=<name>`, then mark it `verified` | `recomp/` | #72 |

A function is `verified` only when step 5 passes on recorded inputs from at least one real run; `testing` until then.
The manifest's prologue bytes keep a function from being installed into a build it was not made for.

## What is kept in the repository

Addresses, byte signatures (the prologue), our names for functions and our own code. Not the title's code: a
translated function is our implementation of the same behaviour, written or lifted here and tested against the
original on the reporter's machine. Recorded inputs stay on that machine too; a test carries only synthetic inputs.

## Progress

The title-run report says how many functions track B installed; the progress page counts verified functions per
build (#72). Today: the entry point and the install path work and are tested; no function is recompiled yet.
