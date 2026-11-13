# Tasks: Configurable Discover Versions Offers

Implementation is blocked pending approval of `spec.md`.

## Before implementation

- [ ] Record maintainer approval in this feature directory.
- [ ] Rebase/confirm the branch starts from the current approved `release/1.0.0` commit.

## Protocol and client

- [ ] Add Red tests for default `(2,1)`, explicit `[2.1,2.0]`, explicit `[2.0]`, omitted field, invalid offered values, and response intersection behavior.
- [ ] Implement the three distinct request states in `DiscoverVersionsRequest` and preserve the existing no-argument behavior.
- [ ] Expose explicit and omitted typed client batch constructors.
- [ ] Validate successful responses conditionally against an explicit offer list; preserve well-formed raw versions when offers are omitted.
- [ ] Add Refactor commit and public documentation.

## FFI and bindings

- [ ] Add C ABI Red tests compiled and called from C for all request states and invalid pointer/count inputs.
- [ ] Add Java 17 external-package Red tests for all request states and matching response behavior.
- [ ] Add Python 3.12 external-package Red tests for all request states and matching response behavior.
- [ ] Add FFI/JNI/Python implementation and verify ownership, lifetimes, and error mapping.
- [ ] Regenerate C headers, API manifests, wrappers, and parity fixtures with pinned repository generators.

## Cross-language verification and review

- [ ] Compare normalized request/response trees across four languages.
- [ ] Compare deterministic TTLV bytes and independently selected 2.0/2.1 headers.
- [ ] Run protocol/client tests, C ABI tests, Java/Python suites, formatting, lint, coverage, generated checks, and full exact-head CI.
- [ ] Update user documentation, requirement traceability, and the interop Discover Versions cases.
- [ ] Open/maintain a draft `[WIP]` PR and keep it WIP until all required CI is green; maintainer approval and merge remain human-only.
