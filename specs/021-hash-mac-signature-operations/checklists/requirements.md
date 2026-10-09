# Requirements Quality Checklist: KMIP 2.1 Hash, MAC, and Signature Operations

**Purpose**: Review specification completeness, testability, source accuracy, and scope before implementation planning.
**Created**: 2026-10-09
**Feature**: [spec.md](../spec.md)
**Review Ownership**: Independent QA review. `[x]` means the requirement-quality criterion was reviewed and satisfied; it does not mean implementation is complete.

## Normative source coverage

- [x] CHK001 Each operation names the exact pinned OASIS KMIP v2.1 section and request, response, and error tables.
- [x] CHK002 Every client `MAY` requirement has a stable catalog ID, exact clause citation, FR mapping, and planned verification reference.
- [x] CHK003 Table requiredness, optionality, single-part, multi-part, and field descriptions are distinguished from explicit keyword requirements.
- [x] CHK004 Client request duties, server processing duties, and server response duties are clearly separated.
- [x] CHK005 The Hash table rows are traced without inventing per-cell `REQ` IDs unsupported by the catalog schema.
- [x] CHK006 The operation elements, operation values, required data types, relevant enumeration roots, standard values, and extension values are assigned or explicitly reused from another feature.
- [x] CHK007 Official Test Case mappings match catalog links, source labels, and fixture availability; no absent fixture is described as passed or failed.
- [x] CHK008 The Validity Indicator conflict is recorded with both source alternatives, affected operations, tolerant client behavior, and an explicit no-conformance-claim boundary.

## Behavior and testability

- [x] CHK009 User scenarios independently describe Hash, MAC/Sign, and MAC/Signature Verify client journeys.
- [x] CHK010 Acceptance scenarios cover exact request fields and response fields for all five operations.
- [x] CHK011 Multipart behavior and caller-controlled correlation are specified without hidden continuation or retry.
- [x] CHK012 Operation failures, malformed or disputed response shapes, unknown values, and delivery-state outcomes have testable expected behavior.
- [x] CHK013 The distinction between an invalid verification result and a failed KMIP operation is explicit and testable.
- [x] CHK014 Typed and generic TTLV preservation behavior is specified for future/vendor enum values and unrecognized valid fields.
- [x] CHK015 Success criteria are measurable against catalog assignments, tests, round trips, and the evidence limits.

## Security and product boundaries

- [x] CHK016 The specification excludes local cryptographic computation and implicit algorithm, key-size, parameter, usage, or protection choices.
- [x] CHK017 Secret and cryptographic data redaction follows the constitution and is verifiable.
- [x] CHK018 TLS, transport, retry, batch, and delivery-state behavior reuses accepted contracts without expanding scope.
- [x] CHK019 C, Java, Python, JSON, XML, and server-initiated behavior are explicitly excluded from this Rust feature.
- [x] CHK020 Server-only key lookup, usage limits, operation execution, and response generation are not assigned to the client implementation.

## Design readiness

- [x] CHK021 No placeholder text, unresolved clarification marker, or unsupported claim remains in the specification.
- [x] CHK022 Catalog corrections preserve stable IDs, retire inaccurate mappings explicitly, and pass the normative catalog validator.
- [x] CHK023 Research, data model, contracts, quickstart, tasks, and analysis can be produced from the approved specification without changing its scope.
