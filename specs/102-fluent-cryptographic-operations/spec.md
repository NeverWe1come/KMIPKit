# Feature Specification: Fluent Cryptographic Operations (KMIPKIT-0102)

**Status**: Approved implementation slice of KMIPKIT-0093 under the
maintainer's 2026-11-11 instruction to implement the complete plan.
**Owning identifier**: KMIPKIT-0102
**Parent contract**: KMIPKIT-0093, especially FR-004 through FR-011.
**Implementation scope**: Rust client facade only.

## User intent

A Rust application developer should be able to construct, batch, validate, and
send the common cryptographic KMIP operations through semantic builders while
retaining access to every field supported by the existing typed request
models. The caller explicitly chooses cryptographic parameters and supplies
all operation data. KMIPKit transports and validates requests; it does not
perform local cryptographic operations.

## Normative sources

- KMIP Specification v2.1, §6.1.11, Table 196 (Decrypt).
- KMIP Specification v2.1, §6.1.17, Table 214 (Encrypt).
- KMIP Specification v2.1, §6.1.24, Table 235 (Hash).
- KMIP Specification v2.1, §6.1.32, Table 259 (MAC).
- KMIP Specification v2.1, §6.1.33, Table 262 (MAC Verify).
- KMIP Specification v2.1, §6.1.55, Table 334 (Sign).
- KMIP Specification v2.1, §6.1.56, Table 337 (Signature Verify).
- KMIP Specification v2.1, §4.16 and Table 59 (Cryptographic Parameters).
- KMIP Specification v2.1, §§7.8, 7.9, 7.14, 7.17, 7.20, 7.38, and 7.39
  (operation data and multipart fields).
- Approved KMIPKIT-0093 requirements FR-004 through FR-011.

The operation inventory and existing typed request/response models determine
supported direction, request shape, and version gates. The user guide and
examples are informative and do not add protocol requirements.

## Requirements

- **KMIPKIT-0102-FR-001**: `RequestBuilder` MUST provide semantic constructors
  for Encrypt, Decrypt, Sign, Signature Verify, MAC, MAC Verify, and Hash.
  Each builder MUST construct the corresponding existing typed request and
  batch item, preserve its KMIP 2.1 client-to-server version gate, and execute
  through the existing one-exchange typed client route.
- **KMIPKIT-0102-FR-002**: Every builder MUST expose all request fields offered
  by its typed request model. It MUST preserve optional-field omission, caller
  field values, order, and multipart framing. It MUST provide typed
  `CryptographicParameters` input and an expert path for a structurally valid
  ordered `Structure` when the model supports parameters.
- **KMIPKIT-0102-FR-003**: The builder API MUST require caller-supplied data and
  parameters according to the existing request model and local validation
  rules. It MUST NOT select an algorithm, key length, mode, padding, hash,
  IV/nonce, authenticated-encryption tag length, or key identifier
  implicitly.
- **KMIPKIT-0102-FR-004**: Byte inputs representing operation data, IV/nonce,
  MAC, signature, correlation value, or authenticated-encryption fields MUST
  move into the existing zeroizing `SecretBytes` owner without hidden copies
  by KMIPKit. Debug, Display, and validation failures MUST redact their
  contents. Any caller-side or third-party copies remain outside KMIPKit's
  zeroization guarantee.
- **KMIPKIT-0102-FR-005**: `build` MUST run the typed request's current local
  validation before producing a sendable request. Invalid single-part or
  multipart shapes MUST fail before transport with `NotSent` delivery
  evidence. No builder or execution path may retry or poll automatically.
- **KMIPKIT-0102-FR-006**: Builders MUST expose exchange options and Request
  Header fields through their existing separate APIs. `with_options` MUST
  contain only real exchange options; authentication and header fields MUST
  not be represented as exchange options.
- **KMIPKIT-0102-FR-007**: Tests MUST prove exact typed request and batch
  conversion, negative validation, redaction, version gating, one-exchange
  execution, and Success/Pending/Failure propagation for the implemented
  operations. Requirement traceability MUST cite the exact pinned KMIP v2.1
  section/table and the parent KMIPKIT-0093 requirement.

## Explicit exclusions

- KMIP 3.0 Encapsulate and Decapsulate are a separate delivery with explicit
  KMIP 3.0 selection and draft-catalog validation.
- Derive Key, RNG operations, Key Agreement, and all non-cryptographic
  operations are outside this slice.
- C, Java, and Python facades are later slices; this Rust-only PR does not
  claim four-language parity for these operations.
- No local cryptographic algorithms, operation presets, server-side policy
  emulation, automatic polling, or retry behavior is added.

## Acceptance

- All seven constructors exist and each builds its existing typed request.
- Caller-selected parameters, optional fields, and secret byte strings remain
  exact; no TTLV tag or KMIP operation integer is required from callers.
- Locally invalid requests are rejected before sending; valid requests execute
  once through the existing typed route and preserve their typed outcome.
- Focused tests, full client tests, traceability, generated API exports,
  formatting, Clippy, coverage, and exact-head CI pass before the `[WIP]` tag
  is removed.
