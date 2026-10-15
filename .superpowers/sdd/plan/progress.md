# SDD ledger — plan: specs/010-profile-conformance/plan.md

## Current authoritative state (refreshed 2026-10-15)
- Worktree: C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit; branch feature/KMIPKIT-0010-profile-conformance-t001-t002; clean at start of this continuation.
- PR #90 is open/draft at remote head 0b1b6e43b96d770a9b8fef92d3cf0b18cbb5dd7a, base release/1.0.0. The worktree branch is clean at that pushed head.
- CI run 38682221194 completed successfully for current head 0b1b6e43: 34 successful jobs, zero failures, and 3 schedule-only skips. Runs 38677798125 and 38675881824 passed only on their preceding heads.
- T001 is complete. T002 is in progress. T003 and later tasks remain incomplete overall.
- The §5.1.1 profile-condition inventory covers eight conditions and 30 nested values; each parent mapping remains pending until its nested atoms are crosswalked. Query under KMIPKIT-PROF-COND-5.1.1-004 is mapped and independently approved. Get, Get Attributes, and Locate are mapped locally; independent review is pending. The parent remains null/pending.
- Existing TLS §3.1.2 correction, §6.13/§6.14 normative inclusion resolutions, Query mapping, and Baseline §5.1.1 item 4 crosswalk are pushed in #90. Local normative tests passed 551 tests with 9 skips; current-head CI passed.
- PR #104 (#0043 shared client mapping) remains open/draft at head a6696836e2eec54f9ee9781b0ccb5b484ad6e4b2, base release/1.0.0. Its T036 profile-validation mapping is explicitly gated on approval and closure of KMIPKIT-0010. The PRs also share seven changed paths, so integration requires updating and revalidating the later branch. Keep this task focused on #90.
## Scoped pre-flight conflict scan
| Tasks | Shared artifact or interface | Producer/consumer relationship | Finding and ruling |
|---|---|---|---|
| T002 / T003 | specification/catalog/kmip-2.1.json; profile-closure-v2.1.json; generated Rust/report | T002 audits exact source mappings and applicability; T003 uses the independent oracle to validate closure and generate metadata | No contradictory requirement. Source adjudication must lead; only source-reviewed profile applicability enters closures. Keep unresolved cases explicitly excluded and open. |
| T005 / T006 / T007 / T008 | catalog, generate_profiles.py, profiles_generated.rs, generator tests | T005 defines data/generator; T006 consumes it; T007 verifies Green; T008 refactors determinism | Ordered dependency, no scope conflict. Generated output must remain tool-produced. |
| T009 / T010 / T013 | profile.rs, client execute/preflight API and tests | T009 specifies protocol validation; T010/T013 integrate preflight and delivery-state behavior | Keep protocol rule contracts authoritative; preserve `NotSent` and response-started semantics. Existing tasks explicitly separate test, implementation, and refactor stages. |

## Current task
Task T002: continue auditing the remaining profile applicability, source ownership, and cross-source test references against the pinned OASIS copies. Do not claim profile conformance. Do not invent owners; use approved feature scope and catalog traceability, then update documentation and evidence in this branch.

## Fresh current-state verification (2026-10-14)
- PR #90 head remains `02049fecf352e1cc59b51c6fc1304274fd2c7bc7`; open draft, base `release/1.0.0`.
- Latest run `38631792275`: multiple checks pass, macOS/Windows coverage and Rust plus Script Contracts jobs remain pending. Do not claim CI complete.
- PR #104 remains open draft at `a6696836e2eec54f9ee9781b0ccb5b484ad6e4b2`; its listed run `38589410370` is green, with remaining KMIPKIT-0043 parity tasks.
- Reproduced 203/203 source-section test links on current #90 checkout with pinned SHA-256 checks: 93 Profiles, 110 Test Cases CN01, zero catalog mismatches; ten exact source-internal ID/href filename divergences are listed in `.superpowers/sdd/plan/task-6-brief.md`.
- T002 remains incomplete beyond this evidence tranche.

Ruling: the separate key-format/material ownership audit targets the 25 `profile_conditional` definition rows in §§3.1–3.12, not the 25 client_1_0 request rows in §§6.1.12–6.1.14 — the research inventory uses the same cardinality for distinct row sets, and section plus catalog IDs disambiguate them — the cost if wrong would be assigning a feature owner or profile obligation to unrelated protocol rows.


## Active task (2026-10-15)
- Task T002 slice: Baseline §5.1.1(4) Query item crosswalk; implementer agent /root/t002_query_crosswalk; brief task-2-brief.md; report task-2-report.md; base 9faec8ff68b6d2ab8b7837f523c74f292a5b0468.
- Ruling: represent a reviewed nested-item crosswalk independently from the parent condition status. Query maps only the exact client Query operation and its two client request REQs; the parent remains pending because Get, Get Attributes, and Locate are not yet mapped. This avoids overclaiming closure; cost if wrong is schema rework in the independent oracle/test.


## Verification update (2026-10-15)
- CI run 38675881824 completed successfully on exact head 9faec8ff68b6d2ab8b7837f523c74f292a5b0468: 34 success, 3 schedule-only skips, zero failures. This closes the pending validation for the prior TLS cipher-suite and profile-inclusion corrections only; a new run is required after the active Query crosswalk commits are pushed.


Task 2: Red complete — commit 468aece4. The focused regression failed because the oracle lacks the Query item mapping, as required; parent condition remains null/pending. Implementer is proceeding with Green.


Task 2: Green verified — focused crosswalk 1/1; test_generate_profiles.py 45 tests, 1 platform skip; catalog validation, generated profile check, and coverage-report check pass. Implementer is refactoring duplicated pinned-HTML normalization and recording complete stage evidence; task review still pending.


Task T002 slice — Baseline §5.1.1(4) Query: complete and independently approved. Commits 468aece4 / 737f4fc1 / ee5ac8cf; reviewer approved spec and quality with no findings. Pushed fast-forward to PR #90 source branch and updated PR body. CI run 38675881824 passed on the prior head 9faec8ff; a new run for ee5ac8cf is required/pending.


## Current CI (2026-10-15)
- PR #90 run 38677798125 completed successfully for head ee5ac8cf955c87b0914cf988b456a6e01d5f2a16: 34 successful jobs, zero failures, and 3 schedule-only skips. The green predecessor run 38675881824 was on 9faec8ff and does not validate this head.
- Completed and reviewed T002 slice: Get, Get Attributes, and Locate under Baseline §5.1.1 item 4. Mapping commits 9b3a87c3 / beda5d64 / 5e306502 plus guard-test commits 76a65daa / 85b39653 / 0b1b6e43 are pushed; the second independent review closed its only P2 with no remaining findings. Full normative inventory passed 551 tests/9 skips; focused validator suite passed 276/1 skip; catalog validation (4 sources, 1,411 clauses, 4,117 records), generated profile/report checks, immutable OASIS check, and `git diff --check` passed. Current-head CI 38682221194 is green. Locate §6.1.28-007 keeps its stable ID/literal keyword but is permission/optional because “not REQUIRED” does not impose a client obligation.
- Next T002 slice: Baseline §5.1.1 item 2 `Attributes`; source audit is complete, and an implementer is starting the crosswalk plus catalog reverse-link correction. The §4 attribute cardinality actor/scope remains explicitly unresolved and excluded.


## Latest verification update (2026-10-15, Baseline Attributes T002)
- Attributes crosswalk slice completed and independently reviewed without findings; PR #90 remains draft/WIP because T002 and the overall profile evidence remain incomplete.
- Pushed exact head `3a7a4933cf5a880d4f006cdad0a6f9db92dd60d9` to PR #90. CI run 38685555360 passed: 34 checks successful, zero failed, 3 scheduled-only skipped. Local normative suite: 553 passed / 9 skipped; catalog, generated profile/report checks, and OASIS immutability passed.
- The parent Baseline §5.1.1 condition remains `null` / pending; the §4 actor/cardinality question remains excluded and unresolved. BL-M-4-21 remains fixture evidence only, with no unsupported formal element-case link.
- Resume with the next uncovered §5.1.1 item in T002. Keep #104 implementation paused until #90 closes the 0010 profile gate; #104 T036 remains directly gated on that closure, while shared changed files still require integration/revalidation.

## T002 follow-up (2026-10-15, Baseline Object Attributes)
- PR #90 updated to head `7fdc0b30e69d6ecff41d62f02f5242a351fa46e3`; open draft. CI run 38689918389 is pending.
- Baseline §5.1.1 item 3 has reviewed element mappings for all eight Object Attributes; no standalone client REQs are assigned because relevant §4 source reviews are server-only or unresolved. Parent condition remains pending. DCO Red/Green/Refactor: `8b2dc9b1` / `721262bb` / `7fdc0b30`; independent review found no source/crosswalk defects.
- Local suite: normative catalog 554 passed / 9 skipped; workspace formatting, Clippy and tests, profile/report checks, immutable-source check passed. `cargo llvm-cov` exited 0 but `kmipkit-ttlv` was 91.99% vs required 95%; track as an existing release/merge-readiness gap. Keep #90 WIP.

## Latest CI (2026-10-15, Object Attributes T002)
- Run 38689918389 passed on exact head `7fdc0b30e69d6ecff41d62f02f5242a351fa46e3`: 34 successful checks, zero failures, 3 scheduled skips. PR #90 remains draft.

## Active T002 slice — Baseline §5.1.1(5) Batch Count (2026-10-15)
- Next nested value selected after independent audits: Batch Count at Profiles v2.1 §5.1.1 item 5, pinned line 4582.
- Safe mapping is to `KMIPKIT-ELEM-MESSAGE-FIELD-9-5-BATCH-COUNT` with §9.5-001 client requirements `KMIPKIT-REQ-SPEC-9.5-001-001/-002`; both source reviews establish client direction/scope. Other item-5 values remain pending; in particular, §9.7 Batch Item actor/scope is unresolved.
- Brief: `.superpowers/sdd/plan/task-4-baseline-message-batch-count-brief.md`; implementer `/root/baseline_batch_count_crosswalk`; base `7fdc0b30e69d6ecff41d62f02f5242a351fa46e3`.

## Active T002 slice — Baseline §5.1.1(5) Batch Error Continuation Option (2026-10-15)
- Next value in source order after Batch Count. The §9.6 clause/source review establishes three client-to-server `client_1_0` REQs. The `Stop` default phrase is not an independent uppercase modal; retain the existing reviewed extraction and document its exact source anchor.
- Exclude Check-specific `KMIPKIT-REQ-SPEC-6.1.7-002-002`; Check is not in the Baseline required operation list. Parent condition remains pending.
- Brief `.superpowers/sdd/plan/task-5-batch-error-continuation-crosswalk-brief.md`; implementer `/root/baseline_batch_error_option_crosswalk`; base `303956a887c1066e71663107c952d2b721a8b4ad`.

## Latest T002 progress (2026-10-15, Batch Error Continuation Option)
- Published the independently approved Batch Error Continuation crosswalk to PR #90. Exact remote head: `5a23553815cf0fe229588f41a5434173c362de89`; PR remains draft. Head CI run `38695788703` is in progress. The previous head’s Windows stable failure was the existing TCP loopback DNS fixture binding with `PermissionDenied` OS error 10013; the exact test passed locally on Windows, so root cause as CI-environment transient is not yet confirmed.
- Started the next Baseline §5.1.1 item-5 value, `Batch Item`, after two independent read-only source/catalog audits. Both support an element-only mapping to the shared §9.7 request/response element, preserving empty client requirement IDs because actor/direction/client scope for the standalone `REQUIRED` candidate remain unresolved.
- The pinned §9.7 paragraph points to Section 12, whose pinned heading is Bit Masks; §8.3/Table 396 and §8.6/Table 399 define the request/response structures. This apparent source cross-reference error is being documented without changing upstream files.
- Batch Item TDD: Red commit `35fe5fd0`; the focused regression failed because the mapping row was absent. Green commit `0d62f7f2`; adding only the fixture mapping made the focused test pass. Current Refactor/evidence changes are local and awaiting full normative suite and independent QA.

## Latest T002 progress (2026-10-15, Baseline Batch Item)
- Completed Batch Item crosswalk with reviewed element-only mapping, preserving empty client REQ attribution and the parent condition as null/pending. Corrected the candidate REQ's exact catalog classification (`role=unspecified`; `direction` and `scope_state=unclear`) and recorded the §9.7 Section 12 cross-reference anomaly.
- Red/Green/Refactor evidence is committed and independently reviewed: `35fe5fd099d5425dbb2249e80b66a44f4e301db5` / `0d62f7f2504c5ff49094e9a1802ca792c24eb4d8` / `457174d46bcc7595d355be71609357fe5f7831ac`. Normative suite: 555 passed / 9 skipped. Catalog, generated profile/report, pinned-source immutability, format, Clippy, and workspace tests pass.
- Published exact head `457174d46bcc7595d355be71609357fe5f7831ac` to PR #90, still open/draft. CI run `38699344031` is queued. Prior run `38695788703` failed Java coverage setup when setup-maven got HTTP 404 from Maven Central; direct requests to both the configured Maven Central binary and Apache distribution returned 200 when checked locally.
- Coverage is a known merge-readiness gap: two current workspace runs measured 92.99% and 93.00%; `kmipkit-ttlv` was 91.99% both times, below the 95% requirement. No Rust production code changed in this tranche.
- Next in source order is Baseline §5.1.1 item-5 `Batch Order Option`; independent read-only source and catalog audits are running. Keep item-5 parent pending and do not claim profile conformance.

## Latest T002 progress (2026-10-15, Baseline Batch Order Option)
- Added the fourth source-ordered item-5 mapping, Batch Order Option → §9.8 Boolean request-header element, with no client requirement IDs; ten item-5 values remain unmapped and the parent condition is still null/pending.
- Red/Green mapping commits: `a9b3cbdb` / `ad531f16`. Independent QA requested a complete source locator and exact candidate-ID assertion; correction Red/Green: `d02a074a` / `464b1f40`. Final QA re-review approved with no findings.
- Verification passed: focused regression 1/1; normative suite 555 passed / 9 skipped; catalog validation (4 sources / 1,411 clauses / 4,117 records); generated profile and report checks; immutable OASIS audit against base `457174d4`; `git diff --check`.
- The prior PR #90 head `457174d46bcc7595d355be71609357fe5f7831ac` has green run `38699344031`; that run predates this local tranche. Publish only after the DCO Refactor/evidence commit, then wait for current-head CI. PR #90 stays WIP; the 95% TTLV coverage gate remains open.

## Active T002 slice — Baseline Maximum Response Size (2026-10-15)
- PR #90 head `8f67dcbb63d6a5a45195da5613767ef79ceb9dfa` is published; run `38701390791` is in progress with 26/34 checks completed and no failures observed at the latest query.
- Independent source and catalog audits agree on `KMIPKIT-ELEM-MESSAGE-FIELD-9-12-MAXIMUM-RESPONSE-SIZE`, all three §9.12 client requirement IDs, and `KMIPKIT-CLAUSE-SPEC-9.12-001`. Required-vs-optional behavior: §8.2 says `Required=No`; §9.12 is OPTIONAL, client SHALL handle any declared size, and SHOULD send only for likely-large replies. No default, numeric range, or server overflow behavior is inferred.
- Brief: `.superpowers/sdd/plan/task-8-baseline-maximum-response-size-crosswalk-brief.md`; one implementer `/root/max_response_crosswalk_implementer` is executing TDD on the current PR branch. No push until independent QA and verification.

## Latest T002 progress (2026-10-15, Maximum Response Size)
- Completed Maximum Response Size mapping plus two independently reviewed documentation corrections. Red/Green/evidence commits: `5a0c3a49` / `77f2fb7d` / `6efc0a1c`; policy wording correction `58364eec` / `ce8ee889` / `24549645`; §11.46 citation correction `8eef6ea5` / `d6cdf21c` / `90ff5750`.
- Independent QA approved the corrections with no remaining findings. Exact-head local verification at `90ff57508670de138753c0c7f63423a083b22854`: normative suite 557 passed / 9 skipped; focused correction tests 2 passed; catalog validation, profile metadata check, report check, pinned-source immutability, and diff check passed.
- Pushed fast-forward to PR #90; remote head verified as `90ff5750`. Updated PR description with tranche evidence and open gates. Current-head CI run `38705750330` is pending; hold further tracked edits until it completes.
- Crosswalk currently has five of 14 item-5 mappings. A source audit found the omitted first value, Asynchronous Indicator, at Profiles §5.1.1 lines 4579–4580. It is a client MAY in `client_1_0`, conditional on accepting asynchronous responses, with no reviewed exclusion. Its item-level mapping is missing even though the general requirement crosswalk contains `KMIPKIT-REQ-SPEC-8-003-001`.
- Next task is this Asynchronous Indicator correction; insert it first, yielding six mapped/eight unmapped, while parent `KMIPKIT-PROF-COND-5.1.1-005` remains null/pending. Do not start Message Extension until then. Brief: `.superpowers/sdd/plan/task-9-baseline-asynchronous-indicator-brief.md`.
Task 9: minor (deferred): Rust checks reported non-fatal AWS-LC/NASM probing and an existing C-ABI linker warning; carry output-cleanliness triage to the final branch review.
Task 9: complete (commits 90ff5750..600d8092, review approved with one deferred minor).

## Latest T002 progress (2026-10-15, Baseline Asynchronous Indicator)
- Completed and independently approved the first source-ordered item-5 mapping; six of 14 values are mapped, eight remain, and parent `KMIPKIT-PROF-COND-5.1.1-005` remains null/pending.
- DCO Red/Green/Refactor commits: `fe2d1aa3` / `0480b79b` / `600d8092`. Normative suite 557 passed / 9 skipped; catalog, profile/report generation, immutable-source audit, formatting, Clippy, workspace tests, and diff checks passed.
- Local coverage remains below the TTLV gate: 91.99% versus 95%; workspace 93.00%. QA approved with one minor deferred warning-output observation.
- Published fast-forward to PR #90, now open/draft at `600d809289fbe813a059ea20f01f176fa51e6bdd`; updated the PR body. Current-head CI run `38709213120` is in progress. Next uncovered value in source order is Message Extension; do not implement until current-head CI closes.

## Recovery and active task (2026-10-15)
- The earlier entries above predate the current published work and must not be treated as the live cursor. Reconciled from Git and GitHub: active worktree is C:/Users/ramp1953/.codex/worktrees/0010-profile-refresh/KMIPKit; branch feature/KMIPKIT-0010-profile-conformance-t001-t002; local and PR #90 head 600d809289fbe813a059ea20f01f176fa51e6bdd; worktree was clean.
- PR #90 is open/draft. Exact-head CI run 38709213120 is still in progress at the last check; no new tracked edits until terminal success.
- Six of 14 Baseline §5.1.1 item-5 values are mapped; eight remain. Asynchronous Indicator is complete/reviewed. Next is Message Extension.
- First independent audit completed and found three applicable client requirements (8.3-004, 9.13-001-004, 9.13-001-005), three unresolved sender-side shape candidates (9.13-001-001/-002/-003), generic canonical element KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION, and placement clauses §§8.3/8.6 plus definition §9.13. Second independent mapping audit is running.
- Prepared ignored task brief .superpowers/sdd/plan/task-10-message-extension-brief.md; no tracked files have been changed for this slice.
- Parent condition KMIPKIT-PROF-COND-5.1.1-005 must remain null/pending. Do not claim profile conformance. Known coverage gap: kmipkit-ttlv 91.99% vs 95%; workspace 93.00%.

- CI gate update: GitHub run 38709213120 completed successfully for exact head 600d809289fbe813a059ea20f01f176fa51e6bdd at 2026-10-15T09:05:58Z. This clears the no-edit gate for the next T002 tranche.

## T002 Message Extension rulings (2026-10-15)
- Ruling: Include source clauses §8.3-004, §8.6-004, and §9.13-001 as provenance for request placement, response placement, and generic structure respectively; use one canonical generic §9.13 element ID. This records both client directions while adding no sender requirement from the §8.6 response row. Cost if wrong: the profile mapping would retain one structural source locator that is not needed for its requirement set; the narrow item can be revised before T002 closes.
- Ruling: Defer the existing catalog metadata defect for §8.6-004 to normative-inventory ownership (KMIPKIT-0002): pinned Table 399 is a server-to-client response row, but the catalog currently records role both/direction client_to_server while disposition is server_only. Use §8.6 only as response-placement evidence, do not derive client obligations from its catalog record, and record the correction needed. Cost if wrong: a known catalog direction defect remains in the normative inventory until its owning correction; it must be tracked and verified before claiming complete traceability.
- Message Extension verification update: Red regression failed for missing mapping as intended; Green commit 3aab78df (Red 22828166). Focused regressions pass 2/2; normative catalog suite passes 558 tests / 9 skips (314.878 s). An initial catalog-validation invocation emitted one unreproduced pinned-source read error; after the suite, an instrumented exact-source read and validation passed for 4 sources/1,411 clauses/4,117 records. Root cause not yet established; no validator changes made. Standard CLI rerun and remaining checks still pending.
- Coverage-metric correction: the earlier 91.99% figure was initially misattributed to the entire kmipkit-ttlv crate. The implementer reports that saved llvm-cov JSON gives decoder.rs 91.99%, kmipkit-ttlv aggregate 748/761 = 98.29%, and workspace 30,038/31,491 = 95.39% (region coverage 92.99%). Independent verification of the exact JSON file and command is pending. Correct prior evidence/PR text after confirmation; do not describe a TTLV coverage gate gap based on decoder.rs alone.

## Superseding T002 current status — Baseline Operation (2026-10-15)
- The Message Extension tranche completed at `ab55760a152a5dedf9b45942c6912d1f673fc04f`, with seven of 14 item-5 values mapped. Exact-head CI run `38714535048` passed for that head: 34 successful checks, 3 schedule-only skips. It is not CI for the Operation commits below.
- Verified line coverage from the saved llvm-cov JSON: `kmipkit-ttlv` 748/761 lines (98.29%), workspace 30,038/31,491 (95.39%), and `decoder.rs` 406/419 (96.90%). Distinct region metrics are TTLV 981/1,033 (94.97%) and decoder 517/562 (91.99%). The earlier 93.00% workspace and 91.99% decoder figures were region coverage, not line coverage.
- The Operation verification rerun saved `C:\Users\ramp1953\AppData\Local\Temp\kmipkit-operation-llvm-cov-summary.json`: TTLV remains 748/761 lines (98.29%), workspace is 30,039/31,491 (95.39%), and decoder remains 406/419 (96.90%). Relative to the prior saved JSON, only `kmipkit-transport/src/timeout.rs` changed its covered-line count (472 to 473 of 508), an unexplained measurement variation. Region totals remain TTLV 981/1,033 (94.97%), decoder 517/562 (91.99%), workspace 39,410/42,377 (93.00%).
- Operation is the eighth source-ordered item-5 mapping: `KMIPKIT-ELEM-MESSAGE-FIELD-9-15-OPERATION`, no separate requirement IDs, and placement provenance clauses `KMIPKIT-CLAUSE-SPEC-8.3-001` then `KMIPKIT-CLAUSE-SPEC-8.6-001`. The request row remains required (`Operation / Yes`); response placement is `Yes, if specified in Request Batch Item`. Six later values remain unmapped; parent `KMIPKIT-PROF-COND-5.1.1-005` remains null / `profile_conditional` / `pending_t002_crosswalk`.
- DCO Red/Green/Refactor commits for this slice are `d92c8859`, `5213fd75`, and the Refactor/evidence commit recorded in its task report. The final operation head has not been pushed; exact-head CI has not run for these commits.
- Generic-tag candidate `KMIPKIT-REQ-SPEC-11.56-001` has unresolved actor/direction/scope; the direction for `KMIPKIT-REQ-PROF-6.1-003` conflicts with its source review (`unspecified`). Both remain deferred to KMIPKIT-0002 and are not used to infer behavior. The §8.6-001 Operation response placement is distinct from the §8.6-004 metadata issue recorded for Message Extension. No catalog, runtime, generated artifact, or pinned upstream source changed; this partial map does not claim conformance or close T002.

## Operation QA correction (2026-10-15)
- Independent QA found wording that could obscure the §8.3 request field's `Required=Yes`; evidence now separates the missing independent requirement ID from the field's requiredness. Test asserts request/response directions and `client_1_0` scope, canonical element `both` / `client_1_0`, and corrects its eight-mapping docstring. Explicit IDs for the deferred §11.56 and profile-direction issues are recorded.
- Fresh focused regressions passed 3/3; `git diff --check` passed. Awaiting independent re-review of the correction before publishing to PR #90.

## Superseding T002 status — Baseline Protocol Version (2026-10-15)
- Exact-head CI for prior Operation commit `779bfdbc` passed after a failed first Windows attempt: rerun 2 completed 37/37 jobs successfully on the same SHA. PR #90 was updated through the terminal with both outcomes.
- Independent source audit confirmed Protocol Version is item 9/14 in Profiles §5.1.1 item 5. Its structural map is `KMIPKIT-ELEM-MESSAGE-FIELD-9-16-PROTOCOL-VERSION`; no direct field-specific requirement IDs are attached. §8.2 and §8.5 rows are each `Yes`; `KMIPKIT-CLAUSE-SPEC-9.16-001` remains a source discrepancy and is preserved only as unresolved provenance. Discover Versions operation requirements are not mapped to this field.
- DCO-signed TDD Red and Green commits: `ba2200715ee139324587d4fb55e6f0ebc6037329` / `f0900989a491b4cf51305dbcd2db96ead155dee6`. Focused aggregate and Protocol Version regressions passed 2/2 after the oracle row was added.
- The first full normative run completed with 560 tests, 9 skipped, and one failure. It confirmed the older Operation regression assumed Operation was the last mapping, but Protocol Version now follows it. Updated that test to select Operation by item and include Protocol Version in source order; aggregate, Operation, and Protocol Version focused regressions passed 3/3. The rerun passed all 560 tests with 9 skipped.
- Serial post-suite checks passed: catalog validation (4 sources, 1,411 clauses, 4,117 records), profile generator `--check`, report `--check`, pinned-source immutability at BASE `779bfdbc`, and `git diff --check`. Generator/report calls that overlapped the suite first returned safe-directory lock errors; both passed after serial rerun. No implementation/catalog/generated source changes.
- Independent QA reviewed base `779bfdbcfe39d2b4562a4e7571111485b0032de0` through head `cd935fe682fbe0ec34860a546c3725d6f026bca0` and reported no findings. Published through head `914df08f490abedc786a8b65c21561925dd6874f`; exact-head CI run `38727128105` passed with 34 successful jobs, zero failures, and 3 scheduled-only skips.
- Nine of 14 item-5 values are mapped, five remain, and `KMIPKIT-PROF-COND-5.1.1-005` remains null / `profile_conditional` / `pending_t002_crosswalk`. No profile conformance claim.

## T002 tranche — Baseline Client Server Correlation Value (2026-10-15)
- Independent source audit confirms the twelfth item maps to `KMIPKIT-ELEM-MESSAGE-FIELD-9-10-SERVER-CORRELATION-VALUE` and `KMIPKIT-CLAUSE-SPEC-9.10-001`, with no client requirement IDs. Server obligations stay assigned to the server; do not confuse this field with Asynchronous Correlation Value. Existing KMIPKIT-0007 FR-016 explains why KMIPKit omits it from client requests.
- Red / Green commits: `88898bbf` / `6621e24c`, DCO-signed. Dedicated and aggregate regressions pass 2/2; focused neighboring crosswalk tests passed 6/6.
- Full normative suite passes 563 tests with 9 skipped. Catalog validation, generator/report checks, immutable-source check against `eb6be1ec`, and `git diff --check` pass. Independent QA found no findings.
- Refactor/evidence commit `2f27511c` is published; exact-head CI run `38744365487` passes 37/37 jobs. Crosswalk is 12/14; Time Stamp and Unique Batch Item ID remain. Parent `KMIPKIT-PROF-COND-5.1.1-005` stays pending; no conformance claim.

## T002 tranche — Baseline Client Time Stamp (2026-10-15)
- Source audit maps the thirteenth item to `KMIPKIT-ELEM-MESSAGE-FIELD-9-20-TIME-STAMP`, with client capabilities `KMIPKIT-REQ-SPEC-9.20-001-001` and `...-001-002`; server-required response behavior is not a client send duty. The conditional countdown-timer capability remains deferred. Catalog direction/element-association inconsistencies are assigned to KMIPKIT-0002.
- Red / Green commits: `90ed2006` / `fd9ca838`, DCO-signed. Red produced the expected 2 assertion failures; aggregate, Time Stamp, Operation, Protocol Version, Result Status, and Server Correlation focused regressions pass 6/6 after Green.
- Full normative suite passes 564 tests / 9 skipped; serial catalog validation, generator/report checks, immutable-source check against `0d3926d6`, and `git diff --check` pass. Independent QA found no blocking findings; its P3 note is documented without freezing the deferred KMIPKIT-0002 catalog association defect. Refactor/evidence commit and exact-head CI remain pending. Item-5 mapping is 13/14; only Unique Batch Item ID remains, while the parent condition stays pending. No conformance claim.

## Active T002 tranche — Baseline Client Unique Batch Item ID (2026-10-15)
- The final Baseline Client §5.1.1 item-5 value now has an independently reviewed oracle mapping to `KMIPKIT-ELEM-MESSAGE-FIELD-9-21-UNIQUE-BATCH-ITEM-ID`. It links the client-side multi-item requirement (§8.3), optional request capability (§9.21), and single-item MAY (Profiles §4.1.2), while excluding the server response echo requirement. The structural subcrosswalk is 14/14 locally; parent condition remains pending and T002 is not complete.
- DCO Red / Green commits are `375c391a` / `2e7b8ebc`. The full generator module passed 59 tests with 1 Windows symlink skip. The full normative suite passed 565 tests with 9 skips in 312.640 seconds.
- Serial checks pass: catalog validation (4 sources / 1,411 clauses / 4,117 records), generated profile and coverage-report checks, pinned-source immutability against `ddba006c`, and `git diff --check`.
- Independent QA found no blockers. It flagged a P3 assertion pinning the response placement clause's `requirement_ids`; the assertion was removed, the full generator module passed again (59 tests / 1 skip), and follow-up QA confirmed the observation is resolved.
- DCO-signed Refactor/evidence commit `98f8c385` is published to PR #90. Exact-head CI run `38753179622` passed 37/37 jobs, with zero failures or skips. A follow-up evidence-only commit will record the CI result and updated task report; T002 remains open.
