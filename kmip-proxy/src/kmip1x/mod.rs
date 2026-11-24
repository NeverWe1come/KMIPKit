//! KMIP 1.X (1.0–1.4) -> KMIP 2.0 / 2.1 / 3.0 protocol translation bridge.
//!
//! Architecture:
//! `KMIP 1.X Legacy App -> Kmip1xBridge -> UpstreamKmipTransport (kmipkit-transport) -> KMIP 2.0/2.1/3.0 Server`
#![forbid(unsafe_code)]

pub mod mapping;
pub mod s2c;
pub mod tags;
pub mod template_store;
pub mod translator;
pub mod ttlv_helpers;
pub mod upstream;

use std::sync::Arc;

use async_trait::async_trait;
use kmipkit_ttlv::{Item, Value};

use crate::codec::Format;
use crate::kmip1x::mapping::{
    OperationPolicyHandling, UpstreamKmipVersion, is_custom_attribute_1x, lookup_by_name_1x,
    operation_supports_id_placeholder, v3_link_tag_to_link_type,
};
use crate::kmip1x::tags::*;
use crate::kmip1x::template_store::TemplateStore;
use crate::kmip1x::translator::{
    TranslatedBatchMeta, TranslatorConfig, build_discover_versions_response_1x,
    extract_max_response_size, extract_protocol_version, is_template_object_request,
    needs_rmw_index_resolution, rewrite_header_version, translate_attribute_1x_to_2x,
    translate_attribute_name_to_reference, translate_request_1x_to_2x,
    translate_request_batch_item, translate_response_2x_to_1x,
};
use crate::kmip1x::ttlv_helpers::{
    build_error_batch_item, build_response_message, build_success_batch_item, children_of,
    clone_item, extract_byte_string, extract_enumeration, extract_integer, extract_text_string,
    find_child, find_children, ttlv_item, ttlv_struct,
};

/// Async transport abstraction for forwarding translated KMIP 2.0 / 2.1 / 3.0 TTLV messages.
#[async_trait]
pub trait UpstreamKmipTransport: Send + Sync + 'static {
    async fn send_ttlv(&self, request: &Item) -> Result<Item, String>;
}

/// Complete KMIP 1.X <-> KMIP 2.0 / 2.1 / 3.0 translation proxy engine.
pub struct Kmip1xBridge {
    upstream: Arc<dyn UpstreamKmipTransport>,
    config: TranslatorConfig,
    templates: Arc<TemplateStore>,
}

impl Kmip1xBridge {
    #[must_use]
    pub fn new(
        upstream: Arc<dyn UpstreamKmipTransport>,
        upstream_version: UpstreamKmipVersion,
    ) -> Self {
        Self::with_config(
            upstream,
            TranslatorConfig {
                upstream_version,
                ..TranslatorConfig::default()
            },
        )
    }

    #[must_use]
    pub fn with_config(upstream: Arc<dyn UpstreamKmipTransport>, config: TranslatorConfig) -> Self {
        Self {
            upstream,
            config,
            templates: Arc::new(TemplateStore::new()),
        }
    }

    #[must_use]
    pub fn template_store(&self) -> &Arc<TemplateStore> {
        &self.templates
    }

    #[must_use]
    pub const fn config(&self) -> TranslatorConfig {
        self.config
    }

    /// Dispatches an incoming KMIP request.
    ///
    /// - If `major >= 2`, forwards directly to upstream (zero-overhead pass-through).
    /// - If `major == 1`, translates KMIP 1.0–1.4 -> configured KMIP 2.0/2.1/3.0,
    ///   handles local interceptions (`Discover Versions`, `Template` lifecycle) and
    ///   Read-Modify-Write `Attribute Index` resolution, forwards to upstream, translates
    ///   the response back to the client's exact 1.X version, and enforces `Maximum Response Size`.
    pub async fn dispatch(&self, request: &Item) -> Result<Item, String> {
        let (client_major, client_minor) = extract_protocol_version(request).unwrap_or((1, 4));

        // Pass-through for modern KMIP 2.0 / 2.1 / 3.0 clients
        if client_major >= 2 {
            return self.upstream.send_ttlv(request).await;
        }

        let max_response_size = if self.config.enforce_max_response_size {
            extract_max_response_size(request)
        } else {
            None
        };

        let batch_items = find_children(request, TAG_BATCH_ITEM);
        if batch_items.is_empty() {
            return Err("KMIP 1.X RequestMessage contains no Batch Items".to_string());
        }

        // Fast-path: check if any batch item needs local interception or RMW
        let requires_per_item_orchestration = batch_items.iter().any(|bi| {
            let op = find_child(bi, TAG_OPERATION)
                .and_then(|i| extract_enumeration(&i))
                .unwrap_or(0);
            op == OP_DISCOVER_VERSIONS
                || is_template_object_request(bi)
                || self.targets_stored_template(bi)
                || needs_rmw_index_resolution(bi)
        });

        let response_1x = if !requires_per_item_orchestration {
            // Single-RTT full batch translation (preserves atomic batch order & ID Placeholder!)
            let (translated_req, metas) =
                match translate_request_1x_to_2x(request, self.config, &self.templates) {
                    Ok(res) => res,
                    Err(err_msg) => {
                        let mut err_items = Vec::new();
                        for bi in &batch_items {
                            let op = find_child(bi, TAG_OPERATION)
                                .and_then(|i| extract_enumeration(&i))
                                .unwrap_or(0);
                            let bid = find_child(bi, TAG_UNIQUE_BATCH_ITEM_ID)
                                .and_then(|i| extract_byte_string(&i));
                            let reason = if self.config.operation_policy_handling
                                == OperationPolicyHandling::Reject
                                && err_msg.contains("Operation Policy Name")
                            {
                                REASON_OPERATION_NOT_SUPPORTED
                            } else {
                                REASON_INVALID_FIELD
                            };
                            err_items.push(build_error_batch_item(
                                op,
                                bid.as_deref(),
                                reason,
                                &err_msg,
                            )?);
                        }
                        return build_response_message(client_major, client_minor, err_items);
                    }
                };
            let upstream_resp = self.upstream.send_ttlv(&translated_req).await?;
            translate_response_2x_to_1x(&upstream_resp, client_major, client_minor, &metas)?
        } else {
            // Per-item orchestrated execution (Discover Versions, Template store, or RMW)
            let req_header = find_child(request, TAG_REQUEST_HEADER)
                .ok_or_else(|| "missing Request Header".to_string())?;
            let mut resp_items = Vec::with_capacity(batch_items.len());
            let mut last_uid: Option<String> = None;

            for (idx, bi) in batch_items.iter().enumerate() {
                let item_resp = self
                    .execute_orchestrated_batch_item(
                        &req_header,
                        bi,
                        idx > 0,
                        last_uid.as_deref(),
                        client_major,
                        client_minor,
                    )
                    .await?;
                if let Some(uid) = extract_response_uid(&item_resp) {
                    last_uid = Some(uid);
                }
                resp_items.push(item_resp);
            }
            build_response_message(client_major, client_minor, resp_items)?
        };

        // Enforce KMIP 1.X Maximum Response Size on final serialized 1.X TTLV payload
        if let Some(limit) = max_response_size {
            let encoded = Format::Ttlv.encode(&response_1x)?;
            if encoded.len() > limit {
                let first_op = find_child(&batch_items[0], TAG_OPERATION)
                    .and_then(|i| extract_enumeration(&i))
                    .unwrap_or(0);
                let first_bid = find_child(&batch_items[0], TAG_UNIQUE_BATCH_ITEM_ID)
                    .and_then(|i| extract_byte_string(&i));
                let err_bi = build_error_batch_item(
                    first_op,
                    first_bid.as_deref(),
                    REASON_RESPONSE_TOO_LARGE,
                    "serialized KMIP 1.X response exceeds Maximum Response Size",
                )?;
                return build_response_message(client_major, client_minor, vec![err_bi]);
            }
        }

        Ok(response_1x)
    }

    fn targets_stored_template(&self, batch_item: &Item) -> bool {
        let Some(payload) = find_child(batch_item, TAG_REQUEST_PAYLOAD) else {
            return false;
        };
        find_child(&payload, TAG_UNIQUE_IDENTIFIER)
            .and_then(|i| extract_text_string(&i))
            .is_some_and(|id| self.templates.contains_id(&id))
    }

    async fn execute_orchestrated_batch_item(
        &self,
        req_header: &Item,
        batch_item: &Item,
        can_use_id_placeholder: bool,
        last_uid: Option<&str>,
        client_major: i32,
        client_minor: i32,
    ) -> Result<Item, String> {
        let op = find_child(batch_item, TAG_OPERATION)
            .and_then(|i| extract_enumeration(&i))
            .ok_or_else(|| "Batch Item missing Operation".to_string())?;
        let bid =
            find_child(batch_item, TAG_UNIQUE_BATCH_ITEM_ID).and_then(|i| extract_byte_string(&i));
        let payload = find_child(batch_item, TAG_REQUEST_PAYLOAD);

        // 1. Local interception: Discover Versions
        if op == OP_DISCOVER_VERSIONS {
            let full_resp = build_discover_versions_response_1x(
                client_major,
                client_minor,
                bid.as_deref(),
                payload.as_ref(),
            )?;
            return find_child(&full_resp, TAG_BATCH_ITEM)
                .ok_or_else(|| "internal Discover Versions response error".to_string());
        }

        // 2. Local emulation: KMIP 1.X Template lifecycle
        if is_template_object_request(batch_item) || self.targets_stored_template(batch_item) {
            return self.handle_template_operation(op, bid.as_deref(), payload.as_ref());
        }

        // 3. Read-Modify-Write for Modify Attribute / Delete Attribute with Attribute Index
        if needs_rmw_index_resolution(batch_item) {
            return self
                .handle_rmw_attribute_operation(
                    req_header,
                    op,
                    bid.as_deref(),
                    payload.as_ref(),
                    if can_use_id_placeholder {
                        last_uid
                    } else {
                        None
                    },
                    client_minor,
                )
                .await;
        }

        // 4. Standard single-item forwarding
        // Because each orchestrated item is sent as a standalone 1-item RequestMessage,
        // replace implicit ID Placeholder with the tracked `last_uid` if present.
        let effective_bi = if can_use_id_placeholder && operation_supports_id_placeholder(op) {
            if let Some(uid) = last_uid {
                inject_uid_if_missing(batch_item, uid)?
            } else {
                clone_item(batch_item)?
            }
        } else {
            clone_item(batch_item)?
        };

        let (up_major, up_minor) = self.config.upstream_version.major_minor();
        let mut single_header = rewrite_header_version(req_header, up_major, up_minor)?;
        single_header = set_header_batch_count(&single_header, 1)?;
        let (translated_bi, meta) = match translate_request_batch_item(
            &effective_bi,
            false,
            self.config,
            &self.templates,
        ) {
            Ok(res) => res,
            Err(err_msg) => {
                let reason = if self.config.operation_policy_handling
                    == OperationPolicyHandling::Reject
                    && err_msg.contains("Operation Policy Name")
                {
                    REASON_OPERATION_NOT_SUPPORTED
                } else {
                    REASON_INVALID_FIELD
                };
                return build_error_batch_item(op, bid.as_deref(), reason, &err_msg);
            }
        };
        let single_req = ttlv_struct(TAG_REQUEST_MESSAGE, vec![single_header, translated_bi])?;
        let up_resp = self.upstream.send_ttlv(&single_req).await?;
        let resp_1x = translate_response_2x_to_1x(&up_resp, client_major, client_minor, &[meta])?;
        find_child(&resp_1x, TAG_BATCH_ITEM)
            .ok_or_else(|| "upstream response missing Batch Item".to_string())
    }

    fn handle_template_operation(
        &self,
        op: u32,
        bid: Option<&[u8]>,
        payload: Option<&Item>,
    ) -> Result<Item, String> {
        let Some(p) = payload else {
            return build_error_batch_item(op, bid, REASON_MISSING_DATA, "missing request payload");
        };

        match op {
            OP_REGISTER => {
                let mut names = Vec::new();
                let mut attrs = Vec::new();

                if let Some(tpl_attr) = find_child(p, TAG_TEMPLATE_ATTRIBUTE) {
                    names.extend(TemplateStore::extract_names_from_structure(&tpl_attr));
                    attrs.extend(find_children(&tpl_attr, TAG_ATTRIBUTE));
                }
                if let Some(tpl_obj) = find_child(p, TAG_TEMPLATE) {
                    names.extend(TemplateStore::extract_names_from_structure(&tpl_obj));
                    attrs.extend(find_children(&tpl_obj, TAG_ATTRIBUTE));
                }

                let uid = self.templates.register(names, attrs);
                build_success_batch_item(
                    OP_REGISTER,
                    bid,
                    vec![
                        ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_TEMPLATE))?,
                        ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                    ],
                )
            }
            OP_LOCATE => {
                let mut name_filter = None;
                for attr in find_children(p, TAG_ATTRIBUTE) {
                    let is_name = find_child(&attr, TAG_ATTRIBUTE_NAME)
                        .and_then(|n| extract_text_string(&n))
                        .is_some_and(|n| n == "Name");
                    if is_name {
                        name_filter = find_child(&attr, TAG_ATTRIBUTE_VALUE)
                            .and_then(|v| find_child(&v, TAG_NAME_VALUE))
                            .and_then(|nv| extract_text_string(&nv));
                    }
                }
                let ids = self.templates.locate(name_filter.as_deref());
                let mut items = Vec::with_capacity(ids.len());
                for id in ids {
                    items.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id))?);
                }
                build_success_batch_item(OP_LOCATE, bid, items)
            }
            OP_GET => {
                let Some(uid) =
                    find_child(p, TAG_UNIQUE_IDENTIFIER).and_then(|i| extract_text_string(&i))
                else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_MISSING_DATA,
                        "missing Unique Identifier",
                    );
                };
                let Some(tpl) = self.templates.get_by_id(&uid) else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_ITEM_NOT_FOUND,
                        "template not found",
                    );
                };
                build_success_batch_item(
                    OP_GET,
                    bid,
                    vec![
                        ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(OBJECT_TYPE_TEMPLATE))?,
                        ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
                        tpl.to_template_object()?,
                    ],
                )
            }
            OP_GET_ATTRIBUTES => {
                let Some(uid) =
                    find_child(p, TAG_UNIQUE_IDENTIFIER).and_then(|i| extract_text_string(&i))
                else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_MISSING_DATA,
                        "missing Unique Identifier",
                    );
                };
                let req_names: Vec<String> = find_children(p, TAG_ATTRIBUTE_NAME)
                    .into_iter()
                    .filter_map(|i| extract_text_string(&i))
                    .collect();
                match self
                    .templates
                    .build_get_attributes_items(&uid, &req_names)?
                {
                    Some(items) => build_success_batch_item(OP_GET_ATTRIBUTES, bid, items),
                    None => {
                        build_error_batch_item(op, bid, REASON_ITEM_NOT_FOUND, "template not found")
                    }
                }
            }
            OP_GET_ATTRIBUTE_LIST => {
                let Some(uid) =
                    find_child(p, TAG_UNIQUE_IDENTIFIER).and_then(|i| extract_text_string(&i))
                else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_MISSING_DATA,
                        "missing Unique Identifier",
                    );
                };
                let Some(tpl) = self.templates.get_by_id(&uid) else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_ITEM_NOT_FOUND,
                        "template not found",
                    );
                };
                let mut items = vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?];
                for name in tpl.attribute_names() {
                    items.push(ttlv_item(TAG_ATTRIBUTE_NAME, Value::text_string(name))?);
                }
                build_success_batch_item(OP_GET_ATTRIBUTE_LIST, bid, items)
            }
            OP_DESTROY => {
                let Some(uid) =
                    find_child(p, TAG_UNIQUE_IDENTIFIER).and_then(|i| extract_text_string(&i))
                else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_MISSING_DATA,
                        "missing Unique Identifier",
                    );
                };
                if self.templates.destroy(&uid) {
                    build_success_batch_item(
                        OP_DESTROY,
                        bid,
                        vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?],
                    )
                } else {
                    build_error_batch_item(op, bid, REASON_ITEM_NOT_FOUND, "template not found")
                }
            }
            _ => build_error_batch_item(
                op,
                bid,
                REASON_OPERATION_NOT_SUPPORTED,
                "operation not supported on local Template object",
            ),
        }
    }

    /// Executes Read-Modify-Write for KMIP 1.X `Modify Attribute` and `Delete Attribute`
    /// targeting a specific `Attribute Index`:
    /// 1. Sends `Get Attributes` upstream for the target attribute reference.
    /// 2. Locates the exact instance at `Attribute Index` (default 0).
    /// 3. Sends KMIP 2.0+ `Modify Attribute(Current Attribute, New Attribute)` or
    ///    `Delete Attribute(Current Attribute)` upstream.
    async fn handle_rmw_attribute_operation(
        &self,
        req_header: &Item,
        op: u32,
        bid: Option<&[u8]>,
        payload: Option<&Item>,
        fallback_uid: Option<&str>,
        client_minor: i32,
    ) -> Result<Item, String> {
        let Some(p) = payload else {
            return build_error_batch_item(op, bid, REASON_MISSING_DATA, "missing request payload");
        };
        let Some(uid) = find_child(p, TAG_UNIQUE_IDENTIFIER)
            .and_then(|i| extract_text_string(&i))
            .or_else(|| fallback_uid.map(ToOwned::to_owned))
        else {
            return build_error_batch_item(
                op,
                bid,
                REASON_MISSING_DATA,
                "missing Unique Identifier",
            );
        };

        let (attr_name, target_index, new_attr_1x) = match op {
            OP_MODIFY_ATTRIBUTE => {
                let Some(attr) = find_child(p, TAG_ATTRIBUTE) else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_MISSING_DATA,
                        "missing Attribute",
                    );
                };
                let Some(name) =
                    find_child(&attr, TAG_ATTRIBUTE_NAME).and_then(|i| extract_text_string(&i))
                else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_MISSING_DATA,
                        "missing Attribute Name",
                    );
                };
                let idx = find_child(&attr, TAG_ATTRIBUTE_INDEX)
                    .and_then(|i| extract_integer(&i))
                    .unwrap_or(0);
                (name, idx, Some(attr))
            }
            OP_DELETE_ATTRIBUTE => {
                let Some(name) =
                    find_child(p, TAG_ATTRIBUTE_NAME).and_then(|i| extract_text_string(&i))
                else {
                    return build_error_batch_item(
                        op,
                        bid,
                        REASON_MISSING_DATA,
                        "missing Attribute Name",
                    );
                };
                let idx = find_child(p, TAG_ATTRIBUTE_INDEX)
                    .and_then(|i| extract_integer(&i))
                    .unwrap_or(0);
                (name, idx, None)
            }
            _ => {
                return build_error_batch_item(
                    op,
                    bid,
                    REASON_GENERAL_FAILURE,
                    "unexpected RMW op",
                );
            }
        };

        if target_index < 0 {
            return build_error_batch_item(
                op,
                bid,
                REASON_INVALID_FIELD,
                "negative Attribute Index",
            );
        }

        let Some(attr_ref) = translate_attribute_name_to_reference(&attr_name)? else {
            return build_error_batch_item(
                op,
                bid,
                REASON_INVALID_FIELD,
                "unknown attribute name for index resolution",
            );
        };

        // Step 1: Read current instances from upstream via Get Attributes
        let (up_major, up_minor) = self.config.upstream_version.major_minor();
        let up_header =
            set_header_batch_count(&rewrite_header_version(req_header, up_major, up_minor)?, 1)?;
        let get_bi = ttlv_struct(
            TAG_BATCH_ITEM,
            vec![
                ttlv_item(TAG_OPERATION, Value::enumeration(OP_GET_ATTRIBUTES))?,
                ttlv_struct(
                    TAG_REQUEST_PAYLOAD,
                    vec![
                        ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid.clone()))?,
                        attr_ref,
                    ],
                )?,
            ],
        )?;
        let get_req = ttlv_struct(TAG_REQUEST_MESSAGE, vec![clone_item(&up_header)?, get_bi])?;
        let get_resp = self.upstream.send_ttlv(&get_req).await?;

        let get_resp_bi = find_child(&get_resp, TAG_BATCH_ITEM)
            .ok_or_else(|| "upstream GetAttributes missing Batch Item".to_string())?;
        let get_status = find_child(&get_resp_bi, TAG_RESULT_STATUS)
            .and_then(|i| extract_enumeration(&i))
            .unwrap_or(STATUS_OPERATION_FAILED);
        if get_status != STATUS_SUCCESS {
            let reason = find_child(&get_resp_bi, TAG_RESULT_REASON)
                .and_then(|i| extract_enumeration(&i))
                .map(|r| crate::kmip1x::mapping::downgrade_result_reason(r, client_minor))
                .unwrap_or(REASON_ITEM_NOT_FOUND);
            return build_error_batch_item(
                op,
                bid,
                reason,
                "failed reading attribute instances for index resolution",
            );
        }

        let get_payload = find_child(&get_resp_bi, TAG_RESPONSE_PAYLOAD)
            .ok_or_else(|| "upstream GetAttributes missing Response Payload".to_string())?;
        let attrs_struct = find_child(&get_payload, TAG_ATTRIBUTES)
            .ok_or_else(|| "upstream GetAttributes missing Attributes".to_string())?;

        // Filter matching instances in order (supporting standard, KMIP 3.0 typed Links, and custom vendor attrs)
        let expected_tag = lookup_by_name_1x(&attr_name).map(|s| s.tag_2x);
        let matching_instances: Vec<Item> = children_of(&attrs_struct)
            .into_iter()
            .filter(|c| match expected_tag {
                Some(TAG_LINK) => {
                    c.tag().raw() == TAG_LINK || v3_link_tag_to_link_type(c.tag().raw()).is_some()
                }
                Some(t) => c.tag().raw() == t,
                None => {
                    if c.tag().raw() != TAG_ATTRIBUTE {
                        return false;
                    }
                    let (expected_vendor, expected_attr_name) =
                        if is_custom_attribute_1x(&attr_name) {
                            attr_name.split_once('-').unwrap_or(("x", &attr_name))
                        } else {
                            (
                                crate::kmip1x::translator::PROXY_VENDOR_ID,
                                attr_name.as_str(),
                            )
                        };
                    let actual_vendor = find_child(c, TAG_VENDOR_IDENTIFICATION)
                        .and_then(|i| extract_text_string(&i));
                    let actual_name =
                        find_child(c, TAG_ATTRIBUTE_NAME).and_then(|i| extract_text_string(&i));
                    actual_vendor.as_deref() == Some(expected_vendor)
                        && actual_name.as_deref() == Some(expected_attr_name)
                }
            })
            .collect();

        let Some(current_val_2x) = matching_instances.get(target_index as usize) else {
            return build_error_batch_item(
                op,
                bid,
                REASON_ITEM_NOT_FOUND,
                "Attribute Index out of bounds",
            );
        };

        // Step 2: Execute Modify Attribute(Current, New) or Delete Attribute(Current)
        let mut write_payload_items = vec![
            ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid))?,
            ttlv_struct(TAG_CURRENT_ATTRIBUTE, vec![clone_item(current_val_2x)?])?,
        ];

        if let Some(new_1x) = new_attr_1x {
            let Some(new_2x) = translate_attribute_1x_to_2x(&new_1x, self.config, false)? else {
                return build_error_batch_item(
                    op,
                    bid,
                    REASON_INVALID_FIELD,
                    "cannot modify dropped attribute",
                );
            };
            write_payload_items.push(ttlv_struct(TAG_NEW_ATTRIBUTE, vec![new_2x])?);
        }

        let write_bi = ttlv_struct(
            TAG_BATCH_ITEM,
            vec![
                ttlv_item(TAG_OPERATION, Value::enumeration(op))?,
                ttlv_struct(TAG_REQUEST_PAYLOAD, write_payload_items)?,
            ],
        )?;
        let write_req = ttlv_struct(TAG_REQUEST_MESSAGE, vec![up_header, write_bi])?;
        let write_resp = self.upstream.send_ttlv(&write_req).await?;

        let meta = TranslatedBatchMeta {
            original_op_1x: op,
            upstream_op_2x: op,
        };
        let resp_1x = translate_response_2x_to_1x(&write_resp, 1, client_minor, &[meta])?;
        let mut resp_bi = find_child(&resp_1x, TAG_BATCH_ITEM)
            .ok_or_else(|| "upstream RMW response missing Batch Item".to_string())?;

        // Re-attach original Unique Batch Item ID if client provided one
        if let Some(bid_bytes) = bid {
            let mut bi_children = children_of(&resp_bi);
            if find_child(&resp_bi, TAG_UNIQUE_BATCH_ITEM_ID).is_none() {
                bi_children.insert(
                    1,
                    ttlv_item(
                        TAG_UNIQUE_BATCH_ITEM_ID,
                        Value::byte_string(bid_bytes.to_vec()),
                    )?,
                );
                resp_bi = ttlv_struct(TAG_BATCH_ITEM, bi_children)?;
            }
        }

        // Ensure ModifyAttribute/DeleteAttribute 1.X response payload preserves the target Attribute Index
        if let Some(rp) = find_child(&resp_bi, TAG_RESPONSE_PAYLOAD) {
            let mut updated_rp_children = Vec::new();
            let mut modified_rp = false;
            for child in children_of(&rp) {
                if child.tag().raw() == TAG_ATTRIBUTE
                    && find_child(&child, TAG_ATTRIBUTE_INDEX).is_none()
                {
                    let mut attr_children = children_of(&child);
                    if !attr_children.is_empty() {
                        attr_children.insert(
                            1,
                            ttlv_item(TAG_ATTRIBUTE_INDEX, Value::integer(target_index))?,
                        );
                        updated_rp_children.push(ttlv_struct(TAG_ATTRIBUTE, attr_children)?);
                        modified_rp = true;
                        continue;
                    }
                }
                updated_rp_children.push(child);
            }
            if modified_rp {
                let mut bi_children = Vec::new();
                for c in children_of(&resp_bi) {
                    if c.tag().raw() == TAG_RESPONSE_PAYLOAD {
                        bi_children.push(ttlv_struct(
                            TAG_RESPONSE_PAYLOAD,
                            std::mem::take(&mut updated_rp_children),
                        )?);
                    } else {
                        bi_children.push(c);
                    }
                }
                resp_bi = ttlv_struct(TAG_BATCH_ITEM, bi_children)?;
            }
        }

        Ok(resp_bi)
    }
}

fn set_header_batch_count(header: &Item, count: i32) -> Result<Item, String> {
    let mut children = Vec::new();
    let mut found = false;
    for child in children_of(header) {
        if child.tag().raw() == TAG_BATCH_COUNT {
            children.push(ttlv_item(TAG_BATCH_COUNT, Value::integer(count))?);
            found = true;
        } else {
            children.push(child);
        }
    }
    if !found {
        children.push(ttlv_item(TAG_BATCH_COUNT, Value::integer(count))?);
    }
    ttlv_struct(header.tag().raw(), children)
}

fn extract_response_uid(resp_bi: &Item) -> Option<String> {
    let payload = find_child(resp_bi, TAG_RESPONSE_PAYLOAD)?;
    find_child(&payload, TAG_UNIQUE_IDENTIFIER).and_then(|i| extract_text_string(&i))
}

fn inject_uid_if_missing(batch_item: &Item, uid: &str) -> Result<Item, String> {
    let mut bi_children = Vec::new();
    for child in children_of(batch_item) {
        if child.tag().raw() == TAG_REQUEST_PAYLOAD {
            let mut p_children = children_of(&child);
            if find_child(&child, TAG_UNIQUE_IDENTIFIER).is_none() {
                p_children.insert(
                    0,
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(uid.to_string()))?,
                );
            }
            bi_children.push(ttlv_struct(TAG_REQUEST_PAYLOAD, p_children)?);
        } else {
            bi_children.push(child);
        }
    }
    ttlv_struct(TAG_BATCH_ITEM, bi_children)
}
