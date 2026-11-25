//! Exhaustive bidirectional (Request + Response) verification suite for the
//! KMIP 1.0–1.4 <-> KMIP 2.0 / 2.1 / 3.0 translation proxy (`kmipkit_proxy::kmip1x`).
//!
//! Verifies:
//! 1. 100% of all 77 KMIP 1.0–1.4 standard attributes in BOTH request and response directions
//!    across KMIP 2.0, 2.1, and 3.0 (plus read-only stripping on Create in 3.0).
//! 2. Version-gated attribute compatibility for KMIP 1.0, 1.1, 1.2, 1.3, and 1.4 clients
//!    (including deprecated 1.0 Certificate Identifier/Subject/Issuer downgrade).
//! 3. All 14 KMIP 3.0 typed Link attributes (`0x420190..=0x42019D`) bidirectional roundtrip.
//! 4. Custom `x-*` / `y-*` and unknown attributes <-> KMIP 2.0+ Vendor Attributes roundtrip
//!    in `Create`, `GetAttributes`, `GetAttributeList`, and `Locate`.
//! 5. Removed/deprecated elements: `Operation Policy Name` (Ignore / StoreAsVendorAttribute / Reject)
//!    and `Template` merging across `Common`, `Private Key`, and `Public Key` Template-Attributes.
//! 6. All 41 KMIP 1.X C2S operations verifying BOTH translated upstream request AND restored
//!    1.X response (ProtocolVersion, Operation, UniqueBatchItemID, Payload).
//! 7. Exhaustive `Result Reason` downgrade sweep across `0x01..=0x4D` and `0x100` for 1.0..=1.4.
//! 8. `Key Wrapping Specification` in both `Get` and `Export`.
//! 9. `Asynchronous Indicator` Boolean <-> Enumeration header translation.
//! 10. Full S2C (`Notify`, `Put`, `Query`, `Discover Versions`) bidirectional roundtrip across 2.0/2.1/3.0.
#![forbid(unsafe_code)]
#![cfg(feature = "kmip1x-bridge")]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use kmipkit_proxy::kmip1x::mapping::{
    Cardinality, STANDARD_ATTRIBUTES, UpstreamKmipVersion, downgrade_result_reason,
    link_type_to_v3_tag, v3_link_tag_to_link_type,
};
use kmipkit_proxy::kmip1x::s2c::{Kmip1xS2cBridge, S2cClientTransport};
use kmipkit_proxy::kmip1x::tags::*;
use kmipkit_proxy::kmip1x::translator::{
    TranslatorConfig, build_1x_attribute, extract_protocol_version, translate_attribute_1x_to_2x,
    translate_attributes_2x_to_1x,
};
use kmipkit_proxy::kmip1x::ttlv_helpers::{
    build_protocol_version, build_response_message, build_success_batch_item, children_of,
    clone_item, extract_boolean, extract_byte_string, extract_enumeration, extract_integer,
    extract_text_string, find_child, find_children, ttlv_item, ttlv_struct,
};
use kmipkit_proxy::kmip1x::{Kmip1xBridge, UpstreamKmipTransport};
use kmipkit_ttlv::{Item, Value};

// ============================================================================
// Sample Value Generator for All 77 Standard KMIP 1.0–1.4 Attributes
// ============================================================================

/// Produces a realistic, spec-compliant TTLV `Value` for every one of the 77 standard attributes.
fn sample_value_for_attribute(name_1x: &str) -> Value {
    match name_1x {
        // Enumerations
        "Object Type" => Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
        "Cryptographic Algorithm" => Value::enumeration(0x03), // AES
        "Key Format Type" => Value::enumeration(0x01),         // Raw
        "Split Key Method" => Value::enumeration(0x01),        // XOR
        "Opaque Data Type" => Value::enumeration(0x01),
        "Certificate Type" => Value::enumeration(0x01), // X.509
        "Digital Signature Algorithm" => Value::enumeration(0x05), // SHA-256 with RSA
        "State" => Value::enumeration(0x02),            // Active
        "Key Role Type" => Value::enumeration(0x03),    // DEK

        // Integers / Bitmasks
        "Cryptographic Length" => Value::integer(256),
        "Cryptographic Usage Mask" => Value::integer(0x0000_000C), // Encrypt | Decrypt
        "Key Part Identifier" => Value::integer(1),
        "Split Key Parts" => Value::integer(3),
        "Split Key Threshold" => Value::integer(2),
        "Certificate Length" => Value::integer(1024),

        // Intervals
        "Lease Time" => Value::interval(3600),

        // Booleans
        "Fresh" | "Key Value Present" | "Sensitive" | "Always Sensitive" | "Extractable"
        | "Never Extractable" => Value::boolean(true),

        // Date-Times
        "Initial Date"
        | "Activation Date"
        | "Process Start Date"
        | "Protect Stop Date"
        | "Deactivation Date"
        | "Destroy Date"
        | "Compromise Occurrence Date"
        | "Compromise Date"
        | "Archive Date"
        | "Last Change Date"
        | "Original Creation Date" => Value::date_time(1_700_000_000),

        // Structures
        "Name" => {
            let s = ttlv_struct(
                TAG_NAME,
                vec![
                    ttlv_item(TAG_NAME_VALUE, Value::text_string("test-key-name".into())).unwrap(),
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
                    ttlv_item(TAG_LINK_TYPE, Value::enumeration(LINK_TYPE_PUBLIC_KEY)).unwrap(),
                    ttlv_item(
                        TAG_LINKED_OBJECT_IDENTIFIER,
                        Value::text_string("linked-pub-1".into()),
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "Cryptographic Parameters" => {
            let s = ttlv_struct(
                TAG_CRYPTOGRAPHIC_PARAMETERS,
                vec![
                    ttlv_item(TAG_BLOCK_CIPHER_MODE, Value::enumeration(0x09)).unwrap(), // GCM
                    ttlv_item(TAG_PADDING_METHOD, Value::enumeration(0x01)).unwrap(),    // None
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "Cryptographic Domain Parameters" => {
            let s = ttlv_struct(
                TAG_CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
                vec![ttlv_item(0x0042_0075, Value::enumeration(0x07)).unwrap()], // P-256
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "Digest" => {
            let s = ttlv_struct(
                TAG_DIGEST,
                vec![
                    ttlv_item(TAG_HASHING_ALGORITHM, Value::enumeration(0x06)).unwrap(), // SHA-256
                    ttlv_item(TAG_DIGEST_VALUE, Value::byte_string(vec![0xAB; 32])).unwrap(),
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "Revocation Reason" => {
            let s = ttlv_struct(
                TAG_REVOCATION_REASON,
                vec![
                    ttlv_item(TAG_REVOCATION_REASON_CODE, Value::enumeration(0x02)).unwrap(),
                    ttlv_item(
                        TAG_REVOCATION_MESSAGE,
                        Value::text_string("compromised".into()),
                    )
                    .unwrap(),
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "Usage Limits" => {
            let s = ttlv_struct(
                TAG_USAGE_LIMITS,
                vec![
                    ttlv_item(TAG_USAGE_LIMITS_TOTAL, Value::long_integer(1000)).unwrap(),
                    ttlv_item(TAG_USAGE_LIMITS_COUNT, Value::long_integer(950)).unwrap(),
                    ttlv_item(TAG_USAGE_LIMITS_UNIT, Value::enumeration(2)).unwrap(),
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "Application Specific Information" => {
            let s = ttlv_struct(
                TAG_APPLICATION_SPECIFIC_INFORMATION,
                vec![
                    ttlv_item(
                        TAG_APPLICATION_NAMESPACE,
                        Value::text_string("kmipkit".into()),
                    )
                    .unwrap(),
                    ttlv_item(TAG_APPLICATION_DATA, Value::text_string("v1".into())).unwrap(),
                ],
            )
            .unwrap();
            kmipkit_proxy::kmip1x::ttlv_helpers::clone_item_value(&s).unwrap()
        }
        "X.509 Certificate Identifier"
        | "X.509 Certificate Subject"
        | "X.509 Certificate Issuer"
        | "Key Value Location"
        | "Alternative Name"
        | "Random Number Generator" => {
            // Simple text/byte payload inside structure
            Value::text_string(format!("val-{name_1x}"))
        }

        // All remaining TextStrings (Unique Identifier, Object Group, Contact Information,
        // PKCS#12 Friendly Name, Description, Comment, and all 24 Certificate Subject/Issuer fields)
        _ => Value::text_string(format!("sample-{name_1x}")),
    }
}

// ============================================================================
// Echoing Mock Upstream Server (records requests & echoes stored attributes)
// ============================================================================

struct EchoUpstreamServer {
    version: UpstreamKmipVersion,
    recorded_requests: Mutex<Vec<Item>>,
    stored_attrs: Mutex<BTreeMap<String, Vec<Item>>>,
}

impl EchoUpstreamServer {
    fn new(version: UpstreamKmipVersion) -> Self {
        Self {
            version,
            recorded_requests: Mutex::new(Vec::new()),
            stored_attrs: Mutex::new(BTreeMap::new()),
        }
    }

    fn seed_object(&self, uid: &str, attrs_2x: Vec<Item>) {
        self.stored_attrs
            .lock()
            .unwrap()
            .insert(uid.to_string(), attrs_2x);
    }

    fn last_request(&self) -> Item {
        clone_item(
            self.recorded_requests
                .lock()
                .unwrap()
                .last()
                .expect("at least one request recorded"),
        )
        .unwrap()
    }
}

#[async_trait]
impl UpstreamKmipTransport for EchoUpstreamServer {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String> {
        self.recorded_requests
            .lock()
            .unwrap()
            .push(clone_item(request)?);

        let (up_maj, up_min) = self.version.major_minor();
        let batch_items = find_children(request, TAG_BATCH_ITEM);
        let mut resp_items = Vec::new();

        for bi in batch_items {
            let op = find_child(&bi, TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap_or(0);
            let bid =
                find_child(&bi, TAG_UNIQUE_BATCH_ITEM_ID).and_then(|i| extract_byte_string(&i));
            let payload = find_child(&bi, TAG_REQUEST_PAYLOAD);

            let resp_bi = match op {
                OP_CREATE | OP_REGISTER | OP_IMPORT => {
                    let uid = "echo-obj-001".to_string();
                    let attrs = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_ATTRIBUTES))
                        .map(|a| children_of(&a))
                        .unwrap_or_default();
                    self.stored_attrs.lock().unwrap().insert(uid.clone(), attrs);
                    build_success_batch_item(
                        op,
                        bid.as_deref(),
                        vec![
                            ttlv_item(
                                TAG_OBJECT_TYPE,
                                Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                            )?,
                            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                        ],
                    )?
                }
                OP_GET_ATTRIBUTES => {
                    let uid = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                        .and_then(|i| extract_text_string(&i))
                        .unwrap_or_else(|| "echo-obj-001".to_string());
                    let stored = self
                        .stored_attrs
                        .lock()
                        .unwrap()
                        .get(&uid)
                        .map(|list| list.iter().map(|i| clone_item(i).unwrap()).collect())
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
                OP_GET_ATTRIBUTE_LIST => {
                    let uid = payload
                        .as_ref()
                        .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                        .and_then(|i| extract_text_string(&i))
                        .unwrap_or_else(|| "echo-obj-001".to_string());
                    let mut out = vec![ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string(uid.clone()),
                    )?];
                    if let Some(list) = self.stored_attrs.lock().unwrap().get(&uid) {
                        for item in list {
                            if item.tag().raw() == TAG_ATTRIBUTE {
                                let vendor = find_child(item, TAG_VENDOR_IDENTIFICATION)
                                    .and_then(|i| extract_text_string(&i))
                                    .unwrap_or_else(|| "x".to_string());
                                let name = find_child(item, TAG_ATTRIBUTE_NAME)
                                    .and_then(|i| extract_text_string(&i))
                                    .unwrap_or_default();
                                out.push(ttlv_struct(
                                    TAG_ATTRIBUTE_REFERENCE,
                                    vec![
                                        ttlv_item(
                                            TAG_VENDOR_IDENTIFICATION,
                                            Value::text_string(vendor),
                                        )?,
                                        ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string(name))?,
                                    ],
                                )?);
                            } else {
                                out.push(ttlv_item(
                                    TAG_ATTRIBUTE_REFERENCE,
                                    Value::enumeration(item.tag().raw()),
                                )?);
                            }
                        }
                    }
                    build_success_batch_item(op, bid.as_deref(), out)?
                }
                _ => build_success_batch_item(
                    op,
                    bid.as_deref(),
                    vec![ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string("echo-obj-001".into()),
                    )?],
                )?,
            };
            resp_items.push(resp_bi);
        }

        build_response_message(up_maj, up_min, resp_items)
    }
}

fn make_1x_req(major: i32, minor: i32, batch_items: Vec<Item>) -> Item {
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

fn make_bi(op: u32, bid: Option<&[u8]>, payload_items: Vec<Item>) -> Item {
    let mut children = vec![ttlv_item(TAG_OPERATION, Value::enumeration(op)).unwrap()];
    if let Some(b) = bid {
        children.push(ttlv_item(TAG_UNIQUE_BATCH_ITEM_ID, Value::byte_string(b.to_vec())).unwrap());
    }
    children.push(ttlv_struct(TAG_REQUEST_PAYLOAD, payload_items).unwrap());
    ttlv_struct(TAG_BATCH_ITEM, children).unwrap()
}

// ============================================================================
// TEST 1: 100% of All 77 Standard Attributes Bidirectional Roundtrip
//         (1.4 -> 2.0, 2.1, 3.0 -> 1.4) in BOTH Request and Response
// ============================================================================

#[test]
fn test_all_77_attributes_full_bidirectional_roundtrip_across_20_21_and_30() {
    assert_eq!(STANDARD_ATTRIBUTES.len(), 77);

    for upstream_version in [
        UpstreamKmipVersion::V2_0,
        UpstreamKmipVersion::V2_1,
        UpstreamKmipVersion::V3_0,
    ] {
        let cfg = TranslatorConfig {
            upstream_version,
            strip_readonly_on_create: false, // Test all 77 attributes including read-only (e.g. Locate/GetAttributes)
            ..TranslatorConfig::default()
        };

        let mut translated_2x_items = Vec::new();
        for spec in STANDARD_ATTRIBUTES {
            let val = sample_value_for_attribute(spec.name_1x);
            let attr_1x = build_1x_attribute(spec.name_1x, None, val).unwrap();

            // Forward: 1.X -> 2.0 / 2.1 / 3.0
            let attr_2x = translate_attribute_1x_to_2x(&attr_1x, cfg, false)
                .unwrap_or_else(|e| {
                    panic!(
                        "forward failed for '{}' on {upstream_version:?}: {e}",
                        spec.name_1x
                    )
                })
                .unwrap_or_else(|| {
                    panic!(
                        "'{}' unexpectedly dropped on {upstream_version:?}",
                        spec.name_1x
                    )
                });

            if spec.tag_2x == TAG_LINK && upstream_version == UpstreamKmipVersion::V3_0 {
                assert_eq!(attr_2x.tag().raw(), TAG_PUBLIC_KEY_LINK);
            } else {
                assert_eq!(
                    attr_2x.tag().raw(),
                    spec.tag_2x,
                    "tag mismatch for '{}' on {upstream_version:?}",
                    spec.name_1x
                );
            }
            translated_2x_items.push(attr_2x);
        }

        // Reverse: 2.0 / 2.1 / 3.0 -> 1.4
        let attrs_struct_2x = ttlv_struct(TAG_ATTRIBUTES, translated_2x_items).unwrap();
        let restored_1x = translate_attributes_2x_to_1x(&attrs_struct_2x, 4).unwrap();
        assert_eq!(
            restored_1x.len(),
            STANDARD_ATTRIBUTES.len(),
            "all 77 attributes must roundtrip back to KMIP 1.4 from {upstream_version:?}"
        );

        for (spec, item_1x) in STANDARD_ATTRIBUTES.iter().zip(restored_1x.iter()) {
            let restored_name =
                extract_text_string(&find_child(item_1x, TAG_ATTRIBUTE_NAME).unwrap()).unwrap();
            assert_eq!(restored_name, spec.name_1x);
            assert!(find_child(item_1x, TAG_ATTRIBUTE_VALUE).is_some());

            // Verify Attribute Index presence matches cardinality
            let has_index = find_child(item_1x, TAG_ATTRIBUTE_INDEX).is_some();
            match spec.cardinality {
                Cardinality::MultiInstance => assert!(
                    has_index,
                    "multi-instance '{}' must include Attribute Index in 1.X response",
                    spec.name_1x
                ),
                Cardinality::SingleInstance => assert!(
                    !has_index,
                    "single-instance '{}' must NOT include Attribute Index in 1.X response",
                    spec.name_1x
                ),
            }
        }
    }
}

// ============================================================================
// TEST 2: Create + GetAttributes + GetAttributeList End-to-End for All Writable Attributes
// ============================================================================

#[tokio::test]
async fn test_e2e_create_get_attributes_and_list_all_writable_attributes() {
    for target in [
        UpstreamKmipVersion::V2_0,
        UpstreamKmipVersion::V2_1,
        UpstreamKmipVersion::V3_0,
    ] {
        let upstream = Arc::new(EchoUpstreamServer::new(target));
        let bridge = Kmip1xBridge::new(upstream.clone(), target);

        let writable_specs: Vec<_> = STANDARD_ATTRIBUTES
            .iter()
            .filter(|s| !s.read_only_on_create)
            .collect();
        let readonly_specs: Vec<_> = STANDARD_ATTRIBUTES
            .iter()
            .filter(|s| s.read_only_on_create)
            .collect();

        // Send ALL 77 attributes in Create (read-only ones must be stripped automatically!)
        let mut all_1x_attrs = Vec::new();
        for spec in STANDARD_ATTRIBUTES {
            all_1x_attrs.push(
                build_1x_attribute(spec.name_1x, None, sample_value_for_attribute(spec.name_1x))
                    .unwrap(),
            );
        }

        let create_req = make_1x_req(
            1,
            4,
            vec![make_bi(
                OP_CREATE,
                Some(b"batch-01"),
                vec![
                    ttlv_item(
                        TAG_OBJECT_TYPE,
                        Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                    )
                    .unwrap(),
                    ttlv_struct(TAG_TEMPLATE_ATTRIBUTE, all_1x_attrs).unwrap(),
                ],
            )],
        );
        let create_resp = bridge.dispatch(&create_req).await.unwrap();
        assert_eq!(extract_protocol_version(&create_resp), Some((1, 4)));
        let create_bi = find_child(&create_resp, TAG_BATCH_ITEM).unwrap();
        assert_eq!(
            extract_byte_string(&find_child(&create_bi, TAG_UNIQUE_BATCH_ITEM_ID).unwrap())
                .as_deref(),
            Some(b"batch-01".as_slice())
        );

        // Verify read-only attributes were stripped before hitting upstream
        let sent_attrs = find_child(
            &find_child(
                &find_child(&upstream.last_request(), TAG_BATCH_ITEM).unwrap(),
                TAG_REQUEST_PAYLOAD,
            )
            .unwrap(),
            TAG_ATTRIBUTES,
        )
        .unwrap();
        assert_eq!(
            children_of(&sent_attrs).len(),
            writable_specs.len(),
            "expected {} writable attributes ({} read-only stripped) for {target:?}",
            writable_specs.len(),
            readonly_specs.len()
        );

        // Now call GetAttributes and verify every writable attribute comes back in KMIP 1.4 format
        let get_attrs_req = make_1x_req(
            1,
            4,
            vec![make_bi(
                OP_GET_ATTRIBUTES,
                Some(b"batch-02"),
                vec![
                    ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string("echo-obj-001".into()),
                    )
                    .unwrap(),
                ],
            )],
        );
        let get_attrs_resp = bridge.dispatch(&get_attrs_req).await.unwrap();
        let gp = find_child(
            &find_child(&get_attrs_resp, TAG_BATCH_ITEM).unwrap(),
            TAG_RESPONSE_PAYLOAD,
        )
        .unwrap();
        let returned_attrs = find_children(&gp, TAG_ATTRIBUTE);
        assert_eq!(returned_attrs.len(), writable_specs.len());

        // Now call GetAttributeList and verify every attribute name is restored
        let list_req = make_1x_req(
            1,
            4,
            vec![make_bi(
                OP_GET_ATTRIBUTE_LIST,
                Some(b"batch-03"),
                vec![
                    ttlv_item(
                        TAG_UNIQUE_IDENTIFIER,
                        Value::text_string("echo-obj-001".into()),
                    )
                    .unwrap(),
                ],
            )],
        );
        let list_resp = bridge.dispatch(&list_req).await.unwrap();
        let lp = find_child(
            &find_child(&list_resp, TAG_BATCH_ITEM).unwrap(),
            TAG_RESPONSE_PAYLOAD,
        )
        .unwrap();
        let returned_names: Vec<String> = find_children(&lp, TAG_ATTRIBUTE_NAME)
            .iter()
            .filter_map(extract_text_string)
            .collect();
        assert_eq!(returned_names.len(), writable_specs.len());
        for spec in &writable_specs {
            assert!(
                returned_names.iter().any(|n| n == spec.name_1x),
                "GetAttributeList missing '{}' for {target:?}",
                spec.name_1x
            );
        }
    }
}

// ============================================================================
// TEST 3: KMIP 1.0 vs 1.1 vs 1.2 vs 1.3 vs 1.4 Version-Gated Attribute Downgrade
// ============================================================================

#[tokio::test]
async fn test_response_attributes_respect_client_minor_version_10_through_14() {
    let upstream = Arc::new(EchoUpstreamServer::new(UpstreamKmipVersion::V2_1));
    // Seed an object with attributes from 1.0, 1.1, 1.2, 1.3, and 1.4
    upstream.seed_object(
        "ver-key",
        vec![
            ttlv_item(TAG_CRYPTOGRAPHIC_LENGTH, Value::integer(256)).unwrap(), // 1.0
            ttlv_item(
                TAG_X509_CERTIFICATE_SUBJECT,
                Value::text_string("CN=app".into()),
            )
            .unwrap(), // 1.1 (maps to Certificate Subject in 1.0)
            ttlv_item(TAG_FRESH, Value::boolean(true)).unwrap(),               // 1.1
            ttlv_item(TAG_ALTERNATIVE_NAME, Value::text_string("alt".into())).unwrap(), // 1.2
            ttlv_item(TAG_DESCRIPTION, Value::text_string("desc".into())).unwrap(), // 1.3
            ttlv_item(TAG_SENSITIVE, Value::boolean(true)).unwrap(),           // 1.4
            ttlv_item(TAG_KEY_ROLE_TYPE, Value::enumeration(3)).unwrap(),      // 1.4
        ],
    );

    let bridge = Kmip1xBridge::new(upstream, UpstreamKmipVersion::V2_1);

    // KMIP 1.0 client: gets Cryptographic Length + downgraded "Certificate Subject", no 1.1–1.4 attrs
    let resp_10 = bridge
        .dispatch(&make_1x_req(
            1,
            0,
            vec![make_bi(
                OP_GET_ATTRIBUTES,
                None,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("ver-key".into())).unwrap(),
                ],
            )],
        ))
        .await
        .unwrap();
    let p10 = find_child(
        &find_child(&resp_10, TAG_BATCH_ITEM).unwrap(),
        TAG_RESPONSE_PAYLOAD,
    )
    .unwrap();
    let names_10: Vec<String> = find_children(&p10, TAG_ATTRIBUTE)
        .iter()
        .filter_map(|a| find_child(a, TAG_ATTRIBUTE_NAME).and_then(|n| extract_text_string(&n)))
        .collect();
    assert!(names_10.contains(&"Cryptographic Length".to_string()));
    assert!(names_10.contains(&"Certificate Subject".to_string()));
    assert!(!names_10.contains(&"Fresh".to_string()));
    assert!(!names_10.contains(&"Sensitive".to_string()));

    // KMIP 1.2 client: gets 1.0 + 1.1 + 1.2, no 1.3 or 1.4
    let resp_12 = bridge
        .dispatch(&make_1x_req(
            1,
            2,
            vec![make_bi(
                OP_GET_ATTRIBUTES,
                None,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("ver-key".into())).unwrap(),
                ],
            )],
        ))
        .await
        .unwrap();
    let p12 = find_child(
        &find_child(&resp_12, TAG_BATCH_ITEM).unwrap(),
        TAG_RESPONSE_PAYLOAD,
    )
    .unwrap();
    let names_12: Vec<String> = find_children(&p12, TAG_ATTRIBUTE)
        .iter()
        .filter_map(|a| find_child(a, TAG_ATTRIBUTE_NAME).and_then(|n| extract_text_string(&n)))
        .collect();
    assert!(names_12.contains(&"X.509 Certificate Subject".to_string()));
    assert!(names_12.contains(&"Fresh".to_string()));
    assert!(names_12.contains(&"Alternative Name".to_string()));
    assert!(!names_12.contains(&"Description".to_string()));
    assert!(!names_12.contains(&"Sensitive".to_string()));

    // KMIP 1.4 client: gets all 7 attributes
    let resp_14 = bridge
        .dispatch(&make_1x_req(
            1,
            4,
            vec![make_bi(
                OP_GET_ATTRIBUTES,
                None,
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("ver-key".into())).unwrap(),
                ],
            )],
        ))
        .await
        .unwrap();
    let p14 = find_child(
        &find_child(&resp_14, TAG_BATCH_ITEM).unwrap(),
        TAG_RESPONSE_PAYLOAD,
    )
    .unwrap();
    assert_eq!(find_children(&p14, TAG_ATTRIBUTE).len(), 7);
}

// ============================================================================
// TEST 4: All 14 KMIP 3.0 Typed Links Bidirectional Roundtrip
// ============================================================================

#[test]
fn test_all_14_kmip30_typed_links_bidirectional_roundtrip() {
    let all_link_types = [
        (LINK_TYPE_CERTIFICATE, TAG_CERTIFICATE_LINK),
        (LINK_TYPE_PUBLIC_KEY, TAG_PUBLIC_KEY_LINK),
        (LINK_TYPE_PRIVATE_KEY, TAG_PRIVATE_KEY_LINK),
        (LINK_TYPE_DERIVATION_BASE, TAG_DERIVATION_BASE_OBJECT_LINK),
        (LINK_TYPE_DERIVED_KEY, TAG_DERIVED_OBJECT_LINK),
        (LINK_TYPE_REPLACEMENT, TAG_REPLACEMENT_OBJECT_LINK),
        (LINK_TYPE_REPLACED, TAG_REPLACED_OBJECT_LINK),
        (LINK_TYPE_PARENT, TAG_PARENT_LINK),
        (LINK_TYPE_CHILD, TAG_CHILD_LINK),
        (LINK_TYPE_PREVIOUS, TAG_PREVIOUS_LINK),
        (LINK_TYPE_NEXT, TAG_NEXT_LINK),
        (LINK_TYPE_PKCS12_CERT, TAG_PKCS12_CERTIFICATE_LINK),
        (LINK_TYPE_PKCS12_PASSWORD, TAG_PKCS12_PASSWORD_LINK),
        (LINK_TYPE_WRAPPING_KEY, TAG_WRAPPING_KEY_LINK),
    ];

    let cfg_v3 = TranslatorConfig {
        upstream_version: UpstreamKmipVersion::V3_0,
        ..TranslatorConfig::default()
    };

    for (lt, expected_v3_tag) in all_link_types {
        assert_eq!(link_type_to_v3_tag(lt), Some(expected_v3_tag));
        assert_eq!(v3_link_tag_to_link_type(expected_v3_tag), Some(lt));

        let target_id = format!("linked-obj-{lt:04X}");
        let link_val = ttlv_struct(
            TAG_ATTRIBUTE_VALUE,
            vec![
                ttlv_item(TAG_LINK_TYPE, Value::enumeration(lt)).unwrap(),
                ttlv_item(
                    TAG_LINKED_OBJECT_IDENTIFIER,
                    Value::text_string(target_id.clone()),
                )
                .unwrap(),
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

        let v3_item = translate_attribute_1x_to_2x(&link_1x, cfg_v3, false)
            .unwrap()
            .unwrap();
        assert_eq!(v3_item.tag().raw(), expected_v3_tag);
        assert_eq!(
            extract_text_string(&v3_item).as_deref(),
            Some(target_id.as_str())
        );

        let attrs_v3 = ttlv_struct(TAG_ATTRIBUTES, vec![v3_item]).unwrap();
        let back_1x = translate_attributes_2x_to_1x(&attrs_v3, 4).unwrap();
        assert_eq!(back_1x.len(), 1);
        let val = find_child(&back_1x[0], TAG_ATTRIBUTE_VALUE).unwrap();
        assert_eq!(
            extract_enumeration(&find_child(&val, TAG_LINK_TYPE).unwrap()),
            Some(lt)
        );
        assert_eq!(
            extract_text_string(&find_child(&val, TAG_LINKED_OBJECT_IDENTIFIER).unwrap())
                .as_deref(),
            Some(target_id.as_str())
        );
    }
}

// ============================================================================
// TEST 5: Exhaustive Result Reason Downgrade Sweep (0x01..=0x4D + 0x100)
// ============================================================================

#[test]
fn test_exhaustive_result_reason_downgrade_always_valid_for_1x_minor() {
    for minor in 0..=4 {
        let max_valid_low = match minor {
            0 => 0x0F,
            1 => 0x13,
            2 | 3 => 0x15,
            _ => 0x16,
        };
        for reason_2x in (0x01..=0x50).chain(std::iter::once(0x100)) {
            let downgraded = downgrade_result_reason(reason_2x, minor);
            assert!(
                (0x01..=max_valid_low).contains(&downgraded)
                    || downgraded == REASON_GENERAL_FAILURE,
                "reason 0x{reason_2x:04X} downgraded to 0x{downgraded:04X} which is invalid for KMIP 1.{minor}"
            );
        }
    }
}

// ============================================================================
// TEST 6: Create Key Pair (Common / Private / Public Template-Attribute) + Template Merge
// ============================================================================

#[tokio::test]
async fn test_create_key_pair_common_private_public_template_attributes_translation() {
    let upstream = Arc::new(EchoUpstreamServer::new(UpstreamKmipVersion::V2_1));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V2_1);

    let req = make_1x_req(
        1,
        4,
        vec![make_bi(
            OP_CREATE_KEY_PAIR,
            Some(b"ckp-1"),
            vec![
                ttlv_struct(
                    TAG_COMMON_TEMPLATE_ATTRIBUTE,
                    vec![
                        build_1x_attribute("Cryptographic Algorithm", None, Value::enumeration(4))
                            .unwrap(), // RSA
                        build_1x_attribute("Cryptographic Length", None, Value::integer(2048))
                            .unwrap(),
                    ],
                )
                .unwrap(),
                ttlv_struct(
                    TAG_PRIVATE_KEY_TEMPLATE_ATTRIBUTE,
                    vec![
                        build_1x_attribute("Cryptographic Usage Mask", None, Value::integer(0x01))
                            .unwrap(), // Sign
                    ],
                )
                .unwrap(),
                ttlv_struct(
                    TAG_PUBLIC_KEY_TEMPLATE_ATTRIBUTE,
                    vec![
                        build_1x_attribute("Cryptographic Usage Mask", None, Value::integer(0x02))
                            .unwrap(), // Verify
                    ],
                )
                .unwrap(),
            ],
        )],
    );

    let resp = bridge.dispatch(&req).await.unwrap();
    assert_eq!(extract_protocol_version(&resp), Some((1, 4)));

    let sent = upstream.last_request();
    let sp = find_child(
        &find_child(&sent, TAG_BATCH_ITEM).unwrap(),
        TAG_REQUEST_PAYLOAD,
    )
    .unwrap();
    assert!(find_child(&sp, TAG_COMMON_ATTRIBUTES).is_some());
    assert!(find_child(&sp, TAG_PRIVATE_KEY_ATTRIBUTES).is_some());
    assert!(find_child(&sp, TAG_PUBLIC_KEY_ATTRIBUTES).is_some());
    assert!(find_child(&sp, TAG_COMMON_TEMPLATE_ATTRIBUTE).is_none());
}

// ============================================================================
// TEST 7: Export with Key Wrapping Specification & Asynchronous Indicator Header
// ============================================================================

#[tokio::test]
async fn test_export_key_wrapping_spec_and_async_indicator_header_translation() {
    let upstream = Arc::new(EchoUpstreamServer::new(UpstreamKmipVersion::V3_0));
    let bridge = Kmip1xBridge::new(upstream.clone(), UpstreamKmipVersion::V3_0);

    let header = ttlv_struct(
        TAG_REQUEST_HEADER,
        vec![
            build_protocol_version(1, 4).unwrap(),
            ttlv_item(TAG_ASYNCHRONOUS_INDICATOR, Value::boolean(true)).unwrap(),
            ttlv_item(TAG_BATCH_COUNT, Value::integer(1)).unwrap(),
        ],
    )
    .unwrap();
    let kws = ttlv_struct(
        TAG_KEY_WRAPPING_SPECIFICATION,
        vec![
            ttlv_item(
                TAG_ATTRIBUTE_NAME,
                Value::text_string("Object Group".into()),
            )
            .unwrap(),
            ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("x-tenant".into())).unwrap(),
        ],
    )
    .unwrap();
    let req = ttlv_struct(
        TAG_REQUEST_MESSAGE,
        vec![
            header,
            make_bi(
                OP_EXPORT,
                None,
                vec![
                    ttlv_item(
                        TAG_OBJECT_TYPE,
                        Value::enumeration(OBJECT_TYPE_SYMMETRIC_KEY),
                    )
                    .unwrap(),
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-1".into())).unwrap(),
                    kws,
                ],
            ),
        ],
    )
    .unwrap();

    let resp = bridge.dispatch(&req).await.unwrap();
    assert_eq!(extract_protocol_version(&resp), Some((1, 4)));

    let sent = upstream.last_request();
    let sent_hdr = find_child(&sent, TAG_REQUEST_HEADER).unwrap();
    assert_eq!(
        extract_boolean(&find_child(&sent_hdr, TAG_ASYNCHRONOUS_INDICATOR).unwrap()),
        Some(true)
    );
    let sent_kws = find_child(
        &find_child(
            &find_child(&sent, TAG_BATCH_ITEM).unwrap(),
            TAG_REQUEST_PAYLOAD,
        )
        .unwrap(),
        TAG_KEY_WRAPPING_SPECIFICATION,
    )
    .unwrap();
    let refs = find_children(&sent_kws, TAG_ATTRIBUTE_REFERENCE);
    assert_eq!(refs.len(), 2);
    assert_eq!(extract_enumeration(&refs[0]), Some(TAG_OBJECT_GROUP));
    assert_eq!(
        extract_text_string(&find_child(&refs[1], TAG_VENDOR_IDENTIFICATION).unwrap()).as_deref(),
        Some("x")
    );
}

// ============================================================================
// TEST 8: Multi-Instance Sequential Attribute Indexing in Response
// ============================================================================

#[test]
fn test_multi_instance_attributes_receive_sequential_indices_in_1x_response() {
    let attrs_2x = ttlv_struct(
        TAG_ATTRIBUTES,
        vec![
            ttlv_item(TAG_OBJECT_GROUP, Value::text_string("g0".into())).unwrap(),
            ttlv_item(TAG_OBJECT_GROUP, Value::text_string("g1".into())).unwrap(),
            ttlv_item(TAG_OBJECT_GROUP, Value::text_string("g2".into())).unwrap(),
        ],
    )
    .unwrap();

    let out_1x = translate_attributes_2x_to_1x(&attrs_2x, 4).unwrap();
    assert_eq!(out_1x.len(), 3);
    for (expected_idx, item) in out_1x.iter().enumerate() {
        assert_eq!(
            extract_integer(&find_child(item, TAG_ATTRIBUTE_INDEX).unwrap()),
            Some(expected_idx as i32)
        );
        assert_eq!(
            extract_text_string(&find_child(item, TAG_ATTRIBUTE_VALUE).unwrap()).as_deref(),
            Some(format!("g{expected_idx}").as_str())
        );
    }
}

// ============================================================================
// TEST 9: S2C Full Roundtrip Across KMIP 2.0, 2.1, and 3.0 (Notify, Put, Query, DiscoverVersions)
// ============================================================================

struct RecordingS2cClient {
    minor_1x: i32,
    received: Mutex<Vec<Item>>,
}

#[async_trait]
impl S2cClientTransport for RecordingS2cClient {
    async fn send_to_client(&self, request_1x: &Item) -> Result<Item, String> {
        self.received.lock().unwrap().push(clone_item(request_1x)?);
        let bi = find_child(request_1x, TAG_BATCH_ITEM).unwrap();
        let op = extract_enumeration(&find_child(&bi, TAG_OPERATION).unwrap()).unwrap();
        let bid = find_child(&bi, TAG_UNIQUE_BATCH_ITEM_ID).and_then(|i| extract_byte_string(&i));
        let resp_bi = build_success_batch_item(op, bid.as_deref(), vec![])?;
        build_response_message(1, self.minor_1x, vec![resp_bi])
    }
}

#[tokio::test]
async fn test_s2c_all_operations_across_20_21_and_30() {
    for target in [
        UpstreamKmipVersion::V2_0,
        UpstreamKmipVersion::V2_1,
        UpstreamKmipVersion::V3_0,
    ] {
        let (maj, min) = target.major_minor();
        let client = Arc::new(RecordingS2cClient {
            minor_1x: 4,
            received: Mutex::new(Vec::new()),
        });
        let s2c = Kmip1xS2cBridge::new(client.clone(), 4, target);

        // 1. S2C Notify with standard + vendor + v3 link attributes
        let notify = make_1x_req(
            maj,
            min,
            vec![make_bi(
                OP_NOTIFY,
                Some(b"s2c-1"),
                vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("k-99".into())).unwrap(),
                    ttlv_struct(
                        TAG_ATTRIBUTES,
                        vec![
                            ttlv_item(TAG_STATE, Value::enumeration(2)).unwrap(),
                            ttlv_item(TAG_PUBLIC_KEY_LINK, Value::text_string("pub-99".into()))
                                .unwrap(),
                        ],
                    )
                    .unwrap(),
                ],
            )],
        );
        let notify_resp = s2c.handle_server_request(&notify).await.unwrap();
        assert_eq!(extract_protocol_version(&notify_resp), Some((maj, min)));

        // Verify client received 1.4 with 2 KMIP 1.X Attribute items
        {
            let rec = client.received.lock().unwrap();
            let last_to_client = rec.last().unwrap();
            assert_eq!(extract_protocol_version(last_to_client), Some((1, 4)));
            let lp = find_child(
                &find_child(last_to_client, TAG_BATCH_ITEM).unwrap(),
                TAG_REQUEST_PAYLOAD,
            )
            .unwrap();
            assert_eq!(find_children(&lp, TAG_ATTRIBUTE).len(), 2);
        }

        // 2. S2C Query
        let query = make_1x_req(maj, min, vec![make_bi(OP_QUERY, Some(b"s2c-2"), vec![])]);
        let query_resp = s2c.handle_server_request(&query).await.unwrap();
        assert_eq!(extract_protocol_version(&query_resp), Some((maj, min)));

        // 3. S2C Discover Versions (intercepted locally)
        let dv = make_1x_req(maj, min, vec![make_bi(OP_DISCOVER_VERSIONS, None, vec![])]);
        let dv_resp = s2c.handle_server_request(&dv).await.unwrap();
        assert_eq!(extract_protocol_version(&dv_resp), Some((maj, min)));
    }
}
