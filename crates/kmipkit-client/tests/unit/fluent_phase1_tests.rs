//! Unit tests for Fase 1 ergonomics: `ClientBuilder`, embedded `CodecLimits`,
//! typed fluent `.execute()` / `.execute_outcome()`, and multi-version adaptation (2.0/2.1/3.0).

use kmipkit_protocol::{
    LinkType, ManagedObject, OpaqueObject, ProtocolVersion, ResultStatus, UniqueIdentifier,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, KmipTag};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::decode_request_message_for_test;
use crate::execute_test_support::{
    ResponseItemFixture, asynchronous_response_bytes, asynchronous_response_bytes_with_version,
    one_item_response_bytes, response_bytes, test_structure,
};
use crate::extension_registry::ClientConfiguration;
use crate::fluent::CryptographicUsageMask;
use crate::{Client, ClientBatch, ClientBatchItem, ClientCauseCategory, ClientRequest};

fn find_child_by_tag(items: &[Item], raw_tag: u32) -> Option<&Item> {
    items.iter().find(|it| it.tag().raw() == raw_tag)
}

#[test]
fn client_configuration_embeds_default_and_custom_codec_limits() {
    let config = ClientConfiguration::default();
    assert_eq!(config.protocol_version(), ProtocolVersion::V2_1);
    assert_eq!(config.codec_limits(), &CodecLimits::defaults());

    let custom_limits = CodecLimits::new(1024 * 1024, 32, 2048).expect("valid custom limits");
    let updated = config.with_codec_limits(custom_limits);
    assert_eq!(updated.codec_limits(), &custom_limits);
}

#[test]
fn request_options_allow_per_call_codec_limit_overrides() {
    let base = CodecLimits::defaults();
    let opts = RequestOptions::default()
        .with_max_message_bytes(4096)
        .with_max_structure_depth(16)
        .with_max_elements(256);
    assert_eq!(opts.max_message_bytes(), Some(4096));
    assert_eq!(opts.max_structure_depth(), Some(16));
    assert_eq!(opts.max_elements(), Some(256));

    let effective = CodecLimits::new(
        opts.max_message_bytes()
            .unwrap_or_else(|| base.max_message_bytes()),
        opts.max_structure_depth()
            .unwrap_or_else(|| base.max_structure_depth()),
        opts.max_elements().unwrap_or_else(|| base.max_elements()),
    )
    .expect("valid effective limits");
    assert_eq!(
        effective,
        CodecLimits::new(4096, 16, 256).expect("expected limits")
    );
}

#[test]
fn client_builder_validates_inputs_locally_without_network() {
    let Err(err) = Client::builder().build() else {
        panic!("building Client without endpoint must fail locally");
    };
    assert_eq!(
        err.cause_category(),
        Some(ClientCauseCategory::InvalidInput)
    );
    assert_eq!(err.delivery_state(), Some(RequestDeliveryState::NotSent));

    let Err(err) = Client::connect_mtls(
        "127.0.0.1",
        5696,
        "/nonexistent/kmipkit/ca.pem",
        "/nonexistent/kmipkit/cert.pem",
        "/nonexistent/kmipkit/key.pem",
    ) else {
        panic!("nonexistent PEM paths must fail immediately");
    };
    assert_eq!(
        err.cause_category(),
        Some(ClientCauseCategory::InvalidInput)
    );
    assert_eq!(err.delivery_state(), Some(RequestDeliveryState::NotSent));
}

#[test]
fn fluent_execute_returns_typed_response_and_errors_on_failure_or_pending() {
    // 1. Success returns direct typed PingResponse
    let ping_ok = asynchronous_response_bytes(0x0000_003b, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: ping_ok,
        request_write_chunks: Vec::new(),
    });
    let resp = client
        .request()
        .ping()
        .execute()
        .expect("Ping execute() returns direct response on success");
    assert_eq!(resp.result().status(), ResultStatus::from_raw(0));

    // 2. Server rejection (OperationFailed = 1) becomes Err(ClientError::ServerResult) on execute()
    //    and Ok(ClientBatchItemResponse) on execute_outcome()
    let fail_bytes = one_item_response_bytes(0x0000_003b, 1, Some(1), None, None, None, None);
    let (mut client_fail, _, _) = client_for(ExchangeScript::Success {
        response: fail_bytes.clone(),
        request_write_chunks: Vec::new(),
    });
    let err = client_fail
        .request()
        .ping()
        .execute()
        .expect_err("execute() must error on ServerResult failure");
    assert!(err.is_server_rejected());
    assert!(!err.is_pending());
    assert_eq!(err.server_status(), Some(ResultStatus::from_raw(1)));

    let (mut client_fail_outcome, _, _) = client_for(ExchangeScript::Success {
        response: fail_bytes,
        request_write_chunks: Vec::new(),
    });
    let outcome = client_fail_outcome
        .request()
        .ping()
        .execute_outcome()
        .expect("execute_outcome() preserves server failure without Err");
    assert_eq!(
        outcome.outcome().result().status(),
        ResultStatus::from_raw(1)
    );

    // 3. Pending (ResultStatus = 2) becomes Err(ClientError::Pending) on execute() with redacted correlation
    let pending_bytes = asynchronous_response_bytes(
        0x0000_001e,
        2,
        None,
        Some(b"SECRET_CORR_99"),
        Some(test_structure([])),
    );
    let (mut client_pending, _, _) = client_for(ExchangeScript::Success {
        response: pending_bytes.clone(),
        request_write_chunks: Vec::new(),
    });
    let err = client_pending
        .request()
        .discover_versions()
        .with_asynchronous_indicator(1)
        .execute()
        .expect_err("execute() must return ClientError::Pending on Pending status");
    assert!(err.is_pending(), "expected Pending error, got {err:?}");
    assert!(!err.is_server_rejected());
    assert_eq!(err.server_status(), Some(ResultStatus::from_raw(2)));
    assert_eq!(
        err.asynchronous_correlation_value(),
        Some(b"SECRET_CORR_99".as_slice())
    );
    assert!(!format!("{err:?}").contains("SECRET_CORR_99"));
    assert!(!format!("{err}").contains("SECRET_CORR_99"));

    let (mut client_pending_outcome, _, _) = client_for(ExchangeScript::Success {
        response: pending_bytes,
        request_write_chunks: Vec::new(),
    });
    let outcome = client_pending_outcome
        .request()
        .discover_versions()
        .with_asynchronous_indicator(1)
        .execute_outcome()
        .expect("execute_outcome() returns Pending outcome without erroring");
    assert!(matches!(
        outcome.outcome(),
        crate::ClientBatchOutcome::Pending(_)
    ));
}

#[test]
#[allow(clippy::too_many_lines)]
fn fluent_adapts_names_links_and_identifiers_across_kmip_versions() {
    let limits = CodecLimits::defaults();
    let (mut client, _, _) = client_for(ExchangeScript::FailBeforeWrite);

    // KMIP 2.1: CreateSymmetricKey::name() -> Name (0x420053)
    let req_21 = client
        .request()
        .create_symmetric_key()
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .name("key-v21")
        .build_request()
        .expect("valid v2.1 CreateSymmetricKey request");
    let payload_21 = req_21.into_ttlv_payload().expect("v2.1 payload");
    let attrs_21 = find_child_by_tag(payload_21.view().children(), KmipTag::Attributes.as_u32())
        .and_then(Item::as_structure)
        .expect("Attributes structure in v2.1 Create")
        .children();
    assert!(find_child_by_tag(attrs_21, KmipTag::Name.as_u32()).is_some());
    assert!(find_child_by_tag(attrs_21, 0x0042_00BF).is_none());

    // KMIP 3.0 via with_protocol_version(V3_0): name() -> AlternativeName (0x4200BF),
    // AlternativeNameValue (0x4200C0) BEFORE AlternativeNameType=1 (0x4200C1)
    let req_30 = client
        .request()
        .create_symmetric_key()
        .with_protocol_version(ProtocolVersion::V3_0)
        .aes_256()
        .usage_mask(CryptographicUsageMask::ENCRYPT)
        .name("key-v30")
        .build_request()
        .expect("valid v3.0 CreateSymmetricKey request");
    let payload_30 = req_30.into_ttlv_payload().expect("v3.0 payload");
    let attrs_30 = find_child_by_tag(payload_30.view().children(), KmipTag::Attributes.as_u32())
        .and_then(Item::as_structure)
        .expect("Attributes structure in v3.0 Create")
        .children();
    assert!(find_child_by_tag(attrs_30, KmipTag::Name.as_u32()).is_none());
    let alt_item = find_child_by_tag(attrs_30, 0x0042_00BF)
        .and_then(Item::as_structure)
        .expect("KMIP 3.0 request must emit AlternativeName (0x4200BF) Structure");
    let tags: Vec<u32> = alt_item.children().iter().map(|c| c.tag().raw()).collect();
    assert_eq!(tags, vec![0x0042_00C0, 0x0042_00C1]);

    // Key Pair names adapt cleanly in KMIP 3.0
    let kp_30 = client
        .request()
        .create_key_pair()
        .with_kmip_version(ProtocolVersion::V3_0)
        .algorithm(kmipkit_protocol::CryptographicAlgorithm::from_raw(6))
        .length(256)
        .private_name("priv-v30")
        .public_name("pub-v30")
        .build_request()
        .expect("valid v3.0 CreateKeyPair request");
    let kp_payload = kp_30.into_ttlv_payload().expect("v3.0 kp payload");
    let has_private_alt = find_child_by_tag(
        kp_payload.view().children(),
        KmipTag::PrivateKeyAttributes.as_u32(),
    )
    .and_then(Item::as_structure)
    .and_then(|s| find_child_by_tag(s.children(), 0x0042_00BF))
    .is_some();
    let has_public_alt = find_child_by_tag(
        kp_payload.view().children(),
        KmipTag::PublicKeyAttributes.as_u32(),
    )
    .and_then(Item::as_structure)
    .and_then(|s| find_child_by_tag(s.children(), 0x0042_00BF))
    .is_some();
    assert!(has_private_alt && has_public_alt);

    // UniqueIdentifier normalization: `&str` -> TextString in 2.1, Identifier in 3.0
    let get_21 = client.request().get("id-21").build();
    assert_eq!(
        get_21.unique_identifier(),
        Some(&UniqueIdentifier::TextString("id-21".to_owned()))
    );
    let get_30 = client
        .request()
        .get("id-30")
        .with_protocol_version(ProtocolVersion::V3_0)
        .build();
    assert_eq!(
        get_30.unique_identifier(),
        Some(&UniqueIdentifier::Identifier("id-30".to_owned()))
    );

    // Register::link adapts Link across 2.1 (Link) vs 3.0 (v3_tag) using LinkNormalizer
    let register_21 = client
        .request()
        .register()
        .object(ManagedObject::OpaqueObject(
            OpaqueObject::try_new(1, b"opaque-bytes".to_vec()).expect("opaque object"),
        ))
        .name("reg-21")
        .link(LinkType::Certificate, "cert-21")
        .build_request()
        .expect("valid v2.1 Register request");
    let register_21_payload = register_21
        .into_ttlv_payload()
        .expect("v2.1 Register payload");
    let register_21_attrs = find_child_by_tag(
        register_21_payload.view().children(),
        KmipTag::Attributes.as_u32(),
    )
    .and_then(Item::as_structure)
    .expect("Attributes in v2.1 Register")
    .children();
    assert!(find_child_by_tag(register_21_attrs, KmipTag::Link.as_u32()).is_some());

    let register_30 = client
        .request()
        .register()
        .with_protocol_version(ProtocolVersion::V3_0)
        .object(ManagedObject::OpaqueObject(
            OpaqueObject::try_new(1, b"opaque-bytes".to_vec()).expect("opaque object"),
        ))
        .name("reg-30")
        .link(LinkType::Certificate, "cert-99")
        .build_request()
        .expect("valid v3.0 Register request");
    let register_30_payload = register_30
        .into_ttlv_payload()
        .expect("v3.0 Register payload");
    let register_30_attrs = find_child_by_tag(
        register_30_payload.view().children(),
        KmipTag::Attributes.as_u32(),
    )
    .and_then(Item::as_structure)
    .expect("Attributes in v3.0 Register")
    .children();
    assert!(find_child_by_tag(register_30_attrs, LinkType::Certificate.v3_tag()).is_some());

    // Client::send and set_protocol_version
    let ping_v30_ok = asynchronous_response_bytes_with_version(
        ProtocolVersion::V3_0,
        0x0000_003b,
        0,
        None,
        None,
        Some(test_structure([])),
    );
    let (mut client_send, _, _) = client_for(ExchangeScript::Success {
        response: ping_v30_ok,
        request_write_chunks: Vec::new(),
    });
    client_send.set_protocol_version(ProtocolVersion::V3_0);
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::ping()));
    let encoded_req = decode_request_message_for_test(batch, &limits).expect("encode batch");
    assert_eq!(
        encoded_req.header().protocol_version(),
        ProtocolVersion::V2_1
    );

    let pong = client_send
        .request()
        .ping()
        .execute()
        .expect("v3.0 ping succeeds");
    assert_eq!(pong.result().status(), ResultStatus::from_raw(0));
}

#[test]
fn discover_and_use_version_negotiates_highest_supported_version() {
    let discover_ok = response_bytes(
        (2, 1),
        &[ResponseItemFixture::success(None).with_supported_version((2, 1))],
    );
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: discover_ok,
        request_write_chunks: Vec::new(),
    });
    assert_eq!(client.protocol_version(), ProtocolVersion::V2_1);
    let chosen = client
        .discover_and_use_version(&[
            ProtocolVersion::V3_0,
            ProtocolVersion::V2_1,
            ProtocolVersion::V2_0,
        ])
        .expect("Discover Versions negotiation succeeds");
    assert_eq!(chosen, ProtocolVersion::V2_1);
    assert_eq!(client.protocol_version(), ProtocolVersion::V2_1);
}
