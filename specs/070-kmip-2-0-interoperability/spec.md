# Specification: OASIS KMIP 2.0 Interoperability and Conformance (KMIPKIT-0070)

## 1. Overview and Purpose

This specification defines the complete interoperability, validation, and test harness capabilities for **OASIS KMIP 2.0** within KMIPKit.
Building upon the multi-version modern foundation established in `KMIPKIT-0069`, this feature implements:
1. Exact OASIS KMIP 2.0 normative boundary enforcement (differentiating 2.0 from 2.1).
2. Protocol-level operation and attribute gating for KMIP 2.0.
3. Test server (`kmipkit-test-server`) KMIP 2.0 mode and version negotiation.
4. Conformance and interoperability test suites validating compatibility against Cosmian KMS, Thales CipherTrust (CT), and Utimaco Enterprise Secure Key Manager (ESKM).

## 2. Normative References

- **OASIS KMIP 2.0 Specification** (OASIS Standard, 19 July 2019):
  - §4 Cryptographic Objects
  - §5 Attributes (§5.1–§5.7 cryptographic attribute read/rotate constraints)
  - §6 Operations (§6.1.1–§6.1.53 operations available in 2.0)
  - §8 Messages
  - §9 Protocol Details
- **OASIS KMIP 2.0 Profiles** (OASIS Standard, 19 July 2019):
  - §3 Profile Definitions (Baseline, Complete Server)
  - §5 Conformance Clauses
- **OASIS KMIP 2.1 Specification** (for normative delta analysis):
  - Operations added in 2.1 (§6.1.54–§6.1.59): `RegisterOpaqueContext`, `RegisterOpaque`, `SetConstraints`, `GetConstraints`, `Export`, `Import`.
- **KMIPKit Constitution**: Principles I, IV, V, and Amendment 1.3.0.
- **ADR-0022**: Multi-Version Modern KMIP Architecture.

## 3. Normative Deltas: KMIP 2.0 vs KMIP 2.1

### 3.1 Prohibited Operations in KMIP 2.0
The following six operations were added in KMIP 2.1 and **MUST NOT** be issued when operating under KMIP 2.0:
1. `RegisterOpaqueContext` (`0x00000036`)
2. `RegisterOpaque` (`0x00000037`)
3. `SetConstraints` (`0x00000038`)
4. `GetConstraints` (`0x00000039`)
5. `Export` (`0x0000003A`)
6. `Import` (`0x0000003B`)

Any client request targeting KMIP 2.0 with one of these operations **MUST** be rejected pre-send by `VersionGate` with `RequestDeliveryState::NotSent`.

### 3.2 Attribute Modification Constraints in KMIP 2.0
In KMIP 2.0, seven core cryptographic attributes are immutable or modifyable solely via `Rekey` / `RekeyKeyPair` / `Rotate` operations, and **MUST NOT** be modified via `SetAttribute`:
1. `Cryptographic Algorithm` (`0x00420028`)
2. `Cryptographic Length` (`0x0042002A`)
3. `Cryptographic Parameters` (`0x0042002B`)
4. `Cryptographic Usage Mask` (`0x0042002C`)
5. `Digest` (`0x00420032`)
6. `Key Value Present` (`0x004200BF`)
7. `Key Format Type` (`0x00420044`)

### 3.3 Server and Wire Protocol Details
- Request and response headers **MUST** carry `ProtocolVersion` major=2, minor=0.
- `kmipkit-test-server` supports a configured or negotiated 2.0 mode. When in 2.0 mode, the server rejects 2.1-only operations with `OperationFailed` / `OperationNotSupported`.
- Existing `v1_compat.rs` in `kmipkit-test-server` **MUST NOT** be modified or deleted.

## 4. Acceptance Criteria

- **`REQ-070-001` (Normative Ingestion & Inventory)**:
  OASIS KMIP 2.0 specifications are pinned in `specification/oasis/kmip-2.0/upstream/` with cryptographic checksums recorded in `CHECKSUMS.sha256` and `SOURCES.md`.
- **`REQ-070-002` (KMIP 2.0 Pre-Send Operation Gating)**:
  `VersionGate` validates that operations prohibited in KMIP 2.0 are rejected pre-send when effective version is `ProtocolVersion::V2_0`.
- **`REQ-070-003` (KMIP 2.0 Attribute Mutation Policy)**:
  The 7 cryptographic attributes that are rotate-only in KMIP 2.0 are prohibited from `SetAttribute` mutation under KMIP 2.0.
- **`REQ-070-004` (Test Server 2.0 Mode)**:
  `kmipkit-test-server` supports KMIP 2.0 mode, responding with version (2, 0) and rejecting 2.1 operations with `OperationNotSupported`.
- **`REQ-070-005` (Deterministic Client Conformance Tests)**:
  Client tests demonstrate full lifecycle operations (Create, Register, Get, Destroy, Locate, Query, DiscoverVersions) under KMIP 2.0 mode against `kmipkit-test-server`.
- **`REQ-070-006` (Commercial/Independent KMS Interoperability Validation)**:
  Test fixtures and vectors validate compatibility against Cosmian KMS, Thales CipherTrust, and Utimaco ESKM KMIP 2.0 profiles.
