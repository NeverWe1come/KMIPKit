//! Preservation regressions for unknown KMIP lifecycle results and extensions.
//!
//! Derived from KMIP 2.1 §§5.1 and 8.6 and KMIPKIT-0018 FR-008. These tests
//! verify accepted generic data retention; they are not official OASIS Test
//! Cases and do not imply server policy for unknown result values.

use kmipkit_protocol::{ActivateRequest, ArchiveRequest, DestroyRequest, RecoverRequest};
use kmipkit_test_support::ExchangeScript;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute::encode_message_for_test;
use crate::execute_test_support::{test_item, test_structure};
use crate::{ClientBatch, ClientBatchItem, ClientMessageExtension, ClientRequest};

const RESPONSE_HEADER: u32 = 0x0042_007A;
const PROTOCOL_VERSION: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
const TIME_STAMP: u32 = 0x0042_0092;
const BATCH_COUNT: u32 = 0x0042_000D;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const RESULT_STATUS: u32 = 0x0042_007F;
const RESULT_REASON: u32 = 0x0042_007E;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const FUTURE_EXTENSION_TAG: u32 = 0x0042_0173;

const UNKNOWN_STATUS: u32 = 0xA1B2_C3D4;
const UNKNOWN_REASON: u32 = 0xC3D4_E5F6;
const UNKNOWN_EXTENSION_ENUMERATION: u32 = 0xE5F6_A1B2;

#[test]
fn lifecycle_results_preserve_unknown_status_reason_and_accepted_extension_values() {
    let requests = [
        (
            0x0000_0012,
            ClientRequest::Activate(ActivateRequest::new(None)),
        ),
        (
            0x0000_0013,
            ClientRequest::Archive(ArchiveRequest::new(None)),
        ),
        (
            0x0000_0014,
            ClientRequest::Destroy(DestroyRequest::new(None)),
        ),
        (
            0x0000_002A,
            ClientRequest::Recover(RecoverRequest::new(None)),
        ),
    ];

    for (operation, request) in requests {
        let (mut client, _, _) = client_for(ExchangeScript::Success {
            response: unknown_result_response(operation),
            request_write_chunks: Vec::new(),
        });
        let response = client
            .execute(
                ClientBatch::new(ClientBatchItem::new(request)),
                &CodecLimits::defaults(),
            )
            .expect("unknown non-Pending lifecycle result with an accepted extension is retained");
        let item = response.get(0).expect("one lifecycle response exists");
        let result = item.outcome().result();

        assert_eq!(result.status().raw(), UNKNOWN_STATUS);
        assert_eq!(result.status().known_name(), None);
        assert_eq!(
            result.reason().map(kmipkit_protocol::ResultReason::raw),
            Some(UNKNOWN_REASON)
        );
        assert_eq!(item.extensions().len(), 1);
        assert_eq!(
            extension_enumeration(&item.extensions()[0]),
            UNKNOWN_EXTENSION_ENUMERATION
        );
    }
}

fn extension_enumeration(extension: &ClientMessageExtension) -> u32 {
    extension.with_ttlv(|view| {
        let vendor_extension = view
            .children()
            .iter()
            .find(|field| field.tag().raw() == VENDOR_EXTENSION)
            .expect("the extension retains Vendor Extension");
        vendor_extension.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Vendor Extension is a Structure");
            };
            let field = payload
                .children()
                .iter()
                .find(|field| field.tag().raw() == FUTURE_EXTENSION_TAG)
                .expect("the extension retains its unknown child tag");
            field.with_value(|value| match value {
                ValueView::Enumeration(raw) => *raw,
                _ => panic!("the extension fixture retains an Enumeration value"),
            })
        })
    })
}

fn unknown_result_response(operation: u32) -> Vec<u8> {
    let version = test_structure([
        test_item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
        test_item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
    ]);
    let header = test_structure([
        test_item(PROTOCOL_VERSION, Value::structure(version)),
        test_item(TIME_STAMP, Value::date_time(1)),
        test_item(BATCH_COUNT, Value::integer(1)),
    ]);
    let extension = test_structure([
        test_item(
            VENDOR_IDENTIFICATION,
            Value::text_string("KMIPKitLifecycleFixture".to_owned()),
        ),
        test_item(CRITICALITY_INDICATOR, Value::boolean(false)),
        test_item(
            VENDOR_EXTENSION,
            Value::structure(test_structure([test_item(
                FUTURE_EXTENSION_TAG,
                Value::enumeration(UNKNOWN_EXTENSION_ENUMERATION),
            )])),
        ),
    ]);
    let batch_item = test_structure([
        test_item(OPERATION, Value::enumeration(operation)),
        test_item(RESULT_STATUS, Value::enumeration(UNKNOWN_STATUS)),
        test_item(RESULT_REASON, Value::enumeration(UNKNOWN_REASON)),
        test_item(RESPONSE_PAYLOAD, Value::structure(test_structure([]))),
        test_item(MESSAGE_EXTENSION, Value::structure(extension)),
    ]);
    encode_message_for_test(
        test_structure([
            test_item(RESPONSE_HEADER, Value::structure(header)),
            test_item(BATCH_ITEM, Value::structure(batch_item)),
        ]),
        &CodecLimits::defaults(),
    )
    .expect("unknown-value lifecycle response remains a valid TTLV message")
}
