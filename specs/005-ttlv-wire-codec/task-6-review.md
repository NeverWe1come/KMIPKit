# T006 Independent Re-review

## Scope and verdict

- Reviewed Green commit: `6419a4304a05e3607e0c088d9d64eb9080f04817`.
- Reviewed follow-up: `71534fda7f74a14e1ecdb12187ed26b9ce51e54b` (first parent: T006 Green commit).
- Verdict: **PASS** for the reviewed T006 implementation and documentation follow-up.
- This re-review was static. No build or test command was run; verification claims in `task-5-report.md` remain independently unverified here.

## Prior findings

1. **Resolved — crate-level rustdoc.** `crates/kmipkit-ttlv/src/lib.rs:1-14` now describes the bounded public decoder, framing/type-length/padding checks, and default limits. It says the crate does not encode TTLV and distinguishes generic wire validity from model construction and operation-schema validity.
2. **Resolved — normative/project attribution.** `crates/kmipkit-ttlv/tests/codec_negative.rs:70-89` separates invalid UTF-8 from unsupported Item Type rejection. The former cites OASIS §10.1.2 / `NR-002`; the latter cites §11.23 for the defined types and explicitly attributes rejection to project requirement `FR-005`, not an OASIS decoder requirement.
3. **Resolved — limit wording.** `crates/kmipkit-ttlv/src/codec/decoder.rs:58,71-72` no longer calls byte limits “configured”; depth and item-count diagnostics say “default limit,” consistent with T006 exposing only `decode` and T010 owning configurable limits.

## Regression and remaining status

The follow-up changes only crate rustdoc, static error wording, test separation/citations, and T006 report counts. No parser logic changed, and no regression was found in the reviewed diff. The T005 private adapter still calls production `decode`; the split raises public negative tests from 15 to 16 and the report from 81 to 82 total tests, consistent with the added test function. `codec_api.rs` remains the public API import/use test. The T006 section in `task-5-report.md` still labels its evidence provisional pending independent review; this re-review supplies that review evidence, but does not alter that report or the unchecked T006 task line.

No T010 configurable-limits API, client API, or T007 refactor was added by the follow-up. T006 remains unchecked in the reviewed commit.

