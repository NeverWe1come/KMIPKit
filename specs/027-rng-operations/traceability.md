# KMIPKIT-0041 Traceability: Remote RNG Operations

Table contracts are mapped to existing stable catalog elements. Before review, link each implementation path and named executable test in the final columns. Do not reassign shared Data fields from their existing feature owner.

| Stable catalog element / requirement | Exact source | Contract | Planned implementation / verification |
|---|---|---|---|
| KMIPKIT-ELEM-OP-C2S-RNG-RETRIEVE | Spec v2.1 §6.1.49, Tables 316–318 | Request/response/error | rng_retrieve.rs; protocol and client RNG Retrieve suites |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-RETRIEVE-00000025 | §11.36 | Operation code | Catalog-backed registration; fake-transport request assertion |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH | §6.1.49 Table 316; §7.10 | Required request Integer | Type/cardinality, boundary-preservation, encoding tests |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA | §6.1.49 Table 317; §7.9 | Required secret response bytes | Round-trip and redaction tests; preserve shared catalog owner |
| KMIPKIT-ELEM-OP-C2S-RNG-SEED | Spec v2.1 §6.1.50, Tables 319–321 | Request/response/error | rng_seed.rs; protocol and client RNG Seed suites |
| KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-SEED-00000026 | §11.36 | Operation code | Catalog-backed registration; fake-transport request assertion |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-9-DATA | §6.1.50 Table 319; §7.9 | Required secret seed bytes | Round-trip and redaction tests; preserve shared catalog owner |
| KMIPKIT-ELEM-OPERATION-STRUCTURE-7-10-DATA-LENGTH | §6.1.50 Table 320; §7.10 | Required result Integer | Zero-use result remains success; malformed/type negatives |
| KMIPKIT-REQ-SPEC-6.1.50-001 | §6.1.50, server-unused-seed paragraph; SHALL NOT | Do not treat unused supplied seed data as an error | rng_seed.rs response conversion; test that Data Length 0 succeeds |

The approved specification links no direct official Test Case. FR-001–FR-008 and SC-001–SC-004 must be covered by source-derived table vectors, fake transport, redaction, full catalog/test traceability, and required review gates. Do not claim RNG generation, a test-vector pass, profile support, or interoperability.
