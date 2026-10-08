# Quickstart: Validate KMIPKIT-0016

Run these commands from the repository root after the KMIPKIT-0013 transport, KMIPKIT-0015 integration, and KMIPKIT-0014 shared attribute-model dependencies have been merged and the KMIPKIT-0016 implementation is present.

## Focused protocol tests

```powershell
cargo test --locked -p kmipkit-protocol attribute
```

Expected result: table-derived model/validation tests pass for all seven operations, including allocated/extension value preservation, Reserved-tag/enum rejection, repeated values, duplicate Get Attributes references, both omitted Delete Attribute selectors, and malformed structures. Get Attribute List has a UID-only request and required response references.

## Deterministic client execution

```powershell
cargo test --locked -p kmipkit-client attribute_execution
```

Expected result: fake transport confirms operation mapping, a single exchange, typed responses, KMIP error preservation, pending/delivery-state behavior, and no automatic retry without contacting a server.

## Normative traceability and generated catalog

```powershell
python tools/normative_catalog/validate.py
python tools/normative_catalog/report.py --repo-root . --check
```

Expected result: all seven operation elements and applicable requirements map to KMIPKIT-0016; generated reports match checked-in output. Use the repository's pinned automation task if it replaces these command paths before implementation.

## Workspace quality gates

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

Expected result: supported CI platforms pass, protocol/new-code coverage is at least 95%, transport/FFI/binding coverage is at least 85%, workspace coverage is at least 90%, and changed-code coverage is at least 95%. No live KMIP server is required for these deterministic tests.
