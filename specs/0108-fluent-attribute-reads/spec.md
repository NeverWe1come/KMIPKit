# Feature Specification: Fluent Attribute Read Builders (KMIPKIT-0108)

**Branch**: `feature/KMIPKIT-0108-fluent-attribute-ops`  
**Base**: `feature/KMIPKIT-0093-typed-results` at `dc9819f610d9f34322f34e3b0b4fbfdbe8f5f776`  
**Status**: Approved for implementation under the maintainer-authorized complete KMIPKIT-0093 plan.  
**Owning identifier**: KMIPKIT-0108

## Purpose

Add Rust fluent builders for Get Attributes and Get Attribute List using the
existing typed protocol requests and client exchange path. Callers will be able
to select an optional Unique Identifier, preserve ordered Attribute References
for Get Attributes, build a typed request or batch item, validate locally, and
execute one exchange without constructing a batch manually.

## Normative and architectural sources

- OASIS KMIP Specification v2.1 §6.1.20, Tables 223–224: Get Attributes
  request references and response attributes.
- OASIS KMIP Specification v2.1 §6.1.21, Tables 226–227: Get Attribute List
  request and response Attribute References.
- `specs/016-attribute-operations/spec.md`: KMIPKIT-0016 read semantics,
  including duplicate-reference rejection, optional identifier preservation,
  response ordering, and absent-versus-empty Attributes.
- `specs/093-fluent-complete-api/spec.md`: KMIPKIT-0093-FR-004, FR-005,
  FR-008, FR-011 and the Rust operation-object delivery slice.
- ADR-0012: execution remains behind the typed request boundary.

## Functional requirements

- **KMIPKIT-0108-FR-001**: `RequestBuilder::get_attributes()` MUST return a
  public builder over the existing `GetAttributesRequest`. It MUST preserve
  the optional Unique Identifier, ordered Attribute References, and the
  protocol model's duplicate-reference validation.
- **KMIPKIT-0108-FR-002**: `RequestBuilder::get_attribute_list()` MUST return a
  public builder over the existing `GetAttributeListRequest` and preserve an
  omitted Unique Identifier without synthesizing one.
- **KMIPKIT-0108-FR-003**: Both builders MUST provide local `validate()`, typed
  request construction, conversion to the corresponding `ClientBatchItem`,
  exchange-local `RequestOptions`, and an explicit asynchronous-indicator
  setter. Validation failures MUST be payload-free and prevent transport.
- **KMIPKIT-0108-FR-004**: `execute()` MUST use the existing typed client
  execution path for at most one exchange and return the existing typed
  `ClientBatchItemResponse`. It MUST NOT retry, Poll, or issue a follow-up
  request. Callers inspect `get_attributes()` or `get_attribute_list()` and
  retain Success, Pending/Failure, delivery state, correlation, and optional
  response fields according to existing contracts.
- **KMIPKIT-0108-FR-005**: Public Rust declarations, guide examples, traceability,
  and `public-api.json` MUST be updated using the repository generator. Existing
  public declarations remain unchanged.

## Acceptance criteria

1. Tests demonstrate exact request shape, optional identifier omission,
   ordered references, and local duplicate-reference rejection.
2. Tests verify batch conversion selects the existing operation variants.
3. Controlled-transport tests execute each operation once and inspect the
   existing typed response, including absent/present-empty Get Attributes
   values and response-reference order.
4. Invalid builders fail before transport with `NotSent`; no hidden follow-up,
   retry, Poll, or raw-message submission is introduced.
5. Documentation, generated API manifest, traceability, formatting, Clippy,
   Rust client tests, and exact-head PR CI pass before the PR is marked ready.

## Explicit limits

This slice adds only the two Rust read-operation builders. It does not change
protocol models, C/Java/Python bindings, mutation builders, server-initiated
routes, response parsing, or the existing generic and procedural APIs. Those
remain separate KMIPKIT-0093 delivery work.
