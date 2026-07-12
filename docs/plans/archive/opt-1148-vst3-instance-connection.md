# OPT-1148 VST3 Instance Connection

## Scope

- Add reusable Toybox infrastructure for identity-safe processor/controller shared state.
- Use the host-provided VST3 `IConnectionPoint` pairing and standard `IMessage` channel rather than plugin creation order or direct-peer interface assumptions.
- Cover shuffled creation and destruction, reconnects, and simultaneous independent instances.

## Definition Of Done

- A host-connected controller adopts only the shared state of its exact processor peer.
- Either `connect` callback direction can establish the shared state.
- Hosts may interpose an `IConnectionPoint` proxy that exposes no Toybox private interface.
- Unsupported private bridge queries return COM-correct `kNoInterface` with a null output pointer.
- Shared-state compatibility uses concrete Rust `TypeId`, not diagnostic type-name strings.
- Message attributes own exported handles while receivers borrow them, preventing rejected offers from double-freeing state.
- Disconnect and destruction do not retain COM peers or create lifetime cycles.
- Focused VST3 tests and the full local Toybox validation lane pass.

## Integration

- Plugin processor and controller classes add `IConnectionPoint` and Toybox private shared-state
  bridge to their COM interface lists; state transfer itself uses `IMessage`, so host proxies are
  tolerated.
- Each class owns an `InstanceConnection<T>` with the appropriate processor/controller role.
- Plugin code obtains the current `Arc<T>` through the connection rather than a global registry.

## Validation Result

- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo test --features vst3 vst3::connection::tests -- --nocapture`
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk cargo clippy --features vst3 --all-targets -- -D warnings`
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/run_agent_request.sh`
- `VST3_SDK_DIR=/Users/portalsurfer/lib/vst3sdk bash scripts/ci_local.sh`

All checks pass. The first preflight without `VST3_SDK_DIR` failed at the expected SDK gate; the
same preflight with the repository local SDK path passes. The VST3 feature lane runs 97 tests,
including both callback directions through a proxy exposing only `IConnectionPoint`, exact
unsupported-interface result semantics, concrete-type mismatch rejection, and exactly-once
`Arc` release for direct adoption and rejected offers.

## Outcome

User review signed off Toybox PR #4. The final implementation head `ce7f8bf` passed Linux and
Windows CI, and the merge/cleanup procedure was authorized. Plugin repositories can now adopt the
canonical merged Toybox revision.

