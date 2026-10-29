# Cosmian 5.28.0 exhaustive integration evidence

**Status: implementation and matrix execution complete; interoperability failures and final PR gates remain open.** The test suite exercises KMIPKit's typed Rust client over the pinned local binary TTLV/mTLS endpoint. A typed success counts as observed interoperability when the operation-specific response postconditions pass, including successful source-discrepancy probes; merely matching an expected finding does not count. This report does not claim complete KMIP 2.1 conformance or certification.

## Scope and measured totals

The pinned target is `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`, source commit `4ab3ee65829a903abb71f1298f400657fcee7ec9`. The authoritative per-case ledger is [`capability-matrix.json`](../../tests/integration/cosmian/capability-matrix.json). It contains 6,429 unique rows: 6,417 executable cases and 12 normative exclusions. Every executable row has an actual live classification or an explicit fixture/setup blocker; no executable row remains `not_run`. A targeted follow-up measured all 15 previously fixture-blocked cryptographic findings against the pinned server.

| Family | Rows | Typed success responses | Interoperability successes | Rejected | Setup rejected | Response validation failed | Other findings |
|---|---:|---:|---:|---:|---:|---:|---|
| Lifecycle and probes | 89 | 80 | 80 | 3 | 0 | 2 | 2 malformed; 1 setup blocked; 1 unsupported |
| Cryptographic | 1,466 | 670 | 650 | 383 | 325 | 81 | 7 semantic mismatches |
| Key management | 216 | 12 | 12 | 201 | 3 | 0 | — |
| Import/Export/Register transfer | 4,640 | 2,986 | 2,986 | 778 | 876 | 0 | — |
| RNG | 6 | 0 | 0 | 0 | 0 | 6 | — |
| **Executable total** | **6,417** | **3,748** | **3,728** | **1,365** | **1,204** | **89** | **7 semantic mismatches; 4 fixture/setup/malformed/unsupported** |

Of the 98 typed-success responses on source-gap/discrepancy rows, 24 Create Key Pair cases and 54 transfer cases now count as observed interoperability because their operation-specific postconditions passed. The remaining 20 cryptographic findings stay separately classified. The 24 Create Key Pair profiles remain source-documentation discrepancies because their algorithm values are outside the pinned KMIP 2.1 Cryptographic Algorithm enumeration; their success is not a KMIP 2.1 conformance claim. The 54 transfer findings retain their source/documentation classifications and likewise are not conformance claims. Rejection means a well-formed KMIP failure response; setup rejection means a prerequisite failed before the target operation was sent; response-validation failure means the request was delivered but KMIPKit could not accept or satisfy a typed-response postcondition; malformed response means required response shape was missing. These categories remain distinct in each matrix record.

The 12 normative exclusions are six invalid RNG Parameters combinations and six invalid Certify subject-OID-only CSR combinations. They were not sent. The exact request/fixture limitations and OASIS citations are recorded on the corresponding rows and in `specs/015-cosmian-kms-integration/research.md`.

## Operation-level interoperability summary

These summaries count only executable cases whose typed response and operation-specific postconditions passed. The denominator is the number of executable cases for that operation; standards-invalid normative exclusions are omitted. The per-case ledger remains authoritative.

### Operations with observed interoperable cases

| KMIP 2.1 operation | Interoperable / executed cases |
|---|---:|
| Activate | 5 / 6 |
| Add Attribute | 6 / 7 |
| Check | 7 / 7 |
| Create | 70 / 72, plus 2 typed DSA successes ✅† |
| Create Key Pair | 57 / 59 ✅‡ |
| Create Split Key | 9 / 9 |
| Decrypt | 222 / 234 |
| Delete Attribute | 7 / 7 |
| Destroy | 7 / 7 |
| Discover Versions | 1 / 1 |
| Encrypt | 287 / 300 |
| Export | 807 / 1,291 |
| Get | 797 / 1,279 |
| Get Attribute List | 7 / 7 |
| Get Attributes | 7 / 7 |
| Hash | 14 / 24 |
| Import | 645 / 1,041 |
| Join Split Key | 3 / 3 |
| Locate | 6 / 7 |
| Register | 737 / 1,029 |
| Revoke | 7 / 7 |
| Set Attribute (Modify) | 16 / 16 |
| Validate | 4 / 6 |

† Both DSA `Create` cases (2048 and 3072 bits) returned typed successful responses and passed their operation-specific postconditions. Cosmian's support documentation does not list these cases, although its implementation accepts them. This is an observed implementation/documentation discrepancy, not evidence of a broader DSA support profile.

‡ The 57 `Create Key Pair` successes comprise 28 cases using standard enumerated values, 5 using vendor extensions, and 24 source-only probes using numeric values outside the KMIP 2.1 enumeration. Those 24 are observed interoperability with Cosmian's implementation, not KMIP 2.1 conformance for the values. The two rejected cases use ConfigurableKEM with P-256 and Curve25519; both returned `Operation Failed / Cryptographic Failure`.

For `Set Attribute (Modify)`, the matrix exercises Set Attribute and Modify Attribute separately: eight cases each, all successful.

### Published as supported, with no interoperable case yet

| Operation | Interoperable / executed cases | Observed result |
|---|---:|---|
| Certify | 0 / 203 | All 203 target requests were rejected with `Operation Not Supported`. Six standards-invalid CSR cases were not sent and are excluded from the denominator. |
| Derive Key | 0 / 129 | All 129 target requests were rejected with `Operation Not Supported`. |
| MAC | 0 / 36 | The target operation was never sent: 9 cases were blocked by prerequisite setup rejections; the other 27 HMAC fixture Import responses omitted `Batch Item Operation`, so KMIPKit rejected setup. |
| MAC Verify | 0 / 54 | The target operation was never sent: all 54 HMAC fixture Import responses omitted `Batch Item Operation`, so KMIPKit rejected setup. |
| Query | 0 / 1 | Cosmian returned success but omitted required `Protection Storage Masks`; KMIPKit rejected the typed response. |
| RNG Retrieve | 0 / 4 | All four executable requests received error responses missing `Batch Item Operation`. Four invalid-parameter cases were not sent and are excluded. |
| RNG Seed | 0 / 2 | Both executable requests received error responses missing `Batch Item Operation`. Two invalid-parameter cases were not sent and are excluded. |
| Re-key | 0 / 40 | All 40 target requests were rejected with `Operation Not Supported`. |
| Re-key Key Pair | 0 / 35 | 32 target requests were rejected with `Operation Not Supported`; 3 cases were rejected during prerequisite setup with `Cryptographic Failure`, before the target operation was sent. |
| Sign | 0 / 163 | All target requests returned `Item Not Found`; this response alone does not establish that the operation is unimplemented. |
| Signature Verify | 0 / 321 | 312 cases were rejected during prerequisite setup with `Item Not Found`, before the target operation was sent; the other 9 target requests returned `Item Not Found`. |

A focused live rerun of the 52 previously rejected native SymmetricKey Import cases passed 52/52 after correcting two integration-builder tags: Symmetric Key is `0x42008F` (the old `0x42007F` is Result Status), and the transparent key material child is `Key` `0x42003F` (the old `0x420044` is Key Part Identifier). The defect was in KMIPKit integration test construction; no production-library or Cosmian change was needed. The ledger and operation totals include these 52 successes. These are measured outcomes against the pinned local Cosmian KMS 5.28.0 binary TTLV/mTLS deployment. The findings and their attribution are tracked per case in [`capability-matrix.json`](../../tests/integration/cosmian/capability-matrix.json); an observed failure is not automatically attributed to KMIPKit or Cosmian without supporting evidence.

### Register retest and attribution

The 446 Register rows that had not previously counted as interoperable were rerun serially against the pinned Cosmian 5.28.0 image with KMIPKit's typed Rust client. The run completed all 446 rows and the Compose deployment was removed afterward. It recorded 154 typed Register successes, 286 prerequisite failures before Register was sent, and six Register target rejections. Register therefore now has **737 / 1,029** interoperable cases. The updated per-case request and response status is in the capability matrix.

| Observed stage/result | Cases | Attribution |
|---|---:|---|
| Typed Register success | 154 | Includes 52 former `Invalid Message` target failures caused by incorrect SymmetricKey/Key tags in KMIPKit's integration fixture builder. Correcting the test TTLV made those requests valid; no production-library defect was involved. The other 102 rows previously blocked by `Cryptographic Failure` setup now completed their typed postconditions in the rerun. |
| Setup `Operation Not Supported` | 165 | 143 failed in prerequisite Export when explicitly combining PKCS#8 with wrapping; pinned Cosmian source rejects that combination. We tried omitting the format selector from one wrapped setup Export, but For this Ed25519 PrivateKey row, Cosmian returned default KeyFormatType 20 (Transparent EC Private Key) rather than the requested PKCS#8, so that result could not be used as an equivalent Register fixture and the target was not sent. An independent wrapped PKCS#8 fixture would be needed to isolate Register. OASIS KMIP v2.1 §4.26, Tables 79–80 permits a requested format and requires the server to comply or fail, but does not require every format conversion to compose with every wrapping profile. Six ConfigurableKEM public-key formats are not listed in Cosmian's format documentation. The remaining 16 failed during prerequisite Create Key Pair: 12 ConfigurableKEM generation profiles and four DSA generation profiles. Register itself was never sent. |
| Setup `Cryptographic Failure` | 115 | 111 prerequisite Export cases used direct RSA wrapping with TTLV encoding: 60 Ed25519 keys in transparent-EC formats, 32 AES SymmetricKeys and 19 SecretData objects. The exact underlying cryptographic cause is not exposed; these are wrapping/setup results, not Register failures. KMIP 2.1 §6.1.18–§6.1.19 error tables do not list `Cryptographic Failure`, so Cosmian should confirm the intended error mapping. Four more cases failed during prerequisite ConfigurableKEM Create Key Pair for pure-classical P-256/Curve25519 profiles. |
| Setup `Sensitive` | 6 | For each ConfigurableKEM private key, KMIPKit sent typed Create Key Pair with `Sensitive=False`; a subsequent typed Get Attributes request for the same returned identifier read `Sensitive=False`. The following unwrapped Export still returned Status 1 / Reason 22 (`Sensitive`), so Register was not sent. This is an observable inconsistency in Cosmian 5.28.0's attribute/readback and Export behavior, not a TTLV construction or KMIPKit response-decoding failure. OASIS §4.55, Tables 139–140 says a true value prevents unwrapped retrieval and marks the attribute client-modifiable. |
| Register target `Operation Not Supported` | 2 | Password-protected PKCS#12 cases reached Register and were rejected. Their Password Link uses the KMIP `SecretData` identifier required by §4.31, Tables 90–92; pinned Cosmian PKCS#12 processing treats the identifier text as the password. This is the previously documented Cosmian/standard discrepancy. |
| Register target `Cryptographic Failure` | 4 | X25519MLKEM768/X448MLKEM1024 PrivateKey/PublicKey requests reached Register and were rejected. These are Cosmian vendor algorithm values, not standard KMIP 2.1 enum values; operation-level Register support does not document support for those algorithm/format intersections. Cosmian documents the hybrid algorithms and marks Register implemented, but does not explicitly document this Register/object-format intersection. Also, `Cryptographic Failure` is not listed in OASIS Register Table 294. Ask Cosmian whether these vendor objects should be accepted and why it returns that reason; the observed responses do not indicate a KMIPKit parsing defect. |

The Register target and setup distinction follows OASIS KMIP v2.1 §6.1.43, Tables 291–294: a failed prerequisite means the Register request was never evaluated. The six `Sensitive` cases are recorded with the explicit false readback in the ledger. The exact pinned implementation paths are [`export_get.rs`](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/export_get.rs#L2277-L2352), [`create_key_pair.rs`](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/create_key_pair.rs#L2047-L2170), and [`import.rs`](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/import.rs#L3354-L3395). The six attribute/Export mismatches and four hybrid vendor failures should be raised with Cosmian; the records do not establish a KMIPKit library defect.

## Full serial run and teardown

The complete ignored suite ran serially against the pinned image with:

```powershell
cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1 --nocapture
```

The latest complete serial run used the pinned image above and rustls 0.23.46. Its sanitized ignored-local log is `.local/cosmian-kms/final-rustls-0.23.46-full-suite.log`, SHA-256 `84947A36B3557C22402DA12F918898F1743F913E4153A4A025120A7E4B2C0051`. It ended `101 passed; 9 failed; 0 ignored; 0 measured; 109 filtered out; finished in 6383.81s`. The nine failed test functions were `cryptographic::cosmian_528_cryptographic_matrix_serial_live_runner`, `key_management::cosmian_528_key_management_matrix`, `lifecycle::cosmian_528_activate_splitkey`, `lifecycle::cosmian_528_add_attribute_splitkey`, `lifecycle::cosmian_528_locate_splitkey`, `lifecycle::cosmian_528_validate_certificate_certificate_values_current`, `lifecycle::cosmian_528_validate_certificate_certificate_values_explicit_time`, `random::cosmian_528_random_matrix_no_parameter_cases`, and `transfer::cosmian_528_transfer_matrix`. The output contained 6,412 distinct `case_id` values and the five standalone lifecycle cases above; four Validate IDs have a second event/classification line, not a conflicting observation. The six RNG rows and run output use `response_validation_failed`: each response had Status 1 / Reason 4 and omitted Operation, so no typed RNG success was recorded. Together, these reconcile all 6,417 executable rows. Compared with the previous full run (98 passed / 10 failed in 6,116.15 seconds), the latest run incorporates the RNG observation contract and its additional offline test. Targeted follow-ups remain part of the ledger. The full-run output retained no raw TTLV or secret material. Compose teardown completed; `docker compose --file tests/integration/cosmian/compose.yaml ps --all` returned no services. After that run, the targeted `cosmian_528_cryptographic_matrix_serial_live_runner` selected the 15 cipher finding IDs from the ledger and passed with 12 typed ConfigurableKEM results and three AES/RSA rejections. Its exit code was 0, Compose teardown succeeded and `ps --all` was empty.

## Measured outcomes and limitations

### Lifecycle and operation probes

- **Discover Versions:** typed success with KMIP version `(2, 1)`.
- **Check SplitKey:** typed success after removing an optional usage mask that caused fixture setup rejection. The earlier full-run Reason 41 result belongs to the superseded fixture and is not the final matrix observation.
- **Activate SplitKey:** setup blocked because the generated SplitKey did not retain the requested future Activation Date.
- **Add Attribute SplitKey:** target rejected with Status 1, Reason 45 (`Invalid Attribute Value`).
- **Locate SplitKey:** KMIP success returned zero matching case-owned identifiers; typed postcondition failed.
- **Validate Certificate Values, current and explicit time:** target rejected with Status 1, Reason 5 (`Operation Not Supported`).
- **Query:** Cosmian returned Success and Operations but omitted Protection Storage Masks required by OASIS KMIP v2.1 §6.1.40, Table 283; KMIPKit rejected the typed response.
- **Ping and Adjust Attribute:** failure responses had Status 1, Reason 4 (`Invalid Message`) and omitted Batch Item Operation required by §8.6, Table 399; these are malformed responses, not successful unsupported-operation exchanges.
- **Re-certify:** the separate source-discrepancy probe returned a well-formed Status 1, Reason 5 (`Operation Not Supported`). It does not add to the vendor-advertised operation set.

### Cryptographic operations and RNG

The 1,466 crypto rows include 1,220 expected typed-success cases, six source gaps and 240 documentation/source discrepancies. Offline typed builders construct all expected-success requests; exact finding preflight serializes 246 requests (284 TTLV payloads including multipart rows). All 30 post-quantum Certify finding cases have encoding-matched CSR fixture paths. The reconciled live results are 650 interoperability successes, 20 other typed responses on finding rows, 383 target rejections, 325 setup rejections, 81 response-validation failures, and seven typed semantic mismatches. The three standalone AES-ECB Encrypt cases also pass with block-aligned fixtures, bringing Encrypt to 287/300 interoperable cases. Create Key Pair recorded 57/59 interoperability successes: 33 previously counted cases plus 24 source-discrepancy cases whose typed response contained both validated key identifiers; two ConfigurableKEM curve-only profiles returned Cryptographic Failure. A targeted run sent the 15 formerly blocked cipher findings: all 12 ConfigurableKEM profile/direction combinations completed typed operation-specific postconditions; three AES/RSA requests were rejected by the target after delivery. The Decrypt subset was separately rechecked: three AES-ECB no-padding round trips and CoverCrypt Decrypt now pass; two pure-classical ConfigurableKEM Decrypt cases are not sent because prerequisite Create Key Pair returns Cryptographic Failure; the RSA/AES Decrypt request is sent but rejected as Operation Not Supported. The test's RSA-OAEP ciphertext is only a probe payload, not a genuine RSA-OAEP/AES-GCM hybrid ciphertext. Those finding rows remain separately classified.

### Encrypt ECB case re-evaluation

Three standalone AES-ECB Encrypt rows (128-, 192-, and 256-bit keys) were rerun against the pinned Cosmian 5.28.0 service after correcting the plaintext fixture. The previous 19-byte input was not a full AES block while ECB used its default no-padding mode; the corrected fixture sends exactly 16 bytes. All three Encrypt requests returned typed success and passed their operation-specific postconditions. This was a KMIPKit integration-test fixture defect, not a production-client or Cosmian interoperability failure. The matrix now records Encrypt at 287/300: six successful typed ConfigurableKEM opaque-fixture rows and three ML-KEM source-only enum rows remain outside the advertised KMIP 2.1 interoperability count; two RSA/AES requests were rejected with Operation Failed / Operation Not Supported (Status 1, Reason 5), matching the documented/source discrepancy; and two pure-classical ConfigurableKEM cases fail during prerequisite key creation with Cryptographic Failure before Encrypt is sent.

### Decrypt case re-evaluation

The seven cases that had appeared as Decrypt failures were rerun against the pinned 5.28.0 server after checking the fixture and the pinned source paths:

| Cases | Current result | Attribution |
|---|---|---|
| AES-ECB 128/192/256, no explicit padding | 3/3 Encrypt/Decrypt round trips pass with a 16-byte fixture | KMIPKit test-fixture defect. ECB defaults to no padding, so the former 19-byte input was invalid. The server's 5.28.0 encrypt/decrypt code defaults ECB to `PaddingMethod::None`. |
| CoverCrypt | Decrypt passes and plaintext matches | KMIPKit integration assertion defect. Cosmian returns a CoverCrypt data container with a LEB128 metadata-length prefix; the fixture had compared the whole container directly with plaintext. The 5.28.0 example documents the corresponding length/attributes/payload framing. |
| ConfigurableKEM P-256-only and Curve25519-only | Both setup attempts return `Operation Failed / Cryptographic Failure` from Create Key Pair; target Decrypt was not sent | Not a Decrypt request or client serialization failure. Cosmian 5.28.0 source explicitly maps curve-only configuration to the pure-classical keygen branch, while its generic wire error does not reveal the internal crypto cause. This is a Cosmian-side setup/provider issue candidate, not yet a proven server defect; server diagnostics are needed to identify the internal cause. |
| RSA-key/AES-algorithm hybrid probe | Target Decrypt returns `Operation Failed / Operation Not Supported` | Cosmian documentation is internally inconsistent: it says the RSA/AES scheme is only for wrapping, then prescribes `Encrypt/Decrypt` with an RSA key and `Cryptographic Algorithm = AES`. Pinned source dispatches RSA-key Decrypt only for `Cryptographic Algorithm = RSA`; AES fails before ciphertext processing. The request fixture is ordinary RSA-OAEP ciphertext, not a valid AES-GCM/RSA hybrid ciphertext, so the row demonstrates the documented algorithm-selection rejection but does not establish behavior for a valid hybrid payload. |

The first two issues are fixed in the KMIPKit integration harness and their ledger rows now count as successful interoperability. No KMIPKit production-library changes were required. The three ECB and CoverCrypt live cases pass; Compose teardown leaves no server running. Cosmian's versioned [encryption algorithm documentation](https://docs.cosmian.com/versions/kms/5.28.0/certifications_and_compliance/cryptographic_algorithms/algorithms.html) describes the RSA/AES scheme and pure/hybrid KEM modes; its [Encrypt example](https://docs.cosmian.com/versions/kms/5.28.0/kmip_support/_encrypt.html) shows CoverCrypt's data framing. The implementation evidence is pinned to source commit `4ab3ee65829a903abb71f1298f400657fcee7ec9` in `crate/server/src/core/operations/{encrypt,decrypt}.rs` and `crate/crypto/src/crypto/kem.rs`.

The 81 crypto response-validation failures are HMAC prerequisite responses where the peer omitted
Operation (Status 1, Reason 4). The seven multipart Hash cases are classified separately: the server
returned typed Success responses, but each final digest differed from the independent known-answer
digest for the three input blocks a, b, and c. KMIPKit accepts the valid multipart response shape
and the test preserves the first response's Correlation Value; these observations are semantic
mismatches, not parser failures or interoperability successes. Cosmian 5.28.0's pinned hash.rs uses
the Correlation Value as additional input to a fresh digest and does not return an updated value
after the intermediate part, which is consistent with the observed wrong final digest. Three SHA-224
single-part requests remain target rejections (Operation Not Supported): the versioned Cosmian
algorithm page lists SHA-224, but the pinned Hash dispatch does not implement it. OASIS KMIP v2.1
§6.1 defines multipart sequencing, §7.8 defines Correlation Value reuse, and Hash §6.1.24, Tables
235–237 define request/response fields. Table 235 marks Hash Data as not required for multipart,
while general §6.1 text says Data is always required except when Init or Final Indicator is true.
The live test supplies Data on all parts and does not claim conformance for the disputed omission of
Data from an intermediate Hash request. All six valid length-only RNG cases returned typed responses missing
Operation (Status 1, Reason 4) and remain response_validation_failed. OASIS KMIP v2.1 §6.1.49/Table
316 and §6.1.50/Table 319 define the RNG request payloads; §8.6/Table 399 governs the response
Operation field. No RNG bytes, seeds, private keys, passwords, or response bodies are recorded.

### Key management and transfer

Key management has 12 interoperability successes, 201 target rejections and three setup rejections across 216 cases.

Transfer now has 2,986 interoperability successes, 778 target rejections, 876 setup rejections and no response-validation failures across 4,640 rows after the focused Register retest. The full run exercised all 72 ConfigurableKEM opaque transfer cases: 48 prerequisite setup rejections and 24 target rejections. These rows are not labeled fixture-unavailable.

| Operation | Interoperable / executable | Target rejections | Setup rejections | Target rejection breakdown | Setup rejection breakdown |
|---|---:|---:|---:|---|---|
| Get | **797 / 1,279** | 382 | 100 | 160 `Operation Not Supported`; 8 `Sensitive`; 214 `Cryptographic Failure` | 78 prerequisite `Invalid Message` replies missing `Operation`; 16 `Operation Not Supported`; 6 `Cryptographic Failure` |
| Export | **807 / 1,291** | 384 | 100 | 160 `Operation Not Supported`; 8 `Sensitive`; 216 `Cryptographic Failure` | 78 prerequisite `Invalid Message` replies missing `Operation`; 16 `Operation Not Supported`; 6 `Cryptographic Failure` |

The previous 50 Get/Export response-validation failures were caused by the integration harness omitting Cosmian's vendor Key Format Type selector `0x88800001`. A focused run selected 66 cases: 62 target requests returned typed success, including all 58 cases now sending the selector; four cases did not reach Get/Export because password-linked PFX Import setup was rejected. Sixteen password-linked PFX setups remain rejected before the dependent operation, while 16 unprotected-password PFX cases succeeded. OASIS KMIP v2.1 §4.31, Tables 90–92 defines Password Link as a reference to `SecretData`; the matrix does not substitute literal password text. The [Get/Export attribution tables](traceability.md#get-and-export-attribution) include per-profile documentation checks, server-source evidence and follow-up questions for Cosmian.

## Test development evidence

- Manifest inventory Red/Green: signed commits `34d9d3a1`, `d70cc950`, and `2c3edc5a` record the initial missing-inventory failure, inventory correction, and typed exhaustive crypto implementation. The inventory validator now passes over 6,429 rows.
- Crypto dispatch and typed-construction contracts cover every applicable crypto case. Signed Red commit `a44bcdf3` adds a regression test requiring typed probes for the 15 cipher findings; Green commit `060c7fcd` provisions an ordinary RSA-OAEP ciphertext solely as a probe payload for the AES/RSA Decrypt rejection. The seven-case Decrypt follow-up corrected CoverCrypt result framing and the ECB block alignment in distinct Red/Green/Refactor commits; live retests pass those four rows. The pinned targeted run observed all 15 cipher findings and Compose teardown left no services. The later lint refactor is `dedbf1e5`.
- RNG malformed-response and peer-rejection classifications have distinct signed Red/Green/Refactor commits `b6f9a54d`, `7fbc91cd`, `f360ab75`, `b14587e1`, and `4f76d547`.
- Adjust Attribute and Re-certify lifecycle parity has signed Red commit `dbe00bc9` and Green implementation commit `c67d0fe4`; the two probes were rerun against the pinned deployment and are separately classified in the matrix.
- The completeness guard has signed Red commit `422dc6ba` (the regression test failed because an executable `not_run` row was accepted) and Green commit `66ce4239` (executable rows now reject `not_run`; normative exclusions remain allowed).
- Clean-checkout CI regression, Red/Green: run `39006703175` failed the offline Cosmian inventory tests on Linux, macOS and Windows because assertions assumed the ignored runtime-generated certificate/CSR directory existed. Green commit `8e81b1bb` validates the 30 PQC CSR-dependent probes against the actual fixture set, requires that set to be complete or absent, and keeps certificate path mapping checked when runtime files are absent. With no generated CSR fixtures, preflight is 216 runnable / 30 `NotSent`; with generated CSR fixtures, it is 246 / 0. The 15 earlier cipher blockers are now exact target probes and require no local private-key fixture. Targeted Clippy and formatting checks pass.
- Latest offline command `cargo test --offline -p kmipkit-client --test cosmian_kms` passed 118 tests, failed zero and ignored 110 live tests. (Cargo emitted existing AWS-LC MSVC stdalign/NASM environment warnings that did not prevent the command from completing.)
- Final local verification on the cipher-probe implementation: `cargo fmt --all --check`, `cargo test --offline --workspace --all-features`, and `cargo clippy --offline --workspace --all-targets --all-features -- -D warnings` passed. `cargo llvm-cov --workspace --all-features --summary-only` reported 92.93% workspace line coverage (above the 90% workspace gate).
- Contract and integration verification passed: requirement traceability and pinned-image Compose
  checks (3 tests), normative inventory and generated-output checks in hosted CI, PowerShell/Compose
  configuration, git diff --check, and empty Compose status after the targeted run. Hosted CI run
  39049893049 passed on the exact matrix head 5c3e4952be02422bff7149256efc617cea12692a across Core,
  bindings, sanitizers, scripts, documentation, normative inventory, dependency policy and aggregate
  coverage. The earlier rustls 0.23.45 dependency-policy issue was resolved by the approved
  KMIPKIT-0045 remediation merged into release/1.0.0; this branch resolves rustls 0.23.46 and adds
  no dependency waiver. Fresh local RustSec and hosted Dependency policy checks passed. The
  branch includes a focused citation-regression test, and the corrected citations require ordinary
  PR CI on the updated head.

## Remaining release/review gates

Keep PR #56 in draft. Hosted CI run 39049893049 passed on the exact matrix head
5c3e4952be02422bff7149256efc617cea12692a, including dependency policy, all platform checks,
sanitizers, bindings, normative inventory and aggregate coverage. The source-citation correction
will receive ordinary PR CI on its updated head. The exhaustive live run retains nine failed test
functions; these target/interoperability outcomes remain recorded. The
two-independent-KMIP-2.1-implementation release gate remains open.

## CodeQL test-diagnostic follow-up — 2026-10-29

The exact PR head `ba066573316f75b76e92093b5e77fb65f0b85638` produced two CodeQL alerts:
dynamic preflight diagnostics in `cryptographic.rs` and a fixed caller nonce in
`cryptographic_cipher.rs`. Independent source review found no production secret flow: the
preflight result contains case-matrix validation metadata, and each old fixed nonce was used for
one Encrypt with a fresh disposable key, then reused only for its matching Decrypt. These findings
were confined to ignored integration-test code.

Red commit `8d7c32168` added a test requiring independently generated caller nonces; it failed
against the fixed value. Green commit `c014b2e97` uses AWS-LC system randomness with a 64-byte
input limit and replaces the dynamic preflight error text written to logs with fixed diagnostics.
Local verification passed: `cargo fmt --all --check`, the focused nonce regression, the complete
Cosmian test binary (122 passed, 110 ignored live tests), and client Clippy with `-D warnings`.
The expected AWS-LC build warnings on Windows did not fail the build. The updated PR head still
requires hosted CodeQL and ordinary CI confirmation. This evidence does not change the measured
interoperability outcomes or establish a KMIP conformance claim.

## PKI output ownership hardening (2026-10-29)

The independent review found a low-risk local data-loss path when an operator
pointed `KMIPKIT_COSMIAN_CERT_DIR` at an unrelated directory containing the
generator's `ca`, `server`, `client`, or `validation` subdirectories. The PKI
generator now writes only into an empty output directory or one carrying its
own marker, and it rejects an unmarked non-empty directory before cleanup.
This is local safety hardening; it does not change the KMIP test results.

| Phase / check | Evidence | Result |
| --- | --- | --- |
| Red | DCO commits `f8b366739`, `84e711391`, `cd3d0d972`, and `fc805c484` | Expected failures covered missing ownership checks, non-portable `find` flags, a complete unmarked directory bypassing the guard, symlinked cleanup subdirectories, and a symlinked ownership marker. |
| Green | DCO commits `a669252b4` and `ef0a1d8b1` | Generator validates the marker before the complete-output fast path; rejects non-empty unmarked roots, marker/root/subdirectory symlinks, and non-directory cleanup paths; uses portable shell globs; and quotes generated output paths. |
| Shell regression | `bash -n tests/integration/cosmian/generate-pki.sh`; `python3 -B -m unittest scripts.tests.test_cosmian_compose -v` under WSL | PASS: all 9 focused tests, including empty/marked-complete output, unmarked-complete rejection, non-empty sentinel preservation, symlinked marker/subdirectory rejection, and portable shell guard. |
| Windows script contracts | `py -3 -B -m unittest scripts.tests.test_cosmian_compose -v` | PASS; the six POSIX runtime cases skip on Windows and execute under WSL and hosted POSIX CI. |
| Full script contracts | `py -3 -B -m unittest discover --failfast -s scripts/tests -p 'test_*.py'` | PASS: 291 tests, 32 platform-specific skips. The six POSIX-only ownership runtime cases pass under WSL. |
| Refactor / documentation | Current change in `scripts/tests/test_cosmian_compose.py`, the integration README, this report, and the quickstart | Adds a runtime regression on POSIX and documents the empty-or-marked directory requirement. |
| Independent security re-review | Read-only review of HEAD `ef0a1d8b1` and the regression cases | PASS: no remaining actionable issue; the marker and path checks precede both the complete-output return and destructive cleanup. |

The independent re-review findings were addressed by `ef0a1d8b1`; the final
post-fix re-review is complete. Hosted CodeQL and CI still need to finish after
publication. T047 remains open until those checks complete.
