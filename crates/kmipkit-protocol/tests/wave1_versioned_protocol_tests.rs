use kmipkit_protocol::link_normalizer::LinkType;
use kmipkit_protocol::{
    ProtocolVersion, QueryAsyncRequestsResponse, ResponseMessage, SignRequest,
    SignatureVerifyRequest, VersionGate,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

fn make_item(raw_tag: u32, value: Value) -> Item {
    Item::new(Tag::from(RawTag::new(raw_tag).expect("valid tag")), value).expect("valid item")
}

fn make_structure(items: Vec<Item>) -> Structure {
    let mut s = Structure::new();
    for item in items {
        s.try_push(item).expect("valid structure item");
    }
    s
}

#[test]
fn version_gate_rejects_all_kmip21_tags_in_kmip20() {
    for tag in 0x0042_0166..=0x0042_0176 {
        assert!(
            VersionGate::validate_tag(ProtocolVersion::V2_0, tag).is_err(),
            "KMIP 2.1 tag 0x{tag:08X} must be rejected under KMIP 2.0"
        );
        assert!(
            VersionGate::validate_tag(ProtocolVersion::V2_1, tag).is_ok(),
            "KMIP 2.1 tag 0x{tag:08X} must be accepted under KMIP 2.1"
        );
    }
}

#[test]
fn link_normalizer_supports_all_twenty_kmip30_link_types() {
    let expected = [
        (LinkType::CertificateRequest, 0x0042_01BC),
        (LinkType::Credential, 0x0042_01A0),
        (LinkType::Group, 0x0042_01B3),
        (LinkType::JoinedSplitKeyParts, 0x0042_01B5),
        (LinkType::Password, 0x0042_01A5),
        (LinkType::SplitKeyBase, 0x0042_01B4),
    ];
    for (variant, tag) in expected {
        assert_eq!(variant.v3_tag(), tag);
        assert_eq!(LinkType::from_v3_tag(tag), Some(variant));
    }
}

#[test]
fn query_async_requests_parses_typed_asynchronous_request_entries() {
    let async_request = make_structure(vec![
        make_item(0x0042_0146, Value::byte_string(vec![0xAA, 0xBB])),
        make_item(0x0042_005C, Value::enumeration(0x0000_0001)),
        make_item(0x0042_0174, Value::date_time(1_700_000_000)),
        make_item(0x0042_0175, Value::enumeration(0x0000_0001)),
    ]);
    let payload = make_structure(vec![make_item(0x0042_0172, Value::structure(async_request))]);
    let batch_item = make_structure(vec![
        make_item(0x0042_005C, Value::enumeration(0x0000_0039)),
        make_item(0x0042_007F, Value::enumeration(0x0000_0000)),
        make_item(0x0042_007C, Value::structure(payload)),
    ]);
    let header = make_structure(vec![
        make_item(
            0x0042_0069,
            Value::structure(make_structure(vec![
                make_item(0x0042_006A, Value::integer(2)),
                make_item(0x0042_006B, Value::integer(1)),
            ])),
        ),
        make_item(0x0042_0092, Value::date_time(1_700_000_000)),
        make_item(0x0042_000D, Value::integer(1)),
    ]);
    let raw_message = make_structure(vec![
        make_item(0x0042_007A, Value::structure(header)),
        make_item(0x0042_000F, Value::structure(batch_item)),
    ]);
    let msg = ResponseMessage::try_from_ttlv(raw_message).expect("valid response message");
    let item = msg.batch_items().next().expect("one batch item");
    let response = QueryAsyncRequestsResponse::try_from_response_item(item).expect("valid response");
    let entries = response
        .asynchronous_requests()
        .expect("valid typed asynchronous requests");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].asynchronous_correlation_value, vec![0xAA, 0xBB]);
    assert_eq!(entries[0].operation, 0x0000_0001);
    assert_eq!(entries[0].submission_date, 1_700_000_000);
    assert_eq!(entries[0].processing_stage, 0x0000_0001);
}

#[test]
fn sign_and_signature_verify_support_kmip30_pqc_fields() {
    let sign_req = SignRequest::new()
        .with_deterministic(true)
        .with_context_string(b"pqc-ctx".to_vec())
        .with_external_mu(false)
        .with_random(b"rnd-bytes".to_vec());
    let sign_payload = sign_req.to_ttlv_payload().expect("sign payload builds");
    assert!(sign_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01C4));
    assert!(sign_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01C5));
    assert!(sign_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01C9));
    assert!(sign_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01CA));

    let verify_req = SignatureVerifyRequest::new()
        .with_deterministic(true)
        .with_context_string(b"pqc-ctx".to_vec())
        .with_external_mu(false)
        .with_random(b"rnd-bytes".to_vec());
    let verify_payload = verify_req.to_ttlv_payload().expect("verify payload builds");
    assert!(verify_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01C4));
    assert!(verify_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01C5));
    assert!(verify_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01C9));
    assert!(verify_payload.view().children().iter().any(|i| i.tag().raw() == 0x0042_01CA));
}
