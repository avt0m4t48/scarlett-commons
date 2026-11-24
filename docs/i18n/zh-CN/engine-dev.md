[English](../../engine-dev.md) · [Русский](../ru/engine-dev.md) · **简体中文** · [Español](../es/engine-dev.md)

> 译文仅供参考，具有约束力的是英文原文；如有出入，以英文为准。

# 引擎开发

如何在本地构建和测试 `engine/`，以及引擎拉取请求需要什么。这项工作所遵循的计划见
[engine-plan.md](../../engine-plan.md)。

## 什么在哪里

| 路径 | 是什么 | 测试 |
|---|---|---|
| `engine/tools/rdna2spirv.py` | RDNA2 → SPIR-V 着色器翻译器（Python） | `engine/tests/`（pytest 和标准库）、`engine/tests/semantic/` |
| `engine/runtime/src/` | 加载器（`pe_loader`）、TLS 宿主、CPU 模拟、入口点 `main.cpp` | `engine/runtime/test/`（CTest） |
| `engine/runtime/shims/` | 主机 API 替代实现：内核、运行时类、D3D12.X、D3D12 → Vulkan、存储、输入、音频、崩溃报告 | `engine/runtime/test/` |
| `engine/runtime/test/shaders/` | 手写的 RDNA2 GPU 测试夹具 | `engine/scripts/run_gpu_tests.py` |
| `engine/scripts/` | 测试运行器和构建辅助脚本 | — |
| `engine/third_party/` | 许可证兼容的引入代码（见 engine/README.md） | — |

## 工具链

| 需要 | 用于 | 说明 |
|---|---|---|
| Python 3.10+（CI 使用 3.12） | 翻译器、工具、测试 | pytest 文件需要 `pip install -r engine/tests/requirements.txt` |
| 带 AMDGPU 目标的 LLVM 19+ 的 `llvm-mc` | 需要汇编 RDNA2 的翻译器测试 | 用 `LLVM_MC` 指向它的路径 |
| SPIRV-Tools（`spirv-as`、`spirv-val`） | 翻译器和语义测试 | Vulkan SDK 自带 |
| AMD RDNA2 ISA XML（`amdgpu_isa_rdna2.xml`，MIT，来自 AMD GPUOpen 的机器可读 ISA） | 部分测试中的解码表 | 设置 `LEONIDA_RDNA2_ISA_XML` |
| Windows：Visual Studio 2022 Build Tools（C++）、CMake、Ninja | 运行时 | Linux 和 macOS 构建跨平台部分；见 engine/runtime/README.md |
| Vulkan SDK 1.4 | Vulkan 设备、GPU 测试 | 设置 `VULKAN_SDK` |

## 构建与测试

```
# translator (from engine/)
cd engine
python -I -m pytest tests -q                       # the pytest files
python -I scripts/run_semantic_tests.py tests/semantic -v

# runtime (from engine/, in a Visual Studio x64 developer prompt on Windows)
cmake -S runtime -B runtime/build -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo
cmake --build runtime/build
ctest --test-dir runtime/build --output-on-failure

# hand-written GPU tests on your GPU (or a software Vulkan driver such as llvmpipe)
python -I scripts/run_gpu_tests.py --build runtime/build --llvm-mc "$LLVM_MC" --work <tmp folder>
```

构建产物（`engine/runtime/build/`）被 git 忽略；绝不要提交它（机器人会关闭带有二进制文件的拉取请求）。

## CI 运行什么

| 工作流 | 任务 | 触发条件 |
|---|---|---|
| `engine-translator` | 翻译器测试（pytest + llvm-mc）、语义测试、标准库测试 | `engine/` 下的变更 |
| `engine-runtime` | Windows 构建 + CTest；Linux 构建、CTest 以及在 llvmpipe 上的 GPU 测试；macOS 构建、CTest 以及在 MoltenVK 上的 GPU 测试 | `engine/runtime`、`engine/scripts`、`engine/third_party` 及其用到的翻译器文件下的变更 |

已知失败，已作为任务跟踪：llvmpipe 上的 TFE 图像加载 GPU 测试夹具（#130），以及托管 Windows 上 `crt_timed_wait` 中
与计时器粒度相关的不稳定失败（#132）。需要此处未导入部分（游戏构建流水线及其文档）的引擎标准库测试，列在
`.github/engine-known-gaps.txt` 中；只有该列表之外的失败才会让任务失败，而列表中的测试一旦开始通过，就会被报告。

## 引擎拉取请求需要什么

- 与任何拉取请求相同的规则（[CONTRIBUTING.md](CONTRIBUTING.md)）：一个已认领的任务、`Provenance:`、签署了的提交。
- 变更旁边要有测试：翻译器和工具用 `engine/tests/`，运行时用 `engine/runtime/test/`（在
  `engine/runtime/CMakeLists.txt` 中用 `add_test` 注册）。当引擎代码变更而没有测试时，机器人会要求补上。
- 你的变更涉及的任务，引擎 CI 要为绿，或者对已知失败的任务附上说明。
- 测试中不得出现任何游戏的内容：夹具都是手写的；在真实游戏上的结果走
  [title-runs.md](title-runs.md)。

## 在游戏上试验你的变更

`tools/local_run.py` 用本仓库的引擎构建你自己的副本，并写出一份经过清理的报告
（[title-runs.md](title-runs.md)）。运行时崩溃时会打印 `[rt] crash: <code> at <module>+0x<offset>`，
并把同样的内容以 JSON 写入 `LEONIDA_CRASH_FILE`。
