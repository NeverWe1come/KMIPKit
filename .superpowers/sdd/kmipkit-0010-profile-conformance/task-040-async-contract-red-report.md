# T040 asynchronous contract Red evidence

## Scope

Added test-only regressions in `tools/normative_catalog/tests/test_async_table_contracts.py`. The tests pin the reviewed semantics for the three asynchronous request table contracts and call the existing `validate_source_table_contracts` validator with isolated copies of the oracle and catalog. No authoritative fixture bytes or validator/catalog production files were changed.

The pinned source rows are KMIP Specification v2.1 §6.1.39, Table 278, lines 31618–31636, and §6.1.41, Table 285, lines 32456–32492. Their reviewed semantics are:

| Contract | Requiredness | Actor | Direction | Scope |
|---|---|---|---|---|
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.39-ASYNC-CORRELATION-VALUE-REQUIRED` | Yes | client | client-to-server Process request payload | `client_1_0` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.41-ASYNC-CORRELATION-VALUES-OPTIONAL` | No | client | client-to-server Query Asynchronous Requests payload | `client_1_0` |
| `KMIPKIT-SOURCE-GAP-SPEC-6.1.41-OPERATIONS-OPTIONAL` | No | client | client-to-server Query Asynchronous Requests payload | `client_1_0` |

The tests do not map Process's asynchronous correlation field to `KMIPKIT-REQ-SPEC-7.8-001`.

The contract `direction` values are prose, not normalized data. The mutation matrix therefore does not substitute arbitrary text for that field. A separate crosslink test changes each linked catalog operation's direction and request-table role in a temporary catalog copy; the existing operation/table validator rejects those mutations. The remaining semantic mutation matrix expects `requiredness` and `actor` to match the independent source-review row and `scope_disposition` to match the linked operation's `scope_state`.

## Red run

Command:

```text
python -m unittest tools.normative_catalog.tests.test_async_table_contracts
```

Result: expected failure, exit code 1. Two test methods passed: exact reviewed-value assertions, and the existing catalog operation-direction/request-table crosslink check for Process and Query Asynchronous Requests. The semantic mutation matrix produced 9 failed subtests (three contracts × `requiredness`, `actor`, and `scope_disposition`): each changed value was accepted and returned `[]` diagnostics. The failures were rejection assertions, not import, fixture, or setup errors.

Mutation values used: toggle `Yes`/`No`, set `actor` to `server`, and set `scope_disposition` to `client_1_1`. Each mutation was applied to a fresh `copy.deepcopy` of the oracle object. The crosslink test mutates `direction` to `server_to_client` or changes the linked request table role to `response` in a temporary catalog fixture.

`git diff --check` passed. This is Red evidence only; no validator fix was included.
