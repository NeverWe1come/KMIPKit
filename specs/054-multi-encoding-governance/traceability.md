# KMIPKIT-0054 Traceability

| Requirement | Source / level | Owning artifact | Verification |
|---|---|---|---|
| KMIPKIT-0054-FR-001 | Approved project scope | Constitution, `AGENTS.md`, project definition, ADR-0018 | Cross-document scope review |
| KMIPKIT-0054-FR-002 | Maintainer decision | ADR-0002 supersession note, ADR-0018 | ADR index and supersession review |
| KMIPKIT-0054-FR-003 | OASIS KMIP Specification v2.1 §§10.1–10.2.3; Profiles §§5.4.1.1–5.4.1.3, 5.4.1.6.4, 5.4.1.6.7, 5.5.1.1–5.5.1.3, 5.5.1.6.4, 5.5.1.6.7 | 0054 common `Item` model and generated tag/Enumeration/Bit Mask/family tables | Catalog generator tests; codec round-trip suites |
| KMIPKIT-0054-FR-004 | AGENTS.md §8; ADR-0012; ADR-0014 | 0054 common codec contract | Limit/error/owner tests in codec and integration specs |
| KMIPKIT-0054-FR-005 | Profiles §§5.4.1.1–5.4.1.3, 5.4.1.6.7, 5.5.1.1–5.5.1.3, 5.5.1.6.7; ADR-0004 | Catalog-derived tag/Enumeration/family metadata contract | Offline deterministic lookup tests in 0055/0056 |
| KMIPKIT-0054-FR-006 | ADR-0004; KMIPKIT-0043 manifest contract | 0054–0056 exclusions; 0057 API integration | Diff ownership and generator check in 0057 |
| KMIPKIT-0054-FR-007 | KMIPKit parallel feature workflow | 0055 and 0056 independent specs | Separate worktree suites against shared contract |
| KMIPKIT-0054-FR-008 | ADR-0005, ADR-0015; KMIPKit raw TLS architecture | ADR-0018; 0057 transport contract | Fragmented document and one-connection lifecycle tests in 0057 |
| KMIPKIT-0054-FR-009 | Profiles §§5.3.1–5.3.2, source discrepancy; project interpretation in ADR-0018 | ADR-0018 and 0057 | HTTPS content type/body tests; no OASIS resolution claim |
| KMIPKIT-0054-FR-010 | Profiles §§5.4 and 5.5; ADR-0014 | 0057 integration contract | Separate actual-byte cap and TTLV-equivalent Maximum Response Size tests |
| KMIPKIT-0054-FR-011 | AGENTS.md §8; ADR-0012 | 0057 client behavior | Mismatch, delivery-state, and no-retry tests |
| KMIPKIT-0054-FR-012 | Maintainer coordination decision | Roadmap and 0054–0057 spec assignments | Identifier and branch-name review |
| KMIPKIT-0054-FR-013 | OASIS KMIP Profiles v2.1 §§5.4.1.6.4, 5.5.1.6.4; reviewed KMIP 2.1 catalog | `WIRE_NAMES` Bit Mask family and assigned component records | Generator inventory tests for all families and assigned values; hexadecimal fallback tests in 0055/0056 |

Codec behavior requirements are intentionally owned by the approved 0055 and
0056 specifications; this table does not mark their implementation evidence
complete.
