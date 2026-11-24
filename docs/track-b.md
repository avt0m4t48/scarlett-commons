# Track B: recompiling the title's functions

Track A (the VIctory runtime in `victory/`) runs the title's own x86-64 code natively. Track B replaces functions of
that code with recompiled versions, each proven equal to the original before it is used. Both run in one process:
`cargo commons run <exe> --execute` (from `victory/`) maps the title, installs the build's track B functions
(`recomp/`), and runs. What track B does not have stays native.

## Generated on your machine, never stored here

A recompiled function is a translation of the title's code, so it is not kept in this repository. Track B is built
the way other static recompilers build theirs: on each player's machine, from their own copy, by tools that are
here. The repository holds the translator, the selection rules, the differential tester and their tests on
synthetic code; the generated code, the build of it and the local manifest stay in your work folder.

```
commons recomp <exe> --work DIR        (planned; the pieces below exist or are tasks)
  1. find every function: the executable's .pdata table gives each one's start and end
  2. keep what the translator covers fully (today: integer code without calls, then wider)
  3. translate each kept function and build them into one library in DIR
  4. differential test every one: the original and the translation on generated inputs (pure functions),
     later on inputs recorded from real play; only passing functions go on
  5. write DIR's manifest (rva, prologue bytes, status verified) for this build
  6. cargo commons run picks the manifest up and installs those functions over the native ones
```

Contributors do not hand-write functions. They widen the translator (an instruction, a calling pattern, memory
access) and every function that was waiting for it joins track B on the next local build. A change is tested on
synthetic code here; its effect on a title shows as a higher count in title-run reports.

## Where it stands

* The install path works and is tested: `recomp/` reads a manifest, checks each function's native bytes and writes a
  jump to the replacement (`cargo commons run` reports how many it installed).
* The census (`engine/tools/recomp_census.py`, reads the executable and never runs it) measures what the translator
  covers. On 1.0.2.1 .pdata lists 299,257 functions (92 MB of code, 20.3 million instructions); the integer subset
  covers 85.4% of the instructions and 151,107 functions (50.5%) completely. That is track B's starting share once
  the pieces below exist.
* The translator emits arm64 today (the groundwork for other CPUs, `engine/tools/x64_recomp.py`). Track B on a PC
  needs an emitter for code that runs here (Rust source, built locally into a library) and the thunk that enters it
  from the native calling convention. Those, the local build and the differential tester are the next tasks: #156
  Rust census, #157 translation to Rust source in the work folder, #158 the thunk, #159 the differential tester,
  #160 loading a locally built library, #161 the one command (`commons recomp`).

## Steps for a function that the automatic path cannot take yet

| Step | What | Task |
|---|---|---|
| Find functions, names | `.pdata` boundaries; names for statically linked CRT and STL | #59, #61 |
| Choose | Crash and stop sites, hottest functions, mod points | #135 |
| Record inputs | Calls under track A: arguments, memory read, results and writes | #68 |
| Translate | Layer 3 by hand from a decompilation, or layer 4 lifting (remill) | #134, #64, #65 |
| Prove | Differential runner on recorded inputs | #69 |
| Count | Per-build census of verified functions on the progress page | #72 |

The hand-translated ones follow the same rule as the generated ones: the translation is produced from the player's
copy by a script in this repository, not committed as code.
