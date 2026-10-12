# T040 asynchronous contract Red evidence

## Scope

Added test-only regressions in `tools/normative_catalog/tests/test_async_table_contracts.py`. The tests pin the reviewed semantics for the three asynchronous request table contracts and call the existing `validate_source_table_contracts` validator with isolated in-memory copies of the oracle. No fixture bytes or validator/catalog production files were changed.

The pinned source rows are KMIP Specification v2.1 §6.1.39, Table 278, lines 31618–31636, and §6.1.41, Table 285, lines 32456–32492. Their reviewed semantics are:

| Contract | Requiredness | Actor | Direction | Scope |
|---|---|---|---|---|
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.39-ASYNC-CORRELATION-VALUE-REQUIRED` | Yes | client | client-to-server Process request payload | `client_1_0` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.41-ASYNC-CORRELATION-VALUES-OPTIONAL` | No | client | client-to-server Query Asynchronous Requests payload | `client_1_0` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.41-OPERATIONS-OPTIONAL` | No | client | client-to-server Query Asynchronous Requests payload | `client_1_0` |

The tests do not map Process's asynchronous correlation field to `KMIPKIT-REQ-SPEC-7.8-001`.

## Red run

Command:

```text
python -m unittest tools.normative_catalog.tests.test_async_table_contracts
```

Result: expected failure, exit code 1. The exact-value assertions passed. The validator regression produced 12 failed subtests (three contracts × four semantic fields): each changed `requiredness`, `actor`, `direction`, and `scope_disposition` value was accepted and returned `[]` diagnostics. The failures were the rejection assertions, not import, fixture, or setup errors.

Mutation values used: toggle `Yes`/`No`, set `actor` to `server`, set `direction` to `server-to-client response payload value`, and set `scope_disposition` to `client_1_1`. Each mutation was applied to a fresh `copy.deepcopy` of the fixture object.

`git diff --check` passed. This is Red evidence only; no validator fix was included.
