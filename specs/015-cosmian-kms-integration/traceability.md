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
| `KMIPKIT-0015-FR-008`, `KMIPKIT-0015-FR-009`, `KMIPKIT-0015-FR-010` | OASIS KMIP Specification v2.1 clauses below; §11.56, Table 487 for cryptographic payload tags | Twenty ignored typed integration tests; each attribute test provisions its own server-generated RSA public key | Explicit serial live run; original ten and eight added operations return successful typed results. Query and Ping are separately classified as observed server interoperability limitations below. |

## OASIS clause-to-test mapping and observed results

| OASIS source | Test assertion | Cosmian 5.28.0 live result |
|---|---|---|
| §8.4, Table 397; §11.56 | Response root uses tag `0x42007B`; Request Message tag `0x420078` is invalid on the response path | Passed in focused raw-TLS tests and original live run |
| §6.1.16, Tables 211–213; §9.16, Table 421 | Discover Versions succeeds and returns `(2, 1)` | Passed (1/1) |
| §6.1.8, Tables 186–188; §11.56 | Create returns Object Type and Unique Identifier | Passed (1/1) after syncing release's KMIPKIT-0014 correction (PR #69, commit `9e51a08`): Create encodes the required `Attributes` Structure tag `0x420125`. The earlier unpatched branch emitted `Attribute` tag `0x420008`; Cosmian logged `OperationFailed / Invalid_Message` and `missing field Attributes`. |
| §6.1.9, Tables 189–192 | Create Key Pair returns private and public identifiers | Passed (1/1) |
| §6.1.10, Tables 193–195; §11.36, Table 470 | Create Split Key returns two identifiers for a two-of-two XOR split | Passed (1/1) after syncing release's KMIPKIT-0014 correction (PR #69, commit `9e51a08`): the source key was created, and the request/response used operation enumeration `0x28`. The earlier unpatched branch stopped at source-key creation and used the Register enum (`0x03`). |
| §6.1.2, Tables 167–169 | Add Attribute returns the object identifier | Passed (1/1) |
| §6.1.13, Tables 202–204 | Delete Attribute returns the object identifier | Passed (1/1) |
| §6.1.20, Tables 223–225 | Get Attributes returns the requested Comment in the typed result | Passed (1/1) |
| §6.1.21, Tables 226–228 | Get Attribute List returns a nonempty typed reference list | Passed (1/1) |
| §6.1.34, Tables 265–267 | Modify Attribute returns the object identifier | Passed (1/1) |
| §6.1.51, Tables 322–324 | Set Attribute returns the object identifier | Passed (1/1) |

## Added operation paths

The following ten opt-in tests were added on 2026-10-15 using the production
KMIPKit typed Rust client over the same pinned Cosmian 5.28.0 mutual-TLS
listener. Outcomes distinguish operation interoperability from expected
limitation probes; 20/20 test cases passing does not mean 20 operations were
successfully interoperable.

| OASIS source | Test and assertion | Cosmian 5.28.0 live result |
|---|---|---|
| §6.1.1, Tables 163–166 | Activate returns a successful typed result for a fresh server-generated key | Passed (1/1) |
| §6.1.40, Tables 282–284 | Query Operations result is checked for a nonempty operation list and required Protection Storage Masks | Expected incompatibility: Cosmian returns Success and 37 Operations but omits required Protection Storage Masks; KMIPKit rejects the structured response |
| §6.1.36, Tables 271–272 | Ping error status/reason and retained result shape are checked | Expected incompatibility: Cosmian rejects Ping as unsupported with Operation Failed / Invalid Message and omits Operation from its error Batch Item |
| §6.1.17, Tables 214–216 | Encrypt returns ciphertext Data for an activated AES test key | Passed (1/1) |
| §6.1.11, Tables 196–198 | Decrypt returns the original plaintext for a typed Encrypt/Decrypt round trip | Passed (1/1) |
| §6.1.24, Tables 235–237 | Hash returns digest Data for the explicit SHA-256 request | Passed (1/1) |
| §6.1.32, Tables 259–261 | MAC returns typed MAC Data | Passed (1/1) |
| §6.1.33, Tables 262–264 | MAC Verify returns a typed Validity Indicator using the MAC result | Passed (1/1) |
| §6.1.55, Tables 334–336 | Sign returns typed Signature Data | Passed (1/1) |
| §6.1.56, Tables 337–339 | Signature Verify returns a typed Validity Indicator using the Sign result | Passed (1/1) |

The cryptographic response/input field tags are from OASIS KMIP v2.1 §11.56,
Table 487: MAC Data `0x4200C6`, Signature Data `0x4200C3`, and Validity
Indicator `0x42009B`. Cosmian responses and requests exposed that KMIPKit's
previous values were incorrect; KMIPKit's shared cryptographic tag constants
were corrected under the already approved KMIPKIT-0021 scope before this final
run. This is a KMIPKit correction, not a Cosmian tag mismatch.

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
and the operation enum was independently found to be incorrect. On 2026-10-10,
PR #56 was synced with `release/1.0.0`, which contains the separately reviewed
KMIPKIT-0014 corrections from PR #69. On merge head `a1945f4e`,
(`.\scripts\integration\cosmian-kms.ps1 -Action test`) reported 10 passed,
0 failed. The results cover Discover Versions, Create, Create Key Pair, Create
Split Key, and the six attribute operations. The fixes are inherited from
release and are not duplicated in this integration PR. After pinning both
Compose images by digest, the serial test command passed 10/10 again on commit
`9a6a6341`. The suite is not a full KMIP 2.1 conformance result. The local
images are pinned to `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`
and `alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412`.
The hosted GitHub CI workflow, including the multi-platform coverage gate,
passed on 2026-10-09.

On 2026-10-15, after adding Activate, Query, Ping, Encrypt, Decrypt, Hash, MAC,
MAC Verify, Sign, and Signature Verify tests and correcting the KMIPKIT-0021
cryptographic tag assignments, the explicit serial test command reported 20
passed, 0 failed. Eighteen operation exchanges produced successful typed
results; Query and Ping passed only as expected-limitation probes and are not
counted as successful interoperability. Query's server response omitted
Protection Storage Masks required by Table 283. Cosmian rejected Ping as
unsupported and returned an error Batch Item without Operation. These are
observations against the pinned local 5.28.0 image, not broad support claims.

The final run after narrowing activation setup and keeping copied cryptographic
outputs in `SecretBytes` used
`$env:KMIPKIT_COSMIAN_CERT_DIR = (Resolve-Path .local/cosmian-kms/certs).Path; cargo test -p kmipkit-client --test cosmian_kms -- --ignored --nocapture --test-threads=1`.
On the final merged integration head `5b86ab87`, it again reported 20
passed, 0 failed. Afterward,
`scripts/integration/cosmian-kms.ps1 -Action down` removed the Cosmian and PKI
containers and their Compose network; the subsequent `-Action status` listed no
running services. Generated local certificates remain ignored for the next
opt-in run.

Post-change local verification on 2026-10-15 passed `cargo fmt --all --check`,
`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
and `cargo test --workspace --all-features --locked --quiet`. The local
`cargo llvm-cov --workspace --all-features --summary-only` run reported 92.92%
workspace line coverage, including 96.17% for `kmipkit-protocol` and 98.29% for
`kmipkit-ttlv`. Hosted CI and its cross-platform coverage aggregation still
need to run on this expanded branch head.

The local `cargo llvm-cov --workspace --all-features --summary-only` run on
2026-10-09 completed with 83.80% line coverage across this Windows workspace
run. This single-platform raw report is not the project's hosted aggregate;
the cross-platform coverage workflow collected platform and adapter reports
and passed the 90% workspace threshold.
