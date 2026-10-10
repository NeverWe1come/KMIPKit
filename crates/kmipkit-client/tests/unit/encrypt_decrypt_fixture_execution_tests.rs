//! Fixture-derived client execution contracts from OASIS KMIP Specification
//! v2.1 §§6.1.11 Tables 196–197 and 6.1.17 Tables 214–215, plus OASIS Test
//! Cases v2.1 CN01 §§2.99–2.101. Traceability: KMIPKIT-0019-FR-001,
//! KMIPKIT-0019-FR-008, KMIPKIT-0019-FR-012,
//! KMIPKIT-REQ-SPEC-6.1-001-001, KMIPKIT-REQ-SPEC-6.1-001-002, and
//! KMIPKIT-TEST-CN01-2-99 / -2-100 / -2-101. These are fixture-derived
//! operation-item checks, not complete official Test Case executions.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use kmipkit_protocol::{
    DecryptRequest, EncryptRequest, OperationData, RequestMessage, ResponseMessage, SecretBytes,
    UniqueIdentifier,
};
use kmipkit_test_support::oasis_crypto_fixtures::{
    OasisCryptoFixture, OasisCryptoOperation, OasisCryptoOperationPair,
};
use kmipkit_transport::{
    RequestDeliveryState, Transport, TransportCauseCategory, TransportError, TransportResponse,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Structure, StructureView, Value, ValueView};
use zeroize::Zeroizing;

use crate::execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchResponse, ClientRequest,
    encode_message_for_test,
};
use crate::execute_test_support::test_item;
use crate::{ClientBatchOutcome, ClientErrorCategory};

const TC_ENC_1_21_ID: &str = "TC-STREAM-ENC-1-21";
const TC_ENC_2_21_ID: &str = "TC-STREAM-ENC-2-21";
const TC_ENCDEC_1_21_ID: &str = "TC-STREAM-ENCDEC-1-21";

const TC_ENC_1_21_XML: &str =
    include_str!("../../../../specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENC-1-21.xml");
const TC_ENC_2_21_XML: &str =
    include_str!("../../../../specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENC-2-21.xml");
const TC_ENCDEC_1_21_XML: &str =
    include_str!("../../../../specification/oasis/kmip-2.1/fixtures/TC-STREAM-ENCDEC-1-21.xml");

const REQUEST_HEADER: u32 = 0x0042_0077;
const BATCH_COUNT: u32 = 0x0042_000D;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
const DATA: u32 = 0x0042_00C2;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const CORRELATION_VALUE: u32 = 0x0042_00D6;
const INIT_INDICATOR: u32 = 0x0042_00D7;
const FINAL_INDICATOR: u32 = 0x0042_00D8;
const AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA: u32 = 0x0042_00FE;
const AUTHENTICATED_ENCRYPTION_TAG: u32 = 0x0042_00FF;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;

const ENCRYPT_OPERATION: u32 = 0x0000_001F;
const DECRYPT_OPERATION: u32 = 0x0000_0020;
const SUCCESS: u32 = 0;

const STEPS_1_TO_10: [usize; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
const STEPS_1_TO_8: [usize; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
const ENCRYPT_ONLY: [OasisCryptoOperation; 10] = [OasisCryptoOperation::Encrypt; 10];
const ENCRYPT_DECRYPT_MIXED: [OasisCryptoOperation; 8] = [
    OasisCryptoOperation::Encrypt,
    OasisCryptoOperation::Decrypt,
    OasisCryptoOperation::Encrypt,
    OasisCryptoOperation::Encrypt,
    OasisCryptoOperation::Encrypt,
    OasisCryptoOperation::Decrypt,
    OasisCryptoOperation::Decrypt,
    OasisCryptoOperation::Decrypt,
];

struct FixtureCase {
    case_id: &'static str,
    xml: &'static str,
    expected_sequences: &'static [usize],
    expected_operations: &'static [OasisCryptoOperation],
}

const FIXTURE_CASES: [FixtureCase; 3] = [
    FixtureCase {
        case_id: TC_ENC_1_21_ID,
        xml: TC_ENC_1_21_XML,
        expected_sequences: &STEPS_1_TO_10,
        expected_operations: &ENCRYPT_ONLY,
    },
    FixtureCase {
        case_id: TC_ENC_2_21_ID,
        xml: TC_ENC_2_21_XML,
        expected_sequences: &STEPS_1_TO_10,
        expected_operations: &ENCRYPT_ONLY,
    },
    FixtureCase {
        case_id: TC_ENCDEC_1_21_ID,
        xml: TC_ENCDEC_1_21_XML,
        expected_sequences: &STEPS_1_TO_8,
        expected_operations: &ENCRYPT_DECRYPT_MIXED,
    },
];

enum FixtureReply {
    Response(Zeroizing<Vec<u8>>),
    Failure(RequestDeliveryState),
}

struct FixtureTransportState {
    reply: Option<FixtureReply>,
    requests: Vec<Zeroizing<Vec<u8>>>,
}

type SharedFixtureTransport = Rc<RefCell<FixtureTransportState>>;

struct OneExchangeTransport {
    state: SharedFixtureTransport,
}

impl Transport for OneExchangeTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        let mut state = self.state.borrow_mut();
        state.requests.push(Zeroizing::new(request.to_vec()));
        let Some(reply) = state.reply.take() else {
            return Err(TransportError::new(
                RequestDeliveryState::NotSent,
                TransportCauseCategory::Other,
                io::Error::other("unexpected additional fixture exchange"),
            ));
        };
        match reply {
            FixtureReply::Response(response) if response.len() <= max_response_bytes => {
                Ok(TransportResponse::new(response.to_vec()))
            }
            FixtureReply::Response(_) => Err(TransportError::new(
                RequestDeliveryState::ResponseStarted,
                TransportCauseCategory::Other,
                io::Error::other("fixture response exceeds the requested limit"),
            )),
            FixtureReply::Failure(delivery_state) => Err(TransportError::new(
                delivery_state,
                TransportCauseCategory::Io,
                io::Error::other("fixture transport failure"),
            )),
        }
    }
}

#[derive(Default)]
struct FixtureRequestFields {
    unique_identifier: Option<UniqueIdentifier>,
    cryptographic_parameters: Option<Structure>,
    data: Option<OperationData>,
    iv_counter_nonce: Option<SecretBytes>,
    correlation_value: Option<SecretBytes>,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
    authenticated_encryption_additional_data: Option<SecretBytes>,
    authenticated_encryption_tag: Option<SecretBytes>,
}

#[derive(Default)]
struct FixtureResponseFields {
    unique_identifier: Option<UniqueIdentifier>,
    data: Option<Zeroizing<Vec<u8>>>,
    iv_counter_nonce: Option<Zeroizing<Vec<u8>>>,
    correlation_value: Option<Zeroizing<Vec<u8>>>,
    authenticated_encryption_tag: Option<Zeroizing<Vec<u8>>>,
    result_status: Option<u32>,
}

fn client_with_reply(reply: FixtureReply) -> (Client, SharedFixtureTransport) {
    let state = Rc::new(RefCell::new(FixtureTransportState {
        reply: Some(reply),
        requests: Vec::new(),
    }));
    let client = Client::for_test(OneExchangeTransport {
        state: Rc::clone(&state),
    });
    (client, state)
}

#[test]
fn fixture_derived_encrypt_decrypt_pairs_execute_once_with_exact_wire_and_paired_response() {
    let mut pair_count = 0;
    let mut encrypt_count = 0;
    let mut decrypt_count = 0;
    let mut failures = Vec::new();

    for fixture_case in FIXTURE_CASES {
        let fixture = OasisCryptoFixture::from_xml(fixture_case.case_id, fixture_case.xml)
            .expect("the pinned OASIS fixture parses its selected operation pairs");
        let pairs = fixture.operation_pairs();
        assert_eq!(pairs.len(), fixture_case.expected_sequences.len());
        assert_eq!(pairs.len(), fixture_case.expected_operations.len());

        for (index, pair) in pairs.iter().enumerate() {
            let expected_sequence = fixture_case.expected_sequences[index];
            let expected_operation = fixture_case.expected_operations[index];
            assert_eq!(pair.case_id(), fixture_case.case_id);
            assert_eq!(pair.source_sequence(), expected_sequence);
            assert_eq!(pair.operation(), expected_operation);
            assert_eq!(
                pair.step_identity(),
                format!("{} step={expected_sequence}", fixture_case.case_id)
            );

            match expected_operation {
                OasisCryptoOperation::Encrypt => encrypt_count += 1,
                OasisCryptoOperation::Decrypt => decrypt_count += 1,
            }

            let expected_request = expected_client_request_bytes(pair);
            let response_bytes =
                encode_message_for_test(pair.response_message().clone(), &CodecLimits::defaults())
                    .expect("the paired fixture response is encodable");
            let (mut client, state) =
                client_with_reply(FixtureReply::Response(Zeroizing::new(response_bytes)));
            let request = client_request_from_fixture(pair);
            let batch = ClientBatch::new(ClientBatchItem::new(request))
                .with_client_correlation_value(pair.step_identity().to_owned());
            let result = client.execute(batch, &CodecLimits::defaults());

            let captured_request_matches = {
                let state = state.borrow();
                if state.requests.len() != 1 {
                    failures.push(format!(
                        "{}: expected one exchange, observed {}",
                        pair.step_identity(),
                        state.requests.len()
                    ));
                }
                state
                    .requests
                    .first()
                    .is_some_and(|request| request.as_slice() == expected_request.as_slice())
            };
            if !captured_request_matches {
                failures.push(format!(
                    "{}: encoded request differs from the source fixture item",
                    pair.step_identity()
                ));
            }

            match result {
                Ok(response) => {
                    if !typed_response_matches_fixture(pair, &response) {
                        failures.push(format!(
                            "{}: typed outcome differs from its paired fixture response",
                            pair.step_identity()
                        ));
                    }
                }
                Err(error) => failures.push(format!(
                    "{}: client returned {:?} instead of the paired result",
                    pair.step_identity(),
                    error.category()
                )),
            }
            pair_count += 1;
        }
    }

    assert_eq!(pair_count, 28, "all in-scope operation pairs execute");
    assert_eq!(encrypt_count, 24);
    assert_eq!(decrypt_count, 4);
    assert!(
        failures.is_empty(),
        "fixture operation-item failures: {failures:?}"
    );
}

#[test]
fn encrypt_transport_failure_preserves_delivery_state_without_retry() {
    assert_transport_failure_preserves_delivery_state(ClientRequest::Encrypt(EncryptRequest::new(
        Some(UniqueIdentifier::TextString("delivery-test-key".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            b"delivery-test-data".to_vec(),
        ))),
    )));
}

#[test]
fn decrypt_transport_failure_preserves_delivery_state_without_retry() {
    assert_transport_failure_preserves_delivery_state(ClientRequest::Decrypt(DecryptRequest::new(
        Some(UniqueIdentifier::TextString("delivery-test-key".to_owned())),
        Some(OperationData::ByteString(SecretBytes::new(
            b"delivery-test-data".to_vec(),
        ))),
    )));
}

fn assert_transport_failure_preserves_delivery_state(request: ClientRequest) {
    let (mut client, state) =
        client_with_reply(FixtureReply::Failure(RequestDeliveryState::PossiblySent));
    let error = client
        .execute(
            ClientBatch::new(ClientBatchItem::new(request)),
            &CodecLimits::defaults(),
        )
        .expect_err("the fake transport returns its configured failure");

    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(
        state.borrow().requests.len(),
        1,
        "the failed call is not retried"
    );
}

fn client_request_from_fixture(pair: &OasisCryptoOperationPair) -> ClientRequest {
    let message = RequestMessage::try_from_ttlv(pair.request_message().clone())
        .expect("the fixture request is a valid one-item KMIP message");
    let mut batch_items = message.batch_items();
    let batch_item = batch_items
        .next()
        .expect("the fixture request contains its selected operation item");
    assert!(batch_items.next().is_none());
    assert_eq!(
        batch_item.operation(),
        Some(operation_value(pair.operation()))
    );
    let fields = batch_item
        .with_request_payload(parse_request_fields)
        .expect("the fixture operation has a Request Payload");

    match pair.operation() {
        OasisCryptoOperation::Encrypt => {
            let mut request = EncryptRequest::new(fields.unique_identifier, fields.data);
            if let Some(parameters) = fields.cryptographic_parameters {
                request = request.with_cryptographic_parameters(parameters);
            }
            if let Some(value) = fields.iv_counter_nonce {
                request = request.with_iv_counter_nonce(value);
            }
            if let Some(value) = fields.correlation_value {
                request = request.with_correlation_value(value);
            }
            if let Some(value) = fields.init_indicator {
                request = request.with_init_indicator(value);
            }
            if let Some(value) = fields.final_indicator {
                request = request.with_final_indicator(value);
            }
            if let Some(value) = fields.authenticated_encryption_additional_data {
                request = request.with_authenticated_encryption_additional_data(value);
            }
            ClientRequest::Encrypt(request)
        }
        OasisCryptoOperation::Decrypt => {
            let mut request = DecryptRequest::new(fields.unique_identifier, fields.data);
            if let Some(parameters) = fields.cryptographic_parameters {
                request = request.with_cryptographic_parameters(parameters);
            }
            if let Some(value) = fields.iv_counter_nonce {
                request = request.with_iv_counter_nonce(value);
            }
            if let Some(value) = fields.correlation_value {
                request = request.with_correlation_value(value);
            }
            if let Some(value) = fields.init_indicator {
                request = request.with_init_indicator(value);
            }
            if let Some(value) = fields.final_indicator {
                request = request.with_final_indicator(value);
            }
            if let Some(value) = fields.authenticated_encryption_additional_data {
                request = request.with_authenticated_encryption_additional_data(value);
            }
            if let Some(value) = fields.authenticated_encryption_tag {
                request = request.with_authenticated_encryption_tag(value);
            }
            ClientRequest::Decrypt(request)
        }
    }
}

fn parse_request_fields(payload: StructureView<'_>) -> FixtureRequestFields {
    let mut fields = FixtureRequestFields::default();
    for item in payload.children() {
        match item.tag().raw() {
            UNIQUE_IDENTIFIER => {
                fields.unique_identifier = Some(item.with_value(parse_unique_identifier));
            }
            CRYPTOGRAPHIC_PARAMETERS => {
                fields.cryptographic_parameters = Some(item.with_value(|value| match value {
                    ValueView::Structure(value) => clone_fixture_structure(&value),
                    _ => panic!("fixture Cryptographic Parameters are a Structure"),
                }));
            }
            DATA => fields.data = Some(item.with_value(parse_operation_data)),
            IV_COUNTER_NONCE => {
                fields.iv_counter_nonce = Some(item.with_value(parse_secret_bytes));
            }
            CORRELATION_VALUE => {
                fields.correlation_value = Some(item.with_value(parse_secret_bytes));
            }
            INIT_INDICATOR => fields.init_indicator = Some(item.with_value(parse_boolean)),
            FINAL_INDICATOR => fields.final_indicator = Some(item.with_value(parse_boolean)),
            AUTHENTICATED_ENCRYPTION_ADDITIONAL_DATA => {
                fields.authenticated_encryption_additional_data =
                    Some(item.with_value(parse_secret_bytes));
            }
            AUTHENTICATED_ENCRYPTION_TAG => {
                fields.authenticated_encryption_tag = Some(item.with_value(parse_secret_bytes));
            }
            _ => panic!("fixture request contains only the operation table fields"),
        }
    }
    fields
}

fn parse_unique_identifier(value: ValueView<'_>) -> UniqueIdentifier {
    match value {
        ValueView::TextString(value) => UniqueIdentifier::TextString(value.to_owned()),
        ValueView::Enumeration(value) => UniqueIdentifier::Enumeration(*value),
        ValueView::Integer(value) => UniqueIdentifier::Integer(*value),
        _ => panic!("fixture Unique Identifier has a supported wire type"),
    }
}

fn parse_operation_data(value: ValueView<'_>) -> OperationData {
    match value {
        ValueView::ByteString(value) => OperationData::ByteString(SecretBytes::new(value.to_vec())),
        ValueView::Enumeration(value) => OperationData::Enumeration(*value),
        ValueView::Integer(value) => OperationData::Integer(*value),
        _ => panic!("fixture Data uses one of the §7.9 wire types"),
    }
}

fn parse_secret_bytes(value: ValueView<'_>) -> SecretBytes {
    match value {
        ValueView::ByteString(value) => SecretBytes::new(value.to_vec()),
        _ => panic!("fixture operation byte field is a Byte String"),
    }
}

fn parse_boolean(value: ValueView<'_>) -> bool {
    match value {
        ValueView::Boolean(value) => *value,
        _ => panic!("fixture multipart indicator is a Boolean"),
    }
}

fn expected_client_request_bytes(pair: &OasisCryptoOperationPair) -> Zeroizing<Vec<u8>> {
    let mut expected_message = Structure::new();
    for root_item in pair.request_message().view().children() {
        let value = root_item.with_value(|value| match value {
            ValueView::Structure(source) if root_item.tag().raw() == REQUEST_HEADER => {
                Value::structure(request_header_with_client_defaults(&source))
            }
            _ => clone_fixture_value(value),
        });
        expected_message
            .try_push(test_item(root_item.tag().raw(), value))
            .expect("the expected fixture request retains its valid message structure");
    }
    Zeroizing::new(
        encode_message_for_test(expected_message, &CodecLimits::defaults())
            .expect("the expected fixture request is valid TTLV"),
    )
}

fn request_header_with_client_defaults(source: &StructureView<'_>) -> Structure {
    let mut header = Structure::new();
    for item in source.children() {
        if item.tag().raw() == BATCH_COUNT {
            header
                .try_push(test_item(
                    ATTESTATION_CAPABLE_INDICATOR,
                    Value::boolean(true),
                ))
                .expect("the client default fits before Request Header Batch Count");
        }
        let value = item.with_value(clone_fixture_value);
        header
            .try_push(test_item(item.tag().raw(), value))
            .expect("the normalized fixture Request Header remains valid");
    }
    header
}

fn clone_fixture_structure(source: &StructureView<'_>) -> Structure {
    let mut structure = Structure::new();
    for item in source.children() {
        let value = item.with_value(clone_fixture_value);
        structure
            .try_push(test_item(item.tag().raw(), value))
            .expect("the fixture Structure retains its valid depth");
    }
    structure
}

fn clone_fixture_value(value: ValueView<'_>) -> Value {
    match value {
        ValueView::Structure(value) => Value::structure(clone_fixture_structure(&value)),
        ValueView::Integer(value) => Value::integer(*value),
        ValueView::LongInteger(value) => Value::long_integer(*value),
        ValueView::BigInteger(value) => Value::big_integer(value.to_vec()),
        ValueView::Enumeration(value) => Value::enumeration(*value),
        ValueView::Boolean(value) => Value::boolean(*value),
        ValueView::TextString(value) => Value::text_string((*value).to_owned()),
        ValueView::ByteString(value) => Value::byte_string(value.to_vec()),
        ValueView::DateTime(value) => Value::date_time(*value),
        ValueView::Interval(value) => Value::interval(*value),
        ValueView::DateTimeExtended(value) => Value::date_time_extended(*value),
    }
}

fn typed_response_matches_fixture(
    pair: &OasisCryptoOperationPair,
    actual: &ClientBatchResponse,
) -> bool {
    if actual.len() != 1 {
        return false;
    }
    let expected = expected_response_fields(pair);
    let Some(item) = actual.get(0) else {
        return false;
    };
    match (pair.operation(), item.outcome()) {
        (OasisCryptoOperation::Encrypt, ClientBatchOutcome::Encrypt(response)) => {
            response.result().status().raw() == expected.result_status.unwrap_or(u32::MAX)
                && response.unique_identifier() == expected.unique_identifier.as_ref()
                && secret_matches(response.data(), expected.data.as_ref())
                && secret_matches(
                    response.iv_counter_nonce(),
                    expected.iv_counter_nonce.as_ref(),
                )
                && secret_matches(
                    response.correlation_value(),
                    expected.correlation_value.as_ref(),
                )
                && secret_matches(
                    response.authenticated_encryption_tag(),
                    expected.authenticated_encryption_tag.as_ref(),
                )
        }
        (OasisCryptoOperation::Decrypt, ClientBatchOutcome::Decrypt(response)) => {
            response.result().status().raw() == expected.result_status.unwrap_or(u32::MAX)
                && response.unique_identifier() == expected.unique_identifier.as_ref()
                && secret_matches(response.data(), expected.data.as_ref())
                && secret_matches(
                    response.correlation_value(),
                    expected.correlation_value.as_ref(),
                )
        }
        _ => false,
    }
}

fn expected_response_fields(pair: &OasisCryptoOperationPair) -> FixtureResponseFields {
    let message = ResponseMessage::try_from_ttlv(pair.response_message().clone())
        .expect("the fixture response is a valid one-item KMIP message");
    let mut batch_items = message.batch_items();
    let item = batch_items
        .next()
        .expect("the fixture response has its paired operation item");
    assert!(batch_items.next().is_none());
    assert_eq!(item.operation(), Some(operation_value(pair.operation())));
    let mut fields = FixtureResponseFields {
        result_status: item.result_status().map(|status| status.raw()),
        ..FixtureResponseFields::default()
    };
    if let Some(payload) = item.with_response_payload(parse_response_fields) {
        fields.unique_identifier = payload.unique_identifier;
        fields.data = payload.data;
        fields.iv_counter_nonce = payload.iv_counter_nonce;
        fields.correlation_value = payload.correlation_value;
        fields.authenticated_encryption_tag = payload.authenticated_encryption_tag;
    }
    fields
}

fn parse_response_fields(payload: StructureView<'_>) -> FixtureResponseFields {
    let mut fields = FixtureResponseFields::default();
    for item in payload.children() {
        match item.tag().raw() {
            UNIQUE_IDENTIFIER => {
                fields.unique_identifier = Some(item.with_value(parse_unique_identifier));
            }
            DATA => fields.data = Some(item.with_value(parse_response_secret_bytes)),
            IV_COUNTER_NONCE => {
                fields.iv_counter_nonce = Some(item.with_value(parse_response_secret_bytes));
            }
            CORRELATION_VALUE => {
                fields.correlation_value = Some(item.with_value(parse_response_secret_bytes));
            }
            AUTHENTICATED_ENCRYPTION_TAG => {
                fields.authenticated_encryption_tag =
                    Some(item.with_value(parse_response_secret_bytes));
            }
            _ => panic!("fixture response contains only Table 197/215 fields"),
        }
    }
    fields
}

fn parse_response_secret_bytes(value: ValueView<'_>) -> Zeroizing<Vec<u8>> {
    match value {
        ValueView::ByteString(value) => Zeroizing::new(value.to_vec()),
        _ => panic!("fixture response byte field is a Byte String"),
    }
}

fn secret_matches(actual: Option<&SecretBytes>, expected: Option<&Zeroizing<Vec<u8>>>) -> bool {
    match (actual, expected) {
        (None, None) => true,
        (Some(actual), Some(expected)) => actual.with_bytes(|actual| actual == expected.as_slice()),
        _ => false,
    }
}

fn operation_value(operation: OasisCryptoOperation) -> u32 {
    match operation {
        OasisCryptoOperation::Encrypt => ENCRYPT_OPERATION,
        OasisCryptoOperation::Decrypt => DECRYPT_OPERATION,
    }
}
