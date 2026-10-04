# Quickstart: Generic TTLV Value Model

This guide describes the validation scenarios for the in-memory model. They become runnable only after the specification and ADR-0010 are approved, KMIPKIT-0003 implementation is merged, and implementation work lands. This model is not a TTLV encoder/decoder.

## Prerequisites

- Rust 1.94 toolchain and Cargo.
- Repository workspace with KMIPKIT-0003 implementation on the feature branch's release base.
- Accepted KMIPKIT-0004 specification and ADR-0010.
- Checked-in OASIS inputs and normative catalog; no network access to OASIS is needed.

## Validation scenarios

Run focused model tests:

```powershell
cargo test -p kmipkit-ttlv
```

Expected result: every one of the eleven value variants round-trips through in-memory construction and closure-scoped observation without width, sign, text, byte, or Big Integer octet changes. Structures retain order and repeated tags, accept nesting through 64 levels, and reject level 65 before mutation.

Check deterministic generated tag data:

```powershell
python tools/normative_catalog/generate_ttlv_tags.py --repo-root . --check
```

Expected result: exit code 0 only when generated Rust data matches the checked-in normative catalog and accepted precedence policy. The generator reads local catalog inputs only.

Run API contract checks:

```powershell
cargo test -p kmipkit-ttlv --test public_api
```

Expected result: runtime boundary tests reject an out-of-range Raw Tag without truncation; `trybuild` cases reject borrowed references escaping the exposure callback and reject `Value`, `ValueView`, `StructureView`, `Item`, and `Structure` as `serde::Serialize`, `Clone`, or `Copy` with matched diagnostic snapshots.

Run safe zeroization probes:

```powershell
cargo test -p kmipkit-ttlv --test zeroization
```

Expected result: a safe test spy confirms the secret wrapper's Drop path invokes `Zeroize` for every payload variant and nested Structure. Review the locked `zeroize` implementation for the String/Vec current-capacity guarantee; tests do not inspect released memory or claim to observe old allocations.

Run workspace checks after focused scenarios:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

Expected result: all commands exit successfully, with at least 95% coverage for changed code and the `kmipkit-ttlv`/protocol-model crates, at least 90% workspace-wide, and at least 85% for transport/FFI only if those paths are affected. Record actual command output and coverage/security results in the implementation PR; the commands above are proposed validation steps, not evidence that they have already passed.

## Later codec validation boundary

This feature cannot validate TTLV wire framing, exact encoded lengths, endianness, padding, malformed input, decoder limits, or OASIS wire vectors. Those scenarios belong to the follow-on codec specification and must be run before claiming structurally valid/wire-valid TTLV support.
