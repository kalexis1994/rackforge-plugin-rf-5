param(
    [switch]$IncludeBuildCache
)

$ErrorActionPreference = "Stop"
$rfRepoRoot = (Resolve-Path -LiteralPath (Split-Path -Parent $PSScriptRoot)).Path
$rfCanonicalPackage = Join-Path $rfRepoRoot "artifacts\rf-5-0.1.13.rfplugin"
$rfCanonicalAuditions = Join-Path $rfRepoRoot "artifacts\auditions"

function Remove-RfGeneratedPath {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return
    }

    $rfResolved = (Resolve-Path -LiteralPath $Path).Path
    $rfRequiredPrefix = $rfRepoRoot.TrimEnd('\') + '\'
    if (-not $rfResolved.StartsWith($rfRequiredPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove a path outside RF-5: $rfResolved"
    }
    if ($rfResolved -eq $rfRepoRoot) {
        throw "Refusing to remove the RF-5 repository root"
    }

    Remove-Item -LiteralPath $rfResolved -Recurse -Force
    Write-Output "RF5_GENERATED_REMOVED path=$rfResolved"
}

$rfArtifacts = Join-Path $rfRepoRoot "artifacts"
if (Test-Path -LiteralPath $rfArtifacts) {
    Get-ChildItem -LiteralPath $rfArtifacts -Force | ForEach-Object {
        if ($_.FullName -ne $rfCanonicalPackage -and $_.FullName -ne $rfCanonicalAuditions) {
            Remove-RfGeneratedPath -Path $_.FullName
        }
    }
}

Remove-RfGeneratedPath -Path (Join-Path $rfRepoRoot "tmp")

if ($IncludeBuildCache) {
    Remove-RfGeneratedPath -Path (Join-Path $rfRepoRoot "target")
}
