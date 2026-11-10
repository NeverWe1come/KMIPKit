# Requirement traceability

| Requirement | Implementation | Verification |
|---|---|---|
| KMIPKIT-0084-FR-001 | validator config and CLI | CLI planning/preflight tests |
| KMIPKIT-0084-FR-002 | artifact installer | installed bundle integrity and provenance tests |
| KMIPKIT-0084-FR-003 | catalog dispatch | exhaustive shared case classification tests |
| KMIPKIT-0084-FR-004 | policy guard and single exchange | mutation denial, dedicated-environment and no-retry tests |
| KMIPKIT-0084-FR-005 | schema 2.0 report writer | JSON-schema, controlled rejection/redaction tests |
| KMIPKIT-0084-FR-006 | request adapters and local model verifier | high/typed equivalence and generic TTLV tests |
| KMIPKIT-0084-FR-007 | isolated Rust workflow | Rust 1.94 full validator checks and loopback evidence |
