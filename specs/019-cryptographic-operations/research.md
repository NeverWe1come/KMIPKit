# Research: KMIP 2.1 Encrypt and Decrypt Operations

## Decision 1: Limit this feature to Encrypt and Decrypt

The normative catalog assigns Encrypt (§6.1.17) and Decrypt (§6.1.11) to the 1.0 client scope. Their payloads form a bounded family. Hash, MAC, Sign, Derive Key, key renewal, and other KMIP operations remain separate feature specifications.

## Decision 2: Preserve each request and response table exactly

Decrypt follows Tables 196–198; Encrypt follows Tables 214–216. Request members are emitted in table order. §7.9 permits Data encodings Byte String, Enumeration, and Integer; both success response tables define Data as an optional Byte String. The request model therefore uses a redacted OperationData union rather than restricting all request Data to byte strings.

A successful operation payload requires Unique Identifier. Non-success results carry the common result without that success payload. Pending is represented by the shared PendingOutcome with an Asynchronous Correlation Value, distinct from multipart Correlation Value (§7.8). Successful responses may omit Data even for single-part operations.

## Decision 3: Make multipart validity explicit

§6.1 says the initial request sets Init Indicator true, later requests include the Correlation Value returned by the server, and the last request sets Final Indicator true. It also says Data is required except when either Init or Final Indicator is true. Therefore an ordinary middle part, with both indicators absent/false, requires Data. AAD and Decrypt Authenticated Encryption Tag, when supplied in multipart calls, are carried on the initial request.

The single-part tables mark Data required, while the general §6.1 rule permits omission if Init or Final is true. A single request with both indicators true has both meanings. KMIPKIT-DISC-045 records this conflict. The typed client gates this form with a local pre-transmission validation error until an authoritative decision resolves the conflict; this scope gate does not select Data requiredness under either OASIS reading.

## Decision 4: Enforce locally knowable Cryptographic Parameters only

Pinned §4.16 states that IV Length SHALL be provided for a selected Block Cipher Mode supporting variable IV lengths (including CTR/GCM), and Tag Length SHALL be provided for GCM. Validate those conditions when caller-supplied parameters expose the mode. Keep unknown parameter values lossless. When parameters or IVs come from server-held object attributes, the client cannot inspect the object and passes the omission through for the server to resolve.

The Table 59 header extraction is retained only as an excluded source-clause audit record because “REQUIRED” is the column heading, while the Cryptographic Parameters row has no requiredness value and Tables 196/214 mark the operation field optional. It is not treated as a normative requirement. The catalog summary for 4.16-001-002 reflects the actual MAY sentence and creates no client-side object-type gate absent from the source.

## Decision 5: Preserve caller-driven multipart and common execution

The caller constructs each part and passes the server Correlation Value forward. KMIPKit performs at most one exchange per invocation, preserves Pending and delivery classification, and never retries or polls automatically. AAD and Tag remain caller-managed; no local cryptographic processing occurs.

Unique Identifier omission is only valid for an eligible later operation within an ordered batch after the conditions in §6.1 and KMIPKIT-0006's batch contract hold. A standalone Encrypt or Decrypt without an identifier is not a valid positive case. If a target object is archived, §6.1-005 makes Recover a caller precondition; the stateless client cannot infer remote object state.

## Decision 6: Preserve shared Result Reason values

The Result Reason values shown in Tables 198 and 216 are not exhaustive: §6.1 permits values defined for Message Data Structures. Parse through the common open result model, preserve known reasons outside these two tables, and retain unknown raw reason codes.

## Decision 7: Reuse redacted, zeroizing ownership

kmipkit_protocol::SecretBytes and TTLV Value redact byte values and zeroize KMIPKit-owned buffers. Operation Data also includes Enumeration and Integer encodings; its wrapper must redact their values from Debug/Display. No KMIP body, plaintext, ciphertext, AAD, AEAD Tag, or private key material may enter errors or logs. Foreign-runtime copies are outside Rust ownership guarantees and documented at the language binding layer.

## Decision 8: Pin official cases and test the in-scope operation items

The catalog links Decrypt to KMIPKIT-TEST-CN01-2-100 based on its HTML description; it links Encrypt to KMIPKIT-TEST-CN01-2-99, KMIPKIT-TEST-CN01-2-100, and KMIPKIT-TEST-CN01-2-101. The linked XML for case 2.100 has no Decrypt items, while case 2.101 has four Decrypt pairs even though its HTML description names Encrypt only. KMIPKIT-DISC-046 records this mismatch. The byte-identical official XML fixtures are pinned in `specification/oasis/kmip-2.1/fixtures/`; each contains a multi-operation workflow with setup/cleanup operations outside KMIPKIT-0019. Execute every in-scope operation item actually present using the exact fixture request/response data and label results as fixture-derived operation-item evidence, not complete official-case passes. Complete workflow runs belong to the 1.0 interoperability gate after all operations in each scenario are supported; table-derived boundary vectors remain separately labeled.

## Remaining uncertainty and evidence

- KMIPKIT-DISC-045: single-request Init=true/Final=true Data optionality remains unresolved between Tables 196/214 and §6.1; the typed client returns a local validation error before transmission for that form pending authoritative resolution, without choosing Data requiredness.
- The Table 59 header extraction is an excluded source-clause audit record, not an open OASIS discrepancy or a requirement to make the Cryptographic Parameters request field mandatory.
- No approved KMIP 2.1 erratum is pinned; the checked-in OASIS v2.1 Standard remains the source.
- The server may reject based on implementation support, key state, or policy; that alone does not demonstrate client payload nonconformance.
