# runtime/

## Responsibility

Loader + compatibilidad Windows x64 para ejecutar el juego traducido: `gta6rt.exe`
carga el PE del juego y le da shims Win32/D3D12 mínimos, y `vk_pipeline_check.exe`
valida los shaders SPIR-V traducidos contra el driver Vulkan real. Solo Windows.

## Design

- C++20, x64 obligatorio (`CMAKE_SIZEOF_VOID_P EQUAL 8`, si no `FATAL_ERROR`).
- Flags MSVC: `/W4 /permissive- /utf-8`, defines `WIN32_LEAN_AND_MEAN NOMINMAX
  UNICODE _UNICODE`, link `/OPT:NOREF` (no descartar tablas de shims con
  registro estático) + `psapi.lib`.
- Build dual: CMake+Ninja si VS trae "C++ CMake tools", si no `cl.exe` directo
  (`scripts/build_runtime.ps1`, lista de fuentes duplicada a mano — mantener en
  sync con `CMakeLists.txt`).
- Vulkan SDK opcional: `find_package(Vulkan QUIET)`; sin SDK solo se compila
  `gta6rt`, con SDK además las herramientas GPU. Cargan `vulkan-1.dll` en runtime.
- Detalle interno delegado: ver `src/codemap.md` (loader + entry point) y
  `shims/codemap.md` (tablas de compatibilidad). Este mapa no duplica ese contenido.

## Flow

1. `scripts/build_runtime.ps1` → `vcvars64.bat` → cmake+ninja o `cl.exe` →
   `runtime/build/gta6rt.exe` (+ `vk_pipeline_check.exe` si hay `VULKAN_SDK`).
2. `gta6rt.exe` arranca → registra shims → `pe_loader` mapea el exe del juego →
   redirige imports Win32/D3D12 a los shims → ejecuta.
3. `vk_pipeline_check.exe <spirv dir> [cache.bin]` crea cada shader module y
   compute pipeline sobre la GPU física para detectar fallos de driver antes de jugar.

## Integration

- Targets CMake (`runtime/CMakeLists.txt`):
  - `gta6rt` ← `src/main.cpp`, `src/pe_loader.cpp`, `shims/registry.cpp`,
    `shims/xmem.cpp`, `shims/d3d12_xs.cpp`.
  - Condicionales (solo con Vulkan SDK): `vk_harness`, `vk_texture_test`,
    `vk_compute_test`, `vk_store_test`, `vk_pipeline_check` ← `test/<tool>.cpp`.
- Layout:
  - `src/` — entry point + PE loader (ver `src/codemap.md`).
  - `shims/` — shims Win32/D3D12 (ver `shims/codemap.md`).
  - `test/` — harness y checks Vulkan + `shaders/`; **excluido del codemap**
    (herramientas de validación, no parte del loader).
  - `build/` — salida (`gta6rt.exe`, `vk_pipeline_check.exe`, generado, no versionado).
- `make_game.py` stages:
  - `runtime` (`runtime()`): solo `os.name == "nt"`; invoca
    `scripts/build_runtime.ps1`, copia `runtime/build/gta6rt.exe` al `out/`.
    Sin el exe original del juego avisa pero igual compila.
  - `pipelines` (`pipelines()`): exige `runtime/build/vk_pipeline_check.exe`
    (falla si se compiló sin Vulkan SDK) y lo corre sobre los SPIR-V generados.
- Requiere Visual Studio 2022 (Desktop development with C++ x64) + Vulkan SDK;
  `scripts/first_look.ps1` reconstruye `gta6rt.exe` si falta.
