# toybox Scripts

## `ci.sh`

Runs the canonical local toybox CI checks that mirror the Linux scope in
`.github/workflows/state-enforcement.yml`.

Usage:

```bash
./scripts/ci.sh
```

Optional VST3 checks are enabled when `VST3_SDK_DIR` is set.

## Other checks

- `check_pinned_git_deps.sh`: verifies git dependencies are pinned with `rev`.
- `check_clap_state_enforcement.sh`: validates CLAP state helper usage.
- `check_declarative_slot_api.sh`: enforces slot API naming conventions.
- `check_lints.sh`: broader lint entrypoint used by existing workflows.
