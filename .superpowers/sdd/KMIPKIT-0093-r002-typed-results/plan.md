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
