# Data Model: KMIPKIT-0050

## `CryptographicParameters`

An owned wrapper over one ordered KMIP `Structure`. `try_from_ttlv` validates known fields, order, singleton cardinality, and defined Item Types while retaining the original TTLV tree, unknown fields, and raw Enumeration values. Typed getters return `Option` because each Table 59 member is optional. `with_ttlv` provides callback-scoped generic access; `into_ttlv` returns the original structure by ownership.

The 18 defined fields in Table 59 are:

| Order | Member | TTLV type | Typed Rust value |
|---:|---|---|---|
| 1 | Block Cipher Mode | Enumeration | raw `u32` |
| 2 | Padding Method | Enumeration | raw `u32` |
| 3 | Hashing Algorithm | Enumeration | raw `u32` |
| 4 | Key Role Type | Enumeration | raw `u32` |
| 5 | Digital Signature Algorithm | Enumeration | raw `u32` |
| 6 | Cryptographic Algorithm | Enumeration | raw `u32` |
| 7 | Random IV | Boolean | `bool` |
| 8 | IV Length | Integer | `i32` |
| 9 | Tag Length | Integer | `i32` |
| 10 | Fixed Field Length | Integer | `i32` |
| 11 | Invocation Field Length | Integer | `i32` |
| 12 | Counter Length | Integer | `i32` |
| 13 | Initial Counter Value | Integer | `i32` |
| 14 | Salt Length | Integer | `i32` |
| 15 | Mask Generator | Enumeration | raw `u32` |
| 16 | Mask Generator Hashing Algorithm | Enumeration | raw `u32` |
| 17 | P Source | Byte String | owned, zeroizing byte string |
| 18 | Trailer Field | Integer | `i32` |

Source: KMIP v2.1 §4.16, Table 59. The builder order follows the table exactly;
the order requirement for defined Structure fields is KMIP v2.1 §10.1.2.
Unknown fields are retained in their source positions when parsing. The builder
appends caller-supplied extension Items after the defined fields because the
specification assigns no Table 59 position to unknown members.

`CryptographicParametersBuilder` owns at most one value for each known field and an ordered list of caller-supplied unknown extension Items. `build` emits the 18 known fields in Table 59 order followed by extensions. It does not infer server policy or select defaults.

## `ClientBatch` Authentication

`ClientBatch` gains `authentication: Option<Authentication>`. The default is `None`. A consuming builder method sets one Authentication structure for this Request Message. The field is not copied to `ClientConfiguration`, `Client`, individual `ClientBatchItem`s, or saved client state.

## Direct operation APIs

Each direct operation method accepts its existing typed request. Its option form additionally accepts `RequestOptions`. The implementation maps the request into a one-item `ClientBatch`, delegates to the shared execution path, and returns the corresponding `ClientBatchItemResponse`. Async operation methods preserve Pending and correlation values through the same existing operation outcome types.
