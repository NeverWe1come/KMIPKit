# KMIPKIT-0010 T002 — Protocol Version crosswalk report

Date: 2026-10-15
Branch: `feature/KMIPKIT-0010-profile-conformance-t001-t002`
Workspace: `C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit`
BASE before this slice: `779bfdbcfe39d2b4562a4e7571111485b0032de0`

## Scope and source decision

This slice adds the ninth source-ordered mapping under Baseline Client Profiles v2.1 §5.1.1 item 5: `Protocol Version` to `KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION`. The independent oracle row has no direct requirement IDs, cites `KMIPKIT-CLAUSE-SPEC-9.16-001` as its direct source provenance, and marks the structural map reviewed. The field's Protocol Version Major and Protocol Version Minor children are retained as structural provenance. The parent condition remains null / `profile_conditional` / `pending_t002_crosswalk`.

The pinned Specification v2.1 §8.2 Request Header and §8.5 Response Header each show `Protocol Version / Yes` (lines 42592–42601 and 43128–43135). The catalog preserves separate client-to-server and server-to-client placement fields and requiredness. Those table rows have no direct field-specific requirement IDs. The existing §8.2-001 and §8.5-001 clauses are generic header-row `REQUIRED` extractor hits marked `non_applicable`, so they are not cited as Protocol Version field clauses.

Specification §9.16 defines the two-part Major/Minor structure and states same-major backward compatibility as a client/server SHALL and different-major backward compatibility as OPTIONAL (lines 45084–45143). The catalog marks `KMIPKIT-CLAUSE-SPEC-9.16-001` `source_discrepancy`, gives it no requirement IDs, and leaves actor/direction/scope unclear because of the KMIP 2.1-only 1.x boundary. The mapping preserves this clause as unresolved provenance; it does not resolve or claim the compatibility behavior. The Discover Versions request-list requirements are distinct operation rules and are not attached to this structural field.

No catalog ownership, runtime behavior, generated output, or pinned upstream OASIS file changed. The item-5 map now covers nine of 14 values; five remain: Result Reason, Result Status, Server Correlation Value, Time Stamp, and Unique Batch Item ID. T002 remains open and this partial crosswalk is not a profile-conformance claim.

## TDD commits and results

### Red — DCO-signed commit `ba2200715ee139324587d4fb55e6f0ebc6037329`

Added the source-backed aggregate and dedicated regressions before changing the independent oracle:

```powershell
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_protocol_version_condition_preserves_structure_and_unresolved_compatibility -v
```

Expected Red: two assertion failures, zero errors. The aggregate expected the ninth row while the oracle had eight; the focused assertion found `Operation` at the final mapped position instead of the absent Protocol Version row. It was an assertion failure proving the missing mapping, not a test-discovery or runtime error.

### Green — DCO-signed commit `f0900989a491b4cf51305dbcd2db96ead155dee6`

Added only the Protocol Version row to the independent profile-closure oracle. The first run exposed overly strict source-span equality because the pinned HTML normalizer retained adjacent style markup; assertions were narrowed to require the field text within the independently pinned row span. The aggregate and dedicated tests then passed 2/2.

### Refactor / evidence

Refactor improved source assertions, checks both placement directions/requiredness and the parent Major/Minor structure, and asserts that §9.16 remains a source discrepancy with no client requirement ID. The full suite exposed a stale Operation test that assumed the most recently mapped item was always last. The focused reproduction returned `Protocol Version` where that test expected `Operation`; the test now selects the mapping by its item and accounts for the updated source order. No normative mapping changed to fix this test.

The first complete run reported 560 tests, 9 skipped, and one failure in the Operation positional assertion. After the test correction, the full suite passed 560 tests with 9 skipped. The aggregate, Operation, and Protocol Version focused tests passed 3/3. Catalog validation, generated profile metadata, coverage report, pinned-source immutability, and `git diff --check` also passed. Two generator/report checks initially returned safe-directory lock errors when started in parallel with the full suite; each passed when rerun serially after the suite, so no source or locking code was changed.

The independent source audit completed read-only and confirmed the mapping and unresolved §9.16 clause classification. Independent QA reviewed base `779bfdbcfe39d2b4562a4e7571111485b0032de0` through head `cd935fe682fbe0ec34860a546c3725d6f026bca0` and reported no findings. Exact-head CI run `38727128105` passed on published head `914df08f490abedc786a8b65c21561925dd6874f` with 34 successful jobs, zero failures, and 3 scheduled-only skips.
