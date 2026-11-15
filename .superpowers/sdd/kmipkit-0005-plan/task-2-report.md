# T002 report — encoder Red harness

**Status**: Complete. Red-only test harness committed; no production encoder,
production dependency, `Client::execute`, permit, or production callsite was
added.

## Prerequisite gate

- Read `specs/005-ttlv-wire-codec/approval-record.md`; the delegated approval
  covers T002's reviewed test-only dependency selections and preserves the
  existing feature boundaries.
- Read `specs/005-ttlv-wire-codec/dependency-review.md`; the independent review
  and maintainer dispositions are complete. ADR-0011 is accepted as a
  KMIPKit policy, not an OASIS decoder requirement.
- Reconfirmed the client test graph before and after the manifest edit. The
  final locked tree uses `kmipkit-ttlv` 0.1.0, `zeroize` 1.9.0 with `alloc`,
  `static_assertions` 1.1.0 with no optional feature, and `serde` 1.0.228 with
  `std`. Cargo added only the reviewed `static_assertions` package to the lock
  file; the other packages were already present in the workspace graph.

## Red harness

- Added the reviewed direct development dependencies and `#[cfg(test)]`
  module wiring. `wire_encoder.rs` contains only private tests, fixtures,
  observers, and deliberately incomplete stubs for T003 to replace.
- Added exact vectors for all eleven Item Types, including signed and unsigned
  big-endian values; Big Integer sign extension and empty-value rejection;
  four-byte, Text String, and Byte String padding; and repeated-tag Structure
  child order. Generic child-order preservation is attributed to FR-003 and is
  explicitly not presented as OASIS schema field-order conformance.
- Added bounded synthetic planning cases for default and per-call byte, depth,
  and element limits at exact and one-over edges; lowered and raised byte/count
  limits; the hard depth ceiling; and U32 maximum/one-over Item Length. No
  large payloads, outputs, or trees are allocated.
- Added a test-only payload-copy observer at the shared helper and a
  test-only Drop observer that checks initialized bytes before `Vec`
  deallocation. Both observers are safe Rust; the owner stub intentionally
  omits zeroization so the assertion fails. Negative trait assertions ensure
  the private owner shape does not expose the prohibited traits.
- Normative exact vectors cite the exact OASIS document/sections and stable
  feature requirement IDs (`KMIPKIT-0005-NR-001` through `NR-006` as
  applicable), plus direct catalog IDs where available. Genuinely
  project-only behavior, such as empty Big Integer rejection, cites only its
  FR. Generic child-order preservation cites FR-003 and does not claim
  schema-defined field-order coverage.

## Verification evidence

Formatting and whitespace checks passed:

```text
cargo fmt --all --check
git diff --check
git diff --cached --check
```

Exact requested Red command:

```text
cargo test -p kmipkit-client
```

The command compiled the client and test harness successfully, then exited 1
with behavioral assertion failures only. Exact test summary:

```text
running 31 tests
test result: FAILED. 3 passed; 28 failed; 0 ignored; 0 measured; 0 filtered out
error: test failed, to rerun pass `-p kmipkit-client --lib`
```

The 28 failing assertions are:

```text
big_integer_padding_is_minimal_and_sign_extended
borrowed_per_call_limits_do_not_share_mutable_state
configured_depth_cannot_raise_the_model_hard_maximum
configured_lowered_byte_limit_accepts_exact_boundary_and_rejects_one_over
configured_lowered_depth_limit_accepts_exact_boundary_and_rejects_one_over
configured_lowered_element_limit_accepts_exact_boundary_and_rejects_one_over
default_byte_limit_accepts_exact_boundary_and_rejects_one_over
default_depth_limit_accepts_64_and_rejects_synthetic_depth_65
default_element_limit_accepts_exact_boundary_and_rejects_100001
encodes_big_integer_golden_vector_with_sign_extension_in_item_length
encodes_boolean_golden_vector_with_exact_eight_byte_value
encodes_byte_string_golden_vector_with_minimum_following_padding
encodes_date_time_extended_golden_vector_with_signed_big_endian_value
encodes_date_time_golden_vector_with_signed_big_endian_value
encodes_empty_structure_golden_vector
encodes_enumeration_golden_vector_with_unsigned_big_endian_value
encodes_integer_golden_vector_with_signed_big_endian_value
encodes_interval_golden_vector_with_unsigned_big_endian_value
encodes_long_integer_golden_vector_with_signed_big_endian_value
encodes_text_string_golden_vector_with_minimum_following_padding
fixed_width_four_byte_values_have_four_following_padding_bytes
item_length_planner_accepts_u32_max_without_allocating
item_length_planner_rejects_u32_max_plus_one_without_allocating
owner_drop_zeroizes_initialized_bytes_before_backing_allocation_deallocation
preflight_rejection_performs_zero_payload_copies
rejects_empty_big_integer_as_a_project_validity_rule
structure_preserves_child_order_and_repeated_tags
text_and_byte_string_padding_is_minimal_at_eight_byte_boundaries
```

There were no compile errors, unresolved symbols, unexpected panics, or
production encoder symbols. The 28 failures are assertion failures against the
intentionally incomplete stubs. Two raised-limit cases pass because the
permissive `check_plan` stub returns `Ok` for every plan. The third passing test,
`preflight_error_does_not_format_payload_bytes`, passes because the stub error
has no payload field. Over-limit and writer/owner cases fail as intended.

## Self-review

- The staged change set contains only the dependency lock/manifest, test-only
  module wiring, the new private Red harness, and this task report.
- The manifest adds no normal dependency. The harness declares no production
  writer, public encoder, callsite, permit, or unsafe block.
- Every fixture is bounded; limit and U32 edges are synthetic. The zeroization
  observer reads only initialized bytes while the allocation is still live.
- The test output shows behavioral assertion failures only, as required for
  the Red commit.

## Commit

This report is included in the DCO-signed Red commit for T002.

T003 and Green implementation are not started by this task.

## Review correction round

- Updated normative exact-vector comments to cite the approved feature NR IDs
  with exact OASIS sections, adding direct catalog IDs where applicable.
  Project-only empty Big Integer rejection cites only FR-002. Generic child
  order remains FR-003, with schema-defined field-order coverage explicitly
  unclaimed. U32 planning cites §10.1.3 / NR-004.
- Corrected the pass explanation: the two raised-limit cases pass because the
  permissive `check_plan` stub accepts all plans; the payload-formatting test
  passes because the fieldless stub error cannot contain payload.
- Updated `dependency-review.md` to record that T002 added only the reviewed
  development-dependency edges and no normal client dependency. Refreshed the
  approval-record digest to `60FE528D34DCB30F8F83BC6176B91E8522B4A6FA9CA1A5875B32697E9F0206B4`.
  The approval-record table check found 16 entries and 0 mismatches.

Commands and results after the review corrections:

```text
cargo fmt --all --check — passed
git diff --check — passed
cargo test -p kmipkit-client — exit 1 as expected for Red
running 31 tests
test result: FAILED. 3 passed; 28 failed; 0 ignored; 0 measured; 0 filtered out
error: test failed, to rerun pass `-p kmipkit-client --lib`
```

The three passing tests are `configured_raised_byte_limit_allows_a_plan_above_default`,
`configured_raised_element_limit_allows_a_plan_above_default`, and
`preflight_error_does_not_format_payload_bytes`. Compilation succeeded; all
28 failures remain behavioral assertions, with no unresolved symbols.

## Independent task review

- Initial review: spec compliance not fully compliant; task quality pass with documentation caveats.
- Fix round 1 addressed all three findings (normative attribution, pass report, dependency status).
- Scoped re-review: all findings ADDRESSED; no new issue; review clean for T002.
