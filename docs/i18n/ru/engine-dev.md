[English](../../engine-dev.md) · **Русский** · [简体中文](../zh-CN/engine-dev.md) · [Español](../es/engine-dev.md)

> Перевод для удобства; обязательны английские оригиналы. Если они расходятся, верен английский.

# Работа над движком

Как собирать и тестировать `engine/` локально и что нужно pull request'у к движку. План, по которому идёт работа,
описан в [engine-plan.md](../../engine-plan.md).

## Что где лежит

| Путь | Что это | Тесты |
|---|---|---|
| `engine/tools/rdna2spirv.py` | Транслятор шейдеров RDNA2 → SPIR-V (Python) | `engine/tests/` (pytest и стандартная библиотека), `engine/tests/semantic/` |
| `engine/runtime/src/` | Загрузчик (`pe_loader`), хост TLS, эмуляция процессора, точка входа `main.cpp` | `engine/runtime/test/` (CTest) |
| `engine/runtime/shims/` | Замена API консоли: ядро, классы среды выполнения, D3D12.X, D3D12 → Vulkan, хранилище, ввод, звук, отчёт о падении | `engine/runtime/test/` |
| `engine/runtime/test/shaders/` | Написанные вручную тестовые данные для GPU на RDNA2 | `engine/scripts/run_gpu_tests.py` |
| `engine/scripts/` | Запускатели тестов и помощники сборки | — |
| `engine/third_party/` | Вложенный код под совместимыми лицензиями (см. engine/README.md) | — |

## Инструменты

| Что нужно | Для чего | Примечания |
|---|---|---|
| Python 3.10+ (в CI используется 3.12) | транслятор, инструменты, тесты | `pip install -r engine/tests/requirements.txt` для файлов pytest |
| `llvm-mc` из LLVM 19+ с целью AMDGPU | тесты транслятора, собирающие RDNA2 | укажите путь в `LLVM_MC` |
| SPIRV-Tools (`spirv-as`, `spirv-val`) | тесты транслятора и семантические тесты | входят в Vulkan SDK |
| XML ISA AMD RDNA2 (`amdgpu_isa_rdna2.xml`, MIT, из машиночитаемого ISA AMD GPUOpen) | таблицы декодера в некоторых тестах | укажите в `LEONIDA_RDNA2_ISA_XML` |
| Windows: Visual Studio 2022 Build Tools (C++), CMake, Ninja | среда выполнения | на Linux и macOS собираются кроссплатформенные части; см. engine/runtime/README.md |
| Vulkan SDK 1.4 | устройство Vulkan, тесты GPU | укажите в `VULKAN_SDK` |

## Сборка и тесты

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

Результат сборки (`engine/runtime/build/`) игнорируется git; никогда не коммитьте его (бот закрывает pull
request'ы с двоичными файлами).

## Что запускает CI

| Рабочий процесс | Задания | Когда |
|---|---|---|
| `engine-translator` | тесты транслятора (pytest + llvm-mc), семантические тесты, тесты стандартной библиотеки | изменения в `engine/` |
| `engine-runtime` | сборка и CTest на Windows; сборка, CTest и тесты GPU на llvmpipe на Linux; сборка, CTest и тесты GPU на MoltenVK на macOS | изменения в `engine/runtime`, `engine/scripts`, `engine/third_party` и в файлах транслятора, которые он использует |

Известные красные, заведены как задачи: фикстуры GPU для загрузки изображений TFE на llvmpipe (#130) и
нестабильность из-за гранулярности таймера в `crt_timed_wait` на хостируемом Windows (#132). Тесты стандартной
библиотеки движка, которым нужны части, не импортированные сюда (конвейер сборки игры и его документация),
перечислены в `.github/engine-known-gaps.txt`; задание падает только из-за сбоя вне этого списка, а если
перечисленный тест начинает проходить, об этом сообщается.

## Что нужно pull request'у к движку

- Те же правила, что и для любого pull request (CONTRIBUTING.md): заявленная задача, `Provenance:`, коммиты с
  подписью.
- Тест рядом с изменением: `engine/tests/` для транслятора и инструментов, `engine/runtime/test/`
  (зарегистрированный в `engine/runtime/CMakeLists.txt` через `add_test`) для среды выполнения. Бот просит тест,
  если код движка изменился без него.
- Зелёный CI движка для тех заданий, которые затрагивает ваше изменение, либо пометка, объясняющая известное
  красное задание.
- Ничего из какой-либо игры в тестах: фикстуры пишутся вручную; результаты на реальной игре идут через
  [title-runs.md](./title-runs.md).

## Проверка изменения на игре

`tools/local_run.py` собирает вашу копию с движком этого репозитория и пишет очищенный отчёт
([title-runs.md](./title-runs.md)). При падении среда выполнения печатает `[rt] crash: <code> at <module>+0x<offset>`
и записывает то же в виде JSON в `LEONIDA_CRASH_FILE`.
