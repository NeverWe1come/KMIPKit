# T034 review follow-up

## Scope

- Repository: `KMIPKit`
- Branch: `feature/KMIPKIT-0014-dns-worker-portability`
- Starting HEAD: `56eeca4216927bc4a3946761031e3f15d159ecc1`

This follow-up adds test coverage only. No production behavior or protocol code changed.

## Changes

- Added a retrying loopback stream-clone helper and retained `lock_network_fixture()` on every loopback worker test.
- Added a case where the accepted-stream clone succeeds, the connection worker is confirmed started, and the injected second clone fails at the response-writer site. The test proves both clone callback invocations, checks the framed DNS response, and waits for request metrics to return to zero with a peak of one.
- Added deterministic injected connection-worker and response-worker spawn-failure cases. Both require successful clones and assert the targeted spawn attempt occurred exactly once; the connection-failure case also confirms the inline fallback's response worker completed.
- Kept the existing fallback cases that tolerate an earlier OS stream-clone failure, preserving coverage of those paths.
- Appended local review evidence to T034 in `specs/014-server-generated-creation/tasks.md`. T034 remains open while cross-platform CI is pending.

## Baseline and TDD evidence

Before editing, the focused existing accepted-connection tests passed on the unchanged baseline (`56eeca4`). These tests did not fail because the relevant behavior already existed; the gap was that the injected spawn failures could be bypassed by an earlier OS clone failure. This follow-up adds deterministic verification and does not claim a new Red/Green/Refactor cycle or introduce a fake production edit.

## Verification

- Baseline: `cargo test --locked -p kmipkit-test-support --all-features accepted_tcp_connection_is_served` — 3 passed.
- Focused inline cases: `cargo test --locked -p kmipkit-test-support --all-features dns_response_is_written_inline` — 2 passed.
- Focused deterministic spawn cases: `cargo test --locked -p kmipkit-test-support --all-features spawn_failure_is_reached` — 2 passed.
- Full crate suite: `cargo test --locked -p kmipkit-test-support --all-features` — 34 passed across unit, integration, and documentation targets; 0 failed.
- Clippy: `cargo clippy --locked -p kmipkit-test-support --all-targets --all-features -- -D warnings` — passed.
- Formatting: `cargo fmt --all --check` — passed. `cargo fmt --all` was applied before verification.
- Whitespace: `git diff --check` — passed.

## Remaining concern

Cross-platform CI, including macOS, was not run in this local follow-up. The T034 cross-platform CI gate remains pending as recorded in the task file.
