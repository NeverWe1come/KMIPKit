# KMIPKIT-0048 Implementation Evidence

## Red

Red tests are committed separately in `3763a38a8` (`test(0048): add red
contracts for typed attributes`) after the specification and API contract
commits.

- `python -B -m unittest tools.normative_catalog.tests.test_generate_typed_attributes -v`
  failed as expected because `collect_typed_attributes` and
  `render_typed_rust` do not exist in the current catalog generator.
- `cargo test --offline -p kmipkit-protocol --lib typed_attribute_tests --all-features`
  failed as expected because the typed API module, markers, collection methods,
  and Current/New wrapper methods do not exist yet.

## Green

Green implementation is committed separately after Red in
`test(0048): implement typed attributes`. It extends the existing catalog
generator, renders 62 direct typed markers, preserves exact source multiplicity
text including conditional rules, and adds sealed owned and borrowed Rust
wrappers for `AttributeSet`, `CurrentAttribute`, and `NewAttribute`. Generic
Items, repeats, and nested Structure contents remain available and ordered.

- `python -B -m unittest tools.normative_catalog.tests.test_generate_typed_attributes tools.normative_catalog.tests.test_generate_attribute_types -v`
  passed (11 tests).
- `cargo test --offline -p kmipkit-protocol --lib typed_attribute_tests --all-features`
  passed (9 tests).
- `cargo test --offline -p kmipkit-protocol --all-features` passed (1,267 unit
  tests, all integration tests, and 3 doctests).
- `cargo fmt --all --check` passed.
- `python tools/normative_catalog/generate_attribute_types.py --check` passed
  for both checked-in generated outputs.

The local result covers the Windows host only. Workspace coverage and the
remaining manifest, documentation, platform, and security controls are not yet
closed by this Green commit.

## Refactor

Pending implementation.

## Verification and limitations

No generated files were edited manually. Local verification beyond the Red
commands and the final aggregate coverage/platform controls remain pending.
The feature is stacked on `feature/KMIPKIT-0047-named-values`; no PR has been
opened for 0048 yet.
