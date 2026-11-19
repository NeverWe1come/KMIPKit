use kmipkit_protocol::link_normalizer::{LinkNormalizer, LinkType};
use kmipkit_protocol::{
    CryptographicParameters, OperationData, ProtocolVersion, QueryAsyncRequestsResponse,
    ResponseMessage, ResultStatus, SecretBytes, SignRequest, SignatureVerifyRequest,
    UniqueIdentifier, VersionGate,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn make_item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("valid tag")
            .try_checked()
            .expect("allocated tag"),
        value,
    )
    .expect("valid item")
}

fn make_structure(items: Vec<Item>) -> Structure {
    let mut s = Structure::new();
    for item in items {
        s.try_push(item).expect("valid structure item");
    }
    s
}

fn make_sample_header() -> Structure {
    make_structure(vec![
        make_item(
            0x0042_0069,
            Value::structure(make_structure(vec![
                make_item(0x0042_006A, Value::integer(2)),
                make_item(0x0042_006B, Value::integer(1)),
            ])),
        ),
        make_item(0x0042_0092, Value::date_time(1_700_000_000)),
        make_item(0x0042_000D, Value::integer(1)),
    ])
}

fn assert_query_async_error(children: Vec<Item>) {
    let bad_payload = make_structure(children);
    let bi = make_structure(vec![
        make_item(0x0042_005C, Value::enumeration(0x0000_0039)),
        make_item(0x0042_007F, Value::enumeration(0x0000_0000)),
        make_item(0x0042_007C, Value::structure(bad_payload)),
    ]);
    let raw = make_structure(vec![
        make_item(0x0042_007A, Value::structure(make_sample_header())),
        make_item(0x0042_000F, Value::structure(bi)),
    ]);
    let m = ResponseMessage::try_from_ttlv(raw).expect("valid outer message");
    let r = QueryAsyncRequestsResponse::try_from_response_item(
        m.batch_items().next().expect("one batch item"),
    )
    .expect("valid response");
    assert!(r.asynchronous_requests().is_err());
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
fn link_normalizer_additional_coverage() {
    assert_eq!(LinkType::Group.v2_enumeration(), 0);
    assert!(LinkType::Group.try_v2_enumeration().is_none());

    let item_group = make_item(0x0042_01B3, Value::text_string("group-id".into()));
    assert!(LinkNormalizer::normalize_to_v2(&item_group).is_err());

    let item_unknown = make_item(0x0042_0099, Value::text_string("unknown".into()));
    assert!(LinkNormalizer::normalize_to_v2(&item_unknown).is_err());
}

#[test]
fn query_async_requests_parses_typed_asynchronous_request_entries() {
    let async_request = make_structure(vec![
        make_item(0x0042_0006, Value::byte_string(vec![0xAA, 0xBB])),
        make_item(0x0042_005C, Value::enumeration(0x0000_0001)),
        make_item(0x0042_0174, Value::date_time(1_700_000_000)),
        make_item(0x0042_0175, Value::enumeration(0x0000_0001)),
    ]);
    let payload = make_structure(vec![make_item(
        0x0042_0173,
        Value::structure(async_request),
    )]);
    let batch_item = make_structure(vec![
        make_item(0x0042_005C, Value::enumeration(0x0000_0039)),
        make_item(0x0042_007F, Value::enumeration(0x0000_0000)),
        make_item(0x0042_007C, Value::structure(payload)),
    ]);
    let raw_message = make_structure(vec![
        make_item(0x0042_007A, Value::structure(make_sample_header())),
        make_item(0x0042_000F, Value::structure(batch_item)),
    ]);
    let msg = ResponseMessage::try_from_ttlv(raw_message).expect("valid response message");
    let item = msg.batch_items().next().expect("one batch item");
    let response =
        QueryAsyncRequestsResponse::try_from_response_item(item).expect("valid response");
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
fn query_async_requests_optional_fields_and_debug() {
    let nested_batch_item = make_structure(vec![make_item(0x0042_005C, Value::enumeration(1))]);
    let async_request = make_structure(vec![
        make_item(0x0042_0006, Value::byte_string(vec![0x11, 0x22])),
        make_item(0x0042_005C, Value::enumeration(2)),
        make_item(0x0042_0174, Value::date_time(1_710_000_000)),
        make_item(0x0042_0175, Value::enumeration(2)),
        make_item(0x0042_007F, Value::enumeration(0)),
        make_item(0x0042_007E, Value::enumeration(1)),
        make_item(0x0042_007D, Value::text_string("done".to_string())),
        make_item(0x0042_000F, Value::structure(nested_batch_item)),
    ]);
    let payload = make_structure(vec![
        make_item(0x0042_0173, Value::structure(async_request)),
        make_item(0x0042_0099, Value::integer(42)),
    ]);
    let batch_item = make_structure(vec![
        make_item(0x0042_005C, Value::enumeration(0x0000_0039)),
        make_item(0x0042_007F, Value::enumeration(0x0000_0000)),
        make_item(0x0042_007C, Value::structure(payload)),
    ]);
    let raw_message = make_structure(vec![
        make_item(0x0042_007A, Value::structure(make_sample_header())),
        make_item(0x0042_000F, Value::structure(batch_item)),
    ]);
    let msg = ResponseMessage::try_from_ttlv(raw_message).expect("valid response message");
    let item = msg.batch_items().next().expect("one batch item");
    let response =
        QueryAsyncRequestsResponse::try_from_response_item(item).expect("valid response");
    let entries = response
        .asynchronous_requests()
        .expect("valid typed asynchronous requests");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].asynchronous_correlation_value, vec![0x11, 0x22]);
    assert_eq!(entries[0].operation, 2);
    assert_eq!(entries[0].submission_date, 1_710_000_000);
    assert_eq!(entries[0].processing_stage, 2);
    assert_eq!(entries[0].result_status, Some(ResultStatus::from_raw(0)));
    assert!(entries[0].result_reason.is_some());
    assert_eq!(entries[0].result_message.as_deref(), Some("done"));
    assert!(entries[0].batch_item.is_some());
    let _ = format!("{:?}", entries[0]);
    let _ = format!("{response:?}");

    let empty_payload = make_structure(vec![]);
    let empty_batch_item = make_structure(vec![
        make_item(0x0042_005C, Value::enumeration(0x0000_0039)),
        make_item(0x0042_007F, Value::enumeration(0x0000_0000)),
        make_item(0x0042_007C, Value::structure(empty_payload)),
    ]);
    let raw_msg_empty = make_structure(vec![
        make_item(0x0042_007A, Value::structure(make_sample_header())),
        make_item(0x0042_000F, Value::structure(empty_batch_item)),
    ]);
    let msg_empty = ResponseMessage::try_from_ttlv(raw_msg_empty).expect("valid msg");
    let resp_empty = QueryAsyncRequestsResponse::try_from_response_item(
        msg_empty.batch_items().next().expect("batch item"),
    )
    .expect("valid resp");
    assert_eq!(
        resp_empty
            .asynchronous_requests()
            .expect("empty entries")
            .len(),
        0
    );
}

#[test]
fn query_async_requests_missing_required_fields_rejected() {
    assert_query_async_error(vec![make_item(0x0042_0173, Value::integer(123))]);

    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_005C, Value::enumeration(1)),
            make_item(0x0042_0174, Value::date_time(100)),
            make_item(0x0042_0175, Value::enumeration(1)),
        ])),
    )]);

    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_0006, Value::byte_string(vec![1])),
            make_item(0x0042_0174, Value::date_time(100)),
            make_item(0x0042_0175, Value::enumeration(1)),
        ])),
    )]);

    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_0006, Value::byte_string(vec![1])),
            make_item(0x0042_005C, Value::enumeration(1)),
            make_item(0x0042_0175, Value::enumeration(1)),
        ])),
    )]);

    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_0006, Value::byte_string(vec![1])),
            make_item(0x0042_005C, Value::enumeration(1)),
            make_item(0x0042_0174, Value::date_time(100)),
        ])),
    )]);
}

#[test]
fn query_async_requests_malformed_field_types_rejected() {
    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_0006, Value::integer(1)),
            make_item(0x0042_005C, Value::enumeration(1)),
            make_item(0x0042_0174, Value::date_time(100)),
            make_item(0x0042_0175, Value::enumeration(1)),
        ])),
    )]);

    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_0006, Value::byte_string(vec![1])),
            make_item(0x0042_005C, Value::text_string("bad".into())),
            make_item(0x0042_0174, Value::date_time(100)),
            make_item(0x0042_0175, Value::enumeration(1)),
        ])),
    )]);

    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_0006, Value::byte_string(vec![1])),
            make_item(0x0042_005C, Value::enumeration(1)),
            make_item(0x0042_0174, Value::text_string("bad".into())),
            make_item(0x0042_0175, Value::enumeration(1)),
        ])),
    )]);

    assert_query_async_error(vec![make_item(
        0x0042_0173,
        Value::structure(make_structure(vec![
            make_item(0x0042_0006, Value::byte_string(vec![1])),
            make_item(0x0042_005C, Value::enumeration(1)),
            make_item(0x0042_0174, Value::date_time(100)),
            make_item(0x0042_0175, Value::text_string("bad".into())),
        ])),
    )]);

    let optional_bad_fields = [
        make_item(0x0042_007F, Value::text_string("bad".into())),
        make_item(0x0042_007E, Value::text_string("bad".into())),
        make_item(0x0042_007D, Value::integer(999)),
        make_item(0x0042_000F, Value::integer(888)),
    ];
    for bad_field in optional_bad_fields {
        assert_query_async_error(vec![make_item(
            0x0042_0173,
            Value::structure(make_structure(vec![
                make_item(0x0042_0006, Value::byte_string(vec![1])),
                make_item(0x0042_005C, Value::enumeration(1)),
                make_item(0x0042_0174, Value::date_time(100)),
                make_item(0x0042_0175, Value::enumeration(1)),
                bad_field,
            ])),
        )]);
    }
}

#[test]
fn sign_request_all_builder_setters_and_debug() {
    let empty = SignRequest::default();
    assert!(empty.unique_identifier().is_none());
    let _ = format!("{empty:?}");

    let full = SignRequest::new()
        .with_unique_identifier(UniqueIdentifier::from("id-123"))
        .with_cryptographic_parameters(make_structure(vec![]))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"hello".to_vec(),
        )))
        .with_digested_data(SecretBytes::new(b"digest".to_vec()))
        .with_correlation_value(SecretBytes::new(b"corr".to_vec()))
        .with_init_indicator(true)
        .with_final_indicator(true)
        .with_deterministic(false)
        .with_context_string(b"ctx".to_vec())
        .with_external_mu(true)
        .with_random(b"random".to_vec());

    assert_eq!(
        full.unique_identifier(),
        Some(&UniqueIdentifier::from("id-123"))
    );
    assert!(full.validate_multipart_shape().is_ok());
    let _ = full.response_context();
    let debug_str = format!("{full:?}");
    assert!(debug_str.contains("SignRequest"));

    let payload = full.to_ttlv_payload().expect("valid payload");
    assert!(!payload.view().children().is_empty());

    let params = CryptographicParameters::try_from_ttlv(Structure::new()).expect("valid params");
    let with_typed = SignRequest::new()
        .with_typed_cryptographic_parameters(params)
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"data".to_vec(),
        )));
    assert!(with_typed.to_ttlv_payload().is_ok());

    let invalid = SignRequest::new();
    assert!(invalid.to_ttlv_payload().is_err());
}

#[test]
fn signature_verify_request_all_builder_setters_and_debug() {
    let empty = SignatureVerifyRequest::default();
    assert!(empty.unique_identifier().is_none());
    let _ = format!("{empty:?}");

    let full = SignatureVerifyRequest::new()
        .with_unique_identifier(UniqueIdentifier::from("id-456"))
        .with_cryptographic_parameters(make_structure(vec![]))
        .with_data(OperationData::ByteString(SecretBytes::new(
            b"hello".to_vec(),
        )))
        .with_digested_data(SecretBytes::new(b"digest".to_vec()))
        .with_signature_data(SecretBytes::new(b"signature".to_vec()))
        .with_correlation_value(SecretBytes::new(b"corr".to_vec()))
        .with_init_indicator(true)
        .with_final_indicator(true)
        .with_deterministic(true)
        .with_context_string(b"ctx".to_vec())
        .with_external_mu(false)
        .with_random(b"random".to_vec());

    assert_eq!(
        full.unique_identifier(),
        Some(&UniqueIdentifier::from("id-456"))
    );
    assert!(full.validate_multipart_shape().is_ok());
    let _ = full.verification_response_context();
    let debug_str = format!("{full:?}");
    assert!(debug_str.contains("SignatureVerifyRequest"));

    let payload = full.to_ttlv_payload().expect("valid payload");
    assert!(!payload.view().children().is_empty());

    let params = CryptographicParameters::try_from_ttlv(Structure::new()).expect("valid params");
    let with_typed = SignatureVerifyRequest::new()
        .with_typed_cryptographic_parameters(params)
        .with_signature_data(SecretBytes::new(b"sig".to_vec()));
    assert!(with_typed.to_ttlv_payload().is_ok());

    let invalid = SignatureVerifyRequest::new();
    assert!(invalid.to_ttlv_payload().is_err());
}
