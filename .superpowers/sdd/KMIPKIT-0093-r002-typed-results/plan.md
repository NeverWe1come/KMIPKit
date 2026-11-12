# KMIPKIT-0093 R002 implementation detail

## Scope and approved contract

Implement the response representation and typed-result slice in
`specs/093-fluent-complete-api/spec.md`: FR-005, FR-008, and FR-009. KMIP 2.1
§§8–9 define response batch/result semantics; KMIP Profiles v2.1 §§5.4–5.5
define XML and JSON. The checked-in owner specification remains authoritative
for operation-specific response payload shapes.

The work is a completion and integration slice over the response APIs already
present in the parent stack. Before this task, the code provides:

- `ClientBatchOutcome`, which owns the typed operation response, exposes the
  common `KmipOperationResult`, operation, response view, and Pending
  correlation value;
- `ClientBatchItemResponse`, which retains the validated `ResponseMessage`,
  exposes its typed outcome and full ordered response tree, and renders TTLV
  hex, JSON, or XML through the shared checked inspection writer;
- C, Java, and Python batch result owners with common result metadata,
  operation-specific typed response adapters, pending correlation, and
  zeroizing/caller-owned encoded response views.

This task MUST NOT add a second response owner, duplicate render path, or
fabricate operation fields. If a missing behavior is found, add the narrowest
shared adapter over the existing typed response and retained tree.

## Acceptance

1. A contract test maps the existing Rust/C/Java/Python typed batch-result and
   full-response renderer declarations to KMIPKIT-0093 FR-005/FR-008/FR-009.
2. Rust tests prove Success, Pending, and Failure retain their exact status,
   reason, typed operation response, and Pending correlation through explicit
   result access; a render error does not replace or mutate that result.
3. Retained response rendering uses one shared checked path for all encodings,
   enforces caller codec limits, preserves field order and unknown allowed
   values, and exposes payload-free errors and redacted diagnostics.
4. C, Java, and Python consumer tests prove typed-result/ownership parity and
   document that caller/runtime string copies cannot be zeroized by KMIPKit.
5. The public API manifest is updated through its generator; traceability cites
   the exact OASIS document, section and table for the response contracts.

## Red/Green/Refactor

- Red: add a focused manifest contract test for the missing KMIPKIT-0093
  mappings, run it, record the expected failure, and commit.
- Green: map the already implemented typed result and representation APIs to
  FR-005/FR-008/FR-009, add only any missing response adapter proven by a
  failing behavior test, regenerate artifacts, and commit.
- Refactor: consolidate traceability/reporting and remove redundant test setup
  without changing the public contract; rerun focused suites and commit.

### Red evidence

Commit `b22cb7b70` adds the failing response-contract test. The command
`/tmp/kmipkit-0095-py312/bin/python -B -m pytest -q
tools/api_manifest/tests/test_0093_typed_response_contract.py --tb=short`
exited 1 as expected. Eight subtests fail because the already implemented
typed result and response renderer declarations do not yet map to
KMIPKIT-0093-FR-005/008/009; two non-targeted subtests pass. The failures are
traceability gaps, not missing runtime behavior.

Commit `401c12d66` adds the C caller-copy contract regression. The same
command exited 1 with three subtest failures because the generated public C
declarations did not tell callers that copied bytes cannot be zeroized by
KMIPKit. The first attempt exposed a test field-path typo (`documentation`
lives under `c`); the test setup was corrected before recording the expected
contract failure.

## Exclusions

- No arbitrary-message or generic TTLV request submission.
- No retry, Poll, automatic follow-up, or synthetic response field.
- No raw wire-byte fidelity or promise to zeroize copies owned by C, Java, or
  Python runtimes.
- No changes to pinned upstream OASIS files or generated outputs by hand.

### Green evidence

The manifest regression test now passes: 3 tests and 11 subtests passed. The
pinned generator check reports all 12 generated outputs verified. The manifest
adds FR-005/008/009 mappings to the existing Rust and foreign typed-result and
response-render APIs; generated C and parity fixtures now include those IDs.
C renderer declarations explicitly state that copies retained by C callers
remain caller-managed and cannot be zeroized by KMIPKit. The FR-005, FR-008,
and FR-009 traceability rows cite the exact KMIP 2.1 sections, Table 417,
operation response tables, and KMIP Profiles v2.1 encoding sections.

The response implementation predates this slice in the approved 0095/0096
stack. Its Rust, C, Java, and Python behavior is being revalidated on the
rebased head before the separate Refactor evidence is recorded.

### Additional Red evidence found during full manifest review

After rebasing onto 0096 Refactor, the manifest regression test
`test_t102_response_view_mappings.py::T102ResponseViewMappingTests.test_exact_response_views_reuse_the_existing_four_language_outcomes`
failed because R002 had added FR-005 to response-view field accessors and
had not propagated FR-008 to their mapped C, Java, and Python declarations.
The new focused test
`test_operation_response_view_requirement_mapping_is_four_language_parity`
reproduces the contract error: FR-005 appears on a `ClientResponseView`
declaration, although FR-005 is reserved for typed result conversion.

### Green evidence for four-language requirement parity

The additional Red test exposed that FR-005 described result conversion but had
also been attached to `ClientResponseView` field accessors, while FR-008 was
missing from their mapped C, Java, and Python declarations. Green removes
FR-005 from those field accessors and maps FR-008 across all four declarations
for each of the 113 shared response-view entries. The existing T102 parity
suite now passes after normalizing only these separately pinned 0093 additions.
Its baseline fingerprints were stale on the unmodified 0096 parent branch; the
updated snapshots reflect that approved slice and the R002 C caller-copy note.

The focused 0093 and T102 tests pass together (8 tests). The manifest generator
reports all 12 outputs verified after regeneration. The caller-copy note remains
in C documentation; Java/Python runtime-copy limitations were already present
and are covered by their existing binding documentation and tests.

### Refactor and final verification evidence

The implementation remains a single retained response owner and a shared
checked renderer. No runtime implementation was duplicated: inspection
confirmed the typed result, owner lifetime, and TTLV/JSON/XML paths were already
implemented in the preceding 0095/0096 stack. R002 closes their 0093 contract
mapping, traceability, and C caller-copy documentation. The T102 baseline
normalization is limited to additive 0093 requirement links and this caller
copy note; the focused parity tests assert the new behavior separately.

Final validation on this branch head:

- `cargo fmt --all --check` and
  `cargo clippy --workspace --all-targets --all-features -- -D warnings` pass.
- `cargo test -p kmipkit-client --all-features`: 880 unit tests, integration
  tests, and doctests pass. `cargo test -p kmipkit-ffi --all-features`: 474
  unit tests, 56 compiled C consumer tests, and the operation-parity test pass.
- `cargo llvm-cov --workspace --all-features --summary-only --fail-under-lines 90`
  passes at 92.57% workspace line coverage. Component gates pass at 96.08% for
  the client and 90.64% for FFI.
- `mvn -q -DargLine=-Djava.io.tmpdir=<worktree>/target/java-tmp verify` passes
  all 708 Java tests (one skipped) at 94.95% line coverage. The explicit temp
  directory keeps Java child-JVM fixtures off the full `/tmp` tmpfs.
- A fresh Python 3.12 editable install built from this worktree passes the
  response-view, raw-navigation, and async-result suites: 55 tests.
- The 0093 contract and T102 response-view parity tests pass together: 8 tests.
  `generate.py --check` verifies all 12 generated outputs.
- The full C/Java/Python declaration audit reports complete exact matches for
  1,998 C, 2,390 Java, and 2,597 Python declarations. The C binary and JNI
  export audits pass.
- `git diff --check` passes. No runtime source, OASIS source, or raw-message
  submission surface changed.

The development evidence is split across Red commits `42e210c84`,
`3db2ed535`, and `f8e8394da`; Green commits `e509c6d4b` and `4079ab380`; and
this final Refactor/documentation commit.
