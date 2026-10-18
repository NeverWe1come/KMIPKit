# Cosmian 5.28.0 exhaustive integration evidence

**Status: implementation and matrix execution complete; interoperability failures and final PR gates remain open.** The test suite exercises KMIPKit's typed Rust client over the pinned local binary TTLV/mTLS endpoint. The matrix records each observed result without counting expected-finding assertions as success. This report does not claim complete KMIP 2.1 conformance or certification.

## Scope and measured totals

The pinned target is `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`, source commit `4ab3ee65829a903abb71f1298f400657fcee7ec9`. The authoritative per-case ledger is [`tests/integration/cosmian/capability-matrix.json`](../../../tests/integration/cosmian/capability-matrix.json). It contains 6,429 unique rows: 6,417 executable cases and 12 normative exclusions. Every executable row has an actual live classification or an explicit fixture/setup blocker; no executable row remains `not_run`. Fifteen cryptographic rows were classified `fixture_unavailable`, and no target request was sent for them.

| Family | Rows | Typed success responses | Interoperability successes | Rejected | Setup rejected | Response validation failed | Other findings |
|---|---:|---:|---:|---:|---:|---:|---|
| Lifecycle and probes | 89 | 80 | 80 | 3 | 0 | 2 | 2 malformed; 1 setup blocked; 1 unsupported |
| Cryptographic | 1,466 | 648 | 616 | 384 | 331 | 88 | 15 fixture unavailable |
| Key management | 216 | 12 | 12 | 201 | 3 | 0 | — |
| Import/Export/Register transfer | 4,640 | 2,722 | 2,668 | 890 | 978 | 50 | — |
| RNG | 6 | 0 | 0 | 0 | 0 | 0 | 6 malformed |
| **Executable total** | **6,417** | **3,462** | **3,376** | **1,478** | **1,312** | **140** | **25 fixture/setup/malformed/unsupported** |

The 86 typed responses on finding rows (32 crypto and 54 transfer) are excluded from the 3,376 interoperability successes. Rejection means a well-formed KMIP failure response; setup rejection means a prerequisite failed before the target operation was sent; response-validation failure means the request was delivered but KMIPKit could not accept or satisfy a typed-response postcondition; malformed response means required response shape was missing. These categories remain distinct in each matrix record.

The 12 normative exclusions are six invalid RNG Parameters combinations and six invalid Certify subject-OID-only CSR combinations. They were not sent. The exact request/fixture limitations and OASIS citations are recorded on the corresponding rows and in `specs/015-cosmian-kms-integration/research.md`.

## Full serial run and teardown

The complete ignored suite ran serially against the pinned image with:

```powershell
cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1 --nocapture
```

The sanitized ignored-local log is `.local/cosmian-kms/task14-full-ignored-suite.log`, SHA-256 `FA7B0CE55E50FD38D14A5C39EA005E99CCD3FB4828207B8F78C3341F8888FC05`. It ended `98 passed; 10 failed; 0 ignored; 0 measured; 104 filtered out; finished in 6116.15s`. The ten failed test functions were the crypto and key-management family runners, six specific lifecycle cases, RNG, and transfer. The family runners still emitted per-case safe classifications; those records were reconciled with the manifest. A corrected targeted Check SplitKey probe and targeted Discover Versions, Query, Ping, Adjust Attribute and Re-certify runs updated their relevant outcomes afterward. The test output retained no raw TTLV or secret material. Compose teardown completed; `docker compose --file tests/integration/cosmian/compose.yaml ps -a` returned no services.

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

The 1,466 crypto rows include 1,220 expected typed-success cases, six source gaps and 240 documentation/source discrepancies. Offline typed builders construct all expected-success requests; exact finding preflight serializes 231 requests (269 payloads including multipart rows). Fifteen fixture-dependent finding rows remain explicitly unavailable. Live results are 616 interoperability successes, 32 typed responses on finding rows, 384 target rejections, 331 setup rejections, 88 response-validation failures, and 15 unavailable fixtures.

The 88 crypto response-validation failures comprise 81 HMAC prerequisite responses where the peer omitted Operation (Status 1, Reason 4) and seven Hash-streaming typed postcondition failures. The matrix does not infer an unrecorded missing field for the Hash responses. All six valid RNG length-only cases returned malformed typed responses missing Operation, with Status 1 and Reason 4. OASIS KMIP v2.1 §6.1.49/Table 316 and §6.1.50/Table 319 define the respective request payloads; §8.6/Table 399 governs the response Operation field. No RNG bytes, seeds, private keys, passwords, or response bodies are recorded.

### Key management and transfer

Key management has 12 interoperability successes, 201 target rejections and three setup rejections across 216 cases.

Transfer has 2,668 interoperability successes, 54 typed finding responses, 890 target rejections, 978 setup rejections and 50 response-validation failures across 4,640 rows. The full run exercised all 72 ConfigurableKEM opaque transfer cases: 48 prerequisite setup rejections and 24 target rejections. These rows are not labeled fixture-unavailable.

The 50 PrivateKey Get/Export rows selecting `Cosmian-Pkcs12Legacy` did not expose the explicitly requested Key Format Type in the typed object. Their response contents were not retained, so these remain response-validation findings rather than a claim of a Cosmian protocol defect. Password-linked PFX import has a separate source-documented discrepancy: the 16 affected fixture setups were rejected with Operation Not Supported before dependent target Get/Export requests were sent. Sixteen unprotected-password PFX cases succeeded. OASIS KMIP v2.1 §4.31, Tables 90–92 defines the Password Link as a link to SecretData; the matrix does not replace that identifier with literal password text. Details and pinned source references are in `specs/015-cosmian-kms-integration/traceability.md` and `research.md`.

## Test development evidence

- Manifest inventory Red/Green: signed commits `34d9d3a1`, `d70cc950`, and `2c3edc5a` record the initial missing-inventory failure, inventory correction, and typed exhaustive crypto implementation. The inventory validator now passes over 6,429 rows.
- Crypto dispatch and typed-construction contracts cover every applicable crypto case; the 15 exact fixture blockers stay explicit. The later lint refactor is `dedbf1e5`.
- RNG malformed-response and peer-rejection classifications have distinct signed Red/Green/Refactor commits `b6f9a54d`, `7fbc91cd`, `f360ab75`, `b14587e1`, and `4f76d547`.
- Adjust Attribute and Re-certify lifecycle parity has signed Red commit `dbe00bc9` and Green implementation commit `c67d0fe4`; the two probes were rerun against the pinned deployment and are separately classified in the matrix.
- The completeness guard has signed Red commit `422dc6ba` (the regression test failed because an executable `not_run` row was accepted) and Green commit `66ce4239` (executable rows now reject `not_run`; normative exclusions remain allowed).
- Current offline command `cargo test --offline -p kmipkit-client --test cosmian_kms` passed 107 tests, failed zero and ignored 110 live tests. (Cargo emitted existing AWS-LC MSVC stdalign/NASM environment warnings that did not prevent the command from completing.)
- Final local verification on this implementation: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; and `cargo test --workspace --all-features` passed. `cargo llvm-cov --workspace --all-features --summary-only` reported 92.93% workspace line coverage (above the 90% workspace gate).
- Contract and inventory verification passed: requirement traceability tests (2), feature traceability tests (15 passed, one Windows symlink case skipped), normative catalog suite (222 passed, seven skipped), exact-base immutable-source check, source-candidate audit (1,411 candidates), catalog validation (4,044 records), and all five generated-output checks. The integration PowerShell file parsed, Compose configuration validated, `git diff --check` passed, and `docker compose ... ps --all` returned no services.
- The prior PR run recorded a dependency-policy failure for `rustls 0.23.45`. This branch now includes the approved KMIPKIT-0045 remediation merged into `release/1.0.0`; its lockfile resolves `rustls 0.23.46`, and `scripts/Test-DependencyPolicy.ps1` passed against fresh RustSec data at commit `64f92cdc64401fab47baa10e2e11fe382aa57155` (2026-10-17 01:27:42 +02:00). KMIPKIT-0015 adds no dependency waiver. Hosted CI remains required on the final pushed head; keep PR #56 in draft until that result is evaluated. The two-independent-KMIP-2.1-implementation release gate also remains open.

## Remaining release/review gates

Keep PR #56 in draft pending hosted CI on the final pushed head. The previous dependency-policy failure was for `rustls 0.23.45`; the merged approved KMIPKIT-0045 release update changes the resolved version to `0.23.46`, and the fresh local dependency-policy check passes. Workspace tests, lint, coverage, traceability, immutable-source, catalog, generated-output, and integration teardown checks pass locally. The exhaustive live suite retains ten failed test functions and the two-independent-KMIP-2.1-implementation release gate remains open.
