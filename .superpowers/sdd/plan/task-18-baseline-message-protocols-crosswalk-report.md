# Task 18 report — Baseline Message Protocols crosswalk

## Result

Mapped Baseline Client Profiles v2.1 §5.1.1 item 6's Authentication, Transport, and TTLV subitems in source order. Each is a protocol capability with `element_id: null`. The parent condition remains null / `profile_conditional` / `pending_t002_crosswalk`; no profile-support claim is made.

The mapping includes only reviewed client-eligible IDs. It leaves Profiles §3-002/-003/-004 and generic TTLV encoding candidate IDs out of the client requirement list where actor attribution is unresolved; preserves server-only source clauses without assigning them to the client; includes the explicit client half of HTTPS §3.2.3 while retaining its conditional scope and unspecified direction; and excludes TLS 1.2 MAY, XML/JSON, and §5.3.1-011 server-to-client behavior from 1.0.

## TDD and review

- Initial Red: `59672b81`; expected failure because item 6 had no mappings.
- Green: `f3787dc4`; focused regression passed.
- Correction Red: `48284913`; expected failure because the direct source and source review identify the client half of HTTPS §3.2.3.
- Correction Green: `5a421128`; focused regression passed with `KMIPKIT-REQ-PROF-3.2.3-001`.
- Independent QA found the §3.2.3 omission, then confirmed the correction. A follow-up request also added assertions that source review and catalog scope remain `profile_conditional`; scoped re-review found no remaining issue.
- Refactor/evidence commit is pending.

## Verification

- Focused crosswalk regression: PASS, 1/1 after Green and again after the final scope assertions.
- Full normative suite: PASS, 566 tests / 9 skipped, 314.858 seconds.
- Complete profile generator test module after final assertion: PASS, 60 tests / 1 skipped (existing Windows symlink creation limitation).
- Catalog validation: PASS, 4 sources / 1,411 clauses / 4,117 records.
- Generated profile metadata check: PASS.
- Generated coverage report check: PASS.
- Pinned OASIS source immutability against `b3ead993306d757025be237a1a2e23fa5ea6f122`: PASS.
- `git diff --check`: PASS.
- Exact-head GitHub CI on code-bearing Refactor head `d60a1beff55fac77f05e446d0c4870f185804bb9`: PASS, run `38760212335`, 34 successful jobs / 0 failures / 3 schedule-only skips. CI for the subsequent evidence-only head is tracked in the PR description.

No production behavior, catalog ownership, generated file, or pinned upstream source changed.
