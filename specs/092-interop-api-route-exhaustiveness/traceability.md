# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, `coverage_matrix.py`, and row schema | Source-backed route inventory and exhaustive per-target/per-version rows; installed artifact probes for all four languages |
| KMIPKIT-0092-FR-002 | `coverage_matrix.py`, `validate_route_evidence.py`, and language CI follow-ups | Negative tests reject missing target/version/route/action cells, duplicate or unplanned evidence, and incorrect scenario axes; actual language CI is pending |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites | Pending until each language executes all applicable rows, including values, fields, batches, async cases, and boundaries |
| KMIPKIT-0092-FR-004 | `validate_parity_collection()`, route evidence schema, language peer fixtures | Tree hashes are compared across routes and languages; batch events retain ordered operation names and peer-observed response counts; actual language events are pending |
| KMIPKIT-0092-FR-005 | Controlled KMIP peer contracts in each language validator | Pending until every supported network row is exercised and the peer checks request/response semantics and cleanup |
| KMIPKIT-0092-FR-006 | Language artifact installers, route manifest, and evidence validator | Exact release commit and per-language installed artifact digest are checked for all four artifacts; comparable runtime reports are pending |
| KMIPKIT-0092-FR-007 | `validation-results.schema.json`, route evidence schema, aggregator, parity validator | Per-run output retains transport, encoding, API, construction route, execution route, and artifact axes; tests cover same-server/config comparisons |
| KMIPKIT-0092-FR-008 | Report writers and redaction helpers | Secret and raw-message negative fixtures |
| KMIPKIT-0092-FR-009 | Capability manifest and per-language validator route registries | Common matrix fails missing target routes unless the exact public API absence is source-backed; per-language recipe gates are pending |
| KMIPKIT-0092-FR-010 | Pinned OASIS inventory, catalog generator, and normative inventory tests | Source-backed inventory tests cover versioned operations, objects, attributes, TTLV types, enum values, flags, field branches, and defined boundary/error classes |

The common contract establishes planned obligations, schema behavior, and
artifact availability. It does not claim that any validator has executed the
full matrix. FR-003, FR-005, and the runtime portion of FR-006 remain open until
the four independent language branches emit evidence from actual public API
calls and controlled-peer exchanges.
