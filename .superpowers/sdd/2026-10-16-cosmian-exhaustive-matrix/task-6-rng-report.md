# Task 6 RNG Report

## Scope and result

Completed the KMIPKIT-0015 RNG Retrieve and RNG Seed matrix family using the typed KMIPKit client for the six operation-valid, parameter-absent cases. The six rows that add `RNG Parameters { RNG Algorithm: Unspecified }` remain non-executable normative exclusions. OASIS KMIP v2.1 §7.36, Table 388 defines the RNG Parameters structure; the request payloads in §§6.1.49–6.1.50, Tables 316 and 319, do not include it. No raw TTLV was hand-built, no production API was changed, and no request was sent for the excluded rows.

The live test is `#[ignore]` by default. Response checks verify the typed response shape and requested or consumed lengths without logging RNG output or seed bytes.

## Case IDs

Live typed-client cases:

- `cosmian-528-rng-retrieve-0-parameters-false`
- `cosmian-528-rng-retrieve-1-parameters-false`
- `cosmian-528-rng-retrieve-32-parameters-false`
- `cosmian-528-rng-retrieve-65536-parameters-false`
- `cosmian-528-rng-seed-0-parameters-false`
- `cosmian-528-rng-seed-32-parameters-false`

Non-executable normative exclusions, all recorded with `request_may_be_sent: false`:

- `cosmian-528-rng-retrieve-0-parameters-true`
- `cosmian-528-rng-retrieve-1-parameters-true`
- `cosmian-528-rng-retrieve-32-parameters-true`
- `cosmian-528-rng-retrieve-65536-parameters-true`
- `cosmian-528-rng-seed-0-parameters-true`
- `cosmian-528-rng-seed-32-parameters-true`

## TDD commits

- Red: `a1f57235fad5a5e300e0e2fc981d1657dbf5d308` — added a regression requiring each exclusion's exact OASIS citations in its reason and notes. `cargo test -p kmipkit-client --test cosmian_rng_matrix` failed because the §7.36 citation used `§7.36/Table 388` instead of `§7.36, Table 388`.
- Green: `041a34fb78b90f16ba4d0c12ed1c9cb6073afdc4` — completed typed RNG cases, 12-ID parity, and exclusion source mapping. The Red regression passed after the matrix notes and operation-specific citations were corrected.

An earlier interim test commit, `fc49e44b1580e76cb4610d5cf0f18f688199ff82`, reflected the initial client-blocker interpretation. It was superseded after review of the pinned OASIS operation payload tables; the final classification is `standards_invalid_exclusion`.

## Verification

- `cargo test -p kmipkit-client --test cosmian_rng_matrix` — passed, 1 test.
- `cargo test -p kmipkit-client --test cosmian_kms rng` — passed, 1 test; validates the six RNG exclusions without sending.
- `cargo test -p kmipkit-client --test cosmian_kms random::` — passed, 4 tests; 1 live Cosmian test ignored as configured.
- `rustfmt --edition 2024 --check crates/kmipkit-client/tests/cosmian_rng_matrix.rs crates/kmipkit-client/tests/cosmian_kms/random.rs` — passed.
- `cargo fmt --all --check` — passed.

Cargo emitted AWS-LC C compiler feature-check and missing NASM warnings in this Windows environment; compilation completed and all listed commands returned success after Green. No live Compose or network test was run. The parent task will execute live families serially when ready.
