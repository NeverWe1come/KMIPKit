# KMIPKIT-0096: Fluent expert attribute insertion

**Status**: Approved for implementation by the maintainer's direct
2026-11-11 instruction to implement the complete fluent API plan.

## Purpose

Allow callers of the fluent Create Symmetric Key builder to add expert Object
Attributes at an explicit position while retaining the typed request boundary.
Catalogued attributes, explicit generic Items, and Vendor Attribute structures
must all pass the same existing validation before an exchange can be sent.

## Normative and architectural sources

- OASIS KMIP v2.1 §4 (Object Attributes), §4.60 and Table 150 (Vendor
  Attribute), §5 (attribute groups), §7.1 (TTLV Structures), and §8 (message
  encoding and field order).
- `specification/catalog/kmip-versioned-tags.json` and the generated typed
  attribute catalog.
- KMIPKIT-0093-FR-006 and ADR-0012; typed request validation and the existing
  single-exchange client boundary remain authoritative.

## Requirements

- **KMIPKIT-0096-FR-001**: The fluent Create Symmetric Key request MUST accept
  catalogued typed attributes and explicit generic TTLV Items. Catalogued
  standard tags MUST use their catalogued TTLV types. A Vendor Attribute MUST
  be constructed as the Table 150 structure and validated in required field
  order. Message Extension tags MUST NOT be accepted as direct Object
  Attributes.
- **KMIPKIT-0096-FR-002**: Expert positional insertion MUST identify the
  `RequestPayload/ObjectAttributes` parent and a zero-based index in the final
  ordered Object Attributes sequence. It MUST preserve all other items and
  reject out-of-range or ambiguous duplicate insertion positions with a
  payload-free build error.
- **KMIPKIT-0096-FR-003**: Building an expert request MUST run the same
  `AttributeSet` and typed request validation as semantic builder input.
  Rejection MUST happen before transport and MUST retain the existing
  `NotSent` delivery state. The API MUST NOT add arbitrary message submission
  or permit mutation of the Request Header, Batch Item, or transport.
- **KMIPKIT-0096-FR-004**: Attribute insertion and rejection MUST preserve
  order, ownership, and redaction. No attribute payload may appear in errors
  or `Debug` output.

## Acceptance

1. Tests prove ordered insertion at the beginning, between semantic
   attributes, and at the end of the final Object Attributes list.
2. The typed path exposes only Object Attributes, so other parent paths cannot
   be represented. Tests reject duplicate insertion indices, out-of-range
   indices, wrong catalogued TTLV types, and message-extension tags.
3. A successful build yields the existing typed `CreateRequest`; the normal
   client execution path remains the only send path.
4. Focused protocol and client tests, formatting, Clippy, manifest checks,
   traceability, and PR CI pass on the exact head.

## Boundary

This slice establishes the Rust request path. Equivalent C, Java, and Python
methods are delivered in the dependent binding slices. It does not add generic
TTLV request submission or positional editing outside Object Attributes.
