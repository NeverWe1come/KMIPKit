# Data Model: Opaque Object Register context

## Register request

The existing request contains Object Type, direct Attributes, the Object item, and optional Protection Storage Masks. Add a caller-selected opaque usage context to local request validation.

| Field | Type/shape | Default | Validation and behavior |
| --- | --- | --- | --- |
| Object Type | Raw-preserving KMIP Enumeration | Caller-supplied | Context selection is valid only for Opaque Object (Enumeration value 8; §11.34/Table 468). Unknown object types remain preservable when no opaque context selector is supplied. |
| Opaque Object context | Closed current choices, future-compatible public representation | Generic opaque | Generic mode does not require crypto attributes. Managed Cryptographic Object mode activates applicable Table 293 checks. |
| Attributes | Ordered Attribute set | Caller-supplied | Preserve tags, values, duplicates where structurally permitted, and order. Do not synthesize or infer attributes. |
| Object | Opaque Object tag with Structure value for Object Type 8 | Caller-supplied | Require the Opaque Object tag, Opaque Data Type as Enumeration, and Opaque Data Value as Byte String. Preserve unknown child values and do not parse Opaque Data Value. |
| Delivery state | Existing client error metadata | NotSent for local validation failure | Invalid context, malformed required child, or missing Table 293 attribute is rejected before any write. |

## Context choices

- **Generic opaque**: Default; represents an Opaque Object not handled as a Managed Cryptographic Object for its attributes.
- **Managed Cryptographic Object**: Explicit caller choice; applies the Register Table 293 rules to direct Attributes.

## Conditional attribute rules

| Attribute | Generic opaque | Managed Cryptographic Object context |
| --- | --- | --- |
| Cryptographic Usage Mask | Optional | Required |
| Cryptographic Algorithm | Optional | Required for Opaque Object because it has no Key Block |
| Cryptographic Length | Optional | Required for Opaque Object because it has no Key Block |
| Algorithm/Length pairing | No new check | Both must be present together |
| Certificate Length | Optional | Not applicable |
| Digital Signature Algorithm | Optional | Not applicable |

These are only Register preflight conditions. They do not authorize subsequent operations or assert that a remote server accepts the values.
