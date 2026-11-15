# Data Model: Fluent Object Creation

This design reuses KMIPKIT-0014, KMIPKIT-0022, and KMIPKIT-0049 models. The
entities below describe the caller-facing selections and their relationship;
they do not add a second protocol model.

## Key-Pair Creation

| Field | Meaning | Validation and preservation |
| --- | --- | --- |
| Preset or explicit algorithm | Caller-selected KMIP cryptographic algorithm and, when needed, length/domain parameters | Presets are limited to RSA 3072, ECDSA P-256, and ECDH P-256. No usage or server policy is selected by a preset. |
| Private Key usage | Caller-selected allowed uses for the private key | Explicit and independent from the public-key usage. |
| Public Key usage | Caller-selected allowed uses for the public key | Explicit and independent from the private-key usage. |
| Common Attributes | Caller-selected attributes shared by the pair | Remains distinct from both key-specific groups. |
| Private Key Attributes | Caller-selected attributes for the private key | Table 191 effective-value validation applies; no synthesis or copying. |
| Public Key Attributes | Caller-selected attributes for the public key | Table 191 effective-value validation applies; no synthesis or copying. |
| Protection Storage Masks | Optional common, private, and public storage choices | Each group preserves independent optionality and existing typed validation. |

The successful response retains the existing typed private and public Unique
Identifiers. Pending/failure outcomes follow the shared execution contract.

## Split-Key Creation

| Field | Meaning | Validation and preservation |
| --- | --- | --- |
| Object Type | KMIP type of the source object to split | Explicit caller value; validated by the existing typed request path. |
| Split Key Method | Caller-selected supported or lossless open KMIP value | Explicit; no method is inferred. |
| Parts and threshold | Caller-selected split count and reconstruction threshold | Explicit signed values; existing source and client policy applies. |
| Attributes | Required caller-selected split-object attributes | The outer Attributes group remains present, including when empty. |
| Input identifier | Optional identifier of an existing object | Omission remains distinct; KMIPKit does not infer the object. |
| Prime Field Size | Optional Big Integer input required by KMIPKit policy for Polynomial Sharing Prime Field | Exact bytes are retained; no value is synthesized. |
| Protection Storage Masks | Optional caller-selected storage choices | Preserved and validated by existing typed rules. |
| Result identifiers | One or more server-returned object identifiers | Every identifier and wire order is preserved. |

## Register Input

The request input consists of a typed managed object, direct object Attributes,
and optional protection-storage choices. The typed object keeps its Object Type
associated with its KMIP 2.1 structure and Key Format Type. All nine existing
object variants remain represented: Certificate, Symmetric Key, Public Key,
Private Key, Split Key, Secret Data, Opaque Object, PGP Key, and Certificate
Request. Unknown/vendor objects require an explicit typed generic-object value.

Key Block, Key Value, Key Material, and Key Wrapping Data follow the existing
KMIPKIT-0049 ownership model. Secret-bearing values are redacted from
formatting and errors and KMIPKit-owned storage is zeroized on release.
Foreign-runtime copies remain subject to Java/Python lifetime limitations.

## Shared Request and Result

All requests use the existing effective protocol version, `RequestOptions`,
typed batch conversion, response and error model, and delivery state. A caller
explicitly invokes Poll or another operation after Pending; these builders
perform no follow-up or retry.
