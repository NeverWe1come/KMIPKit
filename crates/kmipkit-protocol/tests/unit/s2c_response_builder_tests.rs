use crate::server_to_client::ServerResponseBuilder;
use crate::{ProtocolVersion, ResultReason, ResultStatus};
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

#[test]
fn test_server_response_builder_constructors_and_success() {
    let builder = ServerResponseBuilder::new();
    assert!(format!("{builder:?}").contains("ServerResponseBuilder"));

    let custom_ver = ProtocolVersion::from_raw(2, 0);
    let builder2 =
        ServerResponseBuilder::with_version(custom_ver).with_maximum_response_size(Some(65536));

    let mut payload = Structure::new();
    payload
        .try_push(
            Item::new(
                RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(),
                Value::text_string("obj-123".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

    let resp = builder2
        .build_success(0x1C, Some(payload))
        .expect("build_success succeeds");
    assert_eq!(resp.header().protocol_version(), custom_ver);
    assert_eq!(resp.header().batch_count(), 1);

    let items: Vec<_> = resp.batch_items().collect();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].operation(), Some(0x1C));
    assert_eq!(
        items[0]
            .result_status()
            .map(super::result::ResultStatus::raw),
        Some(0)
    ); // Success
}

#[test]
fn test_server_response_builder_build_success_without_payload() {
    let builder = ServerResponseBuilder::default();
    let resp = builder.build_success(0x1B, None).expect("build succeeds");
    assert_eq!(resp.header().batch_count(), 1);
    let item = resp.batch_items().next().expect("item exists");
    assert_eq!(item.operation(), Some(0x1B));
    assert_eq!(
        item.result_status().map(super::result::ResultStatus::raw),
        Some(0)
    );
}

#[test]
fn test_server_response_builder_build_error() {
    let builder = ServerResponseBuilder::new();
    let resp = builder
        .build_error(
            0x1E,
            ResultReason::from_raw(0x05),
            Some("operation unsupported"),
        )
        .expect("build_error succeeds");

    assert_eq!(resp.header().batch_count(), 1);
    let item = resp.batch_items().next().expect("item exists");
    assert_eq!(item.operation(), Some(0x1E));
    assert_eq!(
        item.result_status().map(super::result::ResultStatus::raw),
        Some(1)
    ); // OperationFailed
    assert_eq!(
        item.result_reason().map(super::result::ResultReason::raw),
        Some(0x05)
    );
    assert_eq!(
        item.with_result_message(std::string::ToString::to_string),
        Some("operation unsupported".to_string())
    );
}

#[test]
fn test_server_response_builder_batch_with_unique_ids() {
    let mut builder = ServerResponseBuilder::new();

    // Add success with unique ID
    let mut payload1 = Structure::new();
    payload1
        .try_push(
            Item::new(
                RawTag::new(0x0042_0094).unwrap().try_checked().unwrap(),
                Value::text_string("obj-abc".to_string()),
            )
            .unwrap(),
        )
        .unwrap();
    builder
        .add_success_item(0x1C, Some(vec![1, 2, 3]), payload1)
        .expect("add success item");

    // Add failure with unique ID
    builder
        .add_failure_item(
            0x18,
            Some(vec![4, 5, 6]),
            ResultStatus::from_raw(1),
            Some(ResultReason::from_raw(2)),
            Some("failed to query".to_string()),
        )
        .expect("add failure item");

    // Add raw item structure
    let mut raw_item = Structure::new();
    raw_item
        .try_push(
            Item::new(
                RawTag::new(0x0042_005C).unwrap().try_checked().unwrap(),
                Value::enumeration(0x32),
            )
            .unwrap(),
        )
        .unwrap();
    raw_item
        .try_push(
            Item::new(
                RawTag::new(0x0042_007F).unwrap().try_checked().unwrap(),
                Value::enumeration(0),
            )
            .unwrap(),
        )
        .unwrap();
    raw_item
        .try_push(
            Item::new(
                RawTag::new(0x0042_007C).unwrap().try_checked().unwrap(),
                Value::structure(Structure::new()),
            )
            .unwrap(),
        )
        .unwrap();
    builder.add_item(raw_item);

    let resp = builder.build().expect("build batch succeeds");
    assert_eq!(resp.header().batch_count(), 3);
    let items: Vec<_> = resp.batch_items().collect();
    assert_eq!(items.len(), 3);
    assert_eq!(
        items[0].with_unique_batch_item_id(<[u8]>::to_vec),
        Some(vec![1, 2, 3])
    );
    assert_eq!(
        items[1].with_unique_batch_item_id(<[u8]>::to_vec),
        Some(vec![4, 5, 6])
    );

    // Test malformed response message build failure
    let mut bad_builder = ServerResponseBuilder::new();
    bad_builder.add_item(Structure::new());
    let err = bad_builder.build();
    assert!(err.is_err());

    // Test protocol_model_error helper
    let model_err = kmipkit_ttlv::ModelError::StructureDepthExceeded;
    let proto_err = crate::server_to_client::response_builder::protocol_model_error(model_err);
    assert_eq!(proto_err.kind(), crate::ProtocolErrorKind::InvalidValue);
}
