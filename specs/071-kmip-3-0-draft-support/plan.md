# Implementation Plan: OASIS KMIP 3.0 Working Draft Implementation (KMIPKIT-0071)

## 1. Architectural Strategy

- In `kmipkit-protocol`:
  - `LinkNormalizer`:
    - Define 20 discrete link tags (`0x0042_0153..=0x0042_0166`).
    - Bidirectional normalization functions `to_typed_link(generic_link) -> Item` and `to_generic_link(typed_link) -> Item`.
  - System Objects and `ObjectClass`:
    - Raw-preserving `ObjectClass`; KMIP 3.0 Table 581 assigns only `User=1` and `System=2`.
    - `ObjectType`: `User`, `Group`, `Password Credential`, `Device Credential`, `One Time Password Credential`, and `Hashed Password Credential` (Table 582).
    - Use checksum-verified, exact edition tables for generated enumeration names; do not treat reserved values as assigned.
  - Operations & KEM:
    - `DeactivateRequest` and `DeactivateResponse`.
    - `EncapsulateRequest`, `EncapsulateResponse`, `DecapsulateRequest`, `DecapsulateResponse`.
  - VersionGate:
    - Update `operation_span` from exact Operation Enumeration assignments in Table 583: `0x3C`–`0x42` have the names Create Group, Obliterate, Create User, Create Credential, Deactivate, Encapsulate, and Decapsulate respectively; unknown codes have no inferred span.
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
