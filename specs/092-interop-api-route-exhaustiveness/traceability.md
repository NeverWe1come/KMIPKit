# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, route capability schema, `route_models.py`, and `coverage_matrix.py` | Source-backed route inventory binds typed attribute markers to exact elements, maps Register/Revoke request and batch-item constructors for every language, covers the Batch Error Continuation Option builder across four languages (§§8.2, 9.6, 11.5), limits batch result readers to KMIP 2.0/2.1 response Batch Count and Batch Item fields (§§8.5–8.6, 9.5, 9.7), keeps §11.34 Object Type Enumeration targets separate from §4.36 Object Type Attribute routes, prevents attribute wrappers from claiming managed-object structures, excludes checked-`Tag` constructors and Item-view accessors only when reserved/unused allocations cannot yield an Item, retains RawTag construction and decoder-rejection rows, and binds async operations to exact public executors; installed artifact probes for all four languages |
| KMIPKIT-0092-FR-002 | `coverage_matrix.py`, `validate_route_evidence.py`, and language CI follow-ups | Negative tests reject missing target/version/route/action cells, duplicate or unplanned evidence, and incorrect scenario axes; expected NotSent cases require zero transport calls and omit impossible response decoding; async request cells use per-operation senders; actual full language CI is pending |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites | Java invokes installed TTLV constructors, item creation/views, decoders, and typed attribute wrappers for 41 enum families and six bitmask catalog cases; Activation Date and Query have controlled-peer recipes in KMIP 2.0/2.1. Java still has 67,285 pending cells, including the finite Profile Name reserved range until its named values are expanded; the other languages and remaining Java values, fields, batches, async cases, and boundaries remain pending |
| KMIPKIT-0092-FR-004 | `validate_report()`, `validate_parity_collection()`, route evidence schema, language peer fixtures | Constructed and sent request trees must match; tree hashes are compared across routes and languages; batch events retain ordered operation names and peer-observed response counts; full language events are pending |
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
