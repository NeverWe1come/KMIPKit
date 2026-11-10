# Implementation Tasks

- [x] Specify the bounded C client configuration fix and acceptance criteria.
- [x] Add the mTLS regression test and confirm Red: with the production fix
  temporarily absent, the peer observes KMIP minor version 1 instead of 0.
- [x] Green: preserve the configured protocol version while constructing the
  internal Rust client configuration; confirm the targeted regression passes.
- [ ] Refactor and verification: run all FFI tests, formatting, Clippy,
  traceability and CI checks; record the results before marking review-ready.
- [ ] Publish the feature branch and open a `[WIP]` draft PR for human review.
