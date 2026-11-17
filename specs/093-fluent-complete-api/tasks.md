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
- [x] C004 Run documentation, catalog, diff and CI checks on this exact head;
  repository and CI validation passed for PR #175.
- [x] C005 Remove `[WIP]` and mark ready after all CI jobs succeed; PR #175 is
  ready for human review while downstream implementation continues.

## Catalog and code generation PR

- [x] G001 Add a failing catalog test for all assigned operation and tag
  values, seven KMIP 3.0 codes, direction, version, and no invented codes.
- [x] G002 Record the failed command and commit the Red evidence.
- [x] G003 Add declarative source input and deterministic Rust/C/Java/Python
  generation, preserving current symbols and updating the manifest.
- [x] G004 Run the focused test, generation check, export audit, and full CI;
  commit Green and Refactor separately.

## FR-003 operation-matrix completion follow-up

- [x] M001 Add failing matrix tests requiring each route to declare typed
  request/response owners and advance the generated matrix format.
- [x] M002 Record Red evidence in commit `097bcc859`; the focused test failed
  because operation rows had no typed owner fields.
- [x] M003 Add the reviewed per-route owner and test-evidence catalog; derive
  and validate earliest supported versions against the versioned assignments.
- [x] M004 Regenerate the matrix and pass focused matrix and API parity tests.
- [x] M005 Run the complete required CI on the WIP PR head before requesting
  human review.

## Response and expert request PRs

- [x] R001 Fail a test that transcodes one retained response tree to TTLV hex,
  JSON and XML while preserving unknown values, order, status and ownership.
- [x] R002 Implement shared checked transcoding and typed batch-result
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

### Rust Ping builder slice

- [x] PING001 Commit and reproduce Red tests for the fluent Ping constructor,
  typed build/batch conversion, Success, Pending, and KMIP 2.0 no-send gate.
- [x] PING002 Implement the Rust builder over the existing typed Ping request
  and one-exchange client path; record separate Green and Refactor commits.
- [x] PING003 Reconcile compiler-observed exports through repository tooling,
  update bilingual guides and traceability, and pass focused/local checks.
- [ ] PING004 Push a WIP draft PR and fix every exact-head CI failure. Keep
  the slice WIP until required C, Java, and Python fluent parity is complete.
- [x] PING-B001 Add failing C ABI consumer, Java facade, Python fluent, and
  manifest tests for typed Ping construction and one-exchange execution.
- [x] PING-B002 Implement the C direct helper and opaque builder plus Java and
  Python Ping facades over the existing typed Ping route; update generated API
  declarations and bilingual user guidance.
- [ ] PING-B003 Push the binding slice as a WIP child of the Rust Ping PR and
  fix every exact-head CI failure before marking the stack ready for review.

- [ ] B001 Add failing C consumer tests for opaque handle ownership,
  operation construction, typed results, direct tasks and ABI stability.
- [ ] B002 Generate and implement the C ABI on the Rust owner; run sanitizer,
  manifest and consumer checks.
- [ ] J001 Add failing Java consumer and JNI lifetime tests for generated
  operation objects, typed responses and prior package compatibility.
- [ ] J002 Implement Java facade and generation; run Java and JNI checks.
- [ ] P001 Add failing Python consumer, type-shape and CFFI ownership tests.
- [ ] P002 Implement Python facade and PEP 561 stubs; run Python checks.

### Create Key Pair parity slice

- [x] KPB001 Commit Red consumer and manifest checks for the C, Java, and
  Python Create Key Pair builders.
- [x] KPB002 Implement the C ABI, JNI facade, and Python CFFI facade on the
  existing typed Rust request path; verify ownership and local validation.
- [x] KPB003 Declare exact Rust/C/Java/Python exports, regenerate API outputs,
  and update English and Spanish user guides and traceability evidence.
- [x] KPB004 Push the completed implementation and resolve every required CI
  failure on the exact PR head before marking it ready for human review.

### Add Attribute parity slice

- [x] AAB001 Commit failing Rust, C consumer, Java consumer, Python consumer,
  and public-manifest checks for one typed Add Attribute request.
- [x] AAB002 Implement the Rust builder and C ABI on the existing typed
  request path; return owned typed requests and batch items and preserve the
  existing client error and delivery-state behavior.
- [x] AAB003 Add Java/JNI and Python/CFFI builders, update the generated public
  API inventory, and document the KMIP v2.1 §6.1.2 request in English and
  Spanish.
- [x] AAB004 Run full binding, coverage, traceability, generation, and exact
  head CI checks; resolve failures before review.

### Delete Attribute parity slice

- [x] DAB001 Commit failing C, Java, Python, and public-manifest checks for a
  typed Delete Attribute builder.
- [x] DAB002 Implement the Rust-backed C ABI, JNI, and CFFI builders for
  catalogued values, explicit TTLV Items, vendor attributes, and references.
- [x] DAB003 Declare and generate exact public exports; update the English and
  Spanish guides and traceability for KMIP v2.1 §6.1.13.
- [x] DAB004 Run the complete required checks and resolve exact-head CI before
  marking the PR ready for human review.

### Locate and Revoke parity slice

- [x] LRB001 Commit failing C, Java, Python, Rust option-boundary, and
  public-manifest checks for typed Locate and Revoke builders.
- [x] LRB002 Implement Rust builders and opaque C ABI builders over the
  existing typed requests, retaining validation, options, and delivery state.
- [x] LRB003 Add Java and Python facades, declare and generate exact public
  exports, and update English/Spanish guides and traceability for KMIP v2.1
  §§6.1.28 and 6.1.44.
- [x] LRB004 Run binding, coverage, traceability, generation, and exact-head CI
  checks; resolve failures before requesting human review.

## End-to-end and closeout PR

- [x] F001 Add five progressive TLS 1.3/mTLS journeys in each language and
  assert the fixture's decoded request and encoded response.
  Progress: symmetric key, AES-GCM, key-pair/signing, and expert Vendor
  Attribute, and ordered batch ID Placeholder journeys pass in all four
  consumers.
- [x] F002 Run all required format, Clippy, Rust/C/Java/Python, sanitizer,
  coverage, traceability and generated-export gates.
- [x] F003 Resolve every red CI job on its own PR head. Mark this PR ready only
  after its required CI jobs are green, then request human review; the human
  maintainer merges in dependency order.
