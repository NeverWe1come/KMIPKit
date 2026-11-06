# Key derivation and transfer

KMIPKit sends Derive Key, Export, Import, and Join Split Key requests to a
KMIP 2.1 server. These operations follow OASIS KMIP Specification Version 2.1
§6.1.14 Tables 205–207 and §7.12 Table 364, §6.1.18 Tables 217–219,
§6.1.25 Tables 238–240, and §6.1.27 Tables 244–246. Each explicit client
method makes one exchange. KMIPKit does not derive keys, wrap or unwrap
objects, import material locally, query object attributes, or retry requests.
The caller selects Cryptographic Parameters; KMIPKit preserves the supplied
values, including Block Cipher Mode, and does not infer cryptographic state
from a remotely stored source object. Context-dependent derivation conditions
are evaluated by the KMIP server.

## Prepare caller-controlled inputs

Choose the source objects, derivation method, output attributes, replacement
policy, export selectors, and Split Key context according to your application
policy. The example below uses synthetic bytes only; it does not contain a
usable key or password. Its helpers build the attributes and generic KMIP
object passed to Import.

Before calling Derive Key, select source objects whose Cryptographic Usage
Mask includes Derive Key. KMIPKit sends the supplied identifiers and does not
query remote attributes to verify this caller-selected policy.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientBatchItemResponse, ClientError};
use kmipkit_protocol::{
    AttributeSet, DerivationMethod, DerivationParameters, DeriveKeyRequest,
    ExportRequest, ImportRequest, JoinSplitKeyRequest, ObjectType, SecretBytes,
    SplitKeyReference, UniqueIdentifier,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const ATTR_HASHING_ALGORITHM: u32 = 0x0042_0038;

fn item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("tag fits in TTLV")
        .try_checked()
        .expect("tag is allocated");
    Item::new(tag, value).expect("item is valid TTLV")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut value = Structure::new();
    for child in items {
        value.try_push(child).expect("fixture structure is valid");
    }
    value
}

fn output_attributes() -> AttributeSet {
    AttributeSet::try_new([
        item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ])
    .expect("synthetic output attributes are valid")
}

fn synthetic_symmetric_key() -> Item {
    let key_value = structure([item(
        KEY_MATERIAL,
        Value::byte_string(b"synthetic-import-material".to_vec()),
    )]);
    let key_block = structure([
        item(KEY_FORMAT_TYPE, Value::enumeration(1)),
        item(KEY_VALUE, Value::structure(key_value)),
    ]);
    item(
        SYMMETRIC_KEY,
        Value::structure(structure([item(0x0042_0040, Value::structure(key_block))])),
    )
}

#[allow(dead_code)]
fn derive_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let parameters = structure([item(ATTR_HASHING_ALGORITHM, Value::enumeration(6))]);
    let derivation = DerivationParameters::new()
        .with_cryptographic_parameters(parameters)
        .with_salt(SecretBytes::new(b"synthetic-salt".to_vec()))
        .with_iteration_count(10_000);
    let request = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source-key-example".to_owned())],
        DerivationMethod::from_raw(1),
        derivation,
        output_attributes(),
    );
    client.derive_key(request, limits)
}

#[allow(dead_code)]
fn export_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = ExportRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("key-to-export-example".to_owned()));
    client.export(request, limits)
}

#[allow(dead_code)]
fn import_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let request = ImportRequest::new(
        UniqueIdentifier::TextString("imported-key-example".to_owned()),
        ObjectType::from_raw(2),
        output_attributes(),
        synthetic_symmetric_key(),
    )
    .with_replace_existing(false);
    client.import(request, limits)
}

#[allow(dead_code)]
fn join_split_key(
    client: &mut Client,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    let references = vec![
        SplitKeyReference::new(UniqueIdentifier::TextString("split-a-example".to_owned()))
            .with_split_key_threshold(2),
        SplitKeyReference::new(UniqueIdentifier::TextString("split-b-example".to_owned()))
            .with_split_key_threshold(2),
    ];
    client.join_split_key(JoinSplitKeyRequest::new(ObjectType::from_raw(2), references), limits)
}

#[allow(dead_code)]
fn result_status(response: &ClientBatchItemResponse) -> u32 {
    response.outcome().result().status().raw()
}

#[allow(dead_code)]
fn inspect_export_without_formatting_material(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .export()
        .map(|typed| typed.with_ttlv(|item| item.children().len()))
}

fn main() {}
```

## Export API across languages

All four language surfaces use the same KMIP 2.1 §6.1.18 Table 217 choices.
The identifier, Key Format Type, Key Wrap Type, Key Compression Type, and Key
Wrapping Specification are optional; enum selectors retain unknown raw values.

| Language | Create the request | Inspect the typed result |
|---|---|---|
| Rust | `Client::export(ExportRequest)` | `ClientBatchOutcome::Export` and `ExportResponse` |
| C | `kmipkit_client_batch_item_export` | `kmipkit_client_batch_item_response_export` and `kmipkit_client_batch_item_response_export_ttlv` |
| Java | `ClientBatchItem.export(...)` | `ClientBatchItemResult.exportResponse()` and `ExportResponse.close()` |
| Python | `client_batch_item_export(...)` | `BatchItemResult.export_response` and `ExportResponse.close()` |

Read the batch item's `Result Status` before using successful payload fields.
A KMIP Failure remains a typed Export result and retains its complete ordered
response TTLV. On Success, Table 218 requires Object Type, Unique Identifier,
Attributes, and Any Object. Object Type is exposed as a raw unsigned value,
Unique Identifier retains its wire type, and Attributes and Any Object remain
generic TTLV. The C, Java, and Python Any Object value is a one-item Structure
whose child retains the original polymorphic item tag and order.

Close caller-owned response copies when finished. KMIPKit zeroizes memory it
owns, but Java and Python runtimes may retain external copies outside that
control. Do not format, log, or place object data, Attributes, or raw response
TTLV in errors.

## Import API across languages

Import follows OASIS KMIP Specification v2.1 §6.1.25, Tables 238–240. The
caller supplies the required Unique Identifier, raw Object Type, Attributes,
and one complete Any Object item. `Replace Existing` is omitted until selected
explicitly, so an explicit `false` remains distinct from absence. For wrapped
key objects, pass the raw Key Wrap Type when the object contains Key Wrapping
Data. KMIPKit preserves the supplied representation and performs no unwrap or
local cryptographic operation.

Any Object is built with the generic TTLV value model. Big Integer values are
available in Rust, C, Java, and Python; C callers can use
`kmipkit_ttlv_value_big_integer(limits, bytes, length, &value)`. The public C
header documents ownership transfer for builders that accept opaque handles.

| Language | Create the request | Inspect a successful result |
|---|---|---|
| Rust | `ImportRequest::new(...)` and `Client::import(...)` | `ClientBatchOutcome::Import` and `ImportResponse` |
| C | `kmipkit_client_batch_item_import` | `kmipkit_client_batch_item_response_import` and `kmipkit_client_batch_item_response_import_ttlv` |
| Java | `ClientBatchItem.importObject(...)` | `ClientBatchItemResult.importResponse()` and `ImportResponse.close()` |
| Python | `client_batch_item_import(...)` | `BatchItemResult.import_response` and `ImportResponse.close()` |

On Success, the typed result exposes the resulting Unique Identifier and an
owned copy of the complete ordered response TTLV, including unknown fields.
Pending and Failure remain common batch outcomes and do not expose a success
response TTLV. Read `Result Status` before using typed success data. Each Import
operation is sent once; KMIPKit does not retry, poll, or issue a follow-up
operation. Close response copies when finished. KMIPKit zeroizes memory it
owns; Java and Python runtimes may retain external copies outside that control.

The caller supplies exact identifier occurrences and their known Split Key
thresholds. KMIPKit requires one context per occurrence and checks that the
number of entries, including repeated identifiers, meets the largest supplied
threshold before sending. Thresholds are local metadata: they are not encoded
or fetched from the server. The request preserves identifier order and
repetitions.

## Handle results and sensitive values

Check `Result Status` before using an operation result. Exported object data
and response TTLV can contain secret material. Avoid logging, formatting, or
copying them into ordinary strings. KMIPKit-owned secret byte buffers are
zeroized when dropped; copies made by the caller are outside that guarantee.
When importing a wrapped key, supply the caller-selected Key Wrap Type exactly
when the object's Key Block contains Key Wrapping Data. `Replace Existing` is
set only when the caller chooses it explicitly.

For wrapped Key Values, the caller supplies IV/Counter/Nonce whenever the
selected wrapping method requires it. When Encoding Option is absent, encode
the Key Value Structure as TTLV before wrapping it. These obligations come
from KMIP Specification v2.1 §3.3, Tables 14–15. KMIPKit preserves the
caller-supplied representation; it does not infer parameters from referenced
server keys or inspect the wrapped plaintext or ciphertext.

Every method returns the server result from one request. A local validation
failure reports `NotSent`; transport failures retain their strongest delivery
state. Pending results remain explicit when the caller enables asynchronous
handling; KMIPKit does not poll automatically.

## Join Split Key API across languages

Join Split Key follows OASIS KMIP Specification v2.1 §6.1.27, Tables 244–246.
Each reference pairs one Unique Identifier occurrence with that split key's
caller-known Split Key Threshold. Create one reference per transmitted entry,
including repeated identifiers. Before sending, KMIPKit requires at least one
reference and checks that the entry count is greater than or equal to the
largest supplied threshold. Thresholds stay local and are never serialized or
looked up remotely; different entries may have different thresholds. KMIPKit
does not combine key material locally.

The C, Java, and Python bindings cap the combined UTF-8 text identifier bytes
in one Join Split Key request at 16 MiB before copying them into the native
request. Repeated occurrences each count toward this limit; an over-limit call
returns a resource-limit error.

Table 244 requires Object Type and one or more Unique Identifier entries. The
request preserves their order and repetitions. Secret Data Type, Attributes,
and Protection Storage Masks are optional. On Success, Table 245 requires the
resulting Unique Identifier. Failure and Pending remain common batch outcomes;
read Result Status and retain the shared result and delivery metadata before
using successful response fields.

| Language | Pair identifiers and local thresholds | Read a successful result |
|---|---|---|
| Rust | `SplitKeyReference::new(identifier).with_split_key_threshold(threshold)` | `ClientBatchOutcome::JoinSplitKey` and `JoinSplitKeyResponse::with_ttlv` |
| C | Create each opaque value with `kmipkit_split_key_reference_create`; pass the ordered pointer array to `kmipkit_client_batch_item_join_split_key`; release each handle after the call | `kmipkit_client_batch_item_response_join_split_key` and `kmipkit_client_batch_item_response_join_split_key_ttlv` |
| Java | `new SplitKeyReference(identifier, threshold)` in the ordered list passed to `ClientBatchItem.joinSplitKey(...)`; close each reference | `ClientBatchItemResult.joinSplitKeyResponse()` and `JoinSplitKeyResponse.close()` |
| Python | `extensions.SplitKeyReference(identifier, threshold)` in the ordered sequence passed to `extensions.client_batch_item_join_split_key(...)`; close each reference | `BatchItemResult.join_split_key_response` and `JoinSplitKeyResponse.close()` |

The C response TTLV accessor returns an independent owned copy of the complete
ordered response, including unknown fields; release it with
`kmipkit_ttlv_structure_release`. Java and Python responses own their TTLV
copies and must be closed. C arrays borrow reference handles only during the
builder call, while the builder copies identifier and threshold context into
the request. Java and Python own their reference handles and may retain
external runtime copies; their runtimes do not promise deterministic
zeroization of those copies. Avoid logging or formatting identifiers or raw
response TTLV. These API tests do not claim official fixture, OASIS
conformance, interoperability, profile support, or certification results.
