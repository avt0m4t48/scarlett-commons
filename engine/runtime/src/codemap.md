# runtime/src/

## Responsibility

Punto de entrada (`main.cpp`) y mapeador PE nativo (`pe_loader.h/.cpp`) del runtime `gta6rt`.
Su único contrato actual: `gta6rt --dry-run <image> [--game-dir DIR]` mapea la imagen Xbox Series (x86-64) y sus módulos acompañantes, aplica relocs, resuelve todos los imports contra shims/host/módulos y emite un informe. No ejecuta código de la imagen (no llama a `DllMain`, TLS callbacks ni `AddressOfEntryPoint`).

## Design

- `rt::Loader` (en `pe_loader.h`): fachada con `load(path) -> Module&`, más caché `by_name_` (nombre en minúsculas) y `order_` en orden de carga. `game_dir_` es la raíz donde buscar DLLs acompañantes.
- `rt::Module`: descriptor POD-ish del módulo mapeado — `name/path/base/preferred_base/size/entry_rva/relocated/imports/exports`. `exports` indexa por nombre y por `"#ordinal"`.
- `rt::Binding` / `ImportRecord`: clasificación por import resuelto — `Shim | Host | Module | Unimplemented`. `main.cpp` cuenta por categoría y lista solo `Shim` + `Unimplemented` (superficie a implementar).
- Separación por fases en `Loader`: `map_image` → `bind_imports` (+ `resolve` por thunk) → `protect_sections`, con `parse_exports` tras el mapeo para permitir resolución recursiva módulo-a-módulo.
- `lower()` expuesto en el header como helper compartido (normalización case-insensitive de DLLs).
- Mirada cpp-pro / modern-C++ (C++20):
  - RAII a medias: `Loader::~Loader` libera con `VirtualFree(MEM_RELEASE)` — correcto pero manual. `Module*` son `new`/`delete` crudos en `order_`/`by_name_`; deberían ser `std::unique_ptr<Module>` (o `std::vector<std::unique_ptr<Module>>` + vista de punteros). Un `throw` entre `new Module` y `push_back` (p. ej. imagen inválida) fuga. Un wrapper RAII para la memoria `VirtualAlloc`/`VirtualFree` eliminaría el bucle manual del destructor.
  - Const-correctness débil: `at<T>(base, rva)` no distingue lectura/escritura; `parse_exports/bind_imports/protect_sections` toman `Module&` mutable aunque solo `bind_imports/resolve` mutan `imports`/IAT; `modules()` devuelve `const vector<Module*>&` pero expone punteros mutables. `binding_name` y `prefer_host_copy` bien como funciones libres internas.
  - Compilación estricta: el código asume `-Wall -Wextra -Wpedantic` limpio, pero hay puntos que esos flags/sanitizers señalarían — `reinterpret_cast` masivo sobre datos del fichero sin validar `e_lfanew`/RVAs contra `file.size()` (solo se valida cabecera MZ mínima), `at<>` sin chequeo de límites, truncado `uint32_t` en `AddressOfData`, y el `char tolower` con cast C-style. UBSan/ASan en el test harness son la red de seguridad natural para `map_image`/`bind_imports`.
  - Estilo: `std::filesystem`, `std::map`/`std::vector`, lambda `bind_thunks` para unificar imports y delay-imports (estos últimos ligados de forma eager a propósito para que `--dry-run` muestre la superficie completa). `prefer_host_copy` evita que el CRT de Xbox (`vcruntime140*.dll`, `msvcp140*.dll`, `concrt140.dll`, …) se cargue desde `game_dir_` y fuerza la copia del host.

## Flow

`main (wmain) -> pe_loader mapea imagen -> shims`:

1. `wmain` (`main.cpp`): parsea `argv` (`--dry-run <image> [--game-dir DIR]`, `game_dir` por defecto = directorio de la imagen). Sin `--dry-run` aborta: ejecutar código requiere antes los shims que lista el informe. Crea `rt::Loader(game_dir)` y llama `loader.load(image)`.
2. `Loader::load`: deduplica por nombre en minúsculas (`by_name_`); si es nuevo, `map_image` + `bind_imports` + `protect_sections`.
3. `Loader::map_image` (`pe_loader.cpp`): lee el fichero a `vector<uint8_t>`, valida `MZ` + `PE\0\0` + `AMD64`, reserva con `VirtualAlloc(preferred ImageBase, SizeOfImage, RW)`; si la base preferida está ocupada y hay tabla de relocs, reintenta con `nullptr` y marca `relocated`. Copia cabeceras + secciones (`min(SizeOfRawData, VirtualSize)` con guarda contra `file.size()`), aplica relocs `DIR64` (rechaza tipos desconocidos salvo `ABSOLUTE`), llama `parse_exports`.
4. `Loader::parse_exports`: indexa `AddressOfFunctions/Names/Ordinals` por nombre y `"#ordinal"`. No sigue exports reenviados (asunción documentada: los DLLs acompañantes no los usan).
5. `Loader::bind_imports`: recorre `IMAGE_DIRECTORY_ENTRY_IMPORT` y `DELAY_IMPORT`; por cada thunk resuelve nombre vs. ordinal (`IMAGE_SNAP_BY_ORDINAL64`) y parchea la IAT con el puntero devuelto por `resolve`.
6. `Loader::resolve` — cadena de prioridad por import `(dll, símbolo)`:
   a. `shims::find(dll_lower, sym)` → `Binding::Shim` (GDK, XMem, `d3d12_xs`, etc.).
   b. Si no es CRT de `prefer_host_copy` y existe `game_dir_/dll`, `load()` recursivo del módulo acompañante y búsqueda en sus `exports` → `Binding::Module`.
   c. `LoadLibraryExA(dll, SEARCH_SYSTEM32)` + `GetProcAddress` (nombre u ordinal) → `Binding::Host` (DLLs de sistema y API sets, sin contaminar la búsqueda con la carpeta del juego).
   d. Si todo falla → `Binding::Unimplemented` + `shims::make_unimplemented_thunk(dll, sym)` (thunk que aborta al llamarse, pero permite completar el mapeo/informe).
7. `Loader::protect_sections`: cabeceras a `PAGE_READONLY`, cada sección según `IMAGE_SCN_MEM_EXECUTE/WRITE`, más `FlushInstructionCache`. Deja la imagen lista para (futura) ejecución.
8. Vuelta en `main.cpp`: itera `loader.modules()`, imprime por módulo `base/relocated/size/entry/+RVA` y conteos `shim/host/module/unimplemented`, detalla líneas `shim` y `UNIMPLEMENTED` como `dll!símbolo`, y el total `N module(s) mapped, M unimplemented import(s)`. Errores como excepciones → `stderr` + código 1; uso incorrecto → código 2.

## Integration

- Con `runtime/shims` (`../shims/shims.h`): `pe_loader.cpp` solo depende de dos funciones — `shims::find(dll, sym) -> void*` (primer intento de resolución: GDK, XMem, `d3d12_xs`) y `shims::make_unimplemented_thunk(dll, sym) -> void*` (fallback que registra el hueco sin romper el mapeo). El informe de `--dry-run` es de hecho la lista de trabajo de shims: todo `UNIMPLEMENTED` es un shim por escribir; todo `shim` es un acoplamiento a verificar.
- Con `runtime/test harness`: `Loader` está diseñado para ser instanciado desde tests sin ejecutar la imagen — `load()` + inspección de `modules()/imports/exports/relocated` permite asertar mapeo, relocs, prioridad shim > bundled > host, y conteo de `Unimplemented`. Es el lugar ideal para correr ASan/UBSan sobre PEs de prueba (headers corruptos, RVAs fuera de rango, relocs desconocidos) y para compilar con `-Wall -Wextra -Wpedantic`.
- Límites conocidos que condicionan la integración: sin ejecución (`DllMain`/TLS/entry pendientes del siguiente hito), sin forwarders de exports, delay-imports resueltos de forma eager, y propiedad manual de memoria/`Module*` pendiente de migrar a `unique_ptr` + wrapper RAII de `VirtualAlloc`.
