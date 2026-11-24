# Progress

Generated 2026-11-24 11:28 UTC from the task board by `tools/progress.py`. Do not edit by hand.

**13 of 117 tasks done** `██··················` · 0 in progress · 16 ready to take · 88 waiting on others · 10 pull requests merged by 1 contributors

![Tasks by area](images/progress.svg)

Legend: ✅ done · 🔨 in progress · 🟦 ready to take · ⏳ waiting on other tasks

## Phases

| Phase | What it means | Progress | Done |
|---|---|---|---:|
| 0. Foundation | the bot, the rules and the docs that let anyone join | `██████████` | 9/9 |
| 1. Shared knowledge | tests that define correct behaviour, checked on real hardware | `█·········` | 2/19 |
| 2. Adapters | running other projects' translators against those tests | `··········` | 0/6 |
| 3. Upstream fixes | fixing what the tests find, in the projects themselves | `··········` | 0/0 |
| 4. Shared components | pieces several projects need and nobody has yet | `██████████` | 1/1 |
| engine. Engine track | the hybrid recompiler itself, from loading a title to playing it (docs/engine-plan.md) | `··········` | 1/82 |

## Areas

| Area | Progress | Done | In progress | Ready | Waiting |
|---|---|---:|---:|---:|---:|
| shaders | `██········` | 2 | 0 | 8 | 0 |
| results | `··········` | 0 | 0 | 1 | 1 |
| system | `··········` | 0 | 0 | 2 | 11 |
| graphics | `··········` | 0 | 0 | 1 | 12 |
| audio | `··········` | 0 | 0 | 1 | 6 |
| storage | `··········` | 0 | 0 | 1 | 7 |
| input | `··········` | 0 | 0 | 1 | 3 |
| platforms | `··········` | 0 | 0 | 0 | 2 |
| infra | `██████····` | 8 | 0 | 0 | 6 |
| docs | `██████████` | 3 | 0 | 0 | 0 |
| cpu | `··········` | 0 | 0 | 0 | 6 |
| diagnostics | `··········` | 0 | 0 | 0 | 3 |
| difftest | `··········` | 0 | 0 | 0 | 4 |
| gpu | `··········` | 0 | 0 | 1 | 12 |
| hooking | `··········` | 0 | 0 | 0 | 4 |
| memory | `··········` | 0 | 0 | 0 | 3 |
| recomp | `··········` | 0 | 0 | 0 | 8 |

## Phase 0: Foundation

*The bot, the rules and the docs that let anyone join.*

| | Task | Area | Who |
|---|---|---|---|
| ✅ | [#7](https://github.com/scarlett-commons/scarlett-commons/issues/7) Bot: dry-run mode (comment only, never close or merge) | infra | @DeSunShine |
| ✅ | [#8](https://github.com/scarlett-commons/scarlett-commons/issues/8) Bot: propose new maintainers automatically | infra | — |
| ✅ | [#9](https://github.com/scarlett-commons/scarlett-commons/issues/9) Mirror the repository to a second host | infra | — |
| ✅ | [#11](https://github.com/scarlett-commons/scarlett-commons/issues/11) Gate: the Provenance section swallows the rest of the description | infra | @DeSunShine |
| ✅ | [#13](https://github.com/scarlett-commons/scarlett-commons/issues/13) Foundation docs: roadmap, architecture, getting started, translations, progress page | docs | @DeSunShine |
| ✅ | [#31](https://github.com/scarlett-commons/scarlett-commons/issues/31) Finish phase 0: protected rules, batch PR testing, visual progress, mirror, maintainer proposals, community files | infra | @DeSunShine |
| ✅ | [#38](https://github.com/scarlett-commons/scarlett-commons/issues/38) Presentation: README redesign, detailed progress page, badges, translations of key docs, reverse-engineering tools in the landscape | docs | @DeSunShine |
| ✅ | [#44](https://github.com/scarlett-commons/scarlett-commons/issues/44) Title runs by build hash, engine upstream sync, recompiler direction in the docs | infra | @DeSunShine |
| ✅ | [#46](https://github.com/scarlett-commons/scarlett-commons/issues/46) Engine CI: run the engine's translator, semantic, stdlib and runtime tests here | infra | @DeSunShine |

## Phase 1: Shared knowledge

*Tests that define correct behaviour, checked on real hardware.*

| | Task | Area | Who |
|---|---|---|---|
| 🟦 | [#5](https://github.com/scarlett-commons/scarlett-commons/issues/5) Wave64 cross-lane cases in the shared format | shaders | — |
| 🟦 | [#14](https://github.com/scarlett-commons/scarlett-commons/issues/14) VALU f32 rounding and denormal cases in the shared format | shaders | — |
| 🟦 | [#15](https://github.com/scarlett-commons/scarlett-commons/issues/15) f16 and packed math cases in the shared format | shaders | — |
| 🟦 | [#16](https://github.com/scarlett-commons/scarlett-commons/issues/16) LDS cases in the shared format | shaders | — |
| 🟦 | [#17](https://github.com/scarlett-commons/scarlett-commons/issues/17) Image and sampler cases in the shared format | shaders | — |
| 🟦 | [#18](https://github.com/scarlett-commons/scarlett-commons/issues/18) Buffer addressing cases in the shared format | shaders | — |
| 🟦 | [#19](https://github.com/scarlett-commons/scarlett-commons/issues/19) Reference RDNA2 interpreter for the shared cases | results | — |
| 🟦 | [#21](https://github.com/scarlett-commons/scarlett-commons/issues/21) Spec: virtual memory and page attributes as seen by titles | system | — |
| 🟦 | [#22](https://github.com/scarlett-commons/scarlett-commons/issues/22) Spec: threads, affinity and timing | system | — |
| 🟦 | [#23](https://github.com/scarlett-commons/scarlett-commons/issues/23) Console D3D12 vs desktop D3D12 and Vulkan: difference catalogue | graphics | — |
| 🟦 | [#24](https://github.com/scarlett-commons/scarlett-commons/issues/24) Spec: XAudio2 on console vs desktop | audio | — |
| 🟦 | [#25](https://github.com/scarlett-commons/scarlett-commons/issues/25) Spec: DirectStorage and GDeflate on console vs desktop | storage | — |
| 🟦 | [#26](https://github.com/scarlett-commons/scarlett-commons/issues/26) Spec: GameInput on console vs desktop | input | — |
| 🟦 | [#35](https://github.com/scarlett-commons/scarlett-commons/issues/35) Import Leonida's hand-written semantic tests as shared cases | shaders | — |
| ⏳ | [#20](https://github.com/scarlett-commons/scarlett-commons/issues/20) Hardware run: shared cases on an AMD RDNA2 GPU | results | — |
| ⏳ | [#39](https://github.com/scarlett-commons/scarlett-commons/issues/39) GFX10 tiling and swizzle cases from a compatible detiler | graphics | — |
| ⏳ | [#40](https://github.com/scarlett-commons/scarlett-commons/issues/40) BCPack decoding cases on self-generated data | graphics | — |
| ✅ | [#3](https://github.com/scarlett-commons/scarlett-commons/issues/3) Shared test format for RDNA2 shader semantics | shaders | @DeSunShine |
| ✅ | [#4](https://github.com/scarlett-commons/scarlett-commons/issues/4) Binary64 FMA cases in the shared format | shaders | — |

## Phase 2: Adapters

*Running other projects' translators against those tests.*

| | Task | Area | Who |
|---|---|---|---|
| 🟦 | [#6](https://github.com/scarlett-commons/scarlett-commons/issues/6) Runner: shared cases against a SPIR-V translator on Vulkan | shaders | — |
| ⏳ | [#27](https://github.com/scarlett-commons/scarlett-commons/issues/27) Runner on Linux and macOS (Vulkan, MoltenVK) | platforms | — |
| ⏳ | [#28](https://github.com/scarlett-commons/scarlett-commons/issues/28) Compatibility table: which translator passes which shared cases | infra | — |
| ⏳ | [#32](https://github.com/scarlett-commons/scarlett-commons/issues/32) Adapter: Leonida rdna2spirv (SPIR-V) | infra | — |
| ⏳ | [#33](https://github.com/scarlett-commons/scarlett-commons/issues/33) Adapter: VIctory shader recompiler (DXIL) | infra | — |
| ⏳ | [#34](https://github.com/scarlett-commons/scarlett-commons/issues/34) Adapter: Xbox2Windows (DXIL) and five-star-recomp (HLSL) | infra | — |

## Phase 4: Shared components

*Pieces several projects need and nobody has yet.*

| | Task | Area | Who |
|---|---|---|---|
| ✅ | [#42](https://github.com/scarlett-commons/scarlett-commons/issues/42) Import the recompiler engine from the Leonida continuation, with history | infra | @DeSunShine |

## Phase engine: Engine track

*The hybrid recompiler itself, from loading a title to playing it (docs/engine-plan.md).*

| | Task | Area | Who |
|---|---|---|---|
| 🟦 | [#130](https://github.com/scarlett-commons/scarlett-commons/issues/130) cs_imgload_tfe and cs_imgload_d16_tfe fail on llvmpipe | gpu | — |
| ⏳ | [#48](https://github.com/scarlett-commons/scarlett-commons/issues/48) Loader: read the load-config directory | cpu | — |
| ⏳ | [#49](https://github.com/scarlett-commons/scarlett-commons/issues/49) Crash handler that prints module+offset and a stack | diagnostics | — |
| ⏳ | [#50](https://github.com/scarlett-commons/scarlett-commons/issues/50) Local minidumps on crash, never uploaded | diagnostics | — |
| ⏳ | [#51](https://github.com/scarlett-commons/scarlett-commons/issues/51) Function map file (RVA, size, name, source) | diagnostics | — |
| ⏳ | [#52](https://github.com/scarlett-commons/scarlett-commons/issues/52) Emulate RDPID, RDPRU and SHA in the #UD handler | cpu | — |
| ⏳ | [#53](https://github.com/scarlett-commons/scarlett-commons/issues/53) TSC and QPC rate shim | cpu | — |
| ⏳ | [#54](https://github.com/scarlett-commons/scarlett-commons/issues/54) Physical page aliasing in xmem | memory | — |
| ⏳ | [#55](https://github.com/scarlett-commons/scarlett-commons/issues/55) Vendor Zydis as the runtime's x86-64 decoder | hooking | — |
| ⏳ | [#56](https://github.com/scarlett-commons/scarlett-commons/issues/56) Inline prologue hooks with trampolines | hooking | — |
| ⏳ | [#57](https://github.com/scarlett-commons/scarlett-commons/issues/57) Redirect indirect calls (vtables, function tables) | hooking | — |
| ⏳ | [#58](https://github.com/scarlett-commons/scarlett-commons/issues/58) Runtime function-swap registry | hooking | — |
| ⏳ | [#59](https://github.com/scarlett-commons/scarlett-commons/issues/59) Function database exporter (headless Ghidra) | recomp | — |
| ⏳ | [#60](https://github.com/scarlett-commons/scarlett-commons/issues/60) Cross-check function discovery with angr | recomp | — |
| ⏳ | [#61](https://github.com/scarlett-commons/scarlett-commons/issues/61) Name statically linked CRT and STL functions | recomp | — |
| ⏳ | [#62](https://github.com/scarlett-commons/scarlett-commons/issues/62) Fiber and thread audit | cpu | — |
| ⏳ | [#63](https://github.com/scarlett-commons/scarlett-commons/issues/63) Map the console's game cores onto host topology | cpu | — |
| ⏳ | [#64](https://github.com/scarlett-commons/scarlett-commons/issues/64) Lifter choice: remill vs RetDec vs Ghidra | recomp | — |
| ⏳ | [#65](https://github.com/scarlett-commons/scarlett-commons/issues/65) Lifting pipeline keeping the native Win64 ABI | recomp | — |
| ⏳ | [#66](https://github.com/scarlett-commons/scarlett-commons/issues/66) Calls out of lifted code | recomp | — |
| ⏳ | [#67](https://github.com/scarlett-commons/scarlett-commons/issues/67) Build lifted functions into a replacement DLL | recomp | — |
| ⏳ | [#68](https://github.com/scarlett-commons/scarlett-commons/issues/68) Capture inputs of a hooked function | difftest | — |
| ⏳ | [#69](https://github.com/scarlett-commons/scarlett-commons/issues/69) Native differential runner: original vs lifted | difftest | — |
| ⏳ | [#70](https://github.com/scarlett-commons/scarlett-commons/issues/70) Isolated replay of the original under Unicorn | difftest | — |
| ⏳ | [#71](https://github.com/scarlett-commons/scarlett-commons/issues/71) Shadow mode during real runs | difftest | — |
| ⏳ | [#72](https://github.com/scarlett-commons/scarlett-commons/issues/72) Per-function port progress census | recomp | — |
| ⏳ | [#73](https://github.com/scarlett-commons/scarlett-commons/issues/73) Import trace with arguments | system | — |
| ⏳ | [#74](https://github.com/scarlett-commons/scarlett-commons/issues/74) RDNA2 decoder in C++ for the runtime | gpu | — |
| ⏳ | [#75](https://github.com/scarlett-commons/scarlett-commons/issues/75) Shader translation on first use, with a disk cache | gpu | — |
| ⏳ | [#76](https://github.com/scarlett-commons/scarlett-commons/issues/76) A shared intermediate form inside the translator | gpu | — |
| ⏳ | [#77](https://github.com/scarlett-commons/scarlett-commons/issues/77) SPIR-V to DXIL output path | gpu | — |
| ⏳ | [#78](https://github.com/scarlett-commons/scarlett-commons/issues/78) Evaluate VIctory's shader recompiler as a second frontend | gpu | — |
| ⏳ | [#79](https://github.com/scarlett-commons/scarlett-commons/issues/79) Native wave64 where the GPU supports it | gpu | — |
| ⏳ | [#80](https://github.com/scarlett-commons/scarlett-commons/issues/80) Wave64 LDS and cross-lane code on wave32-only GPUs | gpu | — |
| ⏳ | [#81](https://github.com/scarlett-commons/scarlett-commons/issues/81) All 14 GFX10 2D swizzle modes | graphics | — |
| ⏳ | [#82](https://github.com/scarlett-commons/scarlett-commons/issues/82) Mip chains and mip tails | graphics | — |
| ⏳ | [#83](https://github.com/scarlett-commons/scarlett-commons/issues/83) 3D thick modes, arrays and cubes | graphics | — |
| ⏳ | [#84](https://github.com/scarlett-commons/scarlett-commons/issues/84) Detile on the GPU | graphics | — |
| ⏳ | [#85](https://github.com/scarlett-commons/scarlett-commons/issues/85) GPU DEFLATE decoding for DirectStorage | storage | — |
| ⏳ | [#86](https://github.com/scarlett-commons/scarlett-commons/issues/86) Upload only the title memory the CPU changed | memory | — |
| ⏳ | [#87](https://github.com/scarlett-commons/scarlett-commons/issues/87) GPU-to-CPU coherency for title memory | memory | — |
| ⏳ | [#88](https://github.com/scarlett-commons/scarlett-commons/issues/88) Track descriptors written at run time | gpu | — |
| ⏳ | [#89](https://github.com/scarlett-commons/scarlett-commons/issues/89) Inventory and add console-only D3D12.X entry points | graphics | — |
| ⏳ | [#90](https://github.com/scarlett-commons/scarlett-commons/issues/90) NGG merged stages (ES/GS, LS/HS) | gpu | — |
| ⏳ | [#91](https://github.com/scarlett-commons/scarlett-commons/issues/91) Emulate image_bvh_intersect_ray | gpu | — |
| ⏳ | [#92](https://github.com/scarlett-commons/scarlett-commons/issues/92) Console BVH to host acceleration structures | graphics | — |
| ⏳ | [#93](https://github.com/scarlett-commons/scarlett-commons/issues/93) HTILE and DCC layout | graphics | — |
| ⏳ | [#94](https://github.com/scarlett-commons/scarlett-commons/issues/94) Fill render-target metadata from host targets | graphics | — |
| ⏳ | [#95](https://github.com/scarlett-commons/scarlett-commons/issues/95) Frame pacing with present_wait | graphics | — |
| ⏳ | [#96](https://github.com/scarlett-commons/scarlett-commons/issues/96) HDR output and resolution scaling | graphics | — |
| ⏳ | [#97](https://github.com/scarlett-commons/scarlett-commons/issues/97) Record and replay pipelines with Fossilize | gpu | — |
| ⏳ | [#98](https://github.com/scarlett-commons/scarlett-commons/issues/98) Background pipeline compilation with stand-ins | gpu | — |
| ⏳ | [#99](https://github.com/scarlett-commons/scarlett-commons/issues/99) Parse MicrosoftGame.Config properly | system | — |
| ⏳ | [#100](https://github.com/scarlett-commons/scarlett-commons/issues/100) CI census of runtime classes and slots | system | — |
| ⏳ | [#101](https://github.com/scarlett-commons/scarlett-commons/issues/101) XPackage: remaining slots | system | — |
| ⏳ | [#102](https://github.com/scarlett-commons/scarlett-commons/issues/102) Remaining small runtime classes | system | — |
| ⏳ | [#103](https://github.com/scarlett-commons/scarlett-commons/issues/103) App-state (PLM) notifications | system | — |
| ⏳ | [#104](https://github.com/scarlett-commons/scarlett-commons/issues/104) Close, suspend, save, exit | system | — |
| ⏳ | [#105](https://github.com/scarlett-commons/scarlett-commons/issues/105) Own WerXboxGc and wer.dll, never upload | system | — |
| ⏳ | [#106](https://github.com/scarlett-commons/scarlett-commons/issues/106) Winsock offline guard | system | — |
| ⏳ | [#107](https://github.com/scarlett-commons/scarlett-commons/issues/107) Media Foundation resolver limited to local files | system | — |
| ⏳ | [#108](https://github.com/scarlett-commons/scarlett-commons/issues/108) Kernel pass-through census and fixes | system | — |
| ⏳ | [#109](https://github.com/scarlett-commons/scarlett-commons/issues/109) Linux: run under Wine/Proton in CI | platforms | — |
| ⏳ | [#110](https://github.com/scarlett-commons/scarlett-commons/issues/110) DirectStorage worker pool and priorities | storage | — |
| ⏳ | [#111](https://github.com/scarlett-commons/scarlett-commons/issues/111) BCPack stage in DirectStorage | storage | — |
| ⏳ | [#112](https://github.com/scarlett-commons/scarlett-commons/issues/112) GDeflate requests | storage | — |
| ⏳ | [#113](https://github.com/scarlett-commons/scarlett-commons/issues/113) Swizzled and physical-page destinations | storage | — |
| ⏳ | [#114](https://github.com/scarlett-commons/scarlett-commons/issues/114) Harden save storage | storage | — |
| ⏳ | [#115](https://github.com/scarlett-commons/scarlett-commons/issues/115) Console XAudio2 shim | audio | — |
| ⏳ | [#116](https://github.com/scarlett-commons/scarlett-commons/issues/116) XMA2 source voices | audio | — |
| ⏳ | [#117](https://github.com/scarlett-commons/scarlett-commons/issues/117) Opus decoding in xapu | audio | — |
| ⏳ | [#118](https://github.com/scarlett-commons/scarlett-commons/issues/118) AcpHal XMA contexts and WASAPI output | audio | — |
| ⏳ | [#119](https://github.com/scarlett-commons/scarlett-commons/issues/119) SHAPE / chad DSP: measure, then mix | audio | — |
| ⏳ | [#120](https://github.com/scarlett-commons/scarlett-commons/issues/120) Bink 2 at run time | audio | — |
| ⏳ | [#121](https://github.com/scarlett-commons/scarlett-commons/issues/121) GameInput keyboard and mouse | input | — |
| ⏳ | [#122](https://github.com/scarlett-commons/scarlett-commons/issues/122) XInput shim on SDL3 | input | — |
| ⏳ | [#123](https://github.com/scarlett-commons/scarlett-commons/issues/123) Input binding file | input | — |
| ⏳ | [#124](https://github.com/scarlett-commons/scarlett-commons/issues/124) Local run with an automatic, scrubbed report | infra | — |
| ⏳ | [#125](https://github.com/scarlett-commons/scarlett-commons/issues/125) Before/after comparison of local runs | infra | — |
| ⏳ | [#126](https://github.com/scarlett-commons/scarlett-commons/issues/126) Patch hot #UD sites into stubs | cpu | — |
| ⏳ | [#127](https://github.com/scarlett-commons/scarlett-commons/issues/127) Native, parallel BCPack decoding | storage | — |
| ✅ | [#128](https://github.com/scarlett-commons/scarlett-commons/issues/128) Engine plan document and the engine track on the progress page | docs | @DeSunShine |

## Test cases

| Group | Cases |
|---|---:|
| valu-f32 | 1 |
| valu-f64 | 2 |

**3 cases** in `cases/` (format: docs/test-format.md).

## Hardware results

No reports yet. See docs/results.md.

## Title runs

No reports yet. See docs/title-runs.md.

## Recently merged

| Date | Pull request | Author |
|---|---|---|
| 2026-11-24 | [#129](https://github.com/scarlett-commons/scarlett-commons/pull/129) Engine plan: the hybrid recompiler as 80 tasks, engine track on the progress page | @DeSunShine |
| 2026-11-24 | [#47](https://github.com/scarlett-commons/scarlett-commons/pull/47) Engine CI: run the engine's tests here | @DeSunShine |
| 2026-11-24 | [#45](https://github.com/scarlett-commons/scarlett-commons/pull/45) Title runs by build hash, engine upstream sync, recompiler in README and ROADMAP | @DeSunShine |
| 2026-11-24 | [#43](https://github.com/scarlett-commons/scarlett-commons/pull/43) Import the recompiler engine from the Leonida continuation, with history | @DeSunShine |
| 2026-11-24 | [#41](https://github.com/scarlett-commons/scarlett-commons/pull/41) Presentation: README redesign, detailed progress page, badges, translations, RE tools | @DeSunShine |
| 2026-11-24 | [#37](https://github.com/scarlett-commons/scarlett-commons/pull/37) Shared test format, validator and the first three cases | @DeSunShine |
| 2026-11-24 | [#36](https://github.com/scarlett-commons/scarlett-commons/pull/36) Finish phase 0: protected rules, batch PR testing, visual progress, mirror, community files | @DeSunShine |
| 2026-11-24 | [#29](https://github.com/scarlett-commons/scarlett-commons/pull/29) Foundation docs: roadmap, architecture, getting started, translations, progress page | @DeSunShine |
| 2026-11-24 | [#30](https://github.com/scarlett-commons/scarlett-commons/pull/30) Bot: dry-run mode (comment only) | @DeSunShine |
| 2026-11-24 | [#12](https://github.com/scarlett-commons/scarlett-commons/pull/12) Gate: a Provenance list ends at its last item line | @DeSunShine |

## Contributors

| Contributor | Merged pull requests |
|---|---:|
| @DeSunShine | 10 |

Take a ready task: see docs/getting-started.md.
