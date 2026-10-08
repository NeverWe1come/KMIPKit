# Research: KMIP 2.1 Attribute Operations

## Decision 1: Keep seven operation contracts separate

- **Decision**: Model Add, Adjust, Delete, Get Attributes, Get Attribute List, Modify, and Set as distinct typed operations.
- **Rationale**: Their request fields and semantics differ. In particular, Add preserves existing values, Adjust carries an operation and parameter, Delete selects one/all values, Modify replaces an existing selected value, and Set leaves the add-or-modify choice to the server.
- **Alternatives considered**: One generic read/write method; rejected because its flags and optional values would make operation-specific requirements ambiguous and easier to violate.
- **Evidence**: Pinned KMIP Specification v2.1 §§6.1.2, 6.1.3, 6.1.13, 6.1.20, 6.1.21, 6.1.34, and 6.1.51, with their cited request/response/error tables.

## Decision 2: Preserve Attribute name and complete TTLV value

- **Decision**: Reuse the `AttributeEntry` contract from KMIPKIT-0014: exact Attribute Name and complete generic TTLV value. Keep Attribute Reference, Current Attribute, and New Attribute as separate protocol roles.
- **Rationale**: §§5.5–5.7 define distinct structures. Generic values preserve exact names and unknown values on individually allocated or accepted §11.56 extension tags without guessing tags from names. Raw tags failing KMIPKIT-0004/ADR-0010 allocation checks cannot become public Items or be encoded.
- **Alternatives considered**: Map every name to only known typed values; rejected because unknown and vendor data must survive round trips.
- **Dependency**: The approved KMIPKIT-0014 shared-model implementation must be merged before coding this feature; otherwise this plan's implementation gate applies.

## Decision 3: Keep adjustment semantics at the server

- **Decision**: Send assigned Adjustment Type values or valid `0x80000000–0x8FFFFFFF` extension values and the optional Adjustment Value unchanged. Preserve unknown Enumeration values generically, but reject Reserved values outside the assigned and extension ranges before transmission. Do not compute an adjusted attribute locally.
- **Rationale**: §11.1 Tables 428–429 defines Increment, Decrement, and Negate and their applicable TTLV types/default parameters. §6.1.3 assigns missing-value, multi-instance, read-only, type, range, and object-state behavior to operation processing.
- **Alternatives considered**: Read-modify-write client behavior; rejected because it introduces races, needs server state, and is not equivalent to Adjust Attribute.

## Decision 4: Use catalog and test-case inputs already checked in

- **Decision**: Assign the seven operation elements and all applicable clauses to KMIPKIT-0016 in the normative catalog, and use pinned OASIS test-case evidence only where the complete case can be run within the approved scope. Use clearly labeled table-derived tests for other clauses.
- **Rationale**: The checked-in catalog and exact upstream copies are reviewed source inputs; builds must not fetch OASIS documents.
- **Alternatives considered**: Scrape OASIS pages or infer case support from server examples; rejected by repository policy and the document hierarchy.

## Decision 5: Reuse existing transport, errors, limits, and redaction

- **Decision**: Extend the existing typed execution request/response path and shared Result model; retain the 16 MiB/depth-64/100,000-element defaults, one-exchange behavior, pending outcomes, delivery-state reporting, and body/value redaction.
- **Rationale**: These are accepted workspace contracts and satisfy the client boundaries without a new runtime dependency or writer.
- **Alternatives considered**: Add a separate attribute transport or automatic retry; rejected because either duplicates the execution path or violates explicit security invariants.

## Resolved unknowns

The operation set, fields, optionality, repetition, adjustment values, absence rules, and error tables are stated in the pinned KMIP 2.1 specification. No unresolved technical choice requires clarification before design. Implementation order remains gated on KMIPKIT-0014 shared attributes and integration of KMIPKIT-0013/0015.
