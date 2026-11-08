# KMIPKIT-0049 Rust Quickstart

The examples use the public API in `kmipkit-protocol` and correspond to the
compiled consumer cases in `crates/kmipkit-protocol/tests/managed_object_examples.rs`.
For expanded guidance in both supported documentation languages, see
`docs/user-guide/en/managed-objects-and-key-material.md` and
`docs/user-guide/es/objetos-gestionados-y-material-de-clave.md`.

## Plaintext object

```rust
use kmipkit_protocol::{
    AttributeSet, KeyBlock, KeyBlockValue, KeyFormatType, KeyMaterial, KeyValue,
    SymmetricKey,
};

let material = KeyMaterial::byte_string(1, vec![0xA1, 0xA2])?;
let value = KeyValue::try_new(material, Some(AttributeSet::new()))?;
let block = KeyBlock::try_new(KeyFormatType::from_raw(1))?
    .with_key_value(KeyBlockValue::Plaintext(value));
let item = SymmetricKey::try_new(block)?.into_item();
assert_eq!(item.tag().raw(), 0x0042_008F);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Wrapped object

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
let item = SymmetricKey::try_new(block)?.into_item();
assert_eq!(item.tag().raw(), 0x0042_008F);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Transparent material and unknown fallback

```rust
use kmipkit_protocol::{ManagedObject, ObjectType, TransparentSymmetricKey};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

let transparent = TransparentSymmetricKey::try_new(vec![0xC1, 0xC2])?;
assert_eq!(transparent.into_item().tag().raw(), 0x0042_0043);

let vendor_tag = RawTag::new(0x0054_0102)?.try_checked()?;
let extension = Item::new(vendor_tag, Value::structure(Structure::new()))?;
let unknown = ManagedObject::unknown(ObjectType::from_raw(0x8000_0001), extension)?;
assert_eq!(unknown.object_type().raw(), 0x8000_0001);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Secret owner lifetime

```rust
use kmipkit_protocol::KeyMaterial;

let caller_owned_secret = vec![0xD1, 0xD2];
let secret_owner = KeyMaterial::byte_string(1, caller_owned_secret)?;
drop(secret_owner);
# Ok::<(), Box<dyn std::error::Error>>(())
```

KMIPKit zeroizes the payload storage owned by the TTLV value. Copies held by the
caller, application, foreign runtime, TLS stack, or platform libraries remain
outside KMIPKit's ownership boundary. The models transport key material and do
not perform wrapping, unwrapping, or cryptographic operations.

## KMIP 3.0 system credential objects

The four system credential object structures use their KMIP 3.0 Object Type
and tag assignments. Secret-bearing fields require `SecretText` or
`SecretBytes`; their callback readers do not return borrowed data beyond the
callback. The active-version validator is responsible for rejecting these
types when the request selects KMIP 2.1.

```rust
use kmipkit_protocol::{
    HashedPasswordCredentialObject, HashedPasswordCredentialObjectFields, SecretBytes,
};

let object = HashedPasswordCredentialObject::try_new(HashedPasswordCredentialObjectFields {
    hashed_username_password: Some(SecretBytes::new(vec![0xA1, 0xA2])),
    hashed_password_username: Some(SecretBytes::new(vec![0xB1, 0xB2])),
    ..HashedPasswordCredentialObjectFields::default()
})?;
assert_eq!(object.hashing_algorithm_raw(), None);
assert_eq!(object.effective_hashing_algorithm_raw(), 6); // SHA-256 default
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Local verification

Run the focused Rust tests, API manifest generator check, Rust and binding
export audits, format, Clippy, protocol tests, and coverage gate recorded in
`implementation-evidence.md`. Hosted matrix checks remain separate evidence
and must be recorded when the required GitHub billing is available.
