#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
rackforge_root="${RACKFORGE_ROOT:-$repo_root/../rackforge}"
output="${1:-$repo_root/artifacts/rf-5-0.1.0-linux-aarch64.rfplugin}"
library="$repo_root/target/release/librackforge_rf_5.so"
stage=""

cleanup() {
  if [[ -n "$stage" && -d "$stage" ]]; then rm -rf -- "$stage"; fi
}
trap cleanup EXIT

if [[ "$(uname -s)" != "Linux" || "$(uname -m)" != "aarch64" ]]; then
  printf 'This builder must run on Linux aarch64\n' >&2
  exit 2
fi
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
bash "$repo_root/tools/build-web-ui.sh"
cargo build --locked --release -p rackforge-rf-5 --features realtime-1x

stage="$(mktemp -d "${TMPDIR:-/tmp}/rf-5-native-package.XXXXXX")"
cp -a "$repo_root/plugin/package/." "$stage/"
rm -f -- "$stage/component.wasm"
install -m 0644 "$repo_root/plugin/native/rackforge-plugin.toml" "$stage/rackforge-plugin.toml"
install -d "$stage/lib"
install -m 0755 "$library" "$stage/lib/librackforge_rf_5.so"
install -m 0644 "$repo_root/LICENSE" "$repo_root/NOTICE.md" "$stage/"
mkdir -p "$(dirname "$output")"
cargo run --manifest-path "$rackforge_root/Cargo.toml" --locked -p rackforge-store -- \
  pack "$stage" "$output"

printf 'RFPLUGIN_NATIVE_BUILT path=%s library=%s\n' "$output" "$library"
