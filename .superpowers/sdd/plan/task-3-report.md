# T003 Green Report: Deterministic TTLV Tag Metadata Generator

## Status

T003 implementation is complete. The generator uses the validated local
catalog and emits deterministic private Rust allocation metadata. No generated
Rust output was written or edited in this task.

## Prior T002 RED evidence

The preserved RED record is
[`task-2-report.md`](task-2-report.md). It identifies baseline `99f5d07`, the
reviewed test commit `e25f65c`, and the follow-up test commit `11047cb` that
checks the clean `--check` timestamp. The tests were run again at the assigned
T003 baseline (`1729cd8769eeab0c6eb3833a7c188bb2c9e617f6`) before editing:

```text
python -B -m unittest tools.normative_catalog.tests.test_generate_ttlv_tags -v
```

Result: 14 tests ran and 14 failed, as expected while the generator module was
absent. The 11 renderer cases failed the test's module-presence assertion; the
three CLI cases received Python's missing-script result rather than the
specified generator behavior. The earlier report records the full original
RED output and review context.

## Implementation

Added `tools/normative_catalog/generate_ttlv_tags.py`. It:

- Selects every exact catalog tag, validates 24-bit numeric values and
  assigned/reserved classifications, rejects duplicate numeric values, and
  sorts by numeric value.
- Parses all numeric tag ranges, including the source `420XXX` lower endpoint,
  into numeric 24-bit bounds; validates classifications and duplicate or
  reversed ranges; and sorts by numeric endpoints.
- Renders private `TagAllocationKind`, `EXACT_TAG_ALLOCATIONS`, and
  `TAG_ALLOCATION_RANGES` Rust items. Exact entries remain present even when
  an aggregate range overlaps them; no runtime lookup precedence is generated.
- Uses `load_validated_catalog`, the validator's `MAX_BYTES` bound,
  `safe_read_bytes`, and `atomic_write_bytes` for repository I/O.
- Makes `--check` read-only: it safely reads both inputs and returns stale or
  missing-output status without creating, replacing, or touching output.

## GREEN evidence

Focused T003 command:

```text
python -B -m unittest tools.normative_catalog.tests.test_generate_ttlv_tags -v
```

Result:

```text
Ran 14 tests in 9.107s

OK
```

Related catalog-tool verification:

```text
python -B -m unittest tools.normative_catalog.tests.test_validate tools.normative_catalog.tests.test_safe_io tools.normative_catalog.tests.test_generate_result_values -v
```

Result: 96 tests ran, passed, with 5 platform-specific skips (POSIX-only
fixtures and unavailable Windows symlink privileges).

Syntax verification:

```text
python -B -m py_compile tools/normative_catalog/generate_ttlv_tags.py
```

Result: exit status 0.

## Files and self-review

- Added `tools/normative_catalog/generate_ttlv_tags.py`.
- Added this report.
- In the initial T003 Green commit, did not edit tests, catalog inputs,
  upstream OASIS copies, or generated Rust output. Round 1/5 adds one targeted
  CLI regression test, documented below.
- Reviewed the CLI paths: `--write` validates before using the atomic writer;
  `--check` has no write or directory-creation call and uses safe reads for
  missing, stale, and clean output cases.
- Reviewed the rendering behavior: all exact records and all supplied ranges
  are represented independently; source row ordering cannot affect output;
  no public Tag declaration or runtime precedence rule is emitted.

No unresolved conflicts or implementation concerns were found.

## Round 1/5 review follow-up: fixed catalog-validation errors

The reviewer identified that the CLI printed raw `CatalogValidationError`
text, which can contain identifiers and operation names from catalog data.
The CLI now emits fixed categories for catalog validation, repository path
validation, repository I/O, and invalid allocation data. It does not format
exception text in any of these error paths; exit codes remain unchanged.

Added
`TagAllocationGeneratorCliTests.test_catalog_validation_error_does_not_echo_catalog_values`.
It duplicates a validly shaped record ID with a marker value that the validator
includes in its raw exception, then asserts the CLI returns status 2 and emits
only the fixed catalog-validation message.

RED command:

```text
python -B -m unittest tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationGeneratorCliTests.test_catalog_validation_error_does_not_echo_catalog_values -v
```

RED result before the CLI fix:

```text
Ran 1 test in 1.430s

FAILED (failures=1)
```

The assertion showed stderr contained `duplicate stable record identifier:
KMIPKIT-ELEM-UNTRUSTED-CLI-ERROR` instead of the fixed category.

GREEN focused generator command:

```text
python -B -m unittest tools.normative_catalog.tests.test_generate_ttlv_tags -v
```

GREEN result:

```text
Ran 15 tests in 10.574s

OK
```

GREEN malformed-catalog CLI regression command:

```text
python -B -m unittest tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationGeneratorCliTests.test_catalog_validation_error_does_not_echo_catalog_values -v
```

Result:

```text
Ran 1 test in 1.417s

OK
```

The reviewer’s output-parent check remains assigned to T005 and is not part of
this follow-up.
