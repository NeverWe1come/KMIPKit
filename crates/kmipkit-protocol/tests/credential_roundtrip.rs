//! OASIS KMIP v2.1 §§9.11 and 11.11; Tables 410–416 and 442.
//!
//! These derived cases cover the generic-tree boundary. They do not claim
//! official OASIS vectors, credential verification, or wire encoding.
//!
//! Traceability: KMIPKIT-0008-FR-003, FR-004, FR-011; SC-002.

use std::fmt;

use kmipkit_protocol::Credential;
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Tag, Value, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};

const CREDENTIAL_TYPE: u32 = 0x0042_0024;
const CREDENTIAL_VALUE: u32 = 0x0042_0025;
const USERNAME: u32 = 0x0042_0099;
const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const TIME_STAMP: u32 = 0x0042_0092;
const ATTESTATION_TYPE: u32 = 0x0042_00C7;
const NONCE: u32 = 0x0042_00C8;
const NONCE_ID: u32 = 0x0042_00C9;
const NONCE_VALUE: u32 = 0x0042_00CA;
const ATTESTATION_MEASUREMENT: u32 = 0x0042_00CB;
const ONE_TIME_PASSWORD: u32 = 0x0042_0156;
const HASHED_PASSWORD: u32 = 0x0042_0157;
const TICKET: u32 = 0x0042_0149;
const TICKET_TYPE: u32 = 0x0042_014A;
const TICKET_VALUE: u32 = 0x0042_014B;
const EXTENSION_CHILD: u32 = 0x0054_0001;
const PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5438;

#[derive(Debug, Eq, PartialEq)]
struct Node {
    tag: u32,
    value: NodeValue,
}

#[derive(Debug, Eq, PartialEq)]
enum NodeValue {
    Structure(Vec<Node>),
    Integer(i32),
    LongInteger(i64),
    BigInteger(Vec<u8>),
    Enumeration(u32),
    Boolean(bool),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
    Interval(u32),
    DateTimeExtended(i64),
}

#[derive(Clone)]
struct GeneratedCredential {
    raw_type_bits: u32,
    payload: Vec<u8>,
}

impl fmt::Debug for GeneratedCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeneratedCredential")
            .field("raw_type_bits", &self.raw_type_bits)
            .field("payload_length", &self.payload.len())
            .finish()
    }
}

impl Arbitrary for GeneratedCredential {
    fn arbitrary(generator: &mut Gen) -> Self {
        let payload_len = usize::arbitrary(generator) % 257;
        let payload = (0..payload_len).map(|_| u8::arbitrary(generator)).collect();
        Self {
            raw_type_bits: u32::arbitrary(generator),
            payload,
        }
    }
}

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

fn credential(raw_type: u32, value: Structure) -> Structure {
    structure([
        item(CREDENTIAL_TYPE, Value::enumeration(raw_type)),
        item(CREDENTIAL_VALUE, Value::structure(value)),
    ])
}

fn known_value(raw_type: u32) -> Structure {
    match raw_type {
        1 => structure([item(USERNAME, Value::text_string("user".to_owned()))]),
        2 => structure([item(
            DEVICE_SERIAL_NUMBER,
            Value::text_string("serial".to_owned()),
        )]),
        3 => structure([
            item(
                NONCE,
                Value::structure(structure([
                    item(NONCE_ID, Value::byte_string(vec![0x11])),
                    item(NONCE_VALUE, Value::byte_string(vec![0x22])),
                ])),
            ),
            item(ATTESTATION_TYPE, Value::enumeration(1)),
            item(ATTESTATION_MEASUREMENT, Value::byte_string(vec![0x33])),
        ]),
        4 => structure([
            item(USERNAME, Value::text_string("user".to_owned())),
            item(
                ONE_TIME_PASSWORD,
                Value::text_string("fixture-only".to_owned()),
            ),
        ]),
        5 => structure([
            item(USERNAME, Value::text_string("user".to_owned())),
            item(TIME_STAMP, Value::date_time_extended(1)),
            item(HASHING_ALGORITHM, Value::enumeration(1)),
            item(HASHED_PASSWORD, Value::byte_string(vec![0x44])),
        ]),
        6 => structure([item(
            TICKET,
            Value::structure(structure([
                item(TICKET_TYPE, Value::enumeration(1)),
                item(TICKET_VALUE, Value::byte_string(vec![0x55])),
            ])),
        )]),
        0x8000_0001 => Structure::new(),
        _ => unreachable!("the table-driven fixture lists assigned type values"),
    }
}

fn snapshot(view: &StructureView<'_>) -> Vec<Node> {
    view.children()
        .iter()
        .map(|child| {
            let value = child.with_value(|value| match value {
                ValueView::Structure(nested) => NodeValue::Structure(snapshot(&nested)),
                ValueView::Integer(raw) => NodeValue::Integer(*raw),
                ValueView::LongInteger(raw) => NodeValue::LongInteger(*raw),
                ValueView::BigInteger(raw) => NodeValue::BigInteger(raw.to_vec()),
                ValueView::Enumeration(raw) => NodeValue::Enumeration(*raw),
                ValueView::Boolean(raw) => NodeValue::Boolean(*raw),
                ValueView::TextString(raw) => NodeValue::TextString(raw.to_owned()),
                ValueView::ByteString(raw) => NodeValue::ByteString(raw.to_vec()),
                ValueView::DateTime(raw) => NodeValue::DateTime(*raw),
                ValueView::Interval(raw) => NodeValue::Interval(*raw),
                ValueView::DateTimeExtended(raw) => NodeValue::DateTimeExtended(*raw),
                _ => unreachable!("the generic TTLV view is covered above"),
            });
            Node {
                tag: child.tag().raw(),
                value,
            }
        })
        .collect()
}

fn round_trip_preserves_tree(source: Structure) -> bool {
    let before = snapshot(&source.view());
    let credential: Credential = match Credential::try_from_ttlv(source) {
        Ok(credential) => credential,
        Err(_) => return false,
    };
    let round_trip = credential.into_ttlv();
    snapshot(&round_trip.view()) == before
}

#[test]
fn all_assigned_and_unknown_credential_types_round_trip() {
    for raw_type in [1, 2, 3, 4, 5, 6, 0x8000_0001, 0xF123_4567] {
        let source = credential(raw_type, known_value(raw_type));

        assert!(
            round_trip_preserves_tree(source),
            "credential type discriminator or generic value changed during conversion"
        );
    }
}

#[test]
fn unknown_values_and_extensions_remain_opaque() {
    let opaque_value = structure([item(
        EXTENSION_CHILD,
        Value::byte_string(vec![0x10, 0x80, 0xFF]),
    )]);
    let unknown = credential(0xF123_4567, opaque_value);
    let extension = credential(
        0x8000_0001,
        structure([item(
            EXTENSION_CHILD,
            Value::byte_string(vec![0x20, 0x81, 0xFE]),
        )]),
    );

    assert!(round_trip_preserves_tree(unknown));
    assert!(round_trip_preserves_tree(extension));
}

#[test]
fn typed_roundtrip_preserves_unknown_children() {
    let value = structure([
        item(USERNAME, Value::text_string("user".to_owned())),
        item(EXTENSION_CHILD, Value::byte_string(vec![0x20, 0x81, 0xFE])),
    ]);

    assert!(round_trip_preserves_tree(credential(1, value)));
}

#[test]
fn unknown_credential_payloads_round_trip_as_a_bounded_property() {
    QuickCheck::new()
        .tests(64)
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .quickcheck(unknown_credential_payloads_round_trip as fn(GeneratedCredential) -> bool);
}

// QuickCheck's callback consumes its generated case; Debug intentionally omits payload bytes.
#[allow(clippy::needless_pass_by_value)]
fn unknown_credential_payloads_round_trip(case: GeneratedCredential) -> bool {
    let raw_type = 0xF000_0000 | (case.raw_type_bits & 0x0FFF_FFFF);
    let value = structure([item(EXTENSION_CHILD, Value::byte_string(case.payload))]);
    round_trip_preserves_tree(credential(raw_type, value))
}
