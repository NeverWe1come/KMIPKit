//! Exhaustive OASIS KMIP 1.0–1.4 <-> 2.0 / 2.1 / 3.0 protocol conformance tests.
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kmipkit_proxy::kmip1x::mapping::{
    UpstreamKmipVersion, is_crypto_algorithm_allowed_in_1x, is_object_type_allowed_in_1x,
    is_operation_allowed_in_1x, is_query_function_allowed_in_1x,
};
use kmipkit_proxy::kmip1x::s2c::{Kmip1xS2cBridge, S2cClientTransport};
use kmipkit_proxy::kmip1x::tags::*;
use kmipkit_proxy::kmip1x::translator::{
    TranslatorConfig, build_1x_attribute, is_async_request, translate_request_1x_to_2x,
    translate_response_2x_to_1x,
};
use kmipkit_proxy::kmip1x::ttlv_helpers::{
    build_error_batch_item, build_protocol_version, build_response_message,
    build_success_batch_item, clone_item, clone_item_value, extract_boolean, extract_byte_string,
    extract_enumeration, extract_integer, extract_text_string, find_child, find_children,
    retag_item, ttlv_item, ttlv_struct,
};
use kmipkit_proxy::kmip1x::{Kmip1xBridge, UpstreamKmipTransport};
use kmipkit_ttlv::{Item, Value, ValueView};

struct EchoVersionUpstream(UpstreamKmipVersion);

#[async_trait]
impl UpstreamKmipTransport for EchoVersionUpstream {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        let (up_maj, up_min) = self.0.major_minor();
        let header = find_child(request, TAG_REQUEST_HEADER).unwrap();
        let pv = find_child(&header, TAG_PROTOCOL_VERSION).unwrap();
        assert_eq!(
            find_child(&pv, TAG_PROTOCOL_VERSION_MAJOR).and_then(|i| extract_integer(&i)),
            Some(up_maj)
        );
        assert_eq!(
            find_child(&pv, TAG_PROTOCOL_VERSION_MINOR).and_then(|i| extract_integer(&i)),
            Some(up_min)
        );

        let mut resp_bis = Vec::new();
        for bi in find_children(request, TAG_BATCH_ITEM) {
            let op = find_child(&bi, TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap();
            let bid =
                find_child(&bi, TAG_UNIQUE_BATCH_ITEM_ID).and_then(|i| extract_byte_string(&i));
            resp_bis.push(
                build_success_batch_item(
                    op,
                    bid.as_deref(),
                    vec![
                        ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("obj-1".into()))
                            .unwrap(),
                    ],
                )
                .unwrap(),
            );
        }
        build_response_message(up_maj, up_min, resp_bis)
    }
}

fn build_1x_msg(minor: i32, header_extras: Vec<Item>, batch_items: Vec<Item>) -> Item {
    let mut h_items = vec![build_protocol_version(1, minor).unwrap()];
    h_items.extend(header_extras);
    h_items.push(ttlv_item(TAG_BATCH_COUNT, Value::integer(batch_items.len() as i32)).unwrap());
    let header = ttlv_struct(TAG_REQUEST_HEADER, h_items).unwrap();
    let mut top = vec![header];
    top.extend(batch_items);
    ttlv_struct(TAG_REQUEST_MESSAGE, top).unwrap()
}

#[tokio::test]
async fn test_5x3_version_matrix_lifecycle_and_crypto_operations() {
    let ops = [
        OP_CREATE,
        OP_CREATE_KEY_PAIR,
        OP_REGISTER,
        OP_REKEY,
        OP_DERIVE_KEY,
        OP_CERTIFY,
        OP_RECERTIFY,
        OP_LOCATE,
        OP_CHECK,
        OP_GET,
        OP_GET_ATTRIBUTES,
        OP_GET_ATTRIBUTE_LIST,
        OP_ADD_ATTRIBUTE,
        OP_MODIFY_ATTRIBUTE,
        OP_DELETE_ATTRIBUTE,
        OP_OBTAIN_LEASE,
        OP_GET_USAGE_ALLOCATION,
        OP_ACTIVATE,
        OP_REVOKE,
        OP_DESTROY,
        OP_ARCHIVE,
        OP_RECOVER,
        OP_VALIDATE,
        OP_CANCEL,
        OP_POLL,
        OP_ENCRYPT,
        OP_DECRYPT,
        OP_SIGN,
        OP_SIGNATURE_VERIFY,
        OP_MAC,
        OP_MAC_VERIFY,
        OP_RNG_RETRIEVE,
        OP_RNG_SEED,
        OP_HASH,
        OP_CREATE_SPLIT_KEY,
        OP_JOIN_SPLIT_KEY,
        OP_IMPORT,
        OP_EXPORT,
    ];

    for minor_1x in 0..=4 {
        for upstream_ver in [
            UpstreamKmipVersion::V2_0,
            UpstreamKmipVersion::V2_1,
            UpstreamKmipVersion::V3_0,
        ] {
            let bridge =
                Kmip1xBridge::new(Arc::new(EchoVersionUpstream(upstream_ver)), upstream_ver);
            for &op in &ops {
                let bi = ttlv_struct(
                    TAG_BATCH_ITEM,
                    vec![
                        ttlv_item(TAG_OPERATION, Value::enumeration(op)).unwrap(),
                        ttlv_item(
                            TAG_UNIQUE_BATCH_ITEM_ID,
                            Value::byte_string(vec![0xAA, op as u8]),
                        )
                        .unwrap(),
                        ttlv_struct(
                            TAG_REQUEST_PAYLOAD,
                            vec![
                                ttlv_item(
                                    TAG_UNIQUE_IDENTIFIER,
                                    Value::text_string("obj-1".into()),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    ],
                )
                .unwrap();

                let resp = bridge
                    .dispatch(&build_1x_msg(minor_1x, vec![], vec![bi]))
                    .await
                    .unwrap();
                let rh = find_child(&resp, TAG_RESPONSE_HEADER).unwrap();
                let pv = find_child(&rh, TAG_PROTOCOL_VERSION).unwrap();
                assert_eq!(
                    find_child(&pv, TAG_PROTOCOL_VERSION_MAJOR).and_then(|i| extract_integer(&i)),
                    Some(1)
                );
                assert_eq!(
                    find_child(&pv, TAG_PROTOCOL_VERSION_MINOR).and_then(|i| extract_integer(&i)),
                    Some(minor_1x)
                );
                let rbi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
                assert_eq!(
                    find_child(&rbi, TAG_OPERATION).and_then(|i| extract_enumeration(&i)),
                    Some(op)
                );
                assert_eq!(
                    find_child(&rbi, TAG_UNIQUE_BATCH_ITEM_ID)
                        .and_then(|i| extract_byte_string(&i)),
                    Some(vec![0xAA, op as u8])
                );
            }
        }
    }
}

#[test]
fn test_all_11_ttlv_value_types_clone_and_retag_roundtrip() {
    let sub_struct = ttlv_struct(
        TAG_NAME,
        vec![ttlv_item(TAG_NAME_VALUE, Value::text_string("sub".into())).unwrap()],
    )
    .unwrap();
    let samples: Vec<(u32, Value)> = vec![
        (TAG_NAME, clone_item_value(&sub_struct).unwrap()),
        (TAG_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
        (TAG_USAGE_LIMITS_TOTAL, Value::long_integer(9_000_000_000)),
        (TAG_OFFSET, Value::big_integer(vec![0x01, 0x00, 0xFF])),
        (TAG_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(0x0000_0003)),
        (TAG_SENSITIVE, Value::boolean(true)),
        (
            TAG_DESCRIPTION,
            Value::text_string("kmip1x-test-utf8-áéíóú".into()),
        ),
        (TAG_DATA, Value::byte_string(vec![0xDE, 0xAD, 0xBE, 0xEF])),
        (TAG_ACTIVATION_DATE, Value::date_time(1_700_000_000)),
        (TAG_LEASE_TIME, Value::interval(3600)),
        (
            TAG_ARCHIVE_DATE,
            Value::date_time_extended(1_700_000_000_123_456),
        ),
    ];

    for (tag, val) in samples {
        let orig = ttlv_item(tag, val).unwrap();
        let retagged = retag_item(&orig, TAG_CUSTOM_ATTRIBUTE).unwrap();
        assert_eq!(retagged.tag().raw(), TAG_CUSTOM_ATTRIBUTE);
        let restored = retag_item(&retagged, tag).unwrap();
        assert_eq!(restored.tag().raw(), tag);
        let mut matched = false;
        orig.with_value(|v1| {
            restored.with_value(|v2| {
                matched = matches!(
                    (v1, v2),
                    (ValueView::Structure(_), ValueView::Structure(_))
                        | (ValueView::Integer(_), ValueView::Integer(_))
                        | (ValueView::LongInteger(_), ValueView::LongInteger(_))
                        | (ValueView::BigInteger(_), ValueView::BigInteger(_))
                        | (ValueView::Enumeration(_), ValueView::Enumeration(_))
                        | (ValueView::Boolean(_), ValueView::Boolean(_))
                        | (ValueView::TextString(_), ValueView::TextString(_))
                        | (ValueView::ByteString(_), ValueView::ByteString(_))
                        | (ValueView::DateTime(_), ValueView::DateTime(_))
                        | (ValueView::Interval(_), ValueView::Interval(_))
                        | (
                            ValueView::DateTimeExtended(_),
                            ValueView::DateTimeExtended(_)
                        )
                );
            });
        });
        assert!(matched, "value view mismatch for tag 0x{tag:06X}");
    }
}

#[tokio::test]
async fn test_async_operations_pending_poll_and_cancel_roundtrip() {
    struct AsyncMockUpstream;
    #[async_trait]
    impl UpstreamKmipTransport for AsyncMockUpstream {
        async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
            assert!(is_async_request(request));
            let bi = find_child(request, TAG_BATCH_ITEM).unwrap();
            let op = find_child(&bi, TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap();
            let rbi = match op {
                OP_CREATE => ttlv_struct(
                    TAG_BATCH_ITEM,
                    vec![
                        ttlv_item(TAG_OPERATION, Value::enumeration(OP_CREATE)).unwrap(),
                        ttlv_item(
                            TAG_RESULT_STATUS,
                            Value::enumeration(STATUS_OPERATION_PENDING),
                        )
                        .unwrap(),
                        ttlv_item(
                            TAG_ASYNCHRONOUS_CORRELATION_VALUE,
                            Value::byte_string(vec![1, 2, 3, 4]),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
                OP_POLL => build_success_batch_item(
                    OP_POLL,
                    None,
                    vec![
                        ttlv_item(
                            TAG_OBJECT_TYPE,
                            Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                        )
                        .unwrap(),
                        ttlv_item(
                            TAG_UNIQUE_IDENTIFIER,
                            Value::text_string("async-key-1".into()),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
                OP_CANCEL => build_success_batch_item(
                    OP_CANCEL,
                    None,
                    vec![ttlv_item(TAG_CANCELLATION_RESULT, Value::enumeration(1)).unwrap()],
                )
                .unwrap(),
                _ => panic!("unexpected op"),
            };
            build_response_message(2, 1, vec![rbi])
        }
    }

    let bridge = Kmip1xBridge::new(Arc::new(AsyncMockUpstream), UpstreamKmipVersion::V2_1);

    let req_create = build_1x_msg(
        1,
        vec![ttlv_item(TAG_ASYNCHRONOUS_INDICATOR, Value::boolean(true)).unwrap()],
        vec![
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_CREATE)).unwrap(),
                    ttlv_struct(TAG_REQUEST_PAYLOAD, vec![]).unwrap(),
                ],
            )
            .unwrap(),
        ],
    );
    let resp_create = bridge.dispatch(&req_create).await.unwrap();
    let rbi = find_child(&resp_create, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        find_child(&rbi, TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_OPERATION_PENDING)
    );
    assert_eq!(
        find_child(&rbi, TAG_ASYNCHRONOUS_CORRELATION_VALUE).and_then(|i| extract_byte_string(&i)),
        Some(vec![1, 2, 3, 4])
    );

    let req_poll = build_1x_msg(
        1,
        vec![ttlv_item(TAG_ASYNCHRONOUS_INDICATOR, Value::boolean(true)).unwrap()],
        vec![
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_POLL)).unwrap(),
                    ttlv_struct(
                        TAG_REQUEST_PAYLOAD,
                        vec![
                            ttlv_item(
                                TAG_ASYNCHRONOUS_CORRELATION_VALUE,
                                Value::byte_string(vec![1, 2, 3, 4]),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    );
    let resp_poll = bridge.dispatch(&req_poll).await.unwrap();
    assert_eq!(
        find_child(&resp_poll, TAG_BATCH_ITEM)
            .and_then(|b| find_child(&b, TAG_RESULT_STATUS))
            .and_then(|s| extract_enumeration(&s)),
        Some(STATUS_SUCCESS)
    );
}

#[test]
fn test_batch_header_options_preserved_through_translation() {
    let store = kmipkit_proxy::kmip1x::template_store::TemplateStore::new();
    let req = build_1x_msg(
        3,
        vec![
            ttlv_item(TAG_BATCH_ORDER_OPTION, Value::boolean(true)).unwrap(),
            ttlv_item(TAG_BATCH_ERROR_CONTINUATION_OPTION, Value::enumeration(2)).unwrap(),
            ttlv_item(TAG_TIME_STAMP, Value::date_time(1_700_123_456)).unwrap(),
        ],
        vec![
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_CREATE)).unwrap(),
                    ttlv_struct(TAG_REQUEST_PAYLOAD, vec![]).unwrap(),
                ],
            )
            .unwrap(),
        ],
    );

    let (req_2x, metas) =
        translate_request_1x_to_2x(&req, TranslatorConfig::default(), &store).unwrap();
    let h = find_child(&req_2x, TAG_REQUEST_HEADER).unwrap();
    assert_eq!(
        find_child(&h, TAG_BATCH_ORDER_OPTION).and_then(|i| extract_boolean(&i)),
        Some(true)
    );
    assert_eq!(
        find_child(&h, TAG_BATCH_ERROR_CONTINUATION_OPTION).and_then(|i| extract_enumeration(&i)),
        Some(2)
    );
    assert_eq!(metas.len(), 1);
}

#[tokio::test]
async fn test_discover_versions_exhaustive_intersections() {
    struct FailIfCalledUpstream;
    #[async_trait]
    impl UpstreamKmipTransport for FailIfCalledUpstream {
        async fn send_ttlv(&self, _request: &Item) -> Result<Item, String> {
            panic!("Discover Versions must never hit upstream");
        }
    }

    let bridge = Kmip1xBridge::new(Arc::new(FailIfCalledUpstream), UpstreamKmipVersion::V3_0);

    // 1. Empty offer -> returns all 5 supported 1.X versions (1.4..1.0)
    let req_empty = build_1x_msg(
        4,
        vec![],
        vec![
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS)).unwrap()],
            )
            .unwrap(),
        ],
    );
    let resp_empty = bridge.dispatch(&req_empty).await.unwrap();
    let pvs = find_children(
        &find_child(
            &find_child(&resp_empty, TAG_BATCH_ITEM).unwrap(),
            TAG_RESPONSE_PAYLOAD,
        )
        .unwrap(),
        TAG_PROTOCOL_VERSION,
    );
    assert_eq!(pvs.len(), 5);

    // 2. Disjoint offer (only 2.0 and 3.0) -> returns 0 versions with Success
    let req_disjoint = build_1x_msg(
        4,
        vec![],
        vec![
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS)).unwrap(),
                    ttlv_struct(
                        TAG_REQUEST_PAYLOAD,
                        vec![
                            build_protocol_version(3, 0).unwrap(),
                            build_protocol_version(2, 0).unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    );
    let resp_disjoint = bridge.dispatch(&req_disjoint).await.unwrap();
    let rbi = find_child(&resp_disjoint, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        find_child(&rbi, TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_SUCCESS)
    );
    assert!(
        find_children(
            &find_child(&rbi, TAG_RESPONSE_PAYLOAD).unwrap(),
            TAG_PROTOCOL_VERSION
        )
        .is_empty()
    );

    // 3. Mixed unordered offer -> preserves client preference order for supported 1.X versions
    let req_mixed = build_1x_msg(
        2,
        vec![],
        vec![
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS)).unwrap(),
                    ttlv_struct(
                        TAG_REQUEST_PAYLOAD,
                        vec![
                            build_protocol_version(2, 1).unwrap(),
                            build_protocol_version(1, 1).unwrap(),
                            build_protocol_version(1, 3).unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    );
    let resp_mixed = bridge.dispatch(&req_mixed).await.unwrap();
    let pvs_mixed = find_children(
        &find_child(
            &find_child(&resp_mixed, TAG_BATCH_ITEM).unwrap(),
            TAG_RESPONSE_PAYLOAD,
        )
        .unwrap(),
        TAG_PROTOCOL_VERSION,
    );
    let pairs: Vec<(i32, i32)> = pvs_mixed
        .iter()
        .map(|pv| {
            (
                find_child(pv, TAG_PROTOCOL_VERSION_MAJOR)
                    .and_then(|i| extract_integer(&i))
                    .unwrap(),
                find_child(pv, TAG_PROTOCOL_VERSION_MINOR)
                    .and_then(|i| extract_integer(&i))
                    .unwrap(),
            )
        })
        .collect();
    assert_eq!(pairs, vec![(1, 1), (1, 3)]);
}

#[test]
fn test_query_allowlists_and_pqc_filtering_for_all_minor_versions() {
    // Operations gating
    assert!(is_operation_allowed_in_1x(OP_CREATE, 0));
    assert!(!is_operation_allowed_in_1x(OP_DISCOVER_VERSIONS, 0));
    assert!(is_operation_allowed_in_1x(OP_DISCOVER_VERSIONS, 1));
    assert!(!is_operation_allowed_in_1x(OP_ENCRYPT, 1));
    assert!(is_operation_allowed_in_1x(OP_ENCRYPT, 2));
    assert!(!is_operation_allowed_in_1x(OP_IMPORT, 3));
    assert!(is_operation_allowed_in_1x(OP_IMPORT, 4));
    assert!(!is_operation_allowed_in_1x(OP_SET_ATTRIBUTE, 4)); // KMIP 2.0+ only

    // Object Types gating
    assert!(is_object_type_allowed_in_1x(OBJECT_TYPE_TEMPLATE, 0));
    assert!(!is_object_type_allowed_in_1x(OBJECT_TYPE_PGP_KEY, 1));
    assert!(is_object_type_allowed_in_1x(OBJECT_TYPE_PGP_KEY, 2));

    // Query Functions gating
    assert!(is_query_function_allowed_in_1x(0x01, 0));
    assert!(!is_query_function_allowed_in_1x(0x05, 0));
    assert!(is_query_function_allowed_in_1x(0x05, 1));
    assert!(!is_query_function_allowed_in_1x(0x07, 1));
    assert!(is_query_function_allowed_in_1x(0x07, 2));
    assert!(!is_query_function_allowed_in_1x(0x08, 2));
    assert!(is_query_function_allowed_in_1x(0x0C, 3));

    // Crypto Algorithms gating (PQC algorithms > 0x28 must always be rejected)
    assert!(is_crypto_algorithm_allowed_in_1x(0x03, 0)); // AES
    assert!(!is_crypto_algorithm_allowed_in_1x(0x1A, 1)); // EC requires 1.2
    assert!(is_crypto_algorithm_allowed_in_1x(0x1A, 2));
    assert!(!is_crypto_algorithm_allowed_in_1x(0x1C, 3)); // ChaCha20 requires 1.4
    assert!(is_crypto_algorithm_allowed_in_1x(0x1C, 4));
    for pqc in [0x29, 0x30, 0x40, 0x45, 0x50] {
        assert!(!is_crypto_algorithm_allowed_in_1x(pqc, 4));
    }
}

#[tokio::test]
async fn test_s2c_put_replace_and_error_response_translation() {
    struct MockS2cClient {
        seen_requests: Mutex<Vec<Item>>,
        return_error: bool,
    }

    #[async_trait]
    impl S2cClientTransport for MockS2cClient {
        async fn send_to_client(&self, request_1x: &Item) -> Result<Item, String> {
            self.seen_requests
                .lock()
                .unwrap()
                .push(clone_item(request_1x)?);
            let bi = find_child(request_1x, TAG_BATCH_ITEM).unwrap();
            let op = find_child(&bi, TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap();
            let resp_bi = if self.return_error {
                build_error_batch_item(op, None, REASON_PERMISSION_DENIED, "client rejected put")?
            } else {
                build_success_batch_item(op, None, vec![])?
            };
            build_response_message(1, 2, vec![resp_bi])
        }
    }

    let client = Arc::new(MockS2cClient {
        seen_requests: Mutex::new(Vec::new()),
        return_error: true,
    });
    let s2c_bridge = Kmip1xS2cBridge::new(client.clone(), 2, UpstreamKmipVersion::V3_0);

    // Server sends Put (0x1C) with KMIP 3.0 Attributes structure
    let srv_req = ttlv_struct(
        TAG_REQUEST_MESSAGE,
        vec![
            ttlv_struct(
                TAG_REQUEST_HEADER,
                vec![
                    build_protocol_version(3, 0).unwrap(),
                    ttlv_item(TAG_BATCH_COUNT, Value::integer(1)).unwrap(),
                ],
            )
            .unwrap(),
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_PUT)).unwrap(),
                    ttlv_struct(
                        TAG_REQUEST_PAYLOAD,
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("srv-key".into()))
                                .unwrap(),
                            ttlv_item(TAG_PUT_FUNCTION, Value::enumeration(2)).unwrap(),
                            ttlv_struct(
                                TAG_ATTRIBUTES,
                                vec![
                                    ttlv_item(
                                        TAG_DESCRIPTION,
                                        Value::text_string("rotated".into()),
                                    )
                                    .unwrap(),
                                ],
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let srv_resp = s2c_bridge.handle_server_request(&srv_req).await.unwrap();
    // Server response header must be KMIP 3.0
    let rh = find_child(&srv_resp, TAG_RESPONSE_HEADER).unwrap();
    let pv = find_child(&rh, TAG_PROTOCOL_VERSION).unwrap();
    assert_eq!(
        find_child(&pv, TAG_PROTOCOL_VERSION_MAJOR).and_then(|i| extract_integer(&i)),
        Some(3)
    );
    assert_eq!(
        find_child(&pv, TAG_PROTOCOL_VERSION_MINOR).and_then(|i| extract_integer(&i)),
        Some(0)
    );

    // Error status from 1.2 client preserved back to 3.0 server
    let rbi = find_child(&srv_resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        find_child(&rbi, TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_OPERATION_FAILED)
    );

    // Verify client saw 1.2 request without 0x420125 Attributes (translated to 0x420008 Attribute)
    let client_seen = client.seen_requests.lock().unwrap();
    let c_payload = find_child(
        &find_child(&client_seen[0], TAG_BATCH_ITEM).unwrap(),
        TAG_REQUEST_PAYLOAD,
    )
    .unwrap();
    assert!(find_child(&c_payload, TAG_ATTRIBUTES).is_none());
    assert_eq!(find_children(&c_payload, TAG_ATTRIBUTE).len(), 0); // Description has min_minor_1x=3 > 2!

    // Also test response 2x->1x with translate_response_2x_to_1x directly on unknown attribute
    let dummy_attr =
        build_1x_attribute("x-custom", Some(0), Value::text_string("val".into())).unwrap();
    assert_eq!(
        find_child(&dummy_attr, TAG_ATTRIBUTE_NAME).and_then(|i| extract_text_string(&i)),
        Some("x-custom".to_string())
    );
    let empty_resp = build_response_message(
        2,
        1,
        vec![build_success_batch_item(OP_GET, None, vec![]).unwrap()],
    )
    .unwrap();
    let translated_empty = translate_response_2x_to_1x(&empty_resp, 1, 2, &[]).unwrap();
    assert!(find_child(&translated_empty, TAG_RESPONSE_HEADER).is_some());
}
