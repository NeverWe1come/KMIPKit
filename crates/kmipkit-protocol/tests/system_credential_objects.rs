use kmipkit_protocol::{ManagedObject, ManagedObjectError, ObjectType};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const PASSWORD_CREDENTIAL: u32 = 0x0042_01A1;
const DEVICE_CREDENTIAL: u32 = 0x0042_01A6;
const OTP_CREDENTIAL: u32 = 0x0042_01A7;
const HASHED_PASSWORD_CREDENTIAL: u32 = 0x0042_01AF;

const PASSWORD: u32 = 0x0042_00A1;
const PASSWORD_SALT_ALGORITHM: u32 = 0x0042_01A3;
const SALTED_PASSWORD: u32 = 0x0042_01A4;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw).expect("valid raw tag").try_checked().expect("valid tag")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("valid TTLV item")
}

fn structure(raw_tag: u32, fields: impl IntoIterator<Item = Item>) -> Item {
    let mut value = Structure::new();
    for field in fields {
        value.try_push(field).expect("bounded test structure");
    }
    item(raw_tag, Value::structure(value))
}

#[test]
fn system_credential_objects_reject_invalid_source_schemas() {
    let malformed = [
        (
            ObjectType::PASSWORD_CREDENTIAL,
            structure(
                PASSWORD_CREDENTIAL,
                [
                    item(PASSWORD, Value::text_string("sentinel-password".into())),
                    item(PASSWORD_SALT_ALGORITHM, Value::enumeration(1)),
                    item(SALTED_PASSWORD, Value::byte_string(b"sentinel-hash".to_vec())),
                ],
            ),
        ),
        (
            ObjectType::DEVICE_CREDENTIAL,
            structure(DEVICE_CREDENTIAL, []),
        ),
        (
            ObjectType::ONE_TIME_PASSWORD_CREDENTIAL,
            structure(OTP_CREDENTIAL, []),
        ),
        (
            ObjectType::HASHED_PASSWORD_CREDENTIAL,
            structure(HASHED_PASSWORD_CREDENTIAL, []),
        ),
    ];

    for (object_type, raw) in malformed {
        assert!(
            matches!(
                ManagedObject::try_from_item(object_type, raw),
                Err(ManagedObjectError::InvalidFieldCombination
                    | ManagedObjectError::MissingRequiredField)
            ),
            "Object Type {} accepted an invalid credential structure",
            object_type.raw()
        );
    }
}

#[test]
fn system_credential_object_types_are_not_accepted_by_unknown_fallback() {
    for (object_type, raw_tag) in [
        (ObjectType::PASSWORD_CREDENTIAL, PASSWORD_CREDENTIAL),
        (ObjectType::DEVICE_CREDENTIAL, DEVICE_CREDENTIAL),
        (ObjectType::ONE_TIME_PASSWORD_CREDENTIAL, OTP_CREDENTIAL),
        (
            ObjectType::HASHED_PASSWORD_CREDENTIAL,
            HASHED_PASSWORD_CREDENTIAL,
        ),
    ] {
        let raw = structure(raw_tag, []);
        assert_eq!(
            ManagedObject::unknown(object_type, raw).err(),
            Some(ManagedObjectError::MismatchedObjectType)
        );
    }
}

#[test]
fn system_credential_object_type_requires_its_exact_structure_tag() {
    let wrong_tag = structure(DEVICE_CREDENTIAL, []);

    assert_eq!(
        ManagedObject::try_from_item(ObjectType::PASSWORD_CREDENTIAL, wrong_tag).err(),
        Some(ManagedObjectError::InvalidStructure)
    );
}
