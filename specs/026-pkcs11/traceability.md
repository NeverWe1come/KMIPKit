# KMIPKIT-0040 Traceability: PKCS#11

Before review, link each table field and assigned catalog element to implementation and executable verification. Do not invent client normative clauses from structural table headings.

| Stable catalog element | Exact source | Contract | Planned implementation / verification |
|---|---|---|---|
| KMIPKIT-ELEM-OP-C2S-PKCS-11 | Spec v2.1 §6.1.37 | Operation request/response | pkcs11.rs; execute.rs / pkcs11_operation_tests; pkcs11_execution_tests |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PKCS-11-00000033 | §11.36 | Operation enum value | Catalog-backed registration / fake-transport operation assertion |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-26-PKCS-11-FUNCTION | §6.1.37 Tables 273–274; §7.26 | Required function Structure; Enumeration or Integer | Required, alternate-type, unknown-value, round-trip tests |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-28-PKCS-11-INTERFACE | Tables 273–274; §7.28 | Optional Interface | Absent/present tests |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-8-CORRELATION-VALUE | Tables 273–274; §7.8 | Optional exact bytes | Round trip, Pending passthrough, redaction |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-27-PKCS-11-INPUT-PARAMETERS | Table 273; §7.27 | Optional opaque input | Nested-value preservation and redaction |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-29-PKCS-11-OUTPUT-PARAMETERS | Table 274; §7.29 | Optional opaque output | Response preservation and redaction |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-30-PKCS-11-RETURN-CODE | Table 274; §7.30 | Required return-code structure | Missing/malformed and unknown-member tests |
| Table 275 result alternatives | §6.1.37 Table 275 | Server-reported Failure reasons | Shared KmipOperationResult preservation; never synthesize |

The approved spec has no separate §6.1.37 client-clause ID and no direct catalog Test Case. FR-001–FR-008 and SC-001–SC-004 are covered by protocol, fake-transport, redaction, catalog, and final-gate tests. Do not claim Profile support, a fixture pass, certification, or interoperability.
