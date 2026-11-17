# Feature Specification: Python Fluent Cryptographic API (KMIPKIT-0105)

**Status**: Approved delivery slice under the maintainer's 2026-11-11
authorization to complete KMIPKIT-0093.
**Owning identifier**: KMIPKIT-0105
**Parent contract**: KMIPKIT-0093; Rust API KMIPKIT-0102; C ABI KMIPKIT-0103.
**Branch**: `feature/KMIPKIT-0105-python-fluent-crypto-api`
**Base**: KMIPKIT-0104 Java fluent crypto API at `cfbf6267828ab5b8407eb380ecf782553f086df6`.

## User intent

A Python developer can build and execute the seven supported KMIP 2.1
cryptographic requests from `Client.request()` without constructing numeric
operation codes or a manual batch. Setters remain operation-specific and all
cryptographic choices are supplied by the caller.

## Normative and architectural sources

- OASIS KMIP Specification v2.1 §4.16, Table 59 (Cryptographic Parameters).
- OASIS KMIP Specification v2.1 §6.1.11, Table 196 (Decrypt).
- OASIS KMIP Specification v2.1 §6.1.17, Table 214 (Encrypt).
- OASIS KMIP Specification v2.1 §6.1.24, Table 235 (Hash).
- OASIS KMIP Specification v2.1 §6.1.32, Table 259 (MAC).
- OASIS KMIP Specification v2.1 §6.1.33, Table 262 (MAC Verify).
- OASIS KMIP Specification v2.1 §6.1.55, Table 334 (Sign).
- OASIS KMIP Specification v2.1 §6.1.56, Table 337 (Signature Verify).
- OASIS KMIP Specification v2.1 §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38, and
  7.39 for operation-specific data and multipart fields; §§8–9 for request
  headers, batch execution, and response status.
- KMIPKIT-0093-FR-004–011, KMIPKIT-0102-FR-001–007,
  KMIPKIT-0103-FR-001–006, KMIPKIT-0104-FR-001–005, ADR-0012, and ADR-0018.

## Requirements

- **KMIPKIT-0105-FR-001**: `Client.request()` MUST offer Python snake_case
  entry points for Encrypt, Decrypt, Sign, Signature Verify, MAC, MAC Verify,
  and Hash, each returning an operation-specific builder in
  `kmipkit.fluent`.
- **KMIPKIT-0105-FR-002**: Each builder MUST expose only the semantic inputs
  valid for its operation, following the approved Rust/C API field matrix.
  Callers MUST explicitly provide key identifiers and Cryptographic
  Parameters where the operation requires them. Data, IV/nonce, AAD, tag,
  digest, signature, MAC, and multipart indicators MUST NOT be inferred.
- **KMIPKIT-0105-FR-003**: Builders MUST accept typed
  `CryptographicParameters` or an explicitly ordered TTLV Structure. A typed
  Data value MAY be supplied through the expert API where the underlying
  builder supports it.
- **KMIPKIT-0105-FR-004**: Python byte input MUST be copied synchronously into
  the existing Rust-owned builder storage. KMIPKit MUST zeroize temporary
  mutable staging buffers it creates. Documentation MUST state that immutable
  caller objects and Python runtime copies are outside deterministic
  zeroization control.
- **KMIPKIT-0105-FR-005**: Builders MUST support explicit `build()` /
  `to_batch_item()` and one-exchange `execute()`, MUST be context managers,
  MUST release native handles on close, and MUST reject use after close.
  Execution MUST preserve the existing typed batch result, request options,
  Request Header asynchronous indicator, status, correlation, and delivery
  semantics; it MUST NOT retry or poll automatically.
- **KMIPKIT-0105-FR-006**: The Python facade MUST use the existing C ABI and
  configured client/codec limits. It MUST NOT add a transport, send arbitrary
  TTLV messages, change the C ABI, or alter existing Python entry points.
- **KMIPKIT-0105-FR-007**: The package MUST provide public PEP 561 type
  information for each new builder and request method. New public declarations
  MUST be added to the source API manifest, regenerated with the pinned tool,
  and linked to this specification.

## User stories

### US1 — Construct a cryptographic request (P1)

As a Python client developer, I can select one of the seven operation-specific
builders and set only its supported fields so malformed or incomplete
requests fail before transport.

**Independent acceptance**: consumer tests construct each operation, check
the operation identity and field placement, and verify invalid or missing
inputs are rejected locally.

### US2 — Execute and manage native ownership (P1)

As a Python client developer, I can execute one typed exchange with explicit
options and inspect the existing batch result while the builder's native
storage is released deterministically.

**Independent acceptance**: a TLS 1.3 mutual-TLS fixture decodes each operation
request, returns a real encoded KMIP response, and the test observes the
matching response and one exchange; close, exception, and use-after-close
paths are covered.

### US3 — Use and discover the API safely (P2)

As a Python developer, I can use type information and bilingual examples to
understand operation fields, buffer ownership, result states, and limitations.

**Independent acceptance**: Python export audit matches the manifest, package
type information includes the new API, and both guides explain caller-owned
copies and the absence of automatic Poll/retry.

## Success criteria

- A Python consumer can compile and build all seven request types without a
  numeric Operation value or a manually assembled batch.
- All seven typed builders complete one request/response exchange against the
  controlled TLS 1.3 mutual-TLS fixture and preserve the expected response.
- Negative tests prove operation-specific setters, required fields, invalid
  parameters, closed handles, and invalid indicators are rejected before
  network I/O.
- No new C symbols are introduced, and all public Python declarations are
  represented by the generated API manifest and exact export audit.

## Exclusions

This slice does not add KEM, KMIP 3.0-only operations, server-initiated
operations, arbitrary raw-message submission, a local cryptographic provider,
Poll helpers, retries, or new Rust/C ABI behavior. These remain part of the
broader KMIPKIT-0093 plan and are not implicitly completed by this slice.
