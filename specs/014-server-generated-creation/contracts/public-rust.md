# Public Rust Contract: Server-Generated Object Creation

## Typed request models

- CreateRequest names an explicit Object Type and contains the required outer Attributes structure from Table 186 plus optional protection/storage inputs. The Attributes structure may be empty under §5.1/Table 157.
- CreateKeyPairRequest keeps Common Attributes, Private Key Attributes, and Public Key Attributes separate and exposes every Table 189 field.
- CreateSplitKeyRequest exposes every Table 193 field and the optional source Unique Identifier; its required outer Attributes structure may be empty under §5.1/Table 157.
- Each request has one closed ClientRequest variant and can be placed in an ordered ClientBatch. No API accepts raw bytes or caller-defined payload conversion.
- AttributeSet contains ordered generic TTLV `Item`s for the direct §4 Object Attribute items defined by §§5.1–5.4, Tables 157–160. Each item retains its tag and typed value; repeated and unknown items survive without tag synthesis. The distinct §4.60 Vendor Attribute structure uses Table 150 and includes required Vendor Identification. Outbound validation continues to follow assigned-value and registered-extension policy.
- Public Debug output and diagnostic context for attribute groups redact item contents; value bytes are never formatted as contents.

## Client entry points and outcomes

- Client::execute(ClientBatch, limits) remains the one-exchange writer for multi-item requests.
- Client::create, Client::create_key_pair, and Client::create_split_key are typed single-operation conveniences. Each builds a one-item ClientBatch, calls Client::execute exactly once, and returns that item's ClientBatchItemResponse.
- ClientBatchOutcome gains typed outcomes for CreateResponse, CreateKeyPairResponse, and CreateSplitKeyResponse. A valid non-success Result is retained as an operation result and has no fabricated successful payload.
- A successful Result with a missing or malformed required payload is a sanitized protocol error.
- PendingOutcome is operation-agnostic. It retains the originating operation identity, exact KMIP Result, and required Asynchronous Correlation Value; it is not represented using DiscoverVersionsResponse. Pending is never converted into a completed creation result.
- Multi-operation batching remains available through Client::execute and preserves request/response association and caller-selected batch options.
- A batch containing CreateSplitKey includes Maximum Response Size set to `min(limits.max_message_bytes(), i32::MAX)`, converted to the KMIP Integer encoding. The client independently rejects an over-limit response before TTLV decoder entry, even if the peer ignores the advertised value.

## Validation and errors

- Locally invalid structures fail before dispatch and report NotSent.
- Table 191 same-value constraints apply to Cryptographic Algorithm, Cryptographic Length, Cryptographic Domain Parameters, and Cryptographic Parameters. For each key, its key-specific value overrides Common Attributes; otherwise the common value applies. Both effective values absent remain unspecified; exactly one present or two differing values are rejected; equal effective values are accepted, including equal overrides that differ from Common Attributes.
- For Create Split Key, Table 193 marks request Prime Field Size optional. Requiring an explicit caller value for Polynomial Sharing Prime Field is a KMIPKit client restriction; §2.8/Table 9 describes the Split Key object and is not cited as a request-field MUST.
- The client preserves Common, Private, and Public groups exactly and never copies values across groups. Only the server applies the Table 189 union behavior.
- The request preserves Common, Private, and Public attribute groups and repeated values. The server applies the Table 189 union rule.
- Create Split Key preserves caller-requested attributes as request input. The client does not claim those are the effective attributes when an input key may take precedence.
- Transport failures retain the existing delivery state. Valid KMIP operation failures are returned without retry.
- Debug, Display, errors, and logs redact secret-bearing values and raw bodies. KMIPKit-owned sensitive buffers retain existing zeroization behavior.

## Compatibility

This is an additive pre-1.0 Rust API. The public API manifest and generated C, Java, and Python surfaces are updated by the later API parity specification after operation APIs are frozen. This feature does not modify generated files.
