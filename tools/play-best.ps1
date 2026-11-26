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
