param()

$ErrorActionPreference = "Stop"
$rf5RepoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..")).Path
$rf5InputWasm = Join-Path $rf5RepoRoot "target\wasm32-unknown-unknown\release\rf_5_web.wasm"
$rf5WebRoot = Join-Path $rf5RepoRoot "plugin\package\web"
$rf5GeneratedJs = Join-Path $rf5WebRoot "app.js"

Push-Location $rf5RepoRoot
try {
    cargo build --locked --release --target wasm32-unknown-unknown -p rf-5-web
    if ($LASTEXITCODE -ne 0) { throw "RF-5 Rust web UI build failed" }
    wasm-bindgen $rf5InputWasm --out-dir $rf5WebRoot --out-name app --target web --no-typescript
    if ($LASTEXITCODE -ne 0) { throw "RF-5 wasm-bindgen generation failed" }
    Add-Content -LiteralPath $rf5GeneratedJs -Encoding utf8 -Value "`n// Generated bootstrap: all UI behavior lives in Rust WebAssembly.`n__wbg_init();"
}
finally {
    Pop-Location
}

Write-Output "RF5_WEB_UI_BUILT wasm=$(Join-Path $rf5WebRoot 'app_bg.wasm')"
