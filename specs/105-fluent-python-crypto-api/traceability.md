# KMIPKIT-0105 Traceability

| Requirement | Normative / parent source | Implementation | Verification |
|---|---|---|---|
| KMIPKIT-0105-FR-001 | KMIPKIT-0093-FR-011; KMIPKIT-0102-FR-001; KMIP v2.1 §§6.1.11, 6.1.17, 6.1.24, 6.1.32, 6.1.33, 6.1.55, 6.1.56 | `bindings/python/src/kmipkit/fluent.py`, `Client.request()` | Python consumer API and operation identity tests |
| KMIPKIT-0105-FR-002 | KMIP v2.1 §§4.16, 6.1.11, 6.1.17, 6.1.24, 6.1.32, 6.1.33, 6.1.55, 6.1.56, 7.8, 7.9, 7.14, 7.17, 7.20, 7.38, 7.39; KMIPKIT-0102-FR-002–004 | operation-specific Python builders and KMIPKIT-0103 C ABI | field matrix, request decoding, missing/invalid input tests |
| KMIPKIT-0105-FR-003 | KMIP v2.1 §4.16 Table 59; KMIPKIT-0093-FR-006; KMIPKIT-0102-FR-003 | typed `CryptographicParameters`, TTLV Structure and Data input adapters | typed/ordered parameter and expert Data tests |
| KMIPKIT-0105-FR-004 | KMIPKIT-0103-FR-003; AGENTS.md §8 | bounded staging helpers and Rust-owned C ABI copy | stage wiping and caller-buffer immutability tests |
| KMIPKIT-0105-FR-005 | KMIP v2.1 §§8–9; KMIPKIT-0093-FR-005, FR-008; KMIPKIT-0102-FR-005; KMIPKIT-0103-FR-004–005 | builder lifecycle, typed item conversion, `BatchResult` execution | TLS fixture Success/Pending/Failure, option/header separation, one-exchange and close tests |
| KMIPKIT-0105-FR-006 | KMIPKIT-0093-FR-004–007; ADR-0012, ADR-0018 | existing Python CFFI bridge to Rust-owned builder ABI | ABI symbol parity, malformed input, no-send rejection, source compatibility |
| KMIPKIT-0105-FR-007 | Constitution §§I, III, V; KMIPKIT-0093-FR-002, FR-011 | `py.typed`, API manifest source, generated Python declaration inventory | manifest generation/check, Python export audit and traceability validation |

The Python coverage correction adds explicit regression evidence in
`bindings/python/tests/test_fluent_crypto.py`: FR-004 is protected by
`test_crypto_builder_wipes_staging_when_span_allocation_fails`, which checks
binding-owned staging zeroization and preservation of the caller's buffer
when CFFI descriptor allocation fails. FR-002/003 local input rejection is
covered by the wrong-setter-type and invalid-memoryview tests. FR-005 is
covered by local validation without consuming the builder, rejection of
execution without a client, and deterministic closure after a consumer
exception. These tests extend the existing operation/TLS contracts without
adding protocol requirements or claiming profile conformance.
