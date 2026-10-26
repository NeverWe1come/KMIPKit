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
