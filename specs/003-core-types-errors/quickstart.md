# Quickstart: Validate the Shared Error Contract

Prerequisite: KMIPKIT-0002 normative inventory and generated Result Status/Reason definitions are merged into release/1.0.0.

1. Run focused protocol tests with cargo test -p kmipkit-protocol --test result_contract.
2. Run focused transport tests with cargo test -p kmipkit-transport --test delivery_state.
3. Run focused client tests with cargo test -p kmipkit-client --test error_contract.
4. Run all workspace checks required for the PR: cargo fmt --all --check; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; cargo doc --workspace --all-features --no-deps.
5. Confirm coverage for changed code is at least 95 percent and all applicable protocol/transport/workspace thresholds remain met.

Expected outcomes:
- Known values use generated catalog names while unknown numeric values remain exact.
- Failure requires a reason and Success forbids one.
- Absent and present-empty server messages remain distinguishable and never appear in default error text or logs.
- Pre-send, transmission-started with a zero-byte read, and receipt of the first response byte report NotSent, PossiblySent, and ResponseStarted respectively.
- Safe cause categories remain accessible through the public error contract; original unsafe sources are dropped during sanitization, arbitrary source text and payloads are unreachable through the public source chain, and sentinels are absent from ResultMessage/KmipOperationResult/client-error Display/Debug and any existing generated logs.
- Delivery values make no retry-safety promise; automatic retry behavior remains governed by the client-execution feature.
