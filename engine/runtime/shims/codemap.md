# runtime/shims/

## Responsibility

Capa de compatibilidad entre el juego (compilado contra el system software de Xbox: GDK, XMem, `d3d12_xs`) y el host Windows/Linux. Reimplementa los imports que no existen en PC para que `pe_loader` pueda enlazar la IAT del ejecutable sin las DLLs de consola.

Estado actual: esqueleto funcional. Solo cubre las funciones que importan las DLLs empaquetadas; todo lo demás cae en thunks `unimplemented` que registran la llamada y devuelven 0. El backend D3D12→Vulkan está pendiente: `d3d12_xs.cpp` solo redirige dos funciones libres, las interfaces `ID3D12Device*` / command lists Xbox quedan fuera del alcance actual.

Ficheros:

- `shims.h` — contrato público: `Entry`, `Table`, `find`, `make_unimplemented_thunk`, `report_called_unimplemented`, macro `SHIM_TABLE`.
- `registry.cpp` — tabla global `dll!nombre → fn`, generador de thunks JIT y registro de llamadas no implementadas.
- `xmem.cpp` — shim de `xmem.dll` (allocador con tracking del GDK).
- `d3d12_xs.cpp` — shim de `d3d12_xs.dll` (build de D3D12 para Xbox Series).

## Design

### Registro estático por tabla (`shims.h` + `registry.cpp`)

Cada shim es un `.cpp` con una `SHIM_TABLE(id, ...)` que declara un array estático de `Entry{dll, name, fn}` y un objeto `Table` estático. El constructor de `Table` inserta cada entrada en un `std::map<std::string, void*>` global bajo la clave `"dll!nombre"` (DLL en minúsculas, con `.dll`; el nombre puede ser `"#ordinal"` para exports por ordinal). El registro ocurre en el arranque por inicialización de estáticos, sin lista central que mantener: añadir una DLL es crear `shims/<dll>.cpp` con su tabla y listarlo en `CMakeLists.txt`.

`find(dll_lower, name)` es una búsqueda O(log n) en ese mapa. No hay desregistro ni prioridades: el primer registro gana salvo colisión accidental de claves (hoy silenciosa, último escritor vence).

### Thunks `unimplemented` generados por JIT (`registry.cpp`)

`make_unimplemented_thunk(dll, name)` reserva páginas `PAGE_EXECUTE_READWRITE` en un pool de 1 MiB (`VirtualAlloc`, con `FlushInstructionCache` tras escribir) y emite por import un thunk de 32 bytes:

```text
48 B9 <imm64>   mov rcx, "dll!name"   ; char* con _strdup, vive todo el proceso
48 B8 <imm64>   mov rax, rt_unimplemented
0F 57 C0        xorps xmm0, xmm0       ; limpia retorno float/vector
FF E0           jmp rax               ; tail-call: vuelve directo al llamante del juego
CC ...          int3 de relleno hasta 32 B
```

El destino `rt_unimplemented(const char*)` guarda la cadena en un `std::set` bajo mutex, la imprime una sola vez por `stderr` (`[rt] unimplemented: dll!name`) y devuelve 0 — que llega al llamante tanto en `rax` (enteros/punteros/`HRESULT`-como-0 = `S_OK` falso) como en `xmm0` (float/double). Es deliberadamente permisivo para modo `--dry-run`/sondeo: el juego sigue corriendo aunque el valor sea falso. `report_called_unimplemented()` vuelca al final la lista de lo que realmente se llamó (`called: ...`), que es la fuente para decidir qué shim implementar después.

Concurrencia: `g_called` y el pool JIT tienen cada uno su `std::mutex`. El mapa de `table()` se escribe solo durante la init de estáticos y se lee después sin lock — seguro mientras nadie registre tablas en caliente (hoy cierto).

### `xmem.cpp` — GDK allocator sobre el heap del host

Reimplementa 4 exports de `xmem.dll`:

| Export | Implementación |
|---|---|
| `XMemAlloc(size, attrs)` | `_aligned_malloc(size ? size : 1, 64)` (`__stdcall`) |
| `XMemFree(p, attrs)` | `_aligned_free(p)` |
| `XMemVirtualQuery(addr, info, len)` | `VirtualQuery` directo (misma forma que `MEMORY_BASIC_INFORMATION`) |
| `XMemGetWorkingSetStatistics(stats)` | `memset(stats, 0, 32)` + copia `WorkingSetSize` de `GetProcessMemoryInfo` en el primer campo; `E_INVALIDARG` si `stats == null` |

Decisiones y deuda explícita (documentadas en el propio `.cpp`): los `XALLOC_ATTRIBUTES` (alineamiento y tipo de memoria) se ignoran y todo se alinea a 64 B — cubre línea de caché y SIMD pero no memoria física contigua ni heaps GPU; el layout de `XMEM_WORKING_SET_STATISTICS` se desconoce y se rellena con un tamaño conservador de 32 B. Ambos `TODO` piden verificar contra los call sites reales del exe qué campos se leen.

Sin RAII propio: delega la propiedad al CRT (`_aligned_malloc`/`_aligned_free`) y a las APIs Win32. No hay tracking de bloques, fugas ni estadísticas reales — es un adaptador fino, no un allocator.

### `d3d12_xs.cpp` — pasarela mínima hacia el `d3d12.dll` del host

Expone solo las 2 funciones libres que importan las DLLs empaquetadas:

- `D3D12SerializeVersionedRootSignature`
- `D3D12CreateRootSignatureDeserializer`

Cada una hace `LoadLibraryExA("d3d12.dll", SEARCH_SYSTEM32)` una vez (estático local) y reenvía con la misma firma (`WINAPI`, `HRESULT`). Si el host no tiene la función, avisa una vez por `stderr` y devuelve `E_NOTIMPL`.

La hipótesis arquitectónica está escrita en la cabecera del fichero: los descriptores y blobs de root signatures tendrían el mismo layout en Xbox y en PC, así que el reenvío es válido; las interfaces device/command-list tienen métodos solo-Xbox y extensiones de placement/herencia ESRAM y **no** se shimean aquí. Ese es el trabajo pendiente del backend Vulkan: una capa que implemente `ID3D12Device*` Xbox sobre Vulkan, no más reenvíos a `d3d12.dll`.

### Patrones transversales

- **Sin clases ni herencia salvo `Table`**: `Table` es el único RAII del módulo, y su "adquisición" es el registro en el mapa global en construcción. El resto son funciones libres con linkage interno (`namespace { ... }`) y `__stdcall`/`WINAPI` donde la ABI del juego lo exige.
- **Interfaces C planas**: los `Entry.fn` son `void*` con `reinterpret_cast` desde punteros a función; el tipado real vive solo en la definición de cada shim.
- **Fail-soft por defecto**: lo desconocido devuelve 0/`S_OK` falso y sigue; lo conocido-fallido devuelve `E_NOTIMPL`/`E_INVALIDARG`. El log a `stderr` es el único mecanismo de observabilidad.

## Flow

Resolución de un import (caso feliz y caso fallback):

```text
pe_loader::bind_imports (IAT + delay-imports, eager)
  └─> Loader::resolve(dll, sym)
        ├─> shims::find("xmem.dll", "XMemAlloc") ──hit──> puntero directo al shim
        │                                                  (Binding::Shim, escrito en la IAT)
        ├─> DLL empaquetada en game_dir_ ──hit──> export del módulo (Binding::Module)
        ├─> LoadLibraryExA(SYSTEM32) + GetProcAddress ──hit──> host (Binding::Host)
        └─> miss total ──> make_unimplemented_thunk ──> thunk JIT en la IAT (Binding::Unimplemented)
              └─> en tiempo de ejecución: thunk ──jmp──> rt_unimplemented("dll!name")
                    ├─> log una vez + insert en g_called
                    └─> return 0 (rax + xmm0) ──> vuelve al juego
```

Al salir (o en `--dry-run`), `report_called_unimplemented()` vuelca `g_called`: esa lista es el backlog priorizado de shims.

Flujo específico de `d3d12_xs`:

```text
juego llama D3D12SerializeVersionedRootSignature(desc, blob, err)
  └─> shim d3d12_xs ──> host("D3D12SerializeVersionedRootSignature") [lazy, static]
        ├─> hit ──> d3d12.dll del host hace el trabajo real
        └─> miss ──> E_NOTIMPL
```

## Integration

- **Consumidor principal: `runtime/src/pe_loader.cpp` (`Loader::resolve`, `bind_imports`)**. Incluye `../shims/shims.h` y consulta `shims::find` **antes** que la DLL empaquetada y que el host: un shim siempre tiene prioridad sobre una copia del sistema con el mismo nombre. `bind_imports` enlaza imports normales y delay-imports de forma eager para que `--dry-run` exponga toda la superficie de imports de una pasada. Cada resolución queda clasificada en `Module::imports` (`Shim | Module | Host | Unimplemented`) para diagnóstico.
- **Productor de trabajo futuro**: la salida de `report_called_unimplemented()` + el campo `Binding` de `pe_loader` dicen qué implementar a continuación; el orden natural es GDK (`xmem`, resto de `XMem*`, `XGame*`, threading) antes que gráficos.
- **Futuro backend Vulkan**: `d3d12_xs.cpp` es el punto de anclaje. Hoy solo reenvía funciones libres; cuando se aborde D3D12→Vulkan habrá que (a) definir las vtables `ID3D12*` con el layout Xbox (métodos extra respecto a PC), (b) registrar fábricas (`D3D12CreateDevice` Xbox, creación de command queues/lists) en una `SHIM_TABLE` ampliada, y (c) traducir root signatures/PSOs ya serializables (las 2 funciones actuales) a pipelines Vulkan. Nada de eso existe aún; el fichero marca explícitamente el límite.
- **Cómo añadir un shim nuevo**: crear `runtime/shims/<dll>.cpp` con `SHIM_TABLE(<dll>, {"<dll>.dll", "<Export>", fn}, ...)`, respetar la ABI (`__stdcall`/`WINAPI`, nombres/ordinales exactos, DLL en minúsculas en la tabla) y listarlo en el `CMakeLists` del runtime. Para exports desconocidos no hace falta código: el thunk genérico los cubre y `report_called_unimplemented` los revela.
- **Dependencias externas**: Win32 (`VirtualAlloc/Protect/Query`, `LoadLibraryExA`), CRT alineado (`_aligned_malloc/_free`), `psapi.dll` (`GetProcessMemoryInfo`). No depende de Vulkan, del GDK real ni de ningún otro módulo del repo.
