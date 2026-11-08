//! Central KMIP operation dispatcher for kmipkit-proxy.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use crate::cache::MetadataCache;
use crate::config::KeyCustodyMode;
use crate::providers::{
    CloudKmsProvider, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams, KeyMaterialPayload,
};
use crate::tag_codec::{KmipAttributes, TAG_ACT_DATE, TAG_STATE, TagCodec};

// Protocol operations
pub const CREATE_OPERATION: u32 = 0x0000_0001;
pub const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
pub const REGISTER_OPERATION: u32 = 0x0000_0003;
pub const LOCATE_OPERATION: u32 = 0x0000_0008;
pub const GET_OPERATION: u32 = 0x0000_000A;
pub const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
pub const GET_ATTRIBUTE_LIST_OPERATION: u32 = 0x0000_000C;
pub const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_000D;
pub const MODIFY_ATTRIBUTE_OPERATION: u32 = 0x0000_000E;
pub const DELETE_ATTRIBUTE_OPERATION: u32 = 0x0000_000F;
pub const ACTIVATE_OPERATION: u32 = 0x0000_0012;
pub const REVOKE_OPERATION: u32 = 0x0000_0013;
pub const DESTROY_OPERATION: u32 = 0x0000_0014;
pub const QUERY_OPERATION: u32 = 0x0000_0018;
pub const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
pub const ENCRYPT_OPERATION: u32 = 0x0000_001F;
pub const DECRYPT_OPERATION: u32 = 0x0000_0020;
pub const SIGN_OPERATION: u32 = 0x0000_0021;
pub const SIGNATURE_VERIFY_OPERATION: u32 = 0x0000_0022;
pub const MAC_OPERATION: u32 = 0x0000_0023;
pub const MAC_VERIFY_OPERATION: u32 = 0x0000_0024;
pub const RNG_RETRIEVE_OPERATION: u32 = 0x0000_0025;
pub const PING_OPERATION: u32 = 0x0000_003B;
pub const DEACTIVATE_OPERATION: u32 = 0x0000_003E;

// TTLV Tags
pub const TAG_REQUEST_MESSAGE: u32 = 0x0042_0078;
pub const TAG_REQUEST_HEADER: u32 = 0x0042_0077;
pub const TAG_RESPONSE_MESSAGE: u32 = 0x0042_007B;
pub const TAG_RESPONSE_HEADER: u32 = 0x0042_007A;
pub const TAG_PROTOCOL_VERSION: u32 = 0x0042_0069;
pub const TAG_PROTOCOL_VERSION_MAJOR: u32 = 0x0042_006A;
pub const TAG_PROTOCOL_VERSION_MINOR: u32 = 0x0042_006B;
pub const TAG_TIME_STAMP: u32 = 0x0042_0092;
pub const TAG_BATCH_COUNT: u32 = 0x0042_000D;
pub const TAG_BATCH_ITEM: u32 = 0x0042_000F;
pub const TAG_OPERATION: u32 = 0x0042_005C;
pub const TAG_UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
pub const TAG_RESULT_STATUS: u32 = 0x0042_007F;
pub const TAG_RESULT_REASON: u32 = 0x0042_007E;
pub const TAG_RESULT_MESSAGE: u32 = 0x0042_007D;
pub const TAG_REQUEST_PAYLOAD: u32 = 0x0042_0079;
pub const TAG_RESPONSE_PAYLOAD: u32 = 0x0042_007C;
pub const TAG_UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
pub const TAG_OBJECT_TYPE: u32 = 0x0042_0057;
pub const TAG_DATA: u32 = 0x0042_00C2;
pub const TAG_DATA_LENGTH: u32 = 0x0042_00CD;
pub const TAG_KEY_BLOCK: u32 = 0x0042_0040;
pub const TAG_KEY_FORMAT_TYPE: u32 = 0x0042_0042;
pub const TAG_KEY_COMPRESSION_TYPE: u32 = 0x0042_0041;
pub const TAG_KEY_VALUE: u32 = 0x0042_0045;
pub const TAG_KEY_MATERIAL: u32 = 0x0042_0043;
pub const TAG_CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
pub const TAG_CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
pub const TAG_CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
pub const TAG_STATE_TAG: u32 = 0x0042_008D;
pub const TAG_INITIAL_DATE: u32 = 0x0042_0038;
pub const TAG_ACTIVATION_DATE: u32 = 0x0042_0001;
pub const TAG_PROTECT_STOP_DATE: u32 = 0x0042_0071;
pub const TAG_DEACTIVATION_DATE: u32 = 0x0042_002D;
pub const TAG_NAME: u32 = 0x0042_0053;
pub const TAG_NAME_VALUE: u32 = 0x0042_0055;
pub const TAG_NAME_TYPE: u32 = 0x0042_0054;
pub const TAG_ATTRIBUTES: u32 = 0x0042_0125;
pub const TAG_ATTRIBUTE_NAME: u32 = 0x0042_000A;
pub const TAG_VALIDITY_INDICATOR: u32 = 0x0042_009A;
pub const TAG_SIGNATURE_DATA: u32 = 0x0042_008B;

pub const STATUS_SUCCESS: u32 = 0;
pub const STATUS_OPERATION_FAILED: u32 = 1;

pub const REASON_ITEM_NOT_FOUND: u32 = 0x0000_0001;
pub const REASON_OPERATION_NOT_SUPPORTED: u32 = 0x0000_0005;
pub const REASON_GENERAL_FAILURE: u32 = 0x0000_0100;

pub type RequestDispatcher = Dispatcher;

#[derive(Clone)]
pub struct Dispatcher {
    provider: Arc<dyn CloudKmsProvider>,
    cache: Arc<MetadataCache>,
    custody_mode: KeyCustodyMode,
}

impl Dispatcher {
    #[must_use]
    pub fn new(
        provider: Arc<dyn CloudKmsProvider>,
        behavior: crate::config::ProxyBehaviorConfig,
    ) -> Self {
        let cache = Arc::new(MetadataCache::new(Duration::from_secs(
            behavior.metadata_cache_ttl_secs,
        )));
        Self {
            provider,
            cache,
            custody_mode: behavior.key_custody_mode,
        }
    }

    #[must_use]
    pub fn with_cache_and_custody(
        provider: Arc<dyn CloudKmsProvider>,
        cache: Arc<MetadataCache>,
        custody_mode: KeyCustodyMode,
    ) -> Self {
        Self {
            provider,
            cache,
            custody_mode,
        }
    }

    pub async fn dispatch(&self, request: &Item) -> Result<Item, String> {
        let (req_major, req_minor, items) = extract_request_parts(request)?;

        let major = req_major;
        let minor = req_minor;

        let mut response_items = Vec::new();

        for item in items {
            let (op, unique_batch_id, payload) = extract_batch_item_parts(&item)?;
            let item_res = self
                .execute_operation(major, minor, op, payload.as_ref())
                .await;

            let res_item = build_batch_response_item(op, unique_batch_id.as_deref(), item_res);
            response_items.push(res_item);
        }

        build_response_message(major, minor, response_items)
    }

    async fn execute_operation(
        &self,
        _major: u32,
        _minor: u32,
        op: u32,
        payload: Option<&Structure>,
    ) -> Result<Vec<Item>, (u32, &'static str)> {
        match op {
            DISCOVER_VERSIONS_OPERATION => {
                let mut offered_versions = Vec::new();
                if let Some(p) = payload {
                    for child in p.view().children() {
                        if child.tag().raw() == TAG_PROTOCOL_VERSION {
                            if let Some(v) = extract_version_tuple(child) {
                                offered_versions.push(v);
                            }
                        }
                    }
                }

                let versions_to_return: Vec<(u32, u32)> = if offered_versions.is_empty() {
                    vec![(2, 0), (2, 1), (3, 0)]
                } else {
                    offered_versions
                        .into_iter()
                        .filter(|(maj, min)| matches!((*maj, *min), (2, 0) | (2, 1) | (3, 0)))
                        .collect()
                };

                let mut res = Vec::new();
                for (maj, min) in versions_to_return {
                    let mut version_struct = Structure::new();
                    let _ = version_struct.try_push(ttlv_item(
                        TAG_PROTOCOL_VERSION_MAJOR,
                        Value::integer(maj as i32),
                    ));
                    let _ = version_struct.try_push(ttlv_item(
                        TAG_PROTOCOL_VERSION_MINOR,
                        Value::integer(min as i32),
                    ));
                    res.push(ttlv_item(
                        TAG_PROTOCOL_VERSION,
                        Value::structure(version_struct),
                    ));
                }
                Ok(res)
            }

            QUERY_OPERATION => {
                let mut res = Vec::new();
                for op_code in [
                    CREATE_OPERATION,
                    REGISTER_OPERATION,
                    LOCATE_OPERATION,
                    GET_OPERATION,
                    GET_ATTRIBUTES_OPERATION,
                    ADD_ATTRIBUTE_OPERATION,
                    MODIFY_ATTRIBUTE_OPERATION,
                    DELETE_ATTRIBUTE_OPERATION,
                    ACTIVATE_OPERATION,
                    REVOKE_OPERATION,
                    DEACTIVATE_OPERATION,
                    DESTROY_OPERATION,
                    QUERY_OPERATION,
                    DISCOVER_VERSIONS_OPERATION,
                    ENCRYPT_OPERATION,
                    DECRYPT_OPERATION,
                    SIGN_OPERATION,
                    SIGNATURE_VERIFY_OPERATION,
                    RNG_RETRIEVE_OPERATION,
                    PING_OPERATION,
                ] {
                    res.push(ttlv_item(TAG_OPERATION, Value::enumeration(op_code)));
                }
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(0x02))); // SymmetricKey
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(0x03))); // PublicKey
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(0x04))); // PrivateKey
                Ok(res)
            }

            CREATE_OPERATION => {
                let obj_type = payload
                    .and_then(|p| find_child(p, TAG_OBJECT_TYPE))
                    .and_then(|item| extract_enumeration(&item))
                    .unwrap_or(2); // SymmetricKey

                let len_bits = payload
                    .and_then(|p| find_child(p, TAG_ATTRIBUTES))
                    .and_then(|attr| find_child_in_item(&attr, TAG_CRYPTOGRAPHIC_LENGTH))
                    .and_then(|item| extract_integer(&item))
                    .unwrap_or(256);

                let usage_mask = payload
                    .and_then(|p| find_child(p, TAG_ATTRIBUTES))
                    .and_then(|attr| find_child_in_item(&attr, TAG_CRYPTOGRAPHIC_USAGE_MASK))
                    .and_then(|item| extract_integer(&item))
                    .unwrap_or(0x0000_000C) as u32;

                let name = payload
                    .and_then(|p| find_child(p, TAG_ATTRIBUTES))
                    .and_then(|attr| find_child_in_item(&attr, TAG_NAME))
                    .and_then(|name_item| find_child_in_item(&name_item, TAG_NAME_VALUE))
                    .and_then(|item| extract_text_string(&item));

                let spec = CreateKeySpec {
                    algorithm: "AES".to_string(),
                    length: len_bits as u32,
                    usage_mask,
                    name: name.clone(),
                };

                let kmip_attrs = KmipAttributes {
                    object_type: Some("symmetric_key".to_string()),
                    state: Some("pre_active".to_string()),
                    cryptographic_usage_mask: Some(usage_mask),
                    name,
                    activation_date: None,
                    protect_stop_date: None,
                    deactivation_date: None,
                    revocation_reason: None,
                    custody_mode: Some(match self.custody_mode {
                        KeyCustodyMode::Auto => "auto".to_string(),
                        KeyCustodyMode::NativeKms => "native_kms".to_string(),
                        KeyCustodyMode::AutoEnvelope => "auto_envelope".to_string(),
                    }),
                    links: BTreeMap::new(),
                };
                let tags = TagCodec::encode(&kmip_attrs);

                let unique_id = match self.custody_mode {
                    KeyCustodyMode::NativeKms => self
                        .provider
                        .create_native_key(&spec, &tags)
                        .await
                        .map_err(|_| (REASON_GENERAL_FAILURE, "failed creating native key"))?,
                    _ => self
                        .provider
                        .create_envelope_key(&spec, None, &tags)
                        .await
                        .map_err(|_| (REASON_GENERAL_FAILURE, "failed creating envelope key"))?,
                };

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(obj_type)));
                res.push(ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(unique_id),
                ));
                Ok(res)
            }

            REGISTER_OPERATION => {
                let obj_type = payload
                    .and_then(|p| find_child(p, TAG_OBJECT_TYPE))
                    .and_then(|item| extract_enumeration(&item))
                    .unwrap_or(2);

                let raw_material = payload
                    .and_then(|p| find_child(p, TAG_KEY_BLOCK))
                    .and_then(|kb| find_child_in_item(&kb, TAG_KEY_VALUE))
                    .and_then(|kv| find_child_in_item(&kv, TAG_KEY_MATERIAL))
                    .and_then(|km| extract_byte_string(&km));

                let raw_bytes =
                    raw_material.ok_or((REASON_GENERAL_FAILURE, "missing key material"))?;
                let len_bits = (raw_bytes.len() * 8) as u32;

                let spec = CreateKeySpec {
                    algorithm: "AES".to_string(),
                    length: len_bits,
                    usage_mask: 0x0000_000C,
                    name: None,
                };

                let kmip_attrs = KmipAttributes {
                    object_type: Some("symmetric_key".to_string()),
                    state: Some("pre_active".to_string()),
                    cryptographic_usage_mask: Some(0x0000_000C),
                    name: None,
                    activation_date: None,
                    protect_stop_date: None,
                    deactivation_date: None,
                    revocation_reason: None,
                    custody_mode: Some("auto_envelope".to_string()),
                    links: BTreeMap::new(),
                };
                let tags = TagCodec::encode(&kmip_attrs);

                let unique_id = self
                    .provider
                    .create_envelope_key(&spec, Some(zeroize::Zeroizing::new(raw_bytes)), &tags)
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "failed registering key"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(obj_type)));
                res.push(ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(unique_id),
                ));
                Ok(res)
            }

            GET_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                let (material, _tags) = self
                    .provider
                    .get_key_material(&id)
                    .await
                    .map_err(|_| (REASON_ITEM_NOT_FOUND, "item not found"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(0x02))); // SymmetricKey
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));

                match material {
                    KeyMaterialPayload::Symmetric(raw_key) => {
                        let mut key_val_struct = Structure::new();
                        let _ = key_val_struct.try_push(ttlv_item(
                            TAG_KEY_MATERIAL,
                            Value::byte_string(raw_key.as_slice().to_vec()),
                        ));

                        let mut key_block_struct = Structure::new();
                        let _ = key_block_struct.try_push(ttlv_item(
                            TAG_KEY_FORMAT_TYPE,
                            Value::enumeration(0x01), // Raw
                        ));
                        let _ = key_block_struct.try_push(ttlv_item(
                            TAG_KEY_COMPRESSION_TYPE,
                            Value::enumeration(0x01), // None
                        ));
                        let _ = key_block_struct
                            .try_push(ttlv_item(TAG_KEY_VALUE, Value::structure(key_val_struct)));
                        let _ = key_block_struct.try_push(ttlv_item(
                            TAG_CRYPTOGRAPHIC_ALGORITHM,
                            Value::enumeration(0x03), // AES
                        ));
                        let _ = key_block_struct.try_push(ttlv_item(
                            TAG_CRYPTOGRAPHIC_LENGTH,
                            Value::integer((raw_key.len() * 8) as i32),
                        ));

                        res.push(ttlv_item(TAG_KEY_BLOCK, Value::structure(key_block_struct)));
                    }
                    KeyMaterialPayload::PublicKeyDer(der) => {
                        let mut key_val_struct = Structure::new();
                        let _ = key_val_struct
                            .try_push(ttlv_item(TAG_KEY_MATERIAL, Value::byte_string(der)));

                        let mut key_block_struct = Structure::new();
                        let _ = key_block_struct.try_push(ttlv_item(
                            TAG_KEY_FORMAT_TYPE,
                            Value::enumeration(0x02), // X_509
                        ));
                        let _ = key_block_struct
                            .try_push(ttlv_item(TAG_KEY_VALUE, Value::structure(key_val_struct)));
                        res.push(ttlv_item(TAG_KEY_BLOCK, Value::structure(key_block_struct)));
                    }
                    KeyMaterialPayload::PrivateKeyDer(_) => {
                        return Err((
                            REASON_OPERATION_NOT_SUPPORTED,
                            "private key export restricted",
                        ));
                    }
                }

                Ok(res)
            }

            GET_ATTRIBUTES_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                let meta = if let Some(cached) = self.cache.get(&id) {
                    cached
                } else {
                    let fetched = self
                        .provider
                        .get_tags_and_metadata(&id)
                        .await
                        .map_err(|_| (REASON_ITEM_NOT_FOUND, "item not found"))?;
                    self.cache.insert(id.clone(), fetched.clone());
                    fetched
                };

                let attrs = TagCodec::decode(&meta.tags);
                let mut res = Vec::new();
                res.push(ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id.clone()),
                ));

                let mut attr_struct = Structure::new();
                let _ =
                    attr_struct.try_push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                let _ = attr_struct.try_push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(0x02)));

                let state_enum = match attrs.state.as_deref() {
                    Some("active") => 0x02,
                    Some("deactivated") => 0x03,
                    Some("compromised") => 0x04,
                    Some("destroyed") => 0x05,
                    _ => 0x01, // PreActive
                };
                let _ =
                    attr_struct.try_push(ttlv_item(TAG_STATE_TAG, Value::enumeration(state_enum)));

                if let Some(mask) = attrs.cryptographic_usage_mask {
                    let _ = attr_struct.try_push(ttlv_item(
                        TAG_CRYPTOGRAPHIC_USAGE_MASK,
                        Value::integer(mask as i32),
                    ));
                }
                if let Some(ref name) = attrs.name {
                    let mut name_struct = Structure::new();
                    let _ = name_struct
                        .try_push(ttlv_item(TAG_NAME_VALUE, Value::text_string(name.clone())));
                    let _ =
                        name_struct.try_push(ttlv_item(TAG_NAME_TYPE, Value::enumeration(0x01)));
                    let _ =
                        attr_struct.try_push(ttlv_item(TAG_NAME, Value::structure(name_struct)));
                }
                if let Some(act) = attrs.activation_date {
                    let _ =
                        attr_struct.try_push(ttlv_item(TAG_ACTIVATION_DATE, Value::date_time(act)));
                }
                if let Some(stop) = attrs.protect_stop_date {
                    let _ = attr_struct
                        .try_push(ttlv_item(TAG_PROTECT_STOP_DATE, Value::date_time(stop)));
                }

                res.push(ttlv_item(TAG_ATTRIBUTES, Value::structure(attr_struct)));
                Ok(res)
            }

            LOCATE_OPERATION => {
                let mut filter = CloudTagFilter::default();
                if let Some(p) = payload {
                    if let Some(name_item) = find_child(p, TAG_NAME) {
                        if let Some(name_val) = find_child_in_item(&name_item, TAG_NAME_VALUE)
                            .and_then(|i| extract_text_string(&i))
                        {
                            let encoded = crate::tag_codec::base32hex_encode(name_val.as_bytes());
                            filter
                                .exact_tags
                                .insert(format!("{}0", crate::tag_codec::PREFIX_NAME), encoded);
                        }
                    }
                }

                let ids = self
                    .provider
                    .locate_by_tags(&filter, None)
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "locate failed"))?;

                let mut res = Vec::new();
                for id in ids {
                    res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                }
                Ok(res)
            }

            ACTIVATE_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                self.provider
                    .set_native_enabled_state(&id, true)
                    .await
                    .map_err(|_| (REASON_ITEM_NOT_FOUND, "not found"))?;

                let mut upsert = BTreeMap::new();
                upsert.insert(TAG_STATE.to_string(), "active".to_string());
                upsert.insert(TAG_ACT_DATE.to_string(), now_unix_seconds().to_string());
                self.provider
                    .update_tags(&id, &CloudTagMap(upsert), &[])
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "tag update failed"))?;

                self.cache.invalidate(&id);

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                Ok(res)
            }

            REVOKE_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                self.provider
                    .set_native_enabled_state(&id, false)
                    .await
                    .map_err(|_| (REASON_ITEM_NOT_FOUND, "not found"))?;

                let mut upsert = BTreeMap::new();
                upsert.insert(TAG_STATE.to_string(), "compromised".to_string());
                self.provider
                    .update_tags(&id, &CloudTagMap(upsert), &[])
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "tag update failed"))?;

                self.cache.invalidate(&id);

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                Ok(res)
            }

            DEACTIVATE_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                self.provider
                    .set_native_enabled_state(&id, false)
                    .await
                    .map_err(|_| (REASON_ITEM_NOT_FOUND, "not found"))?;

                let mut upsert = BTreeMap::new();
                upsert.insert(TAG_STATE.to_string(), "deactivated".to_string());
                self.provider
                    .update_tags(&id, &CloudTagMap(upsert), &[])
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "tag update failed"))?;

                self.cache.invalidate(&id);

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                Ok(res)
            }

            DESTROY_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                self.provider
                    .schedule_destruction(&id)
                    .await
                    .map_err(|_| (REASON_ITEM_NOT_FOUND, "not found"))?;
                self.cache.invalidate(&id);

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                Ok(res)
            }

            ENCRYPT_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .ok_or((REASON_GENERAL_FAILURE, "missing data"))?;

                let ciphertext = self
                    .provider
                    .encrypt(&id, &CryptoParams::default(), &data)
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "encrypt failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(TAG_DATA, Value::byte_string(ciphertext)));
                Ok(res)
            }

            DECRYPT_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .ok_or((REASON_GENERAL_FAILURE, "missing data"))?;

                let plaintext = self
                    .provider
                    .decrypt(&id, &CryptoParams::default(), &data)
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "decrypt failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(
                    TAG_DATA,
                    Value::byte_string(plaintext.as_slice().to_vec()),
                ));
                Ok(res)
            }

            SIGN_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .ok_or((REASON_GENERAL_FAILURE, "missing data"))?;

                let sig = self
                    .provider
                    .sign(&id, &CryptoParams::default(), &data)
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "sign failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(TAG_SIGNATURE_DATA, Value::byte_string(sig)));
                Ok(res)
            }

            SIGNATURE_VERIFY_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .ok_or((REASON_GENERAL_FAILURE, "missing data"))?;

                let sig = payload
                    .and_then(|p| find_child(p, TAG_SIGNATURE_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .ok_or((REASON_GENERAL_FAILURE, "missing signature"))?;

                let valid = self
                    .provider
                    .verify(&id, &CryptoParams::default(), &data, &sig)
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "verify failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                let validity_val = if valid { 0x01 } else { 0x02 }; // Valid / Invalid
                res.push(ttlv_item(
                    TAG_VALIDITY_INDICATOR,
                    Value::enumeration(validity_val),
                ));
                Ok(res)
            }

            RNG_RETRIEVE_OPERATION => {
                let len = payload
                    .and_then(|p| find_child(p, TAG_DATA_LENGTH))
                    .and_then(|item| extract_integer(&item))
                    .unwrap_or(32) as usize;

                let bytes = self
                    .provider
                    .random_bytes(len)
                    .await
                    .map_err(|_| (REASON_GENERAL_FAILURE, "rng failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_DATA, Value::byte_string(bytes.to_vec())));
                Ok(res)
            }

            PING_OPERATION => Ok(Vec::new()),

            _ => Err((REASON_OPERATION_NOT_SUPPORTED, "operation not supported")),
        }
    }
}

// TTLV extraction helpers
fn extract_request_parts(request: &Item) -> Result<(u32, u32, Vec<Item>), String> {
    if request.tag().raw() != TAG_REQUEST_MESSAGE {
        return Err("expected RequestMessage root".to_string());
    }

    let mut major = 2;
    let mut minor = 1;
    let mut batch_items = Vec::new();

    request.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                if child.tag().raw() == TAG_REQUEST_HEADER {
                    if let Some((maj, min)) = extract_header_version(child) {
                        major = maj;
                        minor = min;
                    }
                } else if child.tag().raw() == TAG_BATCH_ITEM {
                    if let Ok(cloned) = child.try_clone() {
                        batch_items.push(cloned);
                    }
                }
            }
        }
    });

    Ok((major, minor, batch_items))
}

fn extract_header_version(header_item: &Item) -> Option<(u32, u32)> {
    let mut version = None;
    header_item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                if child.tag().raw() == TAG_PROTOCOL_VERSION {
                    version = extract_version_tuple(child);
                }
            }
        }
    });
    version
}

fn extract_version_tuple(version_item: &Item) -> Option<(u32, u32)> {
    let mut major = None;
    let mut minor = None;

    version_item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                if child.tag().raw() == TAG_PROTOCOL_VERSION_MAJOR {
                    major = extract_integer(child).map(|i| u32::try_from(i).unwrap_or(0));
                } else if child.tag().raw() == TAG_PROTOCOL_VERSION_MINOR {
                    minor = extract_integer(child).map(|i| u32::try_from(i).unwrap_or(0));
                }
            }
        }
    });

    Some((major?, minor?))
}

fn extract_batch_item_parts(
    item: &Item,
) -> Result<(u32, Option<Vec<u8>>, Option<Structure>), String> {
    let mut op = None;
    let mut unique_id = None;
    let mut payload = None;

    item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            for child in s.children() {
                match child.tag().raw() {
                    TAG_OPERATION => op = extract_enumeration(child),
                    TAG_UNIQUE_BATCH_ITEM_ID => unique_id = extract_byte_string(child),
                    TAG_REQUEST_PAYLOAD => {
                        child.with_value(|pv| {
                            if let ValueView::Structure(ps) = pv {
                                let mut cloned_s = Structure::new();
                                for c in ps.children() {
                                    if let Ok(cloned_c) = c.try_clone() {
                                        let _ = cloned_s.try_push(cloned_c);
                                    }
                                }
                                payload = Some(cloned_s);
                            }
                        });
                    }
                    _ => {}
                }
            }
        }
    });

    let op = op.ok_or_else(|| "missing Operation in batch item".to_string())?;
    Ok((op, unique_id, payload))
}

fn build_batch_response_item(
    op: u32,
    unique_batch_id: Option<&[u8]>,
    result: Result<Vec<Item>, (u32, &'static str)>,
) -> Item {
    let mut structure = Structure::new();
    let _ = structure.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(op)));

    if let Some(bid) = unique_batch_id {
        let _ = structure.try_push(ttlv_item(
            TAG_UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(bid.to_vec()),
        ));
    }

    match result {
        Ok(payload_items) => {
            let _ = structure.try_push(ttlv_item(
                TAG_RESULT_STATUS,
                Value::enumeration(STATUS_SUCCESS),
            ));
            let mut payload_struct = Structure::new();
            for item in payload_items {
                let _ = payload_struct.try_push(item);
            }
            let _ = structure.try_push(ttlv_item(
                TAG_RESPONSE_PAYLOAD,
                Value::structure(payload_struct),
            ));
        }
        Err((reason, message)) => {
            let _ = structure.try_push(ttlv_item(
                TAG_RESULT_STATUS,
                Value::enumeration(STATUS_OPERATION_FAILED),
            ));
            let _ = structure.try_push(ttlv_item(TAG_RESULT_REASON, Value::enumeration(reason)));
            let _ = structure.try_push(ttlv_item(
                TAG_RESULT_MESSAGE,
                Value::text_string(message.to_string()),
            ));
        }
    }

    ttlv_item(TAG_BATCH_ITEM, Value::structure(structure))
}

fn build_response_message(major: u32, minor: u32, items: Vec<Item>) -> Result<Item, String> {
    let mut header = Structure::new();

    let mut version = Structure::new();
    let _ = version.try_push(ttlv_item(
        TAG_PROTOCOL_VERSION_MAJOR,
        Value::integer(major as i32),
    ));
    let _ = version.try_push(ttlv_item(
        TAG_PROTOCOL_VERSION_MINOR,
        Value::integer(minor as i32),
    ));
    let _ = header.try_push(ttlv_item(TAG_PROTOCOL_VERSION, Value::structure(version)));

    let _ = header.try_push(ttlv_item(
        TAG_TIME_STAMP,
        Value::date_time(now_unix_seconds()),
    ));
    let _ = header.try_push(ttlv_item(
        TAG_BATCH_COUNT,
        Value::integer(items.len() as i32),
    ));

    let mut message = Structure::new();
    let _ = message.try_push(ttlv_item(TAG_RESPONSE_HEADER, Value::structure(header)));

    for item in items {
        let _ = message.try_push(item);
    }

    Ok(ttlv_item(TAG_RESPONSE_MESSAGE, Value::structure(message)))
}

fn ttlv_item(tag: u32, val: Value) -> Item {
    let raw = RawTag::new(tag).unwrap().try_checked().unwrap();
    Item::new(raw, val).unwrap()
}

fn find_child(structure: &Structure, tag: u32) -> Option<Item> {
    structure
        .view()
        .children()
        .iter()
        .find(|c| c.tag().raw() == tag)
        .and_then(|item| item.try_clone().ok())
}

fn find_child_in_item(item: &Item, tag: u32) -> Option<Item> {
    let mut res = None;
    item.with_value(|view| {
        if let ValueView::Structure(s) = view {
            res = s
                .children()
                .iter()
                .find(|c| c.tag().raw() == tag)
                .and_then(|i| i.try_clone().ok());
        }
    });
    res
}

fn extract_enumeration(item: &Item) -> Option<u32> {
    let mut val = None;
    item.with_value(|v| {
        if let ValueView::Enumeration(e) = v {
            val = Some(*e);
        }
    });
    val
}

fn extract_integer(item: &Item) -> Option<i32> {
    let mut val = None;
    item.with_value(|v| {
        if let ValueView::Integer(i) = v {
            val = Some(*i);
        }
    });
    val
}

fn extract_text_string(item: &Item) -> Option<String> {
    let mut val = None;
    item.with_value(|v| {
        if let ValueView::TextString(s) = v {
            val = Some(s.to_string());
        }
    });
    val
}

fn extract_byte_string(item: &Item) -> Option<Vec<u8>> {
    let mut val = None;
    item.with_value(|v| {
        if let ValueView::ByteString(b) = v {
            val = Some(b.to_vec());
        }
    });
    val
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
