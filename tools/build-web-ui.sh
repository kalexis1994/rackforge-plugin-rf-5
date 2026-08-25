#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
web_root="$repo_root/plugin/package/web"
input_wasm="$repo_root/target/wasm32-unknown-unknown/release/rf_5_web.wasm"

cd "$repo_root"
cargo build --locked --release --target wasm32-unknown-unknown -p rf-5-web
wasm-bindgen "$input_wasm" --out-dir "$web_root" --out-name app --target web --no-typescript
printf '\n// Generated bootstrap: all UI behavior lives in Rust WebAssembly.\n__wbg_init();\n' >> "$web_root/app.js"
printf 'RF5_WEB_UI_BUILT wasm=%s\n' "$web_root/app_bg.wasm"
