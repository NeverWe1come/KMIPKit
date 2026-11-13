# KMIPKIT-0116 Traceability

| Requirement | Normative/project source | Verification |
|---|---|---|
| FR-001 operation semantics | KMIP 3.0 Working Draft §§6.1.9/Table 277, 6.1.10/Table 280, 6.1.13/Table 290, 6.1.14/Table 293, 6.1.15/Table 296, 6.1.22/Table 317, 6.1.41/Table 374; existing typed models | Per-operation builder construction and exact ordered typed payload tests in Rust/C/Java/Python |
| FR-002 version, validation, execution | KMIP 3.0 §6; §11.37/Table 584; KMIPKIT-0007, -0069, -0071, -0093; ADR-0022 | Selected-version accept and mismatch rejection; no-send negative tests; one-exchange outcome and delivery-state tests |
| FR-003 credential and KEM fields | KMIP 3.0 Tables 277, 296–297, 317–318; KMIPKIT-0050 | Closed credential alternatives; explicit KEM parameter/attribute behavior; byte ownership and no shared-secret response tests |
| FR-004 typed results and redaction | KMIP 3.0 operation response tables; KMIPKIT-0093-FR-008/-010; ADR-0018 | Success/Pending/Failure field presence, payload-free errors, redacted formatting, encoded response access |
| FR-005 binding compatibility | KMIPKIT-0043, -0053, -0093-FR-011; Constitution III, V, IX | C ABI export audit and sanitizer; JNI/CFFI ownership tests; Java/Python consumer and type-shape tests |
| FR-006 docs and manifests | KMIPKIT-0093-FR-001/-002/-003/-011; Constitution I, X, XI | Generation `--check`, all-language export audit, capability-matrix parity, bilingual examples, documentation contracts |
