# Feature Specification: Java Fluent Cryptographic Operations (KMIPKIT-0104)

**Status**: Approved delivery slice of KMIPKIT-0093 under the maintainer's
2026-11-11 instruction to complete the fluent API plan.
**Owning identifier**: KMIPKIT-0104
**Parent contract**: KMIPKIT-0093; Rust operation API KMIPKIT-0102; C ABI
KMIPKIT-0103.
**Implementation scope**: idiomatic Java facade over the approved C/Rust
cryptographic-operation builders.

## User intent

A Java developer should build and execute the seven approved KMIP 2.1
cryptographic requests with semantic methods and without constructing KMIP
operation tags or manual batches. Rust remains the owner of request assembly,
validation, and secret storage; Java supplies an idiomatic facade over the
existing opaque C builder ABI.

## Normative and architectural sources

- OASIS KMIP Specification v2.1 §6.1.11, Table 196 (Decrypt).
- OASIS KMIP Specification v2.1 §6.1.17, Table 214 (Encrypt).
- OASIS KMIP Specification v2.1 §6.1.24, Table 235 (Hash).
- OASIS KMIP Specification v2.1 §6.1.32, Table 259 (MAC).
- OASIS KMIP Specification v2.1 §6.1.33, Table 262 (MAC Verify).
- OASIS KMIP Specification v2.1 §6.1.55, Table 334 (Sign).
- OASIS KMIP Specification v2.1 §6.1.56, Table 337 (Signature Verify).
- OASIS KMIP Specification v2.1 §4.16, Table 59 (Cryptographic Parameters);
  §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38, and 7.39 (operation data and multipart
  fields).
- KMIPKIT-0091-FR-009–016, KMIPKIT-0093-FR-004–011,
  KMIPKIT-0102-FR-001–007, KMIPKIT-0103-FR-001–006, ADR-0012, and ADR-0018.

## Requirements

- **KMIPKIT-0104-FR-001**: `FluentRequest` MUST expose operation-specific
  Java builders for Encrypt, Decrypt, Sign, Signature Verify, MAC, MAC Verify,
  and Hash using camelCase entry points in `org.kmipkit.fluent`.
- **KMIPKIT-0104-FR-002**: Each builder MUST expose exactly the semantic fields
  of its corresponding KMIPKIT-0102 builder through the KMIPKIT-0103 opaque C
  builder. Cryptographic parameters MUST be explicitly supplied as a typed
  `CryptographicParameters` or ordered `TtlvStructure`; operation-specific
  inputs MUST remain operation-specific. The facade MUST NOT choose algorithm,
  key identifier, IV, nonce, mode, padding, hash, tag, or usage implicitly.
- **KMIPKIT-0104-FR-003**: Byte setters MUST synchronously transfer data to the
  C builder, which copies secret bytes into Rust-owned zeroizing memory before
  returning. JNI temporary copies MUST be cleared on every exit path where the
  runtime permits it; Java runtime copies and caller-owned arrays MUST be
  documented as outside KMIPKit's zeroization control.
- **KMIPKIT-0104-FR-004**: Builders MUST be `AutoCloseable`, MUST reject use
  after close, MUST produce the corresponding typed `ClientBatchItem`, and
  MUST execute at most one exchange using existing `KmipClient` paths. Build
  and execution MUST preserve Success, Pending, Failure, correlation, and
  delivery semantics. Polling and retries MUST NOT occur automatically.
- **KMIPKIT-0104-FR-005**: Exchange-local `RequestOptions` and the Request
  Header asynchronous indicator MUST remain separate. Existing Java public
  declarations MUST remain compatible. Every new public declaration MUST be
  added to the API manifest source, regenerated with the pinned tool, and
  linked to this feature's traceability.

## Acceptance criteria

1. Java consumer tests compile all seven operation builders and verify their
   operation identities, supported setter surfaces, close behavior, and
   malformed-input rejection.
2. JNI lifetime tests cover native handle ownership, builder use after close,
   empty and non-empty byte arrays, Java caller-buffer ownership, and secret
   clearing on success and failure paths.
3. Each builder builds a typed batch item and executes one request against a
   controlled TLS fixture. The fixture checks the operation and encoded fields;
   the Java facade exposes typed outcomes without retries or polling.
4. Option/header separation, explicit asynchronous values, protocol-version
   rejection before transport, and the existing Java API compatibility tests
   pass.
5. Manifest generation, Java/JNI tests, native C++ tests, Rust and C consumer
   tests, coverage, traceability, formatting, linting, and exact-head PR CI
   pass before removing `[WIP]`.

## Explicit exclusions

This slice does not add C, Python, KEM, KMIP 3.0 Encapsulate/Decapsulate,
server-initiated operations, arbitrary TTLV message submission, local
cryptographic algorithms, Poll helpers, or automatic retry behavior. Those
remain governed by their own approved delivery slices.
