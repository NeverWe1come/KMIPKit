# KMIPKIT-0109 Traceability

| Requirement | Normative source | Verification |
|---|---|---|
| FR-001 | KMIP v2.1 §§6.1.20–6.1.21; KMIPKIT-0093-FR-004, FR-011 | C `c_consumer_executes_both_attribute_read_entrypoints_over_mtls`; Java `attributeReadBuildersExecuteOneTypedExchangePerOperationOverMtls`; Python `test_attribute_read_builders_execute_single_mtls_exchanges_and_preserve_order` and `test_python_direct_attribute_read_methods_execute_single_mtls_exchanges` |
| FR-002 | KMIP v2.1 §6.1.20, Tables 223–225; KMIPKIT-0016-FR-006 | Ordered request checks in the C mTLS consumer, Java `attributeReadBuildersExecuteOneTypedExchangePerOperationOverMtls`, and Python builder/direct mTLS tests; duplicate rejection in each binding's focused consumer test; standalone calls without an identifier fail before transport |
| FR-003 | KMIP v2.1 §6.1.21, Tables 226–228 | C and Java request construction/transport tests; Python `test_attribute_read_builders_preserve_omitted_identifier_and_reject_duplicates`; omitted identifier is asserted in Java and Python typed batch-item construction, while standalone direct methods require an identifier |
| FR-004 | KMIPKIT-0108-FR-003; ADR-0012 | Java `attributeReadBuildersValidateAndBuildOrderedTypedBatchItems`, Python `test_attribute_read_builders_preserve_omitted_identifier_and_reject_duplicates`, C invalid-input consumer checks, and all three mTLS execution tests for options/async-indicator forwarding |
| FR-005 | KMIP v2.1 §§8–9; KMIPKIT-0108-FR-004; KMIPKIT-0007 | C, Java, and Python mTLS consumers assert exactly one exchange per operation and inspect typed Success responses; existing typed response parity suites retain Pending/Failure, correlation, and absent-field behavior without adding retries or polling |
| FR-006 | KMIPKIT-0093-FR-011; C/JNI/CFFI ownership contracts | compiled C consumer plus response release, Java builder and JNI-backed batch tests, Python full CFFI suite; invalid handles and inputs are rejected before exchange |
| FR-007 | KMIPKIT-0093-FR-011; repository API manifest policy | `python3 tools/api_manifest/generate.py --check`, `python3 tools/api_manifest/audit_binding_exports.py --check`, generated API parity, traceability, and bilingual guide updates |

## Red/Green/Refactor Evidence

Red evidence is in commit `3c86f0296`. The implementation and binding
integration tests form the Green evidence. The final Refactor commit will
record the full local checks and generated-output audit; exact-head CI remains
required before this draft can be marked ready.
