# Implementation Tasks: Server-to-Client Operations and Role Inversion (KMIPKIT-0066)

- [ ] `TASK-066-001`: Governance, ADR-0021, and constitution amendment 1.2.0.
- [ ] `TASK-066-002`: `kmipkit-protocol` Set Endpoint Role models (§6.1.54, §6.2.5).
- [ ] `TASK-066-003`: `kmipkit-protocol` Server-to-Client Discover Versions (§6.2.1).
- [ ] `TASK-066-004`: `kmipkit-protocol` Server-to-Client Notify (§6.2.2) with AttributeSet.
- [ ] `TASK-066-005`: `kmipkit-protocol` Server-to-Client Put (§6.2.3) with secret zeroization.
- [ ] `TASK-066-006`: `kmipkit-protocol` Server-to-Client Query (§6.2.4) and ServerResponseBuilder.
- [ ] `TASK-066-007`: `kmipkit-transport` ListenerTransportConfig, RawTlsListener, and HttpsListener.
- [ ] `TASK-066-008`: `kmipkit-transport` RawTlsRoleReversedStream.
- [ ] `TASK-066-009`: `kmipkit-client` Client direct methods (#57 SetEndpointRole) and RoleReversedSession.
- [ ] `TASK-066-010`: `kmipkit-client` ServerRequestDispatcher and ClientListener.
- [ ] `TASK-066-011`: `kmipkit-test-server` active S2C client trigger and push endpoints.
- [ ] `TASK-066-012`: End-to-end integration tests (ClientListener <-> test-server across TTLV, JSON, XML).
- [ ] `TASK-066-013`: `kmipkit-ffi` C ABI symbols for listeners, role reversal, and S2C requests.
- [ ] `TASK-066-014`: Java 17 JNI bindings and integration tests.
- [ ] `TASK-066-015`: Python 3.12 CFFI bindings and integration tests.
- [ ] `TASK-066-016`: Public API manifest reconciliation and bilingual documentation updates.
