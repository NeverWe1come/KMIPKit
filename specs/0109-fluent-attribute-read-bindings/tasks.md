# KMIPKIT-0109 Delivery Tasks

- [x] T001 Add failing C consumer calls for both semantic entry points.
- [x] T002 Add failing Java facade contract checks for both builders and
  entry-point methods.
- [x] T003 Add a failing Python consumer test for both builders and named
  arguments.
- [x] T004 Record Red evidence in a dedicated test commit.
- [x] T005 Implement C direct functions on the Rust execution path, require an
  identifier for standalone exchanges, and verify invalid no-ID calls plus
  valid requests through the compiled C ABI consumer and mTLS fixture.
- [x] T006 Implement Java builders and `FluentRequest` entry points; verify
  JUnit/JNI tests and existing batch consumers.
- [x] T007 Implement Python builders, `Request` entry points, and direct
  `Client` methods; verify named arguments, standalone identifier validation,
  mTLS execution, pytest, and existing CFFI consumers.
- [x] T008 Add stable requirement traceability, update the manifest, regenerate
  outputs, and update English and Spanish guides.
- [x] T009 Record Green and Refactor as separate commits; run formatting,
  Clippy, Rust/C/Java/Python suites, coverage, generation, export audit,
  traceability, and diff checks.
  - `cargo fmt --all --check`, Clippy, and the full Rust workspace suite pass.
  - The compiled C mTLS consumer passes all direct and `_with_options_v1`
    calls for both operations and confirms omitted standalone identifiers fail
    before transport. Java passes its full Gradle suite; Python passes 1,348
    tests and 119 subtests, including wheel packaging and mTLS calls.
  - `cargo llvm-cov --workspace --all-features --locked` reports 92.53% lines,
    91.48% functions, and 90.72% regions.
  - Manifest generation, T081 generator regressions, binding export audit,
    requirement traceability, and `git diff --check` pass.
- [ ] T010 Open a `[WIP]` draft PR based on KMIPKIT-0108; fix every red CI
  result on the exact head before marking it ready for human review.
