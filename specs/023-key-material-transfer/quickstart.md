# Validation Quickstart: KMIPKIT-0037

This is the implementation validation guide for Derive Key, Export, Import, and Join Split Key. It is not a formal OASIS conformance, profile, certification, or interoperability claim.

## Prerequisites

- Start from the active `release/1.0.0` line in the dedicated implementation worktree.
- Confirm shared TTLV, object/attribute, operation-data, Cryptographic Parameters, batch/result, client, transport, and secret-byte contracts are available.
- Verify pinned OASIS source hashes and do not alter upstream files.
- Rust protocol/client operation work can proceed without KMIPKIT-0043. Complete C/Java/Python mapping only after the canonical v2 manifest and operation tasks are available.
- Join Split Key callers must provide one known threshold context for every identifier occurrence; no implicit object lookup is allowed.

## Focused validation

Run each operation's protocol unit tests and fake-transport client tests. Cover tables in [spec.md](spec.md), operation IDs, one exchange, common result and delivery state, negative/malformed input, unknown-value retention, and secret redaction. Exercise the Derive Key §7.12/Table 364 and Join Split Key threshold rules directly.

The pinned inventory links Derive Key to KMIPKIT-TEST-CN01-2-14 through -19; Export and Import to -43 through -47; Join Split Key has no direct catalog case ID. Use only local fixtures if present, and label table-derived vectors separately.

## Repository gates

From the repository root, run focused cargo targets and then:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --check
```

Run pinned generator drift, security/dependency, and supported-platform CI checks per repository automation. Do not report success without output from the command or CI run.

## Release evidence

Record separate Red, Green, and Refactor commit IDs per story. The implementation PR includes focused/full check commands and outputs, coverage/traceability/catalog evidence, parity mapping state, fixture limitations, and risks. Update this file with actual evidence only during implementation.

## Implementation setup evidence

- Managed worktree: `C:\Users\ramp1953\.codex\worktrees\kmipkit-0037-implementation\KMIPKit`.
- Branch: `feature/KMIPKIT-0037-key-material-transfer-implementation` (the prescribed `feature/KMIPKIT-0037-key-material-transfer` name was already checked out by another task; that checkout was not modified).
- Base: active `origin/release/1.0.0` at `af172974cd5c6f56f7f938574e1546d4e295a9ef`; worktree was clean at branch creation.
- `Get-FileHash -Algorithm SHA256` comparison against `specification/oasis/kmip-2.1/CHECKSUMS.sha256`: all pinned source checksums verified; upstream tree clean.
- Baseline `cargo test --workspace --all-features`: passed on the unchanged release base (exit 0).
- Shared Rust API review confirmed `AttributeSet`, `ObjectType`, `UniqueIdentifier`, `OperationData`, `Cryptographic Parameters`, `KmipOperationResult`, generic ordered TTLV, typed batch/result/delivery, fake transport tests, and secret byte/item ownership are available. Cross-language bindings remain gated on KMIPKIT-0043's canonical v2 manifest and mappings.

## Derive Key cycle evidence

- Red commits: protocol `3ca0d0b2`; fake-client `6d583aa0`.
- Green commits: protocol model `7616b79d`; client dispatch `bcb0ad9f`.
- Refactor commit: `3ede3903`.
- Passed `cargo test -p kmipkit-protocol --lib derive_key_operation_tests` (5 tests) and `cargo test -p kmipkit-client --lib derive_key_execution_tests` (1 test).
- Passed `cargo fmt --all --check`, `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`, and `git diff --check`.
- Fixture vectors derive from KMIP 2.1 tables and do not establish official conformance or independent implementation interoperability.
