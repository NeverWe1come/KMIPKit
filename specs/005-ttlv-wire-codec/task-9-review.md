# T009 Independent Static Review

## Scope and verdict

- Reviewed Red commit: `b3bd550a8360a2fbe69a5cbf82072d8c707607e0`.
- First parent: `17ce945c16b37f5935734c518f47aaccd00fa614`.
- Reviewed task report: `task-9-report.md` from the same commit.
- Verdict: **PASS** for T009 Red scope and task quality.
- This review was static; I did not run tests. The root agent independently ran `cargo test -p kmipkit-ttlv codec::limits_tests::` and confirmed two controls passed and nine tests failed at behavioral assertions, with no compile or panic failures. T009 remains unchecked in `tasks.md`.

## Review findings

No actionable findings.

- The changes add `limits_tests.rs`, wire it only under `#[cfg(test)]` in `codec/mod.rs`, and add the T009 report. No production decoder/API, client code, manifest, dependency, or FR-013 behavior is changed.
- The private suite contains 11 tests: a default/explicit getter-equivalent control; default byte, Structure-depth, and item-count exact/one-over cases; configured byte, depth, and count exact/one-over cases; a U32 Item Length header/truncation control; checked-end overflow; and two pre-allocation rejection cases. The two passing controls and nine expected behavioral failures match the task and report.
- The limit stubs expose the existing `DecodeError` and `DecodeErrorKind` types and call the existing public `decode` function; the Red cases do not depend on absent production symbols. Test `expect` calls are limited to bounded fixture conversions and preconditions. Expected failures use behavioral assertions.
- The U32 test uses an eight-byte synthetic header, checks big-endian parsing of `u32::MAX`, and verifies truncation without materializing the declared payload. The arithmetic test exercises a synthetic `usize` overflow. Default one-over values are passed as scalars; configured wire fixtures are bounded to 24 bytes.
- The test-only observer uses `Cell` counters and does not allocate. The Red wrapper deliberately records would-be reservation/copy activity before delegating, so the zero-activity assertions fail until T010 replaces the stub with the production limits path and appropriate test observation. This is consistent with the requested intentional Red behavior.
- Project limit requirements are attributed to KMIPKIT-0005-FR-007; checked length, bounds, and pre-allocation behavior to FR-008. The U32 header case cites OASIS KMIP Specification v2.1 §10.1.3 and KMIPKIT-0005-NR-004. The cases do not introduce FR-013 or claim it is normative.

## Report accuracy and limits

`task-9-report.md` accurately records the three changed paths, 11 tests, the two controls and nine behavioral failures, bounded fixtures, attribution, and unchecked status. The root's independent Red run confirms the reported result. I did not independently run the compile-only, formatting, clippy, or check commands listed in the report.

T009 remains unchecked pending task-owner updates.
