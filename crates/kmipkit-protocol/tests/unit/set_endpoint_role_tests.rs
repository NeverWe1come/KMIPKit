//! Project-derived checks for OASIS KMIP Specification v2.1 §6.1.54,
//! Tables 331–333, and Endpoint Role §11.19, Tables 451–452. These are not
//! official OASIS Test Cases.
//! Traceability: KMIPKIT-025-T331-REQUEST, T332-RESPONSE, T333-ERROR,
//! KMIPKIT-025-COMMON-RESULT.

use crate::async_operation_fixtures::{item, response_message, structure};
use crate::{
    EndpointRole, ResponseMessage, ResultReason, SetEndpointRoleRequest, SetEndpointRoleResponse,
};
use kmipkit_ttlv::{ItemType, Structure, Value, ValueView};

const SET_ENDPOINT_ROLE: u32 = 0x0000_0032;
const ENDPOINT_ROLE: u32 = 0x0042_0151;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const VENDOR_MEMBER: u32 = 0x0054_1234;
const TABLE_333_DISTINCT_REASONS: [u32; 8] = [0x0C, 0x15, 0x14, 0x08, 0x07, 0x04, 0x05, 0x02];

fn response_item(message: &ResponseMessage) -> crate::ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("project fixture contains one response item")
}

#[test]
fn table_331_requires_endpoint_role_and_preserves_open_enumerations() {
    for raw_role in [1, 2, 3, 0x8000_0054] {
        let payload = SetEndpointRoleRequest::new(EndpointRole::from_raw(raw_role))
            .to_ttlv_payload()
            .expect("Table 331 request is representable");
        let view = payload.view();
        let fields = view.children();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].tag().raw(), ENDPOINT_ROLE);
        assert_eq!(fields[0].item_type(), ItemType::Enumeration);
        assert!(
            fields[0].with_value(
                |value| matches!(value, ValueView::Enumeration(raw) if *raw == raw_role)
            )
        );
    }
}

#[test]
fn table_332_requires_and_preserves_known_or_extension_role_values() {
    for raw_role in [1, 2, 3, 0x8000_0054] {
        let payload = structure([item(ENDPOINT_ROLE, Value::enumeration(raw_role))]);
        let message = response_message(SET_ENDPOINT_ROLE, 0, None, None, Some(payload));
        let response = SetEndpointRoleResponse::try_from_response_item(response_item(&message))
            .expect("Table 332 returns the accepted Endpoint Role");
        assert_eq!(
            response.endpoint_role(),
            Some(EndpointRole::from_raw(raw_role))
        );
    }
}

#[test]
fn table_332_rejects_missing_wrong_typed_or_duplicate_endpoint_role() {
    let malformed = [
        Structure::new(),
        structure([item(ENDPOINT_ROLE, Value::integer(1))]),
        structure([
            item(ENDPOINT_ROLE, Value::enumeration(1)),
            item(ENDPOINT_ROLE, Value::enumeration(2)),
        ]),
        structure([item(VENDOR_MEMBER, Value::enumeration(1))]),
    ];
    for payload in malformed {
        let message = response_message(SET_ENDPOINT_ROLE, 0, None, None, Some(payload));
        assert!(SetEndpointRoleResponse::try_from_response_item(response_item(&message)).is_err());
    }
}

#[test]
fn table_332_preserves_unknown_sibling_fields_with_the_required_role() {
    let payload = structure([
        item(VENDOR_MEMBER, Value::text_string("vendor-value".to_owned())),
        item(ENDPOINT_ROLE, Value::enumeration(3)),
    ]);
    let message = response_message(SET_ENDPOINT_ROLE, 0, None, None, Some(payload));
    let response = SetEndpointRoleResponse::try_from_response_item(response_item(&message))
        .expect("unknown Response Payload members do not discard a required role");
    assert_eq!(response.endpoint_role(), Some(EndpointRole::from_raw(3)));
    response.with_ttlv(|item| {
        let response_payload = item
            .children()
            .iter()
            .find(|field| field.tag().raw() == RESPONSE_PAYLOAD)
            .expect("successful response retains its Response Payload");
        response_payload.with_value(|value| {
            let ValueView::Structure(payload) = value else {
                panic!("Response Payload is a Structure");
            };
            assert_eq!(payload.children().len(), 2);
            assert_eq!(payload.children()[0].tag().raw(), VENDOR_MEMBER);
            assert!(
                payload.children()[0]
                    .with_value(|value| matches!(value, ValueView::TextString("vendor-value")))
            );
            assert_eq!(payload.children()[1].tag().raw(), ENDPOINT_ROLE);
            assert!(
                payload.children()[1]
                    .with_value(|value| matches!(value, ValueView::Enumeration(3)))
            );
        });
    });
}

#[test]
fn table_333_preserves_all_distinct_reasons_and_open_unknown_reason() {
    for reason in TABLE_333_DISTINCT_REASONS.into_iter().chain([0x8000_0054]) {
        let message = response_message(SET_ENDPOINT_ROLE, 1, Some(reason), None, None);
        let response = SetEndpointRoleResponse::try_from_response_item(response_item(&message))
            .expect("Table 333 Failure uses the common result contract");
        assert_eq!(response.result().status().raw(), 1);
        assert_eq!(
            response.result().reason(),
            Some(ResultReason::from_raw(reason))
        );
        assert!(response.endpoint_role().is_none());
    }
}

#[test]
fn pending_and_wrong_operation_do_not_fabricate_endpoint_role() {
    let pending = response_message(
        SET_ENDPOINT_ROLE,
        2,
        None,
        Some(b"set-endpoint-role-pending"),
        Some(Structure::new()),
    );
    let response = SetEndpointRoleResponse::try_from_response_item(response_item(&pending))
        .expect("Pending uses the common result contract");
    assert_eq!(response.result().status().raw(), 2);
    assert!(response.endpoint_role().is_none());

    let wrong = response_message(0x0000_0001, 1, Some(0x07), None, None);
    assert!(SetEndpointRoleResponse::try_from_response_item(response_item(&wrong)).is_err());
}
