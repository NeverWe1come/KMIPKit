# KMIPKIT-0049 Data Model

## Ownership layers

| Entity | Data | Ownership and constraints |
|---|---|---|
| `ManagedObject` | One typed object-specific Structure `Item` and its matching Object Type, or an unknown type plus generic Item | Known values convert to the single ordered TTLV `Item` passed directly to Register/Import; unknown values remain generic and lossless. No extra wrapper is encoded. |
| Object-specific owner/view | The exact fields and structure tag in one of nine §2 object tables | Owner consumes TTLV values; view borrows from the source Item and cannot outlive it. |
| `KeyBlock` | Key Format Type; optional Key Value; optional Key Compression Type, Cryptographic Algorithm, Cryptographic Length, and Key Wrapping Data | Field presence follows §3.1 and catalog conditional rules; canonical emission order follows Table 12. |
| `KeyBlockValue` | No value, plaintext `KeyValue`, or wrapped Byte String | Plaintext is a Structure; wrapped payload is an owned TTLV Byte String and is never interpreted by KMIPKit. |
| `KeyValue` | Required Key Material and optional Attributes | Attributes use the ordered generic/typed `AttributeSet` from KMIPKIT-0048. |
| `KeyMaterial` | A Key Format Type-selected Byte String or Structure, including transparent forms | Raw/opaque/PKCS and extension forms retain their exact bytes/structure. All owned TTLV payloads use the existing zeroizing owner. |
| Transparent key material | Table-defined fields for Transparent Symmetric, DSA private/public, RSA private/public, DH private/public, and EC private/public | Big Integer/Byte String/Enumeration members preserve exact values, requiredness, and order; no cryptographic math is performed. |
| `KeyWrappingData` | Wrapping Method; optional encryption/MAC key info, MAC/signature, IV/counter/nonce, and encoding option | Parameters and identifiers are caller-supplied; the model does not wrap or unwrap. |
| Key information | Required Unique Identifier and optional Cryptographic Parameters | Text and parameter structure remain raw-preserving and payload-redacted. |
| `AttributeSet` | Ordered repeated Attributes | Existing 0048 multiplicity semantics remain intact; operation schemas remain authoritative. |

## Object structure inventory

| Typed object | Structure tag | Fields and source constraints |
|---|---|---|
| Certificate | `0x00420013` | Certificate Type Enumeration; Certificate Value Byte String; both required. |
| Certificate Request | `0x00420018` | Certificate Request Type Enumeration; Certificate Request Value Byte String; both required. |
| Opaque Object | `0x0042005B` | Opaque Data Type Enumeration and Opaque Data Value Byte String; both required. The local OpaqueObjectContext remains separate from encoded fields. |
| PGP Key | `0x004200BD` | PGP Key Version Integer and Key Block; both required. Key Block contents remain opaque. |
| Private Key | `0x00420064` | Key Block required. |
| Public Key | `0x0042006D` | Key Block required. |
| Secret Data | `0x00420085` | Secret Data Type Enumeration and Key Block; both required; wrapped Key Value is supported. |
| Split Key | `0x00420089` | Key Block, Key Part Identifier, Split Key Method, Split Key Parts, and Split Key Threshold required; Prime Field Size required only for Polynomial Sharing Prime Field; Key Part Identifier is 1..=Split Key Parts. |
| Symmetric Key | `0x0042008F` | Key Block required. |

These object structure tags are distinct from their Object Type Enumeration
values. The typed representation associates both values; Register and Import
continue to carry the object-specific tagged Item directly, preserving their
current wire shape.

Table-defined order controls output. In particular, Table 9 emits Split Key
Parts, Key Part Identifier, Split Key Threshold, Split Key Method, optional
Prime Field Size, then Key Block. Table 12 emits Key Format Type, optional Key
Compression Type, Key Value, Cryptographic Algorithm, Cryptographic Length,
and Key Wrapping Data, omitting absent optional fields without reordering the
remaining fields.

## Key-material forms

The `Key Format Type` remains an open raw Enumeration from §11.25. Typed
transparent formats are:

| Format | Material fields |
|---|---|
| Transparent Symmetric Key | Key Byte String |
| Transparent DSA Private Key | P, Q, G, X |
| Transparent DSA Public Key | P, Q, G, Y |
| Transparent RSA Private Key | Modulus required; private representation satisfies §3.7 alternative field rule; other components remain optional as specified |
| Transparent RSA Public Key | Modulus and Public Exponent |
| Transparent DH Private Key | P, G, X; optional Q and J |
| Transparent DH Public Key | P, G, Y; optional Q and J |
| Transparent EC Private Key | Recommended Curve and D |
| Transparent EC Public Key | Recommended Curve and Q String |

Raw, Opaque, PKCS1, PKCS8, ECPrivateKey, and extension formats retain the
source-defined Byte String or Structure representation. No format is inferred
from payload bytes.

## Schema order and unknown data

Each builder emits fields in the order in the defining OASIS table. Typed
views read by tag and return a sanitized error for a wrong TTLV category.
Unknown child Items and repeated occurrences remain in the retained generic
tree. Converting typed owners to generic Items does not normalize text, bytes,
Big Integer octets, enum values, or extension tags.

## Sensitive data lifetime

Owned values are backed by the existing `Value` owner, whose drop path clears
all payloads recursively. Debug and errors show type/count metadata only.
Borrowed material views are scoped to their owner. Typed API methods do not
create implicit secret copies; explicit caller copies and external runtime or
dependency copies are outside the guarantee.
