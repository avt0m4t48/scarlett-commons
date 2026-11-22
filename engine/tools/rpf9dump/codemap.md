# tools/rpf9dump/

## Responsibility
Wrapper C# de exportación masiva sobre `RpfPreview.Core` de `third_party/rpf9-explorer` (submódulo git). Extrae recursos validados desde RPF9 y pools `.ttpf` del juego propio: modelos a OBJ, texturas a DDS, pools ttpf a DDS, bancos AWC y streams XMA2. No descifra TOC ni usa claves.

## Design
- Dos ficheros: `Program.cs` (top-level statements, ~188 líneas + clase `Util` estática) y `rpf9dump.csproj` (`net8.0`, `ImplicitUsings`, `Nullable`, `ServerGarbageCollection`, única `ProjectReference` a `../../third_party/rpf9-explorer/Core/RpfPreview.Core.csproj`).
- CLI: `rpf9dump <modo> <source> <outdir>` con salida `<outdir>/<basename-sin-ext>/`. Idempotente: salta ficheros ya existentes (`File.Exists`), contadores `ok/fail/skip`, progreso por % de escaneo.
- `Util.Safe()`: sanea nombres (chars inválidos/control → `_`, máx 120). `Util.BgraDds()`: cabecera DDS legacy 128 B (A8R8G8B8) para texturas embebidas decodificadas a BGRA32.
- `models`/`textures`: carve en dos fases — `Rpf9ModelRecovery` / `Rpf9TextureRecovery.FindCandidateOffsets()` + `RecoverAtOffsets()` en chunks de 32, con `RecoveryIndexCache.Load/Save` (clave: modo + longitud + 64 B de cabecera) para no reescanear; luego filtra `ArchiveReader.Open(src).Entries` por `Compression == "Recovered model" / "Recovered texture dictionary"`.
  - `models`: `RageModelDecoder.Decode(sys, gfx)` (split `ResourceSystemBytes` o `Segments[0].Size`) → `CreateObj()` (posiciones cuantizadas, UVs, índices). Manifiesto con nombre real, bounds, `Meshes/Verts/Tris`, `IsFragment`, shader por mesh, materiales (shader/preset hash, bucket, refs de textura con param/name hash, `OpaqueId64`, asset key). Texturas embebidas → `<stem>_tex/<idx>_<name>.dds` vía `DecodeEmbeddedTexturePixels + BgraDds`.
  - `textures`: `RageTextureDecoder.Inspect/CreateDds` → `<stem>/<idx>_<name>.dds` por entrada.
  - Manifiesto streameado fila a fila con `Utf8JsonWriter` a `manifest.json.partial` + `File.Move` atómico (150k filas de `levels.rpf` agotaban memoria con lista en RAM).
- `ttpf`: `TtpfReader.Inspect/CreateDds` → `{idx:D6}_{name}.dds` + `manifest.json` (versión, `DxgiFormat`, mips, `BaseMipLevel`, `AssetKey` X16, `File: null` + `skip++` si `!CanExtract`).
- `audio`: si `!AwcRecovery.HasCompleteCache` → `RecoverFull` con progreso + `WriteCache`; extrae `.awc` con `CanExtract` en lotes de 500 (`ExtractAsync`) a `<out>/awc/` + `audio_manifest.json` vía `AwcInfo.Inspect` (streams: id X8, codec, canales, sample rate, duración, `IsStreamed`).
- `xma`: solo streams con `Codec == 5` → `<out>/xma/<banco>/<id:X8>.xma.wav` vía `AwcRecovery.WriteXmaWave` (RIFF XMA2 que decodifica ffmpeg).

## Flow
`args[modo, src, outRoot]` → `outDir = outRoot/basename` → por modo: `ttpf` (inspección directa), `audio`/`xma` (caché AWC + `ArchiveReader.Open` + `AwcInfo`), `models`/`textures` (caché de índice o scan completo candidatos→validación→orden por offset→reindex→`ArchiveReader.Open`→decode→OBJ/DDS→manifiesto streameado). Todo error se cuenta en `fail` (solo los 5 primeros se imprimen) y el resumen final es `<modo> <base>: ok=N fail=N skip=N in Ns`.

## Integration
- Dependencias: .NET 8 SDK, `RpfPreview.Core` (`ArchiveReader`, `Rpf9ModelRecovery`, `Rpf9TextureRecovery`, `RageModelDecoder`, `RageTextureDecoder`, `TtpfReader`, `AwcRecovery`, `AwcInfo`, `RecoveryIndexCache`), `System.Text.Json`. Submódulo requerido (`git submodule update --init`); CI (`ci.yml`) compila con `dotnet build tools/rpf9dump/rpf9dump.csproj -c Release`.
- Consumidor: `make_game.py::rpf9dump(work)` compila a `<work>/tools/rpf9dump/` (`dotnet build -c Release -o <dir> --nologo -v q`, exige `dotnet` y Core presente) y `assets()` lo invoca: `models`+`textures` por cada `MODEL_ARCHIVES`, `ttpf` por cada `.ttpf` (`ttdb/**`, packs bajo `ttpf_pack/<pack>/`, ordenados por tamaño, skip si `manifest.json` existe), `audio` sobre `audio.rpf`. Downstream: `tools/awc_opus.py` (AWC→Ogg Opus) y ffmpeg (XMA2→WAV).
