# KMIPKIT-0010 T002 — Server Correlation Value crosswalk report

Date: 2026-10-15
Branch: `feature/KMIPKIT-0010-profile-conformance-t001-t002`
Workspace: `C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit`
BASE before this slice: `eb6be1ec00fb816a758468802a7e094249d2b022`

## Scope and source decision

This slice adds the twelfth source-ordered mapping under Baseline Client Profiles v2.1 §5.1.1 item 5: `Server Correlation Value` → `KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE`. It cites `KMIPKIT-CLAUSE-SPEC-9.10-001` and has no KMIP client requirement IDs. The parent condition remains null / `profile_conditional` / `pending_t002_crosswalk`.

Profiles v2.1 §5.1.1 item 5 lists the value at pinned HTML lines 4605–4607. Specification v2.1 §9.10 says the value SHOULD be provided by the server, SHOULD be globally unique, and SHOULD be logged by the server with each request (pinned lines 43957–43959). For client-to-server operations, it is provided in the response; for server-to-client operations, in the request (lines 43961–43963). In the 1.0 client-initiated scope, this is response metadata. The §8.2 request-header and §8.5 response-header tables both mark it `Required=No`; §9.10 Table 409 encodes it as Text String; §11.56 assigns tag `0x420106`.

The source clause's `role`/`direction` catalog metadata conflicts with its `server_only` disposition and pinned prose. The regression asserts source text and the existing server-only disposition, not the contradictory direction. KMIPKIT-0002 remains responsible for inventory correction. The item is distinct from Asynchronous Correlation Value (`0x420006`) used by Poll and Cancel. Project invariant KMIPKIT-0007-FR-016 and test `outgoing_client_request_omits_server_correlation_value` prohibit setting or emitting Server Correlation Value in client requests; that project behavior is not misrepresented as an OASIS requirement.

## TDD commits and results

### Red — DCO-signed commit `88898bbf`

Added the aggregate and dedicated regression before the oracle row:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_server_correlation_value_condition_maps_server_response_metadata -v
```

Expected Red: two assertion failures, zero errors. The aggregate lacked the twelfth row and the dedicated regression could not find the mapping.

### Green — DCO-signed commit `6621e24c`

Added only the reviewed structural row to the independent oracle. The aggregate and dedicated regressions pass 2/2.

### Refactor / evidence

The dedicated and aggregate tests pass. The full normative suite, serial catalog validation, profile generation check, coverage report check, pinned-source immutability check, `git diff --check`, independent read-only QA, and exact-head GitHub CI all pass. QA reported no findings.

## Verification status

| Check | Result |
|---|---|
| Focused aggregate and Server Correlation Value regressions | PASS: 2/2. |
| Full normative catalog suite | PASS: 563 tests in 315.107 seconds; 9 skipped. |
| Catalog validation | PASS: 4 sources, 1,411 clauses, 4,117 records. |
| Generated profile metadata check | PASS: generated file is current. |
| Coverage report check | PASS: report is current. |
| Pinned OASIS source immutability | PASS against base `eb6be1ec00fb816a758468802a7e094249d2b022`. |
| Independent QA | PASS: no findings. |
| Exact-head GitHub CI | PASS: run `38744365487` on `2f27511c4a271275ed91af7986374c5f05d0e053`; 37 successful jobs, zero failures, zero skipped. |

The item-5 crosswalk now maps 12/14 values; `Time Stamp` and `Unique Batch Item ID` remain. This partial mapping is not an official OASIS test execution or a profile-conformance claim. T002 stays open.
