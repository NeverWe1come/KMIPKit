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

The contract `direction` values are prose, not normalized data. The mutation matrix therefore does not substitute arbitrary text for that field. Catalog-copy cases change each linked operation's structured direction, request-table role, and scope state. The operation-direction and request-table mutations are rejected with diagnostics for every affected source-table row, including each Query Asynchronous Requests contract independently. The semantic mutations expect `requiredness` and `actor` to match the independent source-review row selected by `source_gap_id`, and `scope_disposition` to match the linked operation's `scope_state`.

## Red run

Command:

```text
python -m unittest tools.normative_catalog.tests.test_async_table_contracts
```

Result: expected failure, exit code 1. Exact reviewed-value assertions and the catalog operation-direction/request-table cases passed. Twelve row-specific rejection assertions failed: nine oracle mutations (three contracts × `requiredness`, `actor`, and `scope_disposition`) and three linked catalog `scope_state` mutations (Process and both Query Asynchronous Requests rows). Each failed mutation was accepted with no diagnostic for its specific `source_table_contracts[index]` row. The failures were rejection assertions, not import, fixture, or setup errors.

Mutation values used: toggle requiredness relative to the review row's `required` value, set actor to the opposite of the review row's actor, and change `scope_disposition` relative to the linked operation's `scope_state`. Each contract mutation uses a fresh `copy.deepcopy` of the oracle object. Catalog-copy cases set operation direction to `server_to_client`, change the linked request table role to `response`, or set `scope_state` to `client_1_1`.

`git diff --check` passed. This is Red evidence only; no validator fix was included.
