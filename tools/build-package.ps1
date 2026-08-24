param(
    [string]$Output = "",
    [string]$RackForgeRoot = ""
)

$ErrorActionPreference = "Stop"
$rfRepoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($Output)) {
    $Output = Join-Path $rfRepoRoot "artifacts\rf-5-0.1.0.rfplugin"
}
if ([string]::IsNullOrWhiteSpace($RackForgeRoot)) {
    $RackForgeRoot = Join-Path (Split-Path -Parent $rfRepoRoot) "rackforge"
}
if (-not $Output.EndsWith(".rfplugin")) {
    throw "Plugin package output must end in .rfplugin"
}
if (Test-Path -LiteralPath $Output) {
    throw "Refusing to overwrite existing package: $Output"
}
if (-not (Test-Path -LiteralPath (Join-Path $RackForgeRoot "Cargo.toml"))) {
    throw "RackForge checkout not found at $RackForgeRoot"
}

$rfOriginalPath = $env:Path
$rfMsysCompilerBin = "C:\msys64\ucrt64\bin"
if (-not (Get-Command gcc -ErrorAction SilentlyContinue) -and (Test-Path -LiteralPath $rfMsysCompilerBin)) {
    $env:Path = "$rfMsysCompilerBin;$($env:Path)"
}

Push-Location $rfRepoRoot
try {
    cargo build --locked --release -p rackforge-rf-5 --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw "RF-5 WebAssembly build failed" }

    $rfOutputParent = Split-Path -Parent $Output
    New-Item -ItemType Directory -Path $rfOutputParent -Force | Out-Null
    $rfStage = Join-Path ([System.IO.Path]::GetTempPath()) ("rf-5-package-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $rfStage | Out-Null
    try {
        Copy-Item -Path (Join-Path $rfRepoRoot "plugin\package\*") -Destination $rfStage -Recurse
        Copy-Item -LiteralPath (Join-Path $rfRepoRoot "LICENSE") -Destination $rfStage
        Copy-Item -LiteralPath (Join-Path $rfRepoRoot "NOTICE.md") -Destination $rfStage
        $rfComponent = Join-Path $rfRepoRoot "target\wasm32-unknown-unknown\release\rackforge_rf_5.wasm"
        cargo run --manifest-path (Join-Path $RackForgeRoot "Cargo.toml") --locked -p rackforge-store -- pack-wasm $rfStage $rfComponent $Output
        if ($LASTEXITCODE -ne 0) { throw "RackForge packaging failed" }
    }
    finally {
        if (Test-Path -LiteralPath $rfStage) {
            Remove-Item -LiteralPath $rfStage -Recurse -Force
        }
    }
}
finally {
    Pop-Location
    $env:Path = $rfOriginalPath
}

Write-Output "RFPLUGIN_BUILT path=$Output"
