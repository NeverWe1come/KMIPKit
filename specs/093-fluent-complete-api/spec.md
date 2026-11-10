# Feature Specification: Complete Fluent API Contract (KMIPKIT-0093)

**Branch**: `feature/KMIPKIT-0093-fluent-master-contract`
**Base**: `feature/KMIPKIT-0091-fluent-dx-implementation` at `a61c5aab3aaa9616e78a4798ee39bad3c8dcb1fe`
**Status**: Approved in scope by the maintainer's direct 2026-11-11 instruction; implementation and source mapping remain subject to PR review.
**Owning identifier**: KMIPKIT-0093

## Purpose

Complete a fluent, operation-oriented API in Rust, C, Java, and Python for every
assigned operation already supported by the project's versioned protocol
catalog. An application can choose an operation, fill its semantic fields,
validate it locally, execute one exchange where its direction permits, and
inspect its typed outcome without constructing a KMIP batch. The existing
procedural and typed APIs remain supported and interoperable.

This specification supersedes the Fluent DX scope restrictions in
KMIPKIT-0091 and KMIPKIT-0092. It is an additive expansion of their public
APIs, not permission to reinterpret protocol assignments, fabricate response
fields, or bypass the version and ownership rules. The maintainer explicitly
selected the complete supplied master plan and publication within the 1.x
product line. The supplied plan is a design input; pinned OASIS sources and the
approved catalog determine wire semantics and assigned values.

## Normative and architectural sources

- OASIS KMIP Specification v2.0 and v2.1: §§4–5 attributes, §6.1 client
  operations, §6.2 server operations, §§8–9 messages and result behavior,
  §11.36/Table 470 operation values and §11.56/Table 487 tag values in v2.1.
- OASIS KMIP Profiles v2.1 §§5.3–5.5: HTTPS, XML, and JSON encodings.
- OASIS KMIP 3.0 Working Draft (13 July 2026): §6 operation payloads,
  §11.37/Table 584 operation assignments and §11.58/Table 605 tags.
- `specification/catalog/` and immutable `specification/oasis/` sources supply
  the checked-in, versioned assignment and requirement inventory.
- Accepted ADR-0012, ADR-0014, ADR-0018 (encoding), ADR-0018 (response-tree
  ownership), ADR-0021, and ADR-0022 govern the typed execution boundary,
  multi-encoding, response ownership, server direction, and versions.
- Existing operation models and their owning specifications remain the source
  of required fields, conditional checks, and response payload shapes.

## Corrections to the supplied design input

1. KMIP 3.0 Table 584 assigns `0x3C`–`0x42` to Create Group, Obliterate,
   Create User, Create Credential, Deactivate, Encapsulate, and Decapsulate.
   Encapsulate and Decapsulate are `0x41` and `0x42`, respectively. There are
   seven assigned 3.0 operations in this range, so the versioned inventory has
   59 earlier operation codes plus seven 3.0 codes, subject to actual
   direction and version availability.
2. The production client supports KMIP 2.0, 2.1, and explicitly selected 3.0
   routes. Legacy KMIP 1.x is excluded by ADR-0022. References to KMIP 1.x in
   examples describe historical operation introduction only.
3. A response property is populated only when its operation response payload
   or common response metadata actually carries that value. A Create response
   that returns only a Unique Identifier has no known key name, algorithm,
   length, state, or UUID unless the identifier itself parses as a UUID. Request
   values are available from the request object and are never labeled as
   server-confirmed response values. No implicit Get Attributes is performed.
4. A `0x54xxxx` message extension tag is not automatically a KMIP Vendor
   Attribute. The dedicated Vendor Attribute helper builds the catalogued
   structure. Unknown tags require an explicit TTLV type and value or Item.
5. Server-to-client request objects are received by the configured listener
   and answered by its handler. They do not expose a client-side `.send()`.
   Client-to-server objects do. One explicit send performs at most one exchange.

## Functional requirements

### Versioned catalog and generation

- **KMIPKIT-0093-FR-001**: A checked-in declarative catalog MUST describe each
  assigned operation, tag, enum value, attribute, type, version span,
  direction, request field, response field, and applicable attribute group.
  Generated surfaces MUST preserve unknown values through explicit raw-value
  wrappers. Numeric ranges MUST NOT create unassigned symbols.
- **KMIPKIT-0093-FR-002**: Generation MUST produce consistent Rust, C, Java,
  and Python symbols from one reviewed source, update `public-api.json` and
  generated exports through repository tools, and prove reproducibility in CI.
  Existing public names remain source and ABI compatible.
- **KMIPKIT-0093-FR-003**: A versioned operation matrix MUST list all 59
  assigned `0x01`–`0x3B` operation values and the seven assigned KMIP 3.0
  `0x3C`–`0x42` values. Each row MUST identify its direction, earliest
  supported version, typed request/response owners, client or listener route,
  and test evidence. The matrix MUST distinguish an assigned value from an
  executable route and from a server capability claim.

### Request construction and execution

- **KMIPKIT-0093-FR-004**: Every eligible client-to-server operation MUST have
  an operation-specific fluent request object with semantic setters for its
  actual request fields, `.validate()`, `.to_batch_item()`, and `.send()` or
  idiomatic equivalent. The fluent object MUST build the existing typed
  request variant, use its validator and version gate, and execute through
  the existing `Client` without a second transport or codec stack.
- **KMIPKIT-0093-FR-005**: Fluent requests MUST accept a prebuilt
  `AttributeSet` when their protocol operation supports attributes. An
  explicit conversion from a typed batch result to the matching typed
  response MUST check the operation and result status and preserve Success,
  Pending, Failure, correlation, reason, and delivery state.
- **KMIPKIT-0093-FR-006**: Catalogued attributes MAY take primitive values
  only after checking their TTLV type and operation/version placement.
  Unknown attributes MUST take an explicit typed Item. Vendor Attribute MUST
  use its KMIP structure. Expert positional injection MUST be explicit about
  parent path, index, tag, TTLV type and value; the final typed request MUST
  pass the same structural, version, size, secret-ownership, and redaction
  checks before transport. Reserved or unassigned tags MUST be rejected.
  No public `Client::execute(Item)`, raw-message send, or arbitrary transport
  constructor is introduced.
- **KMIPKIT-0093-FR-007**: No method may silently choose a cryptographic
  usage mask, mode, IV, padding, hash, retry, Poll, key material format, or
  protocol version. Named presets MUST document every value they set and
  remain opt-in. The default effective protocol version remains KMIP 2.1.

### Responses and encodings

- **KMIPKIT-0093-FR-008**: Every operation-specific response object MUST
  expose its actual typed result fields and the common status, reason,
  message, operation, protocol version, and correlation where carried. Fields
  absent from the wire MUST remain optional/absent. Secret bytes MUST use
  existing owned containers and redacted formatting.
- **KMIPKIT-0093-FR-009**: A successfully decoded response MUST offer
  deterministic TTLV hex, OASIS JSON, and OASIS XML views of its retained
  semantic response tree, independent of the selected transport encoding.
  These views are generated on explicit access, inherit the response tree's
  limits, retain order and unknown allowed values, and document that string
  copies in Java/Python/C are caller-managed. The raw wire bytes and padding
  are not promised. Failure to transcode MUST be explicit and MUST NOT replace
  or hide the operation result.
- **KMIPKIT-0093-FR-010**: A request MUST offer explicit TTLV/JSON/XML
  inspection only through the already approved typed encoder and zeroizing
  owner. Diagnostics, `Debug`, exceptions, and logs MUST not print request or
  response bodies or secrets.

### Four-language surface

- **KMIPKIT-0093-FR-011**: Rust MUST expose the shared fluent model and typed
  result adapters. C MUST expose opaque request/response handles, fixed-width
  status returns, explicit ownership, direct common-task functions, and the
  advanced builder. Java MUST offer operation objects with idiomatic camelCase
  methods; Python MUST offer snake_case operation objects, named arguments,
  and PEP 561 type information. All four surfaces MUST cover the same
  executable operation matrix without changing existing entry points.
- **KMIPKIT-0093-FR-012**: Server-to-client Notify, Put, Query, Discover
  Versions, and other approved listener operations MUST use received typed
  request objects and typed handler responses. They MUST preserve listener
  authentication, version, encoding, and handler ownership. The client
  fluent `.send()` API MUST reject or omit these directions.
- **KMIPKIT-0093-FR-013**: Four-language examples MUST complete the five
  progressive journeys: symmetric key, AES-GCM, key pair and signing,
  expert attributes, and batch ID Placeholder. Each MUST use a controlled
  TLS 1.3/mTLS fixture that returns real encoded responses, plus separate
  external-KMS evidence when configured.

## Delivery slices and dependencies

1. **Catalog and boundary**: correct the master plan's assignments, build
   operation/attribute/enum schema, code generation, manifest, and ADR.
2. **Response representation**: shared retained response-tree transcoding
   and typed batch-result conversion with secret-safe views.
3. **Expert attributes**: schema-checked common/vendor attributes and
   positional insertion over typed request payloads.
4. **Operation objects in Rust**: all assigned executable client operations,
   with explicit version and direction behavior; server handler objects use
   their separate listener flow.
5. **C ABI**: opaque builders and responses plus direct common tasks.
6. **Java facade**: generated operation objects and typed responses.
7. **Python facade**: operation objects, typed responses, and PEP 561 stubs.
8. **Journeys and audit**: four-language TLS fixture applications, negative
   cases, source compatibility, traceability, coverage, and generated-output
   audit.

Every slice is a separate stacked branch and `[WIP]` draft PR. When CI is
green for that PR, remove `[WIP]` and mark ready for human review. While a
review or merge is pending, the next branch starts from the unmerged parent
head. Red CI is fixed before a PR is marked ready. Only a human approves or
merges. The graph is rebased onto `release/1.0.0` in dependency order after
merges, and each new head is verified again.

## Acceptance

- All catalogued operation rows map to a typed route or an explicit,
  source-backed direction/version exclusion; no class name is generated for
  an unassigned value.
- The common create/use/destroy path in all four languages needs no numeric
  KMIP tag, request payload, or manual batch construction.
- At least one real TLS fixture exchange per operation family proves the
  generated objects use the production execution path; all operation rows
  receive focused local validation and result mapping tests.
- Existing consumer suites pass unchanged; changed Rust/C/JNI/CFFI ownership,
  redaction, and version boundaries have negative tests and traceability.
- Formatting, Clippy, binding suites, coverage gates, manifest generation,
  and required CI jobs pass at each PR head before it is marked ready.

## Explicit limits

The library does not perform local cryptographic operations. A server may
reject a valid request based on policy or unsupported capability. KMIP 3.0 is
a pinned working draft in this repository; support of its catalogued routes
does not assert final-standard or formal profile conformance. The API does not
automatically issue follow-up operations to fill missing response fields.
