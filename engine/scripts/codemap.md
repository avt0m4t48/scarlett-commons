# scripts/

Windows-only PowerShell helpers for day-one inspection, bulk carving, metadata decoding and runtime builds. No game code ever runs from these scripts.

## Responsibility

- Proveer el flujo "primer contacto" con el ejecutable del juego (inspección + dry run del loader sin ejecutar nada).
- Compilar `runtime/build/gta6rt.exe` con MSVC a prioridad reducida.
- Automatizar tareas de investigación por lotes: carveo de todos los `.rpf` y conversión de metadatos PSO/RBF a XML.
- Ningún script forma parte del build final: son herramientas de desarrollo manual o invocadas puntualmente por `make_game.py`.

## Design

- Cuatro scripts autocontenidos, cada uno con su cabecera de uso; solo `first_look.ps1` reutiliza a otro (`build_runtime.ps1` como fallback si falta `gta6rt.exe`).
- Todo resuelve rutas relativas a la raíz del repo vía `Split-Path $PSScriptRoot -Parent`; no asumen cwd.
- `$ErrorActionPreference = "Stop"` en `build_runtime`, `first_look` y `meta_to_xml` para fallar rápido; `carve_all` es best-effort por archivo y agrega `coverage.json`.
- Prioridad `BelowNormal` en compilación y conversión masiva para no bloquear la máquina (mitad de los cores en el build, `ProcessorCount / 2`).
- `build_runtime.ps1` prefiere el CMake + Ninja que trae Visual Studio (componente "C++ CMake tools") y cae a `cl.exe` directo con la lista de fuentes sincronizada a mano con `runtime/CMakeLists.txt`.
- `first_look.ps1` y `carve_all.ps1` delegan el trabajo pesado a Python con `python -I` (`tools/pe_inspect.py`, `tools/rpf9_carve.py`); los PS1 solo orquestan rutas y reportes.
- `meta_to_xml.ps1` corre en Windows PowerShell 5.1 y carga `CodeWalker.Core.dll` (+ SharpDX) desde una ruta externa (`<ref>\codewalker-gen9`) para parsear blobs.

## Flow

1. Llega el exe → `first_look.ps1 -Exe <game>\game_scarlett_master.exe` verifica cabecera `MZ`, calcula SHA256, corre `tools/pe_inspect.py`, hace dry run del loader (`gta6rt.exe --dry-run`) y genera la shim to-do list por DLL en `reports/exe/<name>/`.
2. Si el runtime no está compilado, `first_look.ps1` invoca `build_runtime.ps1` automáticamente; el flujo normal de build lo hace `make_game.py` (ver Integration).
3. Investigación de datos (manual, una sola vez por copia): `carve_all.ps1 -Game <game> -Out <work>\carve` recorre todos los `.rpf` de menor a mayor, salta los que ya tienen `index.json`, y escribe `coverage.json` con `files / bink2 / span / readable / unread / inflated` por archivo.
4. Sobre el carveo: `meta_to_xml.ps1 -Carve <work>\carve -Out <work>\meta [-Limit N]` filtra blobs por magic (`PSIN` = PSO, `RBF0` = RBF), los carga con `PsoFile`/`RbfFile`, emite `<offset>.pso.xml` / `<offset>.rbf.xml` por archivo y acumula fallos en `failures.txt`.

## Integration

| Script | Qué hace | Quién lo llama |
|---|---|---|
| `build_runtime.ps1` (`[-Debug]`) | Localiza VS vía `vswhere.exe`, inicializa `vcvars64.bat`, configura `runtime/build` con CMake+Ninja o compila `main.cpp`, `pe_loader.cpp`, `shims/registry.cpp`, `shims/xmem.cpp`, `shims/d3d12_xs.cpp` con `cl.exe` (`/O2 /MD`, o `/Od /MDd` en Debug) más `vk_pipeline_check.cpp` si hay `VULKAN_SDK`. Genera `_build.cmd` intermedio para evitar el quote-stripping de `cmd.exe` y lanza la compilación con `Start-Process` en `BelowNormal`. Salida: `runtime/build/gta6rt.exe`. | `make_game.py runtime()` — lo invoca con `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build_runtime.ps1` (solo Windows; en otro OS falla con "The runtime builds on Windows only"), luego copia el exe a `<out>/gta6rt.exe` y escribe `game_path.txt`. Uso manual: `.\scripts\build_runtime.ps1` (ver `docs/design.md`). Llamado implícito desde `first_look.ps1` si falta el runtime. |
| `first_look.ps1` (`-Exe`, `[-GameDir]`) | Reporte del día uno, solo lectura del exe: valida `MZ`, escribe `hash.txt` (SHA256 + tamaño), corre `tools/pe_inspect.py --game-dir --out`, ejecuta `gta6rt.exe --dry-run` (mapea, reubica y enlaza imports sin ejecutar) a `dryrun.txt`, deriva `shim-todo.txt` (DLLs pendientes ordenadas por nº de funciones, `where == "shim"`) desde `<name>.json` y avisa de secciones de alta entropía (cifradas/empaquetadas). Salida: `reports/exe/<name>/`. | Solo dev manual: `.\scripts\first_look.ps1 -Exe "<game>\game_scarlett_master.exe"`. Nadie lo llama desde `make_game.py`. |
| `carve_all.ps1` (`-Game`, `-Out`) | Herramienta de investigación: `Get-ChildItem -Recurse -Filter *.rpf \| Sort-Object Length`, por cada archivo calcula `dest = <Out>\<ruta_con__>` y si no existe `index.json` corre `python -I tools/rpf9_carve.py`; luego lee el `summary` de cada `index.json` y vuelca `coverage.json` + tabla (archivo, ficheros, bink2, GiB, readable/unread). | Solo dev manual. El build no lo necesita (el propio script lo indica en la cabecera). |
| `meta_to_xml.ps1` (`[-Carve]`, `[-Out]`, `[-CodeWalker]`, `[-Limit]`) | Decodifica metadatos ya carveados: por cada subdirectorio con `index.json` filtra entradas por `head` (`5053494e`/`52424630`), lee `<offset>.bin`, parsea con `CodeWalker.GameFiles.PsoFile`/`RbfFile` y escribe `<offset>.<pso\|rbf>.xml` vía `PsoXml::GetXml`/`RbfXml::GetXml`. Cuenta `ok/bad` por tipo e imprime resumen; fallos a `<Out>\failures.txt` con el mensaje de la excepción interna. | Solo dev manual. Requiere copia externa de CodeWalker y PowerShell 5.1; no lo llama `make_game.py`. |
