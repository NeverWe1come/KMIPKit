# T099 execution report — KMIPKIT-0019 declaration reconciliation

**Status:** Complete

**Worktree:** `C:/Users/ramp1953/.codex/worktrees/0043-t076-client-mapping/KMIPKit`

**Branch:** `feature/KMIPKIT-0043-t076-shared-client-mapping`

## Result

Resolved all 165 baseline Rust declaration candidates for KMIPKIT-0019. The exact partition is 43 semantic four-language mappings and 122 individually approved exact exclusions. The final KMIPKIT-0019 owner slice has 95 mapped declarations and 134 approved exclusions, including the 52 existing T040 mappings and 12 prior exclusions. No KMIPKIT-0019 candidate remains blocked.

The mapped set is 17 protocol request declarations (two request types, two constructors, and thirteen `with_*` builders), two response `result()` accessors, and 24 client/root operation declarations. Each mapping retains the exact Rust identity/package; C, Java, and Python target the existing T040 operation facades. Encrypt and Decrypt request targets are the established batch-item operation constructors; outcome declarations target the matching typed response selectors. The response `result()` accessors use the C `kmipkit_client_batch_item_response_metadata_v2`, Java `org.kmipkit.extensions.ClientBatchItemResult.metadataV2`, and Python `kmipkit.client.BatchItemResult.metadata_v2` surfaces, preserving status/reason/message metadata. Every mapped row cites the approved owner source, the applicable manifest-listed KMIPKIT-0019 requirements, KMIPKIT-0043-FR-004/005/010/017/021/022, and existing parity evidence.

The exclusions comprise 92 Rust trait artifacts, 18 operation conversion-error declarations, eight request readback/validation/serialization helpers, and four standalone response parsers. Every exclusion records its exact package/export identity, owner source, a specific reason, applicable owner FR citations, and all six T099 API FR citations. The parser helpers are excluded because no standalone C, Java, or Python parse call exists; typed success, completed failure, and Pending conversion are represented by the existing client outcome flow.

The combined 229-identity SHA-256 is unchanged: `9ea6a5e0e3ce1c9c37859a2d9232289fbf5c02c1398b5f8d6b6b99438a3535b3`. The Red baseline candidate inventory was 165 identities with SHA-256 `8622dae80e3772bab0af270c14d70c6c4ef658f73a73f059c0d4a7eb9d2ba470`. All 10,050 public declaration records outside the original 165-candidate slice remained byte-for-byte unchanged; all 4,469 pre-existing exclusions remain intact.

No runtime API or protocol behavior changed. The source manifest is the edited generation input. The pinned generator produced the affected FFI Rust inventory, Java parity fixtures, and JSON parity inventory; all 12 outputs verify current.

## Red / Green / Refactor evidence

| Phase | Commit | Evidence |
|---|---|---|
| Red | `d734f32134eabb0d7d96c550d0c0f25e9ad3f0ea` | Added the T099 manifest contract before source changes. The initial identity count/fingerprint assertion passed. Four resolution assertions failed as expected: 165 unresolved candidates, 0 T099 mappings, 0 T099 exclusions, and only 52 existing owner mappings instead of 95. |
| Green | `047bf9199313bd0f074351573a25884261474f1c` | Resolved the exact 43/122 partition, preserved the 229-identity fingerprint, and regenerated manifest outputs using `generate.py --write`. The focused T099 contract passed 5/5. |
| Refactor/evidence | `3e5ec939526eecf213b0826b3624d2d245549193` | Consolidated normative requirement expectations in the focused test and recorded task/evidence closeout. The combined focused T099/T040 suites passed 9/9; generator and whitespace checks passed. |

### T040 operation-pair correction

The relevant T040 contract initially failed against the Red-base manifest: both Encrypt and Decrypt had three Rust IDs in `operations[].declarations.rust`, while the parity test requires the canonical request/response pair. The exact subtest assertion was `AssertionError: 2 != 3 : rust` at `test_encrypt_decrypt_parity_manifest.py:144`. The narrow correction is commit `1ab4f8b8a8d6fc5b5e2f31b551114d0663a43edb`: each operation now lists the canonical `kmipkit_client::ClientRequest::{Encrypt,Decrypt}` followed by `kmipkit_client::ClientBatchOutcome::{encrypt_response,decrypt_response}`. The root response alias remains individually mapped in the T099 public declaration inventory. This correction regenerated its outputs and made the existing T040 contract pass without weakening it.

## Verification

- `python -B -m unittest -v tools.api_manifest.tests.test_t099_crypto_declaration_mapping` — 5 passed.
- `python -B -m unittest -v tools.api_manifest.tests.test_encrypt_decrypt_parity_manifest` — 4 passed after the canonical-pair correction.
- `python -B -m unittest -v tools.api_manifest.tests.test_t099_crypto_declaration_mapping tools.api_manifest.tests.test_encrypt_decrypt_parity_manifest` — 9 passed.
- `python -B tools/api_manifest/generate.py --write` — regenerated 12 outputs.
- `python -B tools/api_manifest/generate.py --check` — all 12 outputs verified.
- Direct `_validate_v2_manifest` validation — passed.
- `git diff --check` — passed.

The recorded parity evidence paths are `bindings/c/tests/client_abi_consumer.c`, `bindings/java/src/test/java/org/kmipkit/CryptoOperationsJavaFacadeTest.java`, `bindings/python/tests/test_encrypt_decrypt_operations.py`, and `crates/kmipkit-ffi/tests/support/client_abi_tests.rs`. This report does not claim PR-wide CI or coverage gates; those remain part of the parent task's review.


### Review correction — complete Encrypt/Decrypt result containers (2026-11-01)

A review found that 14 shared outcome declarations were mapped to success-only Encrypt/Decrypt selectors. Added a test first in Red commit `d5a30df99` (`t099 require complete crypto result container red`); it failed in 14 export subtests, showing the old targets `kmipkit_client_batch_item_response_encrypt/decrypt`, `ClientBatchItemResult.encryptResponse/decryptResponse`, and `BatchItemResult.encrypt_response/decrypt_response` where the complete containers were required.

The Green manifest change updates exactly those 14 `languageMappings`: both Rust crates' `ClientBatchOutcome::{Encrypt,Decrypt}` variants and tuple fields, both crates' `ClientBatchOutcome::encrypt_response/decrypt_response` methods, and `ClientResponseView::{encrypt,decrypt}`. Their C/Java/Python targets now name `kmipkit_client_batch_item_response_t`, `org.kmipkit.extensions.ClientBatchItemResult`, and `kmipkit.client.BatchItemResult`. Rust targets remain the exact declaration paths. Existing T040 typed-success selectors and typed response fields were left unchanged.

The contract and diff review confirm owner counts remain 95 mappings/134 exclusions and the 229-identity SHA-256 remains `9ea6a5e0e3ce1c9c37859a2d9232289fbf5c02c1398b5f8d6b6b99438a3535b3`. The Green manifest/evidence change is committed separately from the Red test in the following commit.

Verification after Green:

- `python -B -m unittest -v tools.api_manifest.tests.test_t099_crypto_declaration_mapping tools.api_manifest.tests.test_encrypt_decrypt_parity_manifest` — 10 passed.
- `python -B tools/api_manifest/generate.py --write` regenerated 12 outputs; `python -B tools/api_manifest/generate.py --check` verified all 12.
- `git diff --check` passed.
