# KMIPKIT-0053 Existing-Capability Gap Matrix

Baseline: commit a1ae6b402 on feature/KMIPKIT-0052-typed-responses, 2026-11-06.

| Capability | Baseline | KMIPKIT-0053 gap | Closure evidence |
|---|---|---|---|
| Named values (0047) | Four-language values exist for 64 enumerations, 644 assigned values, 3 masks, and 33 assigned bits. Capability verified. | Preserve and test values; do not reinterpret reserved/wildcard rows. | Parity fixtures, exact counts, generator check. |
| Typed attributes (0048) | Rust has 62 applicable definitions and generic bridges. Manifest has 286 candidate routes blocked. C has generic AttributeSet only. | Typed C/Java/Python constructors/readers/conversion for all 62 definitions; preserve repeats and generic values. | Four-language 62-definition matrix and mappings for all 286 declarations. |
| Managed objects/key (0049) | Rust models nine objects, KeyBlock/KeyValue/KeyMaterial/wrapping. 356 declarations blocked. Foreign wrappers deferred. | Foreign owners/views, generic fallback, explicit release, redaction/zeroization. | Four-language object/material matrix and exact mappings for 356 routes. |
| Parameters (0050) | Rust has all 18 Table 59 members and typed setters. 106 declarations blocked. | C/Java/Python constructors/readers/setters; retain generic Structure and future values. | Table 59 matrix and mappings for 106 declarations. |
| Request Authentication (0050) | Rust attaches one Authentication to ClientBatch; two declarations blocked. | Foreign batch API and header/scope/lifecycle checks. | Deterministic one-/multi-item tests and mappings for both declarations. |
| Pre-send validation (0051) | Rust has schema-aware reports and rejects before I/O. Foreign report access is not closed. | Expose the shared Rust report; prove NotSent and zero writes. | Report parity, fake transport, redaction. |
| Response and TTLV (0052) | Rust has typed readers/raw views; PR #123 adds owned C/JNI/CFFI copies. | Audit/fill typed response and async result parity without changing 0052 semantics. | 56-operation matrix and nested vendor tests in four languages. |
| Operations/package/facade | Rust implements 56 operations and one-crate facade. Foreign routes need behavior/options parity evidence. | Verify operation/options/results and runnable packages; no low-level exchange exposure. | Consumer tests, version sync, Rust one-crate example. |

## Manifest Baseline
The reviewed manifest has 750 candidateSpecId KMIPKIT-0053 declarations: 286 attributes, 356 managed-object/key routes, 106 parameters, and 2 Authentication routes. Four capabilities are blocked. Inventory audits do not prove behavior parity. Every candidate must become an evidenced mapping or exact justified exclusion; no general language exclusion is allowed.
