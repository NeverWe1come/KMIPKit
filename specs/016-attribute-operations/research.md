# Research: KMIP 2.1 Attribute Operations

## Decision 1: Keep seven operation contracts separate

- **Decision**: Model Add, Adjust, Delete, Get Attributes, Get Attribute List, Modify, and Set as distinct typed operations.
- **Rationale**: Their request fields and semantics differ. In particular, Add preserves existing values, Adjust carries an operation and parameter, Delete selects one/all values, Modify replaces an existing selected value, and Set leaves the add-or-modify choice to the server.
- **Alternatives considered**: One generic read/write method; rejected because its flags and optional values would make operation-specific requirements ambiguous and easier to violate.
- **Evidence**: Pinned KMIP Specification v2.1 §§6.1.2, 6.1.3, 6.1.13, 6.1.20, 6.1.21, 6.1.34, and 6.1.51, with their cited request/response/error tables.

## Decision 2: Follow the distinct OASIS attribute wire forms

- **Decision**: Model Table 161 `Attribute Reference` as either the name form (Vendor Identification and Attribute Name) or tag form (Enumeration). Model Current Attribute and New Attribute as wrappers around one direct generic TTLV Item whose tag identifies the §4 attribute. Reuse KMIPKIT-0014's `AttributeSet` for the Get Attributes response's ordered direct Items.
- **Rationale**: The exact structures in §§5.5–5.7 are different and must not be narrowed to a name/value pair. Generic Items preserve unknown values on individually allocated or accepted §11.56 extension tags; raw tags failing KMIPKIT-0004/ADR-0010 allocation checks cannot become public Items or be encoded.
- **Alternatives considered**: Map every name to only known typed values; rejected because unknown and vendor data must survive round trips.
- **Dependency**: The merged KMIPKIT-0013 transport, focused merged response-root correction, and accepted KMIPKIT-0014 `AttributeSet` implementation are required before coding Get Attributes. KMIPKIT-0015's Cosmian deployment/live smoke test is complementary integration evidence and does not gate implementation; its response-root code fix must land independently.

## Decision 3: Keep adjustment semantics at the server

- **Decision**: Send assigned Adjustment Type values or valid `0x80000000–0x8FFFFFFF` extension values and the optional Adjustment Value unchanged. Preserve unknown Enumeration values generically, but reject Reserved values outside the assigned and extension ranges before transmission. Do not compute an adjusted attribute locally.
- **Rationale**: §11.1 Tables 428–429 defines Increment, Decrement, and Negate and their applicable TTLV types/default parameters. §6.1.3 assigns missing-value, multi-instance, type, range, and object-state behavior to operation processing. Any unconditional client prohibition stated by §4 or §6.1.3 is checked locally from pinned source-backed metadata; conditions requiring remote object state remain server-authoritative.
- **Alternatives considered**: Read-modify-write client behavior; rejected because it introduces races, needs server state, and is not equivalent to Adjust Attribute.

## Decision 4: Use catalog and test-case inputs already checked in

- **Decision**: Assign the seven operation elements and all applicable clauses to KMIPKIT-0016 in the normative catalog, and use pinned OASIS test-case evidence only where the complete case can be run within the approved scope. Use clearly labeled table-derived tests for other clauses.
- **Rationale**: The checked-in catalog and exact upstream copies are reviewed source inputs; builds must not fetch OASIS documents.
- **Alternatives considered**: Scrape OASIS pages or infer case support from server examples; rejected by repository policy and the document hierarchy.

## Decision 5: Reuse existing transport, errors, limits, and redaction

- **Decision**: Extend the existing typed execution request/response path and shared Result model; retain the 16 MiB/depth-64/100,000-element defaults, one-exchange behavior, pending outcomes, delivery-state reporting, and body/value redaction.
- **Rationale**: These are accepted workspace contracts and satisfy the client boundaries without a new runtime dependency or writer.
- **Alternatives considered**: Add a separate attribute transport or automatic retry; rejected because either duplicates the execution path or violates explicit security invariants.

## Decision 6: Enforce source-backed unconditional mutation prohibitions

- **Decision**: Keep client-side policy narrow and auditable. Identify standard attributes from direct Item tags or Attribute Reference tag values; a name-form reference is matched to standard policy only if its exact Vendor Identification/Attribute Name pair has a source-backed mapping. Reject only a statically known unconditional prohibition before the exchange, report `NotSent`, and do not include submitted values in diagnostics. A supplied Vendor Attribute Item or name-form Attribute Reference may expose Vendor Identification `y`; tag-form references do not, and no identifier is inferred. For Add/Modify, reject every New Attribute identified as `Usage Limits`: §7.40 Table 392 requires Count, and §4.59 prohibits setting or modifying it through these operations.
- **Rationale**: OASIS §§4, 6.1.2, 6.1.3, 6.1.13, and 6.1.51 include client prohibitions that cannot be represented by blanket server-only validation. The catalog is the normative inventory and provides exact source references; deterministic generation avoids hand-maintained runtime policy drift.
- **Alternatives considered**: Reject every mutation that might be restricted, or defer all checks to the server; rejected because the former invents restrictions and the latter ignores explicit unconditional client prohibitions.

## Resolved unknowns

The operation set, fields, optionality, repetition, adjustment values, absence rules, and error tables are stated in the pinned KMIP 2.1 specification. No unresolved protocol choice requires clarification before design. Implementation order is gated on merged KMIPKIT-0013 transport and response-root correction, plus KMIPKIT-0014's `AttributeSet` implementation for the Get Attributes response. The KMIPKIT-0015 Cosmian deployment/live smoke test is deferred interoperability evidence and does not block typed operation implementation.
