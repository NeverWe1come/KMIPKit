# KMIPKIT-0052 Traceability

Normative identifiers use `KMIPKIT-0052-R###`. Project API/ownership decisions use `KMIPKIT-0052-P###` and do not claim to be OASIS requirements.

## Normative Requirements

| ID | OASIS source | Requirement interpretation | Implementation | Executable verification |
|---|---|---|---|---|
| KMIPKIT-0052-R001–R056 | KMIP v2.1 §6.1 operation-specific Response Payload tables listed row-by-row in `gap-matrix.md`; §8.6 Table 399 for Response Payload presence | Preserve and expose each supported operation's response result and payload presence without assigning new protocol semantics. Poll has no standalone payload table and is governed by §6.1.38 and Table 399. Query Asynchronous Requests Table 286 retains its caption/content discrepancy and remains generic. | Existing `ClientResponseView` readers for 52 operation identities; `ClientOperationOutcome::{cancel_response,poll_response,process_response,query_async_requests_response}` for the four asynchronous flows | Existing per-operation parser tests; new matching/mismatching async reader cases in the client execution tests; complete table map in `gap-matrix.md` |
| KMIPKIT-0052-R057 | KMIP v2.1 §8.5 Table 398 | Expose the complete ordered response header structure to callers. | `crates/kmipkit-protocol/src/message/header.rs::ResponseHeaderView::raw`; `crates/kmipkit-client/src/execute.rs::ClientBatchItemResponse::response_header_raw` | `crates/kmipkit-protocol/tests/message_accessors.rs::response_header_raw_preserves_order_unknown_fields_and_repetitions`; compiled C consumer and Java/Python TLS response tests |
| KMIPKIT-0052-R058 | KMIP v2.1 §8.6 Table 399; §10.1.2 | Return the Response Payload only when present and preserve its decoded ordered Structure children. Do not fabricate a missing payload. | `crates/kmipkit-protocol/src/message/batch.rs::ResponseBatchItemView::raw`; `crates/kmipkit-client/src/execute.rs::ClientBatchItemResponse::raw`, `ClientOperationOutcome::raw` | `crates/kmipkit-protocol/tests/message_accessors.rs::{failure_response_payload_raw_is_absent,success_response_payload_raw_preserves_empty_structure_and_extensions,pending_response_payload_raw_preserves_empty_shape_and_correlation}`; client async execution tests and `hash_mac_signature_execution_tests.rs` |
| KMIPKIT-0052-R059 | KMIP v2.1 §9.13; §11.56 | Preserve generic vendor extension fields in the accepted `0x540000`–`0x54FFFF` tag range when they occur in a response tree. | `ClientBatchItemResponse::raw`; owned C/JNI/CFFI response-tree copies; `crates/kmipkit-ttlv/src/item.rs::Item::as_structure` | Rust KeyBlock and ServerInformation tests; compiled C consumer tests for both paths; Java and Python TLS 1.3/mTLS nested traversal tests |
| KMIPKIT-0052-R060 | KMIP v2.1 §3.1 Table 12; §7.37 Table 389; §6.1.19 Table 221 | Keep nested Key Block, Server Information, and Get Any Object descendants reachable as ordered structures in generic response data. | `crates/kmipkit-ttlv/src/item.rs::Item::as_structure` and the borrowed response `StructureView` tree | `crates/kmipkit-ttlv/tests/public_api.rs::item_exposes_a_borrowed_nested_structure_without_copying`; `tests/ui/item_structure_view_borrows_owner.rs`; nested Get/Query response tests |

## Project Decisions

| ID | Decision | Evidence |
|---|---|---|
| KMIPKIT-0052-P001 | A high-level typed response item owns a shared `Arc<ResponseMessage>` and its batch index; raw views borrow from that owner. This preserves one source tree per decoded response message and avoids duplicating each payload. | `crates/kmipkit-client/src/execute.rs::ClientBatchItemResponse`; ADR-0018; response owner established once in `validate_response`; `reordered_hash_batch_responses_keep_the_original_request_context` proves pointer sharing, correct associated raw payload, and final owner release |
| KMIPKIT-0052-P002 | `.raw()` represents the decoded ordered TTLV tree, not received bytes or padding. | English and Spanish response-navigation sections document that decoder-discarded padding prevents byte-exact replay; accessors lend/copy the decoded `Structure` tree. |
| KMIPKIT-0052-P003 | Failure or Pending outcomes with no Response Payload return `None`; an actual empty Structure remains `Some(empty_view)`. | `crates/kmipkit-protocol/tests/message_accessors.rs::response_batch_and_extension_accessors_preserve_all_result_shapes`; Rust client tests cover failure, Pending, and empty successful payloads; C handles the absent convention. |
| KMIPKIT-0052-P004 | Rust response views borrow; C/JNI/CFFI raw Structure handles own zeroizing copies and require explicit release/close. | `tests/ui/item_structure_view_borrows_owner.rs`; C consumer releases the response before traversing its copy; Java `AutoCloseable` and Python `close()` integration tests traverse after closing the response. |
| KMIPKIT-0052-P005 | The Table 286 query-async response remains generic until the separate approved discrepancy resolution; no interpretation is inferred from its caption. | `crates/kmipkit-client/tests/unit/query_async_execution_tests.rs`; `query_async_requests_response()` returns its typed outer model while raw payload access remains generic. |
| KMIPKIT-0052-P006 | Generic raw traversal never formats payload values automatically; explicit caller access does not weaken redaction of Debug/errors. | Rust Get response Debug tests, compiled C nested-byte consumer checks, Java TLS canary assertions, and Python `test_raw_response_copies_navigate_vendor_extensions_and_outlive_result_close`. |

## Verification Record

Local verification on Windows x86_64-pc-windows-msvc:

- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0052_response_navigation -v` — 3 passed.
- `python tools/api_manifest/generate.py --check` — all 12 generated outputs verified.
- `cargo test --workspace --all-features` — passed, including workspace tests and doctests; the compiled C consumer is enabled by `--all-features`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — passed.
- `python -B tools/api_manifest/audit_rust_exports.py --repo-root . --report <temporary-report> --check` — 7,757/7,757 declarations matched; 0 missing, unobserved, kind mismatches, conditional mismatches, or issues.
- `python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report <temporary-report> --check` — passed; C, Java, and Python declaration inventories match the manifest with no missing, unobserved, or issue entries.
- `uv run --python 3.12 --project . --with-requirements requirements-coverage.txt python -m pytest tests/test_response_raw_navigation.py -q` (from `bindings/python`) — 1 passed.
- `gradlew.bat test --rerun-tasks --tests 'org.kmipkit.ClientFacadeTlsIntegrationTest.pingQueryAndLogoutRoundTripThroughNativeJniOverDeterministicTls13Peer' --tests 'org.kmipkit.ClientFacadeTlsIntegrationTest.requestBuildersEncodeValuesAndOmissionsOverMtls'` (from `bindings/java`) — passed; both selected tests were rerun.

Coverage thresholds and remote CI remain deferred under the maintainer's active instruction about GitHub billing. The local Windows run emitted non-fatal AWS-LC/MSVC feature-probe and missing-NASM warnings; test, lint, and Java builds completed successfully. This evidence does not claim cross-platform CI, formal conformance, certification, or interoperability with a KMIP server.

## Stack synchronization — 2026-11-07

The latest KMIPKIT-0051 head `17b6d5708` was merged into this branch by
`bba190446`. After synchronization, `cargo fmt --all --check`,
`python -B tools/api_manifest/generate.py --check --repo-root .` (12 outputs),
`python -B -m unittest tools.api_manifest.tests.test_reconcile_0052_response_navigation -v`
(3 tests), and `git diff --check` passed. The inherited untracked
`bindings/java/target/` and `bindings/python/uv.lock` artifacts were not staged.

## Final predecessor synchronization — 2026-11-07

The replacement stack's final KMIPKIT-0051 head `c0aecd29c` was merged through
`65019697e`. This resolved the previously observed manifest conflict by taking
the reviewed predecessor manifest, rerunning
`reconcile_0052_response_navigation.py --write`, and regenerating all public
outputs with `tools/api_manifest/generate.py --write`; no generated output was
manually edited. The official generator then verified all 12 outputs with
`--check`.

The first synchronized T102 run exposed stale totals and fingerprints: 0052
adds 28 public declarations and two exact JNI bridge exclusions. Red commit
`a875e6272` pins those deltas against the predecessor manifest at
`c0aecd29c`; its focused test failed at the old declaration total (13,344 vs.
13,372) and old exclusion total (17,426 vs. 17,428). Green commit
`08f1eceb8` updates the shared totals and the three affected historical
fingerprints. The focused snapshot modules passed 18/18, and the full T102
discovery passed 403/403 in 152.943 seconds. `git diff --check` passed.

Independent review also confirmed that four async typed readers currently
exist only on the Rust outcome API; foreign bindings expose generic outcome
metadata/payload. The approved four-language parity requirement is tracked as
an open convergence task in KMIPKIT-0053. Generic metadata mappings are not
accepted as proof of typed-reader parity. A review observation about the
nested-vendor test commit ordering is recorded for the final Red/Green/Refactor
audit; the existing Red tests fail for missing borrowed nested-Structure and
raw-response access, while the later vendor-path cases remain acceptance
tests.
