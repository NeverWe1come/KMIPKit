//! Public Device Credential binding contract.
//!
//! Traceability: KMIPKIT-0008-FR-005; KMIPKIT-REQ-SPEC-9.11-004-001,
//! KMIPKIT-REQ-SPEC-9.11-004-002, and KMIPKIT-REQ-SPEC-9.11-004-003;
//! OASIS KMIP v2.1 §9.11 Table 412.

use kmipkit_protocol::{CredentialValidationErrorKind, DeviceCredential};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const PASSWORD: u32 = 0x0042_00A1;
const DEVICE_IDENTIFIER: u32 = 0x0042_00A2;
const NETWORK_IDENTIFIER: u32 = 0x0042_00AB;
const MACHINE_IDENTIFIER: u32 = 0x0042_00A9;
const MEDIA_IDENTIFIER: u32 = 0x0042_00AA;
const EXTENSION_CHILD: u32 = 0x0054_0001;

fn item(raw: u32, value: Value) -> Item {
    let tag = RawTag::new(raw)
        .expect("fixture tag is within TTLV bounds")
        .try_checked()
        .expect("fixture tag is allocated");
    Item::new(tag, value).expect("fixture item is structurally valid")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for child in items {
        result
            .try_push(child)
            .expect("fixture is within depth limits");
    }
    result
}

#[test]
fn device_credential_direct_constructor_preserves_table_412_fields_and_order() {
    let source = structure([
        item(DEVICE_SERIAL_NUMBER, Value::text_string(String::new())),
        item(
            PASSWORD,
            Value::text_string("sensitive-sentinel".to_owned()),
        ),
        item(
            DEVICE_IDENTIFIER,
            Value::text_string("device-id".to_owned()),
        ),
        item(
            NETWORK_IDENTIFIER,
            Value::text_string("network-id".to_owned()),
        ),
        item(
            MACHINE_IDENTIFIER,
            Value::text_string("machine-id".to_owned()),
        ),
        item(MEDIA_IDENTIFIER, Value::text_string("media-id".to_owned())),
        item(EXTENSION_CHILD, Value::integer(7)),
    ]);

    let credential = DeviceCredential::try_from_ttlv(source)
        .expect("all six Table 412 text fields and an unknown child are preserved");
    let tags = credential.with_ttlv(|view| {
        view.children()
            .iter()
            .map(|child| child.tag().raw())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        tags,
        [
            DEVICE_SERIAL_NUMBER,
            PASSWORD,
            DEVICE_IDENTIFIER,
            NETWORK_IDENTIFIER,
            MACHINE_IDENTIFIER,
            MEDIA_IDENTIFIER,
            EXTENSION_CHILD,
        ]
    );
    let diagnostics = format!("{credential:?}; {credential}");
    assert!(!diagnostics.contains("sensitive-sentinel"));
    let restored = credential.into_ttlv();
    assert_eq!(
        restored
            .view()
            .children()
            .iter()
            .map(|child| child.tag().raw())
            .collect::<Vec<_>>(),
        tags
    );
}

#[test]
fn device_credential_direct_constructor_requires_one_of_the_four_identifiers() {
    let error = DeviceCredential::try_from_ttlv(structure([item(
        PASSWORD,
        Value::text_string("sensitive-sentinel".to_owned()),
    )]))
    .expect_err("Password alone does not satisfy the Table 412 identifier requirement");

    assert_eq!(error.kind(), CredentialValidationErrorKind::MissingField);
    assert!(!error.to_string().contains("sensitive-sentinel"));
}
