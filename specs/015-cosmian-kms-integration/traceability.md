# Requirement Traceability: Cosmian KMS Integration

| Requirement | Source | Implementation | Verification |
|---|---|---|---|
| `KMIPKIT-0015-FR-001` | `spec.md`; Cosmian KMS 5.28.0 configuration; project transport boundary | `tests/integration/cosmian/compose.yaml`, `cosmian-kms.toml` | `docker compose config`; deployment status confirms local service after deployment |
| `KMIPKIT-0015-FR-002` | `spec.md`; project secret-handling policy (`AGENTS.md` §8) | `tests/integration/cosmian/generate-pki.sh`, `.gitignore` | Shell syntax check; confirm generated path is ignored and private keys are not in the KMS bind mount |
| `KMIPKIT-0015-FR-003` | OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; §9.16, Table 421 | `crates/kmipkit-client/tests/cosmian_kms.rs` | Explicit ignored live run; assert success and returned `(2, 1)` |
| `KMIPKIT-0015-FR-004` | `spec.md`; project default-test isolation policy | `#[ignore]` on all tests in `crates/kmipkit-client/tests/cosmian_kms.rs`; `scripts/integration/cosmian-kms.ps1`; quickstarts | Ordinary Cargo tests skip these; explicit script runs them serially |
| `KMIPKIT-0015-FR-005` | `spec.md`; KMIPKit no-retry policy (`AGENTS.md` §8) | Server-generated objects and metadata changes stay in container-local SQLite; no import or Destroy request is sent | Source review; compose teardown removes all server-side test data |
| `KMIPKIT-0015-FR-006` | Evidence-backed interoperability reporting policy | This document, `quickstart.md`, and `tests/integration/cosmian/README.md` | Record each pass, unsupported operation, and blocked path separately |
| `KMIPKIT-0015-FR-007` | OASIS KMIP Specification v2.1 §8.4, Table 397; §11.56 | `crates/kmipkit-transport/src/raw_tls.rs` | Raw-TLS tests accept Response Message `0x42007B` and reject Request Message `0x420078`; live Discover Versions passed |
| `KMIPKIT-0015-FR-008`, `KMIPKIT-0015-FR-009`, `KMIPKIT-0015-FR-010` | OASIS KMIP Specification v2.1 clauses below | One ignored typed integration test per selected operation; each attribute test provisions its own RSA public key with Create Key Pair | Explicit serial live run; per-operation result below. All ten selected operations passed with the candidate KMIPKIT-0014 fixes applied. |

## OASIS clause-to-test mapping and observed results

| OASIS source | Test assertion | Cosmian 5.28.0 live result |
|---|---|---|
| §8.4, Table 397; §11.56 | Response root uses tag `0x42007B`; Request Message tag `0x420078` is invalid on the response path | Passed in focused raw-TLS tests and original live run |
| §6.1.16, Tables 211–213; §9.16, Table 421 | Discover Versions succeeds and returns `(2, 1)` | Passed (1/1) |
| §6.1.8, Tables 186–188; §11.56 | Create returns Object Type and Unique Identifier | Passed (1/1) with the candidate KMIPKIT-0014 fix: Create encodes the required `Attributes` Structure tag `0x420125`. The unpatched integration branch's earlier run failed because it emitted `Attribute` tag `0x420008`; Cosmian logged `OperationFailed / Invalid_Message` and `missing field Attributes`. |
| §6.1.9, Tables 189–192 | Create Key Pair returns private and public identifiers | Passed (1/1) |
| §6.1.10, Tables 193–195; §11.36, Table 470 | Create Split Key returns two identifiers for a two-of-two XOR split | Passed (1/1) with the candidate KMIPKIT-0014 fixes: the source key was created, and the request/response used operation enumeration `0x28`. The unpatched integration branch initially stopped at source-key creation; its operation enum had also been incorrectly set to Register (`0x03`). |
| §6.1.2, Tables 167–169 | Add Attribute returns the object identifier | Passed (1/1) |
| §6.1.13, Tables 202–204 | Delete Attribute returns the object identifier | Passed (1/1) |
| §6.1.20, Tables 223–225 | Get Attributes returns the requested Comment in the typed result | Passed (1/1) |
| §6.1.21, Tables 226–228 | Get Attribute List returns a nonempty typed reference list | Passed (1/1) |
| §6.1.34, Tables 265–267 | Modify Attribute returns the object identifier | Passed (1/1) |
| §6.1.51, Tables 322–324 | Set Attribute returns the object identifier | Passed (1/1) |

### Excluded operation probe

Adjust Attribute (§6.1.3, Tables 170–172) was probed against the same pinned
server. Cosmian 5.28.0 rejected the request during KMIP message parsing with
`OperationFailed / Invalid_Message` and `unsupported operation:
AdjustAttribute`. The request did not reach KMIPKit's typed result parser. The
operation is therefore excluded from the supported-operation integration
target; this does not establish a KMIPKit Adjust Attribute defect.

## Run evidence

On 2026-10-09, the initial unpatched `feature/KMIPKIT-0015-cosmian-kms-integration`
run reported 8 passed and 2 failed. Create failed because its request builder
emitted the wrong Attributes tag; Create Split Key stopped during test setup,
and the operation enum was independently found to be incorrect. After applying
the candidate fixes from the separate `feature/KMIPKIT-0014-create-attributes-tag`
branch to the test checkout, the same command
(`.\scripts\integration\cosmian-kms.ps1 -Action test`) reported 10 passed,
0 failed. The results cover Discover Versions, Create, Create Key Pair, Create
Split Key, and the six attribute operations. PR #56 keeps the infrastructure
and integration tests scoped to KMIPKIT-0015; it remains a draft until the
KMIPKIT-0014 fix is integrated and this suite passes on the updated branch.
The suite is not a full KMIP 2.1 conformance result. The local Compose image is pinned to
`ghcr.io/cosmian/kms:5.28.0` and resolved in the original deployment to
`sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`.
The hosted GitHub CI workflow, including the multi-platform coverage gate,
passed on 2026-10-09.

The local `cargo llvm-cov --workspace --all-features --summary-only` run on
2026-10-09 completed with 83.80% line coverage across this Windows workspace
run. This single-platform raw report is not the project's hosted aggregate;
the cross-platform coverage workflow collected platform and adapter reports
and passed the 90% workspace threshold.
