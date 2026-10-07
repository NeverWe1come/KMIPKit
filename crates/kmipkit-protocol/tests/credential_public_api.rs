//! Derived public API contract tests for KMIP 2.1 §§9.4, 9.11, and 9.14.
//!
//! These cases exercise the documented callback views, conversions, and
//! redacted diagnostics without adding credential transmission behavior.
//!
//! Traceability: KMIPKIT-0008-FR-001–FR-010; SC-001, SC-002, SC-005.

use kmipkit_protocol::{
    AttestationCredential, Authentication, Credential, CredentialType, CredentialValidationError,
    CredentialValidationErrorKind, CredentialValue, HashedPasswordCredential, Nonce,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const CREDENTIAL: u32 = 0x0042_0023;
const USERNAME: u32 = 0x0042_0099;
const PASSWORD: u32 = 0x0042_00A1;
const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const TIME_STAMP: u32 = 0x0042_0092;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const ONE_TIME_PASSWORD: u32 = 0x0042_0156;
const HASHED_PASSWORD: u32 = 0x0042_0157;
const NONCE: u32 = 0x0042_00C8;
const NONCE_ID: u32 = 0x0042_00C9;
const NONCE_VALUE: u32 = 0x0042_00CA;
const ATTESTATION_TYPE: u32 = 0x0042_00C7;
const ATTESTATION_MEASUREMENT: u32 = 0x0042_00CB;
const EXTENSION_CHILD: u32 = 0x0054_0001;
const SECRET_SENTINEL: &str = "credential-public-api-secret-sentinel";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("checked fixture tag and value form an Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("fixture stays within the local Structure depth limit");
    }
    structure
}

fn credential_tree(raw_type: u32) -> Structure {
    structure([
        item(CREDENTIAL_TYPE, Value::enumeration(raw_type)),
        item(
            CREDENTIAL_VALUE,
            Value::structure(structure([item(
                EXTENSION_CHILD,
                Value::text_string(SECRET_SENTINEL.to_owned()),
            )])),
        ),
    ])
}

fn assert_error_kind(
    result: Result<Credential, CredentialValidationError>,
    expected: CredentialValidationErrorKind,
) {
    let error = result.expect_err("the malformed Credential is rejected");
    assert_eq!(error.kind(), expected);
    assert!(!error.to_string().contains(SECRET_SENTINEL));
}

macro_rules! check_wrapper {
    ($value:expr, $expected_fields:expr) => {{
        let value = $value;
        assert_eq!(
            value.with_ttlv(|tree| tree.children().len()),
            $expected_fields
        );
        let diagnostics = format!("{value:?}; {value}");
        assert!(!diagnostics.contains(SECRET_SENTINEL));
        value.into_ttlv()
    }};
}

fn wrapper_round_trip(value: CredentialValue, raw_type: u32, expected_fields: usize) -> Structure {
    let diagnostics = format!("{value:?}; {value}");
    assert!(!diagnostics.contains(SECRET_SENTINEL));
    match value {
        CredentialValue::UsernameAndPassword(value) => check_wrapper!(value, expected_fields),
        CredentialValue::Device(value) => check_wrapper!(value, expected_fields),
        CredentialValue::Attestation(value) => check_wrapper!(value, expected_fields),
        CredentialValue::OneTimePassword(value) => check_wrapper!(value, expected_fields),
        CredentialValue::HashedPassword(value) => check_wrapper!(value, expected_fields),
        CredentialValue::Ticket(value) => check_wrapper!(value, expected_fields),
        CredentialValue::Extensions(value) => check_wrapper!(value, expected_fields),
        CredentialValue::Unknown {
            credential_type,
            value,
        } => {
            assert_eq!(credential_type, raw_type);
            check_wrapper!(value, expected_fields)
        }
        _ => panic!("the public wrapper matrix lists every current variant"),
    }
}

#[test]
fn credential_and_authentication_public_views_preserve_order_and_redact() {
    let raw_type = 0xF123_4567;
    let credential = Credential::try_from_ttlv(credential_tree(raw_type))
        .expect("unknown raw Credential Type and opaque value remain valid");
    assert_eq!(
        credential.credential_type(),
        CredentialType::Unknown(raw_type)
    );
    assert_eq!(credential.credential_type_raw(), raw_type);
    assert_eq!(credential.with_ttlv(|tree| tree.children().len()), 2);
    let diagnostics = format!("{credential:?}; {credential}");
    assert!(!diagnostics.contains(SECRET_SENTINEL));
    let credential = Credential::try_from_ttlv(credential.into_ttlv())
        .expect("Credential conversion preserves its complete tree");

    let second_type = 0x8000_0001;
    let second = Credential::try_from_ttlv(credential_tree(second_type))
        .expect("extension Credential Type retains its raw value");
    let authentication = Authentication::new(vec![credential, second])
        .expect("Authentication accepts a non-empty ordered Credential list");
    assert_eq!(authentication.credentials().len(), 2);
    assert_eq!(
        authentication
            .credentials()
            .map(|view| view.credential_type_raw())
            .collect::<Vec<_>>(),
        [raw_type, second_type]
    );
    for view in authentication.credentials() {
        assert_eq!(
            view.credential_type(),
            CredentialType::from_raw(view.credential_type_raw())
        );
        assert_eq!(view.with_ttlv(|tree| tree.children().len()), Some(2));
        let diagnostics = format!("{view:?}");
        assert!(!diagnostics.contains(SECRET_SENTINEL));
    }
    assert_eq!(authentication.with_ttlv(|tree| tree.children().len()), 2);
    let diagnostics = format!("{authentication:?}; {authentication}");
    assert!(!diagnostics.contains(SECRET_SENTINEL));
    let round_trip = authentication.into_ttlv();
    assert_eq!(round_trip.view().children().len(), 2);

    let parsed = Authentication::try_from_ttlv(round_trip)
        .expect("Authentication can be parsed again without changing order");
    assert_eq!(parsed.credentials().len(), 2);
}

#[test]
fn typed_credential_wrappers_expose_views_and_lossless_conversions() {
    let cases = [
        (
            1,
            structure([
                item(USERNAME, Value::text_string("user".to_owned())),
                item(PASSWORD, Value::text_string(SECRET_SENTINEL.to_owned())),
            ]),
            2,
        ),
        (
            2,
            structure([item(
                DEVICE_SERIAL_NUMBER,
                Value::text_string("serial".to_owned()),
            )]),
            1,
        ),
        (
            3,
            structure([
                item(
                    NONCE,
                    Value::structure(structure([
                        item(NONCE_ID, Value::byte_string(vec![0x10, 0x80])),
                        item(NONCE_VALUE, Value::byte_string(vec![0x20, 0xFE])),
                    ])),
                ),
                item(ATTESTATION_TYPE, Value::enumeration(0xF123_4567)),
                item(
                    ATTESTATION_MEASUREMENT,
                    Value::byte_string(SECRET_SENTINEL.as_bytes().to_vec()),
                ),
            ]),
            3,
        ),
        (
            4,
            structure([
                item(USERNAME, Value::text_string("user".to_owned())),
                item(
                    ONE_TIME_PASSWORD,
                    Value::text_string(SECRET_SENTINEL.to_owned()),
                ),
            ]),
            2,
        ),
        (
            5,
            structure([
                item(USERNAME, Value::text_string("user".to_owned())),
                item(TIME_STAMP, Value::date_time_extended(1)),
                item(HASHING_ALGORITHM, Value::enumeration(0xF123_4567)),
                item(
                    HASHED_PASSWORD,
                    Value::byte_string(SECRET_SENTINEL.as_bytes().to_vec()),
                ),
            ]),
            4,
        ),
        (
            6,
            structure([item(
                TICKET,
                Value::structure(structure([
                    item(TICKET_TYPE, Value::enumeration(0xF123_4567)),
                    item(
                        TICKET_VALUE,
                        Value::byte_string(SECRET_SENTINEL.as_bytes().to_vec()),
                    ),
                ])),
            )]),
            1,
        ),
        (
            0x8000_0001,
            structure([item(EXTENSION_CHILD, Value::boolean(true))]),
            1,
        ),
        (
            0xF123_4567,
            structure([item(EXTENSION_CHILD, Value::boolean(false))]),
            1,
        ),
    ];

    for (raw_type, source, expected_fields) in cases {
        let value = CredentialValue::try_from_ttlv(CredentialType::from_raw(raw_type), source)
            .expect("each assigned or opaque fixture has a valid typed shape");
        let round_trip = wrapper_round_trip(value, raw_type, expected_fields);
        assert_eq!(round_trip.view().children().len(), expected_fields);
    }
}

#[test]
fn standalone_attestation_hashed_password_and_nonce_views_are_lossless() {
    let attestation_tree = structure([
        item(
            NONCE,
            Value::structure(structure([
                item(NONCE_ID, Value::byte_string(vec![0x11])),
                item(NONCE_VALUE, Value::byte_string(vec![0x22])),
            ])),
        ),
        item(ATTESTATION_TYPE, Value::enumeration(1)),
        item(ATTESTATION_MEASUREMENT, Value::byte_string(vec![0x33])),
    ]);
    let attestation = AttestationCredential::try_from_ttlv(attestation_tree)
        .expect("the Attestation fields and nested Nonce are valid");
    assert_eq!(attestation.with_ttlv(|tree| tree.children().len()), 3);
    let attestation_diagnostics = format!("{attestation:?}; {attestation}");
    assert!(!attestation_diagnostics.contains(SECRET_SENTINEL));
    assert_eq!(attestation.into_ttlv().view().children().len(), 3);

    let hashed_tree = structure([
        item(USERNAME, Value::text_string("user".to_owned())),
        item(TIME_STAMP, Value::date_time_extended(1)),
        item(HASHED_PASSWORD, Value::byte_string(vec![0x44])),
    ]);
    let hashed = HashedPasswordCredential::try_from_ttlv(hashed_tree)
        .expect("required Hashed Password members are present");
    assert_eq!(hashed.with_ttlv(|tree| tree.children().len()), 3);
    assert_eq!(hashed.hashing_algorithm_raw(), None);
    assert_eq!(hashed.effective_hashing_algorithm_raw(), 6);
    let hashed_diagnostics = format!("{hashed:?}; {hashed}");
    assert!(!hashed_diagnostics.contains(SECRET_SENTINEL));
    assert_eq!(hashed.into_ttlv().view().children().len(), 3);

    let nonce = Nonce::try_from_ttlv(structure([
        item(NONCE_ID, Value::byte_string(vec![0x10, 0x80])),
        item(NONCE_VALUE, Value::byte_string(vec![0x20, 0xFE])),
    ]))
    .expect("both server Nonce byte strings are present");
    assert_eq!(nonce.with_ttlv(|tree| tree.children().len()), 2);
    let nonce_diagnostics = format!("{nonce:?}; {nonce}");
    assert!(!nonce_diagnostics.contains(SECRET_SENTINEL));
    assert_eq!(nonce.into_ttlv().view().children().len(), 2);
}

#[test]
fn credential_validation_error_categories_are_stable_and_payload_free() {
    let categories = [
        (
            CredentialValidationErrorKind::EmptyAuthentication,
            "Authentication must contain a Credential",
        ),
        (
            CredentialValidationErrorKind::DuplicateField,
            "duplicate Credential field",
        ),
        (
            CredentialValidationErrorKind::MissingField,
            "missing required Credential field",
        ),
        (
            CredentialValidationErrorKind::WrongFieldType,
            "Credential field has an invalid Item Type",
        ),
        (
            CredentialValidationErrorKind::FieldOutOfOrder,
            "Credential fields are out of order",
        ),
        (
            CredentialValidationErrorKind::InvalidTtlvStructure,
            "invalid generic TTLV structure",
        ),
    ];
    for (category, message) in categories {
        assert_eq!(category.to_string(), message);
    }

    assert_error_kind(
        Credential::try_from_ttlv(Structure::new()),
        CredentialValidationErrorKind::MissingField,
    );
    assert_error_kind(
        Credential::try_from_ttlv(structure([
            item(CREDENTIAL_TYPE, Value::boolean(true)),
            item(CREDENTIAL_VALUE, Value::structure(Structure::new())),
        ])),
        CredentialValidationErrorKind::WrongFieldType,
    );
    assert_error_kind(
        Credential::try_from_ttlv(structure([
            item(CREDENTIAL_VALUE, Value::structure(Structure::new())),
            item(CREDENTIAL_TYPE, Value::enumeration(0xF123_4567)),
        ])),
        CredentialValidationErrorKind::FieldOutOfOrder,
    );
    assert_error_kind(
        Credential::try_from_ttlv(structure([
            item(CREDENTIAL_TYPE, Value::enumeration(1)),
            item(CREDENTIAL_TYPE, Value::enumeration(2)),
            item(CREDENTIAL_VALUE, Value::structure(Structure::new())),
        ])),
        CredentialValidationErrorKind::DuplicateField,
    );

    let empty_authentication =
        Authentication::new(Vec::new()).expect_err("empty Authentication is rejected");
    assert_eq!(
        empty_authentication.kind(),
        CredentialValidationErrorKind::EmptyAuthentication
    );
    let malformed_authentication =
        Authentication::try_from_ttlv(structure([item(CREDENTIAL, Value::boolean(true))]))
            .expect_err("a non-Structure Credential field is rejected");
    assert_eq!(
        malformed_authentication.kind(),
        CredentialValidationErrorKind::WrongFieldType
    );

    let mut deepest = Structure::new();
    for _ in 0..62 {
        deepest = structure([item(EXTENSION_CHILD, Value::structure(deepest))]);
    }
    let deep_credential = Credential::try_from_ttlv(structure([
        item(CREDENTIAL_TYPE, Value::enumeration(0xF123_4567)),
        item(CREDENTIAL_VALUE, Value::structure(deepest)),
    ]))
    .expect("the generic Credential is at the accepted model depth");
    let invalid_ttlv = Authentication::new(vec![deep_credential])
        .expect_err("Authentication cannot add a Structure beyond the local depth limit");
    assert_eq!(
        invalid_ttlv.kind(),
        CredentialValidationErrorKind::InvalidTtlvStructure
    );
}
