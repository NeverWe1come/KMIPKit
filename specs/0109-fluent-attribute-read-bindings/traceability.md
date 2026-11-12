# KMIPKIT-0109 Traceability

| Requirement | Normative source | Verification |
|---|---|---|
| FR-001 | KMIP v2.1 §§6.1.20–6.1.21; KMIPKIT-0093-FR-004, FR-011 | C/Java/Python semantic builder consumer tests |
| FR-002 | KMIP v2.1 §6.1.20, Tables 223–225; KMIPKIT-0016-FR-006 | ordered-reference and duplicate-rejection tests in all bindings |
| FR-003 | KMIP v2.1 §6.1.21, Tables 226–228 | omitted-identifier request and batch-item tests in all bindings |
| FR-004 | KMIPKIT-0108-FR-003; ADR-0012 | build, validation, options, async-indicator and batch-conversion tests |
| FR-005 | KMIP v2.1 §§8–9; KMIPKIT-0108-FR-004; KMIPKIT-0007 | controlled transport tests for one exchange, Success/Pending/Failure, correlation, and exact response fields |
| FR-006 | KMIPKIT-0093-FR-011; C/JNI/CFFI ownership contracts | compiled C consumer, Java/JNI tests, Python/CFFI tests, negative ownership cases |
| FR-007 | KMIPKIT-0093-FR-011; repository API manifest policy | generated API parity, export audit, traceability and bilingual guide checks |

## Red/Green/Refactor Evidence

Evidence will be added as the three language test, implementation, and
integration commits are completed. Every normative requirement remains mapped
to at least one executable test.
