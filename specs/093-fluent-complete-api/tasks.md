# KMIPKIT-0093 Delivery Tasks

**Prerequisite**: the complete scope is authorized by the maintainer's
2026-11-11 instruction. Human review and merge of each PR remain mandatory.
Dependent work branches from the unmerged parent head while review is pending.

## Contract PR

- [x] C001 Compare the supplied plan with current constitution, accepted
  ADRs, pinned OASIS assignments, and implemented operation models.
- [x] C002 Record corrected scope, direction, response truth, expert insertion,
  encoding, and secret boundaries in the specification and ADR.
- [x] C003 Define the eight dependent delivery slices and PR workflow.
- [ ] C004 Run documentation, catalog, diff and CI checks on this exact head.
- [ ] C005 Remove `[WIP]` and mark ready after all CI jobs succeed.

## Catalog and code generation PR

- [ ] G001 Add a failing catalog test for all assigned operation and tag
  values, seven KMIP 3.0 codes, direction, version, and no invented codes.
- [ ] G002 Record the failed command and commit the Red evidence.
- [ ] G003 Add declarative source input and deterministic Rust/C/Java/Python
  generation, preserving current symbols and updating the manifest.
- [ ] G004 Run the focused test, generation check, export audit, and full CI;
  commit Green and Refactor separately.

## Response and expert request PRs

- [ ] R001 Fail a test that transcodes one retained response tree to TTLV hex,
  JSON and XML while preserving unknown values, order, status and ownership.
- [ ] R002 Implement shared checked transcoding and typed batch-result
  conversion; verify secret-safe diagnostics and foreign copy contracts.
- [ ] E001 Fail tests for wrong-type, reserved, misplaced, repeated and
  version-incompatible expert tags, plus accepted typed Item insertion.
- [ ] E002 Implement catalogued attribute, Vendor Attribute and positional
  insertion adapters over typed requests; verify no send on rejection.

## Rust operation objects PRs

- [ ] O001 Group every assigned operation by request/response model and
  direction. Fail a focused compile/runtime test for each missing fluent
  constructor and result adapter before implementing that family.
- [ ] O002 Add all client-to-server operation objects, explicit decisions,
  local validation, batch conversion, and one-exchange send.
- [ ] O003 Add typed listener request/response wrappers for the approved
  server-to-client operations without a client send method.
- [ ] O004 Verify every matrix row against the pinned source and exact route.

## Binding PRs

- [ ] B001 Add failing C consumer tests for opaque handle ownership,
  operation construction, typed results, direct tasks and ABI stability.
- [ ] B002 Generate and implement the C ABI on the Rust owner; run sanitizer,
  manifest and consumer checks.
- [ ] J001 Add failing Java consumer and JNI lifetime tests for generated
  operation objects, typed responses and prior package compatibility.
- [ ] J002 Implement Java facade and generation; run Java and JNI checks.
- [ ] P001 Add failing Python consumer, type-shape and CFFI ownership tests.
- [ ] P002 Implement Python facade and PEP 561 stubs; run Python checks.

## End-to-end and closeout PR

- [ ] F001 Add five progressive TLS 1.3/mTLS journeys in each language and
  assert the fixture's decoded request and encoded response.
- [ ] F002 Run all required format, Clippy, Rust/C/Java/Python, sanitizer,
  coverage, traceability and generated-export gates.
- [ ] F003 Resolve every red CI job on its own PR head. Mark each PR ready only
  after its required CI jobs are green, then request human review and merge in
  dependency order.
