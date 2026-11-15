# Implementation Tasks

- [x] Verify the pinned KMIP 2.0 and 2.1 assignments and record source hashes.
- [x] Specify the bounded version-gate correction and acceptance criteria.
- [x] Red: protocol and client tests fail against the baseline: `cargo test -p kmipkit-protocol --lib version_gate_validates_tags` reports `0x0042014A` incorrectly has `introduced: V2_1`; `cargo test -p kmipkit-client --lib kmip_20_ -- --nocapture` shows Login, Logout and Delegated Login are rejected before transport (`NotSent`).
- [x] Green: `VersionGate::tag_span` now classifies `0x420168..=0x420172` as KMIP 2.1-only. `cargo test -p kmipkit-protocol --lib version_gate_validates_tags` passes, and all three focused KMIP 2.0 client tests pass.
- [x] Refactor: checked every KMIP 2.1 tag in `0x420168..=0x420172`, corrected the Delegated Login test fixture's `Right` tag to `0x420150`, grouped response fixture fields to satisfy Clippy, and updated the stale CipherTrust profile expectation to require KMIP 1.4 plus explicit exclusion of 2.0/2.1. The complete workspace test suite, Clippy, formatting, normative traceability tests, catalog validation and generated-file checks pass.
- [x] Commit Spec, Red, Green, and Refactor in separate DCO-signed commits.
- [x] Push the feature branch. It tracks `origin/feature/KMIPKIT-0089-kmip20-session-tags`.
- [ ] Run remote CI. The repository workflow runs for a feature branch on `pull_request`; this remains pending until the reviewed branch enters that workflow.
