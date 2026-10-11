//! Project-derived tests for OASIS KMIP Specification v2.1 §6.1.52,
//! Tables 325–327 and §7.7 Constraints. These are not official test cases.
//! Traceability: KMIPKIT-025-T325-REQUEST, T326-RESPONSE, T327-ERROR,
//! KMIPKIT-025-COMMON-RESULT.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{ResponseMessage, ResultReason, SetConstraintsRequest, SetConstraintsResponse};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};

const SET_CONSTRAINTS: u32 = 0x0000_0037;
const CONSTRAINTS: u32 = 0x0042_0168;
const VENDOR_MEMBER: u32 = 0x0054_1234;

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("project fixture contains one response item")
}

fn constraints() -> Structure {
    structure([
        item(VENDOR_MEMBER, Value::integer(17)),
        item(
            VENDOR_MEMBER,
            Value::structure(structure([item(
                VENDOR_MEMBER,
                Value::text_string("opaque nested constraint".to_owned()),
            )])),
        ),
    ])
}

#[test]
fn table_325_requires_and_preserves_the_ordered_constraints_structure() {
    let requested = constraints();
    let payload = SetConstraintsRequest::new(requested)
        .to_ttlv_payload()
        .expect("Table 325 Constraints Structure is representable");
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].tag().raw(), CONSTRAINTS);
    assert_eq!(fields[0].item_type(), ItemType::Structure);
    fields[0].with_value(|value| {
        let ValueView::Structure(returned) = value else {
            panic!("Constraints is a Structure");
        };
        assert_eq!(returned.children().len(), 2);
        assert_eq!(returned.children()[0].tag().raw(), VENDOR_MEMBER);
        assert_eq!(returned.children()[1].tag().raw(), VENDOR_MEMBER);
        assert!(returned.children()[0].with_value(|value| matches!(value, ValueView::Integer(17))));
        returned.children()[1].with_value(|value| {
            let ValueView::Structure(nested) = value else {
                panic!("second constraint child is a Structure");
            };
            assert_eq!(nested.children().len(), 1);
            assert!(nested.children()[0].with_value(|value| matches!(
                value,
                ValueView::TextString("opaque nested constraint")
            )));
        });
    });
}

#[test]
fn table_326_success_has_empty_operation_payload_and_retains_complete_response() {
    let message = response_message(SET_CONSTRAINTS, 0, None, None, Some(Structure::new()));
    let response = SetConstraintsResponse::try_from_response_item(response_item(&message))
        .expect("Table 326 empty success response is typed");
    assert_eq!(response.result().status().raw(), 0);
    response.with_ttlv(|item| {
        let fields = item.children();
        let payload = fields
            .iter()
            .find(|field| field.tag().raw() == 0x0042_007C)
            .expect("Table 326 success carries an empty Response Payload");
        payload.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert!(payload.children().is_empty());
        });
    });
}

#[test]
fn table_326_rejects_success_without_response_payload() {
    // The common ResponseMessage validator rejects a successful response
    // without Response Payload before a typed batch-item view can be created.
    let version = structure([
        item(0x0042_006A, Value::integer(2)),
        item(0x0042_006B, Value::integer(1)),
    ]);
    let header = structure([
        item(0x0042_0069, Value::structure(version)),
        item(0x0042_0092, Value::date_time(1)),
        item(0x0042_000D, Value::integer(1)),
    ]);
    let batch = structure([
        item(0x0042_005C, Value::enumeration(SET_CONSTRAINTS)),
        item(0x0042_007F, Value::enumeration(0)),
    ]);
    let message = structure([
        item(0x0042_007A, Value::structure(header)),
        item(0x0042_000F, Value::structure(batch)),
    ]);
    assert!(ResponseMessage::try_from_ttlv(message).is_err());
}

#[test]
fn table_326_retains_unknown_success_payload_fields() {
    let vendor_payload = structure([item(VENDOR_MEMBER, Value::integer(1))]);
    let message = response_message(SET_CONSTRAINTS, 0, None, None, Some(vendor_payload));
    let response = SetConstraintsResponse::try_from_response_item(response_item(&message))
        .expect("unknown structurally valid response fields are retained");
    response.with_ttlv(|item| {
        let fields = item.children();
        let payload = fields
            .iter()
            .find(|field| field.tag().raw() == 0x0042_007C)
            .expect("Response Payload remains present");
        payload.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert_eq!(payload.children().len(), 1);
            assert_eq!(payload.children()[0].tag().raw(), VENDOR_MEMBER);
            assert!(
                payload.children()[0].with_value(|value| matches!(value, ValueView::Integer(1)))
            );
        });
    });
}

#[test]
fn table_327_preserves_all_distinct_reasons_and_open_unknown_reason() {
    // Table 327 repeats Invalid Field; test each distinct listed reason once.
    for reason in [
        0x07,
        0x30,
        0x15,
        0x14,
        0x08,
        0x04,
        0x05,
        0x0C,
        0x02,
        0x8000_0054,
    ] {
        let message = response_message(SET_CONSTRAINTS, 1, Some(reason), None, None);
        let response = SetConstraintsResponse::try_from_response_item(response_item(&message))
            .expect("Table 327 Failure uses the common result contract");
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
    }
}

#[test]
fn pending_is_preserved_without_fabricating_success_fields() {
    let message = response_message(
        SET_CONSTRAINTS,
        2,
        None,
        Some(b"set-constraints-pending"),
        Some(Structure::new()),
    );
    let response = SetConstraintsResponse::try_from_response_item(response_item(&message))
        .expect("Pending uses the common result contract");
    assert_eq!(response.result().status().raw(), 2);
}
