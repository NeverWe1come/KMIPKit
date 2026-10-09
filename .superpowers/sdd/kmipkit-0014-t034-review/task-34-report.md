# T034 review follow-up

## Scope

- Repository: `KMIPKit`
- Branch: `feature/KMIPKIT-0014-dns-worker-portability`
- Starting HEAD: `56eeca4216927bc4a3946761031e3f15d159ecc1`

This follow-up began as deterministic test coverage for the DNS fixture's existing worker and stream-clone fallbacks. The current branch also contains the T034 accepted-socket portability correction in `kmipkit-test-support`; no production KMIP transport, protocol, or public API code changed.

## Changes

- Added a retrying loopback stream-clone helper and retained `lock_network_fixture()` on every loopback worker test.
- Added a case where the accepted-stream clone succeeds, the connection worker is confirmed started, and the injected second clone fails at the response-writer site. The test proves both clone callback invocations, checks the framed DNS response, and waits for request metrics to return to zero with a peak of one.
- Added deterministic injected connection-worker and response-worker spawn-failure cases. Both require successful clones and assert the targeted spawn attempt occurred exactly once; the connection-failure case also confirms the inline fallback's response worker completed.
- Kept the existing fallback cases that tolerate an earlier OS stream-clone failure, preserving coverage of those paths.
- Appended local review evidence to T034 in `specs/014-server-generated-creation/tasks.md`. T034 remains open while cross-platform CI is pending.
- Normalized accepted DNS TCP sockets to blocking mode before cloning or dispatch, because the fixture uses `read_exact` and Darwin may preserve a listener's nonblocking mode on accepted sockets.
- Added a delayed-writer regression that marks an accepted stream nonblocking, passes it through the normalization helper, and proves the subsequent read waits for the byte.
- When a response-writer clone cannot be created, write the framed DNS reply through the owned reader socket; share the frame-writing helper with the threaded path.

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

CI run `37919458298` tested the earlier published head `56eeca4` and failed on macOS. Its unchanged framed-query fixture test returned `UnexpectedEof`; fallback tests also failed. Later root-cause analysis identified accepted-socket nonblocking-mode inheritance as the likely cause, and the current local head includes the normalization fix plus a regression test. The current local head has not yet been pushed, so that older CI run does not verify the correction. Independent QA and security reviews are complete; T034 remains open pending a fresh cross-platform CI run on the corrected head.

## Accepted-stream portability correction

The fix and its chronology are recorded in `specs/014-server-generated-creation/tasks.md` under “Accepted-stream portability root-cause correction.” In summary: behavioral Red evidence is the pre-fix macOS failure in run `37919458298`; the new helper-specific test first had a compile-time Red because the helper did not exist, and is not presented as a behavioral Red. Green `8037b7e` normalizes each accepted stream before clone or dispatch. Refactors `8cb9e13` and `fe1609a` make the normalization an owned-stream gate and remove an unnecessary lint exception. The focused delayed-read regression passed. Local Clippy, format, and diff checks passed. On this Windows host, two `transport_fixtures` tests could not bind loopback UDP sockets (error 10013, `PermissionDenied`); the remaining five passed. The independent macOS and Linux evidence for the current head is still pending.
