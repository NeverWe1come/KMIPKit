# Feature Specification: Configurable Discover Versions Offers (KMIPKIT-0113)

**Feature Branch**: `feature/KMIPKIT-0113-discover-versions-offers`
**Created**: 2026-11-13
**Status**: Draft for maintainer review; implementation is not yet authorized
**Base**: `release/1.0.0` at `9a16cc90a12205101d22a38501a5e74c665df4dc`
**Scope owner**: Typed client-to-server Discover Versions request construction

## 1. Purpose

Allow callers to announce KMIP 2.0 alongside KMIP 2.1 in a Discover Versions
request, while keeping KMIP 2.1 as the default offer and default client
protocol version. Expose the same offer-list behavior through the public Rust,
C, Java, and Python client APIs. This enables the interop validators to ask a
2.0 or 2.1 server what versions it supports using the same installed library
package.

The request Message Header version and the Discover Versions payload are
independent. The configured or per-request client protocol version continues
to determine the header. An offered-version list only describes the client's
preference list in the Discover Versions payload. This feature does not
negotiate a version, change client configuration, retry a request, or
implicitly change any later request.

## 2. Normative References and Pinned Evidence

- **OASIS KMIP Specification v2.0**, §6.1.16, Tables 196 and 197. The request's
  Protocol Version field is optional and repeatable; supplied versions are
  ordered by decreasing preference. A successful response lists server
  versions and, when a list was supplied, is limited to the client/server
  intersection. Pinned source: `specification/oasis/kmip-2.0/upstream/kmip-spec-v2.0.html`,
  SHA-256 `41346355fc966e555dbcbae68b4d73421b0da0f32fb1a86fc8e3cb5fd35bb4a2`.
- **OASIS KMIP Specification v2.1**, §6.1.16, Tables 211 and 212. The same
  optional, repeatable request field and response intersection rules apply.
  Pinned source: `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`,
  SHA-256 `8bf9d914c097e98a6509aa1ffcbf03406f738066e940597aee93d0a5e07addcf`.
- **KMIPKIT-0007**, `KMIPKIT-0007-FR-002`, and **KMIPKIT-0069**, `REQ-069-006`.
  This feature supersedes only the KMIPKIT-0007 restriction that every request
  advertise KMIP 2.1 alone. Its default remains unchanged.
- **ADR-0022**, §§3–6: KMIP 2.1 remains the default; version selection is
  explicit; catalogued versions and four-language parity are supported.
- **AGENTS.md**, §§8–10: no automatic retries, generated artifacts are not
  edited by hand, and public APIs remain additive and language-equivalent.

The exact upstream OASIS source copies are immutable. This feature adds no
vendor extension, KMIP 1.x support, or claim of KMIP 3.0 server
interoperability.

## 3. User Scenarios & Testing

### User Story 1 — Offer KMIP 2.1 and 2.0 to a server (Priority: P1)

An interop-validator user selects Discover Versions and supplies an ordered
list containing KMIP 2.1 and KMIP 2.0. The client sends the values as repeated
Protocol Version structures in that order, and the caller can inspect the
server's supported-version response.

**Why this priority**: This is the requested capability needed to use one
validator package against KMIP 2.0 and 2.1 servers.

**Independent Test**: A controlled peer captures a request offering
`[(2,1), (2,0)]`, returns either the offered intersection or an empty list,
and verifies the typed response preserves the server's exact order.

**Acceptance Scenarios**:

1. **Given** a caller offers `(2,1)` then `(2,0)`, **When** the request is
   encoded, **Then** the Protocol Version structures appear in precisely that
   preference order.
2. **Given** the request header is configured as `(2,0)` and the offer list is
   `[(2,1), (2,0)]`, **When** the message is sent, **Then** the header remains
   `(2,0)` and the payload carries both offers in order.
3. **Given** the server returns an empty list or an ordered subset of the
   caller's explicit offers, **When** the response is converted, **Then** the
   list and duplicates are preserved without inference about other
   operations.

### User Story 2 — Keep existing calls unchanged (Priority: P1)

A caller continues using the existing no-argument Discover Versions builder
and receives the same KMIP 2.1-only payload and configured header behavior.

**Independent Test**: Compare the no-argument request's normalized TTLV tree
and deterministic bytes with the pre-feature KMIP 2.1 request.

**Acceptance Scenarios**:

1. **Given** existing code calls the no-argument builder, **When** it sends
   Discover Versions, **Then** it advertises only `(2,1)` and uses the
   independently configured message-header version.
2. **Given** a caller uses KMIP 2.1 configuration without an offer override,
   **When** it sends any existing operation, **Then** the feature has not
   changed that operation or its header.

### User Story 3 — Omit the optional offer field (Priority: P2)

A caller deliberately asks the server to return its supported version list
without sending a Protocol Version item in the request payload.

**Independent Test**: A controlled peer asserts the request payload has no
Protocol Version child and returns multiple supported versions, including a
well-formed version not in the library's current catalog; the typed response
preserves the list.

**Acceptance Scenarios**:

1. **Given** the explicit no-offers builder is used, **When** the request is
   encoded, **Then** the Protocol Version request field is absent.
2. **Given** the server returns a valid version list after no offers were
   supplied, **When** the response is converted, **Then** every well-formed
   version is retained in server order, including repeated and uncatalogued
   values.

### Edge Cases

- Reject an explicit empty list and any caller-offered version that is not in
  the compiled KMIPKit version catalog; use the explicit no-offers builder to
  omit the optional field.
- Reject malformed response Protocol Version structures, even when no offers
  were supplied.
- When offers were supplied, reject a successful response containing a
  version outside the offered list, while preserving a valid empty response.
- Preserve caller preference order and repeated offered entries. Do not sort
  or deduplicate them.
- Reject invalid list pointer/count pairs at the C boundary without reading
  past caller-provided memory.
- Bound encoding by the existing message and codec limits; add no unbounded
  allocation or silent truncation.

## 4. Functional Requirements

- **KMIPKIT-0113-FR-001 — Explicit preference list**: The public typed client
  API MUST allow a caller to provide one or more catalogued protocol versions
  for Discover Versions. It MUST preserve the values, their order, and
  repeated values. The caller's preference ordering is not rewritten by
  comparing numeric version values. Source: OASIS KMIP Specification v2.0
  §6.1.16, Table 196; v2.1 §6.1.16, Table 211.
- **KMIPKIT-0113-FR-002 — KMIP 2.0 availability**: The API MUST accept
  `ProtocolVersion(2,0)` and allow it to be sent alone or with `(2,1)` in one
  Discover Versions offer list. A mixed list `[(2,1),(2,0)]` MUST be
  encodable in the specified order. Source: OASIS KMIP Specification v2.0
  §6.1.16, Table 196; v2.1 §6.1.16, Table 211; ADR-0022 §1.
- **KMIPKIT-0113-FR-003 — No-argument compatibility**: Existing no-argument
  Discover Versions builders MUST continue to advertise only `(2,1)`. The
  default client protocol version MUST remain `(2,1)`. Source: ADR-0022 §3;
  KMIPKIT-0007-FR-002 as superseded only for explicit offer lists.
- **KMIPKIT-0113-FR-004 — Omitted offer list**: Each language MUST expose an
  explicit way to omit the optional Protocol Version request field. This MUST
  remain distinct from the backward-compatible no-argument builder. When the
  field is omitted, a successful response is not restricted to an offered
  intersection; all well-formed returned version pairs MUST be preserved in
  server order, including unknown catalog values. Source: OASIS KMIP
  Specification v2.0 §6.1.16, Tables 196–197; v2.1 §6.1.16, Tables 211–212.
- **KMIPKIT-0113-FR-005 — Offered-list validation**: An explicit list MUST be
  non-empty and contain only versions recognized by KMIPKit's compiled
  version catalog. A successful response to that request MUST contain only
  versions present in the explicit list. Malformed entries and unoffered
  response values MUST produce typed, redacted errors. Source: OASIS KMIP
  Specification v2.0 §6.1.16; v2.1 §6.1.16; KMIPKIT-0069 `REQ-069-005`.
- **KMIPKIT-0113-FR-006 — Header independence**: The message-header protocol
  version MUST continue to come from the client configuration or explicit
  per-request override. Selecting or omitting an offer list MUST NOT change
  the header, client configuration, or version used by subsequent requests.
  Source: ADR-0022 §§3–4; OASIS KMIP Specification v2.0 §9.16 and v2.1 §9.16.
- **KMIPKIT-0113-FR-007 — Four-language parity**: Rust, C, Java, and Python
  MUST provide equivalent behavior for default offers, explicit ordered
  offers, no offers, input validation, and response validation. Public API
  declarations and generated bindings MUST be updated through their pinned
  generators.
- **KMIPKIT-0113-FR-008 — No implicit negotiation**: Receiving supported
  versions MUST NOT alter client or request configuration, trigger a retry,
  or select a version for later operations. Any subsequent use of a different
  version MUST remain an explicit caller action. Source: ADR-0022 §§4–5 and
  AGENTS.md §8.

## 5. Success Criteria

- **KMIPKIT-0113-SC-001**: All four language APIs emit equivalent normalized
  Discover Versions trees for default, explicit `[(2,1),(2,0)]`, explicit
  `[(2,0)]`, and omitted-field construction.
- **KMIPKIT-0113-SC-002**: The existing default request remains byte-identical
  under deterministic TTLV inputs and retains its KMIP 2.1-only offer.
- **KMIPKIT-0113-SC-003**: Controlled-peer tests pass for offered
  intersections, empty intersections, no-offer responses, duplicate values,
  malformed fields, and unoffered response values.
- **KMIPKIT-0113-SC-004**: CI validates generated API manifests, headers and
  wrappers, and reports no difference in the established client default or
  unrelated operation behavior.

## 6. Assumptions and Exclusions

- KMIPKit's compiled version catalog remains authoritative for values a client
  may offer. Existing catalogued versions such as KMIP 3.0 retain their
  existing library status; this feature does not claim interoperability with
  KMIP 3.0 servers.
- KMIP 2.0 and 2.1 are the live interop test versions. KMIP 1.x and vendor
  extensions remain out of scope.
- No automatic version negotiation, downgrade, retry, server capability
  cache, or change to client defaults is introduced.
- The server's SHOULD-level completeness of its returned supported-version
  list is reported as an observation; KMIPKit cannot assert that a server
  listed every version it supports.

## 7. Traceability

| Requirement | Normative or project source | Verification |
|---|---|---|
| FR-001 | OASIS v2.0 §6.1.16 Table 196; v2.1 §6.1.16 Table 211 | Rust unit and four-language normalized request-tree tests |
| FR-002 | OASIS v2.0 §6.1.16 Table 196; v2.1 §6.1.16 Table 211; ADR-0022 §1 | Explicit 2.0 and `[2.1,2.0]` controlled-peer tests |
| FR-003 | ADR-0022 §3; supersedes only KMIPKIT-0007-FR-002's single-offer restriction | Existing default golden vectors and package consumer tests |
| FR-004 | OASIS v2.0 §6.1.16 Tables 196–197; v2.1 §6.1.16 Tables 211–212 | Omitted-field request and unknown-version response tests |
| FR-005 | OASIS v2.0/v2.1 §6.1.16; KMIPKIT-0069 REQ-069-005 | Empty/uncatalogued offer rejection, malformed response, and unoffered-value tests |
| FR-006 | ADR-0022 §§3–4; OASIS v2.0/v2.1 §9.16 | Header and payload independence tests for 2.0 and 2.1 |
| FR-007 | Constitution Core Principle III; ADR-0022 §6 | Generated API parity audit and external-package consumer tests |
| FR-008 | ADR-0022 §§4–5; AGENTS.md §8 | Follow-up request verifies no implicit change, negotiation, or retry |

## 8. Human Approval Gate

This document is a draft proposal prepared at the maintainer's direction. It is
not an approved specification and does not authorize library implementation.
Implementation starts only after maintainer approval is recorded in this
feature directory. No approval or merge is performed by the author of this
draft.
