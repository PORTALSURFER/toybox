#!/usr/bin/env bash
set -euo pipefail

# Enforce toybox CLAP state defaults for plugin crates.
#
# A crate is considered compliant when each file that declares `impl Plugin for`
# both:
# 1) registers PluginState (directly or through toybox default helpers), and
# 2) has a corresponding `impl PluginStateImpl for` in the same crate's `src/`.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v rg >/dev/null 2>&1; then
  echo "[state-check] ripgrep (rg) is required" >&2
  exit 1
fi

plugin_files=()
while IFS= read -r file; do
  plugin_files+=("$file")
done < <(rg -l "^[[:space:]]*impl[[:space:]]+Plugin[[:space:]]+for[[:space:]]+" . -g '*.rs')

if [[ ${#plugin_files[@]} -eq 0 ]]; then
  echo "No plugin declarations found; nothing to check."
  exit 0
fi

failed=0

crate_root_for() {
  local path="$1"
  local dir
  dir="$(dirname "$path")"
  while [[ "$dir" != "/" && "$dir" != "." ]]; do
    if [[ -f "$dir/Cargo.toml" ]]; then
      printf '%s\n' "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done

  # Fallback to repo root for unusual layouts.
  printf '%s\n' "$ROOT"
}

for file in "${plugin_files[@]}"; do
  if ! rg -n "register::<PluginState>\(\)|register_default_extensions(_with_gui)?\(" "$file" >/dev/null; then
    echo "[state-check] missing PluginState registration in: $file"
    failed=1
  fi

  crate_root="$(crate_root_for "$file")"
  if ! rg -n "^[[:space:]]*impl[[:space:]]+PluginStateImpl[[:space:]]+for[[:space:]]+" "$crate_root/src" -g '*.rs' >/dev/null; then
    echo "[state-check] missing PluginStateImpl in crate: $crate_root (found plugin in $file)"
    failed=1
  fi
done

if [[ "$failed" -ne 0 ]]; then
  echo "[state-check] failed"
  exit 1
fi

echo "[state-check] ok"
