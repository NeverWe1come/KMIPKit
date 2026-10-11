//! Fake-transport key-transfer contracts from KMIP 2.1 §§6.1.14, 6.1.18,
//! 6.1.25, 6.1.27 and Tables 205–207, 217–219, 238–240, 244–246. These are
//! source-derived client tests, not claims of official profile conformance.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_protocol::{
    AttributeSet, DerivationMethod, DeriveKeyRequest, ExportRequest, ImportRequest,
    JoinSplitKeyRequest, ObjectType, SplitKeyReference, UniqueIdentifier,
};
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Structure, Value};

use super::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
use super::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};

const DERIVE_KEY: u32 = 0x0000_0005;
const EXPORT: u32 = 0x0000_002b;
const IMPORT: u32 = 0x0000_002a;
const JOIN_SPLIT_KEY: u32 = 0x0000_0029;
const RESPONSE_HEADER: u32 = 0x0042_007a;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006a;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006b;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000d;
const BATCH_ITEM: u32 = 0x0042_000f;
const OPERATION: u32 = 0x0042_005c;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const RESULT_STATUS: u32 = 0x0042_007f;
const RESULT_REASON: u32 = 0x0042_007e;
const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const RESPONSE_TOO_LARGE: u32 = 2;

const OPERATION_SPECS: [(u32, &[u8], &[u8]); 4] = [
    (DERIVE_KEY, b"derive-key", b"derive-key-correlation"),
    (EXPORT, b"export", b"export-correlation"),
    (IMPORT, b"import", b"import-correlation"),
    (JOIN_SPLIT_KEY, b"join-split-key", b"join-split-key-correlation"),
];

fn requests() -> Vec<ClientBatchItem> {
    let attributes = AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("symmetric-key output attributes use KMIP item types");
    let derive = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source-key".to_owned())],
        DerivationMethod::from_raw(1),
        Structure::new(),
        attributes,
    );
    let export = ExportRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("export-key".to_owned()));
    let secret_object = test_item(
        0x0042_0085,
        Value::structure(test_structure([
            test_item(0x0042_0086, Value::enumeration(1)),
            test_item(0x0042_0087, Value::byte_string(b"synthetic-secret".to_vec())),
        ])),
    );
    let import = ImportRequest::new(
        UniqueIdentifier::TextString("import-secret".to_owned()),
        ObjectType::from_raw(7),
        AttributeSet::new(),
        secret_object,
    );
    let join = JoinSplitKeyRequest::new(
        ObjectType::from_raw(2),
        vec![SplitKeyReference::new(
            UniqueIdentifier::TextString("split-key".to_owned()),
            1,
        )],
    );

    vec![
        ClientBatchItem::new(ClientRequest::DeriveKey(derive)).with_unique_batch_item_id(
            OPERATION_SPECS[0].1.to_vec(),
        ),
        ClientBatchItem::new(ClientRequest::Export(export))
            .with_unique_batch_item_id(OPERATION_SPECS[1].1.to_vec()),
        ClientBatchItem::new(ClientRequest::Import(import))
            .with_unique_batch_item_id(OPERATION_SPECS[2].1.to_vec()),
        ClientBatchItem::new(ClientRequest::JoinSplitKey(join))
            .with_unique_batch_item_id(OPERATION_SPECS[3].1.to_vec()),
    ]
}

fn batch_response(
    responses: impl IntoIterator<Item = (u32, &'static [u8], u32, Option<&'static [u8]>)>,
) -> Vec<u8> {
    let responses = responses.into_iter().collect::<Vec<_>>();
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(
            BATCH_COUNT,
            Value::integer(i32::try_from(responses.len()).expect("fixture count fits i32")),
        ),
    ]);
    let items = responses.into_iter().map(|(operation, id, status, correlation)| {
        let mut fields = vec![
            test_item(OPERATION, Value::enumeration(operation)),
            test_item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())),
            test_item(RESULT_STATUS, Value::enumeration(status)),
        ];
        if status == OPERATION_FAILED {
            fields.push(test_item(RESULT_REASON, Value::enumeration(RESPONSE_TOO_LARGE)));
        }
        if let Some(correlation) = correlation {
            fields.push(test_item(
                ASYNCHRONOUS_CORRELATION_VALUE,
                Value::byte_string(correlation.to_vec()),
            ));
        }
        test_item(BATCH_ITEM, Value::structure(test_structure(fields)))
    });
    let mut fields = vec![test_item(RESPONSE_HEADER, Value::structure(header))];
    fields.extend(items);
    encode_message_for_test(test_structure(fields), &CodecLimits::defaults())
        .expect("fixture response message is encodable")
}

fn client_for_script(script: ExchangeScript) -> (Client, Rc<RefCell<ScriptedTransport>>) {
    let fake = Rc::new(RefCell::new(ScriptedTransport::new(script)));
    let client = Client::for_test(SharedTransport(Rc::clone(&fake)));
    (client, fake)
}

#[derive(Clone)]
struct SharedTransport(Rc<RefCell<ScriptedTransport>>);

impl Transport for SharedTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.0.borrow_mut().exchange(request, max_response_bytes)
    }
}

#[test]
fn four_operations_preserve_codes_batch_order_pending_and_one_exchange() {
    let response = batch_response([
        (JOIN_SPLIT_KEY, OPERATION_SPECS[3].1, OPERATION_PENDING, Some(OPERATION_SPECS[3].2)),
        (IMPORT, OPERATION_SPECS[2].1, OPERATION_PENDING, Some(OPERATION_SPECS[2].2)),
        (EXPORT, OPERATION_SPECS[1].1, OPERATION_PENDING, Some(OPERATION_SPECS[1].2)),
        (DERIVE_KEY, OPERATION_SPECS[0].1, OPERATION_PENDING, Some(OPERATION_SPECS[0].2)),
    ]);
    let (mut client, fake) = client_for_script(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 11, 5],
    });
    let result = client
        .execute(
            ClientBatch::from_items(requests()).with_asynchronous_indicator(1),
            &CodecLimits::defaults(),
        )
        .expect("all four Pending outcomes associate by operation and batch ID");

    assert_eq!(result.len(), 4);
    for (index, (operation, id, correlation)) in OPERATION_SPECS.iter().enumerate() {
        let response = result.get(index).expect("request order is preserved");
        assert_eq!(response.unique_batch_item_id(), Some(*id));
        assert_eq!(response.outcome().asynchronous_correlation_value(), Some(*correlation));
        assert_eq!(response.outcome().result().status().raw(), OPERATION_PENDING);
        let ClientBatchOutcome::Pending(pending) = response.outcome() else {
            panic!("the server Pending result remains explicit");
        };
        let expected = match operation {
            &DERIVE_KEY => ClientOperation::DeriveKey,
            &EXPORT => ClientOperation::Export,
            &IMPORT => ClientOperation::Import,
            &JOIN_SPLIT_KEY => ClientOperation::JoinSplitKey,
            _ => unreachable!("the fixture contains only the four key-transfer operations"),
        };
        assert_eq!(pending.operation(), expected);
    }
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn each_operation_reports_failure_delivery_state_without_retry() {
    for (index, operation) in OPERATION_SPECS.iter().copied().enumerate() {
        let cases = [
            (ExchangeScript::FailBeforeWrite, RequestDeliveryState::NotSent),
            (
                ExchangeScript::FailAfterPartialWrite { written_bytes: 2 },
                RequestDeliveryState::PossiblySent,
            ),
        ];
        for (script, expected_delivery) in cases {
            let item = requests()
                .into_iter()
                .nth(index)
                .expect("operation fixture exists at the same index");
            let (mut client, fake) = client_for_script(script);
            let error = client
                .execute(
                    ClientBatch::new(item),
                    &CodecLimits::defaults(),
                )
                .expect_err("transport failures are returned without retry");
            assert_eq!(
                error.delivery_state(),
                Some(expected_delivery),
                "op={operation:?}"
            );
            assert_eq!(fake.borrow().exchange_count(), 1, "op={operation:?}");
        }
    }
}
