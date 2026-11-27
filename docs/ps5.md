# The PS5 line

The same engine code ships for Xbox Series and for PS5, and both machines run x86-64 on an RDNA2 GPU. The PS5
line takes the player's own decrypted copy, `eboot.bin` and the modules beside it. It runs that copy through
the same hybrid as the Xbox line. Only the parts that are the console's own change.

Nothing of a title goes into this repository: no code, no data, no shaders, no NIDs read from a title, no logs
quoting them. The tools read the player's copy on the player's machine and print counts.

## Layers

| Layer | Xbox line | PS5 line |
|---|---|---|
| Executable | PE, functions from `.pdata` | ELF (decrypted SELF), functions from `.eh_frame` (`recomp/src/elf.rs`) |
| CPU code | native x86-64, track B translations installed over it | the same: track B reads ELF |
| System services | VIctory's HLE of the GDK | an HLE of the PS5's libraries, keyed by library and NID; unknown calls logged |
| Linking | PE imports | SCE dynamic linking: modules, libraries, NID symbols (`recomp/src/ps5.rs`) |
| GPU commands | D3D12 command lists to host D3D12 | the PS5's command buffers to the same host D3D12 backend |
| Shaders | RDNA2 ISA translated to DXIL | the same ISA: the same translator once the container is read |
| Files | the title's archives, BCPack and zlib | the PS5's package layout, read from the player's copy |

## Tools

- `commons-ps5-scout <eboot.bin|.prx>`: which modules and libraries the executable imports and how many symbols
  from each, its functions, and how much of their code track B covers today. A signed (encrypted) executable is
  refused with the reason. Decryption happens on the console the copy came from, never here.
- `commons-census`, `commons-recomp`: take an ELF as well as a PE.

## Order

1. Scout the player's copy: imports by library, functions, coverage.
2. Load: map the ELF, resolve its imports to stubs that log the library and NID, and reach the first unknown
   call.
3. HLE the libraries the scout ranks first (kernel, threads, memory, files), the way VIctory does for the GDK.
4. GPU: read the command buffers into the host backend, and translate the shaders with the shared translator.
5. Track B over the ELF, checked function by function against the native code, as on Xbox.

## Other projects

- **AnyPS5** (GPL-2.0-only): a relinker with its own system libraries. Its licence cannot be combined with ours.
  We read it for ideas and run it beside ours as a separate program; we do not copy its code.
- **VIctory and its forks** (GPL-3.0): the HLE and GPU work our PS5 layer copies its shape from.
