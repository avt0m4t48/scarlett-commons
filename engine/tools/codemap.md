# tools/

## Responsibility

Caja de herramientas offline de extracción y traducción de assets Xbox (RPF9 con TOC cifrada) a artefactos PC: shaders DXBC→FSSM→asm RDNA2→SPIR-V validado, texturas detiled a DDS, audio AWC→Ogg/WAV, e inventario (manifiestos, hashes, RTTI/PE). Todo opera por carving sin descifrar la TOC; `tools/rpf9dump/` (C#, ver `tools/rpf9dump/codemap.md`) es la vía bulk con decodificadores, los `.py` son la vía de investigación/forense y el pipeline de shaders.

## Design

- **Pipeline por etapas con contratos JSON:** `shader_carve.py` → `fssm_unpack.py` → `shader_disasm.py` → `rdna2spirv.py`; `rpf9_carve.py` → `classify_carve.py` → `tex_headers.py` → `tex_export.py`. Cada etapa escribe `index.json` / `manifest.json` / `fssm_index.json` que la siguiente consume sin releer el `.rpf`.
- **Carver sin TOC (mmap + regex/zlib):** `shader_carve.py` (patrón `<name>\0 kind stored id stored size` + inflate raw-deflate, valida `DXBC` y parte con `parts()`/`shader_model()`), `rpf9_carve.py` (barre `ZHDR` zlib/Bink2 con `mmap`, mide cobertura `unread`/`zero`). `rpf_probe.py` solo lee 64 KiB y estima cifrado por entropía.
- **Unpacker XBCH→FSSM:** `fssm_unpack.py` navega `DXBC→DXIL→XDAT` (tags `0x0D=XBCH`, `0x12=hash`, `0x07=NOBC`), hace `zlib.decompress(blob[24:])` y verifica `unpacked`.
- **Parser → Translator (Strategy por familia ISA):** `shader_ir.py` (`parse_file()`, operandos `sgpr/vgpr/special/imm/target`) desacopla sintaxis LLVM de semántica; `rdna2spirv.py:class Gen` emite SPIR-V assembly por tablas (`F2/U2/SHIFT_REV/F1/S2`, `NOOPS`) con modelo "una invocación = un lane" (VGPR/SGPR como arrays `Function`, `EXEC=all-ones`); lo no soportado lanza `Unsupported`, nunca adivina. `gfx10_formats.json` es la tabla `GFX10_FORMAT_*` (Mesa) para decodificar operandos buffer.
- **Profiler/Probe empírico + scoring por suavidad:** `shader_profile.py` (conteo por mnemónico/familia `smem/scalar/vector/lds/buffer/image/export`), `tile_probe.py` / `swizzle_search.py` / `tex_mode_probe.py` prueban layouts candidatos (Morton/Z-order, patrones addrlib 4K/64K, top-mip al inicio o al final — GFX10 guarda mips smallest-first) y eligen por `bc_feature.py:block_average()/smoothness()` (promedio exacto BC1/BC3). `amd_swizzle.py` (`load()/equation()/detile()`) construye ecuaciones XOR desde `gfx10SwizzlePattern.h` de Mesa.
- **Utilidades de inventario:** `manifest.py` (SHA-256 `create/verify`), `hashdict.py` (joaat lowercase desde blobs/strings para resolver hashes RAGE), `classify_carve.py` (solo `index.json`: `bink2/rbf0/respage/gpu-zerohdr`), `pe_inspect.py` / `rtti_scan.py` (solo lectura con `pefile`; RTTI x64 por RVAs + `CompleteObjectLocator→ClassHierarchy`), `awc_opus.py` (reempaquetado lossless AWC `ADAT` 2048-byte blocks → Ogg con `ogg_crc()/ogg_page()`, PCM→WAV; música tag `0x48` a mono por canal con skip de prefijo repetido).

## Flow

1. **Shaders:** `shaders.rpf` → `shader_carve.py` (`ENTRY` regex → `*.dxbc` + `index.json` con `stage/sm/parts`) → `fssm_unpack.py` (`XBCH` → `zlib` → `fssm/*.fssm`) → `shader_disasm.py` (`code_range()` por qword `0xC0` + `0x28`, hasta último `s_endpgm=0xBF810000`, fallback scan hacia atrás; `llvm-mc --disassemble -triple=amdgcn--amdpal -mcpu=gfx1013 -show-encoding` en `ThreadPoolExecutor` → `*.s`) → `shader_ir.py` → `rdna2spirv.py` (`--out` → `*.spvasm` + `*.spv` vía `spirv-as`, chequeo con `spirv-val`, `index.json` de cobertura). `shader_profile.py` guía qué mnemónicos añadir al traductor.
2. **Texturas legacy/forense:** `rpf9_carve.py` (`<off>.bin` + `index.json`) → `classify_carve.py` (`types.json`) → `tex_headers.py` (descriptor `q[k-2..k+5]`: tipo `0x980`, puntero GPU `0x60000000+off`, `WxHxD/format DXGI`, tiling; CPU blob + cadena GPU blobs → `textures_index.json`) → `tex_export.py` / `amd_swizzle.py` (detile `SW_4K_S`, elige placement de top-mip por suavidad → `*.dds` + `export_log.json`).
3. **Audio:** bancos `.awc` (crudos en `audio.rpf`) → `awc_opus.py` (`<bank>/<stream id>.ogg`/`.wav` sin re-encode, granule recortado a samples declarados).
4. **Inventario:** `manifest.py` / `hashdict.py` (`hashes.json`) sobre `carve/` + `--extra` (exe cuando llegue); `pe_inspect.py` / `rtti_scan.py` sobre imagen PE sin ejecutarla.

## Integration

- **Llamado por `make_game.py:shaders()`:** secuencia `py("shader_carve.py",…)` → `py("fssm_unpack.py",…)` → `py("shader_disasm.py",… --mcpu gfx1013)` (requiere `LLVM_MC` ≥21 AMDGPU) → `py("rdna2spirv.py",…)` (requiere `spirv-as`/`spirv-val` del Vulkan SDK); salidas bajo `<work>/shaders/{dxbc,fssm,disasm,spirv}`.
- **Llamado por `make_game.py:assets()` + `rpf9dump()`:** compila `tools/rpf9dump/rpf9dump.csproj` (necesita `third_party/rpf9-explorer` + .NET 8) y corre `models|textures|ttpf|audio`; solo `awc_opus.py` sigue en la ruta bulk (`audio/awc` → `audio/ogg`). `rpf_probe.py`, `rpf9_carve.py`, `tex_*.py`, `tile_probe.py`, `swizzle_search.py`, `hashdict.py`, `classify_carve.py` no los llama el orquestador: son forenses/manuales.
- **Consumido por `runtime/`:** `runtime/test/vk_pipeline_check.cpp` lee `<spirv>/index.json` y crea un `VkShaderModule` por entrada.
- **Dependencias externas:** `llvm-mc`, `spirv-as`/`spirv-val`, `pefile`, `numpy` (+ Pillow para scoring DDS), `dotnet` para `rpf9dump`.
