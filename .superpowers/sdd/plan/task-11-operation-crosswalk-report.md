# KMIPKIT-0010 T002 — Operation crosswalk report

Date: 2026-10-15
Branch: `feature/KMIPKIT-0010-profile-conformance-t001-t002`
Workspace: `C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit`
Recorded BASE before this slice: `ab55760a152a5dedf9b45942c6912d1f673fc04f`

## Scope and source decision

This slice adds the eighth source-ordered mapping under Baseline Client Profiles v2.1 §5.1.1 item 5: `Operation` to `KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION`. Its independent-oracle row has `requirement_ids: []`, `source_clause_ids: [KMIPKIT-CLAUSE-SPEC-8.3-001, KMIPKIT-CLAUSE-SPEC-8.6-001]`, in that order, and `mapping_status: reviewed`. The parent condition remains null / `profile_conditional` / `pending_t002_crosswalk`.

Specification §§8.3 and 8.6 provide separately pinned placement rows: Request Batch Item `Operation / Yes` at [42882, 42890], and Response Batch Item `Operation / Yes, if specified in Request Batch Item` at [43344, 43353]. The catalog's matching clauses are table-header `REQUIRED` extractor hits, classified `non_applicable` with no separate requirement IDs; this does not make the request field optional, because its row explicitly says `Yes`. Section 8.6 is response-placement provenance only. Section 9.15 lines [45042, 45080] define the requested/returned Operation message field and its Enumeration encoding. Section 11.36 lines [52885, 53697] define 60 enumeration values. Section 11.56 lines [60122, 60128] assign tag `42005C`.

The map does not attach item-4 Get/Get Attributes/Locate/Query requirements or the shared profile requirement. FR-005 remains explicit: profile-required operations are a minimum and not an allowlist. `Operation` also names the §11.36 enumeration, §§8.3/8.6 placement fields, §7.2/§7.25 structure members, and §11.56 tag; those are excluded as substitutes for the canonical §9.15 message field. Generic-tag candidate `KMIPKIT-REQ-SPEC-11.56-001` has unresolved actor, direction, and scope and is not mapped; separately, the direction for `KMIPKIT-REQ-PROF-6.1-003` conflicts with its source review (`unspecified`). Both are deferred to KMIPKIT-0002 and do not support inferred behavior. The §8.6-001 row used here is distinct from the §8.6-004 metadata issue recorded for Message Extension.

Six item-5 values remain unmapped in source order: Protocol Version, Result Reason, Result Status, Server Correlation Value, Time Stamp, and Unique Batch Item ID. No catalog ownership, runtime, generated artifact, or pinned upstream OASIS file changed. T002 remains open; this mapping is not a profile-conformance claim.

## TDD commits and results

### Red — DCO-signed commit `d92c8859241431783e4f4adc23dde289cb8d4588`

The first attempted command used a nonexistent unittest class:

```powershell
python -m unittest tools.normative_catalog.tests.test_generate_profiles.TestGenerateProfiles.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk
```

It exited 1 at test discovery with `AttributeError: module 'tools.normative_catalog.tests.test_generate_profiles' has no attribute 'TestGenerateProfiles'`. This was a class-name invocation error, not a product or assertion result.

Complete output:

```text
E
======================================================================
ERROR: TestGenerateProfiles (unittest.loader._FailedTest.TestGenerateProfiles)
----------------------------------------------------------------------
AttributeError: module 'tools.normative_catalog.tests.test_generate_profiles' has no attribute 'TestGenerateProfiles'

----------------------------------------------------------------------
Ran 1 test in 0.000s

FAILED (errors=1)
```

Corrected command:

```powershell
python -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk
```

Result: expected Red, exit 1; one assertion failure, zero errors. The assertion reported `operation_mapping` was `None` instead of the reviewed Operation mapping. No implementation code was present in the Red commit.

Complete failure output:

```text
F
======================================================================
FAIL: test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk (tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk)
Operation maps to its §9.15 field while placements stay provenance, not client REQs.
----------------------------------------------------------------------
Traceback (most recent call last):
  File "C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\tools\normative_catalog\tests\test_generate_profiles.py", line 2141, in test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk
    self.assertEqual(operation_mapping, expected_mapping)
AssertionError: None != {'item': 'Operation', 'element_id': 'KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION', 'requirement_ids': [], 'source_clause_ids': ['KMIPKIT-CLAUSE-SPEC-8.3-001', 'KMIPKIT-CLAUSE-SPEC-8.6-001'], 'mapping_status': 'reviewed'}

----------------------------------------------------------------------
Ran 1 test in 0.059s

FAILED (failures=1)
```

### Green — DCO-signed commit `5213fd756564aaa6aae16577f28f119494de1ce4`

Added the reviewed Operation row to the independent closure fixture and changed the aggregate crosswalk expectation from seven to eight mappings, leaving the parent pending and six later values unmapped.

```powershell
python -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk
python -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk
```

Result: the direct Operation regression passed 1/1; the aggregate and Operation regressions passed together 2/2. The full source-backed checks were then added during Refactor.

### Refactor/evidence — DCO-signed commit

Refactor hardened the regression with exact pinned source ranges, separate request/response requiredness, table-clause classifications, §9.15 type/definition, all 60 §11.36 values, tag 0x42005C, exact same-name exclusions, FR-005, source order, six unmapped values, and unchanged parent status. It also changed the older Message Extension regression to select its mapping by item; the aggregate regression continues to pin exact source order.

During test hardening, a one-line Profiles range `[4598, 4598]` included an attribute tail because the HTML fragment begins mid-`li`. The source inspection showed the item spans lines [4597, 4598]; the test now normalizes that exact two-line span, which yields `Operation`. The focused Operation regression then passed. This was a test locator correction, not a change to the source mapping.

The first full normative suite after appending Operation exposed this stale positional expectation:

```powershell
py -3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'
```

Complete failure output:

```text
FAIL: test_baseline_message_extension_condition_has_exact_reviewed_client_crosswalk (test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_extension_condition_has_exact_reviewed_client_crosswalk)
Message Extension cites both placements while only reviewed client duties enter the mapping.
----------------------------------------------------------------------
Traceback (most recent call last):
  File "C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\tools\normative_catalog\tests\test_generate_profiles.py", line 1921, in test_baseline_message_extension_condition_has_exact_reviewed_client_crosswalk
    self.assertEqual(condition["supported_item_mappings"][-1], expected_mapping)
AssertionError: {'item': 'Operation', 'element_id': 'KMIPKIT-ELEM-M[162 chars]wed'} != {'item': 'Message Extension', 'element_id': 'KMIPKI[302 chars]wed'}
Diff is 1029 characters long. Set self.maxDiff to None to see it.
Ran 559 tests in 313.706s
FAILED (failures=1, skipped=9)
```

At BASE `ab55760a`, the oracle had seven mappings and Message Extension was last, so its positional assertion had matched the then-current list. After adding the next source-ordered row, `[-1]` correctly resolved to Operation. The Message Extension test reproduced the failure by itself; the Operation regression passed by itself. The test now selects the mapping whose item is Message Extension. The exact commands and results after that correction are below.

## Final verification commands and results

The three focused regressions were run serially:

| Command | Result |
|---|---|
| `py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk -v` | PASS, 1/1. |
| `py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_extension_condition_has_exact_reviewed_client_crosswalk -v` | PASS, 1/1; selects by item. |
| `py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk -v` | PASS, 1/1. |
| `py -3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'` | PASS: 559 tests in 313.176s; 9 skipped. |
| `py -3 -B tools/normative_catalog/validate.py --repo-root .` | PASS: `Catalog valid: sources=4 clauses=1411 records=4117`. |
| `py -3 -B tools/normative_catalog/generate_profiles.py --repo-root . --check` | PASS: `Profile metadata verified: crates/kmipkit-protocol/src/profiles_generated.rs`. |
| `py -3 -B tools/normative_catalog/report.py --repo-root . --check` | PASS: coverage report verified at `specification/catalog/coverage-report.md`. |
| `py -3 -B -m tools.normative_catalog.check_immutable_sources --repo-root . --base-sha ab55760a152a5dedf9b45942c6912d1f673fc04f` | PASS: pinned upstream OASIS sources match the exact task base. |
| `cargo fmt --all --check` | PASS, exit 0; no output. |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS, exit 0. Existing AWS-LC compiler-feature/`stdalign_check` and missing NASM probe warnings were non-fatal. |
| `cargo test --workspace --all-features --locked` | PASS, exit 0 across workspace targets and doc tests. Existing AWS-LC/`stdalign_check`, NASM and C-ABI linker LNK4217 warnings were non-fatal. |
| `cargo llvm-cov --workspace --all-features --locked --summary-only` | PASS, exit 0. Workspace regions 39,410/42,377 (93.00%); TTLV regions 981/1,033 (94.97%); decoder regions 517/562 (91.99%). |
| `$operationCoverageSummaryPath = Join-Path $env:TEMP 'kmipkit-operation-llvm-cov-summary.json'; cargo llvm-cov report --json --summary-only --locked --output-path $operationCoverageSummaryPath` | PASS; saved `C:\Users\ramp1953\AppData\Local\Temp\kmipkit-operation-llvm-cov-summary.json`. Workspace lines 30,039/31,491 (95.39%); TTLV lines 748/761 (98.29%); decoder lines 406/419 (96.90%). |
| `git diff --check` | PASS, exit 0. The only output was Git's line-ending notice that the working copies of `implementation-evidence.md` and `research.md` will be normalized from CRLF to LF the next time Git touches them; no whitespace errors were reported. |

The prior Message Extension coverage JSON at `C:\Users\ramp1953\AppData\Local\Temp\kmipkit-t002-llvm-cov-summary.json` reported workspace lines 30,038/31,491. In the Operation rerun, only `kmipkit-transport/src/timeout.rs` changed its covered-line count from 472 to 473 of 508. This is recorded as unexplained measurement variation; no code or test cause is inferred. Both snapshots round to 95.39% workspace line coverage, and TTLV remains 98.29%.

The earlier unreproduced validator read diagnostic, its exact emitted error line, instrumented successful source reads, and serial CLI rerun remain recorded in `task-10-message-extension-report.md`. The Operation validator command above was run serially after the normative suite and passed. No validator code changed.

The final exact `git rev-parse HEAD` is reported in the implementation handoff after the DCO-signed Refactor/evidence commit. This report is included in that commit; no push or PR update was performed.

## Independent QA correction

QA review of `c3e86ec66a1a0c6e4c562dab74761a48b41863d2` approved the mapping and test shape but blocked the documentation wording because a reader could confuse “no separate requirement ID from the table-header extractor hit” with “no request-field requiredness.” The evidence, research, report, and progress ledger now state explicitly that §8.3 says `Operation / Yes`, while §8.6 says `Yes, if specified in Request Batch Item`; `requirement_ids: []` does not make the request field optional. The regression additionally pins direction and `client_1_0` scope for both placements and `both` / `client_1_0` for the canonical §9.15 field. The stale test docstring now reports eight mappings. The deferred generic-tag candidate `KMIPKIT-REQ-SPEC-11.56-001` and the profile-direction mismatch for `KMIPKIT-REQ-PROF-6.1-003` are distinguished from the §8.6-004 metadata issue.

Fresh verification after these corrections: the three focused crosswalk regressions passed (3/3) with `py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_extension_condition_has_exact_reviewed_client_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_operation_condition_has_exact_reviewed_structural_crosswalk -v`; `git diff --check` exited 0. These edits are documentation and test assertions only. Re-review of the corrected commit remains pending.
