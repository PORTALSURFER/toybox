# toybox Scripts

## `ci.sh`

Runs the canonical local toybox CI checks that mirror the Linux scope in
`.github/workflows/state-enforcement.yml`.

Usage:

```bash
./scripts/ci.sh
```

Optional VST3 checks are enabled when `VST3_SDK_DIR` is set.

## `ci_local.sh`

Canonical local validation entrypoint for agent handoff and manual checks.
Currently delegates to `ci.sh`.

Usage:

```bash
./scripts/ci_local.sh
```

## `run_agent_request.sh`

Wake-up preflight entrypoint. Verifies required handoff files exist and then
runs `ci_local.sh`.

Usage:

```bash
./scripts/run_agent_request.sh
```

## Other checks

- `check_pinned_git_deps.sh`: verifies git dependencies are pinned with `rev`.
- `check_clap_state_enforcement.sh`: validates CLAP state helper usage.
- `check_declarative_slot_api.sh`: enforces slot API naming conventions.
- `check_lints.sh`: broader lint entrypoint used by existing workflows.
