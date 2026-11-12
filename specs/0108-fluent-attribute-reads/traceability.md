# KMIPKIT-0108 Traceability

| Requirement | Normative source | Verification |
|---|---|---|
| FR-001 | KMIP v2.1 §6.1.20, Tables 223–224; KMIPKIT-0016-FR-006 | Builder field/order and duplicate-reference tests; controlled typed response test |
| FR-002 | KMIP v2.1 §6.1.21, Tables 226–227 | Omitted identifier and response reference-order tests |
| FR-003 | KMIPKIT-0093-FR-004–005; ADR-0012 | Local validation, typed request and batch-variant tests; no-send failure test |
| FR-004 | KMIP v2.1 §§8–9; KMIPKIT-0093-FR-005, FR-008; KMIPKIT-0007 | One-exchange fake transport; typed status, response fields, and delivery-state assertions |
| FR-005 | KMIPKIT-0093-FR-011; repository public API manifest policy | Rust docs, bilingual guides, generated manifest/parity checks |
