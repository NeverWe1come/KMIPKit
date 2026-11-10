# Requirement Traceability

Implementation paths and verification entries below are planned for this
delivery. Update them to exact symbols and test names as each change lands.

| Requirement | Normative / project source | Planned implementation | Planned verification |
|---|---|---|---|
| KMIPKIT-0091-FR-001 | KMIP 2.1 §§6.1.13, 6.1.15, 6.1.19; KMIPKit-0007, -0052 | `crates/kmipkit-client/src/` typed request builders | Rust Create/Get/Destroy execution and batch conversion tests |
| KMIPKIT-0091-FR-002 | KMIP 2.1 §6.1.13; KMIP 2.1 §11.56; KMIPKit-0047 | `crates/kmipkit-protocol/src/` Create request builder | Required-field, preset, and usage-policy validation tests |
| KMIPKIT-0091-FR-003 | KMIP 2.1 §7.34; KMIPKit-0048 | Rust semantic Name conversion; C/Java/Python adapters | Encoded Name Type assertion in four-language fixture tests |
| KMIPKIT-0091-FR-004 | KMIP 2.1 §§4, 7.1; KMIPKit-0048, -0051 | Rust attribute accumulator and existing `AttributeSet` validation | Catalogued conversion, repeated field, wrong-type, and pre-send tests |
| KMIPKIT-0091-FR-005 | KMIP 2.1 §§4, 7.1, 4.60, 11.56; KMIPKit-0048 | Rust explicit Item and Vendor Attribute construction | Unknown tag/type and Vendor Attribute structure/order tests |
| KMIPKIT-0091-FR-006 | KMIP 2.1 §§6.1.15, 6.1.19; KMIPKit-0049 | Typed Get and Destroy builders in `crates/kmipkit-client/src/` | Missing/invalid identifier and no-send tests |
| KMIPKIT-0091-FR-007 | KMIP 2.1 §6.1; KMIPKit-0007, -0052 | Existing typed response/outcome path | Success/Pending/Failure and correlation propagation tests; assert no Poll/retry |
| KMIPKIT-0091-FR-008 | KMIPKit-0007 and -0013 | Existing `RequestOptions` and batch-header APIs | Exchange-local timeout tests; authentication/header boundary tests |
| KMIPKIT-0091-FR-009 | Project constitution III; KMIPKit-0053 | `crates/kmipkit-ffi/src/`, `bindings/c/include/` | Compiled C consumer, ABI, ownership, status, and release tests |
| KMIPKIT-0091-FR-010 | KMIPKit-0053 | `bindings/java/src/main/java/org/kmipkit/client/` and `fluent/` | Java/JNI consumer, compatibility, and native lifetime tests |
| KMIPKIT-0091-FR-011 | KMIPKit-0053 | `bindings/python/src/kmipkit/fluent.py` and Client facade | Python consumer, compatibility, typing, and lifetime tests |
| KMIPKIT-0091-FR-012 | Project constitution III; ADR-0014 | Rust protocol/client core and existing FFI adapters | Four-language parity and manifest/export audit |
| KMIPKIT-0091-FR-013 | ADR-0014; KMIPKit-0007, -0051 | Existing typed request and batch execution boundaries | Boundary tests reject arbitrary Request Message/raw-byte routes |
| KMIPKIT-0091-FR-014 | KMIP 2.1 §11.56; ADR-0022; KMIPKit-0069 | Existing version gate and typed routes | KMIP 2.1 default and unsupported-version pre-send tests |
| KMIPKIT-0091-FR-015 | Project constitution IV; KMIPKit-0043, -0049 | Existing secret owners, redaction and binding lifetimes | Redaction, zeroization, ownership, and foreign-copy tests |
| KMIPKIT-0091-FR-016 | Project constitution I, III; KMIPKit-0053 | API manifest inputs, generated exports, four binding guides | Generator check, export audit, docs, and traceability checks |

References to project requirement IDs are resolved through the approved
specifications listed in [spec.md](spec.md). Exact test identifiers and
implementation symbols must replace planned paths before the PR is ready for
review.
