# Cosmian 5.28.0 exhaustive integration evidence

**Status: implementation and matrix execution complete; interoperability failures and final PR gates remain open.** The test suite exercises KMIPKit's typed Rust client over the pinned local binary TTLV/mTLS endpoint. The matrix records each observed result without counting expected-finding assertions as success. This report does not claim complete KMIP 2.1 conformance or certification.

## Scope and measured totals

The pinned target is `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`, source commit `4ab3ee65829a903abb71f1298f400657fcee7ec9`. The authoritative per-case ledger is [`capability-matrix.json`](../../tests/integration/cosmian/capability-matrix.json). It contains 6,429 unique rows: 6,417 executable cases and 12 normative exclusions. Every executable row has an actual live classification or an explicit fixture/setup blocker; no executable row remains `not_run`. A targeted follow-up measured all 15 previously fixture-blocked cryptographic findings against the pinned server.

| Family | Rows | Typed success responses | Interoperability successes | Rejected | Setup rejected | Response validation failed | Other findings |
|---|---:|---:|---:|---:|---:|---:|---|
| Lifecycle and probes | 89 | 80 | 80 | 3 | 0 | 2 | 2 malformed; 1 setup blocked; 1 unsupported |
| Cryptographic | 1,466 | 660 | 616 | 387 | 331 | 88 | — |
| Key management | 216 | 12 | 12 | 201 | 3 | 0 | — |
| Import/Export/Register transfer | 4,640 | 2,722 | 2,668 | 890 | 978 | 50 | — |
| RNG | 6 | 0 | 0 | 0 | 0 | 0 | 6 malformed |
| **Executable total** | **6,417** | **3,474** | **3,376** | **1,481** | **1,312** | **140** | **10 fixture/setup/malformed/unsupported** |

The 98 typed responses on finding rows (44 crypto and 54 transfer) are excluded from the 3,376 interoperability successes. Rejection means a well-formed KMIP failure response; setup rejection means a prerequisite failed before the target operation was sent; response-validation failure means the request was delivered but KMIPKit could not accept or satisfy a typed-response postcondition; malformed response means required response shape was missing. These categories remain distinct in each matrix record.

The 12 normative exclusions are six invalid RNG Parameters combinations and six invalid Certify subject-OID-only CSR combinations. They were not sent. The exact request/fixture limitations and OASIS citations are recorded on the corresponding rows and in `specs/015-cosmian-kms-integration/research.md`.

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

The 1,466 crypto rows include 1,220 expected typed-success cases, six source gaps and 240 documentation/source discrepancies. Offline typed builders construct all expected-success requests; exact finding preflight serializes 246 requests (284 TTLV payloads including multipart rows). All 30 post-quantum Certify finding cases have encoding-matched CSR fixture paths. Live results are 616 interoperability successes, 44 typed responses on finding rows, 387 target rejections, 331 setup rejections, and 88 response-validation failures. A targeted run sent the 15 formerly blocked cipher findings: all 12 ConfigurableKEM profile/direction combinations completed typed operation-specific postconditions; the three AES/RSA hybrid requests were rejected by the target after delivery. All remain non-interoperability findings.

The 88 crypto response-validation failures comprise 81 HMAC prerequisite responses where the peer omitted Operation (Status 1, Reason 4) and seven Hash-streaming typed postcondition failures. The matrix does not infer an unrecorded missing field for the Hash responses. All six valid RNG length-only cases returned malformed typed responses missing Operation, with Status 1 and Reason 4. OASIS KMIP v2.1 §6.1.49/Table 316 and §6.1.50/Table 319 define the respective request payloads; §8.6/Table 399 governs the response Operation field. No RNG bytes, seeds, private keys, passwords, or response bodies are recorded.

### Key management and transfer

Key management has 12 interoperability successes, 201 target rejections and three setup rejections across 216 cases.

Transfer has 2,668 interoperability successes, 54 typed finding responses, 890 target rejections, 978 setup rejections and 50 response-validation failures across 4,640 rows. The full run exercised all 72 ConfigurableKEM opaque transfer cases: 48 prerequisite setup rejections and 24 target rejections. These rows are not labeled fixture-unavailable.

The 50 PrivateKey Get/Export rows selecting `Cosmian-Pkcs12Legacy` did not expose the explicitly requested Key Format Type in the typed object. Their response contents were not retained, so these remain response-validation findings rather than a claim of a Cosmian protocol defect. Password-linked PFX import has a separate source-documented discrepancy: the 16 affected fixture setups were rejected with Operation Not Supported before dependent target Get/Export requests were sent. Sixteen unprotected-password PFX cases succeeded. OASIS KMIP v2.1 §4.31, Tables 90–92 defines the Password Link as a link to SecretData; the matrix does not replace that identifier with literal password text. Details and pinned source references are in `specs/015-cosmian-kms-integration/traceability.md` and `research.md`.

## Test development evidence

- Manifest inventory Red/Green: signed commits `34d9d3a1`, `d70cc950`, and `2c3edc5a` record the initial missing-inventory failure, inventory correction, and typed exhaustive crypto implementation. The inventory validator now passes over 6,429 rows.
- Crypto dispatch and typed-construction contracts cover every applicable crypto case. Signed Red commit `a44bcdf3` adds a regression test requiring exact typed probes for the 15 cipher findings; Green commit `060c7fcd` removes their fixture blockers and provisions an RSA OAEP ciphertext for the AES/RSA Decrypt probe. The pinned targeted run observed all 15 outcomes and Compose teardown left no services. The later lint refactor is `dedbf1e5`.
- RNG malformed-response and peer-rejection classifications have distinct signed Red/Green/Refactor commits `b6f9a54d`, `7fbc91cd`, `f360ab75`, `b14587e1`, and `4f76d547`.
- Adjust Attribute and Re-certify lifecycle parity has signed Red commit `dbe00bc9` and Green implementation commit `c67d0fe4`; the two probes were rerun against the pinned deployment and are separately classified in the matrix.
- The completeness guard has signed Red commit `422dc6ba` (the regression test failed because an executable `not_run` row was accepted) and Green commit `66ce4239` (executable rows now reject `not_run`; normative exclusions remain allowed).
- Clean-checkout CI regression, Red/Green: run `39006703175` failed the offline Cosmian inventory tests on Linux, macOS and Windows because assertions assumed the ignored runtime-generated certificate/CSR directory existed. Green commit `8e81b1bb` validates the 30 PQC CSR-dependent probes against the actual fixture set, requires that set to be complete or absent, and keeps certificate path mapping checked when runtime files are absent. With no generated CSR fixtures, preflight is 216 runnable / 30 `NotSent`; with generated CSR fixtures, it is 246 / 0. The 15 earlier cipher blockers are now exact target probes and require no local private-key fixture. Targeted Clippy and formatting checks pass.
- Latest offline command `cargo test --offline -p kmipkit-client --test cosmian_kms` passed 109 tests, failed zero and ignored 110 live tests. (Cargo emitted existing AWS-LC MSVC stdalign/NASM environment warnings that did not prevent the command from completing.)
- Final local verification on the cipher-probe implementation: `cargo fmt --all --check`, `cargo test --offline --workspace --all-features`, and `cargo clippy --offline --workspace --all-targets --all-features -- -D warnings` passed. `cargo llvm-cov --workspace --all-features --summary-only` reported 92.93% workspace line coverage (above the 90% workspace gate).
- Contract and integration verification passed: requirement traceability and pinned-image Compose checks (3 tests), normative inventory and generated-output checks in hosted CI, PowerShell/Compose configuration, `git diff --check`, and empty Compose status after the targeted run. Hosted CI run `39014453167` passed for implementation/matrix head `297b98d5` across Core, bindings, sanitizers, scripts, documentation, normative inventory, dependency policy and aggregate coverage. The final task-closeout documentation commit receives a final-head CI rerun.
- The earlier dependency-policy issue for `rustls 0.23.45` is resolved by the approved KMIPKIT-0045 remediation merged into `release/1.0.0`; this branch resolves `rustls 0.23.46` and adds no dependency waiver. Fresh local RustSec policy and hosted Dependency policy both pass. Hosted run `39014453167` passed on `297b98d5`; the close-out docs update triggers a final-head rerun. Keep PR #56 in draft. The two-independent-KMIP-2.1-implementation release gate remains open.

## Remaining release/review gates

Keep PR #56 in draft. Hosted CI run `39014453167` passed on implementation and matrix head `297b98d5`, including dependency policy, all platform checks, sanitizers, bindings, normative inventory and aggregate coverage. The final T044 documentation close-out commit triggers one last head check. The exhaustive live run retains nine failed test functions; these target/interoperability outcomes remain recorded. The two-independent-KMIP-2.1-implementation release gate remains open.
