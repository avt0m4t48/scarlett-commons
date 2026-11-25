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

## Where it stands (1.0.2.1, 2026-11-25)

Track B runs in the title. On the reporter's copy, `commons-recomp` built 908 functions from the first 3000
candidates, each equal to the native function on 200 generated inputs, and `cargo commons run` installed them over
the native code: the title ran as before (30 fps on the first screen) with `track B 908 function(s) installed`.

What each piece does today:

| Piece | Where | State |
|---|---|---|
| Census of a build's functions | `commons-census` (#156) | 299,257 functions, 1.3 s for the whole executable |
| Emitter: x86-64 to Rust | `recomp/emit` (#157, #169, #171) | integer core, calls, imul, the common AVX; 66% of functions fully covered |
| Entry from native code | `commons_entry`, `e_<name>` stubs (#158) | side stack per thread, nested entries |
| Differential test | `pipeline.rs` (#159) | image mapped at its base, pointers into a guarded sandbox, faults and endless loops caught, writes to globals compared and undone through a write watch, callees run natively when they are safe |
| Local build and load | `commons-recomp`, `COMMONS_RECOMP_DIR` (#160, #161) | library and manifest in the work folder, installed by `cargo commons run` |

What keeps the rest out, in order of size:

* **Inconclusive functions** (about half of those selected): on most generated inputs the native function faults,
  so there is too little to compare. Inputs recorded from real play (#68) would settle them.
* **Coverage**: shuffles, horizontal operations, compares, FMA, atomics and the rest (#174 to #179).
* **Indirect calls and imports**: a callee reached through a pointer or the import table is not run in the test yet.

Building it on your own copy:

```powershell
cd recomp
cargo run --release --bin commons-recomp -- <exe> --work <folder outside the repository>
cd ..ictory
$env:COMMONS_RECOMP_DIR = '<that folder>'
cargo commons run <exe> --execute
```

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
