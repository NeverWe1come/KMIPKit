# T004 Refactor Report: TTLV Tag Generator Documentation

## Status

T004 is complete at baseline `c5fc513`. The generator already followed the
repository's validation and I/O conventions, so this refactor makes the
requested module-documentation change without restructuring working code or
changing behavior.

## Changes

- `tools/normative_catalog/generate_ttlv_tags.py`: documented the repository-
  relative input `specification/catalog/kmip-2.1.json`, output
  `crates/kmipkit-ttlv/src/generated/tag_allocations.rs`, `--repo-root` default,
  atomic `--write` behavior, and read-only `--check` behavior.
- `.superpowers/sdd/plan/task-4-report.md`: recorded the conventions reviewed,
  verification output, and self-review.

No tests, catalog files, OASIS sources, or generated Rust output were changed.

## Existing conventions verified and preserved

- The CLI safely reads the catalog under `MAX_BYTES`, then calls
  `load_validated_catalog` before rendering.
- `--write` uses shared `atomic_write_bytes` for the generated output.
- `--check` uses `safe_read_bytes` and only compares bytes; it does not write or
  create output files.
- CLI failures use fixed, value-free diagnostics and do not print exception
  text containing catalog data.
- Exact tag allocations and ranges stay represented separately, numerically
  sorted, with no runtime precedence logic.

## Verification

Command:

```text
python -B -m unittest tools.normative_catalog.tests.test_generate_ttlv_tags -v
```

Exact output:

```text
test_catalog_validation_error_does_not_echo_catalog_values (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationGeneratorCliTests.test_catalog_validation_error_does_not_echo_catalog_values) ... ok
test_check_reports_missing_output_without_creating_it (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationGeneratorCliTests.test_check_reports_missing_output_without_creating_it) ... ok
test_check_reports_stale_output_without_rewriting_it (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationGeneratorCliTests.test_check_reports_stale_output_without_rewriting_it) ... ok
test_write_is_idempotent_and_clean_check_is_read_only (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationGeneratorCliTests.test_write_is_idempotent_and_clean_check_is_read_only) ... ok
test_keeps_exact_assignments_visible_alongside_overlapping_reserved_range (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_keeps_exact_assignments_visible_alongside_overlapping_reserved_range) ... ok
test_numeric_order_and_rendering_are_stable_when_catalog_rows_are_reordered (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_numeric_order_and_rendering_are_stable_when_catalog_rows_are_reordered) ... ok
test_rejects_aggregate_range_endpoints_outside_the_24_bit_range (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_rejects_aggregate_range_endpoints_outside_the_24_bit_range) ... ok
test_rejects_duplicate_aggregate_ranges (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_rejects_duplicate_aggregate_ranges) ... ok
test_rejects_duplicate_numeric_exact_tag_values (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_rejects_duplicate_numeric_exact_tag_values) ... ok
test_rejects_exact_tag_values_outside_the_24_bit_range (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_rejects_exact_tag_values_outside_the_24_bit_range) ... ok
test_rejects_malformed_aggregate_range (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_rejects_malformed_aggregate_range) ... ok
test_rejects_malformed_exact_tag_value (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_rejects_malformed_exact_tag_value) ... ok
test_renders_assigned_and_individually_reserved_exact_entries (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_renders_assigned_and_individually_reserved_exact_entries) ... ok
test_renders_every_exact_catalog_tag_and_range (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_renders_every_exact_catalog_tag_and_range) ... ok
test_renders_extension_range_as_numeric_endpoints_and_classification (tools.normative_catalog.tests.test_generate_ttlv_tags.TagAllocationRenderingTests.test_renders_extension_range_as_numeric_endpoints_and_classification) ... ok

----------------------------------------------------------------------
Ran 15 tests in 10.528s

OK
```

`git diff --check` also completed successfully.

## Self-review

The change is documentation-only. The output path and input path are exact,
the mode descriptions match their implementations, and the descriptions
surface existing shared safe-I/O and atomic-write behavior. The focused tests
confirm rendering stability, repeated-write byte identity, and clean-check
read-only behavior. No outstanding concern found.
