//! Regression and bugfix validation tests for the KMIP 1.X bridge.
//!
//! Covers:
//! - ID Placeholder chaining across mixed orchestrated batches and RMW operations
//! - RMW custom attribute filtering across multiple distinct `x-*`/`y-*` names
//! - RMW `Link` modification/deletion against KMIP 3.0 typed link upstreams
//! - `Get Attribute List` deduplication of KMIP 3.0 typed links and KMIP 1.0 aliases
//! - `TemplateStore` duplicate name overwrite/destroy safety and 64-thread concurrency
//! - `Operation Policy Name = Reject` inside orchestrated batches
//! - `Maximum Response Size` zero/negative/disabled edge cases
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::thread;

use async_trait::async_trait;
use kmipkit_proxy::kmip1x::mapping::{
    OperationPolicyHandling, UpstreamKmipVersion, link_type_to_v3_tag,
};
use kmipkit_proxy::kmip1x::tags::*;
use kmipkit_proxy::kmip1x::template_store::TemplateStore;
use kmipkit_proxy::kmip1x::translator::{
    TranslatorConfig, build_1x_attribute, extract_max_response_size,
};
use kmipkit_proxy::kmip1x::ttlv_helpers::{
    build_protocol_version, build_response_message, build_success_batch_item, children_of,
    clone_item, extract_enumeration, extract_integer, extract_text_string, find_child,
    find_children, ttlv_item, ttlv_struct,
};
use kmipkit_proxy::kmip1x::{Kmip1xBridge, UpstreamKmipTransport};
use kmipkit_ttlv::{Item, Value};

type ResponderFn = Box<dyn Fn(&Item) -> Result<Item, String> + Send + Sync>;

#[derive(Default)]
struct RecordingUpstream {
    requests: Mutex<Vec<Item>>,
    responder: Mutex<Option<ResponderFn>>,
}

impl RecordingUpstream {
    fn with_responder<F>(f: F) -> Arc<Self>
    where
        F: Fn(&Item) -> Result<Item, String> + Send + Sync + 'static,
    {
        Arc::new(Self {
            requests: Mutex::new(Vec::new()),
            responder: Mutex::new(Some(Box::new(f))),
        })
    }

    fn captured_requests(&self) -> Vec<Item> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .map(|i| clone_item(i).unwrap())
            .collect()
    }
}

#[async_trait]
impl UpstreamKmipTransport for RecordingUpstream {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        self.requests.lock().unwrap().push(clone_item(request)?);
        let guard = self.responder.lock().unwrap();
        if let Some(ref f) = *guard {
            f(request)
        } else {
            Err("no responder configured".to_string())
        }
    }
}

fn build_1x_request(major: i32, minor: i32, batch_items: Vec<Item>) -> Item {
    let header = ttlv_struct(
        TAG_REQUEST_HEADER,
        vec![
            build_protocol_version(major, minor).unwrap(),
            ttlv_item(TAG_BATCH_COUNT, Value::integer(batch_items.len() as i32)).unwrap(),
        ],
    )
    .unwrap();
    let mut top = vec![header];
    top.extend(batch_items);
    ttlv_struct(TAG_REQUEST_MESSAGE, top).unwrap()
}

fn build_1x_request_with_max_size(
    major: i32,
    minor: i32,
    max_size: i32,
    batch_items: Vec<Item>,
) -> Item {
    let header = ttlv_struct(
        TAG_REQUEST_HEADER,
        vec![
            build_protocol_version(major, minor).unwrap(),
            ttlv_item(TAG_MAXIMUM_RESPONSE_SIZE, Value::integer(max_size)).unwrap(),
            ttlv_item(TAG_BATCH_COUNT, Value::integer(batch_items.len() as i32)).unwrap(),
        ],
    )
    .unwrap();
    let mut top = vec![header];
    top.extend(batch_items);
    ttlv_struct(TAG_REQUEST_MESSAGE, top).unwrap()
}

#[tokio::test]
async fn test_orchestrated_batch_chains_id_placeholder_to_concrete_uid_and_rmw() {
    let upstream = RecordingUpstream::with_responder(|req| {
        let bi = find_child(req, TAG_BATCH_ITEM).unwrap();
        let op = find_child(&bi, TAG_OPERATION)
            .and_then(|i| extract_enumeration(&i))
            .unwrap();
        let payload = find_child(&bi, TAG_REQUEST_PAYLOAD).unwrap();

        let resp_bi = match op {
            OP_CREATE => build_success_batch_item(
                OP_CREATE,
                None,
                vec![
                    ttlv_item(
                        TAG_OBJECT_TYPE,
                        Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                    )
                    .unwrap(),
                    ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string("key-chained-1".into()),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
            OP_ACTIVATE => {
                // Must have received the concrete UID from Create, NOT ID_PLACEHOLDER enum!
                let uid_item = find_child(&payload, TAG_UNIQUE_IDENTIFIER).unwrap();
                assert_eq!(
                    extract_text_string(&uid_item).as_deref(),
                    Some("key-chained-1")
                );
                build_success_batch_item(
                    OP_ACTIVATE,
                    None,
                    vec![
                        ttlv_item(
                            TAG_UNIQUE_IDENTIFIER,
                            Value::text_string("key-chained-1".into()),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap()
            }
            OP_GET_ATTRIBUTES => {
                let uid_item = find_child(&payload, TAG_UNIQUE_IDENTIFIER).unwrap();
                assert_eq!(
                    extract_text_string(&uid_item).as_deref(),
                    Some("key-chained-1")
                );
                build_success_batch_item(
                    OP_GET_ATTRIBUTES,
                    None,
                    vec![
                        ttlv_item(
                            TAG_UNIQUE_IDENTIFIER,
                            Value::text_string("key-chained-1".into()),
                        )
                        .unwrap(),
                        ttlv_struct(
                            TAG_ATTRIBUTES,
                            vec![
                                ttlv_item(TAG_OBJECT_GROUP, Value::text_string("grp-0".into()))
                                    .unwrap(),
                            ],
                        )
                        .unwrap(),
                    ],
                )
                .unwrap()
            }
            OP_MODIFY_ATTRIBUTE => {
                let uid_item = find_child(&payload, TAG_UNIQUE_IDENTIFIER).unwrap();
                assert_eq!(
                    extract_text_string(&uid_item).as_deref(),
                    Some("key-chained-1")
                );
                build_success_batch_item(
                    OP_MODIFY_ATTRIBUTE,
                    None,
                    vec![
                        ttlv_item(
                            TAG_UNIQUE_IDENTIFIER,
                            Value::text_string("key-chained-1".into()),
                        )
                        .unwrap(),
                        ttlv_struct(
                            TAG_ATTRIBUTES,
                            vec![
                                ttlv_item(
                                    TAG_OBJECT_GROUP,
                                    Value::text_string("grp-updated".into()),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    ],
                )
                .unwrap()
            }
            other => panic!("unexpected upstream op: 0x{other:X}"),
        };
        build_response_message(2, 1, vec![resp_bi])
    });

    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    // Mixed orchestrated batch: Discover Versions + Create + Activate (no UID) + ModifyAttribute RMW (no UID)
    let bi_discover = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS)).unwrap()],
    )
    .unwrap();
    let bi_create = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(OP_CREATE)).unwrap(),
            ttlv_struct(
                TAG_REQUEST_PAYLOAD,
                vec![
                    ttlv_item(
                        TAG_OBJECT_TYPE,
                        Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let bi_activate = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(OP_ACTIVATE)).unwrap(),
            ttlv_struct(TAG_REQUEST_PAYLOAD, vec![]).unwrap(),
        ],
    )
    .unwrap();
    let bi_modify_rmw = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(OP_MODIFY_ATTRIBUTE)).unwrap(),
            ttlv_struct(
                TAG_REQUEST_PAYLOAD,
                vec![
                    build_1x_attribute(
                        "Object Group",
                        Some(0),
                        Value::text_string("grp-updated".into()),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let req = build_1x_request(
        1,
        4,
        vec![bi_discover, bi_create, bi_activate, bi_modify_rmw],
    );
    let resp = bridge.dispatch(&req).await.unwrap();
    let resp_items = find_children(&resp, TAG_BATCH_ITEM);
    assert_eq!(resp_items.len(), 4);
    for (idx, bi) in resp_items.iter().enumerate() {
        let status = find_child(bi, TAG_RESULT_STATUS)
            .and_then(|i| extract_enumeration(&i))
            .unwrap();
        assert_eq!(status, STATUS_SUCCESS, "batch item {idx} failed");
    }
    assert_eq!(upstream.captured_requests().len(), 4); // Create + Activate + GetAttributes + ModifyAttribute
}

#[tokio::test]
async fn test_rmw_custom_attributes_filters_by_exact_vendor_and_name() {
    let upstream = RecordingUpstream::with_responder(|req| {
        let bi = find_child(req, TAG_BATCH_ITEM).unwrap();
        let op = find_child(&bi, TAG_OPERATION)
            .and_then(|i| extract_enumeration(&i))
            .unwrap();
        let payload = find_child(&bi, TAG_REQUEST_PAYLOAD).unwrap();

        let resp_bi = match op {
            OP_GET_ATTRIBUTES => {
                // Return multiple different custom attributes; x-alpha appears AFTER x-beta
                let attr_beta = ttlv_struct(
                    TAG_ATTRIBUTE,
                    vec![
                        ttlv_item(TAG_VENDOR_IDENTIFICATION, Value::text_string("x".into()))
                            .unwrap(),
                        ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("beta".into())).unwrap(),
                        ttlv_item(TAG_ATTRIBUTE_VALUE, Value::text_string("beta-val".into()))
                            .unwrap(),
                    ],
                )
                .unwrap();
                let attr_alpha_0 = ttlv_struct(
                    TAG_ATTRIBUTE,
                    vec![
                        ttlv_item(TAG_VENDOR_IDENTIFICATION, Value::text_string("x".into()))
                            .unwrap(),
                        ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("alpha".into())).unwrap(),
                        ttlv_item(TAG_ATTRIBUTE_VALUE, Value::text_string("alpha-0".into()))
                            .unwrap(),
                    ],
                )
                .unwrap();
                let attr_alpha_1 = ttlv_struct(
                    TAG_ATTRIBUTE,
                    vec![
                        ttlv_item(TAG_VENDOR_IDENTIFICATION, Value::text_string("x".into()))
                            .unwrap(),
                        ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("alpha".into())).unwrap(),
                        ttlv_item(TAG_ATTRIBUTE_VALUE, Value::text_string("alpha-1".into()))
                            .unwrap(),
                    ],
                )
                .unwrap();

                build_success_batch_item(
                    OP_GET_ATTRIBUTES,
                    None,
                    vec![
                        ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-1".into())).unwrap(),
                        ttlv_struct(TAG_ATTRIBUTES, vec![attr_beta, attr_alpha_0, attr_alpha_1])
                            .unwrap(),
                    ],
                )
                .unwrap()
            }
            OP_MODIFY_ATTRIBUTE => {
                let curr = find_child(&payload, TAG_CURRENT_ATTRIBUTE).unwrap();
                let inner = children_of(&curr).into_iter().next().unwrap();
                let name = find_child(&inner, TAG_ATTRIBUTE_NAME)
                    .and_then(|i| extract_text_string(&i))
                    .unwrap();
                let val = find_child(&inner, TAG_ATTRIBUTE_VALUE)
                    .and_then(|i| extract_text_string(&i))
                    .unwrap();
                assert_eq!(name, "alpha");
                assert_eq!(val, "alpha-1");
                build_success_batch_item(
                    OP_MODIFY_ATTRIBUTE,
                    None,
                    vec![
                        ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-1".into())).unwrap(),
                    ],
                )
                .unwrap()
            }
            other => panic!("unexpected op 0x{other:X}"),
        };
        build_response_message(2, 1, vec![resp_bi])
    });

    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V2_1);
    let bi = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(OP_MODIFY_ATTRIBUTE)).unwrap(),
            ttlv_struct(
                TAG_REQUEST_PAYLOAD,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-1".into())).unwrap(),
                    build_1x_attribute("x-alpha", Some(1), Value::text_string("alpha-new".into()))
                        .unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let resp = bridge
        .dispatch(&build_1x_request(1, 4, vec![bi]))
        .await
        .unwrap();
    let rbi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        find_child(&rbi, TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_SUCCESS)
    );
}

#[tokio::test]
async fn test_rmw_link_attribute_against_kmip30_typed_links() {
    let upstream = RecordingUpstream::with_responder(|req| {
        let bi = find_child(req, TAG_BATCH_ITEM).unwrap();
        let op = find_child(&bi, TAG_OPERATION)
            .and_then(|i| extract_enumeration(&i))
            .unwrap();
        let payload = find_child(&bi, TAG_REQUEST_PAYLOAD).unwrap();

        let resp_bi = match op {
            OP_GET_ATTRIBUTES => build_success_batch_item(
                OP_GET_ATTRIBUTES,
                None,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-v3".into())).unwrap(),
                    ttlv_struct(
                        TAG_ATTRIBUTES,
                        vec![
                            ttlv_item(
                                link_type_to_v3_tag(LINK_TYPE_CERTIFICATE).unwrap(),
                                Value::text_string("cert-0".into()),
                            )
                            .unwrap(),
                            ttlv_item(
                                link_type_to_v3_tag(LINK_TYPE_PUBLIC_KEY).unwrap(),
                                Value::text_string("pub-1".into()),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
            OP_DELETE_ATTRIBUTE => {
                let curr = find_child(&payload, TAG_CURRENT_ATTRIBUTE).unwrap();
                let inner = children_of(&curr).into_iter().next().unwrap();
                assert_eq!(
                    inner.tag().raw(),
                    link_type_to_v3_tag(LINK_TYPE_PUBLIC_KEY).unwrap()
                );
                assert_eq!(extract_text_string(&inner).as_deref(), Some("pub-1"));
                build_success_batch_item(
                    OP_DELETE_ATTRIBUTE,
                    None,
                    vec![
                        ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-v3".into()))
                            .unwrap(),
                    ],
                )
                .unwrap()
            }
            other => panic!("unexpected op 0x{other:X}"),
        };
        build_response_message(3, 0, vec![resp_bi])
    });

    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V3_0);
    let bi = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(OP_DELETE_ATTRIBUTE)).unwrap(),
            ttlv_struct(
                TAG_REQUEST_PAYLOAD,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-v3".into())).unwrap(),
                    ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Link".into())).unwrap(),
                    ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(1)).unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let resp = bridge
        .dispatch(&build_1x_request(1, 4, vec![bi]))
        .await
        .unwrap();
    let rbi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        find_child(&rbi, TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_SUCCESS)
    );
}

#[tokio::test]
async fn test_get_attribute_list_deduplicates_v3_links_and_maps_kmip10_x509_aliases() {
    let upstream = RecordingUpstream::with_responder(|_req| {
        let bi = build_success_batch_item(
            OP_GET_ATTRIBUTE_LIST,
            None,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("cert-1".into())).unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_REFERENCE,
                    Value::enumeration(TAG_CERTIFICATE_LINK),
                )
                .unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_REFERENCE,
                    Value::enumeration(TAG_PUBLIC_KEY_LINK),
                )
                .unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_REFERENCE,
                    Value::enumeration(TAG_X509_CERTIFICATE_SUBJECT),
                )
                .unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_REFERENCE,
                    Value::enumeration(TAG_X509_CERTIFICATE_ISSUER),
                )
                .unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_REFERENCE,
                    Value::enumeration(TAG_X509_CERTIFICATE_IDENTIFIER),
                )
                .unwrap(),
            ],
        )
        .unwrap();
        build_response_message(3, 0, vec![bi])
    });

    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V3_0);
    let bi = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(OP_GET_ATTRIBUTE_LIST)).unwrap(),
            ttlv_struct(
                TAG_REQUEST_PAYLOAD,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("cert-1".into())).unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();

    // Test with KMIP 1.0 client
    let resp = bridge
        .dispatch(&build_1x_request(1, 0, vec![bi]))
        .await
        .unwrap();
    let rbi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    let rp = find_child(&rbi, TAG_RESPONSE_PAYLOAD).unwrap();
    let names: Vec<String> = find_children(&rp, TAG_ATTRIBUTE_NAME)
        .into_iter()
        .filter_map(|i| extract_text_string(&i))
        .collect();

    assert_eq!(
        names,
        vec![
            "Link",
            "Certificate Subject",
            "Certificate Issuer",
            "Certificate Identifier"
        ]
    );
}

#[test]
fn test_template_store_duplicate_name_overwrite_and_concurrent_stress() {
    let store = Arc::new(TemplateStore::new());
    let id1 = store.register(vec!["shared-name".into()], vec![]);
    let id2 = store.register(vec!["shared-name".into()], vec![]);
    assert_ne!(id1, id2);
    assert_eq!(store.get_by_name("shared-name").unwrap().unique_id, id2);

    // Destroying id1 must NOT remove "shared-name" pointing to id2
    assert!(store.destroy(&id1));
    assert_eq!(store.get_by_name("shared-name").unwrap().unique_id, id2);
    assert!(store.destroy(&id2));
    assert!(store.get_by_name("shared-name").is_none());

    // 64-thread concurrent register/locate/get/destroy stress
    let mut handles = Vec::new();
    for t in 0..64 {
        let s = Arc::clone(&store);
        handles.push(thread::spawn(move || {
            let name = format!("concurrent-tpl-{t}");
            let id = s.register(vec![name.clone()], vec![]);
            assert!(s.contains_id(&id));
            assert_eq!(s.get_by_name(&name).unwrap().unique_id, id);
            assert!(s.locate(Some(&name)).contains(&id));
            assert!(s.destroy(&id));
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert!(store.locate(None).is_empty());
}

#[tokio::test]
async fn test_reject_operation_policy_inside_orchestrated_batch_returns_error_item() {
    let upstream = RecordingUpstream::with_responder(|_req| {
        panic!("upstream must not be called for local/rejected items");
    });

    let bridge = Kmip1xBridge::with_config(
        upstream,
        TranslatorConfig {
            upstream_version: UpstreamKmipVersion::V2_1,
            operation_policy_handling: OperationPolicyHandling::Reject,
            ..TranslatorConfig::default()
        },
    );

    let bi_discover = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS)).unwrap()],
    )
    .unwrap();
    let bi_create_rejected = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(OP_CREATE)).unwrap(),
            ttlv_struct(
                TAG_REQUEST_PAYLOAD,
                vec![
                    ttlv_item(
                        TAG_OBJECT_TYPE,
                        Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                    )
                    .unwrap(),
                    ttlv_struct(
                        TAG_TEMPLATE_ATTRIBUTE,
                        vec![
                            build_1x_attribute(
                                "Operation Policy Name",
                                None,
                                Value::text_string("strict-policy".into()),
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

    let resp = bridge
        .dispatch(&build_1x_request(
            1,
            4,
            vec![bi_discover, bi_create_rejected],
        ))
        .await
        .unwrap();
    let items = find_children(&resp, TAG_BATCH_ITEM);
    assert_eq!(items.len(), 2);
    assert_eq!(
        find_child(&items[0], TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_SUCCESS)
    );
    assert_eq!(
        find_child(&items[1], TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_OPERATION_FAILED)
    );
    assert_eq!(
        find_child(&items[1], TAG_RESULT_REASON).and_then(|i| extract_enumeration(&i)),
        Some(REASON_OPERATION_NOT_SUPPORTED)
    );
}

#[tokio::test]
async fn test_max_response_size_zero_negative_and_disabled() {
    let req_zero = build_1x_request_with_max_size(1, 4, 0, vec![]);
    assert_eq!(extract_max_response_size(&req_zero), None);

    let req_neg = build_1x_request_with_max_size(1, 4, -100, vec![]);
    assert_eq!(extract_max_response_size(&req_neg), None);

    let req_pos = build_1x_request_with_max_size(1, 4, 10, vec![]);
    assert_eq!(extract_max_response_size(&req_pos), Some(10));

    // When enforce_max_response_size = false, even tiny limit 10 succeeds
    let upstream = RecordingUpstream::with_responder(|_req| {
        let bi = build_success_batch_item(
            OP_DISCOVER_VERSIONS,
            None,
            vec![build_protocol_version(1, 4).unwrap()],
        )
        .unwrap();
        build_response_message(2, 1, vec![bi])
    });
    let bridge = Kmip1xBridge::with_config(
        upstream,
        TranslatorConfig {
            enforce_max_response_size: false,
            ..TranslatorConfig::default()
        },
    );
    let bi = ttlv_struct(
        TAG_BATCH_ITEM,
        vec![ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS)).unwrap()],
    )
    .unwrap();
    let resp = bridge
        .dispatch(&build_1x_request_with_max_size(1, 4, 10, vec![bi]))
        .await
        .unwrap();
    let rbi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        find_child(&rbi, TAG_RESULT_STATUS).and_then(|i| extract_enumeration(&i)),
        Some(STATUS_SUCCESS)
    );
    assert_eq!(
        find_child(&resp, TAG_RESPONSE_HEADER)
            .and_then(|h| find_child(&h, TAG_BATCH_COUNT))
            .and_then(|b| extract_integer(&b)),
        Some(1)
    );
}
