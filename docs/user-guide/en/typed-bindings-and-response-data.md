# Typed bindings, validation, and response data

KMIPKit offers typed models alongside generic TTLV access. The generic path
remains useful for repeated values, unknown values, and vendor extensions.
These APIs describe how to construct and inspect data; their presence alone
does not establish server support, profile conformance, certification, or
interoperability.

## Typed attributes and managed objects

The Rust reference models are documented in [Attribute operations](attribute-operations.md)
and [Managed objects and key material](managed-objects-and-key-material.md).
The corresponding foreign wrappers use each language's normal naming and
ownership conventions:

| Capability | C | Java | Python |
| --- | --- | --- | --- |
| Check a typed attribute Item | `kmipkit_typed_attribute_from_item` | `TypedAttribute.fromItem(...)` | `TypedAttribute.from_item(...)` |
| Read a managed object or key structure | `kmipkit_managed_object_from_item`, `kmipkit_key_block_from_item`, `kmipkit_key_value_from_item`, `kmipkit_key_material_from_item`, `kmipkit_key_wrapping_data_from_item` | `ManagedObject.fromItem(...)`, `KeyBlock.fromItem(...)`, `KeyValue.fromItem(...)`, `KeyMaterial.fromItem(...)`, `KeyWrappingData.fromItem(...)` | `ManagedObject.from_item(...)`, `KeyBlock.from_item(...)`, `KeyValue.from_item(...)`, `KeyMaterial.from_item(...)`, `KeyWrappingData.from_item(...)` |

Typed wrappers validate the structure they claim to represent. Keep the
underlying Item available when an application must inspect a tag or child it
does not recognize. Repeated and unknown TTLV children are not discarded just
because a typed view is used. C callers release returned handles; Java
wrappers implement `AutoCloseable`; Python owners implement `close()` and
context-manager methods. Rust borrows used by view accessors are tied to their
owner.

## Cryptographic Parameters

Cryptographic Parameters retain their raw Enumeration values, including
values unknown to this version. Builders emit the known members in KMIP 2.1
Table 59 order. The field accessors report absence separately from a present
zero or `false` value.

| Language | Construct or parse | Read fields | Release |
| --- | --- | --- | --- |
| Rust | `CryptographicParametersBuilder::new()` or `CryptographicParameters::try_from_ttlv(...)` | `block_cipher_mode()`, `random_iv()`, `p_source()`, and the other Table 59 getters | Drop the owned value |
| C | Fill `kmipkit_cryptographic_parameters_fields_v1_t` and call `kmipkit_cryptographic_parameters_from_fields_v1`, or call `kmipkit_cryptographic_parameters_from_ttlv` | `kmipkit_cryptographic_parameters_read_fields_v1` fills a caller-owned v1 record | `kmipkit_cryptographic_parameters_release` |
| Java | `CryptographicParameters.builder()` or `fromTtlv(...)` | `blockCipherMode()`, `randomIv()`, `pSource()`, and the other Table 59 getters | `close()` |
| Python | `CryptographicParameters.builder()` or `from_ttlv(...)` | `block_cipher_mode()`, `random_iv()`, `p_source()`, and the other Table 59 getters | `close()` or a `with` block |

The C `p_source` pointer returned in the fields record is borrowed from the
live Cryptographic Parameters handle. It is read-only and remains valid only
while that handle is live. Java returns a caller-owned `byte[]`; Python returns
caller-owned immutable `bytes`; Rust lends P Source bytes to a callback. The
Java and Python copies, and any copies made by a callback or application, are
outside KMIPKit's deterministic zeroization guarantee. KMIPKit clears only
secret storage it owns.

## Request Header Authentication

KMIP `Authentication` and TLS client authentication are separate. KMIP
Authentication is attached to one request message and appears once in its
Request Header; it applies to every Batch Item in that message. It is not
stored in `Client` configuration.

| Language | Attach Authentication |
| --- | --- |
| Rust | `ClientBatch::with_authentication(authentication)` |
| C | `kmipkit_client_batch_set_authentication(batch, authentication)` |
| Java | `ClientBatch.setAuthentication(authentication)` |
| Python | `Batch.set_authentication(authentication)` |

The attachment consumes the Authentication value on success. On a rejected
call, ownership remains with the caller. Close or release values according to
their language's ownership rules, and do not reuse credentials implicitly in
later exchanges. For TLS setup, see [Production transports](production-transports.md).

## Validate before sending

The request validator returns a report containing safe issue metadata:
code, path, severity, and normative source reference. It omits the rejected
values and request body. A typed request report can be obtained with:

| Language | Typed request report | Generic Structure report |
| --- | --- | --- |
| Rust | `ClientRequest::validation_report()` | `validation::validate_structure(structure, schema)` |
| C | `kmipkit_client_batch_item_validation_report(...)` | `kmipkit_ttlv_structure_validation_report_v1(...)` |
| Java | `ValidationReport.validateRequest(item)` | `ValidationReport.validateStructure(structure, schema)` |
| Python | `validation.validate_request(item)` | `validation.validate_structure(structure, schema=...)` |

For a generic Structure, omit the schema only when no operation-specific
schema is known. KMIPKit does not guess the operation from field values.
Providing a supported schema identity enables the corresponding structural
checks.

Client execution also runs the shared request checks before writing. An
unconditional violation that can be proved locally fails with delivery state
`NotSent` and writes no request bytes. Checks that require server capabilities,
authorization, policy, or current object state remain server decisions.

## Typed response and decoded TTLV access

Read the typed operation result first, then use the generic decoded-tree view
when an application needs fields outside that typed result. The response
payload may be absent or present-but-empty; the APIs preserve that distinction.
The Response Header is available independently.

| Language | Response Payload | Response Header | Lifetime |
| --- | --- | --- | --- |
| Rust | `ClientResponseView::raw()` returns an optional borrowed `StructureView` | `response_header_raw()` returns a borrowed view | Views borrow the response owner |
| C | `kmipkit_client_batch_item_response_raw_ttlv(...)` returns a presence flag and optional owned Structure handle | `kmipkit_client_batch_item_response_header_ttlv(...)` | Release returned handles with `kmipkit_ttlv_structure_release` |
| Java | `ClientBatchItemResult.raw()` returns `Optional<TtlvStructure>` | `responseHeaderTtlv()` returns `TtlvStructure` | Returned structures are caller-owned; close them |
| Python | `BatchItemResult.raw()` returns `Structure` or `None` | `response_header_ttlv()` returns `Structure` | Returned structures are caller-owned; close them |

These accessors expose the decoded TTLV tree, preserving item order,
repetitions, value types, unknown tags, and nested vendor tags. The decoder
does not retain padding bytes, so the tree is not a byte-for-byte copy of the
network frame. Java and Python return owned structures that callers close;
Rust's view borrows the response for its lifetime. C callers must release each
returned handle.

The high-level API does not expose transport byte exchange. Operation results
and asynchronous outcomes remain explicit; KMIPKit does not poll or retry a
request automatically. See [Client execution](client-execution.md) for
operation calls and delivery outcomes.
