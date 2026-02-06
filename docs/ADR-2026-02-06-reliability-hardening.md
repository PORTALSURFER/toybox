# ADR: Reliability Hardening (2026-02-06)

## Status
Accepted

## Context
The workspace already had strong CI quality gates, but several reliability risks
remained:

- Git dependencies were not pinned in manifests, so fresh resolves could drift.
- GUI automation events used an unbounded queue, which could grow without bound
  if producers outpaced drains.
- `toybox` and `patchbay-gui` carried near-identical file logger
  implementations, increasing maintenance drift risk.

## Decision

### 1. Pin git dependencies to explicit revisions
- Add `rev = "<sha>"` to each git dependency declaration.
- Add `scripts/check_pinned_git_deps.sh` and run it in CI.

### 2. Bound automation queue growth
- Add `AutomationQueueConfig` and `AutomationDropPolicy`.
- Default queue capacity is `DEFAULT_AUTOMATION_QUEUE_MAX_EVENTS = 4096`.
- Default overflow policy is `DropNewest` so old events are preserved unless
  callers explicitly choose otherwise.

### 3. Share logger implementation
- Introduce `crates/process-log` with `ProcessFileLogger`.
- Keep crate-local logging wrappers to preserve existing call sites and log
  filename prefixes.

## Consequences

### Positive
- Dependency resolution is deterministic and reviewable.
- Automation queue memory use is bounded under sustained UI load.
- Logger behavior changes are centralized and testable in one place.

### Tradeoffs
- Queue overflow now has explicit behavior that callers should monitor via
  `AutomationEnqueueStatus`.
- Dependency bumps now require explicit `rev` updates.

## Dependency Update Workflow
1. Update the desired `rev` in manifest entries.
2. Run `cargo update -p <crate>` (or `cargo update`) to refresh `Cargo.lock`.
3. Run:
   - `./scripts/check_pinned_git_deps.sh`
   - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
   - `cargo test --workspace`
4. Commit manifest and lockfile updates together.
