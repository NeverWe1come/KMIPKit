use crate::extension::{extension_information, with_description, with_tag, with_type};
use crate::{
    ProtocolVersion, QueryFunction, RequestMessage, ServerCapabilities, ServerQueryRequest,
    ServerQueryResponse,
};
use kmipkit_ttlv::{Item, ItemType, KmipTag, RawTag, Structure, Value, ValueView};

fn wrap_request_item(operation: u32, payload: Structure) -> RequestMessage {
    let mut pv = Structure::new();
    pv.try_push(
        Item::new(
            RawTag::new(0x0042_006A).unwrap().try_checked().unwrap(),
            Value::integer(2),
        )
        .unwrap(),
    )
    .unwrap();
    pv.try_push(
        Item::new(
            RawTag::new(0x0042_006B).unwrap().try_checked().unwrap(),
            Value::integer(1),
        )
        .unwrap(),
    )
    .unwrap();

    let mut header = Structure::new();
    header
        .try_push(
            Item::new(
                RawTag::new(0x0042_0069).unwrap().try_checked().unwrap(),
                Value::structure(pv),
            )
            .unwrap(),
        )
        .unwrap();
    header
        .try_push(
            Item::new(
                RawTag::new(0x0042_000D).unwrap().try_checked().unwrap(),
                Value::integer(1),
            )
            .unwrap(),
        )
        .unwrap();

    let mut item_structure = Structure::new();
    item_structure
        .try_push(
            Item::new(
                RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(),
                Value::enumeration(operation),
            )
            .unwrap(),
        )
        .unwrap();
    item_structure
        .try_push(
            Item::new(
                RawTag::new(0x0042_0079).unwrap().try_checked().unwrap(),
                Value::structure(payload),
            )
            .unwrap(),
        )
        .unwrap();

    let mut msg = Structure::new();
    msg.try_push(
        Item::new(
            RawTag::new(0x0042_0077).unwrap().try_checked().unwrap(),
            Value::structure(header),
        )
        .unwrap(),
    )
    .unwrap();
    msg.try_push(
        Item::new(
            RawTag::new(0x0042_000F).unwrap().try_checked().unwrap(),
            Value::structure(item_structure),
        )
        .unwrap(),
    )
    .unwrap();

    RequestMessage::try_from_ttlv(msg).expect("valid request message fixture")
}

#[test]
fn test_server_query_request_decoding() {
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0074).unwrap().try_checked().unwrap(),
                Value::enumeration(1),
            )
            .unwrap(),
        )
        .unwrap(); // Query Operations (1)
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0074).unwrap().try_checked().unwrap(),
                Value::enumeration(2),
            )
            .unwrap(),
        )
        .unwrap(); // Query Objects (2)

    let msg = wrap_request_item(0x0000_0018, payload);
    let view = msg.batch_items().next().expect("valid view");

    let req = ServerQueryRequest::try_from_request_item(view).expect("valid query request");
    assert_eq!(req.query_functions().len(), 2);
    assert_eq!(req.query_functions()[0], QueryFunction::OPERATIONS);
    assert_eq!(req.query_functions()[1], QueryFunction::OBJECTS);
}

#[test]
fn test_server_query_response_extension_map_supersedes_extension_list() {
    // Normative rule §6.2.4: If both Extension List (5) and Extension Map (6) are requested,
    // only Extension Map is returned.
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0074).unwrap().try_checked().unwrap(),
                Value::enumeration(5),
            )
            .unwrap(),
        )
        .unwrap(); // Extension List
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0074).unwrap().try_checked().unwrap(),
                Value::enumeration(6),
            )
            .unwrap(),
        )
        .unwrap(); // Extension Map

    let msg = wrap_request_item(0x0000_0018, payload);
    let view = msg.batch_items().next().expect("valid view");

    let req = ServerQueryRequest::try_from_request_item(view).expect("valid query request");

    let mut capabilities = ServerCapabilities::default();
    capabilities.set_vendor_identification("KMIPKit");

    let resp = ServerQueryResponse::respond(&req, &capabilities);
    let payload_structure = resp.to_ttlv_payload().expect("payload build succeeds");

    assert!(payload_structure.view().children().is_empty());
}

#[test]
fn test_server_query_constructors_and_capabilities_coverage() {
    use crate::QueryFunction;
    use crate::server_to_client::query::ServerQueryError;

    let mut caps = ServerCapabilities::new();
    caps.set_vendor_identification("AcmeVendor");
    caps.set_operations(vec![0x18, 0x1E]);
    caps.set_object_types(vec![1, 2]);
    caps.set_extension_information(vec![extension_information("TenantLabel").unwrap()]);
    assert_eq!(caps.vendor_identification(), Some("AcmeVendor"));

    let req = ServerQueryRequest::new(vec![
        QueryFunction::OPERATIONS,
        QueryFunction::OBJECTS,
        QueryFunction::SERVER_INFORMATION,
        QueryFunction::EXTENSION_LIST,
    ]);
    assert_eq!(req.query_functions().len(), 4);

    let resp = ServerQueryResponse::respond(&req, &caps);
    let ttlv = resp.to_ttlv_payload().expect("to_ttlv_payload succeeds");
    let binding = ttlv.view();
    let children = binding.children();
    assert_eq!(children.len(), 6); // 2 operations + 2 object types + 1 server information + 1 extension list

    let err1 = ServerQueryError::UnexpectedOperation;
    assert!(format!("{err1}").contains("Query"));

    let err2 = ServerQueryError::MissingPayload;
    assert!(format!("{err2}").contains("missing payload"));

    let err3 = ServerQueryError::MalformedPayload;
    assert!(format!("{err3}").contains("malformed Query payload"));

    let msg = wrap_request_item(0x0000_0001, Structure::new());
    let view = msg.batch_items().next().expect("valid view");
    let err = ServerQueryRequest::try_from_request_item(view).unwrap_err();
    assert_eq!(err, ServerQueryError::UnexpectedOperation);

    let model_err = kmipkit_ttlv::ModelError::StructureDepthExceeded;
    let proto_err = crate::server_to_client::query::protocol_model_error(model_err);
    assert_eq!(proto_err.kind(), crate::ProtocolErrorKind::InvalidValue);
}

#[test]
fn test_server_query_extension_map_and_unknown_payload_fields() {
    use crate::QueryFunction;

    let mut caps = ServerCapabilities::new();
    caps.set_extension_information(vec![
        mapped_information("TenantLabel", 0x54_0001, ItemType::TextString),
        mapped_information("TenantFlags", 0x54_0002, ItemType::Integer),
    ]);

    let req = ServerQueryRequest::new(vec![QueryFunction::EXTENSION_MAP]);
    let resp = ServerQueryResponse::respond(&req, &caps);
    let ttlv = resp.to_ttlv_payload().expect("to_ttlv_payload succeeds");
    assert_eq!(ttlv.view().children().len(), 2);

    // Mismatched value type in QUERY_FUNCTION_TAG (Integer instead of Enumeration) and unknown tag
    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0074).unwrap().try_checked().unwrap(),
                Value::integer(1),
            )
            .unwrap(),
        )
        .unwrap();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0001).unwrap().try_checked().unwrap(),
                Value::integer(42),
            )
            .unwrap(),
        )
        .unwrap();

    let msg = wrap_request_item(0x0000_0018, payload);
    let view = msg.batch_items().next().expect("valid view");
    let parsed = ServerQueryRequest::try_from_request_item(view).expect("valid query request");
    assert_eq!(parsed.query_functions(), []);
}

fn mapped_information(
    name: &str,
    tag: u32,
    item_type: ItemType,
) -> crate::extension::ExtensionInformation {
    with_type(
        with_tag(extension_information(name).unwrap(), tag).unwrap(),
        item_type,
    )
    .unwrap()
}

#[test]
fn extension_query_encodes_the_kmip_structure_for_each_version() {
    let versions = [
        ProtocolVersion::V2_0,
        ProtocolVersion::V2_1,
        ProtocolVersion::V3_0,
    ];
    let mut caps = ServerCapabilities::new();
    caps.set_extension_information(vec![
        with_description(
            mapped_information("Tenant🔑", 0x54_0002, ItemType::TextString),
            "optional",
        )
        .unwrap(),
        mapped_information("TenantID", 0x54_0001, ItemType::Integer),
    ]);

    for version in versions {
        for (functions, map) in [
            (vec![QueryFunction::EXTENSION_LIST], false),
            (vec![QueryFunction::EXTENSION_MAP], true),
            (
                vec![QueryFunction::EXTENSION_LIST, QueryFunction::EXTENSION_MAP],
                true,
            ),
        ] {
            let payload = ServerQueryResponse::respond(&ServerQueryRequest::new(functions), &caps)
                .to_ttlv_payload_for_version(version)
                .unwrap();
            let view = payload.view();
            assert_eq!(view.children().len(), 2);
            for (index, item) in view.children().iter().enumerate() {
                assert_eq!(item.tag(), KmipTag::ExtensionInformation.into());
                item.with_value(|value| {
                    let ValueView::Structure(fields) = value else {
                        panic!("expected Extension Information structure")
                    };
                    let fields = fields.children();
                    let (name, tag, item_type) = if index == 0 {
                        ("Tenant🔑", 0x54_0002, 7)
                    } else {
                        ("TenantID", 0x54_0001, 2)
                    };
                    assert_eq!(fields[0].tag(), KmipTag::ExtensionName.into());
                    fields[0].with_value(|v| {
                        assert!(matches!(v, ValueView::TextString(value) if value == name));
                    });
                    if map {
                        assert_eq!(fields.len(), if index == 0 { 4 } else { 3 });
                        assert_eq!(fields[1].tag(), KmipTag::ExtensionTag.into());
                        fields[1].with_value(|v| {
                            assert!(matches!(v, ValueView::Integer(value) if *value == tag));
                        });
                        assert_eq!(fields[2].tag(), KmipTag::ExtensionType.into());
                        fields[2].with_value(|v| {
                            assert!(
                                matches!(v, ValueView::Enumeration(value) if *value == item_type)
                            );
                        });
                        if index == 0 {
                            assert_eq!(fields[3].tag(), KmipTag::ExtensionDescription.into());
                            fields[3].with_value(|v| {
                                assert!(matches!(v, ValueView::TextString("optional")));
                            });
                        }
                    } else {
                        assert_eq!(fields.len(), 1);
                    }
                });
            }
        }
    }
}

#[test]
#[allow(deprecated)]
fn extension_map_requires_complete_metadata_and_a_versioned_item_type() {
    let request = ServerQueryRequest::new(vec![QueryFunction::EXTENSION_MAP]);
    let mut caps = ServerCapabilities::new();
    caps.set_extension_tags(vec![0x54_0001]);
    assert!(
        ServerQueryResponse::respond(&request, &caps)
            .to_ttlv_payload()
            .is_err()
    );
    for information in [
        extension_information("NameOnly").unwrap(),
        mapped_information("StandardTag", 0x42_0001, ItemType::TextString),
    ] {
        caps.set_extension_information(vec![information]);
        assert!(
            ServerQueryResponse::respond(&request, &caps)
                .to_ttlv_payload()
                .is_err()
        );
    }
    caps.set_extension_information(vec![mapped_information(
        "TenantRef",
        0x54_0001,
        ItemType::NameReference,
    )]);
    let response = ServerQueryResponse::respond(&request, &caps);
    for version in [ProtocolVersion::V2_0, ProtocolVersion::V2_1] {
        assert!(response.to_ttlv_payload_for_version(version).is_err());
    }
    assert!(
        response
            .to_ttlv_payload_for_version(ProtocolVersion::V3_0)
            .is_ok()
    );
}
