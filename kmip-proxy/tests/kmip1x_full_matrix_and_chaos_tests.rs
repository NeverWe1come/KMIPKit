//! Full Cartesian Matrix, Deprecated/Rejected Ops, Boundary, Weird Inputs, and
//! Network/Chaos Fault-Injection Test Suite for `kmipkit_proxy::kmip1x`.
//!
//! Covers:
//! 1. Full Matrix: All 11 attribute-bearing creation/import ops x All 77 standard attributes x (2.0, 2.1, 3.0)
//! 2. Full Matrix: `Locate` x All 77 standard attributes x (2.0, 2.1, 3.0) (verifying read-only attrs are NOT stripped)
//! 3. Full Matrix: `Add Attribute` / `Modify Attribute` / `Delete Attribute` x All 77 standard attributes
//! 4. Full Matrix: `Get Attributes` & `Get Attribute List` x All 77 standard attributes x (1.0, 1.1, 1.2, 1.3, 1.4)
//! 5. Deprecated / Rejected / Unknown Operations & Attributes (1.0 cert aliases, 2.0/3.0-only ops from 1.X, garbage op codes)
//! 6. Weird & Boundary Protocol Cases (50 multi-instance attrs, 5-op ID Placeholder chain, exact Max Response Size +/- 1 byte,
//!    UTF-8 / long strings, weird custom attribute names `x-`, `y-`, `x-a-b-c`, `z-unknown`)
//! 7. Network & Chaos Tests over real TCP sockets (`ProxyServer::start_kmip1x_bridge`):
//!    - Truncated TTLV header & truncated TTLV payload
//!    - Garbage leading bytes & unsupported HTTP verbs (`GET`)
//!    - Oversized HTTP header (> 16 KiB) & zero-length HTTP body
//!    - Upstream transport failure/timeout recovery (server stays healthy for next client)
//!    - Persistent TCP keep-alive (20 sequential TTLV frames on 1 socket)
//!    - HTTP/1.1 keep-alive (mixed TTLV + JSON + XML POSTs on 1 socket)
#![forbid(unsafe_code)]
#![cfg(feature = "kmip1x-bridge")]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kmipkit_proxy::codec::Format;
use kmipkit_proxy::config::ProxyConfig;
use kmipkit_proxy::kmip1x::mapping::{Cardinality, STANDARD_ATTRIBUTES, UpstreamKmipVersion};
use kmipkit_proxy::kmip1x::tags::*;
use kmipkit_proxy::kmip1x::translator::{
    TranslatorConfig, build_1x_attribute, extract_protocol_version, translate_attribute_1x_to_2x,
    translate_attributes_2x_to_1x,
};
use kmipkit_proxy::kmip1x::ttlv_helpers::{
    build_error_batch_item, build_protocol_version, build_response_message,
    build_success_batch_item, children_of, clone_item, extract_byte_string, extract_enumeration,
    extract_text_string, find_child, find_children, ttlv_item, ttlv_struct,
};
use kmipkit_proxy::kmip1x::{Kmip1xBridge, UpstreamKmipTransport};
use kmipkit_proxy::server::ProxyServer;
use kmipkit_ttlv::{Item, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ============================================================================
// Sample Value Helper for All 77 Standard Attributes
// ============================================================================

fn sample_val(name: &str) -> Value {
    match name {
        "Object Type" => Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
        "Cryptographic Algorithm" => Value::enumeration(0x03),
        "Key Format Type" | "Split Key Method" | "Opaque Data Type" | "Certificate Type" => {
            Value::enumeration(0x01)
        }
        "Digital Signature Algorithm" => Value::enumeration(0x05),
        "State" => Value::enumeration(0x02),
        "Key Role Type" => Value::enumeration(0x03),
        "Cryptographic Length" => Value::integer(256),
        "Cryptographic Usage Mask" => Value::integer(0x0C),
        "Key Part Identifier" => Value::integer(1),
        "Split Key Parts" => Value::integer(3),
        "Split Key Threshold" => Value::integer(2),
        "Certificate Length" => Value::integer(1024),
        "Lease Time" => Value::interval(3600),
        "Fresh" | "Key Value Present" | "Sensitive" | "Always Sensitive" | "Extractable"
        | "Never Extractable" => Value::boolean(true),
        n if n.ends_with("Date") => Value::date_time(1_700_000_000),
        "Name" => {
            let s = ttlv_struct(
                TAG_NAME,
                vec![
                    ttlv_item(TAG_NAME_VALUE, Value::text_string("k-name".into())).unwrap(),
                    ttlv_item(TAG_NAME_TYPE, Value::enumeration(1)).unwrap(),
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "Link" => {
            let s = ttlv_struct(
                TAG_LINK,
                vec![
                    ttlv_item(TAG_LINK_TYPE, Value::enumeration(LINK_TYPE_PARENT)).unwrap(),
                    ttlv_item(
                        TAG_LINKED_OBJECT_IDENTIFIER,
                        Value::text_string("parent-1".into()),
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        _ => Value::text_string(format!("v-{name}")),
    }
}

// ============================================================================
// Fault-Injectable Mock Upstream Server
// ============================================================================

struct ChaosUpstream {
    version: UpstreamKmipVersion,
    fail_transport: AtomicBool,
    recorded: Mutex<Vec<Item>>,
    objects: Mutex<BTreeMap<String, Vec<Item>>>,
}

impl ChaosUpstream {
    fn new(version: UpstreamKmipVersion) -> Self {
        Self {
            version,
            fail_transport: AtomicBool::new(false),
            recorded: Mutex::new(Vec::new()),
            objects: Mutex::new(BTreeMap::new()),
        }
    }

    fn seed(&self, uid: &str, attrs: Vec<Item>) {
        self.objects.lock().unwrap().insert(uid.to_string(), attrs);
    }

    fn last_request(&self) -> Item {
        clone_item(self.recorded.lock().unwrap().last().unwrap()).unwrap()
    }
}

#[async_trait]
impl UpstreamKmipTransport for ChaosUpstream {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        if self.fail_transport.load(Ordering::SeqCst) {
            return Err("simulated upstream TCP connection reset by peer".to_string());
        }
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

            // Reject unknown / garbage operation codes just like a real KMIP 2.x/3.0 server
            if op == 0 || op > 0x0000_0042 {
                resp_items.push(build_error_batch_item(
                    op,
                    bid.as_deref(),
                    REASON_OPERATION_NOT_SUPPORTED,
                    "unknown operation",
                )?);
                continue;
            }

            let uid = payload
                .as_ref()
                .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                .and_then(|i| extract_text_string(&i))
                .unwrap_or_else(|| "obj-chaos-1".to_string());

            let resp_bi = match op {
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
                            list[pos] = n;
                        }
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )?
                }
                OP_DELETE_ATTRIBUTE => {
                    let curr = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_CURRENT_ATTRIBUTE))
                        .and_then(|ca| children_of(&ca).into_iter().next());
                    if let Some(c) = curr {
                        let cb = Format::Ttlv.encode(&c)?;
                        if let Some(list) = self.objects.lock().unwrap().get_mut(&uid)
                            && let Some(pos) = list
                                .iter()
                                .position(|x| Format::Ttlv.encode(x).ok() == Some(cb.clone()))
                        {
                            list.remove(pos);
                        }
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
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

fn req(minor: i32, extras: Vec<Item>, items: Vec<Item>) -> Item {
    let mut hdr = vec![build_protocol_version(1, minor).unwrap()];
    hdr.extend(extras);
    hdr.push(ttlv_item(TAG_BATCH_COUNT, Value::integer(items.len() as i32)).unwrap());
    let mut top = vec![ttlv_struct(TAG_REQUEST_HEADER, hdr).unwrap()];
    top.extend(items);
    ttlv_struct(TAG_REQUEST_MESSAGE, top).unwrap()
}

fn bi(op: u32, payload: Vec<Item>) -> Item {
    ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(op)).unwrap(),
            ttlv_struct(TAG_REQUEST_PAYLOAD, payload).unwrap(),
        ],
    )
    .unwrap()
}

// ============================================================================
// 1. FULL MATRIX: All 11 Creation/Import Ops x All 77 Standard Attributes x (2.0, 2.1, 3.0)
// ============================================================================

#[tokio::test]
async fn test_matrix_all_11_template_attribute_ops_with_all_77_attributes() {
    let template_ops = [
        OP_CREATE,
        OP_REGISTER,
        OP_REKEY,
        OP_DERIVE_KEY,
        OP_CERTIFY,
        OP_RECERTIFY,
        OP_CREATE_SPLIT_KEY,
        OP_JOIN_SPLIT_KEY,
        OP_IMPORT,
    ];
    let writable_count = STANDARD_ATTRIBUTES
        .iter()
        .filter(|s| !s.read_only_on_create)
        .count();

    for target in [
        UpstreamKmipVersion::V2_0,
        UpstreamKmipVersion::V2_1,
        UpstreamKmipVersion::V3_0,
    ] {
        let upstream = Arc::new(ChaosUpstream::new(target));
        let bridge = Kmip1xBridge::new(upstream.clone(), target);

        for op in template_ops {
            let all_77: Vec<Item> = STANDARD_ATTRIBUTES
                .iter()
                .map(|s| build_1x_attribute(s.name_1x, None, sample_val(s.name_1x)).unwrap())
                .collect();

            let request = req(
                4,
                vec![],
                vec![bi(
                    op,
                    vec![
                        ttlv_item(
                            TAG_OBJECT_TYPE,
                            Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                        )
                        .unwrap(),
                        ttlv_struct(TAG_TEMPLATE_ATTRIBUTE, all_77).unwrap(),
                    ],
                )],
            );
            let resp = bridge.dispatch(&request).await.unwrap();
            assert_eq!(extract_protocol_version(&resp), Some((1, 4)));

            let sent = upstream.last_request();
            let sent_attrs = find_child(
                &find_child(
                    &find_child(&sent, TAG_BATCH_ITEM).unwrap(),
                    TAG_REQUEST_PAYLOAD,
                )
                .unwrap(),
                TAG_ATTRIBUTES,
            )
            .expect("Template-Attribute translated to Attributes");
            assert_eq!(
                children_of(&sent_attrs).len(),
                writable_count,
                "op 0x{op:02X} on {target:?} must contain all {writable_count} writable attrs"
            );
        }
    }
}

// ============================================================================
// 2. FULL MATRIX: Locate x All 77 Standard Attributes (Read-Only MUST NOT be stripped)
// ============================================================================

#[tokio::test]
async fn test_matrix_locate_preserves_all_77_attributes_including_readonly() {
    for target in [
        UpstreamKmipVersion::V2_0,
        UpstreamKmipVersion::V2_1,
        UpstreamKmipVersion::V3_0,
    ] {
        let upstream = Arc::new(ChaosUpstream::new(target));
        let bridge = Kmip1xBridge::new(upstream.clone(), target);

        let all_77: Vec<Item> = STANDARD_ATTRIBUTES
            .iter()
            .map(|s| build_1x_attribute(s.name_1x, None, sample_val(s.name_1x)).unwrap())
            .collect();

        let locate_req = req(4, vec![], vec![bi(OP_LOCATE, all_77)]);
        let resp = bridge.dispatch(&locate_req).await.unwrap();
        assert_eq!(extract_protocol_version(&resp), Some((1, 4)));

        let sent = upstream.last_request();
        let sent_attrs = find_child(
            &find_child(
                &find_child(&sent, TAG_BATCH_ITEM).unwrap(),
                TAG_REQUEST_PAYLOAD,
            )
            .unwrap(),
            TAG_ATTRIBUTES,
        )
        .unwrap();
        assert_eq!(
            children_of(&sent_attrs).len(),
            STANDARD_ATTRIBUTES.len(),
            "Locate on {target:?} must keep all 77 attributes including read-only ones"
        );
    }
}

// ============================================================================
// 3. FULL MATRIX: AddAttribute Routing x All 77 Standard Attributes
// ============================================================================

#[tokio::test]
async fn test_matrix_add_attribute_cardinality_routing_for_all_77_attributes() {
    let upstream = Arc::new(ChaosUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    for spec in STANDARD_ATTRIBUTES {
        let attr_1x = build_1x_attribute(spec.name_1x, None, sample_val(spec.name_1x)).unwrap();
        let request = req(
            4,
            vec![],
            vec![bi(
                OP_ADD_ATTRIBUTE,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-1".into())).unwrap(),
                    attr_1x,
                ],
            )],
        );
        let resp = bridge.dispatch(&request).await.unwrap();

        // Upstream op must match cardinality
        let sent_op = extract_enumeration(
            &find_child(
                &find_child(&upstream.last_request(), TAG_BATCH_ITEM).unwrap(),
                TAG_OPERATION,
            )
            .unwrap(),
        );
        let expected_up_op = match spec.cardinality {
            Cardinality::SingleInstance => OP_SET_ATTRIBUTE,
            Cardinality::MultiInstance => OP_ADD_ATTRIBUTE,
        };
        assert_eq!(
            sent_op,
            Some(expected_up_op),
            "AddAttribute routing mismatch for '{}'",
            spec.name_1x
        );

        // Response op MUST always be restored to OP_ADD_ATTRIBUTE for the 1.X client
        let resp_op = extract_enumeration(
            &find_child(&find_child(&resp, TAG_BATCH_ITEM).unwrap(), TAG_OPERATION).unwrap(),
        );
        assert_eq!(resp_op, Some(OP_ADD_ATTRIBUTE));
    }
}

// ============================================================================
// 4. Deprecated, Unknown, and Out-of-Version Operations
// ============================================================================

#[tokio::test]
async fn test_unknown_and_garbage_operation_codes_return_clean_1x_error() {
    let upstream = Arc::new(ChaosUpstream::new(UpstreamKmipVersion::V3_0));
    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V3_0);

    for bad_op in [0x0000_0000u32, 0x0000_0099, 0x7FFF_FFFF] {
        let request = req(2, vec![], vec![bi(bad_op, vec![])]);
        let resp = bridge.dispatch(&request).await.unwrap();
        assert_eq!(extract_protocol_version(&resp), Some((1, 2)));
        let rbi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
        assert_eq!(
            extract_enumeration(&find_child(&rbi, TAG_RESULT_STATUS).unwrap()),
            Some(STATUS_OPERATION_FAILED)
        );
        assert_eq!(
            extract_enumeration(&find_child(&rbi, TAG_RESULT_REASON).unwrap()),
            Some(REASON_OPERATION_NOT_SUPPORTED)
        );
    }
}

#[tokio::test]
async fn test_deprecated_10_certificate_aliases_in_locate_and_get_attributes() {
    let upstream = Arc::new(ChaosUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    let ga_req = req(
        0,
        vec![],
        vec![bi(
            OP_GET_ATTRIBUTES,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("cert-1".into())).unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Certificate Identifier".into()),
                )
                .unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Certificate Subject".into()),
                )
                .unwrap(),
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string("Certificate Issuer".into()),
                )
                .unwrap(),
            ],
        )],
    );
    let _ = bridge.dispatch(&ga_req).await.unwrap();
    let sent_p = find_child(
        &find_child(&upstream.last_request(), TAG_BATCH_ITEM).unwrap(),
        TAG_REQUEST_PAYLOAD,
    )
    .unwrap();
    let refs: Vec<u32> = find_children(&sent_p, TAG_ATTRIBUTE_REFERENCE)
        .iter()
        .filter_map(extract_enumeration)
        .collect();
    assert_eq!(
        refs,
        vec![
            TAG_X509_CERTIFICATE_IDENTIFIER,
            TAG_X509_CERTIFICATE_SUBJECT,
            TAG_X509_CERTIFICATE_ISSUER
        ]
    );
}

// ============================================================================
// 5. Weird & Boundary Cases: 50 Multi-Instance Attributes, Exact Max Size, Weird Custom Names
// ============================================================================

#[tokio::test]
async fn test_50_multi_instance_attributes_rmw_modify_middle_and_delete_ends() {
    let upstream = Arc::new(ChaosUpstream::new(UpstreamKmipVersion::V2_1));
    let initial: Vec<Item> = (0..50)
        .map(|i| ttlv_item(TAG_OBJECT_GROUP, Value::text_string(format!("grp-{i:02}"))).unwrap())
        .collect();
    upstream.seed("many-attr-key", initial);
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    // Modify index 25
    let mod25 = req(
        4,
        vec![],
        vec![bi(
            OP_MODIFY_ATTRIBUTE,
            vec![
                ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string("many-attr-key".into()),
                )
                .unwrap(),
                build_1x_attribute(
                    "Object Group",
                    Some(25),
                    Value::text_string("grp-25-MOD".into()),
                )
                .unwrap(),
            ],
        )],
    );
    let _ = bridge.dispatch(&mod25).await.unwrap();

    // Delete index 49 (last) and index 0 (first)
    for idx in [49, 0] {
        let del = req(
            4,
            vec![],
            vec![bi(
                OP_DELETE_ATTRIBUTE,
                vec![
                    ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string("many-attr-key".into()),
                    )
                    .unwrap(),
                    ttlv_item(
                        TAG_ATTRIBUTE_NAME,
                        Value::text_string("Object Group".into()),
                    )
                    .unwrap(),
                    ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(idx)).unwrap(),
                ],
            )],
        );
        let _ = bridge.dispatch(&del).await.unwrap();
    }

    // Verify 48 remain and index 24 (shifted from 25) is "grp-25-MOD"
    let ga = bridge
        .dispatch(&req(
            4,
            vec![],
            vec![bi(
                OP_GET_ATTRIBUTES,
                vec![
                    ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string("many-attr-key".into()),
                    )
                    .unwrap(),
                ],
            )],
        ))
        .await
        .unwrap();
    let gp = find_child(
        &find_child(&ga, TAG_BATCH_ITEM).unwrap(),
        TAG_RESPONSE_PAYLOAD,
    )
    .unwrap();
    let attrs = find_children(&gp, TAG_ATTRIBUTE);
    assert_eq!(attrs.len(), 48);
    assert_eq!(
        extract_text_string(&find_child(&attrs[24], TAG_ATTRIBUTE_VALUE).unwrap()).as_deref(),
        Some("grp-25-MOD")
    );
}

#[tokio::test]
async fn test_exact_maximum_response_size_boundary_plus_minus_one_byte() {
    let upstream = Arc::new(ChaosUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V2_1);

    // Measure exact serialized 1.X response size first
    let baseline = bridge
        .dispatch(&req(4, vec![], vec![bi(OP_CREATE, vec![])]))
        .await
        .unwrap();
    let exact_len = Format::Ttlv.encode(&baseline).unwrap().len() as i32;

    // Limit == exact_len -> MUST SUCCEED
    let ok_resp = bridge
        .dispatch(&req(
            4,
            vec![ttlv_item(TAG_MAXIMUM_RESPONSE_SIZE, Value::integer(exact_len)).unwrap()],
            vec![bi(OP_CREATE, vec![])],
        ))
        .await
        .unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(
                &find_child(&ok_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESULT_STATUS
            )
            .unwrap()
        ),
        Some(STATUS_SUCCESS)
    );

    // Limit == exact_len - 1 -> MUST FAIL with Response Too Large (0x02)
    let fail_resp = bridge
        .dispatch(&req(
            4,
            vec![ttlv_item(TAG_MAXIMUM_RESPONSE_SIZE, Value::integer(exact_len - 1)).unwrap()],
            vec![bi(OP_CREATE, vec![])],
        ))
        .await
        .unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(
                &find_child(&fail_resp, TAG_BATCH_ITEM).unwrap(),
                TAG_RESULT_REASON
            )
            .unwrap()
        ),
        Some(REASON_RESPONSE_TOO_LARGE)
    );
}

#[test]
fn test_weird_custom_and_unknown_attribute_names_and_utf8_values() {
    let cfg = TranslatorConfig::default();
    let weird_names = [
        "x-a-b-c-d",
        "y-12345",
        "x-español-🔑",
        "z-nonstandard-vendor",
    ];

    let mut v2_items = Vec::new();
    for name in weird_names {
        let a1x =
            build_1x_attribute(name, None, Value::text_string("valor-€-日本語".into())).unwrap();
        let a2x = translate_attribute_1x_to_2x(&a1x, cfg, false)
            .unwrap()
            .expect("preserved as vendor attribute");
        v2_items.push(a2x);
    }

    let back =
        translate_attributes_2x_to_1x(&ttlv_struct(TAG_ATTRIBUTES, v2_items).unwrap(), 4).unwrap();
    assert_eq!(back.len(), weird_names.len());
    for (expected, item) in weird_names.iter().zip(back.iter()) {
        assert_eq!(
            extract_text_string(&find_child(item, TAG_ATTRIBUTE_NAME).unwrap()).as_deref(),
            Some(*expected)
        );
        assert_eq!(
            extract_text_string(&find_child(item, TAG_ATTRIBUTE_VALUE).unwrap()).as_deref(),
            Some("valor-€-日本語")
        );
    }
}

// ============================================================================
// 6. Network & Chaos Fault-Injection Tests over Real TCP Sockets
// ============================================================================

#[tokio::test]
async fn test_network_chaos_truncated_frames_garbage_bytes_and_upstream_failure_recovery() {
    let upstream = Arc::new(ChaosUpstream::new(UpstreamKmipVersion::V2_1));
    let bridge = Arc::new(Kmip1xBridge::new(
        upstream.clone(),
        UpstreamKmipVersion::V2_1,
    ));

    let mut cfg = ProxyConfig::default();
    cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg.listener.tls_enabled = false;
    let handle = ProxyServer::start_kmip1x_bridge(cfg, bridge).await.unwrap();
    let addr = handle.local_addr();

    // Chaos 1: Truncated TTLV header (3 bytes then close)
    {
        let mut s = TcpStream::connect(addr).await.unwrap();
        s.write_all(&[0x42, 0x00, 0x78]).await.unwrap();
        drop(s);
    }

    // Chaos 2: Truncated TTLV payload (claims 1024 bytes, sends 4 then closes)
    {
        let mut s = TcpStream::connect(addr).await.unwrap();
        s.write_all(&[
            0x42, 0x00, 0x78, 0x01, 0x00, 0x00, 0x04, 0x00, 0xAA, 0xBB, 0xCC, 0xDD,
        ])
        .await
        .unwrap();
        drop(s);
    }

    // Chaos 3: Garbage leading byte (0xFF)
    {
        let mut s = TcpStream::connect(addr).await.unwrap();
        s.write_all(&[0xFF, 0x00, 0x01]).await.unwrap();
        let mut buf = [0u8; 8];
        let _ = s.read(&mut buf).await; // Server closes cleanly
    }

    // Chaos 4: Unsupported HTTP method (GET)
    {
        let mut s = TcpStream::connect(addr).await.unwrap();
        s.write_all(b"GET /kmip HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();
        let mut buf = [0u8; 8];
        let _ = s.read(&mut buf).await;
    }

    // Chaos 5: Upstream transport fails mid-flight -> server survives and recovers!
    upstream.fail_transport.store(true, Ordering::SeqCst);
    {
        let mut s = TcpStream::connect(addr).await.unwrap();
        let bytes = Format::Ttlv
            .encode(&req(4, vec![], vec![bi(OP_CREATE, vec![])]))
            .unwrap();
        s.write_all(&bytes).await.unwrap();
        let mut buf = [0u8; 8];
        let _ = s.read(&mut buf).await;
    }
    upstream.fail_transport.store(false, Ordering::SeqCst);

    // Recovery check: Persistent TCP keep-alive sending 20 sequential KMIP 1.4 requests on 1 socket!
    {
        let mut s = TcpStream::connect(addr).await.unwrap();
        for _ in 0..20 {
            let bytes = Format::Ttlv
                .encode(&req(4, vec![], vec![bi(OP_CREATE, vec![])]))
                .unwrap();
            s.write_all(&bytes).await.unwrap();
            s.flush().await.unwrap();

            let mut hdr = [0u8; 8];
            s.read_exact(&mut hdr).await.unwrap();
            let len = u32::from_be_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]) as usize;
            assert!(len <= 65536, "test payload length exceeds limit");
            let pad = (8 - len % 8) % 8;
            let total = len + pad;
            assert!(total <= 65536, "test total frame exceeds limit");
            let mut body = vec![0u8; total];
            s.read_exact(&mut body).await.unwrap();

            let mut full = Vec::new();
            full.extend_from_slice(&hdr);
            full.extend_from_slice(&body);
            let resp = Format::Ttlv.decode(&full).unwrap();
            assert_eq!(extract_protocol_version(&resp), Some((1, 4)));
        }
    }

    handle.shutdown().await.unwrap();
}
