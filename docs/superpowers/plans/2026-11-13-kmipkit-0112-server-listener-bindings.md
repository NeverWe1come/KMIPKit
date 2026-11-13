# KMIPKIT-0112 Java and Python server-listener facades

## Goal

Complete the Java and Python mappings for approved server-to-client listener
operations under KMIPKIT-0093-FR-011 and FR-012. Keep TLS, framing, decoding,
typed request ownership, response validation, and response encoding in the
existing Rust/C ABI path. The bindings must not add a client-side `send()` for
server-originated request objects.

## Implementation sequence

1. Add failing Java consumer and Python API-shape tests for listener creation,
   bounded accept/read, typed request dispatch for Discover Versions, Notify,
   Put, Query, and Set Endpoint Role, typed response construction, delivery,
   timeout behavior, and deterministic handle release.
2. Open a `[WIP]` draft PR with these tests and the implementation plan, based
   on the existing KMIPKIT-0106 listener ABI head.
3. Add JNI and CFFI marshalling for the already-public server-listener ABI,
   including all versioned configuration fields and opaque-handle ownership.
4. Add idiomatic Java and Python listener, connection, batch, typed request,
   handler, and typed response facades. Preserve caller-owned timeout and
   codec-limit choices; never retry or log request bodies or key material.
5. Add local TLS 1.3/mTLS fixture tests that bind the library listener, accept
   a real server-initiated encoded request, dispatch through the typed handler,
   and verify the encoded typed response. Include malformed input and handler
   failure cases.
6. Update public API declarations, generated exports, user documentation,
   bilingual examples, and KMIPKIT-0093 task traceability through repository
   generators.
7. Run Java/JNI and Python suites, Rust/FFI checks, sanitizers, coverage,
   manifest/export checks, and full PR CI. Keep `[WIP]` until every required
   check on the exact head succeeds, then mark ready for human review.

## Acceptance checks

- Both bindings can configure and bind raw TLS and HTTPS using the existing
  ABI's validated TLS 1.3/mTLS rules.
- Each of the five KMIP 2.1 server-to-client operations is surfaced as a typed
  request and can only be answered through a typed response accepted by the
  request's response builder.
- Java `AutoCloseable` and Python context-manager/`close()` paths release each
  native handle exactly once, including exceptions and timeouts.
- Fixture tests prove request and response encoding on a real local TLS
  exchange; synthetic model-only tests are supplementary.
- The new bindings have no client request `.send()` method for these
  server-originated operations and preserve public API manifest parity.

## Source of authority

This slice implements the already approved KMIPKIT-0093-FR-011 and FR-012.
Normative request/response shapes come from OASIS KMIP v2.1 §§6.2.1–6.2.5 and
the approved KMIPKIT-0066/ADR-0021 listener path. It does not claim that a
successful local fixture proves external server interoperability or profile
conformance.
