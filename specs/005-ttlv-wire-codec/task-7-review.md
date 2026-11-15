# T007 Independent Static Review

## Scope and verdict

- Reviewed Refactor commit: `f576a82d98cbd4c121cbef67392121ebad6ce4e2`.
- First parent: `18e8e4851aa9717f5193832019ca45925ac3296a`.
- Verdict: **PASS** for T007 spec compliance and task quality.
- Static review only: no tests, builds, clippy, or documentation commands were run. The root agent will run the verification checks independently.

## Review findings

No actionable findings. The refactor preserves the prior parser's validation sequence, error categories, and offsets:

- `parse_item_header` retains header completeness, tag/type/length validation, and their original offsets.
- `structure_depth_for` retains the same checked depth increment and limit boundary.
- `item_span` uses checked additions for the value and padded item end, with the original length-field error offset.
- `validate_parent_boundary` keeps the same precedence: nested item overrun is `StructureBoundary`, value overrun is `TruncatedValue`, and root padding overrun is `InvalidPaddingExtent`.
- `checked_tag` preserves Reserved versus unallocated Tag mapping. The coordinator still parses the validated value before constructing the public `Item`.

The public error types moved from the private decoder submodule into the public codec facade without changing their public path, variants, getters, or formatting. `DecodeError` retains only its kind and offset; Debug and Display do not receive input bytes or payloads. No unsafe code or API/configuration expansion was introduced. The existing T005 adapter, public API test, and negative test files are unchanged by this commit.

## Report review and limits

`task-7-report.md` accurately describes the two-file refactor, keeps T007 unchecked pending review, and reports 82 tests consistent with the preceding T006 report. It explicitly calls out that `AllocationFailed` and the defensive `ModelConstraint` mapping lack deterministic direct test injection; this is a reasonable limitation for the current design and does not warrant production test hooks. Its verification commands and results were not independently reproduced in this review.

T006 is outside this review except for checking its referenced report/re-review status. The report records T006 PASS, and its follow-up introduced no behavior changes. T007 remains unchecked in `tasks.md`.
