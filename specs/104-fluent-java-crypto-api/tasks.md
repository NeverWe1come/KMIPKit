# KMIPKIT-0104 Delivery Tasks

- [x] C001 Confirm Java package conventions, native handle ownership, and
  KMIPKIT-0102/0103 operation shapes and OASIS references.
- [x] C002 Record the Java facade scope, JNI copy/zeroization boundary, result
  model, exclusions, and acceptance criteria.
- [x] R001 Add failing Java consumer and JNI lifetime tests for all seven
  operation builders; record the expected Red result.
- [x] G001 Add JNI wrappers over the existing C crypto builder ABI and
  operation-specific Java builders with explicit field setters.
- [x] G002 Exercise build and one-exchange execution for each operation against
  a controlled TLS fixture; verify options, asynchronous headers, versions,
  outcomes, and no retry or poll.
- [x] D001 Update the Rust-owned public API manifest input, regenerate all
  outputs, document secret/runtime ownership in both guides, and complete
  requirement traceability.
- [ ] V001 Run Java, JNI, native, Rust, C, manifest, export, coverage, format,
  lint, security, and traceability checks; repair all failures.
- [ ] P001 Commit Red, Green, and Refactor separately; open/update the `[WIP]`
  draft PR stacked on KMIPKIT-0103 and keep it WIP until exact-head CI is green.
