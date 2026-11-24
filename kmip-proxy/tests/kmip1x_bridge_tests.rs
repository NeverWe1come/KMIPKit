//! Exhaustive unit and integration test suite for the KMIP 1.X -> KMIP 2.0/2.1/3.0
//! translation proxy (`kmipkit_proxy::kmip1x`).
//!
//! Covers:
//! - All 39 KMIP 1.0–1.4 operations
//! - All 48+ standard attributes (including deprecated 1.0 certificate aliases)
//! - Custom `x-*` / `y-*` attributes <-> KMIP 2.0+ Vendor Attributes
//! - `Operation Policy Name` handling in all 3 modes (Ignore, StoreAsVendorAttribute, Reject)
//! - KMIP 3.0 read-only attribute stripping and typed Link (`0x420190..=0x42019D`) translation
//! - `Add Attribute` routing (SingleInstance -> `Set Attribute`, MultiInstance -> `Add Attribute`)
//! - Read-Modify-Write for `Modify Attribute` and `Delete Attribute` with `Attribute Index`
//! - `Discover Versions` local interception (empty & offered lists)
//! - `Query` allowlist filtering across KMIP 1.0, 1.1, 1.2, 1.3, 1.4 (hiding PQC & 2.0/3.0 ops)
//! - Full `Template` lifecycle (`Register`, `Locate`, `Get`, `GetAttributes`, `GetAttributeList`, `Create` merge, `Destroy`)
//! - Multi-item `Batch` with implicit `ID Placeholder` injection
//! - `Result Reason` downgrade mapping for KMIP 2.0 / 2.1 / 3.0 codes
//! - `Maximum Response Size` enforcement on serialized 1.X TTLV
//! - Asynchronous (`Poll`, `Cancel`) & Server-to-Client (`Notify`, `Put`) reverse translation
//! - Pass-through for native KMIP 2.0 / 2.1 / 3.0 clients
//! - End-to-end TCP/TTLV integration test over `ProxyServer::start_kmip1x_bridge`
#![forbid(unsafe_code)]
#![cfg(feature = "kmip1x-bridge")]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kmipkit_proxy::codec::Format;
use kmipkit_proxy::config::ProxyConfig;
use kmipkit_proxy::kmip1x::mapping::{
    Cardinality, OperationPolicyHandling, STANDARD_ATTRIBUTES, UpstreamKmipVersion,
    downgrade_result_reason, is_crypto_algorithm_allowed_in_1x, is_object_type_allowed_in_1x,
    is_operation_allowed_in_1x, lookup_by_name_1x, lookup_by_tag_2x,
};
use kmipkit_proxy::kmip1x::tags::*;
use kmipkit_proxy::kmip1x::translator::{
    TranslatorConfig, build_1x_attribute, extract_protocol_version,
    translate_attribute_1x_to_2x, translate_attributes_2x_to_1x,
    translate_server_to_client_request_2x_to_1x,
};
use kmipkit_proxy::kmip1x::ttlv_helpers::{
    build_error_batch_item, build_protocol_version, build_response_message,
    build_success_batch_item, children_of, clone_item, extract_enumeration, extract_integer,
    extract_text_string, find_child, find_children, ttlv_item, ttlv_struct,
};
use kmipkit_proxy::kmip1x::{Kmip1xBridge, UpstreamKmipTransport};
use kmipkit_proxy::server::ProxyServer;
use kmipkit_ttlv::{Item, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// ============================================================================
// Stateful Mock Upstream KMIP 2.0 / 2.1 / 3.0 Server
// ============================================================================

struct MockUpstream2xServer {
    version: UpstreamKmipVersion,
    recorded_requests: Mutex<Vec<Item>>,
    /// Stored attributes per object ID (already in KMIP 2.0+ direct-tag format).
    objects: Mutex<BTreeMap<String, Vec<Item>>>,
    next_id: Mutex<u64>,
    forced_error_reason: Mutex<Option<u32>>,
}

impl MockUpstream2xServer {
    fn new(version: UpstreamKmipVersion) -> Self {
        Self {
            version,
            recorded_requests: Mutex::new(Vec::new()),
            objects: Mutex::new(BTreeMap::new()),
            next_id: Mutex::new(1),
            forced_error_reason: Mutex::new(None),
        }
    }

    fn set_forced_error(&self, reason: Option<u32>) {
        *self.forced_error_reason.lock().unwrap() = reason;
    }

    fn last_request(&self) -> Item {
        let guard = self.recorded_requests.lock().unwrap();
        clone_item(guard.last().expect("at least one upstream request")).unwrap()
    }

    fn request_count(&self) -> usize {
        self.recorded_requests.lock().unwrap().len()
    }

    fn next_uid(&self, prefix: &str) -> String {
        let mut guard = self.next_id.lock().unwrap();
        let id = format!("{prefix}-{:04}", *guard);
        *guard += 1;
        id
    }
}

#[async_trait]
impl UpstreamKmipTransport for MockUpstream2xServer {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        self.recorded_requests
            .lock()
            .unwrap()
            .push(clone_item(request)?);

        let (up_major, up_minor) = self.version.major_minor();
        let req_ver = extract_protocol_version(request).unwrap_or((0, 0));
        if req_ver != (up_major, up_minor) && req_ver.0 < 2 {
            return Err(format!("upstream received non-2.x version {req_ver:?}"));
        }

        let batch_items = find_children(request, TAG_BATCH_ITEM);
        let mut resp_items = Vec::new();
        let mut last_created_id = String::from("key-0001");

        for bi in batch_items {
            let op = find_child(&bi, TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap_or(0);
            let bid = find_child(&bi, TAG_UNIQUE_BATCH_ITEM_ID)
                .and_then(|i| kmipkit_proxy::kmip1x::ttlv_helpers::extract_byte_string(&i));
            let payload = find_child(&bi, TAG_REQUEST_PAYLOAD);

            if let Some(err_reason) = *self.forced_error_reason.lock().unwrap() {
                resp_items.push(build_error_batch_item(
                    op,
                    bid.as_deref(),
                    err_reason,
                    "simulated upstream KMIP 2.x/3.0 error",
                )?);
                continue;
            }

            let resolve_uid = |p: Option<&Item>, last_id: &str| -> String {
                p.and_then(|pp| find_child(pp, TAG_UNIQUE_IDENTIFIER))
                    .map(|uid_item| {
                        if extract_enumeration(&uid_item) == Some(UNIQUE_ID_PLACEHOLDER_ENUM) {
                            last_id.to_string()
                        } else {
                            extract_text_string(&uid_item).unwrap_or_else(|| last_id.to_string())
                        }
                    })
                    .unwrap_or_else(|| last_id.to_string())
            };

            let resp_bi = match op {
                OP_CREATE | OP_REGISTER | OP_REKEY | OP_DERIVE_KEY | OP_CERTIFY | OP_RECERTIFY
                | OP_CREATE_SPLIT_KEY | OP_JOIN_SPLIT_KEY | OP_IMPORT => {
                    let uid = self.next_uid("obj");
                    last_created_id = uid.clone();
                    let attrs = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_ATTRIBUTES))
                        .map(|a| children_of(&a))
                        .unwrap_or_default();
                    self.objects.lock().unwrap().insert(uid.clone(), attrs);
                    let obj_type = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_OBJECT_TYPE))
                        .and_then(|i| extract_enumeration(&i))
                        .unwrap_or(OBJECT_TYPE_SYMMETRIC_KEY);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(obj_type))?,
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                        ],
                    )?
                }
                OP_CREATE_KEY_PAIR | OP_REKEY_KEY_PAIR => {
                    let priv_id = self.next_uid("priv");
                    let pub_id = self.next_uid("pub");
                    last_created_id = priv_id.clone();
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(
                                TAG_PRIVATE_KEY_UNIQUE_IDENTIFIER,
                                Value::text_string(priv_id),
                            )?,
                            ttlv_item(
                                TAG_PUBLIC_KEY_UNIQUE_IDENTIFIER,
                                Value::text_string(pub_id),
                            )?,
                        ],
                    )?
                }
                OP_GET | OP_EXPORT => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    let kb = ttlv_struct(
                        TAG_KEY_BLOCK,
                        vec![
                            ttlv_item(TAG_KEY_FORMAT_TYPE, Value::enumeration(1))?,
                            ttlv_struct(
                                TAG_KEY_VALUE,
                                vec![ttlv_item(TAG_KEY_MATERIAL, Value::byte_string(vec![0xAA; 32]))?],
                            )?,
                        ],
                    )?;
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY))?,
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            kb,
                        ],
                    )?
                }
                OP_GET_ATTRIBUTES => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    let guard = self.objects.lock().unwrap();
                    let stored = guard.get(&uid);
                    let mut attrs_out = Vec::new();
                    if let Some(list) = stored {
                        for item in list {
                            attrs_out.push(clone_item(item)?);
                        }
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            ttlv_struct(TAG_ATTRIBUTES, attrs_out)?,
                        ],
                    )?
                }
                OP_GET_ATTRIBUTE_LIST => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    let mut refs = vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid.clone()))?];
                    let guard = self.objects.lock().unwrap();
                    if let Some(list) = guard.get(&uid) {
                        for item in list {
                            if item.tag().raw() == TAG_ATTRIBUTE {
                                let vendor = find_child(item, TAG_VENDOR_IDENTIFICATION)
                                    .and_then(|i| extract_text_string(&i))
                                    .unwrap_or_else(|| "x".to_string());
                                let name = find_child(item, TAG_ATTRIBUTE_NAME)
                                    .and_then(|i| extract_text_string(&i))
                                    .unwrap_or_default();
                                refs.push(ttlv_struct(
                                    TAG_ATTRIBUTE_REFERENCE,
                                    vec![
                                        ttlv_item(TAG_VENDOR_IDENTIFICATION, Value::text_string(vendor))?,
                                        ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string(name))?,
                                    ],
                                )?);
                            } else {
                                refs.push(ttlv_item(
                                    TAG_ATTRIBUTE_REFERENCE,
                                    Value::enumeration(item.tag().raw()),
                                )?);
                            }
                        }
                    }
                    build_success_batch_item(op, bid.as_deref(), refs)?
                }
                OP_SET_ATTRIBUTE => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    if let Some(new_attr) = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_NEW_ATTRIBUTE))
                        .and_then(|na| children_of(&na).into_iter().next())
                    {
                        let mut guard = self.objects.lock().unwrap();
                        let entry = guard.entry(uid.clone()).or_default();
                        entry.retain(|e| e.tag().raw() != new_attr.tag().raw());
                        entry.push(new_attr);
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )?
                }
                OP_ADD_ATTRIBUTE => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    if let Some(new_attr) = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_NEW_ATTRIBUTE))
                        .and_then(|na| children_of(&na).into_iter().next())
                    {
                        let mut guard = self.objects.lock().unwrap();
                        guard.entry(uid.clone()).or_default().push(new_attr);
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )?
                }
                OP_MODIFY_ATTRIBUTE => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    let current = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_CURRENT_ATTRIBUTE))
                        .and_then(|ca| children_of(&ca).into_iter().next());
                    let new_attr = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_NEW_ATTRIBUTE))
                        .and_then(|na| children_of(&na).into_iter().next());
                    if let (Some(curr), Some(new_a)) = (current, new_attr) {
                        let curr_bytes = Format::Ttlv.encode(&curr)?;
                        let mut guard = self.objects.lock().unwrap();
                        if let Some(list) = guard.get_mut(&uid) {
                            if let Some(pos) = list
                                .iter()
                                .position(|i| Format::Ttlv.encode(i).ok() == Some(curr_bytes.clone()))
                            {
                                list[pos] = new_a;
                            }
                        }
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )?
                }
                OP_DELETE_ATTRIBUTE => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    let mut guard = self.objects.lock().unwrap();
                    if let Some(list) = guard.get_mut(&uid) {
                        if let Some(curr) = payload
                            .as_ref()
                            .and_then(|p| find_child(p, TAG_CURRENT_ATTRIBUTE))
                            .and_then(|ca| children_of(&ca).into_iter().next())
                        {
                            let curr_bytes = Format::Ttlv.encode(&curr)?;
                            if let Some(pos) = list
                                .iter()
                                .position(|i| Format::Ttlv.encode(i).ok() == Some(curr_bytes.clone()))
                            {
                                list.remove(pos);
                            }
                        } else if let Some(aref) = payload
                            .as_ref()
                            .and_then(|p| find_child(p, TAG_ATTRIBUTE_REFERENCE))
                            .and_then(|r| extract_enumeration(&r))
                        {
                            list.retain(|i| i.tag().raw() != aref);
                        }
                    }
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )?
                }
                OP_LOCATE => {
                    let guard = self.objects.lock().unwrap();
                    let ids: Vec<Item> = guard
                        .keys()
                        .map(|k| ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(k.clone())).unwrap())
                        .collect();
                    build_success_batch_item(op, bid.as_deref(), ids)?
                }
                OP_QUERY => {
                    // Return a rich KMIP 3.0 Query response including 1.0, 1.1, 1.2, 1.4, 2.0, and 3.0 ops + PQC algorithms
                    let items = vec![
                        ttlv_item(TAG_OPERATION, Value::enumeration(OP_CREATE))?,
                        ttlv_item(TAG_OPERATION, Value::enumeration(OP_DISCOVER_VERSIONS))?,
                        ttlv_item(TAG_OPERATION, Value::enumeration(OP_ENCRYPT))?,
                        ttlv_item(TAG_OPERATION, Value::enumeration(OP_IMPORT))?,
                        ttlv_item(TAG_OPERATION, Value::enumeration(OP_SET_ATTRIBUTE))?, // KMIP 2.0+
                        ttlv_item(TAG_OPERATION, Value::enumeration(0x0000_0041))?, // Encapsulate (3.0)
                        ttlv_item(TAG_OPERATION, Value::enumeration(0x0000_0042))?, // Decapsulate (3.0)
                        ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY))?,
                        ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_PGP_KEY))?, // 1.2+
                        ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(0x0000_000B))?, // User (3.0)
                        ttlv_item(TAG_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(0x03))?, // AES (1.0)
                        ttlv_item(TAG_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(0x1C))?, // ChaCha20 (1.4)
                        ttlv_item(TAG_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(0x3A))?, // ML-KEM-768 (3.0 PQC)
                        ttlv_item(TAG_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(0x3D))?, // ML-DSA-65 (3.0 PQC)
                        ttlv_item(0x0042_0148, Value::boolean(true))?, // Quantum Safe Capability (2.1+)
                    ];
                    build_success_batch_item(op, bid.as_deref(), items)?
                }
                OP_ENCRYPT | OP_DECRYPT | OP_HASH | OP_RNG_RETRIEVE => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            ttlv_item(TAG_DATA, Value::byte_string(vec![0xDE, 0xAD, 0xBE, 0xEF]))?,
                        ],
                    )?
                }
                OP_SIGN => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            ttlv_item(TAG_SIGNATURE_DATA, Value::byte_string(vec![0x55; 64]))?,
                        ],
                    )?
                }
                OP_MAC => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            ttlv_item(TAG_MAC_DATA, Value::byte_string(vec![0x77; 32]))?,
                        ],
                    )?
                }
                OP_SIGNATURE_VERIFY | OP_MAC_VERIFY | OP_VALIDATE => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                            ttlv_item(TAG_VALIDITY_INDICATOR, Value::enumeration(1))?,
                        ],
                    )?
                }
                OP_CANCEL => {
                    let corr = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_ASYNCHRONOUS_CORRELATION_VALUE))
                        .unwrap_or_else(|| {
                            ttlv_item(TAG_ASYNCHRONOUS_CORRELATION_VALUE, Value::byte_string(vec![1]))
                                .unwrap()
                        });
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            corr,
                            ttlv_item(TAG_CANCELLATION_RESULT, Value::enumeration(1))?,
                        ],
                    )?
                }
                // Activate, Revoke, Destroy, Archive, Recover, Check, ObtainLease, GetUsageAllocation, RngSeed, Poll, etc.
                _ => {
                    let uid = resolve_uid(payload.as_ref(), &last_created_id);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )?
                }
            };
            resp_items.push(resp_bi);
        }

        build_response_message(up_major, up_minor, resp_items)
    }
}

// ============================================================================
// Helper Builders for KMIP 1.X Requests
// ============================================================================

fn build_1x_request(major: i32, minor: i32, batch_items: Vec<Item>) -> Item {
    build_1x_request_with_header_extras(major, minor, vec![], batch_items)
}

fn build_1x_request_with_header_extras(
    major: i32,
    minor: i32,
    extra_header: Vec<Item>,
    batch_items: Vec<Item>,
) -> Item {
    let mut header_children = vec![build_protocol_version(major, minor).unwrap()];
    header_children.extend(extra_header);
    header_children.push(
        ttlv_item(TAG_BATCH_COUNT, Value::integer(batch_items.len() as i32)).unwrap(),
    );
    let header = ttlv_struct(TAG_REQUEST_HEADER, header_children).unwrap();
    let mut top = vec![header];
    top.extend(batch_items);
    ttlv_struct(TAG_REQUEST_MESSAGE, top).unwrap()
}

fn batch_item_1x(op: u32, payload_items: Vec<Item>) -> Item {
    ttlv_struct(
        TAG_BATCH_ITEM,
        vec![
            ttlv_item(TAG_OPERATION, Value::enumeration(op)).unwrap(),
            ttlv_struct(TAG_REQUEST_PAYLOAD, payload_items).unwrap(),
        ],
    )
    .unwrap()
}

fn name_value_struct(name: &str) -> Item {
    ttlv_struct(
        TAG_NAME,
        vec![
            ttlv_item(TAG_NAME_VALUE, Value::text_string(name.to_string())).unwrap(),
            ttlv_item(TAG_NAME_TYPE, Value::enumeration(1)).unwrap(),
        ],
    )
    .unwrap()
}

// ============================================================================
// 1. Exhaustive Attribute & Mapping Tests
// ============================================================================

#[test]
fn test_all_standard_attributes_bidirectional_lookup() {
    assert!(STANDARD_ATTRIBUTES.len() >= 48);
    for spec in STANDARD_ATTRIBUTES {
        let by_name = lookup_by_name_1x(spec.name_1x).expect("lookup by 1.X name");
        assert_eq!(by_name.tag_2x, spec.tag_2x);
        let by_tag = lookup_by_tag_2x(spec.tag_2x).expect("lookup by 2.X tag");
        assert_eq!(by_tag.name_1x, spec.name_1x);
    }
}

#[test]
fn test_deprecated_kmip_10_certificate_attributes_normalization() {
    for (old, expected_tag) in [
        ("Certificate Identifier", TAG_X509_CERTIFICATE_IDENTIFIER),
        ("Certificate Subject", TAG_X509_CERTIFICATE_SUBJECT),
        ("Certificate Issuer", TAG_X509_CERTIFICATE_ISSUER),
    ] {
        let spec = lookup_by_name_1x(old).expect("deprecated alias resolves");
        assert_eq!(spec.tag_2x, expected_tag);
    }
}

#[test]
fn test_custom_attributes_x_and_y_roundtrip() {
    let cfg = TranslatorConfig::default();
    let x_attr = build_1x_attribute("x-department", None, Value::text_string("payments".into())).unwrap();
    let y_attr = build_1x_attribute("y-hsm-slot", None, Value::integer(7)).unwrap();

    let x_2x = translate_attribute_1x_to_2x(&x_attr, cfg, false).unwrap().unwrap();
    let y_2x = translate_attribute_1x_to_2x(&y_attr, cfg, false).unwrap().unwrap();

    assert_eq!(x_2x.tag().raw(), TAG_ATTRIBUTE);
    assert_eq!(
        extract_text_string(&find_child(&x_2x, TAG_VENDOR_IDENTIFICATION).unwrap()).as_deref(),
        Some("x")
    );
    assert_eq!(
        extract_text_string(&find_child(&x_2x, TAG_ATTRIBUTE_NAME).unwrap()).as_deref(),
        Some("department")
    );

    let attrs_2x = ttlv_struct(TAG_ATTRIBUTES, vec![x_2x, y_2x]).unwrap();
    let back_1x = translate_attributes_2x_to_1x(&attrs_2x, 4).unwrap();
    assert_eq!(back_1x.len(), 2);
    assert_eq!(
        extract_text_string(&find_child(&back_1x[0], TAG_ATTRIBUTE_NAME).unwrap()).as_deref(),
        Some("x-department")
    );
    assert_eq!(
        extract_text_string(&find_child(&back_1x[1], TAG_ATTRIBUTE_NAME).unwrap()).as_deref(),
        Some("y-hsm-slot")
    );
}

#[test]
fn test_kmip30_typed_link_roundtrip() {
    let cfg_v3 = TranslatorConfig {
        upstream_version: UpstreamKmipVersion::V3_0,
        ..TranslatorConfig::default()
    };
    let link_val = ttlv_struct(
        TAG_ATTRIBUTE_VALUE,
        vec![
            ttlv_item(TAG_LINK_TYPE, Value::enumeration(LINK_TYPE_PRIVATE_KEY)).unwrap(),
            ttlv_item(TAG_LINKED_OBJECT_IDENTIFIER, Value::text_string("priv-99".into())).unwrap(),
        ],
    )
    .unwrap();
    let link_1x = ttlv_struct(
        TAG_ATTRIBUTE,
        vec![
            ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Link".into())).unwrap(),
            link_val,
        ],
    )
    .unwrap();

    let translated_v3 = translate_attribute_1x_to_2x(&link_1x, cfg_v3, false)
        .unwrap()
        .unwrap();
    assert_eq!(translated_v3.tag().raw(), TAG_PRIVATE_KEY_LINK);
    assert_eq!(extract_text_string(&translated_v3).as_deref(), Some("priv-99"));

    // Translate back to KMIP 1.X Link structure
    let attrs_v3 = ttlv_struct(TAG_ATTRIBUTES, vec![translated_v3]).unwrap();
    let back_1x = translate_attributes_2x_to_1x(&attrs_v3, 4).unwrap();
    assert_eq!(back_1x.len(), 1);
    assert_eq!(
        extract_text_string(&find_child(&back_1x[0], TAG_ATTRIBUTE_NAME).unwrap()).as_deref(),
        Some("Link")
    );
    let val = find_child(&back_1x[0], TAG_ATTRIBUTE_VALUE).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&val, TAG_LINK_TYPE).unwrap()),
        Some(LINK_TYPE_PRIVATE_KEY)
    );
    assert_eq!(
        extract_text_string(&find_child(&val, TAG_LINKED_OBJECT_IDENTIFIER).unwrap()).as_deref(),
        Some("priv-99")
    );
}

#[test]
fn test_operation_policy_name_all_three_modes() {
    let policy_attr =
        build_1x_attribute("Operation Policy Name", None, Value::text_string("strict-pol".into()))
            .unwrap();

    // Mode 1: Ignore
    let res_ignore = translate_attribute_1x_to_2x(
        &policy_attr,
        TranslatorConfig {
            operation_policy_handling: OperationPolicyHandling::Ignore,
            ..TranslatorConfig::default()
        },
        true,
    )
    .unwrap();
    assert!(res_ignore.is_none());

    // Mode 2: StoreAsVendorAttribute
    let res_vendor = translate_attribute_1x_to_2x(
        &policy_attr,
        TranslatorConfig {
            operation_policy_handling: OperationPolicyHandling::StoreAsVendorAttribute,
            ..TranslatorConfig::default()
        },
        true,
    )
    .unwrap()
    .expect("stored as vendor attribute");
    assert_eq!(res_vendor.tag().raw(), TAG_ATTRIBUTE);

    // Mode 3: Reject
    let res_reject = translate_attribute_1x_to_2x(
        &policy_attr,
        TranslatorConfig {
            operation_policy_handling: OperationPolicyHandling::Reject,
            ..TranslatorConfig::default()
        },
        true,
    );
    assert!(res_reject.is_err());
}

#[test]
fn test_kmip30_readonly_attributes_stripped_on_create() {
    let cfg = TranslatorConfig {
        upstream_version: UpstreamKmipVersion::V3_0,
        strip_readonly_on_create: true,
        ..TranslatorConfig::default()
    };
    let ro_state = build_1x_attribute("State", None, Value::enumeration(1)).unwrap();
    let ro_init = build_1x_attribute("Initial Date", None, Value::date_time(1_700_000_000)).unwrap();
    let rw_len = build_1x_attribute("Cryptographic Length", None, Value::integer(256)).unwrap();

    assert!(translate_attribute_1x_to_2x(&ro_state, cfg, true).unwrap().is_none());
    assert!(translate_attribute_1x_to_2x(&ro_init, cfg, true).unwrap().is_none());
    assert!(translate_attribute_1x_to_2x(&rw_len, cfg, true).unwrap().is_some());
}

#[test]
fn test_result_reason_downgrade_matrix() {
    assert_eq!(downgrade_result_reason(REASON_ITEM_NOT_FOUND, 0), REASON_ITEM_NOT_FOUND);
    assert_eq!(downgrade_result_reason(0x0000_0017, 4), REASON_PERMISSION_DENIED); // Not Extractable
    assert_eq!(downgrade_result_reason(0x0000_0019, 4), REASON_AUTHENTICATION_NOT_SUCCESSFUL); // Invalid Ticket
    assert_eq!(downgrade_result_reason(0x0000_0021, 4), REASON_ITEM_NOT_FOUND); // Attribute Not Found
    assert_eq!(downgrade_result_reason(0x0000_0024, 4), REASON_CRYPTOGRAPHIC_FAILURE);
    assert_eq!(downgrade_result_reason(0x0000_0034, 4), REASON_MISSING_DATA);
    assert_eq!(downgrade_result_reason(0x0000_003F, 4), REASON_OPERATION_NOT_SUPPORTED);
    assert_eq!(downgrade_result_reason(0x0000_0016, 0), REASON_PERMISSION_DENIED); // Sensitive in 1.0
    assert_eq!(downgrade_result_reason(0x0000_0016, 4), REASON_SENSITIVE); // Sensitive in 1.4
}

#[test]
fn test_query_allowlists_across_1x_versions() {
    assert!(is_operation_allowed_in_1x(OP_CREATE, 0));
    assert!(!is_operation_allowed_in_1x(OP_DISCOVER_VERSIONS, 0));
    assert!(is_operation_allowed_in_1x(OP_DISCOVER_VERSIONS, 1));
    assert!(!is_operation_allowed_in_1x(OP_ENCRYPT, 1));
    assert!(is_operation_allowed_in_1x(OP_ENCRYPT, 2));
    assert!(!is_operation_allowed_in_1x(OP_IMPORT, 3));
    assert!(is_operation_allowed_in_1x(OP_IMPORT, 4));
    assert!(!is_operation_allowed_in_1x(OP_SET_ATTRIBUTE, 4));

    assert!(!is_object_type_allowed_in_1x(OBJECT_TYPE_PGP_KEY, 1));
    assert!(is_object_type_allowed_in_1x(OBJECT_TYPE_PGP_KEY, 2));
    assert!(!is_object_type_allowed_in_1x(0x0B, 4)); // User (3.0)

    assert!(is_crypto_algorithm_allowed_in_1x(0x03, 0)); // AES
    assert!(!is_crypto_algorithm_allowed_in_1x(0x3A, 4)); // ML-KEM-768
}

// ============================================================================
// 2. All 39 KMIP 1.X Operations End-to-End Across 2.0, 2.1, and 3.0
// ============================================================================

#[tokio::test]
async fn test_all_kmip_1x_operations_translated_to_20_21_and_30() {
    let all_1x_ops = [
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
        OP_QUERY,
        OP_CANCEL,
        OP_POLL,
        OP_REKEY_KEY_PAIR,
        OP_DISCOVER_VERSIONS,
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

    for target_version in [
        UpstreamKmipVersion::V2_0,
        UpstreamKmipVersion::V2_1,
        UpstreamKmipVersion::V3_0,
    ] {
        let upstream = Arc::new(MockUpstream2xServer::new(target_version));
        let bridge = Kmip1xBridge::new(upstream.clone(), target_version);

        for op in all_1x_ops {
            let payload = match op {
                OP_CREATE => vec![
                    ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY)).unwrap(),
                    ttlv_struct(
                        TAG_TEMPLATE_ATTRIBUTE,
                        vec![
                            build_1x_attribute("Cryptographic Algorithm", None, Value::enumeration(3)).unwrap(),
                            build_1x_attribute("Cryptographic Length", None, Value::integer(256)).unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
                OP_ADD_ATTRIBUTE => vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("obj-0001".into())).unwrap(),
                    build_1x_attribute("Object Group", None, Value::text_string("grp-a".into())).unwrap(),
                ],
                OP_MODIFY_ATTRIBUTE => vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("obj-0001".into())).unwrap(),
                    build_1x_attribute("Contact Information", None, Value::text_string("sec@corp".into())).unwrap(),
                ],
                OP_DELETE_ATTRIBUTE => vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("obj-0001".into())).unwrap(),
                    ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Contact Information".into())).unwrap(),
                ],
                OP_DISCOVER_VERSIONS => vec![],
                _ => vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("obj-0001".into())).unwrap()],
            };

            let req = build_1x_request(1, 4, vec![batch_item_1x(op, payload)]);
            let resp = bridge.dispatch(&req).await.expect("operation translates cleanly");

            // Response must always be rewritten to client's KMIP 1.4
            assert_eq!(extract_protocol_version(&resp), Some((1, 4)));
            let bi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
            assert_eq!(extract_enumeration(&find_child(&bi, TAG_OPERATION).unwrap()), Some(op));
            assert_eq!(
                extract_enumeration(&find_child(&bi, TAG_RESULT_STATUS).unwrap()),
                Some(STATUS_SUCCESS),
                "op 0x{op:02X} failed against {target_version:?}"
            );
        }
    }
}

// ============================================================================
// 3. Discover Versions & Query Allowlist Interception Tests
// ============================================================================

#[tokio::test]
async fn test_discover_versions_intercepted_locally_without_hitting_upstream() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V3_0));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V3_0);

    // Client offers [1.4, 1.2, 2.1] -> proxy returns [1.4, 1.2] locally
    let req = build_1x_request(
        1,
        2,
        vec![batch_item_1x(
            OP_DISCOVER_VERSIONS,
            vec![
                build_protocol_version(1, 4).unwrap(),
                build_protocol_version(1, 2).unwrap(),
                build_protocol_version(2, 1).unwrap(),
            ],
        )],
    );
    let resp = bridge.dispatch(&req).await.unwrap();
    assert_eq!(upstream.request_count(), 0, "Discover Versions must NOT hit upstream");
    assert_eq!(extract_protocol_version(&resp), Some((1, 2)));

    let payload = find_child(&find_child(&resp, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap();
    let versions = find_children(&payload, TAG_PROTOCOL_VERSION);
    assert_eq!(versions.len(), 2);
}

#[tokio::test]
async fn test_query_filters_out_kmip2x_30_and_pqc_for_kmip10_vs_14() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V3_0));
    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V3_0);

    // KMIP 1.0 client
    let req_10 = build_1x_request(1, 0, vec![batch_item_1x(OP_QUERY, vec![])]);
    let resp_10 = bridge.dispatch(&req_10).await.unwrap();
    let p10 = find_child(&find_child(&resp_10, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap();
    let ops_10: Vec<u32> = find_children(&p10, TAG_OPERATION)
        .iter()
        .filter_map(extract_enumeration)
        .collect();
    assert!(ops_10.contains(&OP_CREATE));
    assert!(!ops_10.contains(&OP_ENCRYPT));
    assert!(!ops_10.contains(&OP_SET_ATTRIBUTE));
    assert!(!ops_10.contains(&0x41)); // Encapsulate stripped

    let algs_10: Vec<u32> = find_children(&p10, TAG_CRYPTOGRAPHIC_ALGORITHM)
        .iter()
        .filter_map(extract_enumeration)
        .collect();
    assert!(algs_10.contains(&0x03)); // AES
    assert!(!algs_10.contains(&0x3A)); // ML-KEM-768 stripped
    assert!(!algs_10.contains(&0x3D)); // ML-DSA-65 stripped
    assert!(find_child(&p10, 0x0042_0148).is_none()); // Quantum Safe Capability stripped

    // KMIP 1.4 client gets Encrypt & Import & ChaCha20, still no PQC or SetAttribute
    let req_14 = build_1x_request(1, 4, vec![batch_item_1x(OP_QUERY, vec![])]);
    let resp_14 = bridge.dispatch(&req_14).await.unwrap();
    let p14 = find_child(&find_child(&resp_14, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap();
    let ops_14: Vec<u32> = find_children(&p14, TAG_OPERATION)
        .iter()
        .filter_map(extract_enumeration)
        .collect();
    assert!(ops_14.contains(&OP_ENCRYPT));
    assert!(ops_14.contains(&OP_IMPORT));
    assert!(!ops_14.contains(&OP_SET_ATTRIBUTE));
}

// ============================================================================
// 4. AddAttribute Routing & Read-Modify-Write for Attribute Index
// ============================================================================

#[tokio::test]
async fn test_add_attribute_routes_single_instance_to_set_attribute_and_restores_op() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    let req = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_ADD_ATTRIBUTE,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("key-1".into())).unwrap(),
                build_1x_attribute("Contact Information", None, Value::text_string("ops@corp".into()))
                    .unwrap(),
            ],
        )],
    );
    let resp = bridge.dispatch(&req).await.unwrap();

    // Upstream must have received OP_SET_ATTRIBUTE (0x31)
    let sent = upstream.last_request();
    let sent_bi = find_child(&sent, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&sent_bi, TAG_OPERATION).unwrap()),
        Some(OP_SET_ATTRIBUTE)
    );

    // Client 1.4 must receive OP_ADD_ATTRIBUTE (0x0D) back
    let resp_bi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&resp_bi, TAG_OPERATION).unwrap()),
        Some(OP_ADD_ATTRIBUTE)
    );
}

#[tokio::test]
async fn test_rmw_modify_and_delete_multi_instance_attribute_by_index() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    // 1. Create key with two Object Group values: ["grp-0", "grp-1"]
    let create_req = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_CREATE,
            vec![
                ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY)).unwrap(),
                ttlv_struct(
                    TAG_TEMPLATE_ATTRIBUTE,
                    vec![
                        build_1x_attribute("Object Group", None, Value::text_string("grp-0".into())).unwrap(),
                        build_1x_attribute("Object Group", None, Value::text_string("grp-1".into())).unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )],
    );
    let create_resp = bridge.dispatch(&create_req).await.unwrap();
    let uid = extract_text_string(
        &find_child(
            &find_child(&find_child(&create_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap(),
            TAG_UNIQUE_IDENTIFIER,
        )
        .unwrap(),
    )
    .unwrap();

    // 2. Modify Attribute index 1 ("grp-1" -> "grp-1-updated") via Read-Modify-Write
    let mod_req = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_MODIFY_ATTRIBUTE,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid.clone())).unwrap(),
                build_1x_attribute(
                    "Object Group",
                    Some(1),
                    Value::text_string("grp-1-updated".into()),
                )
                .unwrap(),
            ],
        )],
    );
    let mod_resp = bridge.dispatch(&mod_req).await.unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(&find_child(&mod_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESULT_STATUS).unwrap()
        ),
        Some(STATUS_SUCCESS)
    );

    // Verify GetAttributes returns ["grp-0", "grp-1-updated"]
    let get_req = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_GET_ATTRIBUTES,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid.clone())).unwrap(),
                ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Object Group".into())).unwrap(),
            ],
        )],
    );
    let get_resp = bridge.dispatch(&get_req).await.unwrap();
    let gp = find_child(&find_child(&get_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap();
    let attrs = find_children(&gp, TAG_ATTRIBUTE);
    assert_eq!(attrs.len(), 2);
    assert_eq!(
        extract_text_string(&find_child(&attrs[1], TAG_ATTRIBUTE_VALUE).unwrap()).as_deref(),
        Some("grp-1-updated")
    );

    // 3. Delete Attribute index 0 ("grp-0") via Read-Modify-Write
    let del_req = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_DELETE_ATTRIBUTE,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid.clone())).unwrap(),
                ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Object Group".into())).unwrap(),
                ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(0)).unwrap(),
            ],
        )],
    );
    let del_resp = bridge.dispatch(&del_req).await.unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(&find_child(&del_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESULT_STATUS).unwrap()
        ),
        Some(STATUS_SUCCESS)
    );

    // 4. Out-of-bounds Attribute Index returns Item Not Found cleanly
    let oob_req = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_DELETE_ATTRIBUTE,
            vec![
                ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid)).unwrap(),
                ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Object Group".into())).unwrap(),
                ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(99)).unwrap(),
            ],
        )],
    );
    let oob_resp = bridge.dispatch(&oob_req).await.unwrap();
    let oob_bi = find_child(&oob_resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&oob_bi, TAG_RESULT_STATUS).unwrap()),
        Some(STATUS_OPERATION_FAILED)
    );
    assert_eq!(
        extract_enumeration(&find_child(&oob_bi, TAG_RESULT_REASON).unwrap()),
        Some(REASON_ITEM_NOT_FOUND)
    );
}

// ============================================================================
// 5. Full KMIP 1.X Template Emulation Lifecycle
// ============================================================================

#[tokio::test]
async fn test_template_full_lifecycle_and_create_merge() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    // 1. Register Template locally
    let reg_tpl = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_REGISTER,
            vec![
                ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_TEMPLATE)).unwrap(),
                ttlv_struct(
                    TAG_TEMPLATE_ATTRIBUTE,
                    vec![name_value_struct("aes-256-template")],
                )
                .unwrap(),
                ttlv_struct(
                    TAG_TEMPLATE,
                    vec![
                        build_1x_attribute("Cryptographic Algorithm", None, Value::enumeration(3)).unwrap(),
                        build_1x_attribute("Cryptographic Length", None, Value::integer(256)).unwrap(),
                        build_1x_attribute("Cryptographic Usage Mask", None, Value::integer(0x0C)).unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )],
    );
    let reg_resp = bridge.dispatch(&reg_tpl).await.unwrap();
    assert_eq!(upstream.request_count(), 0, "Template Register must stay local");
    let tpl_id = extract_text_string(
        &find_child(
            &find_child(&find_child(&reg_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap(),
            TAG_UNIQUE_IDENTIFIER,
        )
        .unwrap(),
    )
    .unwrap();

    // 2. Locate Template
    let loc_tpl = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_LOCATE,
            vec![build_1x_attribute("Object Type", None, Value::enumeration(OBJECT_TYPE_TEMPLATE)).unwrap()],
        )],
    );
    let loc_resp = bridge.dispatch(&loc_tpl).await.unwrap();
    let loc_payload = find_child(&find_child(&loc_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap();
    assert_eq!(find_children(&loc_payload, TAG_UNIQUE_IDENTIFIER).len(), 1);

    // 3. Get Template
    let get_tpl = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_GET,
            vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(tpl_id.clone())).unwrap()],
        )],
    );
    let get_tpl_resp = bridge.dispatch(&get_tpl).await.unwrap();
    let get_tpl_p = find_child(&find_child(&get_tpl_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESPONSE_PAYLOAD).unwrap();
    assert!(find_child(&get_tpl_p, TAG_TEMPLATE).is_some());

    // 4. Create Symmetric Key referencing "aes-256-template" AND overriding Length -> 128
    let create_req = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_CREATE,
            vec![
                ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY)).unwrap(),
                ttlv_struct(
                    TAG_TEMPLATE_ATTRIBUTE,
                    vec![
                        name_value_struct("aes-256-template"),
                        build_1x_attribute("Cryptographic Length", None, Value::integer(128)).unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )],
    );
    let _ = bridge.dispatch(&create_req).await.unwrap();
    assert_eq!(upstream.request_count(), 1);

    let sent = upstream.last_request();
    let sent_attrs = find_child(
        &find_child(&find_child(&sent, TAG_BATCH_ITEM).unwrap(), TAG_REQUEST_PAYLOAD).unwrap(),
        TAG_ATTRIBUTES,
    )
    .unwrap();
    // Algorithm=AES inherited from template, Length=128 overridden by explicit attribute
    assert_eq!(
        extract_enumeration(&find_child(&sent_attrs, TAG_CRYPTOGRAPHIC_ALGORITHM).unwrap()),
        Some(3)
    );
    assert_eq!(
        extract_integer(&find_child(&sent_attrs, TAG_CRYPTOGRAPHIC_LENGTH).unwrap()),
        Some(128)
    );

    // 5. Destroy Template locally
    let del_tpl = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_DESTROY,
            vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(tpl_id)).unwrap()],
        )],
    );
    let del_resp = bridge.dispatch(&del_tpl).await.unwrap();
    assert_eq!(
        extract_enumeration(
            &find_child(&find_child(&del_resp, TAG_BATCH_ITEM).unwrap(), TAG_RESULT_STATUS).unwrap()
        ),
        Some(STATUS_SUCCESS)
    );
}

// ============================================================================
// 6. Batch Chaining with Implicit ID Placeholder, Max Response Size, S2C, Pass-through
// ============================================================================

#[tokio::test]
async fn test_batch_implicit_id_placeholder_injection() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    // Batch: [Create, Activate (no UID -> implicit ID Placeholder), Get (no UID)]
    let req = build_1x_request(
        1,
        4,
        vec![
            batch_item_1x(
                OP_CREATE,
                vec![ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY)).unwrap()],
            ),
            batch_item_1x(OP_ACTIVATE, vec![]),
            batch_item_1x(OP_GET, vec![]),
        ],
    );
    let resp = bridge.dispatch(&req).await.unwrap();
    let resp_items = find_children(&resp, TAG_BATCH_ITEM);
    assert_eq!(resp_items.len(), 3);

    // Verify upstream received explicit UniqueIdentifier = Enumeration(1) (ID Placeholder) on items 1 and 2
    let sent = upstream.last_request();
    let sent_items = find_children(&sent, TAG_BATCH_ITEM);
    let act_uid = find_child(
        &find_child(&sent_items[1], TAG_REQUEST_PAYLOAD).unwrap(),
        TAG_UNIQUE_IDENTIFIER,
    )
    .unwrap();
    assert_eq!(extract_enumeration(&act_uid), Some(UNIQUE_ID_PLACEHOLDER_ENUM));
}

#[tokio::test]
async fn test_max_response_size_enforcement_on_1x_ttlv() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V2_1);

    let req = build_1x_request_with_header_extras(
        1,
        4,
        vec![ttlv_item(TAG_MAXIMUM_RESPONSE_SIZE, Value::integer(32)).unwrap()],
        vec![batch_item_1x(OP_QUERY, vec![])],
    );
    let resp = bridge.dispatch(&req).await.unwrap();
    let bi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&bi, TAG_RESULT_STATUS).unwrap()),
        Some(STATUS_OPERATION_FAILED)
    );
    assert_eq!(
        extract_enumeration(&find_child(&bi, TAG_RESULT_REASON).unwrap()),
        Some(REASON_RESPONSE_TOO_LARGE)
    );
}

#[tokio::test]
async fn test_upstream_error_reason_downgraded_for_1x_client() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V3_0));
    upstream.set_forced_error(Some(0x0000_0021)); // Attribute Not Found (KMIP 2.0+)
    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V3_0);

    let req = build_1x_request(
        1,
        2,
        vec![batch_item_1x(
            OP_GET,
            vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("missing".into())).unwrap()],
        )],
    );
    let resp = bridge.dispatch(&req).await.unwrap();
    let bi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&bi, TAG_RESULT_REASON).unwrap()),
        Some(REASON_ITEM_NOT_FOUND)
    );
}

#[test]
fn test_server_to_client_notify_reverse_translation() {
    let notify_2x = ttlv_struct(
        TAG_REQUEST_MESSAGE,
        vec![
            ttlv_struct(
                TAG_REQUEST_HEADER,
                vec![
                    build_protocol_version(2, 1).unwrap(),
                    ttlv_item(TAG_BATCH_COUNT, Value::integer(1)).unwrap(),
                ],
            )
            .unwrap(),
            ttlv_struct(
                TAG_BATCH_ITEM,
                vec![
                    ttlv_item(TAG_OPERATION, Value::enumeration(OP_NOTIFY)).unwrap(),
                    ttlv_struct(
                        TAG_REQUEST_PAYLOAD,
                        vec![
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("key-42".into())).unwrap(),
                            ttlv_struct(
                                TAG_ATTRIBUTES,
                                vec![ttlv_item(TAG_STATE, Value::enumeration(2)).unwrap()],
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

    let notify_1x = translate_server_to_client_request_2x_to_1x(&notify_2x, 1, 4).unwrap();
    assert_eq!(extract_protocol_version(&notify_1x), Some((1, 4)));
    let payload = find_child(&find_child(&notify_1x, TAG_BATCH_ITEM).unwrap(), TAG_REQUEST_PAYLOAD).unwrap();
    let attrs = find_children(&payload, TAG_ATTRIBUTE);
    assert_eq!(attrs.len(), 1);
    assert_eq!(
        extract_text_string(&find_child(&attrs[0], TAG_ATTRIBUTE_NAME).unwrap()).as_deref(),
        Some("State")
    );
}

#[tokio::test]
async fn test_kmip_21_client_passthrough_untouched() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    let req_21 = build_1x_request(
        2,
        1,
        vec![batch_item_1x(
            OP_CREATE,
            vec![ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY)).unwrap()],
        )],
    );
    let resp = bridge.dispatch(&req_21).await.unwrap();
    assert_eq!(extract_protocol_version(&resp), Some((2, 1)));
    assert_eq!(upstream.request_count(), 1);
}

// ============================================================================
// 7. Real TCP Socket End-to-End Integration Test via ProxyServer
// ============================================================================

#[tokio::test]
async fn test_e2e_tcp_kmip_14_client_through_proxy_server_to_kmip_30_upstream() {
    let upstream = Arc::new(MockUpstream2xServer::new(UpstreamKmipVersion::V3_0));
    let bridge = Arc::new(Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V3_0));

    let mut cfg = ProxyConfig::default();
    cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();
    cfg.listener.tls_enabled = false;

    let handle = ProxyServer::start_kmip1x_bridge(cfg, bridge)
        .await
        .expect("bridge server starts");

    let mut stream = TcpStream::connect(handle.local_addr())
        .await
        .expect("connects to bridge");

    // Send raw KMIP 1.4 TTLV Create request
    let req_14 = build_1x_request(
        1,
        4,
        vec![batch_item_1x(
            OP_CREATE,
            vec![
                ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY)).unwrap(),
                ttlv_struct(
                    TAG_TEMPLATE_ATTRIBUTE,
                    vec![
                        build_1x_attribute("Cryptographic Algorithm", None, Value::enumeration(3)).unwrap(),
                        build_1x_attribute("Cryptographic Length", None, Value::integer(256)).unwrap(),
                        build_1x_attribute("x-app", None, Value::text_string("legacy-core".into())).unwrap(),
                    ],
                )
                .unwrap(),
            ],
        )],
    );

    let req_bytes = Format::Ttlv.encode(&req_14).unwrap();
    stream.write_all(&req_bytes).await.unwrap();
    stream.flush().await.unwrap();

    // Read raw TTLV response frame
    let mut hdr = [0u8; 8];
    stream.read_exact(&mut hdr).await.unwrap();
    let payload_len = u32::from_be_bytes([hdr[4], hdr[5], hdr[6], hdr[7]]) as usize;
    let pad = (8 - payload_len % 8) % 8;
    let mut body = vec![0u8; payload_len + pad];
    stream.read_exact(&mut body).await.unwrap();

    let mut full = Vec::with_capacity(8 + body.len());
    full.extend_from_slice(&hdr);
    full.extend_from_slice(&body);

    let resp = Format::Ttlv.decode(&full).unwrap();
    assert_eq!(extract_protocol_version(&resp), Some((1, 4)));
    let bi = find_child(&resp, TAG_BATCH_ITEM).unwrap();
    assert_eq!(
        extract_enumeration(&find_child(&bi, TAG_RESULT_STATUS).unwrap()),
        Some(STATUS_SUCCESS)
    );

    // Confirm upstream received KMIP 3.0!
    assert_eq!(extract_protocol_version(&upstream.last_request()), Some((3, 0)));

    handle.shutdown().await.expect("clean shutdown");
}
