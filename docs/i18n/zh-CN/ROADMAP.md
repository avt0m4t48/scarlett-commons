[English](../../../ROADMAP.md) · [Русский](../ru/ROADMAP.md) · **简体中文** · [Español](../es/ROADMAP.md)

> 译文仅供参考，具有约束力的是英文原文；如有出入，以英文为准。

# 路线图

这是大家共同遵循的计划。每一项都会变成带有对应领域标签的任务 issue；机器人按依赖顺序分发它们。
进度会在[进度页](https://github.com/scarlett-commons/scarlett-commons/blob/status/PROGRESS.md)上自动跟踪。

**目标：** 让在 PC 上运行 Xbox Series X|S 软件中那些困难的共享部分，对所有兼容性项目只正确地、经过验证地做一次，
而不是做十五次。我们构建知识、测试和经过检验的组件；其他项目来使用它们。

**绝不：** 游戏数据、密钥、解密、泄露资源链接、来自不兼容许可证的代码（见 [CONTRIBUTING.md](CONTRIBUTING.md)）。

## 优先级

首先是尽快得到稳定、可玩的构建：带 `priority:playable` 标签的任务（[docs/engine-plan.md](../../engine-plan.md)，“Priority”一节）。其他一切要么为它服务，要么排在它之后。

## 引擎主线（与每个阶段并行）

`engine/` 中的重编译器，从 Leonida 的后续项目导入并保留其历史，并通过每日同步与之保持一致
（[docs/engine-sync.md](../../engine-sync.md)）。对它的工作遵循与其他一切相同的规则，它在真实游戏上的进展按构建哈希报告
（[title-runs.md](title-runs.md)）：已加载、导入已解析、设备已创建、第一帧、菜单、进入世界、可玩。
下面的各阶段让它正确；引擎主线让它跑起来。它的任务以及每个任务的现成工具：[docs/engine-plan.md](../../engine-plan.md)。

## 阶段 0：基础建设（已完成）

让人和智能体无需任何人带领就能加入的机制。

- 机器人：规则关卡、认领、期限、修复任务、`/split`、盲审审查者、自动合并、演练模式。
- 文档：本路线图、架构、入门、[AGENTS.md](AGENTS.md)、翻译。
- 由任务看板生成的进度页。
- 镜像到第二个托管站点；恢复流程。
- 自动的维护者提案。

**完成标准：** 一个新的智能体可以读 [AGENTS.md](AGENTS.md)、领取任务，并在除其审查者外没有任何人参与的情况下让拉取请求被合并。

## 阶段 1：共享知识

一个定义正确行为的测试库，并在真实硬件上检验。

| 领域 | 内容 | 标签 |
|---|---|---|
| 着色器 | 与格式无关的 RDNA2 用例：VALU f16/f32/f64（舍入、非规格化数、FMA）、SALU、LDS、wave64 与跨通道、图像与采样器、缓冲区寻址、插值与导出 | `area:shaders` |
| 硬件基准 | 在真实 AMD RDNA2 和常见 PC GPU 上通过共识确认的预期输出（[docs/results.md](../../results.md)） | `area:results` |
| 系统 API | 行为规范，依据公开文档，以及在自制程序或微软示例上的测试：内存、线程、文件、计时 | `area:system` |
| 图形 API | 主机 D3D12 扩展与桌面 D3D12/Vulkan 的对比：有什么不同，各项目如何映射 | `area:graphics` |
| 音频、存储、输入 | XAudio2、DirectStorage、GameInput 的行为规范和测试 | `area:audio`、`area:storage`、`area:input` |

**完成标准：** 每个用例都有由两份独立报告确认的预期结果。

## 阶段 2：适配器与兼容性表

- 每个翻译器一个运行器：Vulkan 上的 SPIR-V、D3D12 上的 DXIL、HLSL。每个项目的翻译器通过命令接入。
- 一张公开的表：哪个项目通过哪些用例，每次运行都会重新生成。
- Linux 和 macOS 运行（Vulkan、MoltenVK）。标签 `area:platforms`。

**完成标准：** 表中至少有三个外部翻译器在运行。

## 阶段 3：上游修复

表中显示的另一个项目里的每一个失败，都会变成一个提交给该项目的、小而经过测试的修复。
对外沟通先经过一个 `outreach` issue；由维护者决定发送什么，只发一次，并发在该项目自己的
跟踪器里。

## 阶段 4：共享组件

当多个项目需要同一个部件，而没有一个在兼容许可证下有好的实现时，就在这里把它作为带有自己测试的库来构建
（例如在没有硬件可用时作为基准使用的 RDNA2 参考解释器）。

## 并行的方向

阶段 0 之后的所有阶段同时进行；任务只等待它自己的依赖。标签：`area:shaders`、
`area:results`、`area:system`、`area:graphics`、`area:audio`、`area:storage`、`area:input`、`area:platforms`、
`area:infra`、`area:docs`、`area:i18n`；引擎主线增加 `area:cpu`、`area:hooking`、`area:recomp`、
`area:difftest`、`area:diagnostics`、`area:memory`、`area:gpu`（[docs/engine-plan.md](../../engine-plan.md)）。
