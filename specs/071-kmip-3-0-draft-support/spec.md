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
  - §11.58 Table 605 Tag Enumeration
  - §8 Messages
  - §9 Protocol Details
- **OASIS KMIP 3.0 Working Draft Profiles** (13 July 2026)
- **KMIPKit Constitution**: Principles I, IV, V, and Amendment 1.3.0.
- **ADR-0022**: Multi-Version Modern KMIP Architecture.

## 3. Detailed Architectural Requirements

### 3.1 Typed Link Normalization (`LinkNormalizer`)
- In KMIP 2.1, a generic `Link` (`0x00420047`) contains `Link Type`
  (`0x00420048`) and `Linked Object Identifier` (`0x00420049`). Its 14 assigned
  Link Type values are defined in §11.28, Table 465; the linked identifier may
  use Text String, Enumeration, or Integer as specified by §4.31, Table 90.
- KMIP 3.0 defines 20 individual link attributes in §§4.35.1–4.35.20, with
  tags assigned by §11.58. The 14 semantic equivalents and their explicit
  mappings are:

| KMIP 2.1 Link Type | KMIP 3.0 link attribute | KMIP 3.0 tag |
|---|---|---|
| `0x101` Certificate Link | Certificate Link (§4.35.1) | `0x00420190` |
| `0x102` Public Key Link | Public Key Link (§4.35.16) | `0x0042019A` |
| `0x103` Private Key Link | Private Key Link (§4.35.15) | `0x00420199` |
| `0x104` Derivation Base Object Link | Derivation Base Object Link (§4.35.5) | `0x00420192` |
| `0x105` Derived Key Link | Derived Object Link (§4.35.6) | `0x00420193` |
| `0x106` Replacement Object Link | Replacement Object Link (§4.35.18) | `0x0042019C` |
| `0x107` Replaced Object Link | Replaced Object Link (§4.35.17) | `0x0042019B` |
| `0x108` Parent Link | Parent Link (§4.35.10) | `0x00420195` |
| `0x109` Child Link | Child Link (§4.35.3) | `0x00420191` |
| `0x10A` Previous Link | Previous Link (§4.35.14) | `0x00420198` |
| `0x10B` Next Link | Next Link (§4.35.9) | `0x00420194` |
| `0x10C` PKCS#12 Certificate Link | PKCS#12 Certificate Link (§4.35.12) | `0x00420196` |
| `0x10D` PKCS#12 Password Link | PKCS#12 Password Link (§4.35.13) | `0x00420197` |
| `0x10E` Wrapping Key Link | Wrapping Key Link (§4.35.20) | `0x0042019D` |

- Six KMIP 3.0 links have no KMIP 2.1 Link Type equivalent and are not
  normalized to KMIP 2.1: Credential Link (`0x004201A0`), Password Link
  (`0x004201A5`), Group Link (`0x004201B3`), Split Key Base Link
  (`0x004201B4`), Joined Split Key Parts Link (`0x004201B5`), and Certificate
  Request Link (`0x004201BC`). In particular, `0x00420153` is Object Defaults,
  not a link tag.
- `LinkNormalizer` converts only the explicit 14-entry semantic table.
  Enumeration and Integer identifiers are preserved. Text String identifiers
  are rejected when converting to KMIP 3.0 because the §4.35 link attribute
  encodings do not list Text String; Reference and Name Reference values are
  rejected when converting to KMIP 2.1 because its generic Link has no such
  encoding. Unknown children, duplicate required fields, and links without a
  counterpart are rejected instead of silently dropping or coercing data.

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
  `LinkNormalizer` accurately roundtrips the 14 source-defined shared Link Type semantics using the explicit mapping above, preserves Enumeration and Integer identifiers, rejects Text String conversion to KMIP 3.0, and rejects the six KMIP 3.0-only link attributes without changing the input.
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
- **`REQ-071-007` (Exact Tag Enumeration)**:
  The versioned tag catalog records all 60 explicitly assigned values in
  §11.58, Table 605, including credential, system-object, certificate,
  deactivation, and KEM tags. It does not turn reserved or unassigned values
  into assignments by numeric range. The allocation union is not a substitute
  for effective-version validation.
