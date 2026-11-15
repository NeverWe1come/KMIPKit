# Specification: Multi-Version Modern KMIP Foundation (KMIPKIT-0069)

## 1. Overview and Purpose

This specification establishes the architectural foundation for multi-version OASIS KMIP client operations in `KMIPKit`. It governs support for modern KMIP protocol versions:
- **OASIS KMIP 2.0** (Standard, October 2019)
- **OASIS KMIP 2.1** (Standard, December 2020 — canonical default)
- **OASIS KMIP 3.0** (Draft/Standard — PQC and extensible governance)
- Future catalogued minor/major versions.

Simultaneously, it formally records the strategic obsolescence and exclusion of the legacy **KMIP 1.x (1.0–1.4)** family from the client library, while preserving existing test-server backwards adapters (`v1_compat.rs`) without regression.

The design strictly adheres to the **Zero-Breakage Golden Rule**:
All existing KMIP 2.1 public API signatures, defaults, and wire behaviors remain $100\%$ preserved and undisturbed. The default configured client version remains `ProtocolVersion::V2_1`.

## 2. Normative References

- **OASIS KMIP 2.0 Specification**: §8 Messages, §9 Protocol Details, §6.1.16 Discover Versions.
- **OASIS KMIP 2.1 Specification**: §8 Messages, §9 Protocol Details, §6.1.16 Discover Versions, §6.2.1 Server Discover Versions.
- **OASIS KMIP 3.0 Draft**: §8 Messages, §9 Protocol Details.
- **KMIPKit Constitution**: Core Principle I (Traceability), Core Principle IV (Secure Defaults), Core Principle V (Human Governance), and Amendment 1.3.0.
- **ADR-0022**: Multi-Version Modern KMIP Architecture.
- **AGENTS.md**: §8 Security Invariants (No automatic retries, delivery state tracking).

## 3. Scope and Public API Impact

1. **Protocol Version Abstraction (`kmipkit-protocol`)**:
   - `ProtocolVersion`: add constants `V2_0 = (2, 0)`, `V2_1 = (2, 1)`, `V3_0 = (3, 0)`.
   - Ordinal and comparison traits (`PartialOrd`, `Ord`, `Display`, `Hash`, `Eq`).
   - Catalog membership predicate `is_catalogued(&self) -> bool`.
   - `VersionSpan` metadata model representing validity range `[introduced, deprecated, removed)`.
   - `VersionGate` pre-send validator enforcing version compatibility for operations, tags, attributes, and structures.
   - `CodecContext` and `VersionedCodec` trait for wire serialization adaptation.
   - Multi-version `DiscoverVersionsRequest::with_versions(...)` and response parser.

2. **Client Configuration and Request Override (`kmipkit-client` & `kmipkit`)**:
   - `ClientConfiguration::with_protocol_version(version)` and alias `.with_kmip_version(version)`.
   - `RequestOptions::with_protocol_version(version)` and alias `.force_kmip_version(version)`.
   - `ClientBatch::with_protocol_version(version)` and alias `.force_kmip_version(version)`.
   - Dynamic `RequestHeader` protocol version formatting in `Client::execute` and asynchronous operations.
   - Strict version validation: local rejection with `RequestDeliveryState::NotSent` when attempting to execute operations or send tags outside the negotiated/effective version span.
   - Dynamic version handling in Server-to-Client `ServerResponseBuilder` and dispatching.

3. **Multi-Language Parity**:
   - C ABI exports in `kmipkit-ffi` (`kmipkit_client_configuration_with_protocol_version`, `kmipkit_request_options_with_protocol_version`, `kmipkit_request_options_force_kmip_version`, etc.).
   - Java 17 methods in `ClientConfiguration`, `RequestOptions`, and `ClientBatch`.
   - Python 3.12 properties and methods in `ClientConfiguration`, `RequestOptions`, and `ClientBatch`.

## 4. Acceptance Criteria

- **`REQ-069-001` (Zero-Breakage Default)**:
  Constructing a `Client` without explicit version configuration defaults to `ProtocolVersion::V2_1`, generating identical wire bytes and headers as prior releases.
- **`REQ-069-002` (Client-Level Version Configuration)**:
  `ClientConfiguration` allows setting an immutable default version (`V2_0`, `V2_1`, `V3_0`).
- **`REQ-069-003` (Per-Request / Per-Batch Version Override)**:
  `RequestOptions` and `ClientBatch` permit overriding the effective version for a single execution via `.with_protocol_version(...)` or `.force_kmip_version(...)` without mutating the client's configured default.
- **`REQ-069-004` (Strict Pre-Send VersionGate)**:
  Any attempt to execute an operation or send an attribute not supported by the effective version is rejected locally before transport write, returning a typed `ClientError` with `RequestDeliveryState::NotSent`.
- **`REQ-069-005` (Strict Catalog Validation)**:
  Non-catalogued protocol versions (such as KMIP 1.x or unmapped future major/minor pairs) are rejected with a descriptive error.
- **`REQ-069-006` (Discover Versions Multi-Version Capability)**:
  Clients can advertise multiple offered protocol versions in `DiscoverVersionsRequest`.
- **`REQ-069-007` (Four-Language Parity)**:
  Rust, C, Java, and Python provide equivalent version selection and error reporting.
