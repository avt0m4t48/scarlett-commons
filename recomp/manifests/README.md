# Track B manifests

One file per build of a title: `<build hash>.toml`, where the hash is the one the runtime prints as `build` (BLAKE3 of
the executable). It lists the functions that run as recompiled code (track B); everything else runs natively under the
VIctory runtime (track A).

```toml
build = "<build hash>"

[[function]]
name = "our_name_for_it"         # an implementation in commons-recomp's REGISTRY
rva = 0x1234560                  # where the native function starts, relative to the image
prologue = "48 89 5c 24 08 ..."  # its first bytes (at least 14): nothing is written if they differ
status = "verified"              # verified (installed), testing (only with COMMONS_ONLY), off
```

Only facts go here: addresses, byte signatures and our names. No game code or data.

Run with track B: `cargo commons run <exe> --execute` (from `victory/`). `COMMONS_TRACK=a` runs track A alone;
`COMMONS_ONLY=name,...` installs just those functions, testing ones included.
