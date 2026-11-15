# Public managed-object schema evidence (2026-11-09)

This evidence closes the public schema-identity gap for PGP Key, Secret Data,
and Split Key in KMIPKIT-0051. It complements the existing Import validation
and does not infer a schema from input tags.

## Red

Commit `d00b1eedc` added tests that select the three named schemas through the
public `ValidationSchema::Nested` API and check required-field and conditional
rules. Before implementation,
`cargo test -p kmipkit-protocol --lib managed_object_schemas_are_public_and_validate_required_fields -- --nocapture`
failed to compile because `NestedStructureSchema::{PgpKey,SecretData,SplitKey}`
did not exist.

## Green

Commit `704065342` adds those public schema identities, dispatches them to the
KMIP v2.1 Table 5, Table 8, and Table 9 descriptors, recursively validates each
Key Block, and applies the local Split Key Prime Field Size condition when
Polynomial Sharing Prime Field is selected.

- `cargo test -p kmipkit-protocol --lib managed_object_schemas_are_public_and_validate_required_fields -- --nocapture` passed.
- `cargo test -p kmipkit-protocol --lib public_split_key_schema_checks_conditional_prime_field_size -- --nocapture` passed.

## Refactor

The source descriptor and the public API documentation now name the same three
schema identities. Existing Import validation continues to use the same
descriptors, so the public entry point and pre-send path share their rules.
The enclosing Refactor commit records this evidence and updates the feature
traceability and English and Spanish client guides.

## Local verification

- `cargo fmt --all --check` passed.
- `cargo test --workspace --all-features --quiet` passed; the compile-fail
  extension-boundary test took 67.48 seconds and passed.
- `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings` passed after merging the identical `ClientRequest::validation_report` arms.
- Both focused public managed-object schema tests and the two KMIP 3.0 client
  validation/version-gate tests passed.
- `python -B scripts/tests/test_requirement_traceability.py` passed (2 tests);
  normative feature traceability passed (30 tests, 1 skipped because Windows
  could not create the test symlink); `python -B tools/api_manifest/generate.py
  --check` verified all 12 generated outputs.
- The whole-tree Rust export inventory remains an inherited open gate: the
  checked-in post-sync audit reports 4,199 routes missing from the manifest,
  and shared-inventory tests fail on that unresolved report. The versioned
  public route reconciliation remains assigned to KMIPKIT-0053; this branch
  does not claim the full export audit passed.

## KMIP 3.0 validation dispatch

Commit `d98077c3b` reproduced a non-exhaustive `ClientRequest::validation_report`
match after KMIPKIT-0050 added the seven KMIP 3.0 operations. Commit
`673d8c2c4` adds explicit draft-operation dispatch without applying KMIP 2.1
payload schemas. Execution still gates the operation and recursively checks
serialized request tags against the effective protocol version before any
transport exchange. The typed-request dispatch and the zero-exchange KMIP
2.1 rejection tests both pass.
