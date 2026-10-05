//! OASIS KMIP Specification v2.1: §§8.3, 8.6, 9.21; Tables 396 and 399.
//!
//! Traceability: KMIPKIT-0006-FR-004; SC-002 and SC-003.

use kmipkit_protocol::{RequestMessage, ResponseMessage};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};
use quickcheck::{Arbitrary, Gen, QuickCheck};

const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_HEADER: u32 = 0x0042_0077;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_HEADER: u32 = 0x0042_007A;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_STATUS: u32 = 0x0042_007F;
const TIME_STAMP: u32 = 0x0042_0092;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5430;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the fixture tag is allocated by KMIP 2.1")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("the checked tag and value form an item")
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

fn version() -> Value {
    Value::structure(structure([
        item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]))
}

fn request(items: impl IntoIterator<Item = (Vec<u8>,)>) -> Structure {
    request_optional_ids(items.into_iter().map(|(id,)| Some(id)))
}

fn request_optional_ids(items: impl IntoIterator<Item = Option<Vec<u8>>>) -> Structure {
    let items: Vec<_> = items.into_iter().collect();
    let count = i32::try_from(items.len()).expect("test batch count fits i32");
    let header = structure([
        item(PROTOCOL_VERSION, version()),
        item(BATCH_COUNT, Value::integer(count)),
    ]);
    let mut message = vec![item(REQUEST_HEADER, Value::structure(header))];
    message.extend(items.into_iter().map(|id| {
        let mut fields = vec![item(OPERATION, Value::enumeration(1))];
        if let Some(id) = id {
            fields.push(item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id)));
        }
        fields.push(item(REQUEST_PAYLOAD, Value::structure(Structure::new())));
        item(BATCH_ITEM, Value::structure(structure(fields)))
    }));
    structure(message)
}

fn response(id: &[u8]) -> Structure {
    let header = structure([
        item(PROTOCOL_VERSION, version()),
        item(TIME_STAMP, Value::date_time(1)),
        item(BATCH_COUNT, Value::integer(1)),
    ]);
    structure([
        item(RESPONSE_HEADER, Value::structure(header)),
        item(
            BATCH_ITEM,
            Value::structure(structure([
                item(OPERATION, Value::enumeration(1)),
                item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())),
                item(RESULT_STATUS, Value::enumeration(0)),
                item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
            ])),
        ),
    ])
}

#[test]
fn multi_item_request_rejects_repeated_unique_batch_item_ids() {
    let duplicate_id = vec![0x41, 0x00, 0x42];
    let tree = request([(duplicate_id.clone(),), (duplicate_id,)]);

    assert!(RequestMessage::try_from_ttlv(tree).is_err());
}

#[test]
fn response_batch_item_preserves_the_exact_echoed_id() {
    let id = [0x00, 0x91, 0x00, 0xF2];
    let response = ResponseMessage::try_from_ttlv(response(&id))
        .expect("a success response may echo its request item ID");

    assert_eq!(
        response
            .batch_items()
            .next()
            .expect("the response has one item")
            .with_unique_batch_item_id(<[u8]>::to_vec),
        Some(id.to_vec())
    );
}

#[test]
fn a_single_item_request_may_omit_its_unique_id() {
    let message = RequestMessage::try_from_ttlv(request_optional_ids([None]))
        .expect("Table 396 makes the ID optional for a single item");

    assert_eq!(
        message
            .batch_items()
            .next()
            .expect("the request has one item")
            .with_unique_batch_item_id(<[u8]>::to_vec),
        None
    );
}

#[test]
fn every_multi_item_request_requires_an_id() {
    let message = request_optional_ids([Some(vec![0x01]), None]);

    assert!(RequestMessage::try_from_ttlv(message).is_err());
}

#[derive(Clone, Debug)]
struct IdCase(Vec<Vec<u8>>);

impl Arbitrary for IdCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let item_count = 1 + (usize::arbitrary(generator) % 8);
        let ids = (0..item_count)
            .map(|index| {
                let payload_length = usize::arbitrary(generator) % 32;
                let mut id = vec![u8::try_from(index).unwrap_or_default()];
                id.extend((0..payload_length).map(|_| u8::arbitrary(generator)));
                id
            })
            .collect();
        Self(ids)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn ids_remain_attached_to_items_in_source_order(case: IdCase) -> bool {
    let expected = case.0.clone();
    let tree = request(case.0.into_iter().map(|id| (id,)));
    let Ok(message) = RequestMessage::try_from_ttlv(tree) else {
        return false;
    };
    let actual: Vec<_> = message
        .batch_items()
        .map(|batch_item| batch_item.with_unique_batch_item_id(<[u8]>::to_vec))
        .collect();
    actual == expected.into_iter().map(Some).collect::<Vec<_>>()
}

#[test]
fn request_item_ids_remain_associated_in_seeded_batch_order() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(256)
        .quickcheck(ids_remain_attached_to_items_in_source_order as fn(IdCase) -> bool);
}
