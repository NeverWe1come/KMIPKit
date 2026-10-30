# KMIPKIT-0010 T002 — Result Reason crosswalk report

Date: 2026-10-15
Branch: `feature/KMIPKIT-0010-profile-conformance-t001-t002`
Workspace: `C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit`
BASE before this slice: `1ff726e713d8ea1c9d99413be900c565e2553e63`

## Scope and source decision

This slice adds the tenth source-ordered mapping under Baseline Client Profiles v2.1 §5.1.1 item 5: `Result Reason` → `KMIPKIT-ELEM-MESSAGE-FIELD-9-18-RESULT-REASON`. The reviewed oracle row has no client requirement IDs, cites `KMIPKIT-CLAUSE-SPEC-9.18-001` as direct provenance, and keeps the parent condition null / `profile_conditional` / `pending_t002_crosswalk`.

Profiles v2.1 §5.1.1 names Result Reason at pinned HTML line 4602. Specification v2.1 §8.6 Table 399 places it in Response Batch Item with `Yes, if Result Status is Failure` (pinned lines 43399–43418). Section 9.18 requires Result Reason for Failure and prohibits it for Success (lines 45197–45203); Table 423 confirms Enumeration encoding (lines 45207–45240). Read together, the field is required for Failure, absent for Success, and otherwise optional for statuses where its inclusion is permitted. This is server-to-client response structure, not a client request obligation. The §11.46 assigned values describe server response reasons; the crosswalk does not assign a client policy for choosing them.

The canonical target is the §9.18 field definition, matching the prior Operation (§9.15) and Protocol Version (§9.16) crosswalk convention. The §8.6 placement record remains provenance and the Enumeration record remains the value type. The source audit found no field-specific Profile requirement or client requirement. No catalog classification, production behavior, generated output, or pinned OASIS source changes.

The item-5 mapping now covers 10 of 14 values. Four remain: Result Status, Server Correlation Value, Time Stamp, and Unique Batch Item ID. T002 remains open and this partial mapping is not a profile-conformance claim or official OASIS test execution.

## TDD commits and results

### Red — DCO-signed commit `a0e7340581fd2adf56263c95fc5c3e63d3cbc58d`

Added the aggregate and dedicated Result Reason regressions before editing the independent oracle:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_result_reason_condition_preserves_response_structure -v
```

Expected Red: two assertion failures and zero errors. The aggregate did not yet include the tenth mapping; the dedicated test could not find the new structural row.

### Green — DCO-signed commit `0acee975ecea16e2cd6f0af093c720058bfaede2`

Added only the reviewed Result Reason mapping to the independent oracle. The focused aggregate and dedicated regressions passed 2/2.

### Refactor / evidence

Refactor corrects existing tests whose positional/remainder expectations became stale after appending Result Reason: the aggregate now expects 10 mappings and four unmapped values; the Operation test expects the last four source-ordered values; the Protocol Version test selects its mapping by item rather than expecting it to be last. Four focused regressions pass. The full normative suite initially exposed exactly these two stale assertions; both failed with assertions, not errors, and were reproduced directly before correction. The new mapping was not changed to satisfy stale tests.

The DCO-signed Refactor/evidence commit will record the independent source audit, this report, research/evidence updates, QA correction, and final verification results.

## Final verification

| Command | Result |
|---|---|
| Four focused crosswalk regressions (aggregate, Result Reason, Operation, Protocol Version) | PASS: 4/4; the Result Reason test also passes separately after the QA correction. |
| `py -3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` | PASS after QA correction: 561 tests in 311.123 seconds; 9 skipped. |
| `py -3 -B tools/normative_catalog/validate.py --repo-root .` | PASS: 4 sources, 1,411 clauses, 4,117 records. |
| `py -3 -B tools/normative_catalog/generate_profiles.py --repo-root . --check` | PASS. |
| `py -3 -B tools/normative_catalog/report.py --repo-root . --check` | PASS. |
| `py -3 -B -m tools.normative_catalog.check_immutable_sources --repo-root . --base-sha 1ff726e713d8ea1c9d99413be900c565e2553e63` | PASS: pinned upstream sources match exact task base. |
| `git diff --check` | PASS: no whitespace errors; Git emitted only existing CRLF-to-LF normalization notices for the two edited Markdown files. |
| Exact-head GitHub CI run `38733890903` on `62d16b7e0f68cac37edfa6b1575c46452dd0fa8b` | PASS: 34 successful jobs, zero failures, and 3 scheduled/informational skips. |
| Independent QA review of the final slice | Initial review found one P2 because the dedicated Result Reason regression selected the final list position. Fixed to select by item; reviewer confirmed the finding resolved. No remaining findings. |

No catalog, runtime, generated output, or upstream source changed. The item-5 parent condition remains pending; T002 and the broader profile-conformance task remain open.
