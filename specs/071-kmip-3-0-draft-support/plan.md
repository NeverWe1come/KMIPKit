# Implementation Plan: OASIS KMIP 3.0 Working Draft Implementation (KMIPKIT-0071)

## 1. Architectural Strategy

- In `kmipkit-protocol`:
  - `LinkNormalizer`:
    - Define 20 discrete link tags (`0x0042_0153..=0x0042_0166`).
    - Bidirectional normalization functions `to_typed_link(generic_link) -> Item` and `to_generic_link(typed_link) -> Item`.
  - System Objects and `ObjectClass`:
    - `ObjectClass` enum: `CryptographicObject`, `SystemObject`, `MetadataObject`.
    - `ObjectType`: extend with `User`, `Group`, `Credential`.
  - Operations & KEM:
    - `DeactivateRequest` and `DeactivateResponse`.
    - `EncapsulateRequest`, `EncapsulateResponse`, `DecapsulateRequest`, `DecapsulateResponse`.
  - VersionGate:
    - Update `operation_span` to confirm `0x3C..=0x41` are supported in `V3_0`.
- In `kmipkit-test-server`:
  - Support `(3, 0)` protocol version in `v1_compat.rs` `ALL_VERSIONS` and `is_version_supported`.
  - Dispatcher support for `Deactivate`, `Encapsulate`, `Decapsulate`.
- In `kmipkit-client`:
  - Client methods for `deactivate`, `encapsulate`, `decapsulate`.
  - Unit and integration tests executing KMIP 3.0 operations against `kmipkit-test-server`.

## 2. Red, Green, Refactor TDD Steps

1. **Commit 1 (Docs & Governance)**: Pin KMIP 3.0 OASIS documents, spec, plan, tasks, and traceability.
2. **Commit 2 (Red)**: Add unit tests for `LinkNormalizer`, System Objects, `ObjectClass`, and KEM operations in `kmipkit-protocol`.
3. **Commit 3 (Green)**: Implement `LinkNormalizer`, models, and test-server 3.0 mode.
4. **Commit 4 (Refactor)**: Format, pass clippy, run full workspace verification.
