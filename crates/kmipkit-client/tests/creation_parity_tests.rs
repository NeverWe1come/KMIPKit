use kmipkit_protocol::{
    AttributeSet, CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest, ObjectType,
    SplitKeyMethod, UniqueIdentifier,
};
use kmipkit_client::ClientOperation;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use serde_json::Value as JsonValue;

const SHARED_FIXTURE: &str =
    include_str!("../../../tests/fixtures/api/shared-capability-parity.json");

fn fixture_case(id: &str) -> JsonValue {
    let fixture: JsonValue =
        serde_json::from_str(SHARED_FIXTURE).expect("shared creation fixture is valid JSON");
    fixture["cases"]
        .as_array()
        .expect("shared fixture cases are an array")
        .iter()
        .find(|case| case["id"] == id)
        .cloned()
        .expect("shared creation fixture case exists")
}

fn item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("shared fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("shared fixture tag is valid");
    Item::new(tag, value).expect("shared fixture item is valid")
}

fn attribute_set(tags: &[u32]) -> AttributeSet {
    AttributeSet::try_new(
        tags.iter()
            .map(|raw_tag| item(*raw_tag, Value::enumeration(0xdead_beef))),
    )
    .expect("unknown attributes remain generic and ordered")
}

fn payload_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

#[test]
fn create_request_and_result_match_shared_creation_fixture() {
    let case = fixture_case("creation.create.request-response.v1");
    assert_eq!(case["input"]["attributesPresent"], true);
    assert_eq!(case["expectedOutput"]["resultMessage"], "création ✓");

    let payload = CreateRequest::new(ObjectType::from_raw(0xffff_ffff), AttributeSet::new())
        .with_protection_storage_masks(Structure::new())
        .into_ttlv_payload()
        .expect("Create fixture request encodes");
    assert_eq!(
        payload_tags(&payload),
        vec![0x0042_0057, 0x0042_0125, 0x0042_015f]
    );
    assert!(
        payload.view().children()[0]
            .with_value(|value| matches!(value, ValueView::Enumeration(0xffff_ffff)))
    );
}

#[test]
fn create_key_pair_request_and_result_match_shared_creation_fixture() {
    let case = fixture_case("creation.key-pair.request-response.v1");
    assert_eq!(case["expectedOutput"]["groupsRemainDistinct"], true);
    assert_eq!(case["expectedOutput"]["privateMasksPresent"], false);

    let payload = CreateKeyPairRequest::new()
        .with_common_attributes(attribute_set(&[0x0054_0001, 0x0054_0001]))
        .with_private_key_attributes(attribute_set(&[0x0054_0002, 0x0054_0002]))
        .with_public_key_attributes(attribute_set(&[0x0054_0003]))
        .with_common_protection_storage_masks(Structure::new())
        .with_public_protection_storage_masks(Structure::new())
        .into_ttlv_payload()
        .expect("Create Key Pair fixture request encodes");
    assert_eq!(
        payload_tags(&payload),
        vec![
            0x0042_0126,
            0x0042_0127,
            0x0042_0128,
            0x0042_0163,
            0x0042_0165
        ]
    );
    let group_tags = payload.view().children()[..3]
        .iter()
        .map(|group| {
            group.with_value(|value| match value {
                ValueView::Structure(structure) => structure
                    .children()
                    .iter()
                    .map(|attribute| attribute.tag().raw())
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        group_tags,
        vec![
            vec![0x0054_0001, 0x0054_0001],
            vec![0x0054_0002, 0x0054_0002],
            vec![0x0054_0003]
        ]
    );
}

#[test]
fn create_split_key_request_and_result_match_shared_creation_fixture() {
    let case = fixture_case("creation.split-key.request-response.v1");
    assert_eq!(case["input"]["splitKeyParts"], -3);
    assert_eq!(case["expectedOutput"]["automaticFollowUp"], false);
    assert_eq!(
        case["expectedOutput"]["pendingMetadata"]["correlationValueHex"],
        "00ff80"
    );

    let payload = CreateSplitKeyRequest::new(
        ObjectType::from_raw(0xf000_0000),
        -3,
        4,
        SplitKeyMethod::from_raw(0xdead_beef),
        AttributeSet::new(),
    )
    .with_unique_identifier(UniqueIdentifier::Integer(-7))
    .with_protection_storage_masks(Structure::new())
    .into_ttlv_payload()
    .expect("Create Split Key fixture request encodes");
    assert_eq!(
        payload_tags(&payload),
        vec![
            0x0042_0057,
            0x0042_0094,
            0x0042_008b,
            0x0042_008c,
            0x0042_008a,
            0x0042_0125,
            0x0042_015f
        ]
    );
    assert!(
        payload.view().children()[2].with_value(|value| matches!(value, ValueView::Integer(-3)))
    );
    assert!(
        payload.view().children()[4]
            .with_value(|value| matches!(value, ValueView::Enumeration(0xdead_beef)))
    );
}

#[test]
fn creation_operation_values_match_kmip_21_table_470() {
    assert_eq!(ClientOperation::Create.raw(), 0x0000_0001);
    assert_eq!(ClientOperation::CreateKeyPair.raw(), 0x0000_0002);
    assert_eq!(ClientOperation::CreateSplitKey.raw(), 0x0000_0028);
}

#[test]
fn creation_result_message_fixture_is_limited_to_operation_failure() {
    let create = fixture_case("creation.create.request-response.v1");
    assert_eq!(create["expectedOutput"]["resultStatus"], 0);
    assert!(create["expectedOutput"]["resultMessage"].is_null());

    let failure = fixture_case("creation.result-message-on-failure.v1");
    assert_eq!(failure["expectedOutput"]["resultStatus"], 1);
    assert_eq!(failure["expectedOutput"]["resultReason"], 5);
    assert_eq!(failure["expectedOutput"]["resultMessage"], "création refusée ✓");
}
