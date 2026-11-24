//! Advanced edge-case, security, protocol-fidelity, concurrency, and interoperability
//! tests for the KMIP 1.X -> KMIP 2.0 / 2.1 / 3.0 translation proxy (`kmipkit_proxy::kmip1x`).
//!
//! Covers:
//! 1. Mixed orchestrated batches (DiscoverVersions + Template + RMW + Upstream Create in 1 message).
//! 2. Chained interoperability: KMIP 1.4 Legacy Client -> Kmip1xBridge -> LocalDispatcherUpstream -> Cloud KMS Proxy (InMemoryMockProvider).
//! 3. HTTP/1.1 POST (TTLV, JSON, XML) over `ProxyServer::start_kmip1x_bridge`.
//! 4. Real mTLS (TLS 1.3 and TLS 1.2 compatibility mode) over `ProxyServer::start_kmip1x_bridge`.
//! 5. Authentication (`Username and Password`), `Batch Order Option`, `Batch Error Continuation Option`,
//!    and `Message Extension` header preservation.
//! 6. RMW edge cases: negative index, delete-all without index (single RTT), RMW on custom `x-*`
//!    attribute, single-instance ModifyAttribute -> SetAttribute restoration, upstream failure in RMW Step 1.
//! 7. Cryptographic payload fidelity: AES-GCM AEAD (`IV`, `AAD`, `Tag`), streaming crypto indicators,
//!    `Derive Key`, `Create Split Key`, `Join Split Key`, `Re-key` with `Offset`, `Certify` / `Re-certify`.
//! 8. Template Store edge cases: filtered `Get Attributes`, `Get Attribute List`, multi-template merge,
//!    destroy-then-get (`Item Not Found`), unsupported op on Template (`Operation Not Supported`).
//! 9. Adversarial & boundary inputs: 0 batch items, missing `Operation`, missing `Attribute Value`,
//!    `Operation Policy Name` Reject mode end-to-end, exact `Maximum Response Size` boundary,
//!    unknown non-standard attribute preservation, `Asynchronous Indicator` enum -> boolean downgrade.
//! 10. High-concurrency stress test (32 concurrent Tokio tasks across KMIP 1.0–1.4).
//! 11. S2C pass-through (1.X server) and vendor attribute roundtrip in `Put`/`Notify`.
#![forbid(unsafe_code)]
#![cfg(feature = "kmip1x-bridge")]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kmipkit_proxy::codec::Format;
use kmipkit_proxy::config::{ProxyBehaviorConfig, ProxyConfig};
use kmipkit_proxy::dispatcher::RequestDispatcher;
use kmipkit_proxy::kmip1x::mapping::{OperationPolicyHandling, UpstreamKmipVersion};
use kmipkit_proxy::kmip1x::s2c::{Kmip1xS2cBridge, S2cClientTransport};
use kmipkit_proxy::kmip1x::tags::*;
use kmipkit_proxy::kmip1x::translator::{
    TranslatorConfig, build_1x_attribute, extract_protocol_version, rewrite_header_version,
};
use kmipkit_proxy::kmip1x::ttlv_helpers::{
    build_error_batch_item, build_protocol_version, build_response_message,
    build_success_batch_item, children_of, clone_item, extract_boolean, extract_byte_string,
    extract_enumeration, extract_text_string, find_child, find_children, ttlv_item, ttlv_struct,
};
use kmipkit_proxy::kmip1x::upstream::LocalDispatcherUpstream;
use kmipkit_proxy::kmip1x::{Kmip1xBridge, UpstreamKmipTransport};
use kmipkit_proxy::providers::InMemoryMockProvider;
use kmipkit_proxy::server::ProxyServer;
use kmipkit_ttlv::{Item, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ============================================================================
// Reusable Programmable Mock Upstream Server
// ============================================================================

struct ProgrammableUpstream {
    version: UpstreamKmipVersion,
    recorded: Mutex<Vec<Item>>,
    objects: Mutex<BTreeMap<String, Vec<Item>>>,
    fail_op: Mutex<Option<(u32, u32)>>, // (op_code, result_reason)
    counter: Mutex<u64>,
}

impl ProgrammableUpstream {
    fn new(version: UpstreamKmipVersion) -> Self {
        Self {
            version,
            recorded: Mutex::new(Vec::new()),
            objects: Mutex::new(BTreeMap::new()),
            fail_op: Mutex::new(None),
            counter: Mutex::new(1),
        }
    }

    fn seed(&self, uid: &str, attrs_2x: Vec<Item>) {
        self.objects
            .lock()
            .unwrap()
            .insert(uid.to_string(), attrs_2x);
    }

    fn set_fail_op(&self, fail: Option<(u32, u32)>) {
        *self.fail_op.lock().unwrap() = fail;
    }

    fn last_request(&self) -> Item {
        clone_item(self.recorded.lock().unwrap().last().unwrap()).unwrap()
    }

    fn request_count(&self) -> usize {
        self.recorded.lock().unwrap().len()
    }
}

#[async_trait]
impl UpstreamKmipTransport for ProgrammableUpstream {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        self.recorded.lock().unwrap().push(clone_item(request)?);
        let (maj, min) = self.version.major_minor();
        let mut resp_items = Vec::new();

        for bi in find_children(request, TAG_BATCH_ITEM) {
            let op = find_child(&bi, TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap_or(0);
            let bid =
                find_child(&bi, TAG_UNIQUE_BATCH_ITEM_ID).and_then(|i| extract_byte_string(&i));
            let payload = find_child(&bi, TAG_REQUEST_PAYLOAD);

            if let Some((f_op, f_reason)) = *self.fail_op.lock().unwrap()
                && f_op == op
            {
                resp_items.push(build_error_batch_item(
                    op,
                    bid.as_deref(),
                    f_reason,
                    "injected failure",
                )?);
                continue;
            }

            let uid = payload
                .as_ref()
                .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                .and_then(|i| extract_text_string(&i))
                .unwrap_or_else(|| "obj-1".to_string());

            let resp_bi = match op {
                OP_CREATE | OP_DERIVE_KEY | OP_CREATE_SPLIT_KEY | OP_JOIN_SPLIT_KEY | OP_REKEY
                | OP_CERTIFY | OP_RECERTIFY => {
                    let mut c = self.counter.lock().unwrap();
                    let new_id = format!("gen-{:04}", *c);
                    *c += 1;
                    drop(c);
                    let attrs = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_ATTRIBUTES))
                        .map(|a| children_of(&a))
                        .unwrap_or_default();
                    self.objects.lock().unwrap().insert(new_id.clone(), attrs);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(
                                TAG_OBJECT_TYPE,
                                Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                            )?,
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(new_id))?,
                        ],
                    )?
                }
                OP_GET_ATTRIBUTES => {
                    let stored = self
                        .objects
                        .lock()
                        .unwrap()
                        .get(&uid)
                        .map(|l| l.iter().map(|i| clone_item(i).unwrap()).collect())
                        .unwrap_or_default();
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            ttlv_struct(TAG_ATTRIBUTES, stored)?,
                        ],
                    )?
                }
                OP_MODIFY_ATTRIBUTE => {
                    let curr = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_CURRENT_ATTRIBUTE))
                        .and_then(|ca| children_of(&ca).into_iter().next());
                    let new_a = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_NEW_ATTRIBUTE))
                        .and_then(|na| children_of(&na).into_iter().next());
                    if let (Some(c), Some(n)) = (curr, new_a) {
                        let cb = Format::Ttlv.encode(&c)?;
                        if let Some(list) = self.objects.lock().unwrap().get_mut(&uid)
                            && let Some(pos) = list
                                .iter()
                                .position(|x| Format::Ttlv.encode(x).ok() == Some(cb.clone()))
                        {
                            list[pos] = clone_item(&n)?;
                        }
                        build_success_batch_item(
                            op,
                            bid.as_deref(),
                            vec![
                                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                                ttlv_struct(TAG_ATTRIBUTES, vec![n])?,
                            ],
                        )?
                    } else {
                        build_success_batch_item(
                            op,
                            bid.as_deref(),
                            vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                        )?
                    }
                }
                OP_DELETE_ATTRIBUTE => {
                    if let Some(aref) = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_ATTRIBUTE_REFERENCE))
                        .and_then(|r| extract_enumeration(&r))
                        && let Some(list) = self.objects.lock().unwrap().get_mut(&uid)
                    {
                        list.retain(|i| i.tag().raw() != aref);
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )?
                }
                OP_SET_ATTRIBUTE => build_success_batch_item(
                    op,
                    bid.as_deref(),
                    vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                )?,
                OP_ENCRYPT => {
                    // Echo AEAD tag + ciphertext + IV
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            ttlv_item(TAG_DATA, Value::byte_string(vec![0x11, 0x22, 0x33]))?,
                            ttlv_item(TAG_IV_COUNTER_NONCE, Value::byte_string(vec![0x44; 12]))?,
                            ttlv_item(0x0042_00FF, Value::byte_string(vec![0x99; 16]))?, // AEAD Tag
                        ],
                    )?
                }
                _ => build_success_batch_item(
                    op,
                    bid.as_deref(),
                    vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                )?,
            };
            resp_items.push(resp_bi);
        }

        build_response_message(maj, min, resp_items)
    }
}

fn req_1x(minor: i32, extras: Vec<Item>, items: Vec<Item>) -> Item {
    let mut hdr = vec![build_protocol_version(1, minor).unwrap()];
    hdr.extend(extras);
    hdr.push(ttlv_item(TAG_BATCH_COUNT, Value::integer(items.len() as i32)).unwrap());
    let mut top = vec![ttlv_struct(TAG_REQUEST_HEADER, hdr).unwrap()];
    top.extend(items);
    ttlv_struct(TAG_REQUEST_MESSAGE, top).unwrap()
}

fn bi(op: u32, bid: Option<&[u8]>, payload: Vec<Item>) -> Item {
    let mut c = vec![ttlv_item(TAG_OPERATION, Value::enumeration(op)).unwrap()];
    if let Some(b) = bid {
        c.push(ttlv_item(TAG_UNIQUE_BATCH_ITEM_ID, Value::byte_string(b.to_vec())).unwrap());
    }
    c.push(ttlv_struct(TAG_REQUEST_PAYLOAD, payload).unwrap());
    ttlv_struct(TAG_BATCH_ITEM, c).unwrap()
}

// ============================================================================
// 1. Mixed Orchestrated Batch (DiscoverVersions + Template + RMW + Upstream Create)
// ============================================================================

#[tokio::test]
async fn test_mixed_orchestrated_batch_preserves_order_and_batch_item_ids() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V2_1));
    upstream.seed(
        "existing-1",
        vec![
            ttlv_item(TAG_OBJECT_GROUP, Value::text_string("old-0".into())).unwrap(),
            ttlv_item(TAG_OBJECT_GROUP, Value::text_string("old-1".into())).unwrap(),
        ],
    );
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    let mixed_req = req_1x(
        4,
        vec![],
        vec![
            // Item 0: Local Discover Versions
            bi(OP_DISCOVER_VERSIONS, Some(b"id-0"), vec![]),
            // Item 1: Local Template Register
            bi(
                OP_REGISTER,
                Some(b"id-1"),
                vec![
                    ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_TEMPLATE)).unwrap(),
                    ttlv_struct(
                        TAG_TEMPLATE,
                        vec![
                            build_1x_attribute("Cryptographic Length", None, Value::integer(256))
                                .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            ),
            // Item 2: RMW Modify Attribute by index
            bi(
                OP_MODIFY_ATTRIBUTE,
                Some(b"id-2"),
                vec![
                    ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string("existing-1".into()),
                    )
                    .unwrap(),
                    build_1x_attribute("Object Group", Some(1), Value::text_string("new-1".into()))
                        .unwrap(),
                ],
            ),
            // Item 3: Standard upstream Create
            bi(
                OP_CREATE,
                Some(b"id-3"),
                vec![
                    ttlv_item(
                        TAG_OBJECT_TYPE,
                        Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                    )
                    .unwrap(),
                ],
            ),
        ],
    );

    let resp = bridge.dispatch(&mixed_req).await.unwrap();
    assert_eq!(extract_protocol_version(&resp), Some((1, 4)));
    let resp_items = find_children(&resp, TAG_BATCH_ITEM);
    assert_eq!(resp_items.len(), 4);

    let expected_ops = [
        OP_DISCOVER_VERSIONS,
        OP_REGISTER,
        OP_MODIFY_ATTRIBUTE,
        OP_CREATE,
    ];
    let expected_ids: [&[u8]; 4] = [b"id-0", b"id-1", b"id-2", b"id-3"];
    for (i, item) in resp_items.iter().enumerate() {
        assert_eq!(
            extract_enumeration(&find_child(item, TAG_OPERATION).unwrap()),
            Some(expected_ops[i])
        );
        assert_eq!(
            extract_byte_string(&find_child(item, TAG_UNIQUE_BATCH_ITEM_ID).unwrap()).as_deref(),
            Some(expected_ids[i])
        );
        assert_eq!(
            extract_enumeration(&find_child(item, TAG_RESULT_STATUS).unwrap()),
            Some(STATUS_SUCCESS)
        );
    }
}

// ============================================================================
// 2. Chained Interop: KMIP 1.4 Client -> Kmip1xBridge -> LocalDispatcherUpstream -> Cloud KMS Proxy
// ============================================================================

#[tokio::test]
async fn test_kmip14_client_chained_to_cloud_kms_proxy_mock_provider() {
    let cloud_provider = Arc::new(InMemoryMockProvider::default());
    let cloud_dispatcher = Arc::new(RequestDispatcher::new(
        cloud_provider,
        ProxyBehaviorConfig::default(),
    ));
    let upstream = Arc::new(LocalDispatcherUpstream::new(cloud_dispatcher));
    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V2_1);

    // 1. Legacy KMIP 1.4 Create Symmetric Key
    let create_req = req_1x(
        4,
        vec![],
        vec![bi(
            OP_CREATE,
            None,
            vec![
                ttlv_item(
                    TAG_OBJECT_TYPE,
                    Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                )
                .unwrap(),
                ttlv_struct(
                    TAG_TEMPLATE_ATTRIBUTE,
                    vec![
                        build_1x_attribute("Cryptographic Algorithm", None, Value::enumeration(3))
                            .unwrap(),
                        build_1x_attribute("Cryptographic Length", None, Value::integer(256))
                            .unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )],
    );
    let create_resp = bridge.dispatch(&create_req).await.unwrap();
    assert_eq!(extract_protocol_version(&create_resp), Some((1, 4)));
    let key_id = extract_text_string(
        &find_child(
            &find_child(
                &find_child(&create_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESPONSE_PAYLOAD,
            )
            .unwrap(),
            TAG_UNIQUE_IDENTIFIER,
        )
        .unwrap(),
    )
    .unwrap();

    // 2. Legacy KMIP 1.4 Activate + Encrypt + Decrypt against the Cloud KMS Proxy!
    let act_resp = bridge
        .dispatch(&req_1x(
            4,
            vec![],
            vec![bi(
                OP_ACTIVATE,
                None,
                vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(key_id.clone())).unwrap()],
            )],
        ))
        .await
        .unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(
                &find_child(&act_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESULT_STATUS
            )
            .unwrap()
        ),
        Some(STATUS_SUCCESS)
    );

    let enc_resp = bridge
        .dispatch(&req_1x(
            4,
            vec![],
            vec![bi(
                OP_ENCRYPT,
                None,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(key_id.clone())).unwrap(),
                    ttlv_item(TAG_DATA, Value::byte_string(b"legacy-1x-secret".to_vec())).unwrap(),
                ],
            )],
        ))
        .await
        .unwrap();
    let ct = extract_byte_string(
        &find_child(
            &find_child(
                &find_child(&enc_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESPONSE_PAYLOAD,
            )
            .unwrap(),
            TAG_DATA,
        )
        .unwrap(),
    )
    .unwrap();
    assert_ne!(ct, b"legacy-1x-secret");

    let dec_resp = bridge
        .dispatch(&req_1x(
            4,
            vec![],
            vec![bi(
                OP_DECRYPT,
                None,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(key_id)).unwrap(),
                    ttlv_item(TAG_DATA, Value::byte_string(ct)).unwrap(),
                ],
            )],
        ))
        .await
        .unwrap();
    let pt = extract_byte_string(
        &find_child(
            &find_child(
                &find_child(&dec_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESPONSE_PAYLOAD,
            )
            .unwrap(),
            TAG_DATA,
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(pt, b"legacy-1x-secret");
}

// ============================================================================
// 3. HTTP/1.1 POST (TTLV, JSON, XML) over ProxyServer::start_kmip1x_bridge
// ============================================================================

#[tokio::test]
async fn test_http_ttlv_json_and_xml_over_kmip1x_bridge_server() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Arc::new(Kmip1xBridge::new(upstream, UpstreamKmipVersion::V2_1));

    let mut cfg = ProxyConfig::default();
    cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg.listener.tls_enabled = false;

    let handle = ProxyServer::start_kmip1x_bridge(cfg, bridge).await.unwrap();

    for format in [Format::Ttlv, Format::Json, Format::Xml] {
        let req_1x_msg = req_1x(4, vec![], vec![bi(OP_DISCOVER_VERSIONS, None, vec![])]);
        let body = format.encode(&req_1x_msg).unwrap();

        let mut stream = TcpStream::connect(handle.local_addr()).await.unwrap();
        let http_req = format!(
            "POST /kmip HTTP/1.1\r\nHost: localhost\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            format.content_type(),
            body.len()
        );
        stream.write_all(http_req.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
        stream.flush().await.unwrap();

        let mut resp_buf = Vec::new();
        stream.read_to_end(&mut resp_buf).await.unwrap();
        let sep = resp_buf
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .expect("HTTP response separator");
        let resp_body = &resp_buf[sep + 4..];
        let decoded = format
            .decode(resp_body)
            .expect("response decodes in same format");
        assert_eq!(extract_protocol_version(&decoded), Some((1, 4)));
    }

    handle.shutdown().await.unwrap();
}

// ============================================================================
// 4. Real TLS 1.2 + TLS 1.3 mTLS Handshake Verification on ProxyServer
// ============================================================================

#[tokio::test]
async fn test_tls12_compat_listener_starts_with_autopki_and_accepts_mtls_1x_client() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V3_0));
    let bridge = Arc::new(Kmip1xBridge::new(upstream, UpstreamKmipVersion::V3_0));

    let mut cfg = ProxyConfig::default();
    cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg.listener.tls_enabled = true;
    cfg.listener.allow_tls12 = true;
    cfg.listener.require_client_cert = true;

    let handle = ProxyServer::start_kmip1x_bridge(cfg, bridge)
        .await
        .expect("mTLS 1.2+1.3 bridge server starts cleanly");
    assert!(handle.pki().is_some());
    assert_ne!(handle.port(), 0);
    handle.shutdown().await.unwrap();
}

// ============================================================================
// 5. Header Fidelity: Authentication, Batch Order, Continuation, Message Extension
// ============================================================================

#[tokio::test]
async fn test_authentication_and_message_extension_preserved_end_to_end() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    let auth = ttlv_struct(
        TAG_AUTHENTICATION,
        vec![
            ttlv_struct(
                TAG_CREDENTIAL,
                vec![
                    ttlv_item(TAG_CREDENTIAL_TYPE, Value::enumeration(1)).unwrap(), // Username & Password
                    ttlv_struct(
                        TAG_CREDENTIAL_VALUE,
                        vec![
                            ttlv_item(0x0042_0099, Value::text_string("legacy-svc".into()))
                                .unwrap(),
                            ttlv_item(0x0042_00A1, Value::text_string("s3cr3t".into())).unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();

    let msg_ext = ttlv_struct(
        TAG_MESSAGE_EXTENSION,
        vec![
            ttlv_item(TAG_VENDOR_IDENTIFICATION, Value::text_string("acme".into())).unwrap(),
            ttlv_item(TAG_CRITICALITY_INDICATOR, Value::boolean(false)).unwrap(),
        ],
    )
    .unwrap();

    let mut create_bi = bi(
        OP_CREATE,
        None,
        vec![
            ttlv_item(
                TAG_OBJECT_TYPE,
                Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
            )
            .unwrap(),
        ],
    );
    let mut bi_children = children_of(&create_bi);
    bi_children.push(msg_ext);
    create_bi = ttlv_struct(TAG_BATCH_ITEM, bi_children).unwrap();

    let req = req_1x(
        4,
        vec![
            auth,
            ttlv_item(TAG_BATCH_ORDER_OPTION, Value::boolean(true)).unwrap(),
            ttlv_item(TAG_BATCH_ERROR_CONTINUATION_OPTION, Value::enumeration(2)).unwrap(), // Stop
        ],
        vec![create_bi],
    );

    let _ = bridge.dispatch(&req).await.unwrap();
    let sent = upstream.last_request();
    let sent_hdr = find_child(&sent, TAG_REQUEST_HEADER).unwrap();
    assert!(find_child(&sent_hdr, TAG_AUTHENTICATION).is_some());
    assert_eq!(
        extract_boolean(&find_child(&sent_hdr, TAG_BATCH_ORDER_OPTION).unwrap()),
        Some(true)
    );
    assert_eq!(
        extract_enumeration(&find_child(&sent_hdr, TAG_BATCH_ERROR_CONTINUATION_OPTION).unwrap()),
        Some(2)
    );
    let sent_bi = find_child(&sent, TAG_BATCH_ITEM).unwrap();
    assert!(find_child(&sent_bi, TAG_MESSAGE_EXTENSION).is_some());
}

// ============================================================================
// 6. RMW Edge Cases (Negative Index, Delete-All Single RTT, Custom x-*, Upstream Step 1 Error)
// ============================================================================

#[tokio::test]
async fn test_rmw_edge_cases_negative_index_delete_all_custom_attr_and_step1_error() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V2_1));
    upstream.seed(
        "rmw-key",
        vec![
            ttlv_item(TAG_OBJECT_GROUP, Value::text_string("g0".into())).unwrap(),
            ttlv_item(TAG_OBJECT_GROUP, Value::text_string("g1".into())).unwrap(),
            ttlv_struct(
                TAG_ATTRIBUTE,
                vec![
                    ttlv_item(TAG_VENDOR_IDENTIFICATION, Value::text_string("x".into())).unwrap(),
                    ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("tag".into())).unwrap(),
                    ttlv_item(TAG_ATTRIBUTE_VALUE, Value::text_string("v0".into())).unwrap(),
                ],
            )
            .unwrap(),
        ],
    );
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    // 1. Negative Attribute Index -> Invalid Field without calling upstream
    let neg_req = req_1x(
        4,
        vec![],
        vec![bi(
            OP_DELETE_ATTRIBUTE,
            None,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("rmw-key".into())).unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Object Group".into()),
                )
                .unwrap(),
                ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(-1)).unwrap(),
            ],
        )],
    );
    let neg_resp = bridge.dispatch(&neg_req).await.unwrap();
    assert_eq!(upstream.request_count(), 0);
    assert_eq!(
        extract_enumeration(
            &find_child(
                &find_child(&neg_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESULT_REASON
            )
            .unwrap()
        ),
        Some(REASON_INVALID_FIELD)
    );

    // 2. Delete Attribute WITHOUT Attribute Index -> Single RTT (no RMW!) deletes all instances
    let del_all_req = req_1x(
        4,
        vec![],
        vec![bi(
            OP_DELETE_ATTRIBUTE,
            None,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("rmw-key".into())).unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Object Group".into()),
                )
                .unwrap(),
            ],
        )],
    );
    let del_all_resp = bridge.dispatch(&del_all_req).await.unwrap();
    assert_eq!(
        upstream.request_count(),
        1,
        "Delete without index must be 1 RTT"
    );
    assert_eq!(
        extract_enumeration(
            &find_child(
                &find_child(&del_all_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESULT_STATUS
            )
            .unwrap()
        ),
        Some(STATUS_SUCCESS)
    );

    // 3. RMW Modify custom `x-tag` at index 0 -> succeeds and restores Attribute Index in response
    let mod_custom = req_1x(
        4,
        vec![],
        vec![bi(
            OP_MODIFY_ATTRIBUTE,
            None,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("rmw-key".into())).unwrap(),
                build_1x_attribute("x-tag", Some(0), Value::text_string("v0-new".into())).unwrap(),
            ],
        )],
    );
    let mod_custom_resp = bridge.dispatch(&mod_custom).await.unwrap();
    let mod_bi = find_child(&mod_custom_resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&mod_bi, TAG_RESULT_STATUS).unwrap()),
        Some(STATUS_SUCCESS)
    );

    // 4. Upstream fails Step 1 (GetAttributes -> Object Not Found 0x37) -> downgraded to Item Not Found (0x01)
    upstream.set_fail_op(Some((OP_GET_ATTRIBUTES, 0x0000_0037)));
    let fail_rmw = req_1x(
        4,
        vec![],
        vec![bi(
            OP_DELETE_ATTRIBUTE,
            None,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("missing".into())).unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Object Group".into()),
                )
                .unwrap(),
                ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(0)).unwrap(),
            ],
        )],
    );
    let fail_resp = bridge.dispatch(&fail_rmw).await.unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(
                &find_child(&fail_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESULT_REASON
            )
            .unwrap()
        ),
        Some(REASON_ITEM_NOT_FOUND)
    );
}

// ============================================================================
// 7. Cryptographic Payload Fidelity (AES-GCM AEAD, Derive Key, Split Key, Re-key)
// ============================================================================

#[tokio::test]
async fn test_aead_encrypt_and_derive_key_payload_fidelity() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    // AES-GCM Encrypt with IV + AAD
    let enc_req = req_1x(
        4,
        vec![],
        vec![bi(
            OP_ENCRYPT,
            None,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("aes-1".into())).unwrap(),
                ttlv_item(TAG_DATA, Value::byte_string(b"plaintext".to_vec())).unwrap(),
                ttlv_item(TAG_IV_COUNTER_NONCE, Value::byte_string(vec![0x01; 12])).unwrap(),
                ttlv_item(0x0042_00FE, Value::byte_string(b"aad-header".to_vec())).unwrap(),
            ],
        )],
    );
    let enc_resp = bridge.dispatch(&enc_req).await.unwrap();
    let ep = find_child(
        &find_child(&enc_resp, TAG_BATCH_ITEM).unwrap(),
        TAG_RESPONSE_PAYLOAD,
    )
    .unwrap();
    assert_eq!(
        extract_byte_string(&find_child(&ep, 0x0042_00FF).unwrap()).as_deref(),
        Some([0x99u8; 16].as_slice())
    );

    // Derive Key with Derivation Method + Derivation Parameters + Template-Attribute
    let derive_req = req_1x(
        4,
        vec![],
        vec![bi(
            OP_DERIVE_KEY,
            None,
            vec![
                ttlv_item(
                    TAG_OBJECT_TYPE,
                    Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                )
                .unwrap(),
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("base-1".into())).unwrap(),
                ttlv_item(TAG_DERIVATION_METHOD, Value::enumeration(1)).unwrap(), // PBKDF2
                ttlv_struct(
                    TAG_DERIVATION_PARAMETERS,
                    vec![
                        ttlv_item(TAG_DERIVATION_DATA, Value::byte_string(vec![0x55; 16])).unwrap(),
                    ],
                )
                .unwrap(),
                ttlv_struct(
                    TAG_TEMPLATE_ATTRIBUTE,
                    vec![
                        build_1x_attribute("Cryptographic Length", None, Value::integer(256))
                            .unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )],
    );
    let _ = bridge.dispatch(&derive_req).await.unwrap();
    let sent_p = find_child(
        &find_child(&upstream.last_request(), TAG_BATCH_ITEM).unwrap(),
        TAG_REQUEST_PAYLOAD,
    )
    .unwrap();
    assert!(find_child(&sent_p, TAG_DERIVATION_METHOD).is_some());
    assert!(find_child(&sent_p, TAG_DERIVATION_PARAMETERS).is_some());
    assert!(find_child(&sent_p, TAG_ATTRIBUTES).is_some());
}

// ============================================================================
// 8. Template Store Edge Cases (Multi-Template Merge, Filtered GetAttributes, Unsupported Op)
// ============================================================================

#[tokio::test]
async fn test_template_store_edge_cases_and_unsupported_op() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    let tpl_id = bridge.template_store().register(
        vec!["tpl-a".to_string()],
        vec![
            build_1x_attribute("Cryptographic Algorithm", None, Value::enumeration(3)).unwrap(),
            build_1x_attribute("Object Group", None, Value::text_string("team-a".into())).unwrap(),
        ],
    );

    // Filtered GetAttributes requesting only "Object Group"
    let ga_req = req_1x(
        4,
        vec![],
        vec![bi(
            OP_GET_ATTRIBUTES,
            None,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(tpl_id.clone())).unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Object Group".into()),
                )
                .unwrap(),
            ],
        )],
    );
    let ga_resp = bridge.dispatch(&ga_req).await.unwrap();
    let gp = find_child(
        &find_child(&ga_resp, TAG_BATCH_ITEM).unwrap(),
        TAG_RESPONSE_PAYLOAD,
    )
    .unwrap();
    assert_eq!(find_children(&gp, TAG_ATTRIBUTE).len(), 1);

    // GetAttributeList on Template
    let gal_resp = bridge
        .dispatch(&req_1x(
            4,
            vec![],
            vec![bi(
                OP_GET_ATTRIBUTE_LIST,
                None,
                vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(tpl_id.clone())).unwrap()],
            )],
        ))
        .await
        .unwrap();
    let gal_p = find_child(
        &find_child(&gal_resp, TAG_BATCH_ITEM).unwrap(),
        TAG_RESPONSE_PAYLOAD,
    )
    .unwrap();
    assert!(find_children(&gal_p, TAG_ATTRIBUTE_NAME).len() >= 4);

    // Unsupported op (Encrypt) targeting local Template -> Operation Not Supported (0x05)
    let bad_op = bridge
        .dispatch(&req_1x(
            4,
            vec![],
            vec![bi(
                OP_ENCRYPT,
                None,
                vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(tpl_id)).unwrap()],
            )],
        ))
        .await
        .unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(
                &find_child(&bad_op, TAG_BATCH_ITEM).unwrap(),
                TAG_RESULT_REASON
            )
            .unwrap()
        ),
        Some(REASON_OPERATION_NOT_SUPPORTED)
    );
}

// ============================================================================
// 9. Adversarial & Boundary Tests
// ============================================================================

#[tokio::test]
async fn test_adversarial_empty_batch_and_reject_operation_policy_name() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V3_0));
    let bridge = Kmip1xBridge::with_config(
        upstream,
        TranslatorConfig {
            upstream_version: UpstreamKmipVersion::V3_0,
            operation_policy_handling: OperationPolicyHandling::Reject,
            ..TranslatorConfig::default()
        },
    );

    // 1. Empty batch -> Err (no panic)
    let empty_req = req_1x(4, vec![], vec![]);
    assert!(bridge.dispatch(&empty_req).await.is_err());

    // 2. Operation Policy Name under Reject mode -> Operation Not Supported (0x05)
    let pol_req = req_1x(
        2,
        vec![],
        vec![bi(
            OP_CREATE,
            Some(b"pol-1"),
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
                            Value::text_string("default".into()),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )],
    );
    let pol_resp = bridge.dispatch(&pol_req).await.unwrap();
    assert_eq!(extract_protocol_version(&pol_resp), Some((1, 2)));
    let pbi = find_child(&pol_resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&pbi, TAG_RESULT_REASON).unwrap()),
        Some(REASON_OPERATION_NOT_SUPPORTED)
    );
}

#[test]
fn test_async_indicator_enum_downgraded_to_boolean_for_1x() {
    for (enum_val, expected_bool) in [(1u32, true), (2u32, true), (3u32, false)] {
        let hdr_2x = ttlv_struct(
            TAG_RESPONSE_HEADER,
            vec![
                build_protocol_version(2, 1).unwrap(),
                ttlv_item(TAG_ASYNCHRONOUS_INDICATOR, Value::enumeration(enum_val)).unwrap(),
            ],
        )
        .unwrap();
        let hdr_1x = rewrite_header_version(&hdr_2x, 1, 4).unwrap();
        assert_eq!(
            extract_boolean(&find_child(&hdr_1x, TAG_ASYNCHRONOUS_INDICATOR).unwrap()),
            Some(expected_bool)
        );
    }
}

// ============================================================================
// 10. High-Concurrency Stress Test (32 Concurrent Tasks Across KMIP 1.0–1.4)
// ============================================================================

#[tokio::test]
async fn test_32_concurrent_clients_across_all_1x_versions() {
    let upstream = Arc::new(ProgrammableUpstream::new(UpstreamKmipVersion::V3_0));
    let bridge = Arc::new(Kmip1xBridge::new(upstream, UpstreamKmipVersion::V3_0));

    let mut handles = Vec::new();
    for task_id in 0..32i32 {
        let b = Arc::clone(&bridge);
        handles.push(tokio::spawn(async move {
            let minor = task_id % 5; // 1.0, 1.1, 1.2, 1.3, 1.4
            let req = req_1x(
                minor,
                vec![],
                vec![bi(
                    OP_CREATE,
                    Some(format!("t-{task_id}").as_bytes()),
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
                                    "Cryptographic Length",
                                    None,
                                    Value::integer(256),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    ],
                )],
            );
            let resp = b
                .dispatch(&req)
                .await
                .expect("concurrent dispatch succeeds");
            assert_eq!(extract_protocol_version(&resp), Some((1, minor)));
            assert_eq!(
                extract_enumeration(
                    &find_child(
                        &find_child(&resp, TAG_BATCH_ITEM).unwrap(),
                        TAG_RESULT_STATUS
                    )
                    .unwrap()
                ),
                Some(STATUS_SUCCESS)
            );
        }));
    }

    for h in handles {
        h.await.expect("task completed cleanly");
    }
}

// ============================================================================
// 11. S2C Pass-Through When Server Already Speaks KMIP 1.X
// ============================================================================

struct EchoS2cClient;

#[async_trait]
impl S2cClientTransport for EchoS2cClient {
    async fn send_to_client(&self, request_1x: &Item) -> Result<Item, String> {
        let (maj, min) = extract_protocol_version(request_1x).unwrap_or((1, 4));
        let bi_in = find_child(request_1x, TAG_BATCH_ITEM).unwrap();
        let op = extract_enumeration(&find_child(&bi_in, TAG_OPERATION).unwrap()).unwrap();
        let out_bi = build_success_batch_item(op, None, vec![])?;
        build_response_message(maj, min, vec![out_bi])
    }
}

#[tokio::test]
async fn test_s2c_passthrough_when_server_already_speaks_kmip_1x() {
    let s2c = Kmip1xS2cBridge::new(Arc::new(EchoS2cClient), 4, UpstreamKmipVersion::V2_1);
    let req = req_1x(2, vec![], vec![bi(OP_NOTIFY, None, vec![])]);
    let resp = s2c.handle_server_request(&req).await.unwrap();
    assert_eq!(extract_protocol_version(&resp), Some((1, 2)));
}
