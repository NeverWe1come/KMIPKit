# KMIPKIT-0010 T002 — Time Stamp crosswalk report

Date: 2026-10-15
Branch: `feature/KMIPKIT-0010-profile-conformance-t001-t002`
BASE before this slice: `0d3926d6d3b0280c19a8f0be873ad2098eb2bbcf`

## Scope and source decision

This slice adds the thirteenth source-ordered mapping under Baseline Client Profiles v2.1 §5.1.1 item 5: `Time Stamp` → `KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP`. It cites `KMIPKIT-CLAUSE-SPEC-9.20-001` and links the client capability requirements `KMIPKIT-REQ-SPEC-9.20-001-001` and `KMIPKIT-REQ-SPEC-9.20-001-002`. The parent condition remains null / `profile_conditional` / `pending_t002_crosswalk`.

Specification v2.1 §9.20 makes the field optional in a client request (pinned lines 45309–45310). §8.2 Table 395 places it in the request header with `Required=No` (lines 42780–42796). §8.5 Table 398 says `Required=Yes` in the response (lines 43143–43159), but §9.20 states that the required request/response behavior is for a server request or response. This is not a client send duty. Table 425 defines Date-Time (lines 45335–45345); §11.56 assigns tag `0x420092` (lines 60820–60830).

The source-review oracle separates five atoms: optional client request; server request/response duty; unresolved MAY with unspecified actor; server-only MAY to reject a timestamp too far from server time; and conditional client MAY for a client without a real-time clock that uses a countdown timer. The conditional capability remains deferred; the map does not claim timestamp generation or countdown-based serialization.

Known catalog defects remain assigned to KMIPKIT-0002: the mixed §9.20 clause metadata blurs actors, the response placement is server-to-client but links the optional client request requirement, and the canonical element links only `...-001-001` while `...-001-002` has no element association. The crosswalk records source applicability independently and does not fix the normative catalog.

## TDD commits and results

### Red — DCO-signed commit `90ed2006`

Added the aggregate and dedicated regression before the oracle row:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_time_stamp_condition_maps_client_capabilities_not_server_duties -v
```

Expected Red: two assertion failures, zero errors. The aggregate lacked the thirteenth row, and the dedicated regression could not find a Time Stamp mapping.

### Green — DCO-signed commit `fd9ca838`

Added only the reviewed structural row to the independent oracle and updated dependent source-order/remainder assertions. The initial focused run exposed two stale assertions (formatted HTML has a style prefix at the selected line, and the Operation crosswalk count needed incrementing); the corrections passed the rerun:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_time_stamp_condition_maps_client_capabilities_not_server_duties tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_protocol_version_condition_preserves_structure_and_unresolved_compatibility tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_result_status_condition_maps_pending_client_duty tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_server_correlation_value_condition_maps_server_response_metadata -v
```

PASS: 6/6.

## Verification status

| Check | Result |
|---|---|
| Focused aggregate, Time Stamp, and neighboring crosswalk regressions | PASS: 6/6 after Green. |
| Full normative catalog suite | PASS: 564 tests in 312.671 seconds; 9 skipped. |
| Catalog validation | PASS: 4 sources, 1,411 clauses, 4,117 records. |
| Generated profile metadata check | PASS: generated file is current. |
| Coverage report check | PASS: report is current. |
| Pinned OASIS source immutability | PASS against base `0d3926d6d3b0280c19a8f0be873ad2098eb2bbcf`. |
| Independent QA | PASS: no blocking findings. P3 note: the test does not assert the known incorrect response/canonical `requirement_ids` links, avoiding freezing KMIPKIT-0002 catalog defects. |
| Exact-head GitHub CI | Pending. |

The item-5 crosswalk now maps 13/14 values; only `Unique Batch Item ID` remains. This partial mapping is not an official OASIS test execution or a profile-conformance claim. T002 stays open.
