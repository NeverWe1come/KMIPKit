# Data Model: KMIP 2.1 Interop and Diagnostics

Sources: OASIS KMIP Specification v2.1 §§6.1.26, 6.1.29, 6.1.57; Tables 241–243, 250–252, 340–342; §11.36. Stable mappings are in traceability.md.

## Interop

| Direction | Field | Cardinality | Contract |
|---|---|---|---|
| Request | Interop Function | exactly 1 | Enumeration; use an open/lossless value representation. |
| Request | Interop Identifier | exactly 1 | Text String. |
| Success response | Operation payload | empty | Preserve common result; do not fabricate payload fields. |

Interop's “not available in a production server” clause constrains server implementation only. It is not a client precondition or KMIPKit server feature.

## Log

| Direction | Field | Cardinality | Contract |
|---|---|---|---|
| Request | Log Message | exactly 1 | Text String. Caller must not submit secrets; always redact the value in diagnostics. |
| Success response | Operation payload | empty | Preserve common result; no operation-specific payload. |

Use Table 252 per approved KMIPKIT-DISC-017.

## Validate

| Direction | Field | Cardinality | Contract |
|---|---|---|---|
| Request | Certificate | 0..many | Repeated Certificate object values; preserve relative order within the Certificate list. |
| Request | Unique Identifier | 0..many | Repeated Managed Certificate identifiers; preserve relative order within the identifier list. |
| Request | Validity Date | 0..1 | Date-Time; preserve absence and supplied value. Section 6.1.57 passively states that the current date and time SHALL be assumed if omitted; the source actor is unspecified and date evaluation is unverified. |
| Success response | Validity Indicator | exactly 1 | Enumeration with unknown/future values preserved. |

One Validate request represents exactly one certificate chain; its two repeated field lists are the chain material. Read the §6.1.57 paragraph 1 SHALL with paragraph 2 and Table 340: the two lists jointly contain at least one member, while either individual list may be omitted and either may repeat. A Validity Date alone does not provide a chain member. Section 10.1.2, unnumbered Type table Structure row requires Table 340 wire order: Certificate list, Unique Identifier list, optional Validity Date. Each list preserves its own relative order. Certificate inputs follow section 2.1, Table 2 with Certificate Type before Certificate Value. This is request-representation structure only; do not construct or cryptographically validate the chain, choose trust anchors, impose server policy, or infer server acceptance.

## Shared behavior

All operations preserve common Result Status/Reason/permitted Message, Pending and correlation, delivery state, unknown values, and generic TTLV. Each explicit call performs one exchange without retry or implicit Poll. Secrets and raw payloads are redacted.
