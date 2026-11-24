[English](../../engine-dev.md) · [Русский](../ru/engine-dev.md) · [简体中文](../zh-CN/engine-dev.md) · **Español**

> Traducción de cortesía; son vinculantes los originales en inglés. Si difieren, vale el inglés.

# Trabajar en el motor

Cómo compilar y probar `engine/` en local, y qué necesita un pull request del motor. El plan que sigue el trabajo
está en [engine-plan.md](../../engine-plan.md).

## Qué hay dónde

| Ruta | Qué es | Pruebas |
|---|---|---|
| `engine/tools/rdna2spirv.py` | Traductor de shaders RDNA2 → SPIR-V (Python) | `engine/tests/` (pytest y stdlib), `engine/tests/semantic/` |
| `engine/runtime/src/` | Cargador (`pe_loader`), host de TLS, emulación de CPU, punto de entrada `main.cpp` | `engine/runtime/test/` (CTest) |
| `engine/runtime/shims/` | Reemplazos de la API de la consola: kernel, clases de runtime, D3D12.X, D3D12 → Vulkan, almacenamiento, entrada, audio, informe de fallos | `engine/runtime/test/` |
| `engine/runtime/test/shaders/` | Fixtures de GPU RDNA2 escritos a mano | `engine/scripts/run_gpu_tests.py` |
| `engine/scripts/` | Ejecutores de pruebas y ayudantes de compilación | — |
| `engine/third_party/` | Código incorporado con licencias compatibles (ver engine/README.md) | — |

## Cadena de herramientas

| Necesitas | Para | Notas |
|---|---|---|
| Python 3.10+ (CI usa 3.12) | traductor, herramientas, pruebas | `pip install -r engine/tests/requirements.txt` para los archivos pytest |
| `llvm-mc` de LLVM 19+ con el target AMDGPU | pruebas del traductor que ensamblan RDNA2 | define `LLVM_MC` con su ruta |
| SPIRV-Tools (`spirv-as`, `spirv-val`) | pruebas del traductor y semánticas | el Vulkan SDK los incluye |
| XML del ISA RDNA2 de AMD (`amdgpu_isa_rdna2.xml`, MIT, del ISA legible por máquina de GPUOpen de AMD) | tablas del decodificador en algunas pruebas | define `LEONIDA_RDNA2_ISA_XML` |
| Windows: Visual Studio 2022 Build Tools (C++), CMake, Ninja | runtime | Linux y macOS compilan las partes multiplataforma; ver engine/runtime/README.md |
| Vulkan SDK 1.4 | dispositivo Vulkan, pruebas de GPU | define `VULKAN_SDK` |

## Compilar y probar

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

La salida de la compilación (`engine/runtime/build/`) la ignora git; nunca la subas (el bot cierra los pull
requests con binarios).

## Qué ejecuta el CI

| Workflow | Trabajos | Cuándo |
|---|---|---|
| `engine-translator` | pruebas del traductor (pytest + llvm-mc), pruebas semánticas, pruebas de la biblioteca estándar | cambios en `engine/` |
| `engine-runtime` | compilación en Windows + CTest; compilación en Linux, CTest y pruebas de GPU en llvmpipe; compilación en macOS, CTest y pruebas de GPU en MoltenVK | cambios en `engine/runtime`, `engine/scripts`, `engine/third_party` y los archivos del traductor que usa |

Rojos conocidos, seguidos como tareas: los fixtures de GPU de carga de imagen TFE en llvmpipe (#130) y una prueba
inestable por granularidad del temporizador en `crt_timed_wait` en Windows alojado (#132). Las pruebas de la
biblioteca estándar del motor que necesitan partes no importadas aquí (la canalización de compilación de títulos y
su documentación) están en `.github/engine-known-gaps.txt`; solo falla el trabajo un fallo fuera de esa lista, y
se avisa cuando una prueba de la lista empieza a pasar.

## Qué necesita un pull request del motor

- Las mismas reglas que cualquier pull request ([CONTRIBUTING.md](CONTRIBUTING.md)): una tarea reclamada,
  `Provenance:`, commits firmados.
- Una prueba junto al cambio: `engine/tests/` para el traductor y las herramientas, `engine/runtime/test/`
  (registrada en `engine/runtime/CMakeLists.txt` con `add_test`) para el runtime. El bot pide una cuando cambia
  código del motor sin prueba.
- CI del motor en verde para los trabajos que toque tu cambio, o una nota que explique un trabajo ya en rojo.
- Nada de ningún título en las pruebas: los fixtures se escriben a mano; los resultados en un título real pasan
  por [title-runs.md](title-runs.md).

## Probar tu cambio en un título

`tools/local_run.py` compila tu propia copia con el motor de este repositorio y escribe un informe depurado
([title-runs.md](title-runs.md)). El runtime imprime `[rt] crash: <code> at <module>+0x<offset>` cuando falla y
escribe lo mismo como JSON en `LEONIDA_CRASH_FILE`.
