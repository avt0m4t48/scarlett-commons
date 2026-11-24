# Progress

Generated 2026-11-24 10:58 UTC from the task board by `tools/progress.py`. Do not edit by hand.

**11 of 34 tasks done** `██████··············` · 0 in progress · 15 ready to take · 8 waiting on others · 8 pull requests merged by 1 contributors

![Tasks by area](images/progress.svg)

Legend: ✅ done · 🔨 in progress · 🟦 ready to take · ⏳ waiting on other tasks

## Phases

| Phase | What it means | Progress | Done |
|---|---|---|---:|
| 0. Foundation | the bot, the rules and the docs that let anyone join | `██████████` | 8/8 |
| 1. Shared knowledge | tests that define correct behaviour, checked on real hardware | `█·········` | 2/19 |
| 2. Adapters | running other projects' translators against those tests | `··········` | 0/6 |
| 3. Upstream fixes | fixing what the tests find, in the projects themselves | `··········` | 0/0 |
| 4. Shared components | pieces several projects need and nobody has yet | `██████████` | 1/1 |

## Areas

| Area | Progress | Done | In progress | Ready | Waiting |
|---|---|---:|---:|---:|---:|
| shaders | `██········` | 2 | 0 | 8 | 0 |
| results | `··········` | 0 | 0 | 1 | 1 |
| system | `··········` | 0 | 0 | 2 | 0 |
| graphics | `··········` | 0 | 0 | 1 | 2 |
| audio | `··········` | 0 | 0 | 1 | 0 |
| storage | `··········` | 0 | 0 | 1 | 0 |
| input | `··········` | 0 | 0 | 1 | 0 |
| platforms | `··········` | 0 | 0 | 0 | 1 |
| infra | `██████····` | 7 | 0 | 0 | 4 |
| docs | `██████████` | 2 | 0 | 0 | 0 |

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
| @DeSunShine | 8 |

Take a ready task: see docs/getting-started.md.
