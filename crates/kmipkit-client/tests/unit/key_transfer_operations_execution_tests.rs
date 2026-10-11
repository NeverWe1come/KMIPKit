//! Fake-transport key-transfer contracts from KMIP 2.1 §§6.1.14, 6.1.18,
//! 6.1.25, 6.1.27 and Tables 205–207, 217–219, 238–240, 244–246. These are
//! source-derived client tests, not claims of official profile conformance.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_protocol::{
    AttributeSet, DerivationMethod, DerivationParameters, DeriveKeyRequest, ExportRequest,
    ImportRequest, JoinSplitKeyRequest, ObjectType, SecretBytes, SplitKeyReference,
    UniqueIdentifier,
};
use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{RequestDeliveryState, Transport, TransportError, TransportResponse};
use kmipkit_ttlv::Value;
use kmipkit_ttlv::codec::CodecLimits;

use super::encode_message_for_test;
use super::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientOperation, ClientRequest,
};
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
const RESPONSE_PAYLOAD: u32 = 0x0042_007c;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002a;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const OPERATION_SUCCESS: u32 = 0;
const RESPONSE_TOO_LARGE: u32 = 2;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const KEY_OBJECT: u32 = 0x0042_008f;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_MATERIAL: u32 = 0x0042_0043;

const OPERATION_SPECS: [(u32, &[u8], &[u8]); 4] = [
    (DERIVE_KEY, b"derive-key", b"derive-key-correlation"),
    (EXPORT, b"export", b"export-correlation"),
    (IMPORT, b"import", b"import-correlation"),
    (
        JOIN_SPLIT_KEY,
        b"join-split-key",
        b"join-split-key-correlation",
    ),
];

fn attributes() -> AttributeSet {
    AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
    ])
    .expect("symmetric-key attributes use KMIP item types")
}

fn requests() -> Vec<ClientBatchItem> {
    let derive = DeriveKeyRequest::new(
        ObjectType::from_raw(2),
        vec![UniqueIdentifier::TextString("source-key".to_owned())],
        DerivationMethod::from_raw(1),
        DerivationParameters::new()
            .with_salt(SecretBytes::new(b"synthetic-salt".to_vec()))
            .with_iteration_count(10_000),
        attributes(),
    );
    let export = ExportRequest::new()
        .with_unique_identifier(UniqueIdentifier::TextString("export-key".to_owned()));
    let key_value = test_structure([test_item(
        0x0042_0043,
        Value::byte_string(b"synthetic-import-key".to_vec()),
    )]);
    let key_block = test_structure([
        test_item(0x0042_0042, Value::enumeration(1)),
        test_item(0x0042_0045, Value::structure(key_value)),
    ]);
    let import_key_object = test_item(
        0x0042_008F,
        Value::structure(test_structure([test_item(
            0x0042_0040,
            Value::structure(key_block),
        )])),
    );
    let import = ImportRequest::new(
        UniqueIdentifier::TextString("import-key".to_owned()),
        ObjectType::from_raw(2),
        attributes(),
        import_key_object,
    );
    let join = JoinSplitKeyRequest::new(
        ObjectType::from_raw(2),
        vec![
            SplitKeyReference::new(UniqueIdentifier::TextString("split-key".to_owned()))
                .with_split_key_threshold(1),
        ],
    );

    vec![
        ClientBatchItem::new(ClientRequest::derive_key(derive))
            .with_unique_batch_item_id(OPERATION_SPECS[0].1.to_vec()),
        ClientBatchItem::new(ClientRequest::export(export))
            .with_unique_batch_item_id(OPERATION_SPECS[1].1.to_vec()),
        ClientBatchItem::new(ClientRequest::import(import))
            .with_unique_batch_item_id(OPERATION_SPECS[2].1.to_vec()),
        ClientBatchItem::new(ClientRequest::join_split_key(join))
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
    let items = responses
        .into_iter()
        .map(|(operation, id, status, correlation)| {
            let mut fields = vec![
                test_item(OPERATION, Value::enumeration(operation)),
                test_item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(id.to_vec())),
                test_item(RESULT_STATUS, Value::enumeration(status)),
            ];
            if status == OPERATION_FAILED {
                fields.push(test_item(
                    RESULT_REASON,
                    Value::enumeration(RESPONSE_TOO_LARGE),
                ));
            }
            if let Some(correlation) = correlation {
                fields.push(test_item(
                    ASYNCHRONOUS_CORRELATION_VALUE,
                    Value::byte_string(correlation.to_vec()),
                ));
            }
            if status == OPERATION_PENDING {
                fields.push(test_item(
                    RESPONSE_PAYLOAD,
                    Value::structure(test_structure([])),
                ));
            } else if status == OPERATION_SUCCESS {
                let mut payload = vec![test_item(
                    UNIQUE_IDENTIFIER,
                    Value::text_string(format!("returned-{operation}")),
                )];
                if operation == EXPORT {
                    let key_object = test_item(
                        KEY_OBJECT,
                        Value::structure(test_structure([test_item(
                            KEY_BLOCK,
                            Value::structure(test_structure([
                                test_item(KEY_FORMAT_TYPE, Value::enumeration(1)),
                                test_item(
                                    KEY_VALUE,
                                    Value::structure(test_structure([test_item(
                                        KEY_MATERIAL,
                                        Value::byte_string(b"synthetic-export-key".to_vec()),
                                    )])),
                                ),
                            ])),
                        )])),
                    );
                    payload.extend([
                        test_item(OBJECT_TYPE, Value::enumeration(2)),
                        test_item(ATTRIBUTES, Value::structure(test_structure([]))),
                        key_object,
                    ]);
                }
                fields.push(test_item(
                    RESPONSE_PAYLOAD,
                    Value::structure(test_structure(payload)),
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

#[test]
fn request_variants_and_debug_views_cover_all_four_operations_without_secret_bytes() {
    let items = requests();
    let expected = ["DeriveKey", "Export", "Import", "JoinSplitKey"];
    for (index, item) in items.iter().enumerate() {
        assert!(format!("{:?}", item.request()).contains(expected[index]));
    }
    let import_debug = format!("{:?}", items[2].request());
    assert!(!import_debug.contains("synthetic-import-key"));
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
        (
            JOIN_SPLIT_KEY,
            OPERATION_SPECS[3].1,
            OPERATION_PENDING,
            Some(OPERATION_SPECS[3].2),
        ),
        (
            IMPORT,
            OPERATION_SPECS[2].1,
            OPERATION_PENDING,
            Some(OPERATION_SPECS[2].2),
        ),
        (
            EXPORT,
            OPERATION_SPECS[1].1,
            OPERATION_PENDING,
            Some(OPERATION_SPECS[1].2),
        ),
        (
            DERIVE_KEY,
            OPERATION_SPECS[0].1,
            OPERATION_PENDING,
            Some(OPERATION_SPECS[0].2),
        ),
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
        assert_eq!(
            response.outcome().asynchronous_correlation_value(),
            Some(*correlation)
        );
        assert_eq!(
            response.outcome().result().status().raw(),
            OPERATION_PENDING
        );
        let ClientBatchOutcome::Pending(pending) = response.outcome() else {
            panic!("the server Pending result remains explicit");
        };
        let expected = match *operation {
            DERIVE_KEY => ClientOperation::DeriveKey,
            EXPORT => ClientOperation::Export,
            IMPORT => ClientOperation::Import,
            JOIN_SPLIT_KEY => ClientOperation::JoinSplitKey,
            _ => unreachable!("the fixture contains only the four key-transfer operations"),
        };
        assert_eq!(pending.operation(), expected);
        let response_view = pending.response();
        assert_eq!(
            response_view.derive_key().is_some(),
            *operation == DERIVE_KEY
        );
        assert_eq!(response_view.export().is_some(), *operation == EXPORT);
        assert_eq!(response_view.import().is_some(), *operation == IMPORT);
        assert_eq!(
            response_view.join_split_key().is_some(),
            *operation == JOIN_SPLIT_KEY
        );
        let outcome = response.outcome();
        assert_eq!(
            outcome.derive_key_response().is_some(),
            *operation == DERIVE_KEY
        );
        assert_eq!(outcome.export_response().is_some(), *operation == EXPORT);
        assert_eq!(outcome.import_response().is_some(), *operation == IMPORT);
        assert_eq!(
            outcome.join_split_key_response().is_some(),
            *operation == JOIN_SPLIT_KEY
        );
        let _ = format!("{response_view:?}");
        assert_eq!(response_view.result().status().raw(), OPERATION_PENDING);
    }
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn successful_results_expose_each_typed_view_accessor_and_format_safely() {
    let response = batch_response(
        OPERATION_SPECS.map(|(operation, id, _)| (operation, id, OPERATION_SUCCESS, None)),
    );
    let (mut client, fake) = client_for_script(ExchangeScript::Success {
        response,
        request_write_chunks: vec![3, 11],
    });
    let result = client
        .execute(
            ClientBatch::from_items(requests()),
            &CodecLimits::defaults(),
        )
        .expect("all four response payloads satisfy their typed success contracts");

    for (index, (operation, _, _)) in OPERATION_SPECS.iter().enumerate() {
        let outcome = result
            .get(index)
            .expect("response order is preserved")
            .outcome();
        assert_eq!(outcome.result().status().raw(), OPERATION_SUCCESS);
        let view = outcome.response();
        assert_eq!(view.derive_key().is_some(), *operation == DERIVE_KEY);
        assert_eq!(view.export().is_some(), *operation == EXPORT);
        assert_eq!(view.import().is_some(), *operation == IMPORT);
        assert_eq!(
            view.join_split_key().is_some(),
            *operation == JOIN_SPLIT_KEY
        );
        assert_eq!(view.result().status().raw(), OPERATION_SUCCESS);
        assert_eq!(
            outcome.derive_key_response().is_some(),
            *operation == DERIVE_KEY
        );
        assert_eq!(outcome.export_response().is_some(), *operation == EXPORT);
        assert_eq!(outcome.import_response().is_some(), *operation == IMPORT);
        assert_eq!(
            outcome.join_split_key_response().is_some(),
            *operation == JOIN_SPLIT_KEY
        );
        assert_ne!(format!("{view:?}"), "");
        assert_ne!(format!("{outcome:?}"), "");
        assert_ne!(outcome.to_string(), "");
    }
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn each_operation_reports_failure_delivery_state_without_retry() {
    for (index, operation) in OPERATION_SPECS.iter().copied().enumerate() {
        let cases = [
            (
                ExchangeScript::FailBeforeWrite,
                RequestDeliveryState::NotSent,
            ),
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
                .execute(ClientBatch::new(item), &CodecLimits::defaults())
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
