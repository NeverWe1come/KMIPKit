# Requirement Traceability

Implementation paths and verification entries are planned for this delivery.
Replace planned test names with exact tests and record actual evidence as work
is implemented. OASIS references resolve to the immutable local KMIP v2.1
source under `specification/oasis/kmip-2.1/upstream/`.

| Requirement | Normative / project source | Planned implementation | Planned verification |
| --- | --- | --- | --- |
| KMIPKIT-0092-FR-001 | KMIP 2.1 §§6.1.9, 6.1.10, 6.1.43; KMIPKIT-0007, -0014, -0022, -0052 | Rust fluent request builders and existing typed operation routes | Builder `build`/batch conversion/execution tests; controlled TLS fixture journeys in Rust/C/Java/Python |
| KMIPKIT-0092-FR-002 | KMIP 2.1 §6.1.9 Tables 189–192; KMIPKIT-0014-FR-003, -004 | Create Key Pair builder and AttributeSet adaptation | Group/presence/order tests; Table 191 precedence and mismatch rejection tests |
| KMIPKIT-0092-FR-003 | KMIP 2.1 §§4.13–4.14 Tables 53–56, §6.1.9 Tables 189–191; KMIPKIT-0047, -0069 | Named preset definitions in Rust fluent construction | Assert exact algorithm/length/curve output for RSA 3072, ECDSA P-256, ECDH P-256; usage masks remain caller supplied; version rejection before send |
| KMIPKIT-0092-FR-004 | KMIP 2.1 §6.1.10 Tables 193–195 and §2.8 Table 9; KMIPKIT-0014-FR-005, -015 | Create Split Key fluent builder and existing response accessors | Required/missing field tests; Prime Field Size policy tests; fixture verifies exact request and repeated response ID order |
| KMIPKIT-0092-FR-005 | KMIP 2.1 §§2.1–2.9, 3.1–3.12, §6.1.43 Tables 291–294; KMIPKIT-0022, -0049-FR-001–006 | Register builder over `ManagedObject`, `KeyMaterial`, and existing Register validation | Tests cover all nine known object variants, supported Byte String/Structure formats, object/type mismatch, conditional attributes, and fixture request |
| KMIPKIT-0092-FR-006 | KMIP 2.1 §§2–3; KMIPKIT-0049-FR-007, -008; KMIPKIT-0053-FR-010 | Rust secret containers and C/JNI/CFFI ownership adapters | Redacted formatting/errors, owned-memory release/zeroization, foreign runtime copy documentation and boundary tests |
| KMIPKIT-0092-FR-007 | Constitution III; KMIPKIT-0053-FR-011; KMIPKIT-0091-FR-009 | `kmipkit-ffi` handles/functions and generated C declarations | Compiled C consumer, versioned input validation, stable status, null/count, release, and error-snapshot tests |
| KMIPKIT-0092-FR-008 | KMIPKIT-0053-FR-009, -010, -013 | Java `org.kmipkit.client`/`fluent`; Python `kmipkit.fluent`/Client facade | Consumer API shape, package compatibility, native lifetime, and shared fixture tests |
| KMIPKIT-0092-FR-009 | Constitution III; ADR-0014; KMIPKIT-0053-FR-001, -009 | Rust implementation and adapters through existing C ABI | Four-language parity cases and generated API/export audit |
| KMIPKIT-0092-FR-010 | ADR-0014; KMIPKIT-0007, -0051; KMIPKIT-0091-FR-013 | Existing typed request and batch execution boundary | Compile-time/public surface audit and negative route tests; no raw Request Message transport entry point |
| KMIPKIT-0092-FR-011 | KMIP 2.1 §§6.1.9, 6.1.10, 6.1.43; KMIPKIT-0007, -0052 | Existing outcome and delivery-state conversion | Fixture Success/Pending/Failure tests, correlation check, delivery-state assertion, and request-count assertion |
| KMIPKIT-0092-FR-012 | ADR-0022; KMIPKIT-0069; KMIP 2.1 operation tables | Existing version gate and versioned API routes | KMIP 2.1 default assertions; unsupported operation/object/preset version cases fail NotSent |
| KMIPKIT-0092-FR-013 | KMIPKIT-0053-FR-012, -013; API manifest generation contract | Generated manifest, C/JNI/CFFI declarations, English/Spanish guides | Existing consumer compatibility; generator check and export audit; guide/example checks |
| KMIPKIT-0092-FR-014 | Constitution I–II; KMIPKIT-0014, -0022, -0049 and normative catalog | This traceability file and executable test references | Catalog/traceability validation; confirm each applicable client MUST/SHALL and operation-table row is linked; no server/profile claim |
| KMIPKIT-0092-FR-015 | KMIP 2.1 §§4, 6.1.9–6.1.10, 6.1.43, 11.25, 11.36, 11.45, 11.56, 12.1; KMIPKIT-0047, -0048, -0049 | Semantic fluent enum/value wrappers and typed object constructors | Consumer examples use named values only; explicit expert raw values remain separate; no numeric-range inference |

The KMIP specification clauses above define client request/result behavior
only. Server-side generation, key splitting, registration policy, and acceptance
are not attributed to KMIPKit.
