# Task 5 report: Cosmian transfer and key management

**Target:** Cosmian KMS 5.28.0, source commit `4ab3ee65829a903abb71f1298f400657fcee7ec9`, image digest `sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`.

**Scope:** All 4,640 `cosmian_kms::transfer` and 216 `cosmian_kms::key_management` rows in the approved capability matrix. The row-by-row outcomes, target-request delivery flags, sources, and case IDs are in [`capability-matrix.json`](../../tests/integration/cosmian/capability-matrix.json). KMIPKit's typed Rust client and production raw-TTLV TLS/mTLS client path were used. All live cases remain ignored in ordinary Cargo runs.

## Final measured outcomes

| Family | Cases | Typed success | Target rejected | Setup rejected | Fixture unavailable |
|---|---:|---:|---:|---:|---:|
| Transfer | 4,640 | 2,772 | 866 | 930 | 72 |
| Key management | 216 | 12 | 201 | 3 | 0 |

`typed_success` means the target operation received a successful typed response and passed its case-specific postconditions. `rejected` means the target request was sent and Cosmian returned a KMIP rejection; these are failed interoperability cases, even when the response was safely classified. `setup_rejected` means a prerequisite operation received a rejection and the target operation was not sent. `fixture_unavailable` means the row's standards-valid fixture could not be produced, so no request was sent. None of these latter three classifications counts as target interoperability success.

The 72 fixture-unavailable transfer rows are the ConfigurableKEM rows listed in `crates/kmipkit-client/tests/cosmian_kms/configurable-kem-fixture-unavailable-case-ids.txt`. No post-quantum enum value reserved by KMIP 2.1 was sent. Their exclusion is a fixture availability result, not a KMIPKit typed-request limitation or Cosmian response.

## Corrections and reruns

The initial serial family runs classified 4,640 transfer rows and 216 key-management rows. A 230-ID transfer rerun corrected fixture and outcome attribution: 50 rows completed successfully, 12 reached the target and received KMIP rejection, 164 failed during setup, and 4 had response postcondition failures. The final 32-ID PFX Get/Export rerun supersedes the earlier results for those exact IDs: 16 unprotected-password cases passed, and 16 password-linked cases were rejected during prerequisite PFX Import. Those target Get/Export operations were not sent. The final matrix contains only the latest measured result for each case.

Earlier rejected wrapping observations that depended on inactive or incorrectly configured KEKs were replaced by the corrected serial measurement. The corrected fixtures activate and verify owned wrapping keys and use the role-specific Wrap/Encrypt and Unwrap/Decrypt masks. Setup failures remain separate from target-operation rejections.

## PKCS#12 Password Link discrepancy

OASIS KMIP v2.1 §4.31, Tables 90–92, models `PKCS12PasswordLink` as a Link to a `SecretData` object. KMIPKit sends that server-assigned object identifier; the test does not put password text in the Link or the Key Wrapping Specification. The pinned Cosmian 5.28.0 `process_pkcs12` code treats the linked identifier text as the password, instead of resolving the referenced SecretData material. The output path also takes its PKCS#12 password from the Encryption Key Information identifier. All 16 standard-shaped password-link PFX Import setup requests received KMIP Result Reason 5 (`Operation Not Supported`), so their dependent Get/Export request was not sent. This is a server interoperability discrepancy, not evidence that the target Get/Export operation rejects the request. The matrix does not use a literal-password workaround.

The standard and legacy unprotected PFX cases passed. For a Certificate target requested as PKCS#12, Cosmian returns a typed `PrivateKey` package containing the linked private key and certificate. The response check accounts for that object type. The legacy vendor request value `0x88800001` is normalized by Cosmian to standard PKCS#12 response Key Format Type `0x16`.

## Verification and teardown

Offline transfer/key-management constructor and parity tests use typed KMIPKit request builders. The final manifest has 4,856 unique IDs across the two assigned families, with no missing or `not_run` outcomes. Network tests remain `#[ignore]` by default.

Final-head offline checks:

| Command | Result |
|---|---|
| `cargo test -p kmipkit-client --test cosmian_kms` | 56 passed, 0 failed, 106 ignored |
| `cargo test -p kmipkit-client --test cosmian_kms -- transfer::` | 25 passed, 0 failed, 1 ignored |
| `cargo test -p kmipkit-client --test cosmian_kms -- key_management::` | 5 passed, 0 failed, 1 ignored |
| `cargo fmt --all --check` | Passed |
| `cargo clippy -p kmipkit-client --test cosmian_kms --all-features -- -D warnings` | Passed; only aws-lc-sys C/NASM build-environment warnings were emitted |
| `git diff --check` | Passed |
| `cargo llvm-cov -p kmipkit-client --test cosmian_kms --summary-only` | Completed; 0.43% `kmipkit-client` lines covered with live cases ignored. This package report does not measure the integration-test source itself and is not a coverage-gate result. |

The Task 5 family runs and correction reruns used serial test execution. Each live cycle used the pinned Compose deployment through `scripts/integration/cosmian-kms.ps1`; the final 32-case rerun ended with `down` exit code 0 and `status` exit code 0, and `docker compose ps --status running` returned no services. A final `docker compose -f tests/integration/cosmian/compose.yaml ps --all` check also returned no services.

The results are interoperability observations for these exact request and fixture cases. They do not establish full KMIP 2.1 profile conformance or certification. The RNG and cryptographic operation families and the final all-family run remain outside Task 5 and must be completed before the overall KMIPKIT-0015 matrix is considered complete.
