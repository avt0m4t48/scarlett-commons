# NVIDIA: what was measured and why

Measured on one PC: RTX 4090, i7-13700KF, 64 GB, Windows 11, the title 1.0.2.1, 4-minute runs with a scripted
walk and camera turns after loading (`victory/` on the AvatarManz base).

| | median | worst 1% | crashes |
|---|---|---|---|
| ray-traced light, Series S, no relay switches (the script below) | **11.1 fps** | 212 ms | 0 |
| the same, Series X | 10.0 fps | 208 ms | 0 |
| with the relay and card-copy switches (`VICTORY_RELAY=2` and others) | 6.1 fps | 276 ms | 0 |
| the light stand-in (`VICTORY_AMBIENT`) instead of rays | 7.7 fps | 237 ms | 0 |

What was wrong, and what changed:

- **Reads before a copy landed.** Lists of one `ExecuteCommandLists` read buffers a copy earlier in the same call
  had just written, still in COPY_DEST: the D3D12 debug layer counted 22,108 in 5 minutes, and NVIDIA can read
  the bytes from before the copy (pictures that differ from frame to frame, history that smears in motion).
  Each list now goes in its own call (`VICTORY_SPLIT_SUBMITS`, default on): 0 left.
- **Shaders the card refuses.** With the cool_fox and upstream translators 748 of 2,763 compute programs are
  invalid DXIL (Microsoft's validator) and NVIDIA refuses them: shadows, ambient light and ray-traced passes did
  not run. The AvatarManz translator validates all 2,763.
- **TDR.** Windows resets a card after 2 s of GPU work; long but correct submissions ended runs. Raise
  `TdrDelay` and `TdrDdiDelay` (HKLM\System\CurrentControlSet\Control\GraphicsDrivers) to 60.
- **Shared caches.** Pipeline libraries saved by one build fail to load in another (E_INVALIDARG): each build
  keeps its own `VICTORY_CACHE_DIR`.

Still open: 6 compute programs need 64-lane waves (NVIDIA has 32); 71 mesh programs of characters' fire and
decal effects declare more than 32 KB of outputs; one fill a frame into the title's constant ring waits ~45 ms
for the card (the 174 MB heap is declared written as a whole); descriptors the title changes within a frame
reach computed-index buffer reads a frame late.

## Running with these settings

Save as `play-best.ps1` (outside the repository) and run it with `-Game <folder of your own copy>`:

```powershell
# Runs the title with the settings that measured best on an RTX 4090 (2026-11-26, docs/nvidia.md):
# the AvatarManz-based runtime with split submissions, Series S profile, the title's own ray-traced light,
# no relay/card-copy switches, its own cache folder, BCPack tables from outside the repository.
#   tools\play-best.ps1 -Game <folder of your own copy> [-Exe <victory.exe>] [-Tables <bcpack-tables.bin>] [-SeriesX]
param(
    [Parameter(Mandatory)] [string] $Game,
    [string] $Exe = "$PSScriptRoot\..\victory\target\release\victory.exe",
    [string] $Tables = "$env:LOCALAPPDATA\VIctory\bcpack-tables.bin",
    [switch] $SeriesX
)
$tdr = Get-ItemProperty 'HKLM:\System\CurrentControlSet\Control\GraphicsDrivers' -Name TdrDelay -ErrorAction SilentlyContinue
if (-not $tdr -or $tdr.TdrDelay -lt 10) {
    Write-Warning 'Windows resets the card after 2 s of GPU work (TDR); long but correct submissions end the run. See docs/nvidia.md.'
}
$env:VICTORY_BCPACK_TABLES = $Tables
$env:VICTORY_BVH_BUILDER = '1'
$env:VICTORY_TEXTURE_NO_WAIT = '1'
$env:VICTORY_LEVELS_ON_DEMAND = '1'
$env:VICTORY_LOG = 'warn'
$env:VICTORY_CACHE_DIR = "$env:LOCALAPPDATA\VIctory\cache-commons"
$console = if ($SeriesX) { @() } else { @('--console', 'series-s') }
& $Exe run (Join-Path $Game 'game_scarlett_master.exe') @console --execute
```
