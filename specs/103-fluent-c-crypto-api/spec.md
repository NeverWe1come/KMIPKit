# Feature Specification: Fluent C Cryptographic Operations (KMIPKIT-0103)

**Status**: Approved implementation slice of KMIPKIT-0093 under the
maintainer's 2026-11-11 instruction to implement the complete fluent API plan.
**Owning identifier**: KMIPKIT-0103
**Parent contract**: KMIPKIT-0093 and Rust API slice KMIPKIT-0102.
**Implementation scope**: C ABI and C consumer surface for cryptographic operations.

## User intent

A C programmer should be able to configure and execute the common cryptographic
operations through semantic KMIPKit functions without constructing operation
payload tags or operation-number batches. KMIPKit transports caller-provided
data and does not perform local cryptographic algorithms or choose parameters.

## Normative and architectural sources

- OASIS KMIP Specification v2.1 §6.1.11, Table 196 (Decrypt).
- OASIS KMIP Specification v2.1 §6.1.17, Table 214 (Encrypt).
- OASIS KMIP Specification v2.1 §6.1.24, Table 235 (Hash).
- OASIS KMIP Specification v2.1 §6.1.32, Table 259 (MAC).
- OASIS KMIP Specification v2.1 §6.1.33, Table 262 (MAC Verify).
- OASIS KMIP Specification v2.1 §6.1.55, Table 334 (Sign).
- OASIS KMIP Specification v2.1 §6.1.56, Table 337 (Signature Verify).
- OASIS KMIP Specification v2.1 §4.16 and Table 59 (Cryptographic Parameters).
- OASIS KMIP Specification v2.1 §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38, and 7.39
  (operation data and multipart fields).
- KMIPKIT-0093-FR-004–011, KMIPKIT-0102-FR-001–007, ADR-0012, and ADR-0018.
- Existing typed C ABI request, response, secret ownership, and client execution
  contracts.

## Requirements

- **KMIPKIT-0103-FR-001**: The C API MUST provide operation-specific opaque
  builders for Encrypt, Decrypt, Sign, Signature Verify, MAC, MAC Verify, and
  Hash. Constructors and setters MUST use semantic names and fixed-width C
  types; callers MUST NOT need KMIP operation numbers or manually assemble a
  batch to execute these single-operation requests.
- **KMIPKIT-0103-FR-002**: Each builder MUST expose every field supported by
  its corresponding KMIPKIT-0102 typed builder. Cryptographic parameters MUST
  be caller-supplied through the typed Cryptographic Parameters handle or an
  explicitly ordered Structure. Byte inputs MUST have named, operation-specific
  setters, and optional fields MUST remain absent unless supplied.
- **KMIPKIT-0103-FR-003**: Byte inputs MUST be synchronously copied into
  KMIPKit-owned zeroizing storage before the setter returns. Caller buffers
  remain caller-owned. No secret data may appear in logs, errors, or formatted
  diagnostics. The builder MUST NOT choose key identifiers, algorithms, sizes,
  modes, padding, hashes, IVs/nonces, or authentication-tag values implicitly.
- **KMIPKIT-0103-FR-004**: Builders MUST produce the matching existing typed
  client batch item and execute through the existing one-exchange Client path.
  Execution MUST preserve typed Success, Pending, Failure, and delivery state,
  MUST NOT retry or poll automatically, and MUST preserve the current
  pre-transport protocol-version checks. A builder MUST NOT accept an arbitrary
  message TTLV for network submission.
- **KMIPKIT-0103-FR-005**: Exchange-local timeout options and Request Header
  asynchronous indicators MUST be separate inputs. Existing fixed-width status
  codes, opaque-handle ownership, panic containment, and release conventions
  MUST be preserved. Invalid pointers, sizes, indicators, and state MUST fail
  without modifying output handles.
- **KMIPKIT-0103-FR-006**: Every exported declaration MUST be recorded in
  `specification/api/public-api.json`, regenerated with the pinned API tool,
  documented in the C guide, and linked to stable traceability IDs. Existing C
  symbols remain source and ABI compatible.

## Acceptance criteria

1. Compiled C consumer tests construct all seven typed operation items and
   verify their operation identities without creating raw operation payloads.
2. Negative tests cover null and malformed inputs, missing required parameters,
   invalid multipart flags, repeated setters where replacement is ambiguous,
   and wrong builder kinds. All failures occur before transport.
3. Controlled TLS fixture tests exercise the seven operation routes and prove
   one request per execute call, typed outcome access, protocol-version
   validation, and absence of retries or automatic polling.
4. Tests verify data ownership and redaction contracts and verify option/header
   separation.
5. Manifest generation, public C export audit, C ABI and consumer tests,
   focused Rust tests, traceability, documentation checks, formatting, Clippy,
   coverage, and exact-head PR CI pass before the `[WIP]` title is removed.

## Explicit exclusions

This C-only delivery does not add KEM operations, KMIP 3.0 Encapsulate or
Decapsulate, asynchronous Poll helpers, automatic retries, local cryptographic
algorithms, or arbitrary TTLV message submission. Those remain governed by
their approved operation and version specifications. Java and Python facades
are delivered in dependent slices.
