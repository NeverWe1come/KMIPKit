# T071 shared parity audit report

Date: 2026-10-30

Branch: `feature/KMIPKIT-0043-t076-shared-client-mapping`

TDD commit range: `c5d12d1ee..c46630915` (six Red/Green/Refactor commits, all local). The report and fixture formatting follow-ups are also local; nothing was pushed or merged.

Task status: T071 remains open.

## Scope and result

Audited the 13 entries in `tests/fixtures/api/shared-capability-parity.json` and their named Rust, compiled C, Java, and Python consumers. The repository parity test confirms that each fixture has all four language mappings, verifies the named consumer references, and checks the Discover Versions and extension source-fixture relationships. Source review found a concrete fixture-to-behavior gap in the C mTLS path: its assertion used a hard-coded Result Message instead of the shared fixture.

The first repair connected the C expected argument to `expectedOutput.resultMessage`, but independent review exposed an oracle coupling: the harness also used that same expected value to build the simulated server response. A mismatch between fixture input and expected output was therefore hidden. The corrected harness now constructs the loopback server response from `input.resultMessage` and passes `expectedOutput.resultMessage` as the C consumer's expected argv value. A compiled C mTLS regression deliberately makes those values diverge and confirms the consumer rejects the response. The fixture records this supporting harness. No production API or readiness state changed.

The other 12 cases did not yield a second confirmed gap during this bounded audit. This does not establish that every mapped test dynamically consumes every expected field or that the full corpus passed in every language on this head; see verification limits below.

## Fixture inventory

The 13 case IDs and their declared capability areas are:

| Fixture | Capability area |
|---|---|
| `api-levels.shared.v1` | Local API-level discovery |
| `client-configuration.registry-isolation.v1` | Extension registry ownership/isolation |
| `errors.schema-redaction.v1` | Redacted schema errors |
| `batch.ordered-completed-pending-failed.v1` | Ordered outcomes and Pending correlation |
| `discover-versions.supported-versions.v1` | Discover Versions result mapping |
| `ttlv.open-values-and-child-order.v1` | Unknown TTLV values and ordering |
| `extensions.recognition-and-generic-preservation.v1` | Extension recognition and generic preservation |
| `creation.create.request-response.v1` | Create request/result mapping |
| `creation.key-pair.request-response.v1` | Create Key Pair request/result mapping |
| `creation.split-key.request-response.v1` | Create Split Key request/result and Pending metadata |
| `creation.result-message-on-failure.v1` | Failure Result Message presence and redaction |
| `codec-limits.defaults-and-ranges.v1` | Shared CodecLimits defaults and boundaries |
| `usage-allocation.guard-accounting.v1` | Local Usage Allocation accounting boundaries |

The extension recognition case names `tests/fixtures/extensions/cases.json` as its source and identifies its two required source cases. The manifest parity test verifies those IDs exist. Its behavior consumers use the generated extension fixture corpus.

## Declared consumer references

This table records the fixture's named consumers; it is not a claim that every consumer was executed during this turn.

| Fixture | Rust | Compiled C | Java 17 | Python 3.12 |
|---|---|---|---|---|
| `api-levels.shared.v1` | `api_levels.rs#api_levels_match_the_shared_fixture_without_network_calls` | `client_abi_consumer.c#api_level_query_matches_shared_fixture_locally` | `SharedCapabilityParityFixtureTest#apiLevelQueryMatchesSharedFixtureLocally` | `test_shared_parity_fixtures.py#test_api_level_query_matches_shared_fixture_locally` |
| `client-configuration.registry-isolation.v1` | `extension_registry.rs#client_configurations_own_isolated_immutable_registries` | `extension_registry.c#test_configuration_ownership_and_isolation` | `ExtensionRegistryTest#clientConfigurationsOwnImmutableIsolatedRegistrySnapshots` | `test_extension_registry.py#test_registry_registration_is_immutable_owned_by_configuration_and_isolated` |
| `errors.schema-redaction.v1` | `extension_redaction.rs#schema_error_debug_display_and_source_chain_redact_payload_secrets` | `extension_registry.c#test_redacted_stable_errors_and_invalid_handles` | `ExtensionRegistryTest#invalidSchemaErrorsAndValidatedValueFormattingRedactPayloadText` | `test_extension_registry.py#test_failure_and_default_diagnostics_redact_secret_fixture_values` |
| `batch.ordered-completed-pending-failed.v1` | `execute_lifecycle_tests.rs#ordered_mixed_batch_outcomes_preserve_ids_and_use_one_exchange` | `client_abi_consumer.c#execute_public_tls_request` via `c_api_coverage.rs#generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls` | `ClientFacadeTlsIntegrationTest#executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange` | `test_client.py#test_batch_execute_preserves_ordered_mixed_outcomes_and_invokes_native_execute_once` |
| `discover-versions.supported-versions.v1` | Same Rust mixed-batch test; it parses both fixture cases | Same compiled C mTLS harness as the batch case | Same Java mixed-outcome integration test | `test_client.py#test_discover_versions_response_mapping_matches_shared_fixture` |
| `ttlv.open-values-and-child-order.v1` | `shared_api_parity.rs#generic_ttlv_shared_fixture_preserves_open_values_and_child_order` | `client_abi_consumer.c#shared_ttlv_fixture_preserves_open_values_and_order` via `c_api_coverage.rs#c_abi_consumer_preserves_shared_ttlv_fixture_values_and_order` | `SharedCapabilityParityFixtureTest#genericTtlvFixturePreservesOpenEnumerationBytesOrderAndViewLifecycle` | `test_shared_parity_fixtures.py#test_generic_ttlv_shared_fixture_preserves_open_values_order_and_owner_lifecycle` |
| `extensions.recognition-and-generic-preservation.v1` | `extension_registry_tests.rs#every_shared_inbound_fixture_matches_internal_outcome_ids_and_preserved_ttlv_in_both_orders` | `extension_registry.c#test_inbound_inspection_and_generic_preservation` | `ExtensionRegistryTest#everySharedInboundFixtureRunsThroughThePublicJavaAdapterInBothOrders` | `test_extension_registry.py#test_shared_inbound_fixtures_are_inspected_and_generic_ttlv_is_preserved` |
| `creation.create.request-response.v1` | `creation_parity_tests.rs#create_request_and_result_match_shared_creation_fixture` | `client_abi_consumer.c#creation_create_matches_shared_fixture` | `SharedCapabilityParityFixtureTest#creationCreateMatchesSharedFixture` | `test_shared_parity_fixtures.py#test_creation_create_matches_shared_fixture` |
| `creation.key-pair.request-response.v1` | `creation_parity_tests.rs#create_key_pair_request_and_result_match_shared_creation_fixture` | `client_abi_consumer.c#creation_create_key_pair_matches_shared_fixture` | `SharedCapabilityParityFixtureTest#creationKeyPairMatchesSharedFixture` | `test_shared_parity_fixtures.py#test_creation_key_pair_matches_shared_fixture` |
| `creation.split-key.request-response.v1` | `creation_parity_tests.rs#create_split_key_request_and_result_match_shared_creation_fixture` | `client_abi_consumer.c#creation_create_split_key_matches_shared_fixture` | `SharedCapabilityParityFixtureTest#creationSplitKeyMatchesSharedFixture` | `test_shared_parity_fixtures.py#test_creation_split_key_matches_shared_fixture` |
| `creation.result-message-on-failure.v1` | `creation_parity_tests.rs#creation_result_message_fixture_is_limited_to_operation_failure` | `client_abi_consumer.c#execute_public_tls_request` via the fixture-backed C mTLS harness | `CreationResultValueTest#failureResultMessageMatchesTheSharedFixture` | `test_shared_parity_fixtures.py#test_creation_result_message_matches_failure_fixture` |
| `codec-limits.defaults-and-ranges.v1` | `shared_api_parity.rs#codec_limits_shared_fixture_covers_defaults_ranges_and_rejection` | `client_abi_consumer.c#codec_limits_shared_fixture_matches_defaults_ranges_and_errors` via `c_api_coverage.rs#c_abi_consumer_matches_the_shared_codec_limits_fixture` | `SharedCapabilityParityFixtureTest#codecLimitsFixtureMatchesDefaultsRangesAndErrors` | `test_shared_parity_fixtures.py#test_codec_limits_shared_fixture_matches_defaults_ranges_and_errors` |
| `usage-allocation.guard-accounting.v1` | `usage_allocation_guard_tests.rs#fixture_usage_allocation_guard_matches_shared_accounting_contract` | `client_abi_consumer.c#shared_usage_allocation_guard_matches_fixture` via `client_abi_tests.rs#ffi_c_consumer_executes_guarded_encrypt_over_mtls_and_reserves_cost` | `ClientFacadeTlsIntegrationTest#usageAllocationContextGuardsJavaProtectionExecutionLocally` | `test_usage_allocation_guard_native_integration.py#test_native_python_guard_executes_encrypt_and_rejects_over_budget_before_transport` |

## TDD commits

- Red: `c5d12d1ee` — `test(0043): drive C creation failure from shared fixture`. The modified compiled-C test failed before a TLS exchange because the C consumer rejected the newly supplied fixture argument (`status 3`, zero server exchanges).
- Green: `0a4a20826` — `test(0043): bind C result message to parity fixture`. C parses the fixture argument and compares the returned Result Message against it; the server response is also built from the fixture value.
- Refactor: `090c64b34` — `refactor(0043): clarify fixture C argument validation`. Separates fixture argument-index calculation and validation without changing behavior.

The oracle-coupling correction has a separate Red/Green/Refactor sequence:

- Red: `9ddb0c63c` — `test(0043): expose C result message fixture drift`. Added a compiled C mTLS test that changes only `input.resultMessage` and expects the mismatch to be rejected. Before the harness correction, it failed because the consumer returned success (`left: 0`, `right: 0`), proving that the server response was still being sourced from the expected output.
- Green: `264d61d95` — `test(0043): separate C result fixture input and output`. The server simulator now reads `input.resultMessage`; C argv reads `expectedOutput.resultMessage`. Both the matching fixture case and divergent-value regression pass.
- Refactor: `c46630915` — `refactor(0043): name C result message expectations`. Extracts the fixture case loader and uses a named expectation enum instead of a boolean in the shared C mTLS helper.

All six TDD commits use `git commit -s`. They are local only; no push or merge was performed.

## Verification

| Check | Result |
|---|---|
| Red focused compiled-C mTLS consumer | Failed as expected before TLS dispatch: C rejected the fixture-driven argument (`status 3`; 0 exchanges). |
| Green/refactor focused compiled-C mTLS consumer | Passed: 1 test, 0 failed. It exercises the shared batch, Discover Versions, and creation response path, including the fixture-derived failure message. |
| Oracle-coupling Red regression | Failed as expected before the correction: deliberately divergent `input.resultMessage` and `expectedOutput.resultMessage` produced C success (`0`), violating the test's rejection assertion. |
| Corrected compiled C mTLS tests | Passed after Green and Refactor: `generated_public_client_abi_consumer_executes_a_typed_batch_over_mtls` and `compiled_c_consumer_rejects_divergent_creation_message_input_and_expected_output` each passed (1 test, 0 failed). |
| `tools/api_manifest/tests/test_parity.py` | Passed: 18 tests. |
| Fixture JSON and C support link after formatting correction | Valid JSON; 13 cases; the creation failure support entry resolves to the C mTLS harness. |
| Python shared fixture, client, extension registry, and Usage Allocation suites | Passed: 261 tests using Python 3.12 and a freshly built external `kmipkit_ffi.dll`. |
| `cargo fmt --all --check` | Passed after the oracle-coupling Refactor. |
| `git diff --check` | Passed for the correction before commit. |
| `cargo test --workspace --all-features` | Started with `CARGO_TARGET_DIR` outside the worktree, then stopped at the parent agent's request while still running; no workspace-wide final result is claimed. The observed output included passing API-level (1), client unit (566), creation parity (6), extension recognition (13), and extension registry (17) tests. |
| Java 17 consumer suite | Not run in this turn. |

The Windows build emitted non-fatal AWS-LC probe/NASM warnings and linker `LNK4217` warnings. The focused build and tests completed successfully. The first Python attempt used a stale test-copy DLL and failed to find the newer Usage Allocation symbol; rerunning with the newly built DLL outside the worktree passed all 261 selected tests.

Build output used `C:\Users\ramp1953\AppData\Local\Temp\kmipkit-t071-target-red`; the existing worktree `target` directories and other untracked artifacts were not cleaned or staged.

## Remaining T071 prerequisites and limits

- T076, T077, and T078 are checked.
- Within T031–T069, T036 and T042 remain unchecked. T036 requires formal approval of draft `specs/010-profile-conformance/spec.md`, closure of its applicable profile gates, and then the accepted profile-validation API mappings/parity evidence. T042 explicitly remains open while T036 is blocked on draft KMIPKIT-0010.
- T071 remains unchecked. No profile-validation API, profile-support claim, API readiness state, or full-parity claim was added.
- This audit did not complete the full Rust workspace run or execute the Java 17 suite. Therefore the entire 13-case corpus has not been verified on all four languages at the current head. The targeted C mTLS checks validate the shared batch, Discover Versions, and creation Result Message path only.
- The Create, Create Key Pair, and Create Split Key C constructor fixture checks use compiled C behavior, but this audit did not establish a dynamic Rust fixture feed for every constructor input/output field. No second confirmed defect was changed; this remains a review limit rather than a claim of complete fixture-field parity.
