# T056 workspace verification

## Outcome

Completed local workspace, documentation, traceability, and dependency-policy
verification for KMIPKIT-0013. The fail-closed client source inventory now
permits only the two exact crate-level `include_str!` attributes used to embed
the checked-in English and Spanish transport guides. Runtime includes and
other documentation paths remain rejected.

## Verification

- `cargo fmt --all --check`: PASS.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: PASS.
- `cargo test --workspace --all-features --locked`: PASS on rerun, including all doctests.
- `cargo test --workspace --doc --all-features --locked`: PASS — 17 passing doctests and 1 expected compile-fail doctest.
- `cargo test -p kmipkit-client --all-targets --all-features --locked`: PASS.
- `cargo test -p kmipkit-client --lib execute_boundary_tests:: --locked`: PASS — 37 tests.
- `cargo test -p kmipkit-transport --test raw_tls --locked`: PASS — 82 tests.
- `python tools/normative_catalog/validate.py`: PASS — 4 sources, 1,411 clauses, 4,024 records.
- `python -m unittest tools.normative_catalog.tests.test_feature_traceability`: PASS — 14 tests, 1 existing skipped test.
- `pwsh -File scripts/Test-DependencyPolicy.ps1`: PASS. `cargo-deny` 0.20.2 was verified; root and fuzz RustSec checks used advisory database commit `5701a6d2a6158d5d4a933df5141a0415c285075d`; validated exceptions were `KMIPKIT-0011-EX-003` and `KMIPKIT-0011-EX-004`; both lockfile hashes remained unchanged.

The first concurrent workspace-validation attempt reported a single mTLS
handshake assertion in
`raw_tls_response_allocation_observer_is_scoped_to_its_adapter`. The test
passed in isolation, all 82 tests in the complete raw-TLS target passed, and
the subsequent full workspace command passed. No source change was made to
mask or skip that assertion.

## Development commits

- Red: `6f63658` — constrain the source-inventory exception with a regression test.
- Green: `2f90f83` — allow only the two reviewed root guide includes.
- Refactor: `c2a152d` — document why the narrow documentation exception bypasses macro traversal.
