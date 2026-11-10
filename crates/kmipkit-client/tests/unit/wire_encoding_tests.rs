//! Derived KMIPKIT-0057 selection tests, not official OASIS vectors.
//!
//! OASIS KMIP Specification v2.1 §§10.1–10.2.3 and KMIP Profiles v2.1
//! §§5.4.1 and 5.5.1 define the selected formats. Selection precedence,
//! no fallback, borrowed limits, and cleanup are `KMIPKit` project contracts.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::Arc;

use kmipkit_protocol::ProtocolCauseCategory;
use kmipkit_transport::{
    RequestDeliveryState, TimeoutLimit, Transport, TransportCauseCategory, TransportError,
    TransportResponse,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{Item, ValueView};

use super::{Client, ClientTransport, LimitsIdentityObserver, ZeroizationObserver};
use crate::execute_test_support::{ResponseItemFixture, response_bytes};
use crate::extension_registry::{ClientConfiguration, client_extension_registry};
use crate::fake_transport_test_support::{FakeTransportObservation, OneShotFakeTransport};
use crate::{
    ClientBatch, ClientBatchItem, ClientCauseCategory, ClientRequest, RequestOptions, WireEncoding,
};

// Hand-authored semantic response: one successful Discover Versions result,
// deliberately reporting an empty supported-version list.
const JSON_RESPONSE: &[u8] = br#"{"tag":"ResponseMessage","value":[{"tag":"ResponseHeader","value":[{"tag":"ProtocolVersion","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":2},{"tag":"ProtocolVersionMinor","type":"Integer","value":1}]},{"tag":"TimeStamp","type":"DateTime","value":"1970-01-01T00:00:01Z"},{"tag":"BatchCount","type":"Integer","value":1}]},{"tag":"BatchItem","value":[{"tag":"Operation","type":"Enumeration","value":"DiscoverVersions"},{"tag":"ResultStatus","type":"Enumeration","value":"Success"},{"tag":"ResponsePayload","value":[]}]}]}"#;
const XML_RESPONSE: &[u8] = br#"<ResponseMessage><ResponseHeader><ProtocolVersion><ProtocolVersionMajor type="Integer" value="2"/><ProtocolVersionMinor type="Integer" value="1"/></ProtocolVersion><TimeStamp type="DateTime" value="1970-01-01T00:00:01Z"/><BatchCount type="Integer" value="1"/></ResponseHeader><BatchItem><Operation type="Enumeration" value="DiscoverVersions"/><ResultStatus type="Enumeration" value="Success"/><ResponsePayload/></BatchItem></ResponseMessage>"#;

fn configuration() -> ClientConfiguration {
    ClientConfiguration::new(
        client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
            .expect("empty registry is valid"),
    )
}

fn response(encoding: WireEncoding) -> Vec<u8> {
    match encoding {
        WireEncoding::Ttlv => response_bytes(
            (2, 1),
            &[ResponseItemFixture::success(None).with_empty_supported_versions()],
        ),
        WireEncoding::Json => JSON_RESPONSE.to_vec(),
        WireEncoding::Xml => XML_RESPONSE.to_vec(),
        _ => unreachable!("tests enumerate the supported encodings"),
    }
}

fn batch() -> ClientBatch {
    ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()))
}

#[test]
fn encoded_batch_item_operation_reader_rejects_malformed_headers_and_values() {
    let mut bytes = [0_u8; 20];
    bytes[8..16].copy_from_slice(&[0x42, 0x00, 0x5c, 0x05, 0x00, 0x00, 0x00, 0x04]);
    bytes[16..20].copy_from_slice(&7_u32.to_be_bytes());

    assert_eq!(
        super::decode_batch_item_operation(&bytes).expect("valid Enumeration is four bytes"),
        7
    );

    let mut unexpected_header = bytes;
    unexpected_header[11] = 0x04;
    assert!(super::decode_batch_item_operation(&unexpected_header).is_err());
    assert!(super::decode_batch_item_operation(&bytes[..16]).is_err());
    assert!(super::decode_operation_enumeration(&[0x00, 0x00, 0x07]).is_err());
}

fn decode_request(bytes: &[u8], encoding: WireEncoding, limits: &CodecLimits) -> Item {
    match encoding {
        WireEncoding::Ttlv => decode_with_limits(bytes, limits).expect("TTLV request decodes"),
        WireEncoding::Json => {
            crate::json_encoding::decode(bytes, limits).expect("JSON request decodes")
        }
        WireEncoding::Xml => crate::xml_codec::decode(bytes, limits).expect("XML request decodes"),
        _ => unreachable!("tests enumerate the supported encodings"),
    }
}

fn assert_request(
    observation: &FakeTransportObservation,
    encoding: WireEncoding,
    limits: &CodecLimits,
) {
    observation.with_captured_request(|bytes| {
        let bytes = bytes.expect("one request was captured");
        match encoding {
            WireEncoding::Ttlv => assert_eq!(&bytes[..4], &[0x42, 0x00, 0x78, 0x01]),
            WireEncoding::Json => assert_eq!(bytes.first(), Some(&b'{')),
            WireEncoding::Xml => assert_eq!(bytes.first(), Some(&b'<')),
            _ => unreachable!("tests enumerate the supported encodings"),
        }
        let decoded = decode_request(bytes, encoding, limits);
        decoded.with_value(|value| {
            let ValueView::Structure(tree) = value else {
                panic!("request is a Structure")
            };
            assert_eq!(tree.children().len(), 2);
            assert_eq!(tree.children()[0].tag().raw(), 0x0042_0077);
            assert_eq!(tree.children()[1].tag().raw(), 0x0042_000F);
        });
    });
}

fn observe_options(client: &mut Client) -> Rc<RefCell<Vec<RequestOptions>>> {
    let options = Rc::new(RefCell::new(Vec::new()));
    let ClientTransport::Test(transport) = &mut client.transport else {
        panic!("test adapter")
    };
    transport.observe_options(Rc::clone(&options));
    options
}

#[test]
fn default_configuration_keeps_ttlv_and_exposes_resolved_selection_to_transport() {
    let limits = CodecLimits::defaults();
    let config = configuration();
    assert_eq!(config.wire_encoding(), WireEncoding::Ttlv);
    let config = ClientConfiguration::from_shared_registry(config.shared_extension_registry());
    assert_eq!(RequestOptions::default().wire_encoding(), None);
    let (fake, observation) = OneShotFakeTransport::new(response(WireEncoding::Ttlv));
    let mut client = Client::for_test_with_configuration(fake, config);
    let options = observe_options(&mut client);

    let result = client
        .execute(batch(), &limits)
        .expect("legacy TTLV call succeeds");

    assert_eq!(result.len(), 1);
    assert_eq!(
        options.borrow()[0].wire_encoding(),
        Some(WireEncoding::Ttlv)
    );
    assert_eq!(observation.exchange_count(), 1);
    assert_request(&observation, WireEncoding::Ttlv, &limits);
}

#[test]
fn configured_json_and_xml_defaults_encode_and_decode_the_same_selected_format() {
    for encoding in [WireEncoding::Json, WireEncoding::Xml] {
        let limits = CodecLimits::defaults();
        let config = configuration();
        let registry = config.shared_extension_registry();
        let config = config.with_wire_encoding(encoding);
        assert!(Arc::ptr_eq(&registry, &config.shared_extension_registry()));
        assert_eq!(config.wire_encoding(), encoding);
        let (fake, observation) = OneShotFakeTransport::new(response(encoding));
        let mut client = Client::for_test_with_configuration(fake, config);
        let options = observe_options(&mut client);
        let cleanup = ZeroizationObserver::new(None);
        client.request_owner_observer = Some(cleanup.clone());
        let identity = LimitsIdentityObserver::new(&limits);
        client.limits_identity_observer = Some(identity.clone());

        let result = client
            .execute(batch(), &limits)
            .expect("selected text response decodes");

        assert_eq!(result.len(), 1);
        assert_eq!(options.borrow()[0].wire_encoding(), Some(encoding));
        assert!(identity.both_same());
        assert_eq!(cleanup.result(), Some(true));
        assert_eq!(observation.exchange_count(), 1);
        assert_request(&observation, encoding, &limits);
    }
}

#[test]
fn per_exchange_override_wins_without_mutating_client_or_caller_options() {
    for (configured, selected) in [
        (WireEncoding::Ttlv, WireEncoding::Json),
        (WireEncoding::Json, WireEncoding::Xml),
        (WireEncoding::Xml, WireEncoding::Ttlv),
    ] {
        let limits = CodecLimits::defaults();
        let (fake, observation) = OneShotFakeTransport::new(response(selected));
        let mut client = Client::for_test_with_configuration(
            fake,
            configuration().with_wire_encoding(configured),
        );
        let observed = observe_options(&mut client);
        let options = RequestOptions::default()
            .with_read(TimeoutLimit::Unbounded)
            .with_wire_encoding(selected);

        client
            .execute_with_options(batch(), &limits, &options)
            .expect("override exchange succeeds");

        assert_eq!(client.configuration.wire_encoding(), configured);
        assert_eq!(options.wire_encoding(), Some(selected));
        assert_eq!(observed.borrow()[0], options);
        assert_eq!(observation.exchange_count(), 1);
        assert_request(&observation, selected, &limits);

        let (fake, next) = OneShotFakeTransport::new(response(configured));
        client.transport =
            ClientTransport::Test(crate::execute_test_support::TestTransport::new(fake));
        client
            .execute(batch(), &limits)
            .expect("subsequent call uses persistent client default");
        assert_request(&next, configured, &limits);
        assert_eq!(next.exchange_count(), 1);
    }
}

#[test]
fn mismatched_response_formats_return_redacted_protocol_errors_without_retry_or_fallback() {
    for selected in [WireEncoding::Ttlv, WireEncoding::Json, WireEncoding::Xml] {
        for received in [WireEncoding::Ttlv, WireEncoding::Json, WireEncoding::Xml] {
            if selected == received {
                continue;
            }
            let (fake, observation) = OneShotFakeTransport::new(response(received));
            let mut client = Client::for_test_with_configuration(
                fake,
                configuration().with_wire_encoding(selected),
            );
            let error = client
                .execute(batch(), &CodecLimits::defaults())
                .expect_err("different response format is rejected");

            assert_eq!(
                error.delivery_state(),
                Some(RequestDeliveryState::ResponseStarted)
            );
            assert_eq!(
                error.cause_category(),
                Some(ClientCauseCategory::Protocol(
                    ProtocolCauseCategory::InvalidEncoding
                ))
            );
            assert_eq!(observation.exchange_count(), 1);
            for diagnostic in [error.to_string(), format!("{error:?}")] {
                assert!(!diagnostic.contains("ResponseHeader"));
                assert!(!diagnostic.contains("DiscoverVersions"));
            }
        }
    }
}

struct EncodingMismatchTransport;

impl Transport for EncodingMismatchTransport {
    fn exchange(
        &mut self,
        _request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        Err(TransportError::new(
            RequestDeliveryState::ResponseStarted,
            TransportCauseCategory::EncodingMismatch,
            io::Error::other("private response media type"),
        ))
    }
}

#[test]
fn transport_encoding_mismatch_becomes_a_redacted_protocol_error() {
    let mut client = Client::for_test_with_configuration(
        EncodingMismatchTransport,
        configuration().with_wire_encoding(WireEncoding::Json),
    );

    let error = client
        .execute(batch(), &CodecLimits::defaults())
        .expect_err("a response encoded differently from the request is rejected");

    assert_eq!(error.category(), crate::ClientErrorCategory::Protocol);
    assert_eq!(
        error.cause_category(),
        Some(ClientCauseCategory::Protocol(
            ProtocolCauseCategory::InvalidEncoding
        ))
    );
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::ResponseStarted)
    );
    assert!(!error.to_string().contains("private response media type"));
    assert!(!format!("{error:?}").contains("private response media type"));
}

#[test]
fn text_request_limit_failure_stays_not_sent_and_never_exchanges() {
    for encoding in [WireEncoding::Json, WireEncoding::Xml] {
        let limits = CodecLimits::new(200, 64, 100_000).expect("positive limits");
        let (fake, observation) = OneShotFakeTransport::new(response(encoding));
        let mut client =
            Client::for_test_with_configuration(fake, configuration().with_wire_encoding(encoding));

        let error = client
            .execute(batch(), &limits)
            .expect_err("text request exceeds byte cap");

        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert_eq!(observation.exchange_count(), 0);
    }
}

#[test]
fn outbound_text_codec_failure_preserves_its_payload_free_source() {
    use std::error::Error;

    use crate::wire_codec_common::{WireCodecError, WireCodecErrorKind};

    let cause = WireCodecError::new(WireCodecErrorKind::LimitExceeded);
    let error = super::private_wire_writer::EncodeError::WireCodec(cause);

    assert_eq!(
        error
            .source()
            .and_then(|source| source.downcast_ref::<WireCodecError>()),
        Some(&cause)
    );
    assert!(error.to_string().contains("LimitExceeded"));
    assert_eq!(format!("{error:?}"), "WireCodec");
}
