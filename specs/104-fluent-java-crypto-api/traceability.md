# KMIPKIT-0104 Traceability

| Requirement | Source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0104-FR-001 | KMIPKIT-0093-FR-004–011; KMIPKIT-0102-FR-001; KMIPKIT-0103-FR-001; OASIS KMIP v2.1 §§6.1.11, 6.1.17, 6.1.24, 6.1.32, 6.1.33, 6.1.55, 6.1.56 | `bindings/java/src/main/java/org/kmipkit/fluent/FluentRequest.java` and operation builders | Java consumer API-shape and operation identity tests |
| KMIPKIT-0104-FR-002 | OASIS KMIP v2.1 §4.16 Table 59; §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38, 7.39; KMIPKIT-0102-FR-002; KMIPKIT-0103-FR-002 | Operation-specific Java setters forward to Rust-owned typed C builders | Request-shape and negative-input tests for every operation |
| KMIPKIT-0104-FR-003 | KMIPKIT-0103-FR-003; OASIS KMIP v2.1 operation field tables listed above; AGENTS.md Security invariants | JNI byte-span bridge and Rust zeroizing C builder storage | Java source-array mutation, JNI temporary cleanup, and native secret ownership tests |
| KMIPKIT-0104-FR-004 | KMIPKIT-0091-FR-015–016; KMIPKIT-0102-FR-005; KMIPKIT-0103-FR-004; ADR-0012 and ADR-0018 | `CryptoBuilder` lifecycle, typed `ClientBatchItem`, `KmipClient.execute` | All seven controlled TLS fixture exchanges, typed outcomes, correlation and delivery-state tests |
| KMIPKIT-0104-FR-005 | KMIPKIT-0091-FR-015–016; KMIPKIT-0093-FR-011; KMIPKIT-0103-FR-005–006; AGENTS.md §§9–11 | Separate Java options/header inputs and generated public API inventory | API compatibility, manifest generation, export audit, and traceability checks |
