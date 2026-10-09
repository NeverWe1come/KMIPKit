# Contract: Encrypt and Decrypt Operation Payloads

## Scope and ownership

This contract adds Rust protocol request and successful-response types for KMIP v2.1 Encrypt (§6.1.17) and Decrypt (§6.1.11), plus client dispatch through the existing batch/execution pipeline. It adds no algorithms, stateful streams, retries, polling, or cryptographic defaults.

## Request encoding

Encode each present member exactly once, in table order. Do not normalize, reorder, narrow, or synthesize values.

| Position | Tag | Encrypt | Decrypt | TTLV |
| --- | --- | --- | --- | --- |
| 1 | Unique Identifier | optional | optional | Typed identifier; omission only via valid ID Placeholder |
| 2 | Cryptographic Parameters | optional | optional | Structure |
| 3 | Data | optional by part rules | optional by part rules | Byte String, Enumeration, or Integer (§7.9) |
| 4 | IV/Counter/Nonce | optional | optional | Byte String |
| 5 | Correlation Value | optional | optional | Byte String (§7.8) |
| 6 | Init Indicator | optional | optional | Boolean (§7.17) |
| 7 | Final Indicator | optional | optional | Boolean (§7.14) |
| 8 | Authenticated Encryption Additional Data | optional | optional | Byte String (§7.3) |
| 9 | Authenticated Encryption Tag | absent | optional | Byte String (§7.4) |

Tables: Decrypt Table 196; Encrypt Table 214. Reject fields not defined for the selected operation.

## Data and multipart rules

Follow the matrix in data-model.md. The first multipart request has Init Indicator true and no Correlation Value. Subsequent and final requests carry the first server response's Correlation Value. A middle request with neither Init nor Final true includes Data. AAD and a supplied Decrypt tag are placed on the initial request. Each caller call executes no more than one exchange.

While KMIPKIT-DISC-045 remains open, the typed client returns the existing sanitized local validation error before encoding/transmission for a one-request form with both Init and Final true. This gates the ambiguous form without choosing whether OASIS requires or permits Data. The generic TTLV escape hatch remains subject to its separate structural contract.

Unique Identifier may be omitted only for a valid ID Placeholder in an eligible later operation in an ordered batch. The operation request model preserves the optional field; locally detectable eligibility and composition rules stay at the client batch layer. The client cannot know whether a prior operation will succeed at the server, so a structurally eligible later operation is sent in order and its server result is preserved if the earlier operation fails. A standalone request without an identifier is not a valid positive case. Shared batch semantics follow KMIPKIT-0006.

## Parameter rules

Preserve the generic ordered Cryptographic Parameters Structure, including unknown members. If caller-supplied parameters expose a known variable-IV Block Cipher Mode, require IV Length. If they expose GCM, require Tag Length (§4.16). Omitted parameters/IV values whose validity depends on server-held attributes are passed through; do not query hidden state or choose values. The Table 59 header extraction is an excluded source-clause audit record because “REQUIRED” is the column heading, not a value for the Cryptographic Parameters row.

## Response shape

Tables: Decrypt Table 197; Encrypt Table 215.

- On Success, require exactly one Unique Identifier.
- Encrypt success may include Byte String Data, IV/Counter/Nonce, Correlation Value, and Authenticated Encryption Tag.
- Decrypt success may include Byte String Data and Correlation Value.
- Response Data is optional in either operation, including a single-part response.
- A completed non-success result retains the common result and has no success payload; no Unique Identifier is required.
- Pending uses the shared PendingOutcome with Asynchronous Correlation Value. This differs from multipart Correlation Value.
- Retain the generic response tree for unknown fields and preserve all Result Reason values permitted by common Message Data Structures, not only Table 198/216 rows.

Error tables are Decrypt Table 198 and Encrypt Table 216; these do not restrict the shared Result Reason enumeration.

## Redaction and failure semantics

Manual Debug/Display for operation data and errors must redact values, including Enumeration and Integer Data, byte strings, AAD, tags, IVs, correlation values, response plaintext/ciphertext, and raw KMIP bodies. Reuse the existing zeroizing owner for owned bytes. Protocol errors identify only field/category, never contents.

Reuse the current delivery classification: not sent, possibly sent, or response begun. Never retry. Reuse bounded TTLV decoding and common Pending handling.

## Planned public modules

- crates/kmipkit-protocol/src/encrypt.rs
- crates/kmipkit-protocol/src/decrypt.rs
- crates/kmipkit-protocol/src/operation_data.rs (if a shared redacted three-encoding value is needed)
- crates/kmipkit-client/src/execute.rs

Final names follow crate conventions discovered before Green implementation; public behavior and wire contract above do not.
