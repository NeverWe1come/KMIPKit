//! Complete bidirectional TTLV translator between KMIP 1.0–1.4 and KMIP 2.0 / 2.1 / 3.0.
//!
//! Handles all 39 KMIP 1.X operations, all 48+ standard attributes, custom `x-*`/`y-*` vendor
//! attributes, KMIP 3.0 typed links and read-only sanitization, implicit `ID Placeholder`
//! injection, `Discover Versions` local negotiation, `Query` allowlist filtering, `Result Reason`
//! downgrading, and Server-to-Client (`Notify`/`Put`) reverse translation.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use kmipkit_ttlv::{Item, Value, ValueView};

use crate::kmip1x::mapping::{
    Cardinality, OperationPolicyHandling, UpstreamKmipVersion, downgrade_result_reason,
    is_crypto_algorithm_allowed_in_1x, is_custom_attribute_1x, is_object_type_allowed_in_1x,
    is_operation_allowed_in_1x, is_query_function_allowed_in_1x, link_type_to_v3_tag,
    lookup_by_name_1x, lookup_by_tag_2x, operation_supports_id_placeholder,
    v3_link_tag_to_link_type,
};
use crate::kmip1x::tags::*;
use crate::kmip1x::template_store::TemplateStore;
use crate::kmip1x::ttlv_helpers::{
    build_protocol_version, build_response_message, build_success_batch_item, children_of,
    clone_item_value, extract_boolean, extract_enumeration, extract_integer, extract_text_string,
    find_child, find_children, retag_item, ttlv_item, ttlv_struct,
};

pub const PROXY_VENDOR_ID: &str = "kmip1x-proxy";

/// Configuration options controlling KMIP 1.X <-> 2.0/2.1/3.0 translation semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TranslatorConfig {
    pub upstream_version: UpstreamKmipVersion,
    pub operation_policy_handling: OperationPolicyHandling,
    /// Strip server-managed read-only attributes on Create/Register/Import/etc.
    pub strip_readonly_on_create: bool,
    /// Enforce KMIP 1.X `Maximum Response Size` on the final 1.X serialized TTLV.
    pub enforce_max_response_size: bool,
}

impl Default for TranslatorConfig {
    fn default() -> Self {
        Self {
            upstream_version: UpstreamKmipVersion::V2_1,
            operation_policy_handling: OperationPolicyHandling::Ignore,
            strip_readonly_on_create: true,
            enforce_max_response_size: true,
        }
    }
}

/// Metadata recorded per batch item during request translation so the response can be
/// faithfully restored to the exact KMIP 1.X operation expected by the legacy client.
#[derive(Debug, Clone)]
pub struct TranslatedBatchMeta {
    /// Original KMIP 1.X operation requested by the client.
    pub original_op_1x: u32,
    /// Actual KMIP 2.0+ operation sent upstream (e.g. `OP_SET_ATTRIBUTE` for single-instance Add/Modify).
    pub upstream_op_2x: u32,
}

/// Extracts `(major, minor)` from a KMIP RequestMessage or ResponseMessage.
#[must_use]
pub fn extract_protocol_version(msg: &Item) -> Option<(i32, i32)> {
    let header =
        find_child(msg, TAG_REQUEST_HEADER).or_else(|| find_child(msg, TAG_RESPONSE_HEADER))?;
    let pv = find_child(&header, TAG_PROTOCOL_VERSION)?;
    let major = find_child(&pv, TAG_PROTOCOL_VERSION_MAJOR).and_then(|i| extract_integer(&i))?;
    let minor = find_child(&pv, TAG_PROTOCOL_VERSION_MINOR).and_then(|i| extract_integer(&i))?;
    Some((major, minor))
}

/// Extracts `Maximum Response Size` from a KMIP Request Header, if present and positive.
#[must_use]
pub fn extract_max_response_size(req: &Item) -> Option<usize> {
    let header = find_child(req, TAG_REQUEST_HEADER)?;
    let mrs = find_child(&header, TAG_MAXIMUM_RESPONSE_SIZE)?;
    extract_integer(&mrs)
        .filter(|&v| v > 0)
        .and_then(|v| usize::try_from(v).ok())
}

/// Rewrites `Protocol Version` inside a `Request Header` or `Response Header`.
pub fn rewrite_header_version(header: &Item, major: i32, minor: i32) -> Result<Item, String> {
    let mut new_children = Vec::new();
    for child in children_of(header) {
        if child.tag().raw() == TAG_PROTOCOL_VERSION {
            new_children.push(build_protocol_version(major, minor)?);
        } else if child.tag().raw() == TAG_ASYNCHRONOUS_INDICATOR {
            // In KMIP 1.X Asynchronous Indicator is Boolean; in KMIP 2.0+ it can be Boolean or Enumeration.
            // Preserve Boolean when going 1.X -> 2.0+ or convert Enumeration back to Boolean for 1.X.
            if major == 1 {
                if let Some(en) = extract_enumeration(&child) {
                    // 1 = Mandatory, 2 = Optional, 3 = Prohibited
                    new_children.push(ttlv_item(
                        TAG_ASYNCHRONOUS_INDICATOR,
                        Value::boolean(en != 3),
                    )?);
                } else {
                    new_children.push(child);
                }
            } else {
                new_children.push(child);
            }
        } else {
            new_children.push(child);
        }
    }
    ttlv_struct(header.tag().raw(), new_children)
}

/// Translates a single KMIP 1.X `Attribute { Attribute Name, Attribute Index?, Attribute Value }`
/// into its KMIP 2.0 / 2.1 / 3.0 direct attribute `Item`.
///
/// Returns `Ok(None)` if the attribute should be silently omitted (e.g., `Operation Policy Name`
/// under `Ignore` mode, or read-only attributes stripped on `Create`).
pub fn translate_attribute_1x_to_2x(
    attr_item: &Item,
    cfg: TranslatorConfig,
    is_create_like: bool,
) -> Result<Option<Item>, String> {
    let name = find_child(attr_item, TAG_ATTRIBUTE_NAME)
        .and_then(|i| extract_text_string(&i))
        .ok_or_else(|| "KMIP 1.X Attribute missing Attribute Name".to_string())?;
    let val_item = find_child(attr_item, TAG_ATTRIBUTE_VALUE)
        .ok_or_else(|| format!("KMIP 1.X Attribute '{name}' missing Attribute Value"))?;

    // Handle Operation Policy Name (removed in KMIP 2.0)
    if name == "Operation Policy Name" {
        return match cfg.operation_policy_handling {
            OperationPolicyHandling::Ignore => Ok(None),
            OperationPolicyHandling::StoreAsVendorAttribute => {
                let vendor_attr = ttlv_struct(
                    TAG_ATTRIBUTE,
                    vec![
                        ttlv_item(
                            TAG_VENDOR_IDENTIFICATION,
                            Value::text_string(PROXY_VENDOR_ID.to_string()),
                        )?,
                        ttlv_item(
                            TAG_ATTRIBUTE_NAME,
                            Value::text_string("Operation Policy Name".to_string()),
                        )?,
                        val_item,
                    ],
                )?;
                Ok(Some(vendor_attr))
            }
            OperationPolicyHandling::Reject => Err(
                "Operation Policy Name is removed in KMIP 2.0+ and policy is set to Reject"
                    .to_string(),
            ),
        };
    }

    // Custom attributes (`x-*` client-defined or `y-*` server-defined) -> KMIP 2.0+ Vendor Attribute
    if is_custom_attribute_1x(&name) {
        let (vendor_id, attr_name) = name.split_once('-').unwrap_or(("x", &name));
        let vendor_attr = ttlv_struct(
            TAG_ATTRIBUTE,
            vec![
                ttlv_item(
                    TAG_VENDOR_IDENTIFICATION,
                    Value::text_string(vendor_id.to_string()),
                )?,
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string(attr_name.to_string()),
                )?,
                val_item,
            ],
        )?;
        return Ok(Some(vendor_attr));
    }

    let Some(spec) = lookup_by_name_1x(&name) else {
        // Unknown non-standard attribute: preserve as vendor attribute under kmip1x-proxy
        let vendor_attr = ttlv_struct(
            TAG_ATTRIBUTE,
            vec![
                ttlv_item(
                    TAG_VENDOR_IDENTIFICATION,
                    Value::text_string(PROXY_VENDOR_ID.to_string()),
                )?,
                ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string(name))?,
                val_item,
            ],
        )?;
        return Ok(Some(vendor_attr));
    };

    if is_create_like && cfg.strip_readonly_on_create && spec.read_only_on_create {
        return Ok(None);
    }

    // Special case: KMIP 3.0 typed Link attributes (0x420190..=0x42019D)
    if spec.tag_2x == TAG_LINK && cfg.upstream_version == UpstreamKmipVersion::V3_0 {
        if let (Some(lt), Some(linked_id)) = (
            find_child(&val_item, TAG_LINK_TYPE).and_then(|i| extract_enumeration(&i)),
            find_child(&val_item, TAG_LINKED_OBJECT_IDENTIFIER)
                .and_then(|i| extract_text_string(&i)),
        ) {
            if let Some(v3_tag) = link_type_to_v3_tag(lt) {
                return Ok(Some(ttlv_item(v3_tag, Value::text_string(linked_id))?));
            }
        }
    }

    Ok(Some(retag_item(&val_item, spec.tag_2x)?))
}

/// Translates a KMIP 2.0 / 2.1 / 3.0 `Attributes` structure into a Vec of KMIP 1.X `Attribute`
/// structures, assigning sequential `Attribute Index` values for multi-instance attributes.
pub fn translate_attributes_2x_to_1x(
    attrs_struct: &Item,
    client_minor_1x: i32,
) -> Result<Vec<Item>, String> {
    let mut out = Vec::new();
    let mut counts: BTreeMap<String, i32> = BTreeMap::new();

    for child in children_of(attrs_struct) {
        let tag = child.tag().raw();

        // 1. KMIP 2.0+ Vendor Attribute (`0x420008` with Vendor Identification + Attribute Name + Value)
        if tag == TAG_ATTRIBUTE {
            let vendor = find_child(&child, TAG_VENDOR_IDENTIFICATION)
                .and_then(|i| extract_text_string(&i))
                .unwrap_or_else(|| "x".to_string());
            let attr_name =
                find_child(&child, TAG_ATTRIBUTE_NAME).and_then(|i| extract_text_string(&i));
            let val_item = find_child(&child, TAG_ATTRIBUTE_VALUE);

            if let (Some(aname), Some(val)) = (attr_name, val_item) {
                let full_name = if vendor == PROXY_VENDOR_ID {
                    aname
                } else {
                    format!("{vendor}-{aname}")
                };
                let idx = counts.entry(full_name.clone()).or_insert(0);
                let current_idx = *idx;
                *idx += 1;
                out.push(build_1x_attribute(
                    &full_name,
                    Some(current_idx),
                    clone_item_value(&val)?,
                )?);
            }
            continue;
        }

        // 2. KMIP 3.0 typed Link attributes -> KMIP 1.X `Link` { Link Type, Linked Object Identifier }
        if let Some(link_type) = v3_link_tag_to_link_type(tag) {
            if let Some(target_id) = extract_text_string(&child) {
                let idx = counts.entry("Link".to_string()).or_insert(0);
                let current_idx = *idx;
                *idx += 1;
                let link_val = ttlv_struct(
                    TAG_ATTRIBUTE_VALUE,
                    vec![
                        ttlv_item(TAG_LINK_TYPE, Value::enumeration(link_type))?,
                        ttlv_item(TAG_LINKED_OBJECT_IDENTIFIER, Value::text_string(target_id))?,
                    ],
                )?;
                out.push(ttlv_struct(
                    TAG_ATTRIBUTE,
                    vec![
                        ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string("Link".to_string()))?,
                        ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(current_idx))?,
                        link_val,
                    ],
                )?);
            }
            continue;
        }

        // 3. Standard attributes
        if let Some(spec) = lookup_by_tag_2x(tag) {
            // Respect client minor version where strict (allow core 1.0 + anything <= client_minor)
            if spec.min_minor_1x > client_minor_1x && client_minor_1x < 4 {
                // Keep X.509 attrs mapped to 1.0 names if client is 1.0
                let mapped_name = if client_minor_1x == 0 {
                    match spec.name_1x {
                        "X.509 Certificate Identifier" => Some("Certificate Identifier"),
                        "X.509 Certificate Subject" => Some("Certificate Subject"),
                        "X.509 Certificate Issuer" => Some("Certificate Issuer"),
                        _ => None,
                    }
                } else {
                    None
                };
                let Some(compat_name) = mapped_name else {
                    continue;
                };
                out.push(build_1x_attribute(
                    compat_name,
                    None,
                    clone_item_value(&child)?,
                )?);
                continue;
            }

            let idx = counts.entry(spec.name_1x.to_string()).or_insert(0);
            let current_idx = *idx;
            *idx += 1;
            let index_opt = match spec.cardinality {
                Cardinality::MultiInstance => Some(current_idx),
                Cardinality::SingleInstance => None,
            };
            out.push(build_1x_attribute(
                spec.name_1x,
                index_opt,
                clone_item_value(&child)?,
            )?);
        }
    }

    Ok(out)
}

/// Builds a KMIP 1.X `Attribute` structure.
pub fn build_1x_attribute(name: &str, index: Option<i32>, value: Value) -> Result<Item, String> {
    let mut children = vec![ttlv_item(
        TAG_ATTRIBUTE_NAME,
        Value::text_string(name.to_string()),
    )?];
    if let Some(idx) = index {
        children.push(ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(idx))?);
    }
    children.push(ttlv_item(TAG_ATTRIBUTE_VALUE, value)?);
    ttlv_struct(TAG_ATTRIBUTE, children)
}

/// Translates a KMIP 1.X `Attribute Name` string to a KMIP 2.0+ `Attribute Reference` item.
pub fn translate_attribute_name_to_reference(name: &str) -> Result<Option<Item>, String> {
    if is_custom_attribute_1x(name) {
        let (vendor_id, attr_name) = name.split_once('-').unwrap_or(("x", name));
        let ref_struct = ttlv_struct(
            TAG_ATTRIBUTE_REFERENCE,
            vec![
                ttlv_item(
                    TAG_VENDOR_IDENTIFICATION,
                    Value::text_string(vendor_id.to_string()),
                )?,
                ttlv_item(
                    TAG_ATTRIBUTE_NAME,
                    Value::text_string(attr_name.to_string()),
                )?,
            ],
        )?;
        return Ok(Some(ref_struct));
    }

    if let Some(spec) = lookup_by_name_1x(name) {
        return Ok(Some(ttlv_item(
            TAG_ATTRIBUTE_REFERENCE,
            Value::enumeration(spec.tag_2x),
        )?));
    }

    Ok(None)
}

/// Translates a KMIP 1.X `Template-Attribute` (or Common/Private/Public variant) into a
/// KMIP 2.0+ `Attributes` structure, merging any locally stored `Template` referenced by `Name`.
pub fn translate_template_attribute_to_2x(
    tpl_attr: &Item,
    target_tag_2x: u32,
    cfg: TranslatorConfig,
    template_store: &TemplateStore,
) -> Result<Item, String> {
    let mut merged_1x: Vec<Item> = Vec::new();

    // 1. Merge attributes from any locally stored KMIP 1.X Template referenced by Name
    for name_item in find_children(tpl_attr, TAG_NAME) {
        if let Some(nv) = find_child(&name_item, TAG_NAME_VALUE) {
            if let Some(tpl_name) = extract_text_string(&nv) {
                if let Some(stored) = template_store.get_by_name(&tpl_name) {
                    merged_1x.extend(stored.attributes_1x);
                }
            }
        }
    }

    // 2. Append explicit attributes from the request (override single-instance template defaults)
    let explicit_1x = find_children(tpl_attr, TAG_ATTRIBUTE);
    for exp in &explicit_1x {
        if let Some(exp_name) =
            find_child(exp, TAG_ATTRIBUTE_NAME).and_then(|n| extract_text_string(&n))
        {
            let is_single = lookup_by_name_1x(&exp_name)
                .is_some_and(|s| s.cardinality == Cardinality::SingleInstance);
            if is_single {
                merged_1x.retain(|m| {
                    find_child(m, TAG_ATTRIBUTE_NAME)
                        .and_then(|n| extract_text_string(&n))
                        .as_deref()
                        != Some(exp_name.as_str())
                });
            }
        }
    }
    merged_1x.extend(explicit_1x);

    // 3. Translate merged KMIP 1.X attributes into KMIP 2.0+ direct attributes
    let mut attrs_2x = Vec::new();
    for attr in &merged_1x {
        if let Some(translated) = translate_attribute_1x_to_2x(attr, cfg, true)? {
            attrs_2x.push(translated);
        }
    }

    ttlv_struct(target_tag_2x, attrs_2x)
}

/// Builds a local KMIP 1.X `Discover Versions` response intersecting client-offered versions
/// with the proxy's supported KMIP 1.X versions (`1.4, 1.3, 1.2, 1.1, 1.0`).
pub fn build_discover_versions_response_1x(
    client_major: i32,
    client_minor: i32,
    unique_batch_item_id: Option<&[u8]>,
    payload: Option<&Item>,
) -> Result<Item, String> {
    let supported_1x: [(i32, i32); 5] = [(1, 4), (1, 3), (1, 2), (1, 1), (1, 0)];

    let mut offered = Vec::new();
    if let Some(p) = payload {
        for pv in find_children(p, TAG_PROTOCOL_VERSION) {
            if let (Some(maj), Some(min)) = (
                find_child(&pv, TAG_PROTOCOL_VERSION_MAJOR).and_then(|i| extract_integer(&i)),
                find_child(&pv, TAG_PROTOCOL_VERSION_MINOR).and_then(|i| extract_integer(&i)),
            ) {
                offered.push((maj, min));
            }
        }
    }

    let selected: Vec<(i32, i32)> = if offered.is_empty() {
        supported_1x.to_vec()
    } else {
        offered
            .into_iter()
            .filter(|v| supported_1x.contains(v))
            .collect()
    };

    let mut pv_items = Vec::with_capacity(selected.len());
    for (maj, min) in selected {
        pv_items.push(build_protocol_version(maj, min)?);
    }

    let batch = build_success_batch_item(OP_DISCOVER_VERSIONS, unique_batch_item_id, pv_items)?;
    build_response_message(client_major, client_minor, vec![batch])
}

const fn is_key_value_container_tag(tag: u32) -> bool {
    matches!(
        tag,
        TAG_SYMMETRIC_KEY
            | TAG_PUBLIC_KEY
            | TAG_PRIVATE_KEY
            | TAG_SPLIT_KEY
            | TAG_SECRET_DATA
            | TAG_PGP_KEY
            | TAG_KEY_BLOCK
            | TAG_KEY_VALUE
    )
}

fn is_structure_item(item: &Item) -> bool {
    let mut is_struct = false;
    item.with_value(|view| {
        is_struct = matches!(view, ValueView::Structure(_));
    });
    is_struct
}

/// Recursively translates KMIP 1.X `Attribute` (`0x420008`) items inside plaintext
/// `Key Value` (`0x420045`) structures into KMIP 2.0+ `Attributes` (`0x420125`).
/// Encrypted/wrapped `Key Value` items encoded as `ByteString` are left untouched.
fn translate_key_value_attrs_1x_to_2x(
    item: Item,
    cfg: TranslatorConfig,
    is_create_like: bool,
) -> Result<Item, String> {
    let tag = item.tag().raw();
    if !is_key_value_container_tag(tag) || !is_structure_item(&item) {
        return Ok(item);
    }

    if tag == TAG_KEY_VALUE {
        let mut out = Vec::new();
        let mut attrs_2x = Vec::new();
        for child in children_of(&item) {
            if child.tag().raw() == TAG_ATTRIBUTE {
                if let Some(a2x) = translate_attribute_1x_to_2x(&child, cfg, is_create_like)? {
                    attrs_2x.push(a2x);
                }
            } else {
                out.push(child);
            }
        }
        if !attrs_2x.is_empty() {
            out.push(ttlv_struct(TAG_ATTRIBUTES, attrs_2x)?);
        }
        return ttlv_struct(TAG_KEY_VALUE, out);
    }

    let mut out = Vec::new();
    for child in children_of(&item) {
        out.push(translate_key_value_attrs_1x_to_2x(
            child,
            cfg,
            is_create_like,
        )?);
    }
    ttlv_struct(tag, out)
}

/// Recursively translates KMIP 2.0+ `Attributes` (`0x420125`) inside plaintext
/// `Key Value` (`0x420045`) structures back into KMIP 1.X `Attribute` (`0x420008`) items.
fn translate_key_value_attrs_2x_to_1x(item: Item, client_minor: i32) -> Result<Item, String> {
    let tag = item.tag().raw();
    if !is_key_value_container_tag(tag) || !is_structure_item(&item) {
        return Ok(item);
    }

    if tag == TAG_KEY_VALUE {
        let mut out = Vec::new();
        for child in children_of(&item) {
            if child.tag().raw() == TAG_ATTRIBUTES {
                out.extend(translate_attributes_2x_to_1x(&child, client_minor)?);
            } else {
                out.push(child);
            }
        }
        return ttlv_struct(TAG_KEY_VALUE, out);
    }

    let mut out = Vec::new();
    for child in children_of(&item) {
        out.push(translate_key_value_attrs_2x_to_1x(child, client_minor)?);
    }
    ttlv_struct(tag, out)
}

/// Translates a KMIP 1.X `RequestMessage` into a KMIP 2.0 / 2.1 / 3.0 `RequestMessage`,
/// returning both the translated message and per-batch-item metadata for response restoration.
pub fn translate_request_1x_to_2x(
    req_1x: &Item,
    cfg: TranslatorConfig,
    template_store: &TemplateStore,
) -> Result<(Item, Vec<TranslatedBatchMeta>), String> {
    let (up_major, up_minor) = cfg.upstream_version.major_minor();
    let mut new_top = Vec::new();
    let mut metas = Vec::new();
    let mut batch_idx = 0usize;

    for child in children_of(req_1x) {
        match child.tag().raw() {
            TAG_REQUEST_HEADER => {
                new_top.push(rewrite_header_version(&child, up_major, up_minor)?);
            }
            TAG_BATCH_ITEM => {
                let (translated_item, meta) =
                    translate_request_batch_item(&child, batch_idx > 0, cfg, template_store)?;
                new_top.push(translated_item);
                metas.push(meta);
                batch_idx += 1;
            }
            _ => new_top.push(child),
        }
    }

    let msg = ttlv_struct(TAG_REQUEST_MESSAGE, new_top)?;
    Ok((msg, metas))
}

/// Translates a single KMIP 1.X request `Batch Item` into KMIP 2.0 / 2.1 / 3.0.
pub fn translate_request_batch_item(
    batch_item: &Item,
    can_use_id_placeholder: bool,
    cfg: TranslatorConfig,
    template_store: &TemplateStore,
) -> Result<(Item, TranslatedBatchMeta), String> {
    let original_op = find_child(batch_item, TAG_OPERATION)
        .and_then(|i| extract_enumeration(&i))
        .ok_or_else(|| "Batch Item missing Operation".to_string())?;

    let mut upstream_op = original_op;
    let mut out_children = Vec::new();

    for child in children_of(batch_item) {
        match child.tag().raw() {
            TAG_OPERATION => {
                // Inserted at index 0 after payload inspection
            }
            TAG_REQUEST_PAYLOAD => {
                let (translated_payload, maybe_new_op) = translate_request_payload(
                    original_op,
                    &child,
                    can_use_id_placeholder,
                    cfg,
                    template_store,
                )?;
                if let Some(new_op) = maybe_new_op {
                    upstream_op = new_op;
                }
                out_children.push(translated_payload);
            }
            _ => out_children.push(child),
        }
    }

    out_children.insert(
        0,
        ttlv_item(TAG_OPERATION, Value::enumeration(upstream_op))?,
    );
    let item = ttlv_struct(TAG_BATCH_ITEM, out_children)?;
    Ok((
        item,
        TranslatedBatchMeta {
            original_op_1x: original_op,
            upstream_op_2x: upstream_op,
        },
    ))
}

fn translate_request_payload(
    op: u32,
    payload: &Item,
    can_use_id_placeholder: bool,
    cfg: TranslatorConfig,
    template_store: &TemplateStore,
) -> Result<(Item, Option<u32>), String> {
    let mut out = Vec::new();
    let mut override_op = None;
    let mut has_unique_id = false;
    let mut locate_attrs_2x = Vec::new();

    for child in children_of(payload) {
        let tag = child.tag().raw();
        if tag == TAG_UNIQUE_IDENTIFIER {
            has_unique_id = true;
        }

        match tag {
            // Template-Attribute -> Attributes (Create, Register, Re-key, Derive Key, Certify, Re-certify, Create Split Key, Join Split Key, Import)
            TAG_TEMPLATE_ATTRIBUTE => {
                out.push(translate_template_attribute_to_2x(
                    &child,
                    TAG_ATTRIBUTES,
                    cfg,
                    template_store,
                )?);
            }
            TAG_COMMON_TEMPLATE_ATTRIBUTE => {
                out.push(translate_template_attribute_to_2x(
                    &child,
                    TAG_COMMON_ATTRIBUTES,
                    cfg,
                    template_store,
                )?);
            }
            TAG_PRIVATE_KEY_TEMPLATE_ATTRIBUTE => {
                out.push(translate_template_attribute_to_2x(
                    &child,
                    TAG_PRIVATE_KEY_ATTRIBUTES,
                    cfg,
                    template_store,
                )?);
            }
            TAG_PUBLIC_KEY_TEMPLATE_ATTRIBUTE => {
                out.push(translate_template_attribute_to_2x(
                    &child,
                    TAG_PUBLIC_KEY_ATTRIBUTES,
                    cfg,
                    template_store,
                )?);
            }
            // Get Attributes: Attribute Name (TextString) -> Attribute Reference (Enum or Vendor struct)
            TAG_ATTRIBUTE_NAME if op == OP_GET_ATTRIBUTES => {
                if let Some(name) = extract_text_string(&child) {
                    if let Some(attr_ref) = translate_attribute_name_to_reference(&name)? {
                        out.push(attr_ref);
                    }
                }
            }
            // Delete Attribute without index: Attribute Name -> Attribute Reference
            TAG_ATTRIBUTE_NAME if op == OP_DELETE_ATTRIBUTE => {
                if let Some(name) = extract_text_string(&child) {
                    if let Some(attr_ref) = translate_attribute_name_to_reference(&name)? {
                        out.push(attr_ref);
                    }
                }
            }
            // Attribute Index in Delete Attribute is handled by RMW in Kmip1xBridge; strip if reaching here
            TAG_ATTRIBUTE_INDEX if op == OP_DELETE_ATTRIBUTE => {}
            // Add Attribute: single-instance -> Set Attribute(New Attribute); multi-instance -> Add Attribute(New Attribute)
            TAG_ATTRIBUTE if op == OP_ADD_ATTRIBUTE => {
                if let Some(name) =
                    find_child(&child, TAG_ATTRIBUTE_NAME).and_then(|n| extract_text_string(&n))
                {
                    if let Some(spec) = lookup_by_name_1x(&name) {
                        if spec.cardinality == Cardinality::SingleInstance {
                            override_op = Some(OP_SET_ATTRIBUTE);
                        }
                    }
                }
                if let Some(a2x) = translate_attribute_1x_to_2x(&child, cfg, false)? {
                    out.push(ttlv_struct(TAG_NEW_ATTRIBUTE, vec![a2x])?);
                }
            }
            // Modify Attribute (single-instance path; multi-instance with Attribute Index uses RMW in Kmip1xBridge)
            TAG_ATTRIBUTE if op == OP_MODIFY_ATTRIBUTE => {
                if let Some(name) =
                    find_child(&child, TAG_ATTRIBUTE_NAME).and_then(|n| extract_text_string(&n))
                {
                    if let Some(spec) = lookup_by_name_1x(&name) {
                        if spec.cardinality == Cardinality::SingleInstance {
                            override_op = Some(OP_SET_ATTRIBUTE);
                        }
                    }
                }
                if let Some(a2x) = translate_attribute_1x_to_2x(&child, cfg, false)? {
                    out.push(ttlv_struct(TAG_NEW_ATTRIBUTE, vec![a2x])?);
                }
            }
            // Locate: list of 1.X Attribute items -> single 2.0+ Attributes structure
            TAG_ATTRIBUTE if op == OP_LOCATE => {
                if let Some(a2x) = translate_attribute_1x_to_2x(&child, cfg, false)? {
                    locate_attrs_2x.push(a2x);
                }
            }
            // Generic Attribute in any other payload -> wrap in Attributes
            TAG_ATTRIBUTE => {
                if let Some(a2x) = translate_attribute_1x_to_2x(&child, cfg, false)? {
                    locate_attrs_2x.push(a2x);
                }
            }
            // Query Function: pass through supported functions
            TAG_QUERY_FUNCTION if op == OP_QUERY => {
                out.push(child);
            }
            // Key Wrapping Specification: translate 1.X Attribute Name items to 2.0+ Attribute Reference
            TAG_KEY_WRAPPING_SPECIFICATION => {
                let mut kws_children = Vec::new();
                for sub in children_of(&child) {
                    if sub.tag().raw() == TAG_ATTRIBUTE_NAME {
                        if let Some(name) = extract_text_string(&sub) {
                            if let Some(attr_ref) = translate_attribute_name_to_reference(&name)? {
                                kws_children.push(attr_ref);
                                continue;
                            }
                        }
                    }
                    kws_children.push(sub);
                }
                out.push(ttlv_struct(TAG_KEY_WRAPPING_SPECIFICATION, kws_children)?);
            }
            // Managed objects / Key Block / Key Value: translate nested Key Value Attributes (1.X -> 2.0+)
            tag if is_key_value_container_tag(tag) => {
                out.push(translate_key_value_attrs_1x_to_2x(child, cfg, false)?);
            }
            // Everything else (Unique Identifier, Object Type, Crypto Params, Data, IV, etc.)
            _ => out.push(child),
        }
    }

    if !locate_attrs_2x.is_empty() {
        out.push(ttlv_struct(TAG_ATTRIBUTES, locate_attrs_2x)?);
    }

    // Inject explicit KMIP 2.0+ `ID Placeholder` enum when omitted in a chained 1.X batch item
    if can_use_id_placeholder && !has_unique_id && operation_supports_id_placeholder(op) {
        out.insert(
            0,
            ttlv_item(
                TAG_UNIQUE_IDENTIFIER,
                Value::enumeration(UNIQUE_ID_PLACEHOLDER_ENUM),
            )?,
        );
    }

    Ok((ttlv_struct(TAG_REQUEST_PAYLOAD, out)?, override_op))
}

/// Translates a KMIP 2.0 / 2.1 / 3.0 `ResponseMessage` back to the client's KMIP 1.X version.
pub fn translate_response_2x_to_1x(
    resp_2x: &Item,
    client_major: i32,
    client_minor: i32,
    metas: &[TranslatedBatchMeta],
) -> Result<Item, String> {
    let mut new_top = Vec::new();
    let mut batch_idx = 0usize;

    for child in children_of(resp_2x) {
        match child.tag().raw() {
            TAG_RESPONSE_HEADER => {
                new_top.push(rewrite_header_version(&child, client_major, client_minor)?);
            }
            TAG_BATCH_ITEM => {
                let meta = metas.get(batch_idx);
                new_top.push(translate_response_batch_item(&child, client_minor, meta)?);
                batch_idx += 1;
            }
            _ => new_top.push(child),
        }
    }

    ttlv_struct(TAG_RESPONSE_MESSAGE, new_top)
}

fn translate_response_batch_item(
    batch_item: &Item,
    client_minor: i32,
    meta: Option<&TranslatedBatchMeta>,
) -> Result<Item, String> {
    let upstream_op = find_child(batch_item, TAG_OPERATION)
        .and_then(|i| extract_enumeration(&i))
        .unwrap_or(0);
    let restored_op = meta.map_or(upstream_op, |m| m.original_op_1x);

    let mut out = Vec::new();
    for child in children_of(batch_item) {
        match child.tag().raw() {
            TAG_OPERATION => {
                out.push(ttlv_item(TAG_OPERATION, Value::enumeration(restored_op))?);
            }
            TAG_RESULT_REASON => {
                if let Some(r) = extract_enumeration(&child) {
                    out.push(ttlv_item(
                        TAG_RESULT_REASON,
                        Value::enumeration(downgrade_result_reason(r, client_minor)),
                    )?);
                } else {
                    out.push(child);
                }
            }
            TAG_RESPONSE_PAYLOAD => {
                out.push(translate_response_payload(
                    restored_op,
                    &child,
                    client_minor,
                )?);
            }
            _ => out.push(child),
        }
    }

    ttlv_struct(TAG_BATCH_ITEM, out)
}

fn translate_response_payload(
    op_1x: u32,
    payload: &Item,
    client_minor: i32,
) -> Result<Item, String> {
    let mut out = Vec::new();
    let mut seen_attr_list_names = std::collections::BTreeSet::new();

    for child in children_of(payload) {
        match child.tag().raw() {
            // Attributes (0x420125) -> list of KMIP 1.X Attribute (0x420008) items
            TAG_ATTRIBUTES => {
                out.extend(translate_attributes_2x_to_1x(&child, client_minor)?);
            }
            // Get Attribute List: Attribute Reference (Enum or Vendor) -> Attribute Name (TextString)
            TAG_ATTRIBUTE_REFERENCE if op_1x == OP_GET_ATTRIBUTE_LIST => {
                let maybe_name: Option<String> = if let Some(tag_val) = extract_enumeration(&child)
                {
                    if let Some(spec) = lookup_by_tag_2x(tag_val) {
                        if spec.min_minor_1x <= client_minor {
                            Some(spec.name_1x.to_string())
                        } else if client_minor == 0 {
                            match spec.name_1x {
                                "X.509 Certificate Identifier" => {
                                    Some("Certificate Identifier".to_string())
                                }
                                "X.509 Certificate Subject" => {
                                    Some("Certificate Subject".to_string())
                                }
                                "X.509 Certificate Issuer" => {
                                    Some("Certificate Issuer".to_string())
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else if v3_link_tag_to_link_type(tag_val).is_some() {
                        // Map KMIP 3.0 typed link reference back to "Link"
                        Some("Link".to_string())
                    } else {
                        None
                    }
                } else if let (Some(vendor), Some(aname)) = (
                    find_child(&child, TAG_VENDOR_IDENTIFICATION)
                        .and_then(|i| extract_text_string(&i)),
                    find_child(&child, TAG_ATTRIBUTE_NAME).and_then(|i| extract_text_string(&i)),
                ) {
                    if vendor == PROXY_VENDOR_ID {
                        Some(aname)
                    } else {
                        Some(format!("{vendor}-{aname}"))
                    }
                } else {
                    None
                };

                if let Some(name) = maybe_name {
                    if seen_attr_list_names.insert(name.clone()) {
                        out.push(ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string(name))?);
                    }
                }
            }
            // Query response allowlist filtering per client KMIP 1.X minor version
            TAG_OPERATION if op_1x == OP_QUERY => {
                if let Some(op_val) = extract_enumeration(&child) {
                    if is_operation_allowed_in_1x(op_val, client_minor) {
                        out.push(child);
                    }
                }
            }
            TAG_OBJECT_TYPE if op_1x == OP_QUERY => {
                if let Some(ot) = extract_enumeration(&child) {
                    if is_object_type_allowed_in_1x(ot, client_minor) {
                        out.push(child);
                    }
                }
            }
            TAG_QUERY_FUNCTION if op_1x == OP_QUERY => {
                if let Some(qf) = extract_enumeration(&child) {
                    if is_query_function_allowed_in_1x(qf, client_minor) {
                        out.push(child);
                    }
                }
            }
            TAG_CRYPTOGRAPHIC_ALGORITHM if op_1x == OP_QUERY => {
                if let Some(alg) = extract_enumeration(&child) {
                    if is_crypto_algorithm_allowed_in_1x(alg, client_minor) {
                        out.push(child);
                    }
                }
            }
            // Strip KMIP 2.0+/3.0-only Query capability tags (e.g., Quantum Safe Capability 0x420148)
            0x0042_0148 | 0x0042_0152 | 0x0042_015F | 0x0042_01B2 if op_1x == OP_QUERY => {}
            tag if is_key_value_container_tag(tag) => {
                out.push(translate_key_value_attrs_2x_to_1x(child, client_minor)?);
            }
            _ => out.push(child),
        }
    }

    ttlv_struct(TAG_RESPONSE_PAYLOAD, out)
}

/// Reverse translator for Server-to-Client operations (`Notify` 0x1B and `Put` 0x1C)
/// initiated by a KMIP 2.0 / 2.1 / 3.0 server toward a legacy KMIP 1.X client.
pub fn translate_server_to_client_request_2x_to_1x(
    server_req_2x: &Item,
    client_major: i32,
    client_minor: i32,
) -> Result<Item, String> {
    let mut out = Vec::new();
    for child in children_of(server_req_2x) {
        match child.tag().raw() {
            TAG_REQUEST_HEADER => {
                out.push(rewrite_header_version(&child, client_major, client_minor)?);
            }
            TAG_BATCH_ITEM => {
                let mut bi_children = Vec::new();
                for bi_child in children_of(&child) {
                    if bi_child.tag().raw() == TAG_REQUEST_PAYLOAD {
                        let mut p_children = Vec::new();
                        for p_child in children_of(&bi_child) {
                            if p_child.tag().raw() == TAG_ATTRIBUTES {
                                p_children
                                    .extend(translate_attributes_2x_to_1x(&p_child, client_minor)?);
                            } else if is_key_value_container_tag(p_child.tag().raw()) {
                                p_children.push(translate_key_value_attrs_2x_to_1x(
                                    p_child,
                                    client_minor,
                                )?);
                            } else {
                                p_children.push(p_child);
                            }
                        }
                        bi_children.push(ttlv_struct(TAG_REQUEST_PAYLOAD, p_children)?);
                    } else {
                        bi_children.push(bi_child);
                    }
                }
                out.push(ttlv_struct(TAG_BATCH_ITEM, bi_children)?);
            }
            _ => out.push(child),
        }
    }
    ttlv_struct(TAG_REQUEST_MESSAGE, out)
}

/// Reverse translator for Server-to-Client responses initiated by a legacy KMIP 1.X client
/// toward an upstream KMIP 2.0 / 2.1 / 3.0 server.
pub fn translate_server_to_client_response_1x_to_2x(
    resp_1x: &Item,
    upstream_version: crate::kmip1x::mapping::UpstreamKmipVersion,
) -> Result<Item, String> {
    let (up_maj, up_min) = upstream_version.major_minor();
    let mut out = Vec::new();
    for child in children_of(resp_1x) {
        if child.tag().raw() == TAG_RESPONSE_HEADER {
            out.push(rewrite_header_version(&child, up_maj, up_min)?);
        } else {
            out.push(child);
        }
    }
    ttlv_struct(TAG_RESPONSE_MESSAGE, out)
}

/// Helper to check whether a `Batch Item` is a `Modify Attribute` or `Delete Attribute`
/// that requires a Read-Modify-Write roundtrip because it targets a multi-instance attribute
/// or specifies an `Attribute Index`.
#[must_use]
pub fn needs_rmw_index_resolution(batch_item: &Item) -> bool {
    let Some(op) = find_child(batch_item, TAG_OPERATION).and_then(|i| extract_enumeration(&i))
    else {
        return false;
    };
    let Some(payload) = find_child(batch_item, TAG_REQUEST_PAYLOAD) else {
        return false;
    };

    match op {
        OP_MODIFY_ATTRIBUTE => {
            let Some(attr) = find_child(&payload, TAG_ATTRIBUTE) else {
                return false;
            };
            if find_child(&attr, TAG_ATTRIBUTE_INDEX).is_some() {
                return true;
            }
            find_child(&attr, TAG_ATTRIBUTE_NAME)
                .and_then(|n| extract_text_string(&n))
                .and_then(|name| lookup_by_name_1x(&name))
                .is_some_and(|spec| spec.cardinality == Cardinality::MultiInstance)
        }
        OP_DELETE_ATTRIBUTE => find_child(&payload, TAG_ATTRIBUTE_INDEX).is_some(),
        _ => false,
    }
}

/// Helper to check if a `Batch Item` targets a KMIP 1.X `Template` (`Object Type = 0x06`).
#[must_use]
pub fn is_template_object_request(batch_item: &Item) -> bool {
    let Some(payload) = find_child(batch_item, TAG_REQUEST_PAYLOAD) else {
        return false;
    };
    if find_child(&payload, TAG_OBJECT_TYPE).and_then(|i| extract_enumeration(&i))
        == Some(OBJECT_TYPE_TEMPLATE)
    {
        return true;
    }
    // Locate with Attribute { Name: "Object Type", Value: Template }
    for attr in find_children(&payload, TAG_ATTRIBUTE) {
        let is_ot = find_child(&attr, TAG_ATTRIBUTE_NAME)
            .and_then(|n| extract_text_string(&n))
            .is_some_and(|n| n == "Object Type");
        if is_ot
            && find_child(&attr, TAG_ATTRIBUTE_VALUE).and_then(|v| extract_enumeration(&v))
                == Some(OBJECT_TYPE_TEMPLATE)
        {
            return true;
        }
    }
    false
}

/// Helper to extract `Boolean` or `Enumeration` Asynchronous Indicator.
#[must_use]
pub fn is_async_request(req: &Item) -> bool {
    find_child(req, TAG_REQUEST_HEADER)
        .and_then(|h| find_child(&h, TAG_ASYNCHRONOUS_INDICATOR))
        .is_some_and(|ind| {
            extract_boolean(&ind).unwrap_or(false)
                || matches!(extract_enumeration(&ind), Some(1 | 2))
        })
}
