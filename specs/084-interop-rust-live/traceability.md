# Requirement traceability

| Requirement | Implementation | Verification |
|---|---|---|
| KMIPKIT-0084-FR-001 | validator config and CLI | CLI planning/preflight tests |
| KMIPKIT-0084-FR-002 | artifact installer | installed bundle integrity and provenance tests |
| KMIPKIT-0084-FR-003 | catalog dispatch | exhaustive shared case classification tests |
| KMIPKIT-0084-FR-004 | policy guard and single exchange | mutation denial, dedicated-environment and no-retry tests |
| KMIPKIT-0084-FR-005 | schema 2.0 report writer | JSON-schema, controlled rejection/redaction tests |
| KMIPKIT-0084-FR-006 | request adapters and local model verifier | high/typed equivalence and generic TTLV tests |
| KMIPKIT-0084-FR-007 | isolated Rust workflow | Rust 1.94 full validator checks and loopback evidence |

## Catalog execution evidence

Each immutable catalog row supplies its exact OASIS document and section.

| Family | Installed public implementation | Verification |
|---|---|---|
| 67 enum/bitmask cases | Generated named-value lookup and generic TTLV build/read | Local model test; no server-support claim |
| Synchronous/object/attribute cases | adapters, fixtures, recipes, direct methods and typed batches | Exhaustive constructor validation and complete 207-row TLS scenario |
| Async and lifecycle | Public Poll/Cancel/Process/query methods | Pending setup and explicit follow-up/cleanup |
| Split/join, batch, multipart, crypto | recipes::{join,batch,multipart,crypto} | Controlled full-catalog reachability and public shape validation |
| Five incoming operations, §§6.2.1–6.2.5 | ClientListener and ServerRequestDispatcher | Five successful dedicated mTLS exchanges |
| 2.0 version boundaries | Public pre-send version gate | Both probes pass with zero peer requests |
| State server-negative | §4.57, KMIPKIT-0051-R282 | library_preflight_rejection: library passed, server unknown, non-run independently of mutation opt-in |
| Generic outbound | KMIPKIT-0083-FR-007 | Explicit API limit, never server-unsupported |

`tools/generate_local_cases.py` regenerates consumer dispatch from reviewed
normative input. CI regenerates and requires no diff. Generated files are never
manually edited.

KMIPKIT-0084-FR-006 also verifies the complete case inventory through high-level
and typed APIs in both versions and compares normalized case outcomes.
KMIPKIT-0084-FR-007 uses `tools/check_coverage.py` for the >=90 percent line gate,
including real subprocess CLI and controlled TLS tests. The generated lookup
table is the only exclusion, justified by exhaustive behavioral/regeneration
checks rather than handwritten branch coverage.
