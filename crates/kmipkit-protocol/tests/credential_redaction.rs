//! OASIS KMIP v2.1 §§9.11 and 9.14; Tables 411–416 and 419.
//!
//! These are derived diagnostic-redaction tests. Sentinel values are test
//! fixtures only and do not represent real credentials or Nonces.
//!
//! Traceability: KMIPKIT-0008-FR-010; SC-005.

use std::fmt::{Debug, Display};

use kmipkit_protocol::{Authentication, Credential, CredentialType, CredentialValue, Nonce};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const USERNAME: u32 = 0x0042_0099;
const PASSWORD: u32 = 0x0042_00A1;
const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const TIME_STAMP: u32 = 0x0042_0092;
const ATTESTATION_TYPE: u32 = 0x0042_00C7;
const NONCE: u32 = 0x0042_00C8;
const NONCE_ID: u32 = 0x0042_00C9;
const NONCE_VALUE: u32 = 0x0042_00CA;
const ATTESTATION_MEASUREMENT: u32 = 0x0042_00CB;
const ATTESTATION_ASSERTION: u32 = 0x0042_00CC;
const ONE_TIME_PASSWORD: u32 = 0x0042_0156;
const HASHED_PASSWORD: u32 = 0x0042_0157;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;

const USERNAME_SENTINEL: &str = "KMIPKIT_USERNAME_SENTINEL_736563726574";
const PASSWORD_SENTINEL: &str = "KMIPKIT_PASSWORD_SENTINEL_736563726574";
const DEVICE_SENTINEL: &str = "KMIPKIT_DEVICE_SENTINEL_736563726574";
const OTP_SENTINEL: &str = "KMIPKIT_OTP_SENTINEL_736563726574";
const HASH_SENTINEL: &str = "KMIPKIT_HASH_SENTINEL_736563726574";
const TICKET_SENTINEL: &str = "KMIPKIT_TICKET_SENTINEL_736563726574";
const NONCE_ID_SENTINEL: &str = "KMIPKIT_NONCE_ID_SENTINEL_736563726574";
const NONCE_VALUE_SENTINEL: &str = "KMIPKIT_NONCE_VALUE_SENTINEL_736563726574";
const MEASUREMENT_SENTINEL: &str = "KMIPKIT_MEASUREMENT_SENTINEL_736563726574";
const ASSERTION_SENTINEL: &str = "KMIPKIT_ASSERTION_SENTINEL_736563726574";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the sentinel fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the sentinel fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("the checked fixture tag forms a valid Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the sentinel fixture stays within the TTLV depth limit");
    }
    structure
}

fn typed_value(raw_type: u32) -> Structure {
    match raw_type {
        1 => structure([
            item(USERNAME, Value::text_string(USERNAME_SENTINEL.to_owned())),
            item(PASSWORD, Value::text_string(PASSWORD_SENTINEL.to_owned())),
        ]),
        2 => structure([item(
            DEVICE_SERIAL_NUMBER,
            Value::text_string(DEVICE_SENTINEL.to_owned()),
        )]),
        3 => structure([
            item(
                NONCE,
                Value::structure(structure([
                    item(
                        NONCE_ID,
                        Value::byte_string(NONCE_ID_SENTINEL.as_bytes().to_vec()),
                    ),
                    item(
                        NONCE_VALUE,
                        Value::byte_string(NONCE_VALUE_SENTINEL.as_bytes().to_vec()),
                    ),
                ])),
            ),
            item(ATTESTATION_TYPE, Value::enumeration(0xF123_4567)),
            item(
                ATTESTATION_MEASUREMENT,
                Value::byte_string(MEASUREMENT_SENTINEL.as_bytes().to_vec()),
            ),
            item(
                ATTESTATION_ASSERTION,
                Value::byte_string(ASSERTION_SENTINEL.as_bytes().to_vec()),
            ),
        ]),
        4 => structure([
            item(USERNAME, Value::text_string(USERNAME_SENTINEL.to_owned())),
            item(
                ONE_TIME_PASSWORD,
                Value::text_string(OTP_SENTINEL.to_owned()),
            ),
        ]),
        5 => structure([
            item(USERNAME, Value::text_string(USERNAME_SENTINEL.to_owned())),
            item(TIME_STAMP, Value::date_time_extended(1)),
            item(HASHING_ALGORITHM, Value::enumeration(6)),
            item(
                HASHED_PASSWORD,
                Value::byte_string(HASH_SENTINEL.as_bytes().to_vec()),
            ),
        ]),
        6 => structure([item(
            TICKET,
            Value::structure(structure([
                item(TICKET_TYPE, Value::enumeration(0xF123_4567)),
                item(
                    TICKET_VALUE,
                    Value::byte_string(TICKET_SENTINEL.as_bytes().to_vec()),
                ),
            ])),
        )]),
        _ => unreachable!("the redaction fixture lists every secret-bearing type"),
    }
}

fn credential_tree(raw_type: u32) -> Structure {
    structure([
        item(CREDENTIAL_TYPE, Value::enumeration(raw_type)),
        item(CREDENTIAL_VALUE, Value::structure(typed_value(raw_type))),
    ])
}

fn captured_log_record(value: &(impl Debug + Display)) -> String {
    // The protocol crate has no production logging dependency or call sites;
    // this sink captures the Debug/Display records an application logger sees.
    format!("debug={value:?}; display={value}")
}

fn assert_redacted(text: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            !text.contains(sentinel),
            "diagnostic leaked sentinel {sentinel}"
        );
    }
}

#[test]
fn secret_sentinels_are_absent_from_variant_diagnostics_and_captured_log_records() {
    let variants: &[(u32, &[&str])] = &[
        (1, &[USERNAME_SENTINEL, PASSWORD_SENTINEL]),
        (2, &[DEVICE_SENTINEL]),
        (
            3,
            &[
                NONCE_ID_SENTINEL,
                NONCE_VALUE_SENTINEL,
                MEASUREMENT_SENTINEL,
                ASSERTION_SENTINEL,
            ],
        ),
        (4, &[USERNAME_SENTINEL, OTP_SENTINEL]),
        (5, &[USERNAME_SENTINEL, HASH_SENTINEL]),
        (6, &[TICKET_SENTINEL]),
    ];

    for (raw_type, sentinels) in variants {
        let typed = CredentialValue::try_from_ttlv(
            CredentialType::from_raw(*raw_type),
            typed_value(*raw_type),
        )
        .expect("the sentinel fixture is structurally valid");
        assert_redacted(&format!("{typed:?}; {typed}"), sentinels);
        assert_redacted(&captured_log_record(&typed), sentinels);

        let credential = Credential::try_from_ttlv(credential_tree(*raw_type))
            .expect("the outer Credential fields are valid");
        assert_redacted(&format!("{credential:?}; {credential}"), sentinels);
        assert_redacted(&captured_log_record(&credential), sentinels);

        let authentication = Authentication::try_from_ttlv(structure([item(
            0x0042_0023,
            Value::structure(credential_tree(*raw_type)),
        )]))
        .expect("Authentication contains one structurally valid Credential");
        assert_redacted(&format!("{authentication:?}; {authentication}"), sentinels);
        assert_redacted(&captured_log_record(&authentication), sentinels);
    }
}

#[test]
fn nonce_and_validation_error_diagnostics_do_not_expose_sentinels() {
    let nonce_sentinels = [NONCE_ID_SENTINEL, NONCE_VALUE_SENTINEL];
    let nonce = Nonce::try_from_ttlv(structure([
        item(
            NONCE_ID,
            Value::byte_string(NONCE_ID_SENTINEL.as_bytes().to_vec()),
        ),
        item(
            NONCE_VALUE,
            Value::byte_string(NONCE_VALUE_SENTINEL.as_bytes().to_vec()),
        ),
    ]))
    .expect("the Nonce fixture contains both required fields");
    assert_redacted(&format!("{nonce:?}; {nonce}"), &nonce_sentinels);
    assert_redacted(&captured_log_record(&nonce), &nonce_sentinels);

    let invalid_values: [(u32, Structure, &[&str]); 6] = [
        (
            1,
            structure([item(
                PASSWORD,
                Value::text_string(PASSWORD_SENTINEL.to_owned()),
            )]),
            &[PASSWORD_SENTINEL],
        ),
        (
            2,
            structure([item(
                PASSWORD,
                Value::text_string(PASSWORD_SENTINEL.to_owned()),
            )]),
            &[PASSWORD_SENTINEL],
        ),
        (
            3,
            structure([item(
                ATTESTATION_MEASUREMENT,
                Value::byte_string(MEASUREMENT_SENTINEL.as_bytes().to_vec()),
            )]),
            &[MEASUREMENT_SENTINEL],
        ),
        (
            4,
            structure([
                item(USERNAME, Value::text_string(USERNAME_SENTINEL.to_owned())),
                item(PASSWORD, Value::text_string(PASSWORD_SENTINEL.to_owned())),
            ]),
            &[USERNAME_SENTINEL, PASSWORD_SENTINEL],
        ),
        (
            5,
            structure([
                item(USERNAME, Value::text_string(USERNAME_SENTINEL.to_owned())),
                item(
                    HASHED_PASSWORD,
                    Value::byte_string(HASH_SENTINEL.as_bytes().to_vec()),
                ),
            ]),
            &[USERNAME_SENTINEL, HASH_SENTINEL],
        ),
        (
            6,
            structure([item(
                TICKET,
                Value::structure(structure([item(
                    TICKET_VALUE,
                    Value::byte_string(TICKET_SENTINEL.as_bytes().to_vec()),
                )])),
            )]),
            &[TICKET_SENTINEL],
        ),
    ];

    for (raw_type, invalid, sentinels) in invalid_values {
        let error = CredentialValue::try_from_ttlv(CredentialType::from_raw(raw_type), invalid)
            .expect_err("missing required fields make each sentinel fixture invalid");
        let diagnostics = format!("{error:?}; {error}");
        assert_redacted(&diagnostics, sentinels);
        assert_redacted(&captured_log_record(&error), sentinels);
    }

    let invalid_nonce = Nonce::try_from_ttlv(structure([item(
        NONCE_ID,
        Value::byte_string(NONCE_ID_SENTINEL.as_bytes().to_vec()),
    )]))
    .expect_err("Nonce Value is required");
    assert_redacted(
        &format!("{invalid_nonce:?}; {invalid_nonce}"),
        &nonce_sentinels,
    );
}
