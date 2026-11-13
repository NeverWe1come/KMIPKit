# Feature Specification: Fluent Register Object (KMIPKIT-0100)

**Branch**: `feature/KMIPKIT-0100-fluent-register`
**Base**: KMIPKIT-0099 fluent Create Split Key, PR #181 at `d1eaf235a6aa39fa5db4a53e0562f1bdddf5afe6`
**Status**: Approved in scope by the maintainer's direct 2026-11-11 authorization of the complete KMIPKIT-0093 plan.
**Owner**: KMIPKIT-0093-FR-004, FR-005, FR-006, FR-007, FR-008, FR-010

## Purpose

Let a Rust caller register an existing typed or generic managed object by
providing its object and attributes through a task-oriented builder, validate
it locally, convert it to a typed batch item, or execute one request through
the existing client. The builder reuses `RegisterRequest`, its source-backed
validation, the existing protocol-version gates, and the shared exchange.

## Normative and architectural sources

- OASIS KMIP Specification v2.1 §6.1.43, Tables 291–293: Register request,
  response, and conditional Register Attributes.
- OASIS KMIP 3.0 §6.1.50, Tables 400–402: Register request and attribute
  requirements for the extended versioned object model.
- OASIS KMIP Specification v2.1 §§2.1–2.10, Tables 2–11: managed object
  structures and their object-type relationships.
- OASIS KMIP 3.0 specification §§2.1.3.1–2.1.3.4: credential objects
  supported by the existing versioned managed-object model.
- `specs/029-opaque-object-context/spec.md` (`KMIPKIT-0044`) owns the explicit
  Opaque Object context behavior. `specs/049-managed-objects/spec.md`
  (`KMIPKIT-0049`) owns typed object and secret ownership contracts.
- `specs/093-fluent-complete-api/spec.md` (`KMIPKIT-0093`) owns fluent
  construction, execution, result, TTLV, version, and cross-language policy.

## Functional requirements

- **KMIPKIT-0100-FR-001**: `Client::request().register()` MUST return a
  `RegisterBuilder` with semantic inputs for an existing `ManagedObject`, an
  explicit `(ObjectType, Item)` pair for a generic object, Attributes,
  optional Protection Storage Masks, and the existing explicit Opaque Object
  context. The existing Password, Device, One Time Password, and Hashed
  Password Credential objects defined by KMIP 3.0 §2.1.3 MUST retain their
  typed Object Type to item-tag relationship when used with Register.
- **KMIPKIT-0100-FR-002**: The builder MUST preserve caller object type, object
  fields, attribute values and order; MUST NOT infer cryptographic choices or
  inspect opaque object bytes. It MUST pass the final values to the existing
  `RegisterRequest` validator, including Table 293 and opaque-context rules.
  KMIP 3.0 credential objects MUST be accepted where Table 400 permits Any
  Object (Section 2), subject to the selected protocol version.
- **KMIPKIT-0100-FR-003**: The builder MUST provide `validate`, `build`,
  `build_request`, and `to_batch_item`. Missing or invalid inputs MUST yield
  payload-free `FluentBuildError` values. The resulting request MUST retain
  the current version gate and generic extension behavior.
- **KMIPKIT-0100-FR-004**: `execute` MUST send exactly one
  `ClientRequest::Register` using the current typed client path and preserve
  the existing response result, correlation, and delivery-state behavior. It
  MUST NOT retry or Poll.
- **KMIPKIT-0100-FR-005**: Builder-owned TTLV values MUST use the existing
  zeroizing TTLV ownership paths. Formatting and errors MUST NOT include
  object bytes, secret attributes, or raw KMIP bodies.
- **KMIPKIT-0100-FR-006**: The new Rust public declarations MUST be documented,
  traced to this feature and KMIPKIT-0093, recorded in the API manifest, and
  generated with repository tooling. Foreign-language parity remains a later
  delivery and MUST be marked pending.
- **KMIPKIT-0100-FR-007**: Fluent object builders MUST reject a known standard
  KMIP tag that is not a catalogued direct Object Attribute before request
  execution. An explicit `Item` MAY supply a catalogued attribute's exact TTLV
  type and value or an allocated unknown extension tag; extension tags MUST be
  preserved as-is and MUST NOT be reinterpreted as a Vendor Attribute. Reserved
  and unassigned tags remain rejected by the existing TTLV allocation check.

Compiler-visible standard trait implementations, such as `Debug`, are
language-level behavior, not separate KMIPKit product capabilities. The
manifest records them as exclusions while the explicit Register methods and
builder remain the declared public API.

## User scenarios and acceptance

1. A caller builds a typed Certificate or key object, supplies direct
   attributes, calls `.register().object(...).attributes(...)`, and obtains a
   valid existing typed request without constructing a batch by hand.
2. A caller supplies an unknown object as an explicit Object Type and TTLV
   `Item`; unknown tags and child order remain intact after build.
3. A caller selects `ManagedCryptographicObject` for an Opaque Object only
   when intended; Table 293 requirements are enforced by the existing
   validator and no context marker is put on the wire.
4. Invalid object shape or attributes fail before transport with delivery
   state `NotSent`; valid execution uses one exchange and exposes the existing
   typed Register result.
5. Debug output and payload-free failures contain no sentinel object or key
   bytes.

## Explicit limits

This slice adds the Rust fluent builder only and repairs the existing typed
object-type/tag mapping for KMIP 3.0 credential objects already supported by
the protocol model. It does not change Register wire semantics, credential
object fields, C ABI, Java or Python APIs, server behavior, encoding, transport,
or any KMIP assignment. It does not perform cryptography or claim OASIS profile
conformance.
