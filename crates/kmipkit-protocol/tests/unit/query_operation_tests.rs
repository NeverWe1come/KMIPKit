//! Query behavior derived from OASIS KMIP v2.1 §6.1.40, Tables 281–284.
//! These structural vectors do not claim an official OASIS fixture pass.

use crate::query_ping_fixtures::{
    OBJECT_GROUP, OBJECT_GROUPS, QUERY_FUNCTION, QUERY_OPERATION, QUERY_RESPONSE_TAGS, item,
    response_item, response_message, structure,
};
use crate::{QueryFunction, QueryRequest, QueryResponse};
use kmipkit_ttlv::{Value, ValueView};

const STANDARD_QUERY_FUNCTIONS: [u32; 14] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];

#[test]
fn names_all_standard_query_function_values_and_preserves_future_values() {
    let named = [
        QueryFunction::OPERATIONS,
        QueryFunction::OBJECTS,
        QueryFunction::SERVER_INFORMATION,
        QueryFunction::APPLICATION_NAMESPACES,
        QueryFunction::EXTENSION_LIST,
        QueryFunction::EXTENSION_MAP,
        QueryFunction::ATTESTATION_TYPES,
        QueryFunction::RNGS,
        QueryFunction::VALIDATIONS,
        QueryFunction::PROFILES,
        QueryFunction::CAPABILITIES,
        QueryFunction::CLIENT_REGISTRATION_METHODS,
        QueryFunction::DEFAULTS_INFORMATION,
        QueryFunction::STORAGE_PROTECTION_MASKS,
    ];
    assert_eq!(named.map(QueryFunction::raw), STANDARD_QUERY_FUNCTIONS);
    assert_eq!(QueryFunction::from_raw(0x8000_0042).raw(), 0x8000_0042);
    assert!(
        QueryRequest::new([QueryFunction::from_raw(15)])
            .to_ttlv_payload()
            .is_err()
    );
}

#[test]
fn query_request_preserves_required_repeated_functions_and_object_group_order() {
    let request = QueryRequest::new([
        QueryFunction::OPERATIONS,
        QueryFunction::OBJECTS,
        QueryFunction::OPERATIONS,
        QueryFunction::from_raw(0x8000_0042),
    ])
    .with_object_groups(["group-b", "group-a", "group-b"]);
    let payload = request
        .to_ttlv_payload()
        .expect("valid Query values are representable");
    let payload_view = payload.view();
    let fields = payload_view.children();

    assert_eq!(fields.len(), 5);
    assert_eq!(fields[0].tag().raw(), QUERY_FUNCTION);
    let functions = fields
        .iter()
        .filter(|field| field.tag().raw() == QUERY_FUNCTION)
        .map(|field| {
            field.with_value(|value| match value {
                ValueView::Enumeration(value) => Some(*value),
                _ => None,
            })
        })
        .collect::<Option<Vec<_>>>();
    assert_eq!(functions, Some(vec![1, 2, 1, 0x8000_0042]));

    assert_eq!(fields[4].tag().raw(), OBJECT_GROUPS);
    let groups = fields[4].with_value(|value| match value {
        ValueView::Structure(structure) => Some(
            structure
                .children()
                .iter()
                .map(|group| {
                    assert_eq!(group.tag().raw(), OBJECT_GROUP);
                    group.with_value(|value| match value {
                        ValueView::TextString(text) => (*text).to_owned(),
                        _ => panic!("Object Group is a Text String"),
                    })
                })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    });
    assert_eq!(
        groups,
        Some(vec!["group-b".into(), "group-a".into(), "group-b".into()])
    );

    let single_group = QueryRequest::new([QueryFunction::OPERATIONS])
        .with_object_groups(["one"])
        .to_ttlv_payload()
        .expect("one Object Group is valid");
    single_group.view().children()[1].with_value(|value| match value {
        ValueView::Structure(groups) => {
            assert_eq!(groups.children().len(), 1);
            groups.children()[0].with_value(|value| match value {
                ValueView::TextString(text) => assert_eq!(text, "one"),
                _ => panic!("Object Group is a Text String"),
            });
        }
        _ => panic!("Object Groups is a Structure"),
    });
}

#[test]
fn query_request_distinguishes_absent_and_present_empty_object_groups() {
    let absent = QueryRequest::new([QueryFunction::OPERATIONS])
        .to_ttlv_payload()
        .expect("one function is valid");
    let present_empty = QueryRequest::new([QueryFunction::OPERATIONS])
        .with_object_groups([String::new()].into_iter().take(0))
        .to_ttlv_payload()
        .expect("an empty Object Groups structure is valid");

    assert_eq!(absent.view().children().len(), 1);
    assert_eq!(present_empty.view().children().len(), 2);
    assert_eq!(
        present_empty.view().children()[1].tag().raw(),
        OBJECT_GROUPS
    );
    present_empty.view().children()[1].with_value(|value| match value {
        ValueView::Structure(groups) => assert!(groups.children().is_empty()),
        _ => panic!("Object Groups is a Structure"),
    });
}

#[test]
fn query_rejects_an_empty_function_list_before_it_can_be_encoded() {
    assert!(QueryRequest::new([]).to_ttlv_payload().is_err());
}

#[test]
fn query_accepts_the_empty_response_payload_form_and_preserves_common_failure_results() {
    let empty_message = response_message(QUERY_OPERATION, 0, None, None, Some(structure([])));
    let empty = QueryResponse::try_from_response_item(response_item(&empty_message))
        .expect("§6.1.40 describes the empty payload form");
    assert!(empty.is_empty_payload());
    assert!(empty.response_fields().is_empty());

    let failure = response_message(QUERY_OPERATION, 1, Some(1), Some("query refused"), None);
    let response = QueryResponse::try_from_response_item(response_item(&failure))
        .expect("valid failures preserve the common result");
    assert_eq!(response.result().status().raw(), 1);
    assert_eq!(
        response.result().reason().map(|reason| reason.raw()),
        Some(1)
    );
    assert_eq!(
        response.result().message().map(|message| message.as_str()),
        Some("query refused")
    );
    assert!(!response.is_empty_payload());
}

#[test]
fn query_response_exposes_all_table_283_members_and_unknown_nested_items() {
    let response_fields = vec![
        item(QUERY_RESPONSE_TAGS[0], Value::enumeration(0x8000_0042)),
        item(QUERY_RESPONSE_TAGS[0], Value::enumeration(0x8000_0043)),
        item(QUERY_RESPONSE_TAGS[1], Value::enumeration(0x8000_0044)),
        item(
            QUERY_RESPONSE_TAGS[2],
            Value::text_string("vendor".to_owned()),
        ),
        item(
            QUERY_RESPONSE_TAGS[3],
            Value::structure(structure([item(
                0x0042_0012,
                Value::text_string("server build".to_owned()),
            )])),
        ),
        item(
            QUERY_RESPONSE_TAGS[4],
            Value::text_string("urn:app:one".to_owned()),
        ),
        item(
            QUERY_RESPONSE_TAGS[4],
            Value::text_string("urn:app:two".to_owned()),
        ),
        item(QUERY_RESPONSE_TAGS[5], Value::structure(structure([]))),
        item(QUERY_RESPONSE_TAGS[6], Value::enumeration(0x8000_0045)),
        item(QUERY_RESPONSE_TAGS[7], Value::structure(structure([]))),
        item(QUERY_RESPONSE_TAGS[8], Value::structure(structure([]))),
        item(
            QUERY_RESPONSE_TAGS[9],
            Value::structure(structure([item(0x0042_0057, Value::integer(7))])),
        ),
        item(QUERY_RESPONSE_TAGS[10], Value::structure(structure([]))),
        item(QUERY_RESPONSE_TAGS[11], Value::enumeration(0x8000_0046)),
        item(QUERY_RESPONSE_TAGS[12], Value::structure(structure([]))),
        item(QUERY_RESPONSE_TAGS[13], Value::structure(structure([]))),
        item(
            0x0042_0012,
            Value::structure(structure([item(
                0x0042_0057,
                Value::byte_string(vec![0, 1, 2, 3]),
            )])),
        ),
    ];
    let message = response_message(
        QUERY_OPERATION,
        0,
        None,
        None,
        Some(structure(response_fields)),
    );
    let response = QueryResponse::try_from_response_item(response_item(&message))
        .expect("all Table 283 members and structurally valid unknown Items are preserved");

    assert!(!response.is_empty_payload());
    assert_eq!(response.response_fields().len(), 17);
    assert_eq!(
        response
            .response_fields()
            .iter()
            .map(|field| field.tag())
            .collect::<Vec<_>>(),
        [
            QUERY_RESPONSE_TAGS[0],
            QUERY_RESPONSE_TAGS[0],
            QUERY_RESPONSE_TAGS[1],
            QUERY_RESPONSE_TAGS[2],
            QUERY_RESPONSE_TAGS[3],
            QUERY_RESPONSE_TAGS[4],
            QUERY_RESPONSE_TAGS[4],
            QUERY_RESPONSE_TAGS[5],
            QUERY_RESPONSE_TAGS[6],
            QUERY_RESPONSE_TAGS[7],
            QUERY_RESPONSE_TAGS[8],
            QUERY_RESPONSE_TAGS[9],
            QUERY_RESPONSE_TAGS[10],
            QUERY_RESPONSE_TAGS[11],
            QUERY_RESPONSE_TAGS[12],
            QUERY_RESPONSE_TAGS[13],
            0x0042_0012,
        ]
    );
    assert_eq!(
        response.operations().collect::<Vec<_>>(),
        [0x8000_0042, 0x8000_0043]
    );
    assert_eq!(response.object_types().collect::<Vec<_>>(), [0x8000_0044]);
    assert_eq!(response.vendor_identification(), Some("vendor"));
    assert!(response.server_information().is_some());
    assert_eq!(
        response.application_namespaces().collect::<Vec<_>>(),
        ["urn:app:one", "urn:app:two"]
    );
    assert_eq!(response.extension_information().count(), 1);
    assert_eq!(
        response.attestation_types().collect::<Vec<_>>(),
        [0x8000_0045]
    );
    assert_eq!(response.rng_parameters().count(), 1);
    assert_eq!(response.profile_information().count(), 1);
    assert_eq!(response.validation_information().count(), 1);
    assert_eq!(response.capability_information().count(), 1);
    assert_eq!(
        response.client_registration_methods().collect::<Vec<_>>(),
        [0x8000_0046]
    );
    assert!(response.defaults_information().is_some());
    assert!(response.protection_storage_masks().is_some());
    assert_eq!(response.unknown_items().count(), 1);
    response
        .response_fields()
        .last()
        .unwrap()
        .with_ttlv(|unknown_item| {
            unknown_item.with_value(|value| match value {
                ValueView::Structure(nested) => {
                    assert_eq!(nested.children().len(), 1);
                    assert_eq!(nested.children()[0].tag().raw(), 0x0042_0057);
                }
                _ => panic!("unknown response item remains a Structure"),
            })
        });
}

#[test]
fn query_response_accepts_empty_protection_storage_masks_list() {
    let masks = structure([]);
    let message = response_message(
        QUERY_OPERATION,
        0,
        None,
        None,
        Some(structure([item(
            QUERY_RESPONSE_TAGS[13],
            Value::structure(masks),
        )])),
    );
    let response = QueryResponse::try_from_response_item(response_item(&message))
        .expect("an empty Protection Storage Masks list satisfies Table 283");
    assert_eq!(response.response_fields().len(), 1);
}

#[test]
fn query_response_rejects_missing_required_protection_storage_masks_in_structured_form() {
    let message = response_message(
        QUERY_OPERATION,
        0,
        None,
        None,
        Some(structure([item(
            QUERY_RESPONSE_TAGS[0],
            Value::enumeration(1),
        )])),
    );
    assert!(QueryResponse::try_from_response_item(response_item(&message)).is_err());
}

#[test]
fn query_response_rejects_repeated_singleton_table_283_fields() {
    let message = response_message(
        QUERY_OPERATION,
        0,
        None,
        None,
        Some(structure([
            item(
                QUERY_RESPONSE_TAGS[2],
                Value::text_string("vendor-a".into()),
            ),
            item(
                QUERY_RESPONSE_TAGS[2],
                Value::text_string("vendor-b".into()),
            ),
            item(QUERY_RESPONSE_TAGS[13], Value::structure(structure([]))),
        ])),
    );
    assert!(QueryResponse::try_from_response_item(response_item(&message)).is_err());
}

#[test]
fn query_response_rejects_a_different_operation() {
    let message = response_message(0x0000_001e, 0, None, None, Some(structure([])));
    assert!(QueryResponse::try_from_response_item(response_item(&message)).is_err());
}
