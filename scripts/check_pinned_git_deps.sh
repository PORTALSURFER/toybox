#!/usr/bin/env bash
set -euo pipefail

# Enforce that git dependencies in Cargo manifests are pinned to explicit revs.
# This script validates inline dependency tables that include `git = "..."`
# and expects the same line to also include `rev = "..."`.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

violations=0

while IFS= read -r manifest; do
  while IFS= read -r line; do
    if [[ "${line}" != *"rev ="* ]]; then
      echo "[pinned-git-deps] missing rev pin in ${line}"
      violations=1
    fi
  done < <(rg -n 'git\s*=\s*"' "${manifest}")
done < <(rg --files -g '**/Cargo.toml')

if [[ "${violations}" -ne 0 ]]; then
  echo "[pinned-git-deps] failed"
  exit 1
fi

echo "[pinned-git-deps] ok"
