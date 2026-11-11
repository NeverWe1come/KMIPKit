# SDD ledger — plan: /tmp/KMIPKIT-0096-python-binding-plan.md

## Preflight scan

| Task pair / task | Files and interfaces | Finding |
|---|---|---|
| Task 1 existing C/Java contract | Rust builder, `kmipkit_symmetric_key_builder_insert_attribute_item`, Java `attributeItemAt` | The native insertion path and C/Java surfaces exist; Python should call that same ABI and preserve the typed request boundary. |
| Task 1 manifest and generated outputs | Python facade, `specification/api/public-api.json`, generator outputs | The declaration belongs in the shared manifest; outputs must be regenerated, not edited by hand. |
| Task 1 docs and verification | English/Spanish guides, traceability, Python tests | Document the positional semantics and negative cases with the implementation. |

Ruling: expose Python `attribute_item_at(index, item)` and delegate type/tag/index validation to the existing Rust/C builder after Python rejects invalid Python argument types. This matches Java's explicit index-first method and adds no native API surface. Cost if wrong: adjust the Python signature and dependent examples/tests.

Ruling: stack from the pending 0095 branch head so this delivery retains upstream C/Java/Python API dependencies and can proceed before review/merge. Cost if wrong: rebase onto release/1.0.0 and resolve shared-manifest changes.

## Task 1
Complete KMIPKIT-0096 T006 for Python only, with TDD Red/Green/Refactor commits and exact local checks. Base: `3c48b824e` (KMIPKIT-0095 T009 head).

## Implementation findings and rulings

- FR-003 Red tests showed that the shared C insertion/build ABI records
  validation failures with delivery `NONE` through `record_ffi_error`.
  Ruling from the parent: preserve `ClientExecutionError` but normalize
  validation delivery to `NOT_SENT` only in the Python fluent insertion/build
  path; leave the global status helper, C ABI, and Java implementation intact.
  Tests assert both `NOT_SENT` and no connection to a real listening socket.
- Cross-binding gap: direct C consumers of last-error metadata still receive
  `NONE` for the same unsent builder validation failures. This is recorded for
  parent tracking and prevents claiming complete cross-language validation
  metadata parity from the Python tests.
- Manifest generation rejected the new declaration because its implemented
  Rust/Java mapping targets were still `blocked`. Promoting the two existing
  Rust `attribute_at` rows and Java `attributeItemAt` row to `declared` with
  approved four-language mappings resolves that graph consistently. This
  does not mark them `verified` or close T005/T006 before exact-head CI.
- Red commit: `e90957edc`; Green commit: `868e5a179`.
  Detailed commands/results and Refactor evidence are in `task-1-report.md`.

## FR-003 review follow-up

The parent rebased this delivery onto 0095 head `5791bcadd` and explicitly
authorized fixing the C/JNI metadata gap across the two existing insertion
and typed-build endpoints. This supersedes the initial Python-only ruling.

- Root cause confirmed in the native ABI: generic `record_ffi_error` resets
  delivery to `NONE`. Insertion and typed-build now mark only their own
  validation failures `NOT_SENT`; other statuses and other ABI operations
  retain their existing metadata.
- JNI's generic `check_status`/`return_handle` translated these failures into
  argument exceptions without delivery metadata. Only the insertion/build
  bridges now use the existing `ClientOperationException` conversion, with
  original status/category/cause and native delivery codes. Java argument
  guards and unrelated JNI status conversion remain unchanged.
- Red: `86716c1ce`; compiled C returned 2 for the delivery mismatch, and
  Java had four expected failures across nine tests.
- Green: focused compiled C, Java/JNI and Python suites pass. Full details
  and final checks are recorded in `task-2-report.md`.
- No new ABI, public symbol, exception class, or manifest declaration is
  required. The original cross-binding metadata gap is resolved.
