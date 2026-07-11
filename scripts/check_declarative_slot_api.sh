#!/usr/bin/env bash
set -euo pipefail

# Enforce declarative slot API naming in framework source.
# Legacy section helper names must not be reintroduced.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

if ! command -v rg >/dev/null 2>&1; then
  echo "[slot-api] ripgrep (rg) is required" >&2
  exit 1
fi

forbidden_regex='row_sections|column_sections|weighted_section_lengths|weighted_section|fraction_section|fill_section|GridKind::Section(Column|Row)'
source_roots=(
  "patchbay-gui/src/declarative"
  "src"
)

violations="$(rg -n "${forbidden_regex}" "${source_roots[@]}" --glob '*.rs' || true)"
if [[ -n "${violations}" ]]; then
  printf '%s\n' "${violations}" >&2
  echo "[slot-api] legacy section naming is not allowed in declarative framework source" >&2
  exit 1
fi

echo "[slot-api] ok"
