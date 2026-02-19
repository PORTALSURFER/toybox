#!/usr/bin/env bash
set -euo pipefail

# Run full clippy checks, including the VST3 feature path where stream and ABI
# code is compiled.
#
# In this environment, use `/mnt/e/lib/vst3sdk` if `VST3_SDK_DIR` is unset.

if [[ -z "${VST3_SDK_DIR:-}" ]]; then
  export VST3_SDK_DIR=/mnt/e/lib/vst3sdk
fi

bash scripts/check_declarative_slot_api.sh

cargo clippy --workspace --all-targets --all-features -- -D warnings

VST3_SDK_DIR="$VST3_SDK_DIR" cargo clippy --features vst3 --all-targets -- -D warnings
