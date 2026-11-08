# ADR-0021: Server-to-Client Operations and Role Inversion Scope for KMIPKit

**Status**: Accepted by maintainer instruction on 2026-11-07
**Date**: 2026-11-07
**Decision owner**: KMIPKit maintainer
**Supersedes**: Directional client-only restriction of [ADR-0002](0002-kmip-21-release-scope.md)
**Specification**: [KMIPKIT-0066](../../specs/066-server-to-client-operations/spec.md)

## Context

The original project scope in ADR-0002 reserved Server-to-Client operations and dynamic role inversion for version 1.1.0 under the catalog scope tag `client_1_1`. The OASIS KMIP 2.1 standard specifies five Server-to-Client operations (§6.2.1–§6.2.5: `Discover Versions`, `Notify`, `Put`, `Query`, `Set Endpoint Role`), the symmetric Client-to-Server `Set Endpoint Role` operation (§6.1.54), and the transport clause in KMIP Profiles v2.1 §5.3.1 ("*KMIP clients that support responding to server to client operations SHALL behave as a HTTPS server*").

To achieve complete 100% standard coverage of all 62 operations defined in OASIS KMIP 2.1 across Rust, C, Java, and Python, the maintainer has authorized advancing these operations and transport listeners into the supported product scope.

## Decision

1. **Operations in Scope**:
   - Client-to-Server: `Set Endpoint Role` (Operation `0x0000_0032`, §6.1.54, Tables 331–333).
   - Server-to-Client: `Discover Versions` (Operation `0x0000_001E`, §6.2.1, Tables 343–344).
   - Server-to-Client: `Notify` (Operation `0x0000_001B`, §6.2.2, Table 345).
   - Server-to-Client: `Put` (Operation `0x0000_001C`, §6.2.3, Table 346).
   - Server-to-Client: `Query` (Operation `0x0000_0018`, §6.2.4, Tables 347–348).
   - Server-to-Client: `Set Endpoint Role` (Operation `0x0000_0032`, §6.2.5, Tables 349–351).
   - Profiles v2.1 §5.3.1: HTTPS server listener for receiving Server-to-Client operations.

2. **Security and Memory Invariants**:
   - All incoming connections must enforce mutual TLS 1.3 (`rustls` + `aws-lc-rs`) with mandatory certificate verification against a configured `TrustSource` and optional CRLs.
   - Decoders must strictly enforce `CodecLimits` (16 MiB message size, depth 64, 100,000 elements) before buffer allocation.
   - All secret key material received in `Put` requests (`SymmetricKey`, `PrivateKey`, `SecretData`, `SplitKey`, `PgpKey`) must be zeroized upon drop or handle destruction.
   - The client listener must honor `Maximum Response Size` indicated in incoming `RequestHeader` structures and emit `ResultReason::ResponseTooLarge` if the response exceeds the requested limit.

3. **Multi-Language Architecture**:
   - `kmipkit-protocol`: Defines typed models for all six operations, preserving unknown tags and enumeration values.
   - `kmipkit-transport`: Exposes `RawTlsListener`, `HttpsListener`, and `RawTlsRoleReversedStream`.
   - `kmipkit-client`: Exposes `ClientListener` and `ServerRequestDispatcher` with pull-based batch inspection and high-level handler execution.
   - `kmipkit-ffi` / `bindings`: Exposes equivalent C ABI, Java 17, and Python 3.12 bindings with 100% parity.

4. **Integration Testing and Live Verification**:
   - `crates/kmipkit-test-server` is augmented with an active Server-to-Client trigger mechanism capable of sending `Notify`, `Put`, `Query`, `DiscoverVersions`, and `SetEndpointRole` requests to a listening `KMIPKit` client over Raw TLS, HTTPS, and role-reversed streams across TTLV, JSON, and XML encodings.

## Consequences

- Constitution is updated with Amendment 1.2.0.
- `AGENTS.md` boundaries are aligned with the new capability.
- The library supports full bidirectional KMIP 2.1 communication.
- No public API breaking changes; existing client-to-server operations remain 100% backward-compatible.
