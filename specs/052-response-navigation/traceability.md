# KMIPKIT-0052 Traceability

Normative identifiers use `KMIPKIT-0052-R###`. Project API/ownership decisions use `KMIPKIT-0052-P###` and do not claim to be OASIS requirements.

## Normative Requirements

| ID | OASIS source | Requirement interpretation | Implementation | Executable verification |
|---|---|---|---|---|
| KMIPKIT-0052-R001–R056 | KMIP v2.1 §6.1 operation-specific Response Payload tables listed row-by-row in `gap-matrix.md`; asynchronous operations: §6.1.5 Table 177, §6.1.38 Table 276 and §8.6 Table 399, §6.1.39 Table 279, §6.1.41 Table 286 | Preserve and expose each supported operation's response result and payload presence without assigning new protocol semantics. Poll has no standalone Poll Response Payload table and is governed by §6.1.38 and Table 399. Query Asynchronous Requests Table 286 retains its caption/content discrepancy and remains generic. All four async response readers are operation-checked across Rust, C, Java, and Python. | The 52 `ClientResponseView` readers plus `ClientOperationOutcome::{cancel_response,poll_response,process_response,query_async_requests_response}`; C `kmipkit_client_async_operation_outcome_{cancel,poll,process,query_async_requests}_response_v1`; Java `AsyncOperationOutcome::{cancelResponse,pollResponse,processResponse,queryAsyncRequestsResponse}`; Python `AsyncOperationOutcome::{cancel_response,poll_response,process_response,query_async_requests_response}` | Existing per-operation parser tests; `async_operation_abi_tests::cancel_and_process_typed_readers_are_called_from_the_compiled_c_consumer`, `poll_execution_with_options_returns_one_exact_async_outcome`, and `query_async_requests_executes_once_and_has_no_operation_correlation_value`; Java `ClientFacadeTlsIntegrationTest.defaultAsyncOperationsUseTheirNativeJniEntrypoints`; Python `test_async_typed_response_reader_returns_an_owner_backed_typed_view`, `test_async_typed_response_readers_return_none_for_other_operations`, `test_async_response_view_uses_the_outcome_lifetime`; complete table map in `gap-matrix.md` |
| KMIPKIT-0052-R057 | KMIP v2.1 §8.5 Table 398 | Expose the complete ordered response header structure to callers. | `crates/kmipkit-protocol/src/message/header.rs::ResponseHeaderView::raw`; `crates/kmipkit-client/src/execute.rs::ClientBatchItemResponse::response_header_raw` | `crates/kmipkit-protocol/tests/message_accessors.rs::response_header_raw_preserves_order_unknown_fields_and_repetitions`; compiled C consumer and Java/Python TLS response tests |
| KMIPKIT-0052-R058 | KMIP v2.1 §8.6 Table 399; §10.1.2 | Return the Response Payload only when present and preserve its decoded ordered Structure children. Do not fabricate a missing payload. A present empty Process payload remains present and has zero members even for Pending. | `crates/kmipkit-protocol/src/message/batch.rs::ResponseBatchItemView::raw`; `crates/kmipkit-client/src/execute.rs::ClientBatchItemResponse::raw`, `ClientOperationOutcome::raw`; C/JNI/CFFI async readers query the owner's payload presence and generic raw accessor returns it regardless of result status | `crates/kmipkit-protocol/tests/message_accessors.rs::{failure_response_payload_raw_is_absent,success_response_payload_raw_preserves_empty_structure_and_extensions,pending_response_payload_raw_preserves_empty_shape_and_correlation}`; Rust client async execution tests; compiled C consumer async reader tests; Java/Python async response parity tests; `hash_mac_signature_execution_tests.rs` |
| KMIPKIT-0052-R059 | KMIP v2.1 §9.13; §11.56 | Preserve generic vendor extension fields in the accepted `0x540000`–`0x54FFFF` tag range when they occur in a response tree. | `ClientBatchItemResponse::raw`; owned C/JNI/CFFI response-tree copies; `crates/kmipkit-ttlv/src/item.rs::Item::as_structure` | Rust KeyBlock and ServerInformation tests; compiled C consumer tests for both paths; Java and Python TLS 1.3/mTLS nested traversal tests |
| KMIPKIT-0052-R060 | KMIP v2.1 §3.1 Table 12; §7.37 Table 389; §6.1.19 Table 221 | Keep nested Key Block, Server Information, and Get Any Object descendants reachable as ordered structures in generic response data. | `crates/kmipkit-ttlv/src/item.rs::Item::as_structure` and the borrowed response `StructureView` tree | `crates/kmipkit-ttlv/tests/public_api.rs::item_exposes_a_borrowed_nested_structure_without_copying`; `tests/ui/item_structure_view_borrows_owner.rs`; nested Get/Query response tests |

## Project Decisions

| ID | Decision | Evidence |
|---|---|---|
| KMIPKIT-0052-P001 | A high-level typed response item owns a shared `Arc<ResponseMessage>` and its batch index; raw views borrow from that owner. This preserves one source tree per decoded response message and avoids duplicating each payload. | `crates/kmipkit-client/src/execute.rs::ClientBatchItemResponse`; ADR-0018; response owner established once in `validate_response`; `reordered_hash_batch_responses_keep_the_original_request_context` proves pointer sharing, correct associated raw payload, and final owner release |
| KMIPKIT-0052-P002 | `.raw()` represents the decoded ordered TTLV tree, not received bytes or padding. | English and Spanish response-navigation sections document that decoder-discarded padding prevents byte-exact replay; accessors lend/copy the decoded `Structure` tree. |
| KMIPKIT-0052-P003 | Failure or Pending outcomes with no Response Payload return `None`; an actual empty Structure remains `Some(empty_view)`. Async typed readers report payload presence independently of result status, including present empty Pending Process payload. | `crates/kmipkit-protocol/tests/message_accessors.rs::response_batch_and_extension_accessors_preserve_all_result_shapes`; Rust client tests cover failure, Pending, and empty payloads; C `has_response_payload` and `response_payload_member_count`; Java `payloadMemberCount()`; Python `payload_member_count`; tests distinguish absent from present empty. |
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

## Async reader parity follow-up — 2026-11-07

The approved KMIPKIT-0052 all-language parity requirement was closed with four
operation-specific typed response views in Rust, C, Java, and Python. This
supersedes the earlier review observation above that the foreign async typed
readers were an open KMIPKIT-0053 convergence item. The Table 286 payload
remains generic under KMIPKIT-DEC-015; no caption interpretation or field
schema is added here.

Red evidence: commit `b7011f69119a069b08e24b03dcda40d5dd82c7c1` was checked out
in the isolated `C:\Users\ramp1953\AppData\Local\Temp\kmipkit-0052-red-check`
worktree. Python command (from `bindings/python`):
`$env:PYTHONPATH = 'src'; uv run --python 3.12 --project . --no-sync --with-requirements requirements-coverage.txt python -m pytest tests/test_async_operation_mapping.py -k "async_typed_response_reader or async_response_view_uses_the_outcome_lifetime or cancel_preserves_exact_echo_in_response_payload" -q` — expected Red: 5 failed, 1 passed, 20 deselected; all failures were missing typed reader attributes. C command with `CARGO_TARGET_DIR` set to `C:\Users\ramp1953\AppData\Local\Temp\kmipkit-0052-review-target`: `cargo test -p kmipkit-ffi --lib --features coverage-c-consumer async_operation_abi_tests::cancel_and_process_typed_readers_are_called_from_the_compiled_c_consumer` — expected Red at compile time because the typed response record and reader symbols were absent. Java command `gradlew.bat compileTestJava --no-daemon` — expected Red at compile time with 10 missing `AsyncOperationOutcome` reader methods. Green adds the readers, tests, manifest routes, C v1 record, and generated outputs. Refactor reconciles docs, matrix, traceability, and generator checks. The phase commits are retained separately.

Focused verification on Windows x86_64-pc-windows-msvc:

- C: `cargo test -p kmipkit-ffi --lib --features coverage-c-consumer async_operation_abi_tests::cancel_and_process_typed_readers_are_called_from_the_compiled_c_consumer` — passed; compiled C calls each matching reader and verifies all incompatible readers, including present empty Pending Process payload.
- C: focused Poll and Query cases `poll_execution_with_options_returns_one_exact_async_outcome` and `query_async_requests_executes_once_and_has_no_operation_correlation_value` — passed.
- Java: `gradlew.bat compileTestJava --no-daemon` and `gradlew.bat test --tests org.kmipkit.ClientFacadeTlsIntegrationTest.defaultAsyncOperationsUseTheirNativeJniEntrypoints --no-daemon -x buildNativeLibrary -I <temporary init script>` — passed. JNI was built under the temporary directory; `bindings/java/target/` was not staged or used for this build.
- Python: `uv run --python 3.12 --project . --no-sync --with-requirements requirements-coverage.txt python -m pytest tests/test_async_operation_mapping.py -q` from `bindings/python` — 27 passed. `bindings/python/uv.lock` was not staged.
- Snapshot Red commits: `bab095675` pinned 78 declarations (the prior manifest had delta 28); `47dc5de1a` pinned 5 exclusions before the Python adapter routes were audited (the prior manifest had delta 2); `cc5acb849`, `88e428143`, `99980b254`, and `821fe0b75` pinned the changed T102 declaration, exclusion, and aggregate manifest fingerprints. Each test-only commit was run against its prior manifest and failed on the old count/hash as expected.
- Final T102 totals: 13,422 public declarations (+78) and 17,448 approved exclusions (+22) against baseline `c0aecd29c`. The 22 exclusions comprise the two JNI bridges, three fixed-width C record framing fields, and 17 exact Python internal constructor/slot/inherited-alias routes. The full T102 discovery passed 404/404 after updating the snapshots.
- Final changed T102 fingerprints: Certify declarations `09d1e9f4b68ae00d56a103a86e7396d55912e643245659df612c35d5f40f8055`; optional request structure declarations `ea3e7e4f8ba36bb7ec2d11f45efdb3007ba2a2669d3cea221db73e7958c3d50e`; response-view unchanged declarations `ca2276063a2a02540b5090be87cee56248fa8f01368853172b1caa01e48b9703`; aggregate other manifest `41944b8d3ee99d66bc64c0d4c535cb5851ccfb0cd5aff72e88ca596f3a4634ee`.
- Final focused reconciliation and snapshot tests: `python -B -m unittest tools.api_manifest.tests.test_reconcile_0052_response_navigation tools.api_manifest.tests.test_t102_0052_route_total tools.api_manifest.tests.test_t102_certify_request_builder_mapping.T102CertifyRequestBuilderMappingTests.test_non_target_declarations_and_operation_rows_match_snapshot tools.api_manifest.tests.test_t102_optional_request_structure_mapping.T102OptionalRequestStructureMappingTests.test_all_non_target_declarations_and_operations_match_the_pinned_snapshot tools.api_manifest.tests.test_t102_response_view_mappings.T102ResponseViewMappingTests.test_no_unlisted_declaration_or_operation_row_changes -v` — 8 passed.
- Final generator/export checks: `python -B tools/api_manifest/generate.py --repo-root . --check` — 12 outputs verified; Rust export audit — 7,777/7,777 matched with zero missing, unobserved, mismatch, or issue entries; binding export audit — C 1,724/1,724, Java 1,901/1,901, Python 2,020/2,020, all with zero missing, unobserved, or issues.
- `cargo fmt --all --check` passed. `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed after removing a duplicate FFI doc comment. `cargo test --workspace --all-features` passed, including workspace tests and doctests; three Cosmian server integration tests were ignored because no local server was available.
- Red/Green/Refactor implementation commits: Red `b7011f691`; Green `db5c09451`. Refactor consists of this separate documentation, traceability, task, and gap-matrix update; its commit is recorded in the review summary.
- Generated output is produced only with `python -B tools/api_manifest/generate.py --repo-root . --write`; all generated outputs were verified with the official `--check`. No generated output is edited by hand. Coverage and remote CI remain deferred per the maintainer instruction; this follow-up has not been pushed and no PR metadata was changed.
