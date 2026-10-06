//! OASIS KMIP v2.1 §§9.4, 9.11; Tables 403, 410, and 442.
//!
//! These derived contract cases validate in-memory Authentication and
//! Credential models. They do not claim official OASIS test-vector coverage
//! or server-side credential satisfaction.
//!
//! Traceability: KMIPKIT-REQ-SPEC-9.4-001-001, KMIPKIT-REQ-SPEC-9.4-001-002,
//! KMIPKIT-REQ-SPEC-9.4-002; KMIPKIT-0008-FR-001, FR-002, FR-009; SC-001.

use kmipkit_protocol::{Authentication, RequestMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const CREDENTIAL: u32 = 0x0042_0023;
const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const EXTENSION_CHILD: u32 = 0x0054_0003;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("the checked tag and value form an Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the fixture stays within the model depth limit");
    }
    structure
}

fn credential(raw_type: u32, opaque_value: u32) -> Structure {
    structure([
        item(CREDENTIAL_TYPE, Value::enumeration(raw_type)),
        item(
            CREDENTIAL_VALUE,
            Value::structure(structure([item(opaque_value, Value::boolean(true))])),
        ),
    ])
}

fn request_without_authentication() -> RequestMessage {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = structure([
        item(PROTOCOL_VERSION, Value::structure(version)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION, Value::enumeration(1)),
        item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
    ]);
    RequestMessage::try_from_ttlv(structure([
        item(REQUEST_HEADER, Value::structure(header)),
        item(BATCH_ITEM, Value::structure(batch_item)),
    ]))
    .expect("the request fixture has a valid single-item KMIP envelope")
}

#[test]
fn authentication_absence_is_distinct_from_present_value() {
    let request = request_without_authentication();

    assert!(request.header().with_authentication(|_| ()).is_none());
}

#[test]
fn authentication_rejects_present_empty_credentials() {
    let result = Authentication::try_from_ttlv(Structure::new());

    assert!(result.is_err());
}

#[test]
fn authentication_constructor_rejects_empty_list() {
    assert!(Authentication::new(Vec::new()).is_err());
}

#[test]
fn authentication_preserves_nonempty_credential_order() {
    let authentication = structure([
        item(
            CREDENTIAL,
            Value::structure(credential(0xF123_4567, 0x0054_0001)),
        ),
        item(
            CREDENTIAL,
            Value::structure(credential(0xE234_5678, 0x0054_0002)),
        ),
    ]);

    let parsed = Authentication::try_from_ttlv(authentication)
        .expect("repeated Credential structures remain in source order");
    let raw_types: Vec<_> = parsed
        .credentials()
        .map(|entry| entry.credential_type_raw())
        .collect();

    assert_eq!(raw_types, [0xF123_4567, 0xE234_5678]);
}

fn assert_authentication_tree_preserves_unknown_fields_and_order(roundtrip: &Structure) {
    let view = roundtrip.view();
    let fields: &[Item] = view.children();
    let tags: Vec<u32> = fields.iter().map(|field| field.tag().raw()).collect();
    let extension_values: Vec<Vec<u8>> = fields
        .iter()
        .filter(|field| field.tag().raw() == EXTENSION_CHILD)
        .map(|field| {
            field.with_value(|value| match value {
                ValueView::ByteString(bytes) => bytes.to_vec(),
                _ => Vec::new(),
            })
        })
        .collect();

    assert_eq!(
        tags,
        [EXTENSION_CHILD, CREDENTIAL, CREDENTIAL, EXTENSION_CHILD]
    );
    assert_eq!(extension_values, [vec![0x80, 0x01], vec![0xFE, 0x02]]);
}

#[test]
fn authentication_roundtrip_preserves_unknown_fields_and_credential_order() {
    let source = structure([
        item(EXTENSION_CHILD, Value::byte_string(vec![0x80, 0x01])),
        item(
            CREDENTIAL,
            Value::structure(credential(0xF123_4567, 0x0054_0001)),
        ),
        item(
            CREDENTIAL,
            Value::structure(credential(0xE234_5678, 0x0054_0002)),
        ),
        item(EXTENSION_CHILD, Value::byte_string(vec![0xFE, 0x02])),
    ]);
    let parsed = Authentication::try_from_ttlv(source)
        .expect("repeated Credentials and unknown Authentication fields are retained");

    let roundtrip = parsed.into_ttlv();
    assert_authentication_tree_preserves_unknown_fields_and_order(&roundtrip);
}

#[test]
fn authentication_does_not_claim_server_satisfaction() {
    let authentication = structure([
        item(
            CREDENTIAL,
            Value::structure(credential(0xF123_4567, 0x0054_0001)),
        ),
        item(
            CREDENTIAL,
            Value::structure(credential(0xE234_5678, 0x0054_0002)),
        ),
    ]);

    let parsed = Authentication::try_from_ttlv(authentication)
        .expect("the client model must not decide server-side credential satisfaction");

    assert_eq!(parsed.credentials().len(), 2);
}
