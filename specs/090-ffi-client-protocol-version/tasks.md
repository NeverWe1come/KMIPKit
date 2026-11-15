# Implementation Tasks

- [x] Specify the bounded C client configuration fix and acceptance criteria.
- [x] Add the mTLS regression test and confirm Red: with the production fix
  temporarily absent, the peer observes KMIP minor version 1 instead of 0.
- [x] Green: preserve the configured protocol version while constructing the
  internal Rust client configuration; confirm the targeted regression passes.
- [x] Refactor and local verification: `cargo +1.94 fmt --all -- --check`,
  `cargo +1.94 clippy -p kmipkit-ffi --all-targets --features
  coverage-c-consumer -- -D warnings`, and `cargo +1.94 test -p kmipkit-ffi
  --features coverage-c-consumer` pass (471 unit tests, 50 C API integration
  tests, and one operation-parity test). `git diff --check` passes.
- [ ] Publish the feature branch and open a `[WIP]` draft PR for human review;
  keep it in draft until its complete CI passes.
