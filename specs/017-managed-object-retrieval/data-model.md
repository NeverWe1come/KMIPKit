# Data Model: KMIP 2.1 Get and Locate

**Feature**: KMIPKIT-0017<br>
**Normative sources**: OASIS KMIP Specification v2.1 §§6.1.19 and 6.1.28, Tables 220–222 and 247–249.<br>
**Catalog links**: 2 Get requirement rows and 13 Locate rows are linked to this operation family. Actor classification for six Locate behavior rows remains open; see Open Inventory Questions.

## Shared values

| Entity | Representation | Constraints |
| --- | --- | --- |
| Unique Identifier | Existing kmipkit_protocol::UniqueIdentifier | Preserve Text String, Enumeration, or Integer form; never synthesize one. |
| Object Type | Existing raw-preserving ObjectType | Retain unknown Enumeration values. |
| Enumeration selector | Raw-preserving unsigned value following existing protocol conventions | Retain future values accepted by the generic TTLV model. |
| Attributes | Existing ordered AttributeSet | Required enclosing Structure may be empty; preserve repeated items and order. |
| Any Object | Existing generic kmipkit_ttlv::Item / Structure / Value | Preserve tag, nested order, and unknown allocated values; do not parse cryptographic material. |
| Operation result | Existing KmipOperationResult and ClientOperationOutcome | Preserve status/reason/message, Pending status, response ownership, and request delivery state. |

## Get request and successful response

Sources: §6.1.19, Tables 220 and 221.

The Table 220 request has optional Unique Identifier, Key Format Type, Key Wrap Type, Key Compression Type, and Key Wrapping Specification fields. The request model preserves each field's presence and caller-supplied value and serializes them in Table 220 order.

The Table 221 success response requires Object Type Enumeration, Unique Identifier, and Any Object. The response model exposes the required envelope and retains the Any Object as an opaque generic item. A missing, duplicate, or mistyped required success field is a sanitized protocol error. Error outcomes retain the shared result and expose no success-only values.

The typed model does not perform wrapping, unwrapping, key conversion, PKCS#12 construction, link traversal, or certificate-chain selection. It reuses the shared owned TTLV zeroization and redacted-debug guarantees. Caller-created copies remain outside KMIPKit's memory guarantee.

## Locate request and successful response

Sources: §6.1.28, Tables 247 and 248. Table 247 order is Maximum Items, Offset Items, Storage Status Mask, Object Group Member, then required Attributes Structure. An empty Attributes Structure remains present.

| Field | TTLV form | Presence | Model |
| --- | --- | --- | --- |
| Maximum Items | Integer | Optional | Option<i32> |
| Offset Items | Integer | Optional | Option<i32>; explicit zero remains present |
| Storage Status Mask | Integer bit mask | Optional | Option<StorageStatusMask>; preserve the raw 32-bit pattern, including unknown bits |
| Object Group Member | Enumeration | Optional | Raw-preserving unsigned value |
| Attributes | Structure of ordered direct attributes | Required | AttributeSet, including an empty set |

`StorageStatusMask` is a raw-preserving wrapper over the 32-bit pattern carried by the signed KMIP Integer value. Encoding and decoding reinterpret the same `i32`/`u32` bit pattern; they do not perform a numeric range conversion, validate assigned bits, or discard bits unknown to this release. OASIS §12.3 assigns Online Storage `0x00000001`, Archival Storage `0x00000002`, and Destroyed Storage `0x00000004`; preserve combinations and every other bit. The Table 248 response has optional Located Items Integer followed by zero or more repeated Unique Identifier items. Preserve all identifiers, repetitions, and wire order. The client does not sort, deduplicate, locally match, filter, or predict the server's ID Placeholder.

An omitted Storage Status Mask remains omitted. OASIS describes the server-side online-only default; any Locate IDs returned contrary to the server requirement remain visible in the generic response and do not become evidence of client-side enforcement.

## Bounded input and errors

Operation errors use the shared result model and Tables 222 and 249. Preserve unknown result reason Enumeration values. Reuse the existing decoder message-size, depth, and element limits. Sanitized errors and debug output never include raw KMIP response bytes or object contents.

## Open inventory questions

- KMIPKIT-DISC-015 leaves the lowercase “shall” in §6.1.19 PKCS#12 output guidance unresolved. Do not assert the affected output-format rule as conformed until the catalog decision is approved.
- KMIPKIT-DISC-032 leaves the official PKCS#12 case label-to-fixture mapping unresolved. Both fixtures are unavailable; no official-case pass may be claimed.
- The Locate rows KMIPKIT-REQ-SPEC-6.1.28-004-002, -008-001, -008-002, -009-001, -009-002, and -012 are cataloged as client requirements although their summaries describe server actions or server response obligations. The implementation specification preserves request/result values, but does not claim that a client can enforce server behavior. Correct the catalog actor/direction allocation before using these rows as client conformance gates.

## Normative identifiers

Get: KMIPKIT-REQ-SPEC-6.1.19-001 and -002. Locate: KMIPKIT-REQ-SPEC-6.1.28-001, -002, -004-001, -004-002, -007, -008-001, -008-002, -009-001, -009-002, -011, -012, -013-001, and -013-002.
