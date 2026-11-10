# KMIPKIT-0092 Traceability

| Requirement | Implementation target | Verification |
|---|---|---|
| KMIPKIT-0092-FR-001 | `interop/shared/coverage/route-capabilities.json`, coverage-matrix generator | Manifest completeness and installed-public-API checks for all four languages |
| KMIPKIT-0092-FR-002 | `interop/shared/coverage/`, matrix validator, language CI workflows | Negative tests for missing, duplicate, unavailable, and mislabeled route cells |
| KMIPKIT-0092-FR-003 | Per-language recipe and local TTLV suites | Case-by-route tests for operations, objects, attributes, enums/masks, edges, batches, async, and boundaries |
| KMIPKIT-0092-FR-004 | Shared normalization/parity fixtures and matrix comparator | Deterministic request/response tree, batch order, numeric result, and byte-equality tests |
| KMIPKIT-0092-FR-005 | Controlled KMIP peer contracts | Request/response exchange and observable setup/cleanup assertions for every supported cell |
| KMIPKIT-0092-FR-006 | Language artifact installers and result provenance | Exact commit and digest validation for same-artifact cross-language runs |
| KMIPKIT-0092-FR-007 | `validation-results.schema.json`, aggregator, renderers | Schema and multi-run matrix tests preserving every axis |
| KMIPKIT-0092-FR-008 | Report writers and redaction helpers | Secret and raw-message negative fixtures |
| KMIPKIT-0092-FR-009 | Capability manifest and validator route registries | Missing recipe must fail; only verified public API gaps or normative inapplicability may remain pending |
| KMIPKIT-0092-FR-010 | Pinned OASIS inventory, catalog generator, and normative inventory tests | Exact source-backed counts for every versioned operation, object, attribute, TTLV type, enum value, bit flag, field branch, and defined boundary/error case |
