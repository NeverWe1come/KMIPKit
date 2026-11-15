# Task 25 — Shared Table 59 Parser

Date: 2026-10-10

Branch: `feature/KMIPKIT-0019-encrypt-decrypt-implementation`

## Scope and behavior review

Before editing production code, the complete `validate_known_cryptographic_parameters_view` bodies in `encrypt.rs` and `decrypt.rs` were compared and found identical. The extraction centralizes only the 18-member Table 59 TTLV Item Type and singleton-cardinality checks. Encrypt and Decrypt payload field acceptance, request/response types, response error variants and messages, and outer validation remain separate.

The helper preserves the prior match order, wrong-type-before-duplicate check, unknown-tag acceptance, and sanitized `InvalidValue` / `InvalidValue` error. It observes borrowed values and does not rebuild the Structure, so unknown members and order remain intact. The operation wrappers still invoke Table 59 validation before §4.16 conditional-presence validation.

## Red characterization

The characterization test passed against the original duplicated implementation as required for a behavior-preserving extraction. It exercises a wrong Item Type and duplicate singleton for each of the 18 members through both real operation validators, then checks equal error values, exact safe categories, and payload-free Display output.

Commands below ran in the assigned worktree through WSL Ubuntu-26.04.

```powershell
cargo test -p kmipkit-protocol --lib encrypt_parameter_structure_tests::encrypt_and_decrypt_table_59_errors_are_payload_free_and_identical --offline -- --exact
```

Result before extraction: 1 passed, 0 failed. DCO-signed Red commit: `0e048e6 test(KMIPKIT-0019): characterize Table 59 validation parity`.

## Green extraction

Moved the shared validator into `crates/kmipkit-protocol/src/cryptographic_parameters.rs::validate_known_cryptographic_parameters_view`, where its tag constants are shared with the existing §4.16 required-member validator. Removed only the duplicate Table 59 constants/helper definitions from `encrypt.rs` and `decrypt.rs`; each operation continues to call the shared parser at its previous position.

Focused verification commands:

```powershell
cargo test -p kmipkit-protocol --lib --offline encrypt_parameter_structure_tests
cargo test -p kmipkit-protocol --lib --offline cryptographic_parameters_tests
cargo test -p kmipkit-protocol --lib --offline malformed_crypto_payload_tests
cargo test -p kmipkit-protocol --lib --offline encrypt_tests
cargo test -p kmipkit-protocol --lib --offline decrypt_tests
```

Results: 5, 5, 14, 4, and 4 tests passed respectively. DCO-signed Green commit: `25e2544 refactor(KMIPKIT-0019): share Table 59 parameter validation`.

## Refactor checks

```powershell
cargo fmt --all --check
cargo clippy -p kmipkit-protocol --all-targets --all-features --offline -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p kmipkit-protocol --no-deps --all-features --offline
```

All three checks passed. No workspace test suite was run. No public API, production dependency, operation-specific field policy, request/response model, or response error type/message changed.
