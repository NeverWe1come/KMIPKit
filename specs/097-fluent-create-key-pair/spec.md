# Feature Specification: Fluent Create Key Pair Builder (KMIPKIT-0097)

**Branch**: `feature/KMIPKIT-0097-fluent-object-api`  
**Base**: `feature/KMIPKIT-0096-fluent-expert-attributes`  
**Status**: Approved for implementation by the maintainer's direct request to implement the complete KMIPKIT-0093 plan.  
**Owning identifier**: KMIPKIT-0097

## Purpose

Add an idiomatic Rust fluent builder for the existing typed Create Key Pair
request. The builder lets a caller set common cryptographic properties and
independent private/public key attributes, then builds a `CreateKeyPairRequest`,
converts it to the existing batch item, or executes it once through `Client`.
Existing protocol, batch, and client APIs remain source-compatible.

This is the first Rust operation-object slice under KMIPKIT-0093-FR-004. It does
not claim that the rest of the operation matrix is complete; later slices add
the remaining operation families and language facades.

## Normative and architectural sources

- OASIS KMIP Specification v2.1 §6.1.9, Tables 189–191: Create Key Pair
  request groups and the common/private/public attribute union rules.
- OASIS KMIP Specification v2.1 §§4, 5.1–5.4 and Tables 157–160: direct
  Object Attributes and their operation-specific groups.
- `specs/014-server-generated-creation/spec.md` and its research: typed
  Create Key Pair payload semantics, ordered groups, Table 191 validation,
  and the distinction between object attributes and Vendor Attribute.
- `specs/093-fluent-complete-api/spec.md`: KMIPKIT-0093-FR-004, FR-005,
  FR-007, FR-008, FR-011, and its operation-object delivery sequence.
- ADR-0012: execution stays behind the closed typed request boundary.

## Functional requirements

- **KMIPKIT-0097-FR-001**: `RequestBuilder::create_key_pair()` MUST return a
  public `CreateKeyPairBuilder` that constructs the existing
  `CreateKeyPairRequest`. The builder MUST support explicit Cryptographic
  Algorithm and Cryptographic Length values, plus separate Common, Private
  Key, and Public Key attribute groups.
- **KMIPKIT-0097-FR-002**: The high-level builder MUST require an explicit
  algorithm and length. It MUST NOT choose a usage mask, curve, key format,
  protocol version, or other cryptographic parameter. Private and public
  Cryptographic Usage Mask attributes, and private/public Name attributes,
  MUST be independently optional and explicitly caller-selected.
- **KMIPKIT-0097-FR-003**: Typed attributes MUST be checked against their
  generated TTLV types. Before returning a request or batch item, the builder
  MUST run the existing `CreateKeyPairRequest` validation, including Table 191
  consistency rules. A local construction or validation error MUST be
  payload-free and MUST prevent an exchange.
- **KMIPKIT-0097-FR-004**: `build`, `build_request`, `to_batch_item`, and
  `execute` MUST preserve the Common, Private Key, and Public Key groups and
  their item order. `execute` MUST use the existing typed client execution
  path, one exchange at most, existing `RequestOptions`, and explicit
  asynchronous-indicator behavior. Pending, failure, correlation, and delivery
  state MUST remain represented by existing result types.
- **KMIPKIT-0097-FR-005**: The builder, its errors, and the existing typed
  response accessors MUST be documented and recorded in
  `specification/api/public-api.json` using the repository generator. Existing
  public declarations MUST remain unchanged.
- **KMIPKIT-0097-FR-006**:
  `CreateKeyPairBuilder::validate(&self) -> Result<(), FluentBuildError>` MUST
  perform the same local construction and existing typed request validation as
  `build_request()`, return a payload-free `FluentBuildError` on failure, and
  MUST NOT invoke the transport or consume the builder.
- **KMIPKIT-0097-FR-007**: The builder MUST accept a prebuilt `AttributeSet`
  through consuming `common_attributes`, `private_key_attributes`, and
  `public_key_attributes` methods for Common, Private Key, and Public Key
  Attributes respectively. Each supplied set MUST be appended after that group's semantic values and previously appended
  typed attributes, preserving the supplied set's item order. Repeated set
  methods append in call order. Final request construction MUST run the
  existing typed request validator so duplicate or conflicting effective
  values are rejected before transport.

## Acceptance criteria

1. Tests prove required algorithm/length handling, correct common/private/public
   group placement, Table 191 mismatch rejection, batch conversion, and one
   typed execution through a controlled transport fixture.
2. Invalid construction is rejected with `RequestDeliveryState::NotSent` and
   does not invoke the transport.
3. Tests confirm no usage masks or curve values are synthesized.
4. The builder and response path preserve Success, Failure, Pending, and exact
   asynchronous correlation using existing result types.
5. Rust formatting, Clippy, client tests, traceability, manifest generation,
   export checks, and required PR CI pass before the draft is marked ready.
6. `validate()` succeeds for a valid builder and reports the same error as
   request construction for invalid inputs without invoking transport.
7. Prebuilt attribute sets remain in their selected group and preserve their
   internal and call order; repeated/conflicting attributes fail through the
   existing typed request validation.

## Explicit limits

This slice adds only the Rust Create Key Pair builder. It does not add a new
protocol payload model, raw-message submission, a default algorithm/length,
automatic polling/retry, or C/Java/Python methods. The underlying typed request
continues to support server-selected fields for callers using the existing
protocol API directly.

Standard Rust trait implementations for the new builder are Rust language
behavior, not separate cross-language KMIPKit product capabilities. Exact
compiler-visible standard-trait paths may be recorded as approved exclusions
under KMIPKIT-0097-FR-001 and FR-005; all explicit builder methods and new
error variants remain public declarations with pending binding parity.
