#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
rackforge_root="${RACKFORGE_ROOT:-$repo_root/../rackforge}"
output="${1:-$repo_root/artifacts/rf-5-0.1.14.rfplugin}"
component="$repo_root/target/wasm32-unknown-unknown/release/rackforge_rf_5.wasm"
optimized_component="$repo_root/target/wasm32-unknown-unknown/release/rackforge_rf_5.optimized.wasm"
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

resolve_wasm_opt() {
  if [[ -n "${WASM_OPT:-}" ]]; then
    if [[ ! -x "$WASM_OPT" ]]; then
      printf 'WASM_OPT is not executable: %s\n' "$WASM_OPT" >&2
      return 2
    fi
    printf '%s\n' "$WASM_OPT"
    return
  fi
  if command -v wasm-opt >/dev/null 2>&1; then
    command -v wasm-opt
    return
  fi

  local version="132"
  local system architecture asset expected_sha256 executable
  system="$(uname -s)"
  architecture="$(uname -m)"
  executable="wasm-opt"
  case "$system:$architecture" in
    Linux:x86_64)
      asset="binaryen-version_${version}-x86_64-linux.tar.gz"
      expected_sha256="195ddc94f9bc89f45abdabb0b9eea86023d727ba90eac8b35b80f2544fc30572"
      ;;
    Linux:aarch64|Linux:arm64)
      asset="binaryen-version_${version}-aarch64-linux.tar.gz"
      expected_sha256="c58562417836c5d0493d89bdefc434933bdc097db641b483df86bcfa557a107f"
      ;;
    Darwin:x86_64)
      asset="binaryen-version_${version}-x86_64-macos.tar.gz"
      expected_sha256="40c3de90bb3766bd0282a895e139a6f50253dba49b4f5bb89e66faca162d832e"
      ;;
    Darwin:arm64)
      asset="binaryen-version_${version}-arm64-macos.tar.gz"
      expected_sha256="98aad827847af7ef990ed7098d885725c8e5b5aae75073403635617ae4e259aa"
      ;;
    *)
      printf 'No pinned Binaryen package for %s %s\n' "$system" "$architecture" >&2
      return 2
      ;;
  esac

  local tools_root archive binary actual_sha256 url
  tools_root="$repo_root/target/tools/binaryen"
  archive="$tools_root/$asset"
  binary="$tools_root/binaryen-version_$version/bin/$executable"
  if [[ -x "$binary" ]]; then
    printf '%s\n' "$binary"
    return
  fi

  mkdir -p "$tools_root"
  if [[ -f "$archive" ]]; then
    actual_sha256="$(sha256sum "$archive" | awk '{print $1}')"
  else
    actual_sha256=""
  fi
  if [[ "$actual_sha256" != "$expected_sha256" ]]; then
    url="https://github.com/WebAssembly/binaryen/releases/download/version_$version/$asset"
    curl --fail --location --retry 3 --output "$archive" "$url"
  fi
  actual_sha256="$(sha256sum "$archive" | awk '{print $1}')"
  if [[ "$actual_sha256" != "$expected_sha256" ]]; then
    printf 'Binaryen archive checksum mismatch: expected %s, got %s\n' "$expected_sha256" "$actual_sha256" >&2
    return 2
  fi
  tar -xzf "$archive" -C "$tools_root"
  if [[ ! -x "$binary" ]]; then
    printf 'Failed to extract the pinned Binaryen wasm-opt\n' >&2
    return 2
  fi
  printf '%s\n' "$binary"
}

cd "$repo_root"
bash "$repo_root/tools/build-web-ui.sh"
cargo build --locked --release -p rackforge-rf-5 --target wasm32-unknown-unknown
wasm_opt="$(resolve_wasm_opt)"
"$wasm_opt" "$component" -O4 -o "$optimized_component"
mkdir -p "$(dirname "$output")"
stage="$(mktemp -d "${TMPDIR:-/tmp}/rf-5-package.XXXXXX")"
cp -a "$repo_root/plugin/package/." "$stage/"
install -m 0644 "$repo_root/LICENSE" "$repo_root/NOTICE.md" "$stage/"
cargo run --manifest-path "$rackforge_root/Cargo.toml" --locked -p rackforge-store -- \
  pack-wasm "$stage" "$optimized_component" "$output"

printf 'RFPLUGIN_BUILT path=%s component=%s optimizer=binaryen-132\n' "$output" "$optimized_component"
