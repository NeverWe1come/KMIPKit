# T038 — Rust quickstart integration coverage report

**Status:** Complete
**Base:** `d3ecbb8adaf3bd3905ac4cc430587f73e61f9870`
**Scope:** Test-only execution coverage for the approved Get and Locate Rust quickstart.

## Result

Added coverage to `crates/kmipkit-client/tests/operation_boundary.rs`. The test reuses the existing verified TLS 1.3 loopback peer and creates a public `Client` from the documented client and transport configuration. It exercises the empty-Attributes/explicit-zero Locate builder, the Get Unique Identifier builder, the quickstart `retrieve` function shape, typed `ClientBatch` execution, completed Get and Locate response accessors, and Locate identifiers in the order sent by the fixture.

The Get response exposes Unique Identifier `object-123`; the test inspects only the Any Object tag (`0x42008F`) and never formats or asserts on its contents. The Locate request is checked on the wire for an empty Attributes Structure and explicit Offset Items zero. The Locate response returns `located-b` followed by `located-a`, and the documented shared response path retains that order.

The fixture is deterministic and local. It makes no external KMIP server, interoperability, OASIS test-case, profile, or conformance claim. No production code, public API, dependency, generated artifact, or normative traceability requirement changed.

## Red/Green evidence

### Red

**Commit:** `c45586d5adbe2a27e6d7e306fd75951d872a64a6` — `test(client): add red get locate quickstart coverage`
**DCO:** Signed off by `NeverWe1come <vgarciaaguilar97@gmail.com>`.

Commands and results:

- `cargo test -p kmipkit-client --test operation_boundary rust_quickstart_get_and_locate_examples_execute_through_the_client -- --exact` — failed as expected, 0 passed / 1 failed. The new test reached the public client and rejected the existing three-item error/empty-result fixture where the quickstart requires a completed one-item Get. The reported `ProtocolResponse` was sanitized and contained no response payload.
- `cargo fmt --all` — passed.
- `git diff --check` — passed.

### Green

**Commit:** `7d4f26394f03ffa5a9202f69106ffc98759de9bd` — `test(client): exercise managed object quickstart`
**DCO:** Signed off by `NeverWe1come <vgarciaaguilar97@gmail.com>`.

Commands and results:

- `cargo fmt --all` — passed.
- `cargo test -p kmipkit-client --test operation_boundary rust_quickstart_get_and_locate_examples_execute_through_the_client -- --exact` — passed, 1/1.
- `cargo test -p kmipkit-client --test operation_boundary` — passed, 2/2.
- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.

During Green fixture development, an initial successful-response fixture included a Unique Batch Item ID although the quickstart request omitted one; the client correctly rejected the mismatch. The final response fixture omits that ID and the test passes. The fixture contains no secret/object payload bytes.

## Changed files

- `crates/kmipkit-client/tests/operation_boundary.rs` — executable quickstart builder, request/response path, order, and wire-presence assertions with successful local Get/Locate TTLV response fixtures.
- `specs/017-managed-object-retrieval/tasks.md` — marks T038 complete and records its evidence.
- `.superpowers/sdd/kmipkit-0017-managed-object-retrieval/task-38-report.md` — this report.

## Self-review

- All examples and response paths named in the quickstart are exercised through public client APIs; the test does not use internal test-only constructors.
- The TLS loopback peer gives deterministic network behavior while exercising the actual client execution boundary.
- Get object handling is limited to obtaining and comparing the object tag; the object itself is never debug-formatted, printed, or compared.
- Locate results are neither sorted nor deduplicated by the test; the assertion matches the controlled server order.
- No additional refactor was needed. The repository-wide coverage gate remains assigned to T043 and was not run for this task.
