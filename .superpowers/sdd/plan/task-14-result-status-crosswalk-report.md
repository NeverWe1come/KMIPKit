# KMIPKIT-0010 T002 — Result Status crosswalk report

Date: 2026-10-15  
Branch: `feature/KMIPKIT-0010-profile-conformance-t001-t002`  
Workspace: `C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit`  
BASE before this slice: `413ea3108369f619790039f0541aba787ec50a5a`

## Scope and source decision

This slice adds the eleventh source-ordered mapping under Baseline Client Profiles v2.1 §5.1.1 item 5: `Result Status` → `KMIPKIT-ELEM-MESSAGE-FIELD-9-19-RESULT-STATUS`. It cites `KMIPKIT-CLAUSE-SPEC-9.19-001` and `KMIPKIT-CLAUSE-SPEC-9.19-002`, and links the conditional client requirement `KMIPKIT-REQ-SPEC-9.19-002`. The parent condition stays null / `profile_conditional` / `pending_t002_crosswalk`.

Profiles v2.1 §5.1.1 names Result Status at pinned HTML line 4604. Specification v2.1 §8.6 Table 399 places it in Response Batch Item and marks it `Yes` (pinned lines 43380–43398). Section 9.19 defines the server response values at lines 45244–45258 and Table 424 confirms Enumeration encoding at lines 45272–45303. The response-value clause is server-only. Separately, §9.19 states that when the received status is Operation Pending, the client polls for the actual result and uses the asynchronous correlation value for subsequent polling. That conditional client-to-server requirement is `KMIPKIT-REQ-SPEC-9.19-002`.

The element map targets the canonical §9.19 field, with the §8.6 field as response placement. The existing §8.6 catalog clause contains a role/direction inconsistency; this regression uses the pinned Table 399 text and element direction, and defers catalog correction to KMIPKIT-0002. The requirement already appears in the Baseline profile requirement set and in the KMIPKIT-0009 asynchronous operations spec, which identifies test `pending_poll_returns_without_repeating_and_zeroizes_request_and_response_copies`. Existing test `unknown_status_preserves_raw_value` covers forward-compatible preservation. The KMIPKIT-0006 CSV transfer row remains `deferred` because ownership moved to KMIPKIT-0009; it requires a later ledger reconciliation, not a runtime change in this slice.

Independent QA found that the first regression version checked the correlation-value SHALL without independently pinning the preceding Operation Pending condition and asynchronous polling procedure. The regression now asserts the exact `Operation Pending` and `actual result via asynchronous polling` phrases alongside the SHALL from the same pinned §9.19 source span. The dedicated Result Status test passes after this correction.

The crosswalk covers 11/14 values. The three remaining are Server Correlation Value, Time Stamp, and Unique Batch Item ID. This partial crosswalk makes no profile-conformance claim and does not close the parent condition.

## TDD commits and results

### Red — DCO-signed commit `aaf583ab`

Added aggregate and dedicated Result Status regressions before adding the oracle row:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_result_status_condition_maps_pending_client_duty -v
```

Expected Red: two assertion failures and zero errors. The aggregate had no eleventh mapping and the dedicated regression could not find a Result Status row.

### Green — DCO-signed commit `8e06241e`

Added only the reviewed mapping to the independent oracle. The focused aggregate and dedicated Result Status tests pass. Five focused regressions covering the aggregate, Result Status, Result Reason, Operation, and Protocol Version pass 5/5.

### Refactor / evidence

The source audit, focused regressions, full normative suite, catalog validation, generated metadata check, coverage report check, immutable-source check, independent QA, and exact-head CI are complete. The local full suite ran before the assertion-only QA refinement; GitHub CI ran the complete matrix after it.

## Verification status

| Check | Result |
|---|---|
| Five focused crosswalk regressions | PASS: 5/5. |
| Full normative catalog suite: `py -3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` | PASS: 562 tests in 314.924 seconds; 9 skipped. |
| Catalog validation: `py -3 -B tools/normative_catalog/validate.py --repo-root .` | PASS: 4 sources, 1,411 clauses, 4,117 records. |
| Generated profile metadata: `py -3 -B tools/normative_catalog/generate_profiles.py --repo-root . --check` | PASS. |
| Coverage report: `py -3 -B tools/normative_catalog/report.py --repo-root . --check` | PASS. |
| Pinned OASIS source immutability | PASS: pinned sources match base `413ea3108369f619790039f0541aba787ec50a5a`. |
| `git diff --check` | PASS. |
| Independent QA | PASS after one P2 test-precision correction. The regression now pins `Operation Pending`, asynchronous polling, and the correlation SHALL from one §9.19 source span. |
| Exact-head GitHub CI run `38738832004` on `fa206ece55d609df1fb934ee179c2e8dc38240ba` | PASS: 34 successful jobs, zero failures, and 3 skipped jobs. |
