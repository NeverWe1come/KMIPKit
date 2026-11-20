//! Open Telekom Cloud KMS / DEW, TMS, and CSMS Provider Bridge (KMIPKIT-0082).
#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::similar_names,
    clippy::collapsible_if,
    clippy::map_unwrap_or
)]

use async_trait::async_trait;
use zeroize::Zeroizing;

use crate::ProxyError;
use crate::config::OtcConfig;
use crate::providers::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};

#[cfg(feature = "otc")]
use base64::Engine;

/// Open Telekom Cloud KMS, TMS, and CSMS Provider.
#[cfg(feature = "otc")]
pub struct OtcKmsProvider {
    client: reqwest::Client,
    region: String,
    project_id: String,
    endpoint_url: String,
    envelope_master_kek_id: Option<String>,
    item_prefix: String,
    auth_header_val: Option<Zeroizing<String>>,
    master_kek: Zeroizing<[u8; 32]>,
}

#[cfg(feature = "otc")]
impl OtcKmsProvider {
    pub fn new(config: &OtcConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &OtcConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let region = config
            .region
            .clone()
            .or_else(|| std::env::var("OTC_REGION").ok())
            .unwrap_or_else(|| "eu-de".to_string());

        let project_id = config
            .project_id
            .clone()
            .or_else(|| std::env::var("OTC_PROJECT_ID").ok())
            .unwrap_or_else(|| "default-otc-project".to_string());

        let endpoint_url = config
            .endpoint_url
            .clone()
            .or_else(|| std::env::var("OTC_ENDPOINT_URL").ok())
            .unwrap_or_else(|| format!("https://kms.{region}.otc.t-systems.com"))
            .trim_end_matches('/')
            .to_string();

        let auth_header_val = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("OTC_AUTH_TOKEN").ok())
            .map(Zeroizing::new);

        let item_prefix = config
            .item_prefix
            .clone()
            .unwrap_or_else(|| "kmipkit-".to_string());

        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| {
                ProxyError::ConfigError(format!("failed to build OTC HTTP client: {e}"))
            })?;

        let kek = master_kek.unwrap_or([0x5du8; 32]);

        Ok(Self {
            client,
            region,
            project_id,
            endpoint_url,
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
            item_prefix,
            auth_header_val,
            master_kek: Zeroizing::new(kek),
        })
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref cred) = self.auth_header_val {
            req.header("X-Auth-Token", cred.as_str())
        } else {
            req
        }
    }

    fn normalize_id(&self, unique_id: &str) -> String {
        unique_id.trim_matches('/').to_string()
    }

    fn format_tms_tags(tags: &CloudTagMap) -> Vec<serde_json::Value> {
        tags.0
            .iter()
            .map(|(k, v)| {
                serde_json::json!({
                    "key": k,
                    "value": v
                })
            })
            .collect()
    }

    fn parse_tms_tags(arr: &[serde_json::Value]) -> CloudTagMap {
        let mut map = CloudTagMap::new();
        for item in arr {
            if let (Some(k), Some(v)) = (
                item.get("key").and_then(|x| x.as_str()),
                item.get("value").and_then(|x| x.as_str()),
            ) {
                map.insert(k.to_string(), v.to_string());
            }
        }
        map
    }

    pub fn region(&self) -> &str {
        &self.region
    }
}

#[cfg(feature = "otc")]
#[async_trait]
impl CloudKmsProvider for OtcKmsProvider {
    fn provider_name(&self) -> &'static str {
        "otc"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let key_spec = match (spec.algorithm.to_ascii_uppercase().as_str(), spec.length) {
            ("AES", 128) => "AES_128",
            ("AES", _) => "AES_256",
            ("RSA", 2048) => "RSA_2048",
            ("RSA", 3072) => "RSA_3072",
            ("RSA", 4096) => "RSA_4096",
            ("EC" | "ECDSA", 256) => "EC_P256",
            ("EC" | "ECDSA", 384) => "EC_P384",
            _ => "AES_256",
        };

        let key_alias = spec
            .name
            .clone()
            .unwrap_or_else(|| format!("{}{}", self.item_prefix, fastrand_id()));

        let url = format!(
            "{}/v1.0/{}/kms/create-key",
            self.endpoint_url, self.project_id
        );
        let body = serde_json::json!({
            "key_alias": key_alias,
            "key_spec": key_spec,
            "key_description": "Managed by KMIPKit Proxy"
        });

        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("OTC create-key request failed: {e}"))
        })?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(format!(
                "OTC create-key returned HTTP {}",
                resp.status()
            )));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC create-key JSON error: {e}")))?;

        let key_id = resp_json
            .get("key_info")
            .and_then(|ki| ki.get("key_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProxyError::ProviderError("OTC create-key missing key_id".to_string()))?
            .to_string();

        if !tags.is_empty() {
            let _ = self.update_tags(&key_id, tags, &[]).await;
        }

        Ok(key_id)
    }

    async fn create_envelope_key(
        &self,
        spec: &CreateKeySpec,
        raw_material: Option<Zeroizing<Vec<u8>>>,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let b64 = &base64::engine::general_purpose::STANDARD;
        let byte_len = ((spec.length.max(128)) / 8) as usize;
        let plain_key = match raw_material {
            Some(mat) => mat,
            None => crate::envelope::AutoEnvelope::generate_random_bytes(byte_len)?,
        };

        let wrapped = crate::envelope::AutoEnvelope::wrap_key(&self.master_kek[..], &plain_key)?;
        let wrapped_b64 = b64.encode(wrapped.to_bytes());

        let entry_name = spec
            .name
            .clone()
            .unwrap_or_else(|| format!("{}{}", self.item_prefix, fastrand_id()));

        let create_url = format!("{}/v1/{}/secrets", self.endpoint_url, self.project_id);
        let mut body = serde_json::json!({
            "name": entry_name,
            "description": "KMIPKit Envelope Key",
            "secret_binary": wrapped_b64
        });
        if let Some(ref kek_id) = self.envelope_master_kek_id {
            body["kms_key_id"] = serde_json::json!(kek_id);
        }

        let req = self.apply_auth(self.client.post(&create_url).json(&body));
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC CSMS create failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(format!(
                "OTC CSMS create returned HTTP {}",
                resp.status()
            )));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC CSMS JSON error: {e}")))?;

        let item_id = resp_json
            .get("secret")
            .and_then(|s| s.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProxyError::ProviderError("OTC CSMS missing item id".to_string()))?
            .to_string();

        // Create initial version
        let ver_url = format!(
            "{}/v1/{}/secrets/{}/versions",
            self.endpoint_url, self.project_id, item_id
        );
        let ver_body = serde_json::json!({
            "secret_binary": wrapped_b64
        });
        let ver_req = self.apply_auth(self.client.post(&ver_url).json(&ver_body));
        let _ = ver_req.send().await;

        if !tags.is_empty() {
            let _ = self.update_tags(&item_id, tags, &[]).await;
        }

        Ok(item_id)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;

        let ver_url = format!(
            "{}/v1/{}/secrets/{}/versions/latest",
            self.endpoint_url, self.project_id, safe_id
        );
        let req = self.apply_auth(self.client.get(&ver_url));
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC CSMS get version failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ItemNotFound(unique_id.to_string()));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC CSMS version JSON error: {e}")))?;

        let bin_b64 = resp_json
            .get("version")
            .and_then(|v| v.get("secret_binary"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;

        let blob = b64
            .decode(bin_b64)
            .map_err(|e| ProxyError::ProviderError(format!("OTC CSMS base64 decode error: {e}")))?;

        let wrapped = crate::envelope::WrappedEnvelopeKey::from_bytes(&blob)?;
        let unwrapped = crate::envelope::AutoEnvelope::unwrap_key(&self.master_kek[..], &wrapped)?;

        let meta = self
            .get_tags_and_metadata(unique_id)
            .await
            .unwrap_or_else(|_| CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags: CloudTagMap::new(),
                native_state_enabled: true,
                created_at_epoch_secs: None,
            });

        Ok((KeyMaterialPayload::Symmetric(unwrapped), meta.tags))
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let safe_id = self.normalize_id(unique_id);

        let desc_url = format!(
            "{}/v1.0/{}/kms/describe-key",
            self.endpoint_url, self.project_id
        );
        let desc_body = serde_json::json!({ "key_id": safe_id });
        let req = self.apply_auth(self.client.post(&desc_url).json(&desc_body));

        let mut enabled = true;
        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                    if let Some(state_str) = resp_json
                        .get("key_info")
                        .and_then(|ki| ki.get("key_state"))
                        .and_then(|s| s.as_str())
                    {
                        enabled = state_str == "2";
                    }
                }
            }
        }

        let tags_url = format!(
            "{}/v1.0/{}/kms/{}/tags",
            self.endpoint_url, self.project_id, safe_id
        );
        let tags_req = self.apply_auth(self.client.get(&tags_url));
        let mut tags = CloudTagMap::new();
        if let Ok(resp) = tags_req.send().await {
            if resp.status().is_success() {
                if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = resp_json.get("tags").and_then(|t| t.as_array()) {
                        tags = Self::parse_tms_tags(arr);
                    }
                }
            }
        }

        Ok(CloudResourceMeta {
            unique_id: unique_id.to_string(),
            tags,
            native_state_enabled: enabled,
            created_at_epoch_secs: None,
        })
    }

    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let url = format!(
            "{}/v1.0/{}/kms/{}/tags/action",
            self.endpoint_url, self.project_id, safe_id
        );

        let body = serde_json::json!({
            "action": "create",
            "tags": Self::format_tms_tags(upsert)
        });

        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC TMS tags action failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(format!(
                "OTC TMS tags action returned HTTP {}",
                resp.status()
            )));
        }

        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let url = format!(
            "{}/v1.0/{}/kms/resource_instances/action",
            self.endpoint_url, self.project_id
        );

        let tags_filter: Vec<serde_json::Value> = filter
            .exact_tags
            .iter()
            .map(|(k, v)| {
                serde_json::json!({
                    "key": k,
                    "values": [v]
                })
            })
            .collect();

        let body = serde_json::json!({
            "action": "filter",
            "tags": tags_filter
        });

        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC locate failed: {e}")))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC locate JSON error: {e}")))?;

        let mut ids = Vec::new();
        if let Some(resources) = resp_json.get("resources").and_then(|r| r.as_array()) {
            for res in resources {
                if let Some(rid) = res.get("resource_id").and_then(|v| v.as_str()) {
                    ids.push(rid.to_string());
                    if let Some(max) = max_items {
                        if ids.len() >= max as usize {
                            break;
                        }
                    }
                }
            }
        }

        Ok(ids)
    }

    async fn set_native_enabled_state(
        &self,
        unique_id: &str,
        enabled: bool,
    ) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let action = if enabled { "enable-key" } else { "disable-key" };
        let url = format!(
            "{}/v1.0/{}/kms/{}",
            self.endpoint_url, self.project_id, action
        );
        let body = serde_json::json!({ "key_id": safe_id });

        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC {action} failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(format!(
                "OTC {action} returned HTTP {}",
                resp.status()
            )));
        }

        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);

        // Try KMS schedule-key-deletion
        let kms_url = format!(
            "{}/v1.0/{}/kms/schedule-key-deletion",
            self.endpoint_url, self.project_id
        );
        let body = serde_json::json!({
            "key_id": safe_id,
            "pending_days": "7"
        });
        let req = self.apply_auth(self.client.post(&kms_url).json(&body));
        let _ = req.send().await;

        // Also try CSMS delete
        let csms_url = format!(
            "{}/v1/{}/secrets/{}",
            self.endpoint_url, self.project_id, safe_id
        );
        let csms_req = self.apply_auth(self.client.delete(&csms_url));
        let _ = csms_req.send().await;

        Ok(())
    }

    async fn encrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;

        let url = format!(
            "{}/v1.0/{}/kms/encrypt-data",
            self.endpoint_url, self.project_id
        );
        let body = serde_json::json!({
            "key_id": safe_id,
            "plain_text": b64.encode(plaintext)
        });

        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC encrypt-data failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(format!(
                "OTC encrypt-data returned HTTP {}",
                resp.status()
            )));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC encrypt JSON error: {e}")))?;

        let ct_b64 = resp_json
            .get("cipher_text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("OTC encrypt missing cipher_text".to_string())
            })?;

        b64.decode(ct_b64)
            .map_err(|e| ProxyError::ProviderError(format!("OTC encrypt base64 decode error: {e}")))
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;

        let url = format!(
            "{}/v1.0/{}/kms/decrypt-data",
            self.endpoint_url, self.project_id
        );
        let body = serde_json::json!({
            "key_id": safe_id,
            "cipher_text": b64.encode(ciphertext)
        });

        let req = self.apply_auth(self.client.post(&url).json(&body));
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC decrypt-data failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(format!(
                "OTC decrypt-data returned HTTP {}",
                resp.status()
            )));
        }

        let resp_json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OTC decrypt JSON error: {e}")))?;

        let pt_b64 = resp_json
            .get("plain_text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("OTC decrypt missing plain_text".to_string())
            })?;

        let decoded = b64.decode(pt_b64).map_err(|e| {
            ProxyError::ProviderError(format!("OTC decrypt base64 decode error: {e}"))
        })?;

        Ok(Zeroizing::new(decoded))
    }

    async fn sign(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        use aws_lc_rs::hmac;
        let s_key = hmac::Key::new(hmac::HMAC_SHA256, unique_id.as_bytes());
        let tag = hmac::sign(&s_key, data);
        Ok(tag.as_ref().to_vec())
    }

    async fn verify(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError> {
        use aws_lc_rs::hmac;
        let s_key = hmac::Key::new(hmac::HMAC_SHA256, unique_id.as_bytes());
        Ok(hmac::verify(&s_key, data, signature).is_ok())
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        crate::envelope::AutoEnvelope::generate_random_bytes(num_bytes)
    }
}

#[cfg(feature = "otc")]
fn fastrand_id() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}

#[cfg(not(feature = "otc"))]
#[derive(Debug, Clone, Default)]
pub struct OtcKmsProvider;

#[cfg(not(feature = "otc"))]
impl OtcKmsProvider {
    pub fn new(_config: &OtcConfig) -> Result<Self, ProxyError> {
        Err(ProxyError::Unsupported(
            "OTC KMS provider is disabled in this build".to_string(),
        ))
    }

    pub fn from_config(
        _config: &OtcConfig,
        _master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        Err(ProxyError::Unsupported(
            "OTC KMS provider is disabled in this build".to_string(),
        ))
    }
}

#[cfg(not(feature = "otc"))]
#[async_trait]
impl CloudKmsProvider for OtcKmsProvider {
    fn provider_name(&self) -> &'static str {
        "otc"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("OTC disabled".to_string()))
    }
}
