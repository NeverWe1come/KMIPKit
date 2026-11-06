use kmipkit_client::ClientOperation;
use kmipkit_protocol::{
    AttributeSet, CreateKeyPairRequest, CreateRequest, CreateSplitKeyRequest, ObjectType,
    SplitKeyMethod, UniqueIdentifier,
};
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
    assert!(case["expectedOutput"]["resultMessage"].is_null());

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
    assert!(case["expectedOutput"]["resultMessage"].is_null());
    assert!(case["expectedOutput"]["pendingMetadata"]["resultMessage"].is_null());

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
fn every_client_operation_value_matches_kmip_21_table_470() {
    let cases = [
        (ClientOperation::Activate, 0x12),
        (ClientOperation::AddAttribute, 0x0d),
        (ClientOperation::AdjustAttribute, 0x30),
        (ClientOperation::Archive, 0x15),
        (ClientOperation::Cancel, 0x19),
        (ClientOperation::Certify, 0x06),
        (ClientOperation::Check, 0x09),
        (ClientOperation::Create, 0x01),
        (ClientOperation::CreateKeyPair, 0x02),
        (ClientOperation::CreateSplitKey, 0x28),
        (ClientOperation::Decrypt, 0x20),
        (ClientOperation::DeleteAttribute, 0x0f),
        (ClientOperation::DeriveKey, 0x05),
        (ClientOperation::Destroy, 0x14),
        (ClientOperation::DiscoverVersions, 0x1e),
        (ClientOperation::Encrypt, 0x1f),
        (ClientOperation::Export, 0x2b),
        (ClientOperation::Get, 0x0a),
        (ClientOperation::GetAttributeList, 0x0c),
        (ClientOperation::GetAttributes, 0x0b),
        (ClientOperation::GetConstraints, 0x38),
        (ClientOperation::GetUsageAllocation, 0x11),
        (ClientOperation::Hash, 0x27),
        (ClientOperation::Import, 0x2a),
        (ClientOperation::Interop, 0x34),
        (ClientOperation::JoinSplitKey, 0x29),
        (ClientOperation::Log, 0x2c),
        (ClientOperation::Login, 0x2d),
        (ClientOperation::Logout, 0x2e),
        (ClientOperation::Locate, 0x08),
        (ClientOperation::Mac, 0x23),
        (ClientOperation::MacVerify, 0x24),
        (ClientOperation::ModifyAttribute, 0x0e),
        (ClientOperation::ObtainLease, 0x10),
        (ClientOperation::Ping, 0x3b),
        (ClientOperation::Poll, 0x1a),
        (ClientOperation::Process, 0x3a),
        (ClientOperation::Query, 0x18),
        (ClientOperation::QueryAsyncRequests, 0x39),
        (ClientOperation::Recertify, 0x07),
        (ClientOperation::Recover, 0x16),
        (ClientOperation::Register, 0x03),
        (ClientOperation::Rekey, 0x04),
        (ClientOperation::RekeyKeyPair, 0x1d),
        (ClientOperation::ReProvision, 0x35),
        (ClientOperation::Revoke, 0x13),
        (ClientOperation::RngRetrieve, 0x25),
        (ClientOperation::RngSeed, 0x26),
        (ClientOperation::SetAttribute, 0x31),
        (ClientOperation::SetConstraints, 0x37),
        (ClientOperation::SetDefaults, 0x36),
        (ClientOperation::Sign, 0x21),
        (ClientOperation::SignatureVerify, 0x22),
        (ClientOperation::Validate, 0x17),
        (ClientOperation::DelegatedLogin, 0x2f),
    ];
    let cases = {
        let mut cases = cases.to_vec();
        cases.push((ClientOperation::Pkcs11, 0x33));
        cases
    };

    for (operation, raw) in cases {
        assert_eq!(operation.raw(), raw, "operation: {operation:?}");
    }
}

#[test]
fn creation_result_message_fixture_is_limited_to_operation_failure() {
    let create = fixture_case("creation.create.request-response.v1");
    assert_eq!(create["expectedOutput"]["resultStatus"], 0);
    assert!(create["expectedOutput"]["resultMessage"].is_null());

    let failure = fixture_case("creation.result-message-on-failure.v1");
    assert_eq!(failure["expectedOutput"]["resultStatus"], 1);
    assert_eq!(failure["expectedOutput"]["resultReason"], 5);
    assert_eq!(
        failure["expectedOutput"]["resultMessage"],
        "création refusée ✓"
    );
    let result = kmipkit_protocol::KmipOperationResult::new(
        kmipkit_protocol::ResultStatus::from_raw(1),
        Some(kmipkit_protocol::ResultReason::from_raw(5)),
        Some(kmipkit_protocol::ResultMessage::new(
            "création refusée ✓".to_owned(),
        )),
    )
    .expect("an Operation Failed result with a Result Reason is valid");
    assert_eq!(
        result
            .message()
            .map(kmipkit_protocol::ResultMessage::as_str),
        Some("création refusée ✓")
    );
    assert!(!format!("{result:?}").contains("création refusée"));
}
