# Data Model: KMIP 2.1 Hash, MAC, and Signature Operations

This model composes with KMIPKIT-0019's `OperationData`, generic ordered Cryptographic Parameters `Structure`, byte-string ownership, and Correlation/Init/Final field contracts. KMIPKIT-0021 does not introduce duplicate shared types.

## Shared request concepts

| Entity | Model | Rules |
| --- | --- | --- |
| Cryptographic Parameters | Optional or required ordered `Structure` per operation | Hash requires the field. MAC, MAC Verify, Sign, and Signature Verify permit omission. Preserve order, unknown members, and raw enumeration values. Do not inspect remote key attributes. |
| Unique Identifier | Optional existing `UniqueIdentifier` model | MAC, MAC Verify, Sign, and Signature Verify tables mark it optional. When absent, keep it absent on the wire. Server ID Placeholder resolution is not client behavior. |
| Operation Data | Shared KMIPKIT-0019 `OperationData` | Preserve the §7.9 Byte String, Enumeration, or Integer form in requests where Data is named. Hash response Data is Byte String. |
| MAC Data | Opaque Byte String (§7.20) | MAC response output or MAC Verify input; single-part required by the relevant table, absent for multipart. Redact formatting and reuse zeroizing byte ownership. |
| Signature Data | Opaque Byte String (§7.38) | Sign response output or Signature Verify input; single-part required by the relevant table, absent for multipart. Redact formatting and reuse zeroizing byte ownership. |
| Digested Data | Opaque Byte String field named by operation Tables 334 and 337 | Sign may supply it instead of Data for a single-part operation; Signature Verify may supply it as verification input. Keep its exact tag/value and do not digest locally. |
| Correlation Value | Shared opaque byte-string field | Optional source table field; preserve exact value and pass it only when the caller constructs a later explicit multipart request. |
| Init/Final Indicators | Shared optional Boolean fields | Caller controls each part. KMIPKit does not store stream state or synthesize indicators/correlation. |
| Validity Indicator | Open raw Enumeration (§11.61) | Preserve raw value. Name standard values Valid (1), Invalid (2), Unknown (3), and the extension range without rejecting future values. |

## Operation request/response shape

| Operation | Request model | Successful response model |
| --- | --- | --- |
| Hash | Required Cryptographic Parameters; optional Data, Correlation Value, Init Indicator, Final Indicator. Table 235 requires Data for single-part and says no Data for multi-part requests. | Data and Correlation Value optional by Table 236. Data is present for single-part and absent for multi-part according to the table. |
| MAC | Optional Unique Identifier, Cryptographic Parameters, Data, Correlation Value, Init Indicator, Final Indicator. Table 259 requires Data for single-part and says no Data for multi-part requests. | exactly one Unique Identifier and MAC Data per Table 260; MAC Data is present for single-part and absent for multi-part. Correlation Value is optional. |
| MAC Verify | Optional Unique Identifier, Cryptographic Parameters, original Data, MAC Data, Correlation Value, Init Indicator, Final Indicator. Table 262 requires MAC Data for single-part and says no MAC Data for multi-part requests. | exactly one Unique Identifier and optional Validity Indicator by Table 263; Validity Indicator is required for single-part, with the final multipart rule governed by open `KMIPKIT-DISC-048`. Correlation Value is optional. |
| Sign | Optional Unique Identifier, Cryptographic Parameters, Data, Digested Data, Correlation Value, Init Indicator, Final Indicator. Table 334 requires Data for single-part unless Digested Data is supplied; Data is absent for multi-part requests. | exactly one Unique Identifier and Signature Data by Table 335; Signature Data is present for single-part and absent for multi-part. Correlation Value is optional. |
| Signature Verify | Optional Unique Identifier, Cryptographic Parameters, Data, Digested Data, Signature Data, Correlation Value, Init Indicator, Final Indicator. Table 337 requires Signature Data for single-part and says no Signature Data for multi-part requests. | exactly one Unique Identifier, optional Validity Indicator, optional recovered Data, and optional Correlation Value by Table 338. The final multipart Validity Indicator is governed by open `KMIPKIT-DISC-048`. |

Request and response table statements describe field occurrence and server output. Common request validation may reject missing single-part input or invalid field types before transmission, but it does not make claims about unknown server key attributes, cryptographic support, or policy. Failure results use the shared `KmipOperationResult`; no successful fields are fabricated on failure.

## Validation and state rules

1. Hash requires Cryptographic Parameters in every request. For a single-part Hash, Data is required; for multi-part requests Data is absent as the table specifies.
2. MAC requires Data for single-part and omits it for multipart. MAC Data follows the same required-single/absent-multipart rule for MAC Verify.
3. Sign single-part input has Data unless Digested Data is supplied; multipart Data is absent by Table 334. Do not create Digested Data or hash the caller's bytes locally.
4. Signature Verify single-part input includes Signature Data; multipart Signature Data is absent by Table 337. Original Data, Digested Data, and Cryptographic Parameters retain the caller's presence/absence.
5. An omitted Unique Identifier remains omitted. The client never resolves an ID Placeholder or substitutes a local identifier.
6. Each typed call performs one request/response exchange. A later multipart part is a new explicit call built by the caller with the returned Correlation Value when supplied.
7. Operation result status, reason, allowed Result Message, and any Pending state are preserved by shared response contracts. No retry, poll, or follow-up operation is automatic.
8. MAC Verify and Signature Verify single-part responses require Validity Indicator in their successful typed shape. On final multipart responses accept either presence or absence because of `KMIPKIT-DISC-048`. On non-final multipart responses, an indicator is a typed response-shape error; generic TTLV remains available for inspection.
9. Verification values Invalid and Unknown are operation results, not transport errors. Unknown and extension enum codes survive round trips.
10. Successful MAC, MAC Verify, Sign, and Signature Verify response decoding requires exactly one well-formed Unique Identifier; missing, duplicate, or malformed values produce a sanitized typed response-shape error while generic response access remains available.
11. Debug, Display, and sanitized errors do not reveal any operation byte strings or Cryptographic Parameters contents. KMIPKit-owned byte storage uses the existing zeroizing owner.

## Open discrepancy

The MAC Verify and Signature Verify prose requires Validity Indicator in a single-part or final multipart response and prohibits it in a non-final response. Tables 263 and 338 say `Yes for single-part. No for multi-part.` Because the final-part behavior is unresolved, the typed response accepts both final forms and makes no server-conformance claim for that field while `KMIPKIT-DISC-048` is open.
