# Data Model: KMIP 2.1 Object Lifecycle and Certificates

This feature composes with the existing TTLV item/structure, managed-object, attribute, message/batch, result, response-view, and secret-byte contracts. It introduces operation models only; it does not duplicate generic protocol entities.

## Shared entities

| Entity | Model and constraints |
|---|---|
| Operation request | Operation-specific ordered fields, optionality, and conditionality from the request table in `spec.md`. Omission remains distinct from an empty or zero value. |
| Operation response | Common result plus operation payload on success. Preserve the original response item and unrecognized structurally valid fields. Do not fabricate success fields on failure. |
| Operation result | Existing common status, reason, permitted message, Pending/correlation, extension, and request-delivery information. Reason codes stay open. |
| Sensitive object material | CSR, certificate, key/certificate object, credential, or opaque Any Object data. Use the existing zeroizing owner for KMIPKit-owned copies and redacted formatting. |
| Unique Identifier | Existing identifier representation. Do not resolve a server ID Placeholder locally or query remote state before sending. |
| Attributes and object | Existing generic or typed object/attribute model, preserving ordered children, unknown values, and caller-supplied presence. |
| Lease metadata | Returned Unique Identifier, Lease Time, and Last Change Date from §6.1.35 Table 269. Values are returned without client timer/cache state. |

## Operation shapes

| Operation | Request-specific model | Success payload | Principal local rules / source |
|---|---|---|---|
| Certify | `CertifyRequest`: optional identifier, Certificate Request Type/Value, Attributes, Protection Storage Masks | Generated certificate Unique Identifier | One certificate; type required with request value; when the request object is omitted, caller identifies the public key and supplies Certificate Type in Attributes as specified. §6.1.6, Tables 179–180. |
| Check | `CheckRequest`: optional identifier, Usage Limits Count, Cryptographic Usage Mask, Lease Time | Identifier and any policy-rejection values returned by server | Preserve recommended ordered/Stop-or-Undo batch configuration as explicit caller choice. Do not infer server policy. §6.1.7, Tables 183–184. |
| Obtain Lease | `ObtainLeaseRequest`: optional identifier | Identifier, Lease Time, Last Change Date | Preserve exact returned values; no hidden lease cache or countdown. §6.1.35, Tables 268–269. |
| Re-certify | `RecertifyRequest`: optional certificate/key identifiers, request material/type, Offset, Attributes, masks | New certificate identifier | One replacement per request; apply only source-backed date/type rules. Server replacement links remain server behavior. §6.1.45, Tables 300–301 and 298–299. |
| Re-key | `RekeyRequest`: optional identifier, Offset, Attributes, masks | Replacement key identifier | Apply Offset/date rules; replacement link behavior is server-side. §6.1.46, Tables 305–306 and 303–304. |
| Re-key Key Pair | `RekeyKeyPairRequest`: optional private-key identifier, Offset, common/private/public Attributes and masks | New private and public identifiers | Preserve separate attribute/mask sets and source date behavior; links remain server-side. §6.1.47, Tables 310–311 and 308–309. |
| Re-Provision | `ReProvisionRequest`: certificate-request form, certificate form, or parameterless request | Response fields by Table 314 | Preserve three maintainer-approved paths; do not correct Table 315 in the pinned source or treat its caption as another operation. §6.1.48, Tables 313–315. |
| Register | `RegisterRequest`: required Object Type, Attributes, and Any Object; optional Protection Storage Masks | Registered object Unique Identifier | Apply table and catalog rules for managed-object attributes and algorithm/length/signature attributes. §6.1.43, Tables 291–292. |
| Revoke | `RevokeRequest`: optional identifier, required Revocation Reason, optional Compromise Occurrence Date | Revoked object Unique Identifier | SHOULD date for key/CA compromise; SHALL NOT date for other reasons. Preserve reason values. §6.1.44, Tables 295–297. |

## Invariants and validation

1. Serialization follows pinned operation-table field order and does not emit absent optional values.
2. Constructors and serialization validate only caller-local requirements in the feature ledger. They do not preflight object type, key status, policy, permissions, or server state.
3. Request/response table requiredness, conditionality, and multiplicity remain distinct; malformed successful response shapes produce sanitized errors while retaining the generic source item.
4. KMIP Failure/Pending is a result, not a client model error. Unknown reasons and future enum values remain available.
5. Each convenience method submits one explicit request. Typed batches preserve order and per-item correlation. No hidden Poll, retry, timer, follow-up Get, or batch rewrite is allowed.
6. Lease values are returned to the caller; KMIPKit does not track application use within a lease period.
7. Tests map every stable requirement ID and mandatory/conditional payload row to its operation test and exact source table in [traceability.md](traceability.md).

## State transitions

No client-maintained object lifecycle state is introduced. Re-key, Re-certify, Register, Revoke, Check, and lease results report the server's per-request outcome; they do not mutate local object state or infer links.
