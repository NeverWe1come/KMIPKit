# KMIPKIT-0046 Traceability

| Requirement | Design/source | Code surface | Verification |
|---|---|---|---|
| KMIPKIT-0046-FR-001 | ADR-0003, ADR-0006, ADR-0017 | `crates/kmipkit/src/lib.rs` protocol and TTLV namespaces | External fixture imports all 56 operation request types; Rust export audit |
| KMIPKIT-0046-FR-002 | ADR-0005, ADR-0017 | `client_config` and `transport_config` facade namespaces | External fixture builds the public client constructor signature |
| KMIPKIT-0046-FR-003 | ADR-0006, ADR-0017 | Client request/response/outcome/error exports | External fixture compile checks representative synchronous and asynchronous types |
| KMIPKIT-0046-FR-004 | ADR-0014, ADR-0017 | Selective facade exports | Export audit confirms low-level exchange symbols are not reachable through `kmipkit` |
| KMIPKIT-0046-FR-005 | KMIPKIT-0043 manifest-v2 | `specification/api/public-api.json` and generated outputs | Manifest generator and full Rust export audit |
| KMIPKIT-0046-FR-006 | ADR-0005, ADR-0006, ADR-0012 | Additive exports only | Workspace tests and existing API compile checks |
