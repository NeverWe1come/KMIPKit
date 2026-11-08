# Specification: OASIS KMIP 3.0 Working Draft Implementation (KMIPKIT-0071)

## 1. Overview and Purpose

This specification defines the comprehensive support for the **OASIS KMIP 3.0 Working Draft** in KMIPKit.
In accordance with maintainer directives, this implementation is developed in a dedicated feature branch and maintained in an open Draft Pull Request (`[WIP]`) until the OASIS Technical Committee formally completes final ratification and standard publication.

The specification introduces:
1. Bidirectional Link Normalization (`LinkNormalizer`) bridging KMIP 2.x generic links and KMIP 3.0 discrete typed links.
2. System Objects (`User`, `Group`, `Password Credential`, `Device Credential`, `One Time Password Credential`, and `Hashed Password Credential`).
3. `Object Class` values `User` (`1`) and `System` (`2`), preserving unrecognized raw values.
4. The KMIP 3.0 Operation Enumeration assignments in §11.37, Table 583: `Create Group` (`0x3C`), `Obliterate` (`0x3D`), `Create User` (`0x3E`), `Create Credential` (`0x3F`), `Deactivate` (`0x40`), `Encapsulate` (`0x41`), and `Decapsulate` (`0x42`).
5. Post-Quantum Cryptography (PQC) algorithm enumerations and Key Encapsulation Mechanism (KEM) data structures.
6. Test Server (`kmipkit-test-server`) KMIP 3.0 execution mode.

## 2. Normative References

- **OASIS KMIP 3.0 Working Draft Specification** (13 July 2026):
  - §4 Cryptographic and System Objects (§4.8 User, §4.9 Group, §4.10 Credential)
  - §5 Attributes (§5.2 Object Class, §5.45–§5.64 Discrete Typed Links)
  - §6 Operations (§6.1.9 Create Credential, §6.1.10 Create Group, §6.1.13 Create User, §6.1.14 Deactivate, §6.1.15 Decapsulate, §6.1.22 Encapsulate, §6.1.41 Obliterate)
  - §11.34 Table 581 Object Class Enumeration; §11.35 Table 582 Object Type Enumeration; §11.37 Table 583 Operation Enumeration
  - §8 Messages
  - §9 Protocol Details
- **OASIS KMIP 3.0 Working Draft Profiles** (13 July 2026)
- **KMIPKit Constitution**: Principles I, IV, V, and Amendment 1.3.0.
- **ADR-0022**: Multi-Version Modern KMIP Architecture.

## 3. Detailed Architectural Requirements

### 3.1 Typed Link Normalization (`LinkNormalizer`)
- In KMIP 2.x, relationships are represented via generic `Link` (`0x00420047`) containing `Link Type` (`0x00420048`) and `Linked Object Identifier` (`0x00420049`).
- In KMIP 3.0, generic `Link` is replaced by 20 discrete typed links (`0x00420153` through `0x00420166`):
  - `CertificateLink` (`0x00420153`)
  - `PublicKeyLink` (`0x00420154`)
  - `PrivateKeyLink` (`0x00420155`)
  - `PreviousKeyLink` (`0x00420156`)
  - `NextKeyLink` (`0x00420157`)
  - `DerivationBaseObjectLink` (`0x00420158`)
  - `DerivedKeyLink` (`0x00420159`)
  - `SplitKeyLink` (`0x0042015A`)
  - `WrappingKeyLink` (`0x0042015B`)
  - `ParentObjectLink` (`0x0042015C`)
  - `ChildObjectLink` (`0x0042015D`)
- `LinkNormalizer` converts 2.x generic links to 3.0 typed links when encoding for a 3.0 endpoint, and unpacks 3.0 typed links to canonical 2.x representations when requested by legacy code.

### 3.2 System Objects and Object Class
- `ObjectType` includes:
  - `User` (`0x0000000B`)
  - `Group` (`0x0000000C`)
  - `Password Credential` (`0x0000000D`)
  - `Device Credential` (`0x0000000E`)
  - `One Time Password Credential` (`0x0000000F`)
  - `Hashed Password Credential` (`0x00000010`)
- `ObjectClass` is a raw-preserving wrapper. The assigned KMIP 3.0 values are
  `User = 1` and `System = 2`; `3` is not an assigned value in Table 581.

### 3.3 Post-Quantum Cryptography & KEM Operations
- `CryptographicAlgorithm` adds PQC variants:
  - `MlKem512`, `MlKem768`, `MlKem1024`
  - `MlDsa44`, `MlDsa65`, `MlDsa87`
  - `SlhDsa`
- KEM operations:
  - `Encapsulate` (`0x0000003F`): takes public key identifier, returns ciphertext and shared secret.
  - `Decapsulate` (`0x00000040`): takes private key identifier and ciphertext, returns shared secret.

### 3.4 Deactivate Operation
- `Deactivate` (`0x0000003E`): explicitly transitions an active object to the deactivated lifecycle state without immediate destruction.

## 4. Acceptance Criteria

The pre-1.0 API corrections to `ObjectType::CREDENTIAL` and the former
`ObjectClass` enum are recorded in the [KMIP 3.0 API migration note](../../docs/development/kmip-3.0-api-migration.md).

- **`REQ-071-001` (Normative Ingestion)**:
  OASIS KMIP 3.0 draft specification files are pinned and tracked with SHA-256 digests.
- **`REQ-071-002` (Link Normalizer)**:
  `LinkNormalizer` accurately roundtrips between KMIP 2.x generic links and KMIP 3.0 discrete typed links.
- **`REQ-071-003` (KMIP 3.0 Operations in VersionGate)**:
  `VersionGate` permits only the seven assigned operation codes in Table 583
  under `ProtocolVersion::V3_0`, and rejects those assignments under `V2_0`
  and `V2_1`. Unassigned values, including `0x43`, receive no inferred span.
- **`REQ-071-004` (System Objects and Object Class)**:
  `ObjectType` and `ObjectClass` types encode and decode without data loss.
- **`REQ-071-005` (PQC & KEM Models)**:
  `EncapsulateRequest`, `EncapsulateResponse`, `DecapsulateRequest`, and `DecapsulateResponse` are implemented and validated.
- **`REQ-071-006` (Test Server KMIP 3.0 Mode)**:
  `kmipkit-test-server` supports KMIP 3.0 mode, returns protocol version 3.0, and executes 3.0 operations.
