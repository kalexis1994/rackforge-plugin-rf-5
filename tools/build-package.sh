#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
rackforge_root="${RACKFORGE_ROOT:-$repo_root/../rackforge}"
output="${1:-$repo_root/artifacts/rf-5-0.1.0.rfplugin}"
component="$repo_root/target/wasm32-unknown-unknown/release/rackforge_rf_5.wasm"
stage=""
cleanup() {
  if [[ -n "$stage" && -d "$stage" ]]; then rm -rf -- "$stage"; fi
}
trap cleanup EXIT

if [[ "$output" != *.rfplugin ]]; then
  printf 'Plugin package output must end in .rfplugin\n' >&2
  exit 2
fi
if [[ -e "$output" ]]; then
  printf 'Refusing to overwrite existing package %s\n' "$output" >&2
  exit 2
fi
if [[ ! -f "$rackforge_root/Cargo.toml" ]]; then
  printf 'RackForge checkout not found at %s\n' "$rackforge_root" >&2
  exit 2
fi

cd "$repo_root"
cargo build --locked --release -p rackforge-rf-5 --target wasm32-unknown-unknown
mkdir -p "$(dirname "$output")"
stage="$(mktemp -d "${TMPDIR:-/tmp}/rf-5-package.XXXXXX")"
cp -a "$repo_root/plugin/package/." "$stage/"
install -m 0644 "$repo_root/LICENSE" "$repo_root/NOTICE.md" "$stage/"
cargo run --manifest-path "$rackforge_root/Cargo.toml" --locked -p rackforge-store -- \
  pack-wasm "$stage" "$component" "$output"

printf 'RFPLUGIN_BUILT path=%s component=%s\n' "$output" "$component"
