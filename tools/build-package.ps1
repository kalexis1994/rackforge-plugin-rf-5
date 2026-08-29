param(
    [string]$Output = "",
    [string]$RackForgeRoot = "",
    [string]$WasmOpt = "",
    [string]$Store = ""
)

$ErrorActionPreference = "Stop"
$rfRepoRoot = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($Output)) {
    $Output = Join-Path $rfRepoRoot "artifacts\rf-5-0.1.14.rfplugin"
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
if ([string]::IsNullOrWhiteSpace($Store)) {
    $Store = Join-Path $RackForgeRoot "target\release\rackforge-store.exe"
}
if (-not (Test-Path -LiteralPath $Store -PathType Leaf)) {
    throw "RackForge store executable not found at $Store"
}

function Resolve-RfWasmOpt {
    param([string]$ExplicitPath)

    foreach ($candidate in @($ExplicitPath, $env:WASM_OPT)) {
        if (-not [string]::IsNullOrWhiteSpace($candidate)) {
            if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
                throw "wasm-opt was not found at $candidate"
            }
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    $command = Get-Command wasm-opt -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    $binaryenVersion = "132"
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" {
            $asset = "binaryen-version_${binaryenVersion}-x86_64-windows.tar.gz"
            $expectedSha256 = "2089428ec98c899b45ee5d00636ddd6e2da8636cc473ef50b165cc25793ef7cb"
        }
        "ARM64" {
            $asset = "binaryen-version_${binaryenVersion}-arm64-windows.tar.gz"
            $expectedSha256 = "1dd7dfac7d4a6021619e6f75dd8475e528d75928d3f0b084f4068fabac694442"
        }
        default { throw "No pinned Binaryen package for Windows architecture $($env:PROCESSOR_ARCHITECTURE)" }
    }

    $toolsRoot = Join-Path $rfRepoRoot "target\tools\binaryen"
    $binary = Join-Path $toolsRoot "binaryen-version_$binaryenVersion\bin\wasm-opt.exe"
    if (Test-Path -LiteralPath $binary -PathType Leaf) {
        return $binary
    }

    New-Item -ItemType Directory -Path $toolsRoot -Force | Out-Null
    $archive = Join-Path $toolsRoot $asset
    $downloadRequired = -not (Test-Path -LiteralPath $archive -PathType Leaf)
    if (-not $downloadRequired) {
        $downloadRequired = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expectedSha256
    }
    if ($downloadRequired) {
        $url = "https://github.com/WebAssembly/binaryen/releases/download/version_$binaryenVersion/$asset"
        Invoke-WebRequest -Uri $url -OutFile $archive
    }
    $actualSha256 = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualSha256 -ne $expectedSha256) {
        throw "Binaryen archive checksum mismatch: expected $expectedSha256, got $actualSha256"
    }
    tar -xzf $archive -C $toolsRoot
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $binary -PathType Leaf)) {
        throw "Failed to extract the pinned Binaryen wasm-opt"
    }
    return $binary
}

$rfOriginalPath = $env:Path
$rfMsysCompilerBin = "C:\msys64\ucrt64\bin"
if (-not (Get-Command gcc -ErrorAction SilentlyContinue) -and (Test-Path -LiteralPath $rfMsysCompilerBin)) {
    $env:Path = "$rfMsysCompilerBin;$($env:Path)"
}

Push-Location $rfRepoRoot
try {
    & (Join-Path $rfRepoRoot "tools\build-web-ui.ps1")
    if ($LASTEXITCODE -ne 0) { throw "RF-5 web UI build failed" }

    cargo build --locked --release -p rackforge-rf-5 --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { throw "RF-5 WebAssembly build failed" }

    $rfWasmOpt = Resolve-RfWasmOpt -ExplicitPath $WasmOpt
    $rfComponent = Join-Path $rfRepoRoot "target\wasm32-unknown-unknown\release\rackforge_rf_5.wasm"
    $rfOptimizedComponent = Join-Path $rfRepoRoot "target\wasm32-unknown-unknown\release\rackforge_rf_5.optimized.wasm"
    & $rfWasmOpt $rfComponent -O4 -o $rfOptimizedComponent
    if ($LASTEXITCODE -ne 0) { throw "Binaryen wasm-opt failed" }

    $rfOutputParent = Split-Path -Parent $Output
    New-Item -ItemType Directory -Path $rfOutputParent -Force | Out-Null
    $rfStage = Join-Path ([System.IO.Path]::GetTempPath()) ("rf-5-package-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $rfStage | Out-Null
    try {
        Copy-Item -Path (Join-Path $rfRepoRoot "plugin\package\*") -Destination $rfStage -Recurse
        Copy-Item -LiteralPath (Join-Path $rfRepoRoot "LICENSE") -Destination $rfStage
        Copy-Item -LiteralPath (Join-Path $rfRepoRoot "NOTICE.md") -Destination $rfStage
        & $Store pack-wasm $rfStage $rfOptimizedComponent $Output
        if ($LASTEXITCODE -ne 0) { throw "RackForge packaging failed" }
    }
    finally {
        if (Test-Path -LiteralPath $rfStage) {
            $rfTempRoot = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
            $rfResolvedStage = [System.IO.Path]::GetFullPath($rfStage)
            if (-not $rfResolvedStage.StartsWith($rfTempRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
                throw "Refusing to remove package stage outside the temporary directory: $rfResolvedStage"
            }
            Remove-Item -LiteralPath $rfStage -Recurse -Force
        }
    }
}
finally {
    Pop-Location
    $env:Path = $rfOriginalPath
}

Write-Output "RFPLUGIN_BUILT path=$Output component=$rfOptimizedComponent optimizer=binaryen-132"
