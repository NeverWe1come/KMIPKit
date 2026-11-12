# KMIPKIT-0109 Delivery Tasks

- [x] T001 Add failing C consumer calls for both semantic entry points.
- [x] T002 Add failing Java facade contract checks for both builders and
  entry-point methods.
- [x] T003 Add a failing Python consumer test for both builders and named
  arguments.
- [x] T004 Record Red evidence in a dedicated test commit.
- [ ] T005 Implement C direct functions on the Rust execution path and verify
  the C ABI consumer and mTLS fixture.
- [ ] T006 Implement Java builders and `FluentRequest` entry points; verify
  JUnit/JNI tests and existing batch consumers.
- [ ] T007 Implement Python builders and `Request` entry points; verify pytest
  and existing CFFI consumers.
- [ ] T008 Add stable requirement traceability, update the manifest, regenerate
  outputs, and update English and Spanish guides.
- [ ] T009 Record Green and Refactor as separate commits; run formatting,
  Clippy, C/Java/Python suites, coverage, generation, export audit, and diff
  checks.
- [ ] T010 Open a `[WIP]` draft PR based on KMIPKIT-0108; fix every red CI
  result on the exact head before marking it ready for human review.
