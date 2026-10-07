# Research: Server-Generated Object Creation

## Decision 1 — Use the pinned OASIS v2.1 operation tables as the schema

**Decision**: Implement payloads from Specification §§6.1.8–6.1.10 and Tables 186–195, with structural definitions from §§7.1–7.2 and common message/result behavior from applicable shared sections.

**Rationale**: The pinned HTML is the normative source and the inventory records operation element IDs and applicable requirement IDs. The Usage Guide is informative and cannot fill a normative gap.

**Alternatives considered**: Derive payloads from a server implementation or Usage Guide examples. Rejected because those may omit normative optional, repeated, or error cases.

## Decision 2 — Preserve a typed operation envelope and dynamic attributes

**Decision**: Expose distinct request/response types for each operation and an Attribute entry modeled from §7.2, Table 150: preserve Attribute Name as an exact Text String and Attribute Value as the existing generic TTLV Value union. Attribute entries remain ordered and may repeat. The client preserves the Common, Private, and Public groups; the server applies the Table 189 union rule. Do not accept caller-defined conversions or arbitrary pre-encoded request bodies.

**Rationale**: Table 150 encodes Attribute Name as a Text String and Attribute Value using a type that varies with the named attribute. KMIP Attributes are heterogeneous and extensible. The Value union preserves integer, enumeration, text, bytes, date-time, interval, and nested Structure representations without lossy conversion, while the exact string preserves unknown and vendor names. Convenience attribute-specific builders may be added before 1.0 if they preserve this wire contract and do not locally merge groups.

**Alternatives considered**: Convert every attribute into a rigid closed enum immediately. Rejected for this operation slice because it duplicates extensible values and risks losing unrecognized attributes. Treat the entire request as an arbitrary TTLV Item. Rejected because it bypasses the closed typed operation boundary from KMIPKIT-0007.

## Decision 3 — Preserve operation result and generic response ownership

**Decision**: Create a typed response view from the validated response batch item and retain a reference to the owning ResponseMessage for fields not modeled by the operation type. A non-success KMIP result has no fabricated success payload.

**Rationale**: This matches existing Discover Versions and async response conventions, preserves unknown fields, and avoids cloning secret-bearing response structures.

**Alternatives considered**: Consume or normalize the full response into the operation model. Rejected because it can discard extensions and field order and increase secret copies.

## Decision 4 — Use one explicit call and preserve asynchronous outcomes

**Decision**: The operation call enters the existing closed typed request writer once. Pending is represented under KMIPKIT-0009; this feature never polls, retries, waits automatically for completion, or claims server completion.

**Rationale**: KMIP asynchronous behavior is part of the existing client contract, and automatic follow-up would hide delivery and server policy from the caller.

## Decision 5 — Add no runtime dependency

**Decision**: Use only workspace TTLV, protocol, client, transport, and zeroization dependencies.

**Rationale**: The payload schemas need no new parser, crypto provider, or transport. Dependency policy stays reproducible.

## Normative evidence

Catalog-linked requirements for Create Key Pair are KMIPKIT-REQ-SPEC-6.1.9-001-001, -001-002, -006, and -007. Create Split Key links KMIPKIT-REQ-SPEC-6.1.10-001. The pinned KMIP Test Cases HTML includes `TC-CREATE-SD-1-21` for Create/Secret Data (§2.12), but its linked XML is not locally pinned; T001 adds a provenance-tracked local fixture. No direct Create Key Pair or Create Split Key case appears in the pinned work product.
