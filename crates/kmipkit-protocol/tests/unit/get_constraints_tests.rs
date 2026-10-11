//! Project-derived checks for OASIS KMIP Specification v2.1 §6.1.22,
//! Tables 229–231, and the Constraints Structure in §7.7.
//! Traceability: KMIPKIT-025-T229-REQUEST, T230-RESPONSE, T231-ERROR.
//! These are not official OASIS Test Cases.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{GetConstraintsRequest, GetConstraintsResponse, ResponseMessage, ResultReason};
use kmipkit_ttlv::{Item, ItemType, Structure, Value};

const GET_CONSTRAINTS: u32 = 0x0000_0038;
const CONSTRAINTS: u32 = 0x0042_0168;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const VENDOR_NESTED: u32 = 0x0054_1235;

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("project fixture has one response item")
}

fn constraints() -> Structure {
    structure([
        item(VENDOR_MEMBER, Value::integer(7)),
        item(
            VENDOR_NESTED,
            Value::structure(structure([
                item(VENDOR_MEMBER, Value::text_string("opaque-first".to_owned())),
                item(
                    VENDOR_MEMBER,
                    Value::text_string("opaque-second".to_owned()),
                ),
            ])),
        ),
        item(VENDOR_MEMBER, Value::integer(-3)),
    ])
}

fn success_payload(children: impl IntoIterator<Item = Item>) -> Structure {
    structure(children)
}

#[test]
fn table_229_request_is_exactly_empty() {
    let payload = GetConstraintsRequest::new()
        .to_ttlv_payload()
        .expect("Table 229 has an empty Request Payload");
    assert!(payload.view().children().is_empty());
}

#[test]
fn table_230_returns_required_constraints_without_reordering_unknown_nested_members() {
    let message = response_message(
        GET_CONSTRAINTS,
        0,
        None,
        None,
        Some(success_payload([item(
            CONSTRAINTS,
            Value::structure(constraints()),
        )])),
    );
    let actual = GetConstraintsResponse::try_from_response_item(response_item(&message))
        .expect("a Table 230 success is typed");

    assert_eq!(actual.result().status().raw(), 0);
    let returned = actual.constraints().expect("success requires Constraints");
    let view = returned.view();
    let fields = view.children();
    assert!(fields[0].with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Integer(7))));
    assert!(fields[2].with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Integer(-3))));
    fields[1].with_value(|value| {
        let kmipkit_ttlv::ValueView::Structure(nested) = value else {
            panic!("second field is a nested Structure");
        };
        assert_eq!(nested.children().len(), 2);
        assert!(nested.children()[0].with_value(|value| matches!(
            value,
            kmipkit_ttlv::ValueView::TextString("opaque-first")
        )));
        assert!(nested.children()[1].with_value(|value| matches!(
            value,
            kmipkit_ttlv::ValueView::TextString("opaque-second")
        )));
    });
    assert_eq!(
        returned
            .view()
            .children()
            .iter()
            .map(|child| (child.tag().raw(), child.item_type()))
            .collect::<Vec<_>>(),
        [
            (VENDOR_MEMBER, ItemType::Integer),
            (VENDOR_NESTED, ItemType::Structure),
            (VENDOR_MEMBER, ItemType::Integer),
        ]
    );
}

#[test]
fn table_230_rejects_missing_wrong_typed_and_duplicate_constraints() {
    let malformed = [
        Some(Structure::new()),
        Some(success_payload([item(CONSTRAINTS, Value::integer(1))])),
        Some(success_payload([
            item(CONSTRAINTS, Value::structure(Structure::new())),
            item(CONSTRAINTS, Value::structure(Structure::new())),
        ])),
        Some(success_payload([item(VENDOR_MEMBER, Value::integer(1))])),
    ];
    for payload in malformed {
        let message = response_message(GET_CONSTRAINTS, 0, None, None, payload);
        assert!(
            GetConstraintsResponse::try_from_response_item(response_item(&message)).is_err(),
            "Table 230 requires one Constraints Structure"
        );
    }
}

#[test]
fn table_230_preserves_unknown_sibling_fields_in_complete_response_ttlv() {
    const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
    let actual = {
        let message = response_message(
            GET_CONSTRAINTS,
            0,
            None,
            None,
            Some(success_payload([
                item(VENDOR_MEMBER, Value::integer(41)),
                item(CONSTRAINTS, Value::structure(constraints())),
                item(
                    VENDOR_MEMBER,
                    Value::text_string("opaque-sibling".to_owned()),
                ),
            ])),
        );
        GetConstraintsResponse::try_from_response_item(response_item(&message))
            .expect("unknown siblings do not invalidate the required Constraints field")
    };

    actual.with_ttlv(|response_item| {
        let payload = response_item
            .children()
            .iter()
            .find(|field| field.tag().raw() == RESPONSE_PAYLOAD)
            .expect("complete response item retains Response Payload");
        payload.with_value(|value| {
            let kmipkit_ttlv::ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert_eq!(
                payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect::<Vec<_>>(),
                [VENDOR_MEMBER, CONSTRAINTS, VENDOR_MEMBER]
            );
            assert!(
                payload.children()[0]
                    .with_value(|value| matches!(value, kmipkit_ttlv::ValueView::Integer(41)))
            );
            assert!(payload.children()[2].with_value(|value| matches!(
                value,
                kmipkit_ttlv::ValueView::TextString("opaque-sibling")
            )));
        });
    });
}

#[test]
fn table_230_missing_success_payload_is_rejected_by_common_message_validator() {
    // The shared ResponseMessage validator rejects this malformed message
    // before a typed ResponseBatchItemView can be constructed.
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
        item(0x0042_005C, Value::enumeration(GET_CONSTRAINTS)),
        item(0x0042_007F, Value::enumeration(0)),
    ]);
    let message = structure([
        item(0x0042_007A, Value::structure(header)),
        item(0x0042_000F, Value::structure(batch)),
    ]);
    assert!(ResponseMessage::try_from_ttlv(message).is_err());
}

#[test]
fn table_231_preserves_every_distinct_reason_and_open_result_reason() {
    // Table 231 repeats Invalid Field. One case per distinct listed value suffices.
    for reason in [0x07, 0x30, 0x15, 0x14, 0x08, 0x04, 0x05, 0x0C, 0x02, 0x54FF] {
        let message = response_message(GET_CONSTRAINTS, 1, Some(reason), None, None);
        let actual = GetConstraintsResponse::try_from_response_item(response_item(&message))
            .expect("failure uses the common result without a success payload");
        assert_eq!(actual.result().status().raw(), 1);
        assert_eq!(
            actual.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
        assert!(actual.constraints().is_none());
    }
}

#[test]
fn non_success_and_wrong_operation_do_not_fabricate_constraints() {
    let pending = response_message(
        GET_CONSTRAINTS,
        2,
        None,
        Some(b"opaque"),
        Some(Structure::new()),
    );
    let pending = GetConstraintsResponse::try_from_response_item(response_item(&pending))
        .expect("Pending uses common result handling");
    assert_eq!(pending.result().status().raw(), 2);
    assert!(pending.constraints().is_none());

    let wrong = response_message(0x0000_000C, 1, Some(0x07), None, None);
    assert!(GetConstraintsResponse::try_from_response_item(response_item(&wrong)).is_err());
}
