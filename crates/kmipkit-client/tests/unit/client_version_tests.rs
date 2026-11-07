//! Unit tests for multi-version client configuration, override, and VersionGate enforcement (KMIPKIT-0069).

use kmipkit_protocol::ProtocolVersion;
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::CodecLimits;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::decode_request_message_for_test;
use crate::extension_registry::{client_extension_registry, ClientConfiguration};
use crate::{ClientBatch, ClientBatchItem, ClientRequest};

#[test]
fn test_client_configuration_default_and_version_methods() {
    let registry = client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
        .expect("empty registry is valid");
    let config = ClientConfiguration::new(registry);

    // Default version is KMIP 2.1 (Zero-Breakage principle)
    assert_eq!(config.protocol_version(), ProtocolVersion::V2_1);

    // with_protocol_version changes the version
    let config_2_0 = config.with_protocol_version(ProtocolVersion::V2_0);
    assert_eq!(config_2_0.protocol_version(), ProtocolVersion::V2_0);

    // with_kmip_version alias works
    let config_3_0 = config_2_0.with_kmip_version(ProtocolVersion::V3_0);
    assert_eq!(config_3_0.protocol_version(), ProtocolVersion::V3_0);
}

#[test]
fn test_request_options_protocol_version_overrides() {
    let options = RequestOptions::default();
    assert_eq!(options.protocol_version(), None);

    let opt_2_0 = options.with_protocol_version(ProtocolVersion::V2_0);
    assert_eq!(opt_2_0.protocol_version(), Some(ProtocolVersion::V2_0));

    let opt_forced = options.force_kmip_version(ProtocolVersion::V3_0);
    assert_eq!(opt_forced.protocol_version(), Some(ProtocolVersion::V3_0));
}

#[test]
fn test_client_batch_protocol_version_overrides() {
    let batch = ClientBatch::empty();
    assert_eq!(batch.protocol_version(), None);

    let batch_2_0 = batch.with_protocol_version(ProtocolVersion::V2_0);
    assert_eq!(batch_2_0.protocol_version(), Some(ProtocolVersion::V2_0));

    let batch_forced = batch_2_0.force_kmip_version(ProtocolVersion::V3_0);
    assert_eq!(batch_forced.protocol_version(), Some(ProtocolVersion::V3_0));
}

#[test]
fn test_request_header_dynamic_protocol_version() {
    let limits = CodecLimits::defaults();

    // Default batch produces KMIP 2.1 in RequestHeader
    let batch_default = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
    let req_default = decode_request_message_for_test(batch_default, &limits)
        .expect("valid request message");
    assert_eq!(req_default.header().protocol_version(), ProtocolVersion::V2_1);

    // Batch configured with KMIP 2.0 produces KMIP 2.0 in RequestHeader
    let batch_2_0 = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
        .with_protocol_version(ProtocolVersion::V2_0);
    let req_2_0 = decode_request_message_for_test(batch_2_0, &limits)
        .expect("valid request message");
    assert_eq!(req_2_0.header().protocol_version(), ProtocolVersion::V2_0);

    // Batch configured with KMIP 3.0 produces KMIP 3.0 in RequestHeader
    let batch_3_0 = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
        .with_protocol_version(ProtocolVersion::V3_0);
    let req_3_0 = decode_request_message_for_test(batch_3_0, &limits)
        .expect("valid request message");
    assert_eq!(req_3_0.header().protocol_version(), ProtocolVersion::V3_0);
}

#[test]
fn test_version_gate_rejects_unsupported_operation_before_send() {
    // Ping was introduced in KMIP 2.1 (0x0000_003b). Attempting it under KMIP 2.0 must fail NotSent.
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);

    let opt_2_0 = RequestOptions::default().with_protocol_version(ProtocolVersion::V2_0);
    let err = client
        .ping_with_options(&CodecLimits::defaults(), &opt_2_0)
        .expect_err("Ping is not supported in KMIP 2.0");

    assert_eq!(err.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}

#[test]
fn test_version_gate_rejects_uncatalogued_version_before_send() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);

    // KMIP 1.4 is excluded (uncatalogued in KMIPKit client)
    let opt_1_4 = RequestOptions::default().with_protocol_version(ProtocolVersion::from_raw(1, 4));
    let err = client
        .ping_with_options(&CodecLimits::defaults(), &opt_1_4)
        .expect_err("KMIP 1.4 is not supported");

    assert_eq!(err.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}
