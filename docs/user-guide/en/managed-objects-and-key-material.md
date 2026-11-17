# Managed objects and key material

KMIPKit provides typed Rust models and C, Java, and Python wrappers for the
nine KMIP 2.1 managed-object structures. Rust builders emit known fields in
the order defined by the corresponding OASIS tables. The resulting
object-specific Structure Item is passed directly to Register or Import; the
`Object Type` Enumeration remains a separate value. The language-specific
wrapper names and ownership rules are summarized in [Typed bindings,
validation, and response data](typed-bindings-and-response-data.md).

## Generic server-generated Create

Use the generic fluent Create builder when the caller has selected an Object
Type and its direct Attributes. The builder emits the required Attributes
Structure even when the set is empty; it can also carry the optional generic
Protection Storage Masks Structure. The server creates the object. KMIPKit
does not choose cryptographic attributes, material, or storage policy.

```rust,kmipkit-test
use kmipkit_client::{Client, ClientOperation};
use kmipkit_protocol::{AttributeSet, ObjectType};
use kmipkit_ttlv::codec::CodecLimits;

fn create_object(
    client: &mut Client,
    limits: &CodecLimits,
    object_type: ObjectType,
    attributes: AttributeSet,
) -> Result<(), kmipkit_client::ClientError> {
    let item = client
        .request()
        .create()
        .object_type(object_type)
        .attributes(attributes)
        .execute(limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Create);
    let response_view = outcome.response();
    let response = response_view.create().expect("typed Create response");
    let _status = response.result().status().raw();
    let _object_type = response.object_type();
    let _unique_identifier = response.unique_identifier();
    Ok(())
}

fn main() { let _example = create_object; }
```

This is KMIP v2.1 §6.1.8, Tables 186–188. The application supplies all
attributes and any Protection Storage Masks explicitly. Inspect the returned
KMIP result and identifier; a successful response does not establish that the
server followed application-specific policy.

The C, Java, and Python bindings expose the same typed builder. C callers pass
an ordered TTLV Structure and receive an owned batch item; Java and Python
callers pass an `AttributeSet`, which the binding copies into the native
request. For example, Java can build the item without sending it:

```java
try (TtlvStructure directItems = TtlvStructure.create();
     AttributeSet attributes = AttributeSet.tryNew(directItems);
     CreateBuilder create = client.request().create()
         .objectType(KmipValues.ObjectType.SYMMETRIC_KEY)
         .attributes(attributes)) {
    create.validate();
    try (ClientBatchItem item = create.build()) {
        assert item.operation() == KmipValues.Operation.CREATE;
    }
}
```

Python uses the same operation and object type with named arguments:

```python
with ttlv.ttlv_structure_create() as direct_items:
    with attribute_operations.AttributeSet.try_new(direct_items) as attributes:
        with client.request().create() as create:
            create.object_type(protocol_values.ObjectType.SYMMETRIC_KEY)
            create.attributes(attributes).validate()
            with create.build() as item:
                assert item.operation == protocol_values.Operation.CREATE
```

In C, pass `KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY` to
`kmipkit_create_builder_set_object_type`, then call
`kmipkit_create_builder_set_attributes`, and
`kmipkit_create_builder_build_batch_item`; release the returned item and builder
with their matching release functions. All three bindings expose validation,
batch conversion, exchange options, and the explicit Request Header
Asynchronous Indicator. Building the batch item does not send a request.

## Plaintext key material

Choose the Key Format Type explicitly. For Raw key bytes, a typed value can be
assembled as follows:

```rust
use kmipkit_protocol::{
    AttributeSet, KeyBlock, KeyBlockValue, KeyFormatType, KeyMaterial, KeyValue,
    SymmetricKey,
};

let material = KeyMaterial::byte_string(1, vec![0xA1, 0xA2])?;
let value = KeyValue::try_new(material, Some(AttributeSet::new()))?;
let block = KeyBlock::try_new(KeyFormatType::from_raw(1))?
    .with_key_value(KeyBlockValue::Plaintext(value));
let symmetric_key = SymmetricKey::try_new(block)?;
let object_item = symmetric_key.into_item();
assert_eq!(object_item.tag().raw(), 0x0042_008F);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The format and any cryptographic attributes are caller choices. KMIPKit does
not infer the algorithm, key size, use, protection policy, or server support.

## Wrapped key material

Wrapped bytes use a TTLV Byte String `Key Value` and retain the supplied
wrapping metadata. KMIPKit transports that data; it does not wrap, unwrap,
encrypt, sign, or compute a MAC.

```rust
use kmipkit_protocol::{
    EncryptionKeyInformation, KeyBlock, KeyBlockValue, KeyFormatType,
    KeyWrappingData, SecretBytes, SymmetricKey,
};

let wrapping = KeyWrappingData::try_new(1)?
    .with_encryption_key_information(
        EncryptionKeyInformation::try_new("wrapping-key".to_owned())?,
    );
let block = KeyBlock::try_new(KeyFormatType::from_raw(1))?
    .with_key_value(KeyBlockValue::Wrapped(SecretBytes::new(vec![0xB1, 0xB2])))
    .with_key_wrapping_data(wrapping);
let symmetric_key = SymmetricKey::try_new(block)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Reading and preserving data

`try_from_item` reads typed structures without dropping unknown descendants.
`with_raw` lends their ordered `StructureView` only for the callback lifetime.
This keeps vendor tags, repeated items, source order, and raw future values
available to callers. It exposes the decoded TTLV tree, not a byte-for-byte
copy of the network frame.

`TransparentSymmetricKey`, `TransparentDsaPrivateKey`, `TransparentRsaPrivateKey`,
and the other transparent models provide typed table-shaped builders. Other
Key Material formats remain available as caller-selected bytes or generic
Structures. An unknown Object Type can be carried by
`ManagedObject::unknown` without interpreting its contents.

### Transparent key material

```rust
use kmipkit_protocol::TransparentSymmetricKey;

let transparent = TransparentSymmetricKey::try_new(vec![0xC1, 0xC2])?;
let key_material_item = transparent.into_item();
assert_eq!(key_material_item.tag().raw(), 0x0042_0043);
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Unknown object fallback

Unknown Object Types and vendor structures stay generic:

```rust
use kmipkit_protocol::{ManagedObject, ObjectType};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

let tag = RawTag::new(0x0054_0102)?.try_checked()?;
let extension = Item::new(tag, Value::structure(Structure::new()))?;
let object = ManagedObject::unknown(ObjectType::from_raw(0x8000_0001), extension)?;
assert_eq!(object.object_type().raw(), 0x8000_0001);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Secret lifetime

Owned KMIPKit `Item` and `Value` payloads redact bytes in `Debug` and zeroize
their owned buffers when dropped. Moving a typed key model transfers that same
owner. KMIPKit cannot clear the caller's original buffers, copies retained by
applications or foreign runtimes, or storage owned by TLS and platform
libraries.

The owner below takes the supplied byte buffer and releases its KMIPKit-owned
copy when dropped:

```rust
use kmipkit_protocol::KeyMaterial;

let caller_owned_secret = vec![0xD1, 0xD2];
let secret_owner = KeyMaterial::byte_string(1, caller_owned_secret)?;
drop(secret_owner);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## KMIP 3.0 system credential objects

KMIP 3.0 defines typed Password, Device, One Time Password, and Hashed
Password Credential system objects. KMIPKit builds their fields in OASIS table
order and exposes callback-scoped readers. The active-version validator must
select KMIP 3.0 before these object types are sent; KMIP 2.1 remains the client
default.

```rust
use kmipkit_protocol::{
    DeviceCredentialObject, DeviceCredentialObjectFields, PasswordCredentialObject,
    PasswordCredentialObjectFields, SecretText,
};

let password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
    password: Some(SecretText::new("caller-provided secret".into())),
    ..PasswordCredentialObjectFields::default()
})?;
assert_eq!(password.effective_iteration_count(), 100);

let device = DeviceCredentialObject::try_new(DeviceCredentialObjectFields {
    device_identifier: Some(SecretText::new("device identifier".into())),
    ..DeviceCredentialObjectFields::default()
})?;
assert!(device.with_device_identifier(|value| value.is_some()));
# Ok::<(), Box<dyn std::error::Error>>(())
```

The Password Link alternative is an Attribute, so checking it requires the
credential object together with its Attribute Set. A salted Password object
must not also contain Password and must include Password Salt Algorithm.
Device Credential must contain at least one identifier; uniqueness remains a
server policy. OTP Algorithm and both Hashed Password hash fields are required.
An omitted Hashing Algorithm uses the SHA-256 default without adding a field.
KMIPKit transports these values; it does not calculate passwords, OTPs, salts,
or hashes. Secret text and bytes are redacted and KMIP-owned payload storage is
zeroized when released; copies made by callers or external runtimes are outside
that guarantee.
