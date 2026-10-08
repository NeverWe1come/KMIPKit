# Requirement Traceability: Cosmian KMS Integration

| Requirement | Source | Implementation | Verification |
|---|---|---|---|
| `KMIPKIT-0015-FR-001` | `spec.md`; Cosmian KMS 5.28.0 configuration; project transport boundary | `tests/integration/cosmian/compose.yaml`, `cosmian-kms.toml` | `docker compose config`; deployment status confirms local service after deployment |
| `KMIPKIT-0015-FR-002` | `spec.md`; project secret-handling policy (`AGENTS.md` §8) | `tests/integration/cosmian/generate-pki.sh`, `.gitignore` | Shell syntax check; confirm generated path is ignored and private keys are not in the KMS bind mount |
| `KMIPKIT-0015-FR-003` | OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; §9.16, Table 421 | `crates/kmipkit-client/tests/cosmian_kms.rs` | Explicit ignored live run after KMIPKIT-0013 clears its gate; assert success and `(2, 1)` |
| `KMIPKIT-0015-FR-004` | `spec.md`; project default-test isolation policy | `#[ignore]` in `crates/kmipkit-client/tests/cosmian_kms.rs`; `scripts/integration/cosmian-kms.ps1`; quickstarts | Inspect Cargo test selection and run only when expressly requested |
| `KMIPKIT-0015-FR-005` | `spec.md`; KMIPKit no-retry policy (`AGENTS.md` §8) | Read-only Discover Versions test; container-local SQLite; compose stop command | Source review; no key-lifecycle requests are constructed |
| `KMIPKIT-0015-FR-006` | Accurate, evidence-backed interoperability reporting policy | `specs/015-cosmian-kms-integration/research.md`, `quickstart.md`, `tests/integration/cosmian/README.md` | Record the initial failure and observed rerun separately; limit the pass claim to the tested Discover Versions exchange |
| `KMIPKIT-0015-FR-007` | OASIS KMIP Specification v2.1 §8.4, Table 397; §11.56 | `crates/kmipkit-transport/src/raw_tls.rs` | `crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_a_response_with_an_invalid_root_tag` and valid Response Message frames; live Cosmian Discover Versions test |

## OASIS clause-to-assertion mapping

| OASIS source | Assertion in test | Limit |
|---|---|---|
| KMIP Specification v2.1 §8.4, Table 397; §11.56 | Response root uses tag `0x42007B`; Request Message tag `0x420078` is invalid in a response | Valid and invalid response-frame tests; transport frame validation only |
| KMIP Specification v2.1 §6.1.16, Table 211 | Request uses KMIPKit's typed Discover Versions operation | Exercises this operation only |
| KMIP Specification v2.1 §6.1.16, Table 212 | Response is parsed as supported Protocol Version list | Requires the server to report the offered version for this smoke-test success criterion |
| KMIP Specification v2.1 §6.1.16, Table 213 | Operation Result Status is asserted to be Success | No failure/policy scenarios are included in this first test |
| KMIP Specification v2.1 §9.16, Table 421 | Supported version contains `(2, 1)` | Does not establish all KMIP 2.1 behavior |

The live integration assertion remains pending and must not be marked successful
until it is actually executed against the pinned local server.
