# Implementation Tasks

- [x] Verify the pinned KMIP 2.0 and 2.1 assignments and record source hashes.
- [x] Specify the bounded version-gate correction and acceptance criteria.
- [x] Red: protocol and client tests fail against the baseline: `cargo test -p kmipkit-protocol --lib version_gate_validates_tags` reports `0x0042014A` incorrectly has `introduced: V2_1`; `cargo test -p kmipkit-client --lib kmip_20_ -- --nocapture` shows Login, Logout and Delegated Login are rejected before transport (`NotSent`).
- [ ] Green: correct only the tag version span; run focused tests.
- [ ] Refactor: review comments, test helpers, traceability and formatting; run the relevant suite.
- [ ] Commit Spec, Red, Green, and Refactor in separate DCO-signed commits.
- [ ] Push the feature branch and report the remote CI result; do not open a PR until independent maintainer review.
