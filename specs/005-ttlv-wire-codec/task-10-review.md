# T010 Independent Static Review

## Scope and verdict

- Reviewed Green commit: `302b8c60a75e22b9d590f1253c949d5369627f36`.
- First parent: `012fde054396b99e44a5c08f679df5239c9b462b`.
- Reviewed provisional report commit: `df2126692f4dd70e1e9f88851c165d272bdddcfe`.
- Verdict: **PASS** for T010 spec compliance and task quality.
- This review was static. I did not run tests, builds, clippy, rustdoc, or coverage commands. The report records the verification results, but I did not independently reproduce them. T010 remains unchecked in `tasks.md`.

## Review findings

No actionable findings.

- The public `CodecLimits` has private fields and only read-only getters, `new`, and `defaults`; it implements neither `Clone` nor `Copy`, as also covered by the external compile-fail cases. Byte and item-count limits accept any `usize`, including zero; depth accepts zero through 64 and rejects 65. There are no setters, builders, or global mutable limits.
- `decode` delegates to `decode_with_limits` with defaults of 16 MiB, depth 64, and 100,000 Items. The decoder checks complete input length before traversal, counts the root, gives scalar roots depth zero and Structure roots depth one, and checks nested depth and element count before accepting each Item. Checked offset additions, item spans, type-specific lengths, and parent boundaries precede payload construction.
- Decoder-owned payload copies check their bound, call `try_reserve_exact`, and only then extend the reserved vector with input bytes. The test-only observer records reservation and copy activity at those boundaries. Truncated U32 and configured-size rejection cases assert zero reservations and copies. The code documents the approved OD-001 limitation that infallible allocations inside existing model constructors may abort.
- `DecodeError` contains only a safe kind and offset, so Debug and Display cannot disclose payload bytes. `LimitsError` is static and payload-free. The public `decode_with_limits` documentation states the limit semantics and the model-constructor OOM caveat; crate-level and codec docs point to the new API.
- Public integration tests import and call the API. They cover default and configured getters, zero and lowered limits, scalar/Structure depth behavior, constructor depth 64/65, raised byte and item-count limits using bounded valid fixtures, and default rejection. The private limit tests preserve the T009 coverage and exercise production helpers.
- The private client writer now receives `&CodecLimits` directly. Its adapter reads values through getters; the identity observer uses pointer equality across repeated preflights to confirm the same instance is passed without cloning or reconstruction. The writer remains in a private module, and source inspection found no production callsite, `Client::execute`, or permit.
- No manifest or dependency changes were made. The changes add no unsafe code or out-of-scope T011 refactor. Project limit tests cite FR-007/FR-008; the U32 Item Length case retains its OASIS KMIP Specification v2.1 §10.1.3 / `KMIPKIT-0005-NR-004` attribution. Final catalog traceability remains assigned to T012.

## Report accuracy and limits

`task-10-report.md` accurately describes the changed scope, public API, decoder allocation behavior, writer identity test, and absence of a production callsite. It lists formatting, Clippy, check, test, rustdoc, and diff checks with results; those commands and reported totals were not independently reproduced in this review. No deterministic allocation-failure injection is present, so the `try_reserve_exact` failure branch was reviewed statically only.

T010 remains unchecked pending task-owner updates.
