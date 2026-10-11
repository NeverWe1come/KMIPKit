# Data Model: KMIP 2.1 PKCS#11 Forwarding

Normative source: OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275; §11.36 defines the Operation value. Catalog mappings and tests are in traceability.md.

## Request

| Field | Cardinality | TTLV/model contract |
|---|---|---|
| PKCS#11 Interface | 0..1 | Catalog Structure; preserve presence; do not insert V3.0 default. |
| PKCS#11 Function | exactly 1 | Catalog Structure holding Enumeration for standard or Integer for vendor functions; preserve raw form/value. |
| Correlation Value | 0..1 | Byte String; preserve exact bytes; redact as sensitive. |
| PKCS#11 Input Parameters | 0..1 | Generic Structure; opaque, lossless, redacted. |

## Successful response

| Field | Cardinality | TTLV/model contract |
|---|---|---|
| PKCS#11 Interface | 0..1 | Preserve response structure without default inference. |
| PKCS#11 Function | exactly 1 | Preserve Enumeration/Integer choice and unknown values. |
| Correlation Value | 0..1 | Preserve exact server bytes; redact. |
| PKCS#11 Output Parameters | 0..1 | Generic opaque Structure with unknown-child preservation and redaction. |
| PKCS#11 Return Code | exactly 1 | Preserve complete catalog Structure and open value domain. |

## Common invariants

- Result Status, Result Reason, permitted Result Message, Pending, unknown values, and extensions use the shared result model.
- Operation value is KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PKCS-11-00000033.
- Empty and absent structures/strings remain distinct.
- One explicit call sends one request; no retry or Poll is automatic.
- Debug, Display, and sanitized errors never expose opaque parameters/correlation; KMIPKit-owned secrets use existing zeroizing storage.


## Nested known-member constraints and Function context

Tables 379–381 require validation of the known nested member even though
profile contents remain opaque: Interface has 0..1 TextString Interface;
Input/Output Parameters have 0..1 ByteString Input/Output member. Duplicate
known members or wrong known scalar types fail. An empty or unknown-only
Structure remains distinct from an absent outer field; repeated unknown
children remain lossless. Function and Return Code each require exactly one
known scalar child.

Vendor Function Integer in the §6.1.37 operation context is resolved and traced
as `KMIPKIT-026-SOURCE-001` in `research.md`. The outer Function Structure is
retained; Table 378's standard Enumeration and Tables 273/274's explicit vendor
Integer alternatives are both covered. This choice makes no PKCS#11 Profile
or complete conformance claim.
