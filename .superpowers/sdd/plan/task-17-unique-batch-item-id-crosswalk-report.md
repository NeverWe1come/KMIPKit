# KMIPKIT-0010 T002 — Unique Batch Item ID crosswalk report

- Date: 2026-10-15
- Branch: `feature/KMIPKIT-0010-profile-conformance-t001-t002`
- BASE before this slice: `ddba006cdba960e765bc3b4bf0e147826544c6cc`

## Scope and source decision

This slice maps the final value in Baseline Client Profiles v2.1 §5.1.1 item 5: `Unique Batch Item ID` → `KMIPKIT-ELEM-MESSAGE-FIELD-9-21-UNIQUE-BATCH-ITEM-ID`. The source-ordered oracle row links three client capabilities: `KMIPKIT-REQ-SPEC-8.3-003`, `KMIPKIT-REQ-SPEC-9.21-001-001`, and `KMIPKIT-REQ-PROF-4.1.2-007`.

Specification v2.1 §8.3 Table 396 requires the field when Batch Count exceeds one (pinned lines 42921–42935). Profiles v2.1 §4.1.2 says a client MAY include or omit the ID for a single-item request (lines 4488–4493). Specification §9.21 makes the request field optional and says a response SHALL echo it when present (lines 45355–45360). The response echo is a server response obligation and `KMIPKIT-REQ-SPEC-9.21-001-002` is intentionally excluded from the client mapping. §8.6 Table 399 is recorded as response-placement evidence only (lines 43361–43377). Table 426 defines Byte String (lines 45378–45395); §11.56 assigns tag `0x420093` (lines 60833–60844).

The reviewed source details and known catalog metadata discrepancies are documented in `specs/010-profile-conformance/research.md`. Corrections to the mismatched actor/direction metadata remain assigned to KMIPKIT-0002; this slice does not modify the normative catalog, runtime, generated outputs, or pinned OASIS sources. Fourteen of fourteen values are now mapped under this one Baseline condition, but full T002 closure across all 18 client profiles is still open. This is not an official OASIS test execution or a profile-conformance claim.

## TDD commits and results

### Red — DCO-signed commit `375c391a`

Added the aggregate mapping assertion, exact source-order/count checks, and a dedicated source regression before the oracle row:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_unique_batch_item_id_maps_client_batch_requirements_not_server_echo -v
```

Expected Red: two assertion failures and zero errors. The aggregate oracle lacked the fourteenth mapping and the dedicated test found no Unique Batch Item ID row.

### Green — DCO-signed commit `2e7b8ebc`

Added only the reviewed oracle row and updated dependent source-order remainder/count assertions. After correcting one case-sensitive source assertion, the full generator module passed:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles -v
```

PASS: 59 tests, 1 skipped. The skipped case requires creating a symlink, which the current Windows account cannot do.

## Verification status

| Check | Result |
|---|---|
| Full normative catalog suite | PASS: 565 tests in 312.640 seconds; 9 skipped. |
| Catalog validation | PASS: 4 sources, 1,411 clauses, 4,117 records. |
| Generated profile metadata | PASS: `generate_profiles.py --check`. |
| Generated coverage report | PASS: `report.py --check`. |
| Pinned OASIS source immutability | PASS against base `ddba006cdba960e765bc3b4bf0e147826544c6cc`. |
| `git diff --check` | PASS. |
| Independent QA | PASS: no blocking findings. A P3 assertion pinning `KMIPKIT-CLAUSE-SPEC-8.6-003.requirement_ids` was removed; follow-up QA confirmed the stable server-only check and direct oracle exclusion remain. The full generator module passed again after this test cleanup (59 tests, 1 skipped). |
| Refactor/evidence commit | PASS: DCO-signed commit `98f8c385`; records source review, TDD/full-suite evidence, QA disposition, and progress. |
| Exact-head GitHub CI | PASS: run `38753179622` on `98f8c38532bc13879e350a7cc646b053d5f0510b`; 37/37 jobs successful, zero failures, zero skipped. |

The parent `KMIPKIT-PROF-COND-5.1.1-005` remains null / `profile_conditional` / `pending_t002_crosswalk`. T002 and PR #90 remain open.
