# Requirement Traceability: Cosmian KMS Integration

The 2026-10-16 exhaustive matrix amendment is being implemented in scoped family tasks. Task 5 measured all transfer and key-management rows; Task14 later ran the complete ignored suite serially against the pinned digest and verified Compose teardown. The suite had ten failing test functions, with unresolved lifecycle, crypto, RNG, and transfer outcomes recorded below and in the Task 6 report. The matrix remains the per-case observation source; the Task14 log is not copied into matrix fields without case-by-case reconciliation. Historical selected-case results below remain separate bounded evidence.

| Requirement | Source | Implementation | Verification |
|---|---|---|---|
| `KMIPKIT-0015-FR-001` | `spec.md`; Cosmian KMS 5.28.0 configuration; project transport boundary | `tests/integration/cosmian/compose.yaml`, `cosmian-kms.toml` | `docker compose config`; deployment status confirms local service after deployment |
| `KMIPKIT-0015-FR-002` | `spec.md`; project secret-handling policy (`AGENTS.md` §8) | `tests/integration/cosmian/generate-pki.sh`, `.gitignore` | Shell syntax check; confirm generated path is ignored and private keys are not in the KMS bind mount |
| `KMIPKIT-0015-FR-003` | OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; §9.16, Table 421 | `crates/kmipkit-client/tests/cosmian_kms.rs` | Explicit ignored live run; assert success and returned `(2, 1)` |
| `KMIPKIT-0015-FR-004` | `spec.md`; project default-test isolation policy | `#[ignore]` on all live tests in `crates/kmipkit-client/tests/cosmian_kms.rs`; `scripts/integration/cosmian-kms.ps1`; quickstarts | Ordinary Cargo tests skip these; explicit script runs them serially |
| `KMIPKIT-0015-FR-005` | `spec.md`; test ownership and secret/no-retry policy (`AGENTS.md` §8); Import §6.1.25, Tables 238–240; Destroy §6.1.15, Tables 208–210 | Existing disposable SQLite; planned solely test-generated imports and Destroy of unique same-case objects | Source review, per-case ownership/cleanup evidence and final Compose teardown; expanded mutation cases pending |
| `KMIPKIT-0015-FR-006` | Evidence-backed interoperability reporting policy | This document, `quickstart.md`, and `tests/integration/cosmian/README.md` | Record each pass, unsupported operation, and blocked path separately |
| `KMIPKIT-0015-FR-007` | OASIS KMIP Specification v2.1 §8.4, Table 397; §11.56 | `crates/kmipkit-transport/src/raw_tls.rs` | Raw-TLS tests accept Response Message `0x42007B` and reject Request Message `0x420078`; live Discover Versions passed |
| `KMIPKIT-0015-FR-008` | OASIS operation contracts in `spec.md`; §11.56, Table 487 | Existing typed `ClientRequest` API; baseline target and planned family modules | Baseline eighteen successful typed exchanges plus Query/Ping findings; expanded typed cases and API blockers pending |
| `KMIPKIT-0015-FR-009` | Attribute operation clauses below; project isolation rule | Baseline each attribute case provisions its own RSA public key; expanded cases own separate fixtures | Historical serial run; planned selected-case independence checks |
| `KMIPKIT-0015-FR-010` | Exact OASIS operation/data registry in `spec.md`; evidence policy | This document and planned per-case manifest/results | Historical baseline below remains separate from pending exhaustive observations |
| `KMIPKIT-0015-FR-011` | All 35 operation clauses in `spec.md`; Set §6.1.51, Tables 322–324; Modify §6.1.34, Tables 265–267; Query §6.1.40, Tables 282–284; separate probes §6.1.3/§6.1.36/§6.1.45 | Complete matrix inventory and ignored typed family cases; Set/Modify remain separate and Query/probes retain separate classifications | Offline exact-path and case/dispatch validators pass. Task14 completed one serial run (98 passed, 10 failed); the unresolved operation outcomes remain listed in the Task 6 report and are not promoted to success. Query remains a required-field defect |
| `KMIPKIT-0015-FR-012` | OASIS §2.1, Table 2; §2.3, Table 4; §§2.5–2.9, Tables 6–10, crossed with each scoped operation contract | Planned source-derived object intersections and exclusion/disagreement ledger | Offline source inventory checks, independent object cases and per-case outcomes pending |
| `KMIPKIT-0015-FR-013` | OASIS §§3.1–3.2, Tables 11–13; §4.26, Tables 79–80; §11.25, Tables 460–461; §2.1, Table 2; §4.39, Tables 107–108; §4.31, Tables 90–92; §11.28, Tables 464–465 | Planned Import/Export formats/encodings and generated PFX fixture; pinned vendor format/implementation sources in `research.md` | Default/explicit format and applicable PEM/DER path inventory, PFX source-omission note, wire/fixture distinction and live outcomes pending |
| `KMIPKIT-0015-FR-014` | OASIS §§4.13–4.16, Tables 53–60; §11.12, Table 443; §11.45, Table 477; §11.6, Table 436; §11.16, Table 447; §11.21, Table 454; §11.30; §11.37, Table 471; derivation §7.12, Table 364/§11.14, Tables 445–446; wrapping §3.3, Tables 14–17/§7.18, Table 370/§11.18, Tables 449–450/§11.29/§11.62, Tables 493–494 | Explicit algorithm/parameter rows and typed crypto pairs; source gaps/discrepancies retain their own dispositions | Offline builders serialize all 1,220 typed-success requests and 231 exact finding probes (269 payloads); 15 probes are `NotSent`. Task14 verified only a subset live: 724 of 1,466 crypto rows were unverified, and all six valid RNG rows received malformed typed responses. These remain unresolved; IV Length bits and Tag Length bytes follow §4.16 |
| `KMIPKIT-0015-FR-015` | Project case isolation, ownership, no-retry and teardown policy; applicable operation cleanup contracts | Planned per-case helpers/object tracking and runner teardown on all paths | Selected-case independence, cleanup preserving original error and empty Compose status evidence pending |
| `KMIPKIT-0015-FR-016` | Project traceability/reporting policy and approved matrix design; exact OASIS per-case clauses | Planned manifest with stable case/FR/source/test/result links and sanitized reports | Offline schema, completeness and case/test parity plus actual per-case live evidence pending |

## Success-criterion mapping for the exhaustive amendment

| Success criterion | Requirement links | Verification target and status |
|---|---|---|
| `KMIPKIT-0015-SC-001` | FR-001, FR-002 | Existing pinned Compose/start infrastructure; repeat final-head configuration/start check during matrix run |
| `KMIPKIT-0015-SC-002` | FR-004, FR-008, FR-016 | Ordinary-run ignored live cases plus offline validator and selected/full run commands; expanded checks pending |
| `KMIPKIT-0015-SC-003` | FR-006, FR-008, FR-011 | Exact 35 paths, separate Set/Modify, known Query defect and separate probes; new inventory/live results pending |
| `KMIPKIT-0015-SC-004` | FR-006, FR-010, FR-016 | Every row individually measured or explicitly blocked; no fake success from a passing finding assertion; pending |
| `KMIPKIT-0015-SC-005` | FR-011–FR-014, FR-016 | Manifest source completeness, schema/unique IDs, no omitted dimensions and case/test parity; pending |
| `KMIPKIT-0015-SC-006` | FR-005, FR-013, FR-016 | Default/explicit Import/Export, applicable PEM/DER, PFX omission note and actual wire/fixture evidence; pending |
| `KMIPKIT-0015-SC-007` | FR-014, FR-016 | Offline contract covers 1,220 success rows, 231 exact finding probes, and 15 `NotSent` rows; full live run left crypto/RNG failures unresolved, so the criterion is pending |
| `KMIPKIT-0015-SC-008` | FR-002, FR-005, FR-008, FR-009, FR-015 | Independent unique state/fixtures, tracked linked objects, cleanup and redacted diagnostics; expanded evidence pending |
| `KMIPKIT-0015-SC-009` | FR-005, FR-015 | Task14 serial full run ended after test failures, then Compose down completed and `ps -a` was empty; remaining final-head gates are pending |

## Task 5: transfer and key-management observations

Task 5 implemented and exercised the complete `cosmian_kms::transfer` and `cosmian_kms::key_management` assignments in `tests/integration/cosmian/capability-matrix.json`. Every recorded case uses the typed KMIPKit client over the pinned local binary TTLV/mTLS deployment. The manifest records each stable case ID, sanitized classification, and whether the target operation request was sent. The counts below include failed interoperation; only a typed response that passed operation-specific postconditions is counted as `typed_success`.

| Family | Measured cases | `typed_success` | Target `rejected` | `setup_rejected` | `fixture_unavailable` |
|---|---:|---:|---:|---:|---:|
| Transfer | 4,640 | 2,772 | 866 | 930 | 72 |
| Key management | 216 | 12 | 201 | 3 | 0 |

### Task 6 progress: RNG inventory and typed cases

The RNG family has offline one-to-one case-ID coverage for all twelve matrix rows. Six valid length-only cases are executable: Retrieve lengths 0, 1, 32, and 65,536 bytes; Seed lengths 0 and 32 bytes. `cosmian::random::valid_rng_rows_construct_and_encode_the_requested_typed_payloads_offline` verifies each request contains only the requested-length Data Length or Data field; the ignored live runner checks the returned byte count and response shape. Six variants adding `RNG Parameters { RNG Algorithm: Unspecified }` are explicitly excluded without sending a request: §6.1.49/Table 316 defines only Data Length in RNG Retrieve, and §6.1.50/Table 319 defines only Data in RNG Seed; §7.36/Table 388's structure definition does not add it to either payload. Their evidence is linked to `cosmian_kms::random::cosmian_random_case_ids_cover_manifest_and_exclusions`.

The accepted cryptographic target inventory has 1,466 standards-valid rows (the six standards-invalid Certify rows remain excluded): 1,220 `typed_success`, six `documented_but_source_gap`, and 240 `source_documentation_discrepancy`. Per-operation typed-success counts are Certify 173, Create 70, Create Key Pair 35, Decrypt 224, Encrypt 289, Hash 14, MAC 27, MAC Verify 54, Sign 110, and Signature Verify 224. Offline `every_typed_success_matrix_row_builds_a_typed_ttlv_payload_offline` constructed a nonempty typed TTLV payload for every typed-success row; `multipart_matrix_rows_construct_all_three_typed_parts` additionally validated all three request parts for 19 multipart cases. Exact source-gap/discrepancy preflight keeps 231 exact request builders separate from 15 `NotSent` fixture/configuration blockers (228 discrepancy and three source-gap builders; 12 discrepancy and three source-gap blockers). The exact builders serialize 269 payloads because 19 exact probe rows use three parts. No finding probe is counted or printed as interoperability success. The ID/dispatch contract reports 1,220 executable success rows, zero dispatch gaps, and 246 findings.

A focused Task 6 run has pinned-digest-correlated results for 33 finding probes: 30 Certify cases returned a failure response with safe fields Result Status 1 (`Operation Failed`) and Result Reason 4 (`Invalid Message`), but no response Batch Item Operation; the typed client rejected those responses. OASIS KMIP v2.1 §8.6/Table 399 requires Operation when present in the request, so these remain `response_validation_failed`, not target rejection or interoperability success. Three ML-KEM Decrypt probes returned typed success, but their expected classification is `source_documentation_discrepancy`; each retains `interoperability=false`. The runner's sanitized line uses the broader `response_rejected` label for the Certify client error, while the matrix records the more specific response-shape failure. The underlying per-row status/reason fields and missing Operation are documented without retaining response bodies. A later complete serial ignored-suite run used the pinned `ghcr.io/cosmian/kms:5.28.0` digest; its sanitized log and summary are recorded in `task-6-report.md`. The full run ended with 98 passed and 10 failed, including 724 of 1,466 crypto rows not verified and all six live RNG rows rejected by typed response validation (Operation absent, Status 1, Reason 4). The new offline RNG classifier records those as `response_validation_failed`, with `request_sent=true`, `response_received=true`, and `interoperability=false`; the full run preceded this formatter fix and was not repeated. The transfer runner counted only 2,668 `interoperability_successes`; its 378 finding observations, 1,594 failures, and 50 Key Format Type postcondition failures do not increase that count. The 72 ConfigurableKEM transfer rows were exercised during the full pinned run, but their matrix observations remain pending case-by-case reconciliation. Final per-case evidence and outstanding blockers are listed in `task-6-report.md`.

The 230-ID transfer correction run superseded initial classifications for fixture and setup attribution. Its provisional response-validation results for four legacy PKCS12 cases were then superseded by the 32-ID PFX run. That final PFX run measured 16 successful unprotected-password cases and 16 password-linked setup rejections. For those 16 cases, Cosmian rejected the standard-shaped PFX Import setup with Result Reason 5 (`Operation Not Supported`); the dependent target Get/Export request was not sent. This is not a target-operation result. Exact final per-case records reside only in the matrix; intermediate run logs are local sanitized audit artifacts.

### PKCS#12 Password Link interoperability discrepancy

OASIS KMIP v2.1 §4.31, Tables 90–92, defines the `PKCS12PasswordLink` as a Link to a `SecretData` object. KMIPKit's typed fixture registers `SecretData` and places its returned Unique Identifier in the Link and the corresponding password selection. It does not send the password text in place of an identifier. In pinned Cosmian 5.28.0 source, `crate/server/src/core/operations/import.rs::process_pkcs12` converts the linked identifier to text and uses it as the PFX password instead of resolving the linked SecretData material; `crate/server/src/core/operations/export_get.rs` reads the outgoing PKCS12 password from the Key Wrapping Specification's Encryption Key Information identifier. This differs from the standard-shaped SecretData UID model. The matrix does not introduce a literal-password workaround. The exact request was rejected during Import setup, leaving the dependent Get/Export target operation untested for those cases.

The unprotected-password PFX rows succeeded. For Certificate-target Get/Export requests selecting PKCS12 or Cosmian legacy PKCS12, the typed response is a `PrivateKey` object containing the package; the response validator checks that object type and the returned Key Format Type. Cosmian normalizes the legacy output format to standard PKCS12 `0x16`. `research.md` contains the pinned source rationale and test limitations.

## Expanded operation coverage targets (not live results)

The exact new operation clauses and `ClientRequest` variants are in `spec.md`. The following sixteen supported paths have no new live evidence: Certify (§6.1.6, Tables 179–181), Check (§6.1.7, Tables 183–185), Derive Key (§6.1.14, Tables 205–207), Destroy (§6.1.15, Tables 208–210), Export (§6.1.18, Tables 217–219), Get (§6.1.19, Tables 220–222), Import (§6.1.25, Tables 238–240), Join Split Key (§6.1.27, Tables 244–246), Locate (§6.1.28, Tables 247–249), Register (§6.1.43, Tables 291–294), Revoke (§6.1.44, Tables 295–297), Re-key (§6.1.46, Tables 303–307), Re-key Key Pair (§6.1.47, Tables 310–312), RNG Retrieve (§6.1.49, Tables 316–318), RNG Seed (§6.1.50, Tables 319–321), and Validate (§6.1.57, Tables 340–342). Each requires its own stable cases and observed outcomes. Expanded variants of previously tested operations are also pending, not inherited successes.

The 34 published supported rows group Set/Modify into one row; the historical mapping below confirms two separate successful exchanges. Query is the nineteenth advertised path already exercised, with an interoperability defect. Ping is outside the advertised set. Re-certify's dispatch/support-table disagreement is an additional separately classified probe whose live result remains pending. Source disagreements (PFX omission, encoding/format gaps, Hash/SHA-224 and algorithm parameter differences) are recorded in `research.md` and must stay attached to concrete matrix rows.


## Historical OASIS clause-to-test mapping and observed results

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

Hosted verification for expanded head `7df810ab` completed as CI run
`38809840838`. All Core platform/toolchain jobs, C/Java/Python bindings,
script and documentation contracts, immutable-source inventory, fuzz smoke,
FFI/JNI sanitizers, language coverage, and the selected-scope aggregate
coverage gate passed. `Dependency policy / pull request` failed because the
unchanged lockfile resolves `rustls 0.23.45`, which has four RustSec advisories
issued on 2026-10-14 without registered exceptions: [RUSTSEC-2026-0346](https://rustsec.org/advisories/RUSTSEC-2026-0346.html),
[RUSTSEC-2026-0347](https://rustsec.org/advisories/RUSTSEC-2026-0347.html),
[RUSTSEC-2026-0348](https://rustsec.org/advisories/RUSTSEC-2026-0348.html), and
[RUSTSEC-2026-0349](https://rustsec.org/advisories/RUSTSEC-2026-0349.html).
RustSec records `>=0.23.46` as patched for these advisories. The integration
branch does not change `Cargo.toml` or `Cargo.lock`; the dependency update is
outside KMIPKIT-0015 and needs a separately approved dependency-security scope.
No advisory waiver was added. The draft PR remains open pending that scope and
a clean rerun of dependency policy.
