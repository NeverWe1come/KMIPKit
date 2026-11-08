# Implementation Plan: Server-to-Client Operations and Role Inversion (KMIPKIT-0066)

## 1. Architectural Strategy

The implementation advances the OASIS KMIP 2.1 Server-to-Client architecture into `KMIPKit` across six functional phases:
1. **Protocol Modeling**: Strongly-typed structures in `kmipkit-protocol` for §§6.1.54, 6.2.1–6.2.5 with `AttributeSet`, `ManagedObject`, secret zeroization, and canonical response building.
2. **Transport Listeners**: `RawTlsListener` and `HttpsListener` in `kmipkit-transport` supporting mutual TLS 1.3, credential hot-reload, and wire decoding (TTLV, JSON, XML).
3. **Client Orchestration**: `ClientListener` and `ServerRequestDispatcher` in `kmipkit-client` exposing both pull-based batch inspection and high-level handler execution.
4. **Active Test Server S2C Trigger**: Extension of `crates/kmipkit-test-server` to actively push requests to client endpoints.
5. **Multi-Language Bindings**: C ABI (`kmipkit-ffi`), Java 17 (`bindings/java`), and Python 3.12 (`bindings/python`).
6. **Documentation and Release Evidence**: Manifest reconciliation and bilingual user guides.

## 2. Test-Driven Development (TDD) Strategy

Strict Red $\to$ Green $\to$ Refactor cycle:
- Phase 1: Unit tests for protocol model encoding/decoding, negative tests (`MUST NOT` / `SHALL NOT`), and secret drop zeroization.
- Phase 2: Unit and integration tests for raw TLS and HTTPS listeners with valid and invalid mTLS credentials.
- Phase 3: Dispatcher tests for error continuation (`Continue`, `Stop`, `Undo`), extension criticality, and maximum response size enforcement.
- Phase 4: End-to-end integration tests between `kmipkit-test-server` and `KMIPKit` client listeners over TLS 1.3.
- Phase 5: Language binding consumer tests in C, Java, and Python.
