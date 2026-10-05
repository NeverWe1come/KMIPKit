# T005 Red Harness Report

## Scope

T005 adds only `#[cfg(test)] mod codec_red_tests;` in
`crates/kmipkit-ttlv/src/lib.rs` and the private test module
`crates/kmipkit-ttlv/src/codec_red_tests.rs`. Its private `decode_candidate`
seam returns an intentionally incomplete error. All test expectations call
that seam directly. There are no production decoder symbols, public codec
imports, external integration tests, OASIS source changes, catalog changes, or
generated-file changes in this Red step.

The T005 brief was not present in the worktree. The approved task line,
specification, traceability table, ADR-0011, and existing Tag/value APIs were
reviewed before editing.

## Baseline

- Branch: `feature/KMIPKIT-0005-ttlv-wire-codec`
- Baseline commit: `ff1dc88dc0d5211410f2157181215f15967446f4`
- Command: `cargo test -p kmipkit-ttlv`
- Result: passed, 32 tests total: 31 unit/integration tests and one doc test.

## Red Evidence

- Command: `cargo test -p kmipkit-ttlv`
- Result: test code compiled without unresolved symbols or compile errors;
  33 decoder Red tests ran and failed their behavior assertions (0 passed,
  33 failed). Each assertion reports the expected decoded value or expected
  error category; the stub currently returns only `CandidateError::Incomplete`.
  Cargo stops after the failing library test binary, so the existing external
  integration suites do not run in this Red invocation.
- Command: `cargo fmt --all --check` — passed.
- Command: `cargo clippy -p kmipkit-ttlv --all-targets --all-features -- -D warnings`
  — passed. Static analysis passes; this command does not execute the Red tests.
- Command: `cargo check -p kmipkit-ttlv --all-features` — passed.
- Command: `git diff --check` — passed.

## Vector and Requirement Coverage

Each OASIS-derived test has an inline citation to the exact OASIS KMIP
Specification v2.1 section and stable requirement ID:

- All eleven Item Types have valid single-item vectors: empty Structure;
  signed Integer `0x80000000`; Long Integer `0x0102030405060708`; an aligned
  eight-octet Big Integer ending in `0x2A`; Enumeration `0xFFFFFFFF`; Boolean
  True; UTF-8 Text String `é` (`C3 A9`); Byte String `00 80 FF`; Date Time `-1`;
  Interval `0xFFFFFFFF`; and Date Time Extended `i64::MIN`. Type/value-form
  vectors cite OASIS §§10.1.2 and 11.23, `KMIPKIT-0005-NR-002`; Item Length
  cases also cite §10.1.3, `KMIPKIT-0005-NR-004`; Big Integer cites §10.1.2,
  `KMIPKIT-0005-NR-003`; padding cases cite §10.1.5,
  `KMIPKIT-0005-NR-005`.
- Header vectors exercise Tag bytes and every incomplete header prefix
  (lengths 0 through 7), citing §10.1.1, `KMIPKIT-0005-NR-001`, and
  §10.1.3, `KMIPKIT-0005-NR-004`.
- Additional value checks preserve an unknown Enumeration as raw `u32` bits
  and an Integer's high mask bits; these are explicitly KMIPKit
  `KMIPKIT-0005-FR-005` preservation requirements, not OASIS operation-level
  enumeration semantics.
- Structure vectors check nested boundaries, child order, repeated Tags,
  children crossing a parent boundary, and declared extents beyond available
  input. OASIS-derived Structure representation and length references are
  §§10.1.2 and 10.1.5, `KMIPKIT-0005-NR-002` and
  `KMIPKIT-0005-NR-005`; generic order preservation and complete-boundary
  rejection are `KMIPKIT-0005-FR-005` and `KMIPKIT-0005-FR-006`.
- Tag vectors cover an assigned Tag and accepted extension Tag, citing
  Chapter 11 introduction and §11.56, `KMIPKIT-0005-NR-006`. Received
  Reserved-tag rejection is explicitly the project policy in `FR-010` and
  accepted ADR-0011; it is not described as an OASIS decoder requirement.
- Negative vectors cover trailing bytes (`FR-004`); truncated values
  (`NR-004`, `FR-006`); invalid lengths for Integer, Long Integer,
  Enumeration, Boolean, Date Time, Interval, and Date Time Extended
  (`NR-002`, `NR-004`); a nonempty Big Integer length not divisible by eight
  (`NR-003`); empty Big Integer (`FR-006`, project validity rule only);
  invalid UTF-8 (`NR-002`); invalid Boolean (`NR-002`); and an unsupported
  Item Type (`FR-005`, project model boundary).
- Padding vectors check exactly four following octets for Integer,
  Enumeration, and Interval, and minimum-to-eight-byte-boundary padding for
  Text String and Byte String (`NR-005`). They reject short and excess extents
  while explicitly accepting nonzero padding values. OASIS §10.1.5 does not
  assign padding-byte contents, so the tests impose no zero-byte requirement.
- Checked available-input vectors include a declared `u32::MAX` Byte String
  length with no body (without allocating that body), a declared 9-byte value
  with only 8 bytes present, and Structure child/parent span mismatches
  (`NR-004`, `FR-006`, `FR-008`).

All vectors are bounded small fixtures. T006 has not started; the seam has not
been promoted and no production decoder is present in this commit.

## Independent review

- Reviewed commit: `0937633685d44aaa921e13aec3449d6cce319f40`.
- Verdict: **PASS** for spec compliance and task quality; no blocking findings.
- The reviewer confirmed the test-only boundary, all 33 behavior-only Red
  cases and eleven type vectors, fixture bounds, and report counts. The assigned
  Tag `0x420173` is Asynchronous Request under OASIS KMIP 2.1 §11.56; `0x541234`
  is in the Extension range. Reserved-tag rejection remains identified as
  KMIPKit policy. Padding extent checks do not constrain padding octet values,
  consistent with §10.1.5.
- The reviewer performed static review only and did not independently rerun
  builds or tests. The Red command results above were observed in the task run.
