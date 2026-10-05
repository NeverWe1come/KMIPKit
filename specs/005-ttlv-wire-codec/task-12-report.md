# T012 Normative Traceability Report

## Scope and status

This report records the provisional T012 catalog update in commit
`60e2014b6a81a0285b23013278996c45af443009`. T012 remains unchecked pending
independent review. No push or pull request was created.

The catalog now links the T010/T011 codec implementation and executable tests
to the applicable BigInteger padding and Item Length requirements in §10.1.2,
the string/fixed-width padding requirements in §10.1.5, and the Tag prefix
requirement in §11.56. The five rows are
`KMIPKIT-REQ-SPEC-10.1.2-002-001`,
`KMIPKIT-REQ-SPEC-10.1.2-002-002`,
`KMIPKIT-REQ-SPEC-10.1.5-001-001`,
`KMIPKIT-REQ-SPEC-10.1.5-001-002`, and
`KMIPKIT-REQ-SPEC-11.56-001`.

`KMIPKIT-REQ-SPEC-10.1.2-001` remains unassigned with empty implementation and
verification references. Its review note records the gap: generic TTLV
Structure child-order preservation does not establish approved typed-spec
ownership and executable order verification for every applicable KMIP
Structure. No official OASIS Test Case IDs were added; project test references
are explicitly described as project evidence. This report does not claim
complete roadmap traceability.

The generated `coverage-report.md` was regenerated with the pinned report
generator. The catalog report test's out-of-scope fixture was narrowed to keep
the five newly assigned requirements out of its unassigned set. No other
catalog collection, source, identifier, keyword, summary, test-case link,
profile, decision, FR-013 record, `tasks.md`, or approval record was changed.

## Gap audit and test evidence

The pre-edit audit found executable BigInteger sign-extension, minimal-padding,
Item Length, and aligned-octet roundtrip tests in the private writer. It found
executable TextString/ByteString and fixed-width decoding/padding tests, plus
the matching writer padding tests. The writer already selected the assigned
`0x42` prefix, and decoder tests covered extension Tags, but there was no
executable writer vector asserting the extension prefix `0x54`.

Added `encodes_extension_tag_with_oasis_0x54_prefix`, which encodes Tag
`0x0054_1234` and compares its three Tag bytes as `54 12 34`. Its comment cites
OASIS KMIP v2.1 §11.56 and `KMIPKIT-REQ-SPEC-11.56-001`. This characterization
test passed immediately against the existing writer; no functional Red
failure or failing commit is claimed. The existing assigned-Tag golden vector
also cites §11.56 and the same trace ID for its `0x42` prefix.

The catalog unit test's existing out-of-scope expectations were updated to
match the five newly assigned requirement rows. The explicit §10.1.2-001 gap
remains in the generated Unassigned requirements section.

## Verification evidence

All final commands below completed successfully:

- `python -B tools/normative_catalog/validate.py --repo-root .` — valid;
  4 sources, 1,411 clauses, 4,021 records.
- `python -B tools/normative_catalog/report.py --repo-root . --write` — wrote
  the generated coverage report.
- `python -B tools/normative_catalog/report.py --repo-root . --check` — report
  verified.
- `python -B tools/normative_catalog/generate_ttlv_tags.py --repo-root . --check`
  — generated tag allocations verified.
- `python -B tools/normative_catalog/generate_result_values.py --repo-root . --check`
  — generated result values verified.
- `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'`
  — 162 tests passed, 7 skipped.
- `cargo fmt --all --check` — passed.
- `cargo test -p kmipkit-ttlv` — 100 passed, 0 failed, including doc tests.
- `cargo test -p kmipkit-client` — 43 passed, 0 failed. This includes the new
  `encodes_extension_tag_with_oasis_0x54_prefix` vector.
- `cargo test -p kmipkit-client encodes_extension_tag_with_oasis_0x54_prefix`
  — 1 passed, 0 failed.
- `git diff --check` — passed before the implementation/catalog commit.

One concurrent `report.py --check` invocation returned a temporary repository
directory lock error while the catalog suite was running. The command was
rerun sequentially and passed. The first catalog-suite run also exposed stale
out-of-scope assertions for the five assigned rows; after updating that test
expectation, the final complete suite passed as recorded above.

## Review status

Pending independent review. T012 remains unchecked. No push or pull request was
created.
