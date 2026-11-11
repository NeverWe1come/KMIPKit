# KMIPKIT-0101 Traceability

| Requirement | Pinned source and existing contract | Verification |
|---|---|---|
| FR-001 constructors | KMIP v2.1 §§6.1.1, 6.1.4, 6.1.28, 6.1.42, 6.1.44–6.1.47; KMIPKIT-0094 operation inventory | One typed request-variant construction test per operation |
| FR-002 semantic construction and validation | Same operation sections; existing `kmipkit-protocol` request validators | Build/validate tests for required and optional fields, local invalid-input tests |
| FR-003 attributes and key groups | KMIP v2.1 §§4–5 and operation request tables; ADR-0012 and ADR-0014 | Exact AttributeSet order, filter/type/version, group placement and mask tests |
| FR-004 typed execution results | KMIP v2.1 §§8–9; KMIPKIT-0007, KMIPKIT-0093-FR-004–008 | Typed batch conversion, one exchange, Success/Pending/Failure, correlation and delivery-state tests |
| FR-005 boundaries and redaction | ADR-0012, ADR-0018, ADR-0022; KMIPKIT-0093-FR-006–010 | No-send invalid/version cases, separated option setters, payload-free formatting tests |
| FR-006 public declarations | Constitution III and KMIPKIT-0093-FR-011 | API manifest generation, exact export audit, traceability checker |
