# Data Model: KMIP 2.1 Encrypt and Decrypt Operations

## Shared request values

| Entity | Representation | Rules |
| --- | --- | --- |
| Unique Identifier | Existing typed UniqueIdentifier | Optional in request Tables 196/214. The operation model preserves omission; client batch construction/execution applies the ID Placeholder conditions for a later eligible item of an ordered batch under KMIPKIT-0006. Required in a successful response. |
| Cryptographic Parameters | Optional ordered generic Structure with validation of recognized required subfields | Preserve unknown tags, values, and order. When caller-supplied parameters identify a variable-IV Block Cipher Mode, IV Length is required; for GCM, Tag Length is required (§4.16). Do not inspect remote object state. |
| Operation Data | OperationData preserving ByteString, Enumeration, or Integer | §7.9, Tables 360–361. A redacted wrapper hides every value in Debug/Display; the ByteString variant uses the existing zeroizing secret-byte owner. |
| IV/Counter/Nonce | Optional operation byte-string value | Preserve caller input or server output exactly; do not generate or interpret locally. |
| Correlation Value | Optional opaque Byte String | Multipart continuation value from §7.8, distinct from Pending Asynchronous Correlation Value. |
| Init Indicator | Optional Boolean | Caller-controlled under §7.17. True marks the first part. |
| Final Indicator | Optional Boolean | Caller-controlled under §7.14. True marks the final part. |
| Authenticated Encryption Additional Data | Optional opaque Byte String | §7.3. When supplied for multipart Encrypt or Decrypt, it is included on the initial request. |
| Authenticated Encryption Tag | Optional opaque Byte String | §7.4. Decrypt request input; optional Encrypt response output. When supplied for multipart Decrypt, it is included on the initial request. |

OperationData must not narrow request encoding to Byte String. Encrypt and Decrypt response Data, by contrast, are Byte Strings under Tables 197 and 215. Sensitive values in all alternatives are redacted from formatting; owned byte storage follows the existing zeroizing TTLV value contract.

## Operation request entities

### EncryptRequest

Fields are encoded in the exact order of Table 214:

1. unique_identifier: Option<UniqueIdentifier>
2. cryptographic_parameters: Option<Structure>
3. data: Option<OperationData>
4. iv_counter_nonce: Option<SecretBytes>
5. correlation_value: Option<SecretBytes>
6. init_indicator: Option<bool>
7. final_indicator: Option<bool>
8. authenticated_encryption_additional_data: Option<SecretBytes>

### DecryptRequest

Fields are encoded in the exact order of Table 196:

1. unique_identifier: Option<UniqueIdentifier>
2. cryptographic_parameters: Option<Structure>
3. data: Option<OperationData>
4. iv_counter_nonce: Option<SecretBytes>
5. correlation_value: Option<SecretBytes>
6. init_indicator: Option<bool>
7. final_indicator: Option<bool>
8. authenticated_encryption_additional_data: Option<SecretBytes>
9. authenticated_encryption_tag: Option<SecretBytes>

Separate request types prevent Decrypt-only input from being encoded in Encrypt. No value is synthesized.

## Data and multipart validity matrix

§6.1 governs multipart progression. The operation tables separately mark request Data Yes for single-part and No for multi-part. Preserve these rules as an explicit matrix:

| Request position | Init Indicator | Final Indicator | Correlation Value | Data |
| --- | --- | --- | --- | --- |
| Single-part, unframed | absent | absent | absent | Required by Tables 196/214 |
| Initial multipart part | true | absent or false | absent | Optional under §6.1 |
| Middle multipart part | absent or false | absent or false | Required, copied from first response | Required under §6.1 |
| Final multipart part after an earlier part | absent or false | true | Required, copied from first response | Optional under §6.1 |
| Single request framed as both first and final | true | true | absent | Typed client returns a local validation error before transmission while KMIPKIT-DISC-045 remains open. This gates support without selecting Data requiredness. |

The first response's Correlation Value is used on every later multipart request, including the final request. Each request is one caller-driven exchange. The client does not create a stream, retain server stream state, invent a correlation value, or submit a follow-up automatically.

## Operation response entities

### Successful payloads

On Result Status = Success, the typed payload requires exactly one Unique Identifier and preserves present optional fields from the corresponding table.

- Encrypt (Table 215): data: Option<SecretBytes> (Byte String), iv_counter_nonce: Option<SecretBytes>, correlation_value: Option<SecretBytes>, authenticated_encryption_tag: Option<SecretBytes>.
- Decrypt (Table 197): data: Option<SecretBytes> (Byte String), correlation_value: Option<SecretBytes>.

Data is optional in either response table, including a single-part success. Do not reject a successful UID-only response.

### Failure and Pending outcomes

A non-success completed response retains the common KmipOperationResult and has no success payload. It is valid without Unique Identifier. A Pending response is exposed through the existing PendingOutcome shared variant, not an Encrypt/Decrypt success payload; its Asynchronous Correlation Value is not the §7.8 multipart Correlation Value. The full generic ResponseMessage remains available through the common outcome contract so unknown elements are not lost.

## Validation and security rules

1. Reject wrong known TTLV types, duplicate singleton fields, and missing required fields before transmission or typed acceptance; report errors without field values or raw bodies.
2. Preserve all three §7.9 request Data encodings and the Byte String-only response Data encoding. Round trips retain exact values and ordering.
3. Apply §4.16 IV Length and Tag Length conditions when supplied Cryptographic Parameters expose the relevant known mode. Preserve unknown parameter members. Omitted parameters or values derived from server object attributes are not guessed or rejected based on unavailable server state.
4. AAD and Decrypt AEAD Tag stay on the caller's initial multipart request; KMIPKit does not move them between parts.
5. A Result Reason may be any value allowed by shared Message Data Structures even when not listed in Tables 198/216. Preserve unknown raw codes.
6. For an object the caller knows is archived, §6.1-005 requires an explicit Recover before Encrypt or Decrypt. KMIPKIT-0018 is a blocking implementation dependency: once merged, the client can expose and execute Recover when the caller requests it. The client does not infer remote object state or recover automatically; a fake-client test covers the explicit Recover-then-Encrypt sequence after that dependency is available.
7. The local 16 MiB message-size, depth-64, and 100,000-element decoder defaults are enforced independently of server metadata.
8. Request/response byte data, AAD, AEAD Tag, and full KMIP bodies are redacted. KMIPKit-owned byte allocations are zeroized on drop; caller-retained or foreign-runtime copies are outside that ownership guarantee.

## Relationships

The client batch item owns one EncryptRequest or DecryptRequest and returns either a completed typed operation result or the shared PendingOutcome. Typed views do not replace the validated generic RequestMessage or ResponseMessage, which retain the complete ordered TTLV tree, including unknown values.
