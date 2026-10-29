# Requirement Traceability: Cosmian KMS Integration

The exhaustive matrix is implemented and reconciled against the latest complete serial ignored-suite output and targeted live follow-ups. The manifest contains 6,429 rows: 6,417 executable combinations with individual observed classifications and 12 normative exclusions that were not sent. It records 3,728 successful typed interoperability outcomes; all other results remain separately classified. The latest full test binary run with rustls 0.23.46 ended with 101 passed and 9 failed test functions. Subsequent targeted follow-ups corrected four Decrypt fixture/postcondition issues, measured the 15 former cipher fixture blockers, reran the Get/Export legacy-PKCS#12 cases, and reran all 446 previously non-interoperable Register rows. This remains an evidence report, not a claim that all cases succeeded or that Cosmian is fully conformant. Historical selected-case runs remain separate bounded evidence. A live follow-up reran all 52 previously rejected native SymmetricKey Import cases after correcting the integration test builder tags for Symmetric Key and transparent Key; all 52 returned typed success.

| Requirement | Source | Implementation | Verification |
|---|---|---|---|
| `KMIPKIT-0015-FR-001` | `spec.md`; Cosmian KMS 5.28.0 configuration; project transport boundary | `tests/integration/cosmian/compose.yaml`, `cosmian-kms.toml` | `docker compose config`; deployment status confirms local service after deployment |
| `KMIPKIT-0015-FR-002` | `spec.md`; project secret-handling policy (`AGENTS.md` §8) | `tests/integration/cosmian/generate-pki.sh`, `.gitignore` | Shell syntax check; confirm generated path is ignored and private keys are not in the KMS bind mount |
| `KMIPKIT-0015-FR-003` | OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; §9.16, Table 421 | `crates/kmipkit-client/tests/cosmian_kms.rs` | Explicit ignored live run; assert success and returned `(2, 1)` |
| `KMIPKIT-0015-FR-004` | `spec.md`; project default-test isolation policy | `#[ignore]` on all live tests in `crates/kmipkit-client/tests/cosmian_kms.rs`; `scripts/integration/cosmian-kms.ps1`; quickstarts | Ordinary Cargo tests skip these; explicit script runs them serially |
| `KMIPKIT-0015-FR-005` | `spec.md`; test ownership and secret/no-retry policy (`AGENTS.md` §8); Import §6.1.25, Tables 238–240; Destroy §6.1.15, Tables 208–210 | Disposable SQLite, per-case-owned imports/objects and cleanup ledger | 4,640 transfer outcomes recorded; cleanup regression tests pass; serial Compose teardown completed and status was empty |
| `KMIPKIT-0015-FR-006` | Evidence-backed interoperability reporting policy | This document, `quickstart.md`, `tests/integration/cosmian/README.md` and capability matrix | Successful typed postconditions are counted separately; all rejections, setup failures, malformed/invalid responses and blockers remain visible per case |
| `KMIPKIT-0015-FR-007` | OASIS KMIP Specification v2.1 §8.4, Table 397; §11.56 | `crates/kmipkit-transport/src/raw_tls.rs` | Raw-TLS tests accept Response Message `0x42007B` and reject Request Message `0x420078`; live Discover Versions passed |
| `KMIPKIT-0015-FR-008` | OASIS operation contracts in `spec.md`; §11.56, Table 487 | Existing typed `ClientRequest` API; operation-family modules and ignored matrix executors | Offline parity covers all executable records; the complete serial run and focused probes record target outcomes without hand-encoded TTLV |
| `KMIPKIT-0015-FR-009` | Attribute operation clauses below; project isolation rule | Each lifecycle case uses a case-owned fixture and unique identifiers | Offline lifecycle ownership/cleanup regression tests pass; per-case live results are in the matrix |
| `KMIPKIT-0015-FR-010` | Exact OASIS operation/data registry in `spec.md`; evidence policy | This document and `tests/integration/cosmian/capability-matrix.json` | 6,429 unique rows; 6,417 executable rows individually classified; historical selected-case results remain separate |
| `KMIPKIT-0015-FR-011` | All 35 operation clauses in `spec.md`; Set §6.1.51, Tables 322–324; Modify §6.1.34, Tables 265–267; Query §6.1.40, Tables 282–284; separate probes §6.1.3/§6.1.36/§6.1.45 | Complete inventory and ignored typed family cases; Set/Modify remain separate and Query/probes retain separate classifications | Offline exact-path, schema, case/dispatch validators pass; all 6,417 executable rows have observations or explicit blockers; Query remains a required-field response failure |
| `KMIPKIT-0015-FR-012` | OASIS §2.1, Table 2; §2.3, Table 4; §§2.5–2.9, Tables 6–10, crossed with each scoped operation contract | Source-derived object intersections, exclusion and disagreement ledger in the JSON manifest and `research.md` | 6,429-row inventory validator and operation-family parity tests pass; applicable intersections were executed or explicitly blocked |
| `KMIPKIT-0015-FR-013` | OASIS §§3.1–3.2, Tables 11–13; §4.26, Tables 79–80; §11.25, Tables 460–461; §2.1, Table 2; §4.39, Tables 107–108; §4.31, Tables 90–92; §11.28, Tables 464–465 | Import/Export formats, generated PFX fixture and pinned vendor format/implementation sources in `research.md` | Default/explicit formats and applicable PEM/DER/PFX paths are individually inventoried and measured or blocked; PFX table omission and wire/fixture distinctions are documented |
| `KMIPKIT-0015-FR-014` | OASIS §§4.13–4.16, Tables 53–60; §11.12, Table 443; §11.45, Table 477; §11.6, Table 436; §11.16, Table 447; §11.21, Table 454; §11.30; §11.37, Table 471; derivation §7.12, Table 364/§11.14, Tables 445–446; wrapping §3.3, Tables 14–17/§7.18, Table 370/§11.18, Tables 449–450/§11.29/§11.62, Tables 493–494 | Explicit algorithm/parameter rows and typed crypto pairs; source gaps/discrepancies retain their own dispositions | Offline builders cover all 1,466 crypto rows and serialize all 246 exact finding probes; live matrix records 650 crypto interoperability successes, 20 remaining typed finding responses, and every rejection, setup failure and response-validation failure |
| `KMIPKIT-0015-FR-015` | Project case isolation, ownership, no-retry and teardown policy; applicable operation cleanup contracts | Case-owned helpers/object tracking, reverse-order cleanup and runner teardown | Offline cleanup/delivery regression tests pass; the full serial run tore down Compose and `ps -a` returned no services |
| `KMIPKIT-0015-FR-016` | Project traceability/reporting policy and approved matrix design; exact OASIS per-case clauses | Versioned manifest with stable IDs, sources, expected/outcome fields and sanitized reports | Offline completeness and case/test parity pass; 6,417 executable rows carry observed results or explicit fixture/setup blockers; no raw response bodies are retained |

| Success criterion | Requirement links | Verification target and status |
|---|---|---|
| `KMIPKIT-0015-SC-001` | FR-001, FR-002 | Pinned loopback Compose deployment and teardown were exercised in the serial run. |
| `KMIPKIT-0015-SC-002` | FR-004, FR-008, FR-016 | All live tests remain ignored by default; the latest offline integration target passed 118 tests with 110 live tests ignored. |
| `KMIPKIT-0015-SC-003` | FR-006, FR-008, FR-011 | Inventory covers all 35 advertised paths, separates Set/Modify, and records Query/Ping/Adjust/Re-certify findings individually. |
| `KMIPKIT-0015-SC-004` | FR-006, FR-010, FR-016 | Each of 6,417 executable rows has a live classification or explicit blocker; 3,728 successful typed postconditions count as observed interoperability. Source-only values remain separate from KMIP 2.1 conformance claims. The 15 cipher findings and all 446 previously non-interoperable Register rows have measured outcomes. |
| `KMIPKIT-0015-SC-005` | FR-011–FR-014, FR-016 | Offline schema, unique-ID, source-reference, dimension and case-to-executor checks pass over the 6,429-row manifest. |
| `KMIPKIT-0015-SC-006` | FR-005, FR-013, FR-016 | Import/Export defaults, explicit formats, PEM/DER and PFX paths are represented; legacy-format Key Format Type failures and PFX source omission remain explicit. |
| `KMIPKIT-0015-SC-007` | FR-014, FR-016 | All crypto and valid RNG rows have live classifications or explicit fixture blockers; failed round trips, verification pairs and target behaviors remain failures. |
| `KMIPKIT-0015-SC-008` | FR-002, FR-005, FR-008, FR-009, FR-015 | Cases own fixtures and linked objects; cleanup/delivery regression tests pass and no response bodies or secrets are retained. |
| `KMIPKIT-0015-SC-009` | FR-005, FR-015 | Full serial run completed Compose teardown and `ps -a` was empty. The final PR gate set remains open while required workspace/hosted checks are evaluated. |
## Complete observed matrix — pinned Cosmian 5.28.0

The manifest contains 6,429 unique records: 6,417 executable rows and 12 standards-invalid/non-executable exclusions. Each executable row has a live observation or an explicit fixture/setup blocker; none remains `not_run`. Of the executable rows, 3,728 passed the typed response and operation-specific postconditions and are counted as observed interoperability successes. A successful expected-finding probe may count when those postconditions pass; the expected/source-discrepancy classification remains separately recorded.

| Family | Cases | `typed_success` observations | Interoperability successes | Target `rejected` | `setup_rejected` | `response_validation_failed` | Other measured findings |
|---|---:|---:|---:|---:|---:|---:|---|
| Lifecycle and probes | 89 | 80 | 80 | 3 | 0 | 2 | 2 malformed, 1 setup blocked, 1 unsupported |
| Cryptographic | 1,466 | 670 | 650 | 383 | 325 | 81 | 7 semantic mismatches |
| Key management | 216 | 12 | 12 | 201 | 3 | 0 | — |
| Import/Export/Register transfer | 4,640 | 2,986 | 2,986 | 778 | 876 | 0 | — |
| RNG | 6 | 0 | 0 | 0 | 0 | 6 | — |
| **Executable total** | **6,417** | **3,748** | **3,728** | **1,365** | **1,204** | **89** | **7 semantic mismatches; 4 fixture/setup/malformed/unsupported findings** |

The 12 exclusions are six RNG parameter variants forbidden by the request structures and six Certify subject-OID-only CSR rows without a source-confirmed valid contract; none was sent. Twenty-four Create Key Pair source-discrepancy probes returned typed success and both key identifiers passed the response postcondition, so they count as observed KMIPKit–Cosmian interoperability. Their algorithm values remain outside the pinned KMIP 2.1 enumeration. The remaining 20 crypto typed finding observations stay outside the interoperability count. The 54 transfer finding rows with typed success and passed operation postconditions now count as observed interoperability, while retaining their source/documentation classifications; this does not claim KMIP 2.1 conformance. A targeted follow-up measured all 15 former cipher fixture blockers; see the Task 6 report.

### Cryptographic and RNG detail

The accepted crypto inventory has 1,466 standards-valid rows across ten operations; six standards-invalid Certify rows are excluded. Expected dispositions are 1,220 `typed_success`, six `documented_but_source_gap`, and 240 `source_documentation_discrepancy`. Offline builders encode every expected-success request. Exact finding preflight builds 246 requests (284 TTLV payloads including multipart rows); the 15 former cipher fixture blockers now have exact typed probes. All 30 post-quantum Certify finding cases have encoding-matched CSR fixture paths. PEM uses Certificate Request Type PEM (`3`), DER/PKCS#10 uses PKCS10 (`2`), following OASIS KMIP v2.1 §11.8/Table 439.

The live matrix records 650 crypto interoperability successes, 20 other typed responses on finding
rows, 383 target rejections, 325 setup rejections, 81 response-validation failures and seven Hash
multipart semantic mismatches. Three standalone AES-ECB Encrypt cases also pass after replacing the
invalid 19-byte input with one complete 16-byte block; Encrypt now records 287/300 interoperability
successes. Create Key Pair has 57/59 successful interoperability cases: 24 additional
source-discrepancy profiles returned valid typed results with both key identifiers; two
ConfigurableKEM curve-only requests were rejected. These additional cases are observed
implementation interoperability, not KMIP 2.1 enum conformance. The targeted follow-up confirms all
15 previously unavailable cipher findings reached the target: 12 vendor KEM cases passed typed
postconditions and three AES/RSA requests were rejected; those cipher findings remain outside the
advertised-operation count. On Decrypt, the earlier six setup rejections fell to two after fixing
the three unaligned AES-ECB fixtures and CoverCrypt metadata framing. The ConfigurableKEM
classical-only Create Key Pair requests still receive Cryptographic Failure before Decrypt is sent;
the RSA/AES probe receives Operation Not Supported. The RSA/AES Decrypt fixture is ordinary RSA-OAEP
data, not a genuine AES-GCM/RSA hybrid ciphertext. The 81 validation failures are HMAC prerequisite
responses where the server omitted Operation (Status 1, Reason 4). Seven multipart Hash requests
returned typed Success, but the final digest differed from the independent known-answer digest for
input abc; they are semantic_mismatch, not parser failures. The corrected flow sends a, b, and c as
explicit input blocks, reuses the first response's Correlation Value as described by OASIS KMIP v2.1
§6.1 and §7.8, and follows Hash §6.1.24, Table 235. Section 6.1 says Data is always required
except when Init or Final is true, while Table 235 marks Hash Data as not required for multipart
requests. The live test supplies Data on all three parts and makes no conformance claim for the
disputed intermediate omission.
The pinned Cosmian 5.28.0 Hash implementation does not propagate updated multipart state to the
final request. All six valid length-only RNG requests received a response missing Operation (Status
1, Reason 4); the matrix classifies them as response_validation_failed, never successful RNG
exchanges. No RNG bytes or seed material are logged.

### Transfer, key-management, and lifecycle detail

Transfer observations are 2,986 interoperability successes, 778 target rejections, 876 setup rejections, and no response-validation failures after the focused Register retest. The 54 typed expected-finding successes count as observed interoperability because the runner's postconditions passed; their source/documentation discrepancy tags remain. The full run exercised all 72 ConfigurableKEM opaque transfer rows; the manifest records 48 setup rejections and 24 target rejections, instead of the earlier provisional fixture-unavailable classification.

### Register retest attribution

All 446 Register rows that were previously setup-rejected or rejected were rerun through the typed client against the pinned local Cosmian 5.28.0 service. The rerun produced 154 typed successes, 286 failures in prerequisite operations before Register, and six target Register rejections. Register is now 737 / 1,029 interoperable cases. OASIS KMIP v2.1 §6.1.43, Tables 291–294 defines the Register request and its error handling; the prerequisites below must not be counted as Register outcomes.

| Observation | Count | Evidence and attribution |
|---|---:|---|
| Target Register succeeds | 154 | Fifty-two prior Reason 4 failures were caused by the integration test builder using incorrect SymmetricKey/Key TTLV tags; after the tag correction, all 52 pass. Another 102 old setup-failure observations now pass their target round trips in the complete retest. |
| Prerequisite `Operation Not Supported` | 165 | 143 Export requests combined explicit PKCS#8 with wrapping, which pinned Cosmian `export_get.rs` rejects at lines 3499–3515. We tried omitting the format selector in one wrapped setup Export; For this Ed25519 PrivateKey row, Cosmian returned default KeyFormatType 20 (Transparent EC Private Key) rather than PKCS#8, so it could not serve as an equivalent Register fixture; six ConfigurableKEM public-key formats are absent from Cosmian's format documentation; 12 ConfigurableKEM and four DSA Create Key Pair requests were unsupported. These are not Register rejections. KMIP v2.1 §4.26, Tables 79–80 allows the server to fail when it cannot provide a requested format; support for a format and wrapping separately does not establish every combination. |
| Prerequisite `Cryptographic Failure` | 115 | 111 Export setup cases used direct RSA wrapping with TTLV encoding (60 Ed25519 transparent-EC, 32 AES SymmetricKey, 19 SecretData); four pure-classical ConfigurableKEM Create Key Pair setups also failed. The exact provider cause is not exposed. The KMIP 2.1 Get/Export error tables do not list `Cryptographic Failure`; the mapping should be confirmed with Cosmian. |
| Prerequisite `Sensitive` | 6 | Each Create Key Pair request included Sensitive=False and a typed Get Attributes response for the same identifier returned False. Unwrapped Export nevertheless returned Reason 22, so Register was not sent. KMIP §4.55, Tables 139–140 makes Sensitive client-modifiable and requires unwrapped retrieval to be prevented when it is True. The observed false/readback plus Export denial is an inconsistency in the server behavior and should be reported to Cosmian. |
| Target Register `Operation Not Supported` | 2 | Password-protected PKCS#12 requests use the SecretData identifier prescribed by §4.31, Tables 90–92. Pinned Cosmian PKCS#12 handling consumes the linked identifier as password text; this remains a vendor/standard discrepancy. |
| Target Register `Cryptographic Failure` | 4 | X25519MLKEM768 and X448MLKEM1024 private/public requests reached Register. These use vendor algorithm values outside the KMIP 2.1 enumeration; operation-level Register support does not promise these vendor-extension intersections. Cosmian documents the hybrid algorithms and marks Register implemented, but does not explicitly document this Register/object-format intersection. `Cryptographic Failure` is also absent from Register Table 294; ask Cosmian whether it supports Register for these vendor objects and why it returns that reason. |

The six Sensitive readback checks use KMIPKit's typed `GetAttributesRequest` and validate the returned identifier and Boolean. They prove that the discrepancy is not an omitted Sensitive field in KMIPKit's test Create Key Pair request. Per-case records include the safe reason and whether the target Register request was sent.

### Get and Export attribution

The previous run had 50 legacy-PKCS#12 response-validation failures because `requested_format_value` existed in the case parameters but the typed Get/Export builders omitted Cosmian's vendor `KeyFormatType` value `0x88800001`. The integration harness now serializes it and accepts Cosmian's normalized standard PKCS#12 response value `0x16`. A focused run selected 66 cases: 62 target requests returned typed success, including all 58 rows that send the selector; four were rejected during password-linked PFX Import setup and never sent Get/Export.

| Operation | Interoperable / executable | Target rejections | Setup rejections | Target rejection breakdown | Setup rejection breakdown |
|---|---:|---:|---:|---|---|
| Get | **797 / 1,279** | 382 | 100 | 160 `Operation Not Supported`; 8 `Sensitive`; 214 `Cryptographic Failure` | 78 prerequisite `Invalid Message` replies missing `Operation`; 16 `Operation Not Supported`; 6 `Cryptographic Failure` |
| Export | **807 / 1,291** | 384 | 100 | 160 `Operation Not Supported`; 8 `Sensitive`; 216 `Cryptographic Failure` | 78 prerequisite `Invalid Message` replies missing `Operation`; 16 `Operation Not Supported`; 6 `Cryptographic Failure` |

The missing legacy selector was a KMIPKit **integration-harness** defect, not a production-library defect. Current ratios have no Get/Export `response_validation_failed` rows. The 100 setup rejections are not target-operation results and stay outside the interoperability numerator.

#### `Operation Not Supported` and published Cosmian coverage

KMIP 2.1 §§6.1.18–6.1.19 allow `Operation Not Supported` as a response reason; Table 478 defines it as the requested operation being unsupported. Cosmian's [5.28.0 support table](https://docs.cosmian.com/versions/kms/5.28.0/kmip_support/support.html) marks Get and Export implemented. That operation-level claim does not guarantee every object, format, and wrapping combination. The following per-operation target rejections were checked against the pinned product documentation:

| Rejected request profile | Cases per operation | Cosmian 5.28.0 documentation check | Assessment |
|---|---:|---|---|
| Explicit PKCS#8 plus wrapping | 143 | PKCS#8 for RSA/EC is listed in the [formats table](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/kmip_support/formats.md#L202-L220); wrapping is described generally in the [algorithms page](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/certifications_and_compliance/cryptographic_algorithms/algorithms.md#L202-L218). The combined profile is not stated explicitly. | Pinned [`export_get.rs`](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/export_get.rs#L3499-L3515) explicitly rejects wrapping with an explicit format. Ask Cosmian whether the documented format and wrapping capabilities are intended to compose. Current case setup does not prove that PKCS#8 was the original registration format, so a mandatory KMIP violation is not established. |
| PKCS7 format | 8 | PKCS7 is absent from the formats and objects tables. | No published support claim for this combination; ONS does not contradict the published matrix. |
| ConfigurableKEM public-key formats | 6 | These transfer formats are absent from the formats and objects tables. | No published support claim for these combinations. |
| Ed448 transparent formats | 2 | Ed448 is listed as a primitive, but the requested output formats are not listed. | Algorithm support alone does not document these format conversions. |
| Ed448 PKCS#12 format | 1 | PKCS#12 output for this object/profile is not listed. | No published support claim for this combination. |

#### Other Get/Export findings and attribution

| Finding | Get | Export | Assessment and follow-up |
|---|---:|---:|---|
| `Sensitive` target rejection | 8 | 8 | Six ConfigurableKEM private keys and two SplitKey cases. Expected KMIP behavior for unwrapped retrieval of sensitive keys (§4.55 and §9.22); no client defect indicated. |
| Direct RSA wrapping exceeds one RSA operation's capacity | 211 | 211 | Test-profile limitation for these payload sizes. The harness now sends `Padding Method=None` for the documented RSA-AES hybrid profile; use that or AES-KW/KWP for larger values. |
| PKCS#12 output with `secp224k1` | 3 | 3 | Cosmian lists NIST EC primitives but does not list PKCS#12 output here; not a published support claim. |
| AES-XTS wrapping with 512-bit KEK | 0 | 2 | Cosmian's [algorithms page](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/certifications_and_compliance/cryptographic_algorithms/algorithms.md#L218-L233) says listed encryption schemes may wrap keys and lists XTS. In pinned [`wrap_key.rs`](https://github.com/Cosmian/kms/blob/5.28.0/crate/crypto/src/crypto/wrap/wrap_key.rs#L1708-L1728), GCM and NISTKeyWrap are explicit; other modes fall through to RFC 5649 AES-KWP. Successful 256-bit XTS rows therefore do not prove XTS was used. Ask Cosmian to confirm support, provide a round-trip vector, or narrow the documentation. |
| Password-linked PFX Import setup receives ONS | 8 | 8 | Cosmian's [objects page](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/kmip_support/objects.md#L210-L215) says certificates can be imported as PKCS#12/PFX. KMIP 2.1 §4.31, Tables 90–92 defines `PKCS12PasswordLink` as a link to a `SecretData` object, but pinned [`import.rs`](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/import.rs) treats the linked identifier text as the password. This is a vendor support/documentation discrepancy to report; dependent Get/Export was not sent. |
| Prerequisite response is `Invalid Message` without `Operation` | 78 | 78 | Cosmian's setup response is malformed; target Get/Export was not sent. |
| DSA transparent-format setup receives ONS | 8 | 8 | The formats table leaves DSA import/export formats blank, so these combinations are not documented as supported. |
| ConfigurableKEM/CoverCrypt key setup receives `Cryptographic Failure` | 6 | 6 | Setup/provider failure before Get/Export; not a target-operation result. |

The three `Cryptographic Failure` responses per operation for `secp224k1` and the 211 oversized direct-RSA cases are included in the count above. There is a separate possible result-reason discrepancy: the [KMIP 2.1 Get and Export error tables](https://docs.oasis-open.org/kmip/kmip-spec/v2.1/kmip-spec-v2.1.html) (Tables 222 and 219) do not list `Cryptographic Failure`, although Cosmian returned it. Ask Cosmian which standard result reason it intends for the RSA and XTS failures; treat this as a question pending vendor interpretation, not a confirmed violation.

Key management recorded 12 interoperability successes, 201 target rejections and three setup rejections. Lifecycle recorded 80 interoperability successes out of 89 rows: three target rejections, two response-validation failures (Query and Locate), two malformed responses (Ping and Adjust Attribute), one setup-blocked Activate SplitKey case, and one unsupported Re-certify probe. The Check SplitKey case was rerun after removing an optional usage mask that Cosmian rejected during setup; the corrected case passed with no mask. Discover Versions passed. Query returned KMIP Success and Operations but omitted required Protection Storage Masks (OASIS §6.1.40, Table 283); Ping's failure response omitted the required Batch Item Operation. Adjust Attribute returned Operation Failed/Invalid Message without Operation; Re-certify returned a well-formed Operation Not Supported response.

The latest complete serial ignored suite ended with 101 passed and 9 failed test functions after 6,383.81 seconds using rustls 0.23.46. Its sanitized log is ignored at `.local/cosmian-kms/final-rustls-0.23.46-full-suite.log` (SHA-256 `84947A36B3557C22402DA12F918898F1743F913E4153A4A025120A7E4B2C0051`). The failed functions are the crypto, key-management and transfer matrix runners; Activate SplitKey, Add Attribute SplitKey, Locate SplitKey, and the two Validate Certificate Values tests; and the RNG matrix runner. A failed matrix-runner assertion remains a failure even when individual rows yield useful observations. The output reconciles 6,412 distinct `case_id` values plus the five standalone lifecycle tests; four Validate IDs have a second event/classification line but no contradictory outcome. The six valid RNG response lines and manifest use `response_validation_failed`; each response had Status 1 / Reason 4 and omitted Operation, so none counts as an interoperability success. Targeted live follow-ups measured corrected Check SplitKey, Discover Versions, Query, Ping, Adjust Attribute and Re-certify probes. Compose teardown succeeded and `docker compose --file tests/integration/cosmian/compose.yaml ps --all` returned no services. The exact per-case records are in `tests/integration/cosmian/capability-matrix.json`.

### PKCS#12 Password Link interoperability discrepancy

OASIS KMIP v2.1 §4.31, Tables 90–92, defines the `PKCS12PasswordLink` as a Link to a `SecretData` object. KMIPKit's typed fixture registers `SecretData` and places its returned Unique Identifier in the Link and the corresponding password selection. It does not send the password text in place of an identifier. In pinned Cosmian 5.28.0 source, `crate/server/src/core/operations/import.rs::process_pkcs12` converts the linked identifier to text and uses it as the PFX password instead of resolving the linked SecretData material; `crate/server/src/core/operations/export_get.rs` reads the outgoing PKCS12 password from the Key Wrapping Specification's Encryption Key Information identifier. This differs from the standard-shaped SecretData UID model. The matrix does not introduce a literal-password workaround. The exact request was rejected during Import setup, leaving the dependent Get/Export target operation untested for those cases.

The unprotected-password PFX rows succeeded. For Certificate-target Get/Export requests selecting PKCS12 or Cosmian legacy PKCS12, the typed response is a `PrivateKey` object containing the package; the response validator checks that object type and the returned Key Format Type. Cosmian normalizes the legacy output format to standard PKCS12 `0x16`. `research.md` contains the pinned source rationale and test limitations.

## Operation scope and separately classified probes

The 34 Cosmian support-table rows represent 35 operation paths because the vendor groups Set Attribute and Modify Attribute. All 35 advertised paths have stable inventory rows and typed executors. Set and Modify are tested independently. The exhaustive matrix crosses the paths with source-valid object, algorithm, parameter, encoding and format dimensions; the operation-level distributions appear in the capability manifest and in `docs/development/kmip-interoperability.md`.

Query is an advertised path with a measured typed-response validation failure. Ping and Adjust Attribute are separate unadvertised probes, not advertised successes; both returned malformed Operation Failed / Invalid Message responses (Status 1, Reason 4) without the required Batch Item Operation. Re-certify is an unadvertised source-discrepancy probe and returned a well-formed Operation Not Supported response; it does not increase the advertised-success set. Other Cosmian source/documentation disagreements, including PFX, PEM/DER, Hash/SHA-224, and algorithm parameter differences, remain linked to individual rows in the manifest and are explained in `research.md`.

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
| §6.1.36, Tables 271–272; §8.6, Table 399 | Ping's typed response is checked for the required Batch Item Operation | Measured incompatibility: Cosmian returned Operation Failed / Invalid Message (Status 1, Reason 4) without Operation; KMIPKit rejected the malformed response |
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
