use kmipkit_protocol::{
    DeviceCredentialObject, DeviceCredentialObjectFields, HashedPasswordCredentialObject,
    HashedPasswordCredentialObjectFields, ManagedObject, ManagedObjectError, ObjectType,
    OneTimePasswordCredentialObject, OneTimePasswordCredentialObjectFields,
    PasswordCredentialObject, PasswordCredentialObjectFields, SecretBytes, SecretText,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const PASSWORD_CREDENTIAL: u32 = 0x0042_01A1;
const DEVICE_CREDENTIAL: u32 = 0x0042_01A6;
const OTP_CREDENTIAL: u32 = 0x0042_01A7;
const HASHED_PASSWORD_CREDENTIAL: u32 = 0x0042_01AF;

const PASSWORD: u32 = 0x0042_00A1;
const PASSWORD_SALT: u32 = 0x0042_01A2;
const PASSWORD_SALT_ALGORITHM: u32 = 0x0042_01A3;
const SALTED_PASSWORD: u32 = 0x0042_01A4;
const ITERATION_COUNT: u32 = 0x0042_003C;
const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const DEVICE_IDENTIFIER: u32 = 0x0042_00A2;
const NETWORK_IDENTIFIER: u32 = 0x0042_00AB;
const MACHINE_IDENTIFIER: u32 = 0x0042_00A9;
const MEDIA_IDENTIFIER: u32 = 0x0042_00AA;
const OTP_ALGORITHM: u32 = 0x0042_01A8;
const OTP_DIGEST: u32 = 0x0042_01A9;
const OTP_SERIAL: u32 = 0x0042_01AA;
const OTP_SEED: u32 = 0x0042_01AB;
const OTP_INTERVAL: u32 = 0x0042_01AC;
const OTP_DIGITS: u32 = 0x0042_01AD;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const HASHED_USERNAME_PASSWORD: u32 = 0x0042_01B0;
const HASHED_PASSWORD_USERNAME: u32 = 0x0042_01B1;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("valid raw tag")
        .try_checked()
        .expect("valid tag")
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
                    item(
                        SALTED_PASSWORD,
                        Value::byte_string(b"sentinel-hash".to_vec()),
                    ),
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
        let error = ManagedObject::try_from_item(object_type, raw)
            .expect_err("malformed credential object is rejected");
        assert!(!format!("{error} {error:?}").contains("sentinel"));
        assert!(
            matches!(
                error,
                ManagedObjectError::InvalidFieldCombination
                    | ManagedObjectError::MissingRequiredField
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

#[test]
fn password_credential_builders_and_readers_follow_source_rules() {
    let password_fields = PasswordCredentialObjectFields {
        password: Some(SecretText::new("secret-password".into())),
        password_salt: Some(SecretBytes::new(b"salt".to_vec())),
        password_salt_algorithm: Some(0xF003),
        iteration_count: Some(250),
        ..PasswordCredentialObjectFields::default()
    };
    let password =
        PasswordCredentialObject::try_new(password_fields).expect("plain password object is valid");
    password.with_password(|value| assert_eq!(value, Some("secret-password")));
    password.with_password_salt(|value| assert_eq!(value, Some(&b"salt"[..])));
    assert_eq!(password.password_salt_algorithm_raw(), Some(0xF003));
    assert_eq!(password.iteration_count(), Some(250));
    assert_eq!(password.effective_iteration_count(), 250);
    password.with_salted_password(|value| assert_eq!(value, None));
    password.with_item(|item| assert_eq!(item.tag().raw(), PASSWORD_CREDENTIAL));
    assert_structure_tags(
        &password.into_item(),
        PASSWORD_CREDENTIAL,
        &[
            PASSWORD,
            PASSWORD_SALT,
            PASSWORD_SALT_ALGORITHM,
            ITERATION_COUNT,
        ],
    );

    let salted_password = PasswordCredentialObject::try_new(PasswordCredentialObjectFields {
        password_salt: Some(SecretBytes::new(b"salted-salt".to_vec())),
        password_salt_algorithm: Some(0xF004),
        salted_password: Some(SecretBytes::new(b"salted-password".to_vec())),
        ..PasswordCredentialObjectFields::default()
    })
    .expect("salted representation requires an algorithm and forbids plaintext Password");
    salted_password.with_salted_password(|value| {
        assert_eq!(value, Some(&b"salted-password"[..]));
    });
    salted_password.with_password(|value| assert_eq!(value, None));
    assert_eq!(salted_password.password_salt_algorithm_raw(), Some(0xF004));
    assert_eq!(salted_password.effective_iteration_count(), 100);
    salted_password.with_item(|item| assert_eq!(item.tag().raw(), PASSWORD_CREDENTIAL));
    assert!(format!("{salted_password:?}").contains("PasswordCredentialObject"));
    assert_structure_tags(
        &salted_password.into_item(),
        PASSWORD_CREDENTIAL,
        &[PASSWORD_SALT, PASSWORD_SALT_ALGORITHM, SALTED_PASSWORD],
    );
}

#[test]
fn device_credential_builders_and_readers_follow_table_order() {
    let device_fields = DeviceCredentialObjectFields {
        device_serial_number: Some(SecretText::new("serial".into())),
        device_identifier: Some(SecretText::new("device-id".into())),
        network_identifier: Some(SecretText::new("network".into())),
        machine_identifier: Some(SecretText::new("machine".into())),
        media_identifier: Some(SecretText::new("media".into())),
    };
    let device =
        DeviceCredentialObject::try_new(device_fields).expect("one device field is sufficient");
    device.with_device_serial_number(|value| assert_eq!(value, Some("serial")));
    device.with_device_identifier(|value| assert_eq!(value, Some("device-id")));
    device.with_network_identifier(|value| assert_eq!(value, Some("network")));
    device.with_machine_identifier(|value| assert_eq!(value, Some("machine")));
    device.with_media_identifier(|value| assert_eq!(value, Some("media")));
    device.with_item(|item| assert_eq!(item.tag().raw(), DEVICE_CREDENTIAL));
    assert!(format!("{device:?}").contains("DeviceCredentialObject"));
    assert_structure_tags(
        &device.into_item(),
        DEVICE_CREDENTIAL,
        &[
            DEVICE_SERIAL_NUMBER,
            DEVICE_IDENTIFIER,
            NETWORK_IDENTIFIER,
            MACHINE_IDENTIFIER,
            MEDIA_IDENTIFIER,
        ],
    );

    let minimal_device = DeviceCredentialObject::try_new(DeviceCredentialObjectFields {
        device_serial_number: Some(SecretText::new("serial-only".into())),
        ..DeviceCredentialObjectFields::default()
    })
    .expect("one device identifier is sufficient");
    minimal_device.with_device_identifier(|value| assert_eq!(value, None));
    minimal_device.with_network_identifier(|value| assert_eq!(value, None));
    minimal_device.with_machine_identifier(|value| assert_eq!(value, None));
    minimal_device.with_media_identifier(|value| assert_eq!(value, None));
}

#[test]
fn one_time_password_builders_and_readers_follow_table_order() {
    let otp_fields = OneTimePasswordCredentialObjectFields {
        otp_algorithm: Some(0xF001),
        otp_digest: Some(0xF005),
        otp_serial: Some(SecretText::new("otp-serial".into())),
        otp_seed: Some(SecretBytes::new(b"seed-secret".to_vec())),
        otp_interval: Some(30),
        otp_digits: Some(8),
    };
    let otp = OneTimePasswordCredentialObject::try_new(otp_fields)
        .expect("unknown future OTP algorithm remains representable");
    assert_eq!(otp.otp_algorithm_raw(), Some(0xF001));
    assert_eq!(otp.otp_digest_raw(), Some(0xF005));
    otp.with_otp_serial(|value| assert_eq!(value, Some("otp-serial")));
    otp.with_otp_seed(|value| assert_eq!(value, Some(&b"seed-secret"[..])));
    assert_eq!(otp.otp_interval(), Some(30));
    assert_eq!(otp.otp_digits(), Some(8));
    otp.with_item(|item| assert_eq!(item.tag().raw(), OTP_CREDENTIAL));
    assert!(format!("{otp:?}").contains("OneTimePasswordCredentialObject"));
    assert_structure_tags(
        &otp.into_item(),
        OTP_CREDENTIAL,
        &[
            OTP_ALGORITHM,
            OTP_DIGEST,
            OTP_SERIAL,
            OTP_SEED,
            OTP_INTERVAL,
            OTP_DIGITS,
        ],
    );

    let minimal_otp =
        OneTimePasswordCredentialObject::try_new(OneTimePasswordCredentialObjectFields {
            otp_algorithm: Some(1),
            ..OneTimePasswordCredentialObjectFields::default()
        })
        .expect("the required OTP Algorithm is sufficient");
    assert_eq!(minimal_otp.otp_digest_raw(), None);
    minimal_otp.with_otp_serial(|value| assert_eq!(value, None));
    minimal_otp.with_otp_seed(|value| assert_eq!(value, None));
    assert_eq!(minimal_otp.otp_interval(), None);
    assert_eq!(minimal_otp.otp_digits(), None);
}

#[test]
fn hashed_password_builders_and_readers_preserve_defaults_and_order() {
    let hashed_fields = HashedPasswordCredentialObjectFields {
        hashing_algorithm: Some(0xF002),
        hashed_username_password: Some(SecretBytes::new(b"hash-a".to_vec())),
        hashed_password_username: Some(SecretBytes::new(b"hash-b".to_vec())),
    };
    let hashed = HashedPasswordCredentialObject::try_new(hashed_fields)
        .expect("both required hash fields are present");
    assert_eq!(hashed.hashing_algorithm_raw(), Some(0xF002));
    hashed.with_hashed_username_password(|value| assert_eq!(value, Some(&b"hash-a"[..])));
    hashed.with_hashed_password_username(|value| assert_eq!(value, Some(&b"hash-b"[..])));
    assert!(format!("{hashed:?}").contains("HashedPasswordCredentialObject"));
    hashed.with_item(|item| assert_eq!(item.tag().raw(), HASHED_PASSWORD_CREDENTIAL));
    assert_structure_tags(
        &hashed.into_item(),
        HASHED_PASSWORD_CREDENTIAL,
        &[
            HASHING_ALGORITHM,
            HASHED_USERNAME_PASSWORD,
            HASHED_PASSWORD_USERNAME,
        ],
    );

    let default_algorithm =
        HashedPasswordCredentialObject::try_new(HashedPasswordCredentialObjectFields {
            hashed_username_password: Some(SecretBytes::new(b"hash-a".to_vec())),
            hashed_password_username: Some(SecretBytes::new(b"hash-b".to_vec())),
            ..HashedPasswordCredentialObjectFields::default()
        })
        .expect("optional algorithm has its source-defined default");
    assert_eq!(default_algorithm.hashing_algorithm_raw(), None);
    assert_eq!(default_algorithm.effective_hashing_algorithm_raw(), 6);
    default_algorithm.with_item(|item| assert_eq!(item.tag().raw(), HASHED_PASSWORD_CREDENTIAL));
    assert!(format!("{default_algorithm:?}").contains("HashedPasswordCredentialObject"));
    assert_structure_tags(
        &default_algorithm.into_item(),
        HASHED_PASSWORD_CREDENTIAL,
        &[HASHED_USERNAME_PASSWORD, HASHED_PASSWORD_USERNAME],
    );
}

fn assert_structure_tags(item: &Item, expected_tag: u32, expected_children: &[u32]) {
    assert_eq!(item.tag().raw(), expected_tag);
    item.with_value(|value| {
        let kmipkit_ttlv::ValueView::Structure(structure) = value else {
            panic!("credential object is a Structure")
        };
        let actual: Vec<_> = structure
            .children()
            .iter()
            .map(|child| child.tag().raw())
            .collect();
        assert_eq!(actual, expected_children);
    });
}

#[test]
fn system_credential_objects_retain_extensions_and_redact_owned_secrets() {
    let sentinel = "never-print-this-password";
    let raw = structure(
        PASSWORD_CREDENTIAL,
        [
            item(PASSWORD, Value::text_string(sentinel.into())),
            item(ITERATION_COUNT, Value::integer(100)),
            item(0x0054_0123, Value::byte_string(b"extension".to_vec())),
        ],
    );
    let object = PasswordCredentialObject::try_from_item(raw)
        .expect("password plus an unknown extension is preserved");
    let debug = format!("{object:?}");
    assert!(!debug.contains(sentinel));
    let round_trip = object.into_item();
    round_trip.with_value(|value| {
        let kmipkit_ttlv::ValueView::Structure(structure) = value else {
            panic!("credential object is a Structure")
        };
        assert_eq!(structure.children()[0].tag().raw(), PASSWORD);
        assert_eq!(structure.children()[1].tag().raw(), ITERATION_COUNT);
        assert_eq!(structure.children()[2].tag().raw(), 0x0054_0123);
    });
}

#[test]
fn system_credential_objects_reject_wrong_types_order_and_duplicates() {
    let wrong_password_type = structure(
        PASSWORD_CREDENTIAL,
        [item(PASSWORD, Value::byte_string(b"wrong type".to_vec()))],
    );
    assert_eq!(
        PasswordCredentialObject::try_from_item(wrong_password_type).err(),
        Some(ManagedObjectError::IncorrectFieldType)
    );

    let device_fields_out_of_order = structure(
        DEVICE_CREDENTIAL,
        [
            item(DEVICE_IDENTIFIER, Value::text_string("device".into())),
            item(DEVICE_SERIAL_NUMBER, Value::text_string("serial".into())),
        ],
    );
    assert_eq!(
        DeviceCredentialObject::try_from_item(device_fields_out_of_order).err(),
        Some(ManagedObjectError::IncorrectFieldOrder)
    );

    let duplicate_otp_algorithm = structure(
        OTP_CREDENTIAL,
        [
            item(OTP_ALGORITHM, Value::enumeration(1)),
            item(OTP_ALGORITHM, Value::enumeration(2)),
        ],
    );
    assert_eq!(
        OneTimePasswordCredentialObject::try_from_item(duplicate_otp_algorithm).err(),
        Some(ManagedObjectError::DuplicateField)
    );

    let hash_fields_out_of_order = structure(
        HASHED_PASSWORD_CREDENTIAL,
        [
            item(HASHED_USERNAME_PASSWORD, Value::byte_string(b"a".to_vec())),
            item(HASHING_ALGORITHM, Value::enumeration(6)),
            item(HASHED_PASSWORD_USERNAME, Value::byte_string(b"b".to_vec())),
        ],
    );
    assert_eq!(
        HashedPasswordCredentialObject::try_from_item(hash_fields_out_of_order).err(),
        Some(ManagedObjectError::IncorrectFieldOrder)
    );
}

#[test]
fn managed_object_dispatches_each_system_credential_structure() {
    let objects = [
        (
            ObjectType::PASSWORD_CREDENTIAL,
            structure(
                PASSWORD_CREDENTIAL,
                [item(PASSWORD, Value::text_string("p".into()))],
            ),
        ),
        (
            ObjectType::DEVICE_CREDENTIAL,
            structure(
                DEVICE_CREDENTIAL,
                [item(
                    DEVICE_SERIAL_NUMBER,
                    Value::text_string("serial".into()),
                )],
            ),
        ),
        (
            ObjectType::ONE_TIME_PASSWORD_CREDENTIAL,
            structure(OTP_CREDENTIAL, [item(OTP_ALGORITHM, Value::enumeration(1))]),
        ),
        (
            ObjectType::HASHED_PASSWORD_CREDENTIAL,
            structure(
                HASHED_PASSWORD_CREDENTIAL,
                [
                    item(HASHED_USERNAME_PASSWORD, Value::byte_string(b"a".to_vec())),
                    item(HASHED_PASSWORD_USERNAME, Value::byte_string(b"b".to_vec())),
                ],
            ),
        ),
    ];

    for (object_type, item) in objects {
        let object = ManagedObject::try_from_item(object_type, item)
            .expect("source-valid system credential object dispatches");
        assert_eq!(object.object_type(), object_type);
        assert!(!matches!(object, ManagedObject::Unknown { .. }));
    }
}
