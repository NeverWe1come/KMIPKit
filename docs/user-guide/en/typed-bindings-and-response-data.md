# Typed bindings, validation, and response data

KMIPKit provides typed views for common KMIP structures alongside generic TTLV access. The generic path remains available for repeated fields, values unknown to this library version, and vendor extensions. An API being present does not by itself establish that a server supports a capability, that a profile is conformant, or that interoperability has been demonstrated.

[Versión en español](../es/enlaces-tipados-validacion-respuestas.md).

## Typed attributes

Use the catalogued attribute definition to construct or check a direct attribute. Typed wrappers verify the assigned tag and permitted TTLV value type; they do not remove the generic representation.

| Language | Construct or check one attribute | Add and read occurrences in an Attribute Set |
| --- | --- | --- |
| Rust | <code>Attribute::new(value)</code> or <code>Attribute::from_item(item)</code> with a catalog marker type | <code>AttributeSet::try_push_typed(attribute)</code> and <code>typed::&lt;Definition&gt;()</code> |
| C | <code>kmipkit_typed_attribute_from_value(...)</code> or <code>kmipkit_typed_attribute_from_item(...)</code> | <code>kmipkit_attribute_set_try_push_typed(...)</code>, <code>kmipkit_attribute_set_typed_count(...)</code>, and <code>kmipkit_attribute_set_typed_at(...)</code> |
| Java | <code>TypedAttribute.create(...)</code> or <code>TypedAttribute.fromItem(...)</code> | <code>AttributeSet.tryPushTyped(...)</code>, <code>typedCount(...)</code>, <code>typedAt(...)</code>, or <code>typed(...)</code> |
| Python | <code>TypedAttribute.create(...)</code> or <code>TypedAttribute.from_item(...)</code> | <code>AttributeSet.try_push_typed(...)</code>, <code>typed_count(...)</code>, <code>typed_at(...)</code>, or <code>typed(...)</code> |

Attribute Set insertion preserves wire order and repeated occurrences. It does not enforce operation-specific cardinality where the standard makes that conditional on context. Keep generic Items available for vendor attributes and unrecognized fields. Rust uses catalog marker types; C, Java, and Python use the generated Standard Attribute catalog.

See [Attribute operations](attribute-operations.md) for attribute definitions and operation-specific use.

## Managed objects and key structures

The typed models cover the nine standard managed object types and the Key Block, Key Value, Key Material, and Key Wrapping Data structures. Use typed constructors and readers for recognized fields, and retain generic TTLV Items for fields that the application does not interpret.

| Language | Build from fields | Read or inspect fields |
| --- | --- | --- |
| Rust | Use the object-specific constructors and <code>KeyBlock::try_new(...)</code> builder; <code>KeyMaterial::byte_string(...)</code> and <code>KeyMaterial::structure(...)</code> cover key material forms. | <code>ManagedObject::try_from_item(...)</code>, <code>KeyBlock::try_from_item(...)</code>, <code>KeyValue::try_from_item(...)</code>, <code>KeyMaterial::try_from_item(...)</code>, <code>KeyWrappingData::try_from_item(...)</code>; use <code>ManagedValueReader</code> for tag-based reads. |
| C | <code>kmipkit_managed_object_from_fields(...)</code>, <code>kmipkit_key_block_from_fields(...)</code>, <code>kmipkit_key_value_from_fields(...)</code>, <code>kmipkit_key_material_from_fields(...)</code>, and <code>kmipkit_key_wrapping_data_from_fields(...)</code>; object-specific constructors are also available. | The corresponding <code>..._from_item(...)</code> readers and <code>kmipkit_managed_value_field_read(...)</code> / <code>kmipkit_managed_value_field_item_read(...)</code>. |
| Java | <code>ManagedObject.fromFields(...)</code>, object-specific helpers such as <code>ManagedObject.symmetricKey(...)</code>, and <code>KeyBlock.fromFields(...)</code>, <code>KeyValue.fromFields(...)</code>, <code>KeyMaterial.fromFields(...)</code>, <code>KeyWrappingData.fromFields(...)</code>. | The corresponding <code>fromItem(...)</code> readers, named scalar readers, and <code>fieldItem(rawTag)</code>. |
| Python | <code>ManagedObject.from_fields(...)</code>, object-specific helpers such as <code>ManagedObject.symmetric_key(...)</code>, and <code>KeyBlock.from_fields(...)</code>, <code>KeyValue.from_fields(...)</code>, <code>KeyMaterial.from_fields(...)</code>, <code>KeyWrappingData.from_fields(...)</code>. | The corresponding <code>from_item(...)</code> readers, named scalar readers, and <code>field_item(raw_tag)</code>. |

The foreign <code>fromFields</code> / <code>from_fields</code> constructors copy caller-supplied field Items and order recognized fields according to the KMIP structure table, regardless of the order in the input list. Typed readers validate known fields while retaining the original TTLV tree, including unknown children, their order, and repeated values. Rust builders likewise emit defined fields in their specified order. A wrapped Key Value is represented as wrapped bytes; it is not exposed as plaintext Key Material.

The managed-value field accessors read a direct child by its raw tag. To inspect a deeper descendant, continue through the returned Structure or TTLV view. This permits applications to reach vendor-tagged nodes without requiring KMIPKit to understand their semantics. See [Managed objects and key material](managed-objects-and-key-material.md) for the model details.

## Ownership and secret data

Follow the lifetime rules of each binding:

- Rust owned model values are dropped normally; borrowed field or Structure views cannot outlive their owner or callback.
- C constructors return handles that the caller must release with the matching release function. Borrowed pointers, such as P Source in the Cryptographic Parameters fields record, remain valid only while their owner handle is alive.
- Java native-backed owners implement <code>AutoCloseable</code>; close each owner and each independently owned child Item or Structure when finished. Byte arrays returned to Java are copies owned by the application.
- Python native-backed owners provide <code>close()</code> and context-manager support; close returned child Items, Structures, and typed wrappers when finished. Python bytes are immutable application-owned copies.

KMIPKit clears secret buffers that it owns when their owning object is released. Copies returned to Java or Python, or copied onward by application code, are outside KMIPKit's deterministic zeroization guarantee. Error messages and debug/string representations are designed to avoid printing secret values or complete KMIP messages.

## Cryptographic Parameters

Cryptographic Parameters retain raw Enumeration values, including values not known to this library version. Builders emit known members in KMIP 2.1 Table 59 order. Accessors distinguish an absent field from a present zero or <code>false</code> value. The modeled fields are Block Cipher Mode, Padding Method, Hashing Algorithm, Key Role Type, Digital Signature Algorithm, Cryptographic Algorithm, Random IV, IV Length, Tag Length, Fixed Field Length, Invocation Field Length, Counter Length, Initial Counter Value, Salt Length, Mask Generator, Mask Generator Hashing Algorithm, P Source, and Trailer Field.

| Language | Construct or parse | Read fields | Release |
| --- | --- | --- | --- |
| Rust | <code>CryptographicParametersBuilder::new()</code> or <code>CryptographicParameters::try_from_ttlv(...)</code> | <code>block_cipher_mode()</code>, <code>random_iv()</code>, <code>p_source()</code>, and the other Table 59 getters | Drop the owned value |
| C | Fill <code>kmipkit_cryptographic_parameters_fields_v1_t</code> and call <code>kmipkit_cryptographic_parameters_from_fields_v1</code>, or call <code>kmipkit_cryptographic_parameters_from_ttlv</code> | <code>kmipkit_cryptographic_parameters_read_fields_v1</code> fills a caller-owned v1 record | <code>kmipkit_cryptographic_parameters_release</code> |
| Java | <code>CryptographicParameters.builder()</code> or <code>fromTtlv(...)</code> | <code>blockCipherMode()</code>, <code>randomIv()</code>, <code>pSource()</code>, and the other Table 59 getters | <code>close()</code> |
| Python | <code>CryptographicParameters.builder()</code> or <code>from_ttlv(...)</code> | <code>block_cipher_mode()</code>, <code>random_iv()</code>, <code>p_source()</code>, and the other Table 59 getters | <code>close()</code> or a <code>with</code> block |

The C P Source pointer in the fields record is borrowed from the live Cryptographic Parameters handle. Java and Python return copies of P Source bytes, while Rust lends the bytes only for the accessor's callback lifetime. Copies made by the application follow the ownership and zeroization rules above.

## Request Header Authentication and batch parameters

KMIP Authentication is separate from TLS client authentication. It is attached to one request message in the Request Header and applies to that message's Batch Items. It is not retained as global Client configuration.

| Language | Create Authentication | Attach to a batch |
| --- | --- | --- |
| Rust | <code>Authentication::new(credentials)</code> | <code>ClientBatch::with_authentication(authentication)</code> |
| C | <code>kmipkit_authentication_create(...)</code> | <code>kmipkit_client_batch_set_authentication(batch, authentication)</code> |
| Java | <code>Authentication.create(credentials)</code> | <code>ClientBatch.setAuthentication(authentication)</code> |
| Python | <code>Authentication.create(credentials)</code> | <code>Batch.set_authentication(authentication)</code> |

Successful attachment transfers ownership of the Authentication value to the batch. If an attachment call is rejected, ownership remains with the caller. Close or release values using the binding's normal rules. KMIPKit does not imply that a server will accept a credential, and it does not silently reuse credentials in later messages.

Cryptographic Parameters and other operation-specific parameters belong to the relevant operation request structure. Set them per request; do not treat them as connection-wide settings. For TLS setup, see [Production transports](production-transports.md).

## Validate before sending

A validation report contains safe issue metadata: a code, path, severity, and normative source reference. It omits rejected field values, credentials, and the request body.

| Language | Validate a request | Validate a generic Structure |
| --- | --- | --- |
| Rust | <code>ClientRequest::validation_report()</code> | <code>validation::validate_structure(structure, schema)</code> |
| C | <code>kmipkit_client_batch_item_validation_report(...)</code> | <code>kmipkit_ttlv_structure_validation_report_v1(...)</code> |
| Java | <code>ValidationReport.validateRequest(item)</code> | <code>ValidationReport.validateStructure(structure, schema)</code> |
| Python | <code>validation.validate_request(item)</code> | <code>validation.validate_structure(structure, schema=...)</code> |

For a generic Structure, omit the schema only when no operation-specific schema is known. KMIPKit does not guess the operation from field values. Supplying a supported schema identity enables the corresponding structural checks.

Client execution runs shared request checks before writing. An unconditional violation that can be proved locally fails with delivery state <code>NotSent</code> and writes zero request bytes. Checks that require server capabilities, authorization, policy, or current object state remain server decisions. <code>NotSent</code> describes the local delivery state; it does not imply that a server evaluated the request.

## Typed responses and decoded TTLV access

Use the operation-specific response reader when one is available, then use the generic decoded-tree view for fields outside that model. The Response Payload may be absent or present but empty; these states remain distinct. The Response Header can be read independently.

| Language | Response Payload | Response Header | Lifetime |
| --- | --- | --- | --- |
| Rust | <code>ClientResponseView::raw()</code> returns an optional borrowed <code>StructureView</code> | <code>response_header_raw()</code> returns a borrowed view | Views borrow the response owner |
| C | <code>kmipkit_client_batch_item_response_raw_ttlv(...)</code> returns a presence flag and optional owned Structure handle | <code>kmipkit_client_batch_item_response_header_ttlv(...)</code> | Release returned handles with <code>kmipkit_ttlv_structure_release</code> |
| Java | <code>ClientBatchItemResult.raw()</code> returns <code>Optional&lt;TtlvStructure&gt;</code> | <code>responseHeaderTtlv()</code> returns <code>TtlvStructure</code> | Returned structures are caller-owned; close them |
| Python | <code>BatchItemResult.raw()</code> returns <code>Structure</code> or <code>None</code> | <code>response_header_ttlv()</code> returns <code>Structure</code> | Returned structures are caller-owned; close them |

The accessors expose the decoded TTLV tree, preserving item order, repetitions, item types, unknown tags, and nested vendor tags such as <code>0x54xxxx</code>. A managed object or key wrapper's <code>fieldItem(rawTag)</code> / <code>field_item(raw_tag)</code> returns an owned child Item with its complete subtree. Continue navigating that subtree through the generic TTLV APIs to inspect arbitrary descendants, including fields nested inside Key Block or Server Information. Rust views borrow from the response; C, Java, and Python return owned handles or structures that the caller releases or closes.

The decoder does not retain padding bytes, so <code>raw()</code> is a view or copy of the decoded TTLV tree, not a byte-for-byte reproduction of the network frame. Do not log response bodies or include them in errors.

### Typed Query response fields

All four language bindings expose Rust-classified fields for the standard KMIP 2.1 Query response. Java, Python, and C read the classification supplied by Rust rather than independently guessing field types.

| Language | Typed entry point | Typical typed access | Generic TTLV access |
| --- | --- | --- | --- |
| Rust | <code>ClientResponseView::query()</code> returns an optional <code>QueryResponse</code> | <code>operations()</code>, <code>object_types()</code>, <code>response_fields()</code>, <code>unknown_items()</code> | <code>to_ttlv_payload()</code> rebuilds the ordered generic payload |
| C | <code>kmipkit_client_batch_item_response_query_field_count(...)</code> and <code>kmipkit_client_batch_item_response_query_field_at(...)</code> | Read tag, field kind, and raw Enumeration; non-Enumeration results return an owned TTLV Item | <code>kmipkit_client_batch_item_response_query_ttlv(...)</code> |
| Java | <code>ClientBatchItemResult.typedQueryResponse()</code> returns an optional <code>QueryResponse</code> | <code>operations()</code>, <code>applicationNamespaces()</code>, <code>fieldAt(index)</code>, <code>unknownItems()</code> | <code>rawView()</code>; <code>ClientBatchItemResult.queryResponse()</code> returns the owned payload Structure |
| Python | <code>BatchItemResult.typed_query_response()</code> returns <code>QueryResponse</code> or <code>None</code> | <code>operations</code>, <code>application_namespaces</code>, <code>field_at(index)</code>, <code>unknown_items</code> | <code>raw_view()</code>; <code>BatchItemResult.raw()</code> returns the generic payload Structure |

The C field-kind output uses <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_ENUMERATION</code>, <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_TEXT_STRING</code>, <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_STRUCTURE</code>, or <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_UNKNOWN</code>. Enumeration values use the raw-value output; other kinds return an owned Item that the caller releases.

Query field views expose the raw tag and Rust-selected field kind. Enumeration access preserves the unsigned value, including values added after this KMIPKit release. Java and Python collection accessors return matching occurrences in received order; their singular optional conveniences expose at most their first matching field. Use <code>fieldAt(index)</code> / <code>field_at(index)</code> or C's indexed accessor to retain the complete interleaved field order, including repeated fields; an unknown field remains available as its original generic TTLV Item. In C, callers release returned Items with <code>kmipkit_ttlv_item_release</code>. The independent Java <code>rawView()</code> / Python <code>raw_view()</code> view retains the full decoded payload and nested vendor fields. Close Java <code>QueryResponse</code> and returned native-backed views; use Python's context manager or <code>close()</code> and close returned views as well.

Rust exposes the same classified members through <code>QueryResponseField</code>, typed convenience accessors, and <code>unknown_items()</code>. Its <code>to_ttlv_payload()</code> reconstructs the field sequence, retaining unknown Items and nested Structures. The query execution example in [Query and Ping](query-ping.md) shows how to obtain a Rust response. These API descriptions do not establish server support or interoperability.

The high-level API does not expose transport byte exchange. Asynchronous outcomes remain explicit, and KMIPKit does not poll or retry a request automatically. See [Client execution](client-execution.md) for operation calls and delivery outcomes.
