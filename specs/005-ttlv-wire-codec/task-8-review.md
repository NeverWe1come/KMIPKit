# T008 Independent Static Review

## Scope and verdict

- Reviewed test commit: `06fcdf8f193e3608f51bfa99541ff316d2cefa10`.
- First parent: `14dc27a553f6e120c66bcb8d980c07a4afb3c5e4`.
- Reviewed task report: `task-8-report.md`, added in `f15c365ab55b94da826a94c4cc11873996e6dbf0`.
- Verdict: **PASS** for T008 spec compliance and task quality.
- This was a read-only static review. I did not run tests or other checks; the root agent independently ran the reported formatting, lint, check, test, documentation, and diff checks. T008 remains unchecked in `tasks.md`.

## Review findings

No actionable findings.

- The T008 diff changes only `crates/kmipkit-client/src/wire_encoder.rs`; every addition is inside its `#[cfg(test)]` module. It adds no production code, public API, dependency, manifest or lockfile change, or client callsite.
- The local xorshift generator uses the fixed seed `0x6d69_706b_6974_0005`. It creates 88 roots using `sample % 11`, giving eight roots for each represented Item Type. Fixtures are bounded to four Structure levels, two to four children per Structure, and payload lengths no greater than 12 octets. Big Integers are generated with lengths 1–12, so the generator excludes empty values. Text is built from valid UTF-8 characters and capped by UTF-8 byte length.
- Structures are generated with repeated allocated Tags. Sibling Item Types are distinct, so a swap changes the compared sequence and cannot pass merely because siblings share a Tag. Nested Structure order is compared recursively. The comparator checks Tag, Item Type, scalar or payload value, child count, and each child position; it does not claim schema-defined field order.
- The generated test invokes the private `encode_item` writer and decodes its actual bytes with public `kmipkit_ttlv::codec::decode`. The expected model is recursively copied from the input, with only Big Integer normalization applied.
- Big Integer normalization prepends the minimum count of `0x00` or `0xff` bytes, selected from the first octet's sign bit, to reach a length divisible by eight. Already aligned octets are copied exactly. Dedicated vectors assert the expected bytes after a positive unaligned value, a negative unaligned value, and an aligned value round-trip through the decoder.
- Inline attribution cites OASIS KMIP Specification v2.1 §§10.1.1–10.1.5 and §11.23 with the applicable `KMIPKIT-0005-NR-*` identifiers. The Big Integer vectors separately cite §§10.1.2–10.1.3 and their catalog IDs. Comments distinguish generic Structure-order policy under FR-003 and the empty Big Integer project rule under FR-002 from OASIS requirements; no schema-order or OASIS empty-value claim is made.

## Report accuracy and limits

`task-8-report.md` accurately describes the single-file test-only scope, fixed seed, fixture bounds, coverage, comparator, Big Integer normalization, attribution, and task status. Static source inspection confirms 34 unit test annotations in `wire_encoder.rs` and six integration test annotations in `crates/kmipkit-client/tests/error_contract.rs`, matching the reported totals. The report's command results were not independently reproduced by this reviewer; root reports that it ran the checks.

No implementation or test behavior issue blocks T009 or T010. T008 remains unchecked pending task-owner updates.
