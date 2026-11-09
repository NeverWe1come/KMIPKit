# Specification: Server-to-Client Operations and Role Inversion (KMIPKIT-0066)

## 1. Overview and Purpose

This specification governs the implementation of Server-to-Client operations and dynamic endpoint role inversion in `KMIPKit` in compliance with OASIS KMIP 2.1 §§6.1.54, 6.2.1–6.2.5 and KMIP Profiles v2.1 §5.3.1.

The library enables a KMIP client to:
1. Initiate the `Set Endpoint Role` operation (§6.1.54) to request server role reversal over an established connection.
2. Accept and dispatch incoming Server-to-Client operations (§6.2.1 `Discover Versions`, §6.2.2 `Notify`, §6.2.3 `Put`, §6.2.4 `Query`, §6.2.5 `Set Endpoint Role`) over mutual TLS 1.3 listeners (`Raw TLS` and `HTTPS / HTTP 1.1`) and established role-reversed TLS streams.
3. Decouple message processing and security policy enforcement (CodecLimits, strict mTLS, zeroization of received secrets) with 100% parity across Rust, C, Java, and Python.

## 2. Normative References

- **OASIS KMIP 2.1 §6.1.54**: Set Endpoint Role (client-to-server), Tables 331–333.
- **OASIS KMIP 2.1 §6.2.1**: Discover Versions (server-to-client), Tables 343–344.
- **OASIS KMIP 2.1 §6.2.2**: Notify (server-to-client), Table 345.
- **OASIS KMIP 2.1 §6.2.3**: Put (server-to-client), Table 346.
- **OASIS KMIP 2.1 §6.2.4**: Query (server-to-client), Tables 347–348.
- **OASIS KMIP 2.1 §6.2.5**: Set Endpoint Role (server-to-client), Tables 349–351.
- **OASIS KMIP Profiles v2.1 §5.3.1**: HTTPS Client behaving as HTTPS Server.
- **RFC 8446**: The Transport Layer Security (TLS) Protocol Version 1.3.
- **RFC 9112**: HTTP/1.1 Message Syntax and Routing.
- **KMIPKit Constitution Amendment 1.2.0** and **ADR-0021**.

## 3. Scope and Public API Impact

- **New protocol models** in `kmipkit-protocol`:
  - `EndpointRole` (`CLIENT = 1`, `SERVER = 2`).
  - `SetEndpointRoleRequest`, `SetEndpointRoleResponse`.
  - `ServerDiscoverVersionsRequest`, `ServerDiscoverVersionsResponse`.
  - `ServerNotifyRequest`, `ServerNotifyResponse`.
  - `ServerPutRequest`, `ServerPutResponse`, `PutFunction` (`NEW = 1`, `REPLACE = 2`).
  - `ServerQueryRequest`, `ServerQueryResponse`, `ServerCapabilities`.
  - `ServerSetEndpointRoleRequest`, `ServerSetEndpointRoleResponse`.
  - `ServerResponseBuilder`.
- **New transport listeners** in `kmipkit-transport`:
  - `ListenerTransportConfig`, `RawTlsListener`, `HttpsListener`, and `RawTlsRoleReversedStream`.
- **New client facade and dispatcher** in `kmipkit-client` and `kmipkit`:
  - `Client::set_endpoint_role` / `Client::set_endpoint_role_with_options`.
  - `Client::enter_server_role_session`.
  - `ClientListener`, `ServerRequestDispatcher`, `IncomingServerBatch`, `ServerRequestItemView`.
- **Test server enhancement** in `crates/kmipkit-test-server`:
  - Active Server-to-Client trigger capabilities to push `Notify`, `Put`, `Query`, `DiscoverVersions`, and `SetEndpointRole` to client listeners.
- **Multi-language bindings**:
  - Full C ABI, Java 17, and Python 3.12 parity.

## 4. Acceptance Criteria

1. **`REQ-066-001` (Set Endpoint Role C2S / S2C)**:
   The client can send and receive `Set Endpoint Role` (§6.1.54 / §6.2.5), validating required fields, encoding/decoding role enumerations, and preserving unknown vendor roles.
2. **`REQ-066-002` (Server-to-Client Discover Versions)**:
   Incoming `Discover Versions` requests (§6.2.1) are negotiated against client-supported protocol versions strictly following the 3 normative rules of §6.2.1.
3. **`REQ-066-003` (Server-to-Client Notify)**:
   Incoming `Notify` requests (§6.2.2) require `UniqueIdentifier`, expose changed attributes as typed `AttributeSet`, deleted attributes as `AttributeReference`, and validate Table 150 server-modified exemptions.
4. **`REQ-066-004` (Server-to-Client Put)**:
   Incoming `Put` requests (§6.2.3) require `UniqueIdentifier`, `PutFunction`, and managed object payload. Validates that `Replaced Unique Identifier` is present when `PutFunction == Replace`, evaluates `effective_put_function`, and automatically zeroizes received secret keys.
5. **`REQ-066-005` (Server-to-Client Query)**:
   Incoming `Query` requests (§6.2.4) return configured capabilities and extension lists/maps, enforcing that `Query Extension Map` supersedes `Query Extension List` when both are present.
6. **`REQ-066-006` (mTLS 1.3 Transport Listeners)**:
   `RawTlsListener` and `HttpsListener` enforce TLS 1.3 with client certificate validation against trusted CA roots and CRLs, hot-reload integration, and strict `CodecLimits` pre-allocation.
7. **`REQ-066-007` (Role-Reversed Stream)**:
   `RawTlsRoleReversedStream` permits receiving server requests and returning responses over the same TLS connection after an accepted `Set Endpoint Role` exchange.
8. **`REQ-066-008` (Test Server Active Trigger & Integration)**:
   `kmipkit-test-server` can initiate outbound requests (`Notify`, `Put`, `Query`, `DiscoverVersions`, `SetEndpointRole`) to test client listeners live across TTLV, JSON, and XML.
9. **`REQ-066-009` (Four-Language Parity)**:
   Rust, C, Java, and Python provide equivalent listener, dispatcher, and typed event accessors with zero secret leakage.
