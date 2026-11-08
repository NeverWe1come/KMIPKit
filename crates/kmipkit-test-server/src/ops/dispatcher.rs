//! Central KMIP operation dispatcher for kmipkit-test-server.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

use super::crypto;
use crate::store::{JsonStore, LocateFilter, StoredAttribute, StoredObject};
use crate::v1_compat::{self, is_version_supported};

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
pub const HASH_OPERATION: u32 = 0x0000_0027;
pub const PING_OPERATION: u32 = 0x0000_003B;
pub const SET_ENDPOINT_ROLE_OPERATION: u32 = 0x0000_0032;
pub const DEACTIVATE_OPERATION: u32 = 0x0000_0040;
pub const ENCAPSULATE_OPERATION: u32 = 0x0000_0041;
pub const DECAPSULATE_OPERATION: u32 = 0x0000_0042;

// Tag constants
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
pub const TAG_ENDPOINT_ROLE: u32 = 0x0042_0151;
pub const TAG_DATA: u32 = 0x0042_00C2;
pub const TAG_DATA_LENGTH: u32 = 0x0042_00CD;
pub const TAG_KEY_BLOCK: u32 = 0x0042_0040;
pub const TAG_KEY_FORMAT_TYPE: u32 = 0x0042_0042;
pub const TAG_KEY_VALUE: u32 = 0x0042_0045;
pub const TAG_KEY_MATERIAL: u32 = 0x0042_0043;
pub const TAG_CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
pub const TAG_CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
pub const TAG_ATTRIBUTES: u32 = 0x0042_0125;
pub const TAG_ATTRIBUTE_NAME: u32 = 0x0042_000A;
pub const TAG_IV_COUNTER_NONCE: u32 = 0x0042_003A;
pub const TAG_VALIDITY_INDICATOR: u32 = 0x0042_009A;
pub const TAG_MAC_DATA: u32 = 0x0042_004F;
pub const TAG_SIGNATURE_DATA: u32 = 0x0042_008B;
pub const TAG_PRIVATE_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_0066;
pub const TAG_PUBLIC_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_006F;

// Result status & reasons
pub const STATUS_SUCCESS: u32 = 0;
pub const STATUS_OPERATION_FAILED: u32 = 1;

pub const REASON_ITEM_NOT_FOUND: u32 = 0x0000_0001;
pub const REASON_OPERATION_NOT_SUPPORTED: u32 = 0x0000_0005;
pub const REASON_GENERAL_FAILURE: u32 = 0x0000_0100;

#[derive(Clone)]
pub struct Dispatcher {
    store: Arc<JsonStore>,
    fixed_version: Option<(u32, u32)>,
}

impl Dispatcher {
    pub fn new(store: Arc<JsonStore>) -> Self {
        Self {
            store,
            fixed_version: None,
        }
    }

    pub fn with_fixed_version(store: Arc<JsonStore>, version: Option<(u32, u32)>) -> Self {
        Self {
            store,
            fixed_version: version,
        }
    }

    pub fn dispatch(&self, request: &Item) -> Result<Item, String> {
        let (req_major, req_minor, items) = extract_request_parts(request)?;

        let (major, minor) = self.fixed_version.unwrap_or((req_major, req_minor));

        if !is_version_supported(major, minor) {
            return Ok(build_error_response(
                major,
                minor,
                REASON_OPERATION_NOT_SUPPORTED,
                "Unsupported protocol version",
            ));
        }

        let mut response_items = Vec::new();
        let mut id_placeholder = None;

        for item in items {
            let (op, unique_batch_id, payload) = extract_batch_item_parts(&item)?;
            let item_res =
                self.execute_operation(major, minor, op, payload.as_ref(), &mut id_placeholder);

            let res_item = build_batch_response_item(op, unique_batch_id.as_deref(), item_res);
            response_items.push(res_item);
        }

        build_response_message(major, minor, response_items)
    }

    fn execute_operation(
        &self,
        major: u32,
        minor: u32,
        op: u32,
        payload: Option<&Structure>,
        id_placeholder: &mut Option<String>,
    ) -> Result<Vec<Item>, (u32, &'static str)> {
        if (major, minor) == (2, 0) && (0x0000_0036..=0x0000_003B).contains(&op) {
            return Err((
                REASON_OPERATION_NOT_SUPPORTED,
                "Operation not supported in KMIP 2.0",
            ));
        }

        if major < 3 && (0x0000_003C..=0x0000_0042).contains(&op) {
            return Err((
                REASON_OPERATION_NOT_SUPPORTED,
                "Operation not supported in KMIP versions prior to 3.0",
            ));
        }

        let now_secs = now_unix_seconds();

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

                let versions_to_return = if offered_versions.is_empty() {
                    v1_compat::ALL_VERSIONS.to_vec()
                } else {
                    offered_versions
                        .into_iter()
                        .filter(|(maj, min)| is_version_supported(*maj, *min))
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

                let id = generate_uuid();
                let key_bytes =
                    crypto::generate_random_bytes(usize::try_from(len_bits / 8).unwrap_or(32))
                        .map_err(|_| (REASON_GENERAL_FAILURE, "crypto failure"))?;

                let mut stored =
                    StoredObject::new(id.clone(), "SymmetricKey", "PreActive", now_secs);
                stored.key_material_base64 =
                    Some(crate::codec::datetime::octets_to_hex(&key_bytes));
                stored.cryptographic_length = Some(u32::try_from(len_bits).unwrap_or(256));
                stored.cryptographic_algorithm = Some("AES".to_string());

                self.store
                    .save(&stored)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;
                *id_placeholder = Some(id.clone());

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(obj_type)));
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                Ok(res)
            }

            CREATE_KEY_PAIR_OPERATION => {
                let priv_id = generate_uuid();
                let pub_id = generate_uuid();

                let priv_key_bytes = crypto::generate_random_bytes(32)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "crypto failure"))?;
                let pub_key_bytes = crypto::generate_random_bytes(32)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "crypto failure"))?;

                let mut priv_obj =
                    StoredObject::new(priv_id.clone(), "PrivateKey", "PreActive", now_secs);
                priv_obj.key_material_base64 =
                    Some(crate::codec::datetime::octets_to_hex(&priv_key_bytes));
                priv_obj.cryptographic_algorithm = Some("RSA".to_string());
                self.store
                    .save(&priv_obj)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;

                let mut pub_obj =
                    StoredObject::new(pub_id.clone(), "PublicKey", "PreActive", now_secs);
                pub_obj.key_material_base64 =
                    Some(crate::codec::datetime::octets_to_hex(&pub_key_bytes));
                pub_obj.cryptographic_algorithm = Some("RSA".to_string());
                self.store
                    .save(&pub_obj)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;

                *id_placeholder = Some(priv_id.clone());

                let mut res = Vec::new();
                res.push(ttlv_item(
                    TAG_PRIVATE_KEY_UNIQUE_IDENTIFIER,
                    Value::text_string(priv_id),
                ));
                res.push(ttlv_item(
                    TAG_PUBLIC_KEY_UNIQUE_IDENTIFIER,
                    Value::text_string(pub_id),
                ));
                Ok(res)
            }

            REGISTER_OPERATION => {
                let id = generate_uuid();
                let obj_type = payload
                    .and_then(|p| find_child(p, TAG_OBJECT_TYPE))
                    .and_then(|item| extract_enumeration(&item))
                    .unwrap_or(2);

                let stored = StoredObject::new(
                    id.clone(),
                    format!("Type_{obj_type}"),
                    "PreActive",
                    now_secs,
                );
                self.store
                    .save(&stored)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;
                *id_placeholder = Some(id.clone());

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id),
                )])
            }

            ACTIVATE_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let mut obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                obj.state = "Active".to_string();
                obj.activation_date = Some(now_secs);
                self.store
                    .save(&obj)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;
                *id_placeholder = Some(id.clone());

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id),
                )])
            }

            REVOKE_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let mut obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                obj.state = "Deactivated".to_string();
                obj.deactivation_date = Some(now_secs);
                self.store
                    .save(&obj)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id),
                )])
            }

            DESTROY_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let mut obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                obj.state = "Destroyed".to_string();
                obj.destroy_date = Some(now_secs);
                obj.key_material_base64 = None;
                self.store
                    .save(&obj)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id),
                )])
            }

            GET_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                if obj.state == "Destroyed" {
                    return Err((REASON_ITEM_NOT_FOUND, "object has been destroyed"));
                }

                let key_bytes = obj
                    .key_material_base64
                    .as_deref()
                    .and_then(|hex| crate::codec::datetime::hex_to_octets(hex).ok())
                    .unwrap_or_else(|| vec![0u8; 32]);

                let mut key_val_struct = Structure::new();
                let _ = key_val_struct
                    .try_push(ttlv_item(TAG_KEY_MATERIAL, Value::byte_string(key_bytes)));

                let mut key_block_struct = Structure::new();
                let _ = key_block_struct
                    .try_push(ttlv_item(TAG_KEY_FORMAT_TYPE, Value::enumeration(1))); // Raw
                let _ = key_block_struct
                    .try_push(ttlv_item(TAG_KEY_VALUE, Value::structure(key_val_struct)));
                let _ = key_block_struct.try_push(ttlv_item(
                    TAG_CRYPTOGRAPHIC_ALGORITHM,
                    Value::enumeration(3),
                )); // AES
                let _ = key_block_struct.try_push(ttlv_item(
                    TAG_CRYPTOGRAPHIC_LENGTH,
                    Value::integer(
                        i32::try_from(obj.cryptographic_length.unwrap_or(256)).unwrap_or(256),
                    ),
                ));

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(2))); // SymmetricKey
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(TAG_KEY_BLOCK, Value::structure(key_block_struct)));
                Ok(res)
            }

            LOCATE_OPERATION => {
                let filter = LocateFilter::default();
                let ids = self.store.locate(&filter);
                let mut res = Vec::new();
                for id in ids {
                    res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                }
                Ok(res)
            }

            GET_ATTRIBUTES_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;

                let mut attr_struct = Structure::new();
                for (name, attrs) in &obj.attributes {
                    if let Some(tag_num) = crate::codec::names::tag_value(name) {
                        if let Ok(raw_tag) = RawTag::new(tag_num) {
                            if let Ok(checked) = raw_tag.try_checked() {
                                for a in attrs {
                                    let s_val = a.value.as_str().unwrap_or("");
                                    let _ = attr_struct.try_push(
                                        Item::new(checked, Value::text_string(s_val.to_string()))
                                            .unwrap(),
                                    );
                                }
                            }
                        }
                    }
                }

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                if major == 1 {
                    for child in attr_struct.view().children() {
                        if let Some(v1_attr) = v1_compat::convert_v2_to_v1_attribute(child) {
                            res.push(v1_attr);
                        }
                    }
                } else {
                    res.push(ttlv_item(TAG_ATTRIBUTES, Value::structure(attr_struct)));
                }
                Ok(res)
            }

            GET_ATTRIBUTE_LIST_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let mut res = vec![ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id))];
                for key in obj.attributes.keys() {
                    res.push(ttlv_item(
                        TAG_ATTRIBUTE_NAME,
                        Value::text_string(key.clone()),
                    ));
                }
                Ok(res)
            }

            ADD_ATTRIBUTE_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let mut obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let list = obj
                    .attributes
                    .entry("CustomAttribute".to_string())
                    .or_default();
                list.push(StoredAttribute {
                    name: "CustomAttribute".to_string(),
                    index: 0,
                    value: serde_json::Value::String("Value".to_string()),
                });
                self.store
                    .save(&obj)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id),
                )])
            }

            DELETE_ATTRIBUTE_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let mut obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                obj.attributes.remove("CustomAttribute");
                self.store
                    .save(&obj)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id),
                )])
            }

            ENCRYPT_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let key_bytes = obj
                    .key_material_base64
                    .as_deref()
                    .and_then(|hex| crate::codec::datetime::hex_to_octets(hex).ok())
                    .unwrap_or_else(|| vec![0u8; 32]);

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let iv = payload
                    .and_then(|p| find_child(p, TAG_IV_COUNTER_NONCE))
                    .and_then(|item| extract_byte_string(&item));

                let (ciphertext, out_iv) = crypto::encrypt_aes(&key_bytes, iv.as_deref(), &data)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "encryption failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(TAG_DATA, Value::byte_string(ciphertext)));
                if let Some(iv_bytes) = out_iv {
                    res.push(ttlv_item(
                        TAG_IV_COUNTER_NONCE,
                        Value::byte_string(iv_bytes),
                    ));
                }
                Ok(res)
            }

            DECRYPT_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let key_bytes = obj
                    .key_material_base64
                    .as_deref()
                    .and_then(|hex| crate::codec::datetime::hex_to_octets(hex).ok())
                    .unwrap_or_else(|| vec![0u8; 32]);

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let iv = payload
                    .and_then(|p| find_child(p, TAG_IV_COUNTER_NONCE))
                    .and_then(|item| extract_byte_string(&item));

                let plaintext = crypto::decrypt_aes(&key_bytes, iv.as_deref(), &data)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "decryption failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(TAG_DATA, Value::byte_string(plaintext)));
                Ok(res)
            }

            SIGN_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let key_bytes = obj
                    .key_material_base64
                    .as_deref()
                    .and_then(|hex| crate::codec::datetime::hex_to_octets(hex).ok())
                    .unwrap_or_else(|| vec![0u8; 32]);

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let sig = crypto::hmac(0x0000_0005, &key_bytes, &data)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "signing failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(TAG_SIGNATURE_DATA, Value::byte_string(sig)));
                Ok(res)
            }

            SIGNATURE_VERIFY_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let key_bytes = obj
                    .key_material_base64
                    .as_deref()
                    .and_then(|hex| crate::codec::datetime::hex_to_octets(hex).ok())
                    .unwrap_or_else(|| vec![0u8; 32]);

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let sig = payload
                    .and_then(|p| find_child(p, TAG_SIGNATURE_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let valid = crypto::hmac_verify_tag(0x0000_0005, &key_bytes, &data, &sig);
                let validity = if valid { 0 } else { 1 };

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(
                    TAG_VALIDITY_INDICATOR,
                    Value::enumeration(validity),
                ));
                Ok(res)
            }

            MAC_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let key_bytes = obj
                    .key_material_base64
                    .as_deref()
                    .and_then(|hex| crate::codec::datetime::hex_to_octets(hex).ok())
                    .unwrap_or_else(|| vec![0u8; 32]);

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let mac = crypto::hmac(0x0000_0005, &key_bytes, &data)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "mac failed"))?;

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(TAG_MAC_DATA, Value::byte_string(mac)));
                Ok(res)
            }

            MAC_VERIFY_OPERATION => {
                let id = resolve_id(payload, id_placeholder)?;
                let obj = self
                    .store
                    .get(&id)
                    .ok_or((REASON_ITEM_NOT_FOUND, "item not found"))?;
                let key_bytes = obj
                    .key_material_base64
                    .as_deref()
                    .and_then(|hex| crate::codec::datetime::hex_to_octets(hex).ok())
                    .unwrap_or_else(|| vec![0u8; 32]);

                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let mac = payload
                    .and_then(|p| find_child(p, TAG_MAC_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let valid = crypto::hmac_verify_tag(0x0000_0005, &key_bytes, &data, &mac);
                let validity = if valid { 0 } else { 1 };

                let mut res = Vec::new();
                res.push(ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(id)));
                res.push(ttlv_item(
                    TAG_VALIDITY_INDICATOR,
                    Value::enumeration(validity),
                ));
                Ok(res)
            }

            HASH_OPERATION => {
                let data = payload
                    .and_then(|p| find_child(p, TAG_DATA))
                    .and_then(|item| extract_byte_string(&item))
                    .unwrap_or_default();

                let digest = crypto::hash(0x0000_0005, &data)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "hash failed"))?;

                Ok(vec![ttlv_item(TAG_DATA, Value::byte_string(digest))])
            }

            RNG_RETRIEVE_OPERATION => {
                let len = payload
                    .and_then(|p| find_child(p, TAG_DATA_LENGTH))
                    .and_then(|item| extract_integer(&item))
                    .unwrap_or(32);

                let bytes = crypto::generate_random_bytes(usize::try_from(len).unwrap_or(32))
                    .map_err(|_| (REASON_GENERAL_FAILURE, "rng failed"))?;

                Ok(vec![ttlv_item(TAG_DATA, Value::byte_string(bytes))])
            }

            SET_ENDPOINT_ROLE_OPERATION => {
                let role = payload
                    .and_then(|p| find_child(p, TAG_ENDPOINT_ROLE))
                    .and_then(|item| extract_enumeration(&item))
                    .unwrap_or(2); // 2 = Server
                Ok(vec![ttlv_item(TAG_ENDPOINT_ROLE, Value::enumeration(role))])
            }

            DEACTIVATE_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .or_else(|| id_placeholder.clone())
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                if self.store.get(&id).is_none() {
                    return Err((REASON_ITEM_NOT_FOUND, "object not found"));
                }
                *id_placeholder = Some(id.clone());

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(id),
                )])
            }

            ENCAPSULATE_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .or_else(|| id_placeholder.clone())
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                if self.store.get(&id).is_none() {
                    return Err((REASON_ITEM_NOT_FOUND, "object not found"));
                }
                *id_placeholder = Some(id.clone());

                let ciphertext = crypto::generate_random_bytes(32)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "crypto failure"))?;
                let shared_secret = crypto::generate_random_bytes(32)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "crypto failure"))?;
                let secret_id = generate_uuid();
                let mut secret_object = StoredObject::new(
                    secret_id.clone(),
                    "SecretData",
                    "PreActive",
                    now_secs,
                );
                secret_object.key_material_base64 =
                    Some(crate::codec::datetime::octets_to_hex(&shared_secret));
                self.store
                    .save(&secret_object)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;
                *id_placeholder = Some(secret_id.clone());

                Ok(vec![
                    ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string(secret_id)),
                    ttlv_item(TAG_DATA, Value::byte_string(ciphertext)),
                ])
            }

            DECAPSULATE_OPERATION => {
                let id = payload
                    .and_then(|p| find_child(p, TAG_UNIQUE_IDENTIFIER))
                    .and_then(|item| extract_text_string(&item))
                    .or_else(|| id_placeholder.clone())
                    .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))?;

                if self.store.get(&id).is_none() {
                    return Err((REASON_ITEM_NOT_FOUND, "object not found"));
                }
                *id_placeholder = Some(id.clone());

                let shared_secret = crypto::generate_random_bytes(32)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "crypto failure"))?;
                let secret_id = generate_uuid();
                let mut secret_object = StoredObject::new(
                    secret_id.clone(),
                    "SecretData",
                    "PreActive",
                    now_secs,
                );
                secret_object.key_material_base64 =
                    Some(crate::codec::datetime::octets_to_hex(&shared_secret));
                self.store
                    .save(&secret_object)
                    .map_err(|_| (REASON_GENERAL_FAILURE, "storage write failure"))?;
                *id_placeholder = Some(secret_id.clone());

                Ok(vec![ttlv_item(
                    TAG_UNIQUE_IDENTIFIER,
                    Value::text_string(secret_id),
                )])
            }

            QUERY_OPERATION | PING_OPERATION => Ok(Vec::new()),

            _ => Err((REASON_OPERATION_NOT_SUPPORTED, "Operation not supported")),
        }
    }
}

// Helpers

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(0))
        .unwrap_or(0)
}

fn ttlv_item(tag: u32, val: Value) -> Item {
    let raw = RawTag::new(tag).unwrap().try_checked().unwrap();
    Item::new(raw, val).unwrap()
}

fn generate_uuid() -> String {
    let b = crypto::generate_random_bytes(16).unwrap_or_else(|_| vec![0; 16]);
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],
        b[1],
        b[2],
        b[3],
        b[4],
        b[5],
        b[6],
        b[7],
        b[8],
        b[9],
        b[10],
        b[11],
        b[12],
        b[13],
        b[14],
        b[15]
    )
}

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

fn resolve_id(
    payload: Option<&Structure>,
    id_placeholder: &mut Option<String>,
) -> Result<String, (u32, &'static str)> {
    if let Some(p) = payload {
        if let Some(child) = find_child(p, TAG_UNIQUE_IDENTIFIER) {
            if let Some(s) = extract_text_string(&child) {
                return Ok(s);
            }
        }
    }
    id_placeholder
        .clone()
        .ok_or((REASON_ITEM_NOT_FOUND, "missing unique identifier"))
}

fn find_child(s: &Structure, tag: u32) -> Option<Item> {
    s.view()
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

fn build_batch_response_item(
    op: u32,
    unique_batch_id: Option<&[u8]>,
    res: Result<Vec<Item>, (u32, &'static str)>,
) -> Item {
    let mut batch_struct = Structure::new();
    let _ = batch_struct.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(op)));

    if let Some(id_bytes) = unique_batch_id {
        let _ = batch_struct.try_push(ttlv_item(
            TAG_UNIQUE_BATCH_ITEM_ID,
            Value::byte_string(id_bytes.to_vec()),
        ));
    }

    match res {
        Ok(payload_items) => {
            let _ = batch_struct.try_push(ttlv_item(
                TAG_RESULT_STATUS,
                Value::enumeration(STATUS_SUCCESS),
            ));
            let mut payload_struct = Structure::new();
            for item in payload_items {
                let _ = payload_struct.try_push(item);
            }
            let _ = batch_struct.try_push(ttlv_item(
                TAG_RESPONSE_PAYLOAD,
                Value::structure(payload_struct),
            ));
        }
        Err((reason, msg)) => {
            let _ = batch_struct.try_push(ttlv_item(
                TAG_RESULT_STATUS,
                Value::enumeration(STATUS_OPERATION_FAILED),
            ));
            let _ = batch_struct.try_push(ttlv_item(TAG_RESULT_REASON, Value::enumeration(reason)));
            let _ = batch_struct.try_push(ttlv_item(
                TAG_RESULT_MESSAGE,
                Value::text_string(msg.to_string()),
            ));
        }
    }

    ttlv_item(TAG_BATCH_ITEM, Value::structure(batch_struct))
}

fn build_response_message(
    major: u32,
    minor: u32,
    response_items: Vec<Item>,
) -> Result<Item, String> {
    let mut version_struct = Structure::new();
    let _ = version_struct.try_push(ttlv_item(
        TAG_PROTOCOL_VERSION_MAJOR,
        Value::integer(major as i32),
    ));
    let _ = version_struct.try_push(ttlv_item(
        TAG_PROTOCOL_VERSION_MINOR,
        Value::integer(minor as i32),
    ));

    let now_secs = now_unix_seconds();

    let mut header_struct = Structure::new();
    let _ = header_struct.try_push(ttlv_item(
        TAG_PROTOCOL_VERSION,
        Value::structure(version_struct),
    ));
    let _ = header_struct.try_push(ttlv_item(TAG_TIME_STAMP, Value::date_time(now_secs)));
    let _ = header_struct.try_push(ttlv_item(
        TAG_BATCH_COUNT,
        Value::integer(response_items.len() as i32),
    ));

    let mut response_struct = Structure::new();
    let _ = response_struct.try_push(ttlv_item(
        TAG_RESPONSE_HEADER,
        Value::structure(header_struct),
    ));
    for item in response_items {
        let _ = response_struct.try_push(item);
    }

    Ok(ttlv_item(
        TAG_RESPONSE_MESSAGE,
        Value::structure(response_struct),
    ))
}

fn build_error_response(major: u32, minor: u32, reason: u32, message: &'static str) -> Item {
    let mut batch_struct = Structure::new();
    let _ = batch_struct.try_push(ttlv_item(
        TAG_RESULT_STATUS,
        Value::enumeration(STATUS_OPERATION_FAILED),
    ));
    let _ = batch_struct.try_push(ttlv_item(TAG_RESULT_REASON, Value::enumeration(reason)));
    let _ = batch_struct.try_push(ttlv_item(
        TAG_RESULT_MESSAGE,
        Value::text_string(message.to_string()),
    ));

    let batch_item = ttlv_item(TAG_BATCH_ITEM, Value::structure(batch_struct));
    build_response_message(major, minor, vec![batch_item]).unwrap()
}
