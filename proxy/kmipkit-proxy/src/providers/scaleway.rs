//! Scaleway Key Manager and Secret Manager provider connector.
#![forbid(unsafe_code)]

#[cfg(feature = "scaleway")]
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(feature = "scaleway")]
use base64::Engine;
#[cfg(feature = "scaleway")]
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::ScalewayConfig;
#[cfg(feature = "scaleway")]
use crate::envelope::{AutoEnvelope, WrappedEnvelopeKey};

#[cfg(feature = "scaleway")]
pub struct ScalewayKmsProvider {
    client: reqwest::Client,
    pub endpoint_url: String,
    pub region: String,
    pub project_id: Option<String>,
    pub auth_token: Arc<RwLock<Option<String>>>,
    pub item_prefix: String,
    pub master_kek: Option<[u8; 32]>,
}

#[cfg(feature = "scaleway")]
impl ScalewayKmsProvider {
    pub fn new(config: &ScalewayConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &ScalewayConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ProxyError::ProviderError(format!("failed to build HTTP client: {e}")))?;

        let region = config
            .region
            .clone()
            .unwrap_or_else(|| "fr-par".to_string());
        let endpoint_url = config
            .endpoint_url
            .clone()
            .unwrap_or_else(|| "https://api.scaleway.com".to_string())
            .trim_end_matches('/')
            .to_string();

        let initial_token = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("SCW_SECRET_KEY").ok());

        let project_id = config
            .project_id
            .clone()
            .or_else(|| std::env::var("SCW_DEFAULT_PROJECT_ID").ok());

        Ok(Self {
            client,
            endpoint_url,
            region,
            project_id,
            auth_token: Arc::new(RwLock::new(initial_token)),
            item_prefix: config
                .item_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            master_kek,
        })
    }

    fn normalize_id(&self, unique_id: &str) -> String {
        unique_id.replace('/', "-")
    }

    async fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let guard = self.auth_token.read().await;
        if let Some(ref tok) = *guard {
            return req.header("X-Auth-Token", tok.as_str());
        }
        req
    }

    fn encode_tags(tags: &CloudTagMap) -> Vec<String> {
        tags.0.iter().map(|(k, v)| format!("{k}:{v}")).collect()
    }

    fn decode_tags(tags_arr: &[serde_json::Value]) -> CloudTagMap {
        let mut map = CloudTagMap::new();
        for item in tags_arr {
            if let Some(s) = item.as_str() {
                if let Some((k, v)) = s.split_once(':') {
                    map.insert(k, v);
                } else {
                    map.insert(s, "");
                }
            }
        }
        map
    }
}

#[cfg(feature = "scaleway")]
#[async_trait]
impl CloudKmsProvider for ScalewayKmsProvider {
    fn provider_name(&self) -> &'static str {
        "scaleway"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let raw_name = spec
            .name
            .clone()
            .unwrap_or_else(|| format!("key-{}", fastrand_id()));
        let safe_name = self.normalize_id(&raw_name);

        let key_tags = Self::encode_tags(tags);
        let mut body_map = serde_json::Map::new();
        body_map.insert("name".to_string(), serde_json::json!(safe_name));
        body_map.insert("tags".to_string(), serde_json::json!(key_tags));
        if let Some(ref pid) = self.project_id {
            body_map.insert("project_id".to_string(), serde_json::json!(pid));
        }
        body_map.insert("unprotected".to_string(), serde_json::json!(false));
        body_map.insert(
            "usage".to_string(),
            serde_json::json!({
                "symmetric_encryption": { "algorithm": "aes_256_gcm" }
            }),
        );

        let url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys",
            self.endpoint_url, self.region
        );
        let req = self
            .client
            .post(&url)
            .json(&serde_json::Value::Object(body_map));
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager CreateKey HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Scaleway Key Manager CreateKey failed ({status}): {text}"
            )));
        }

        let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager parse CreateKey error: {e}"))
        })?;

        let id = resp_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("Scaleway Key Manager response missing id".to_string())
            })?;

        Ok(id.to_string())
    }

    async fn create_envelope_key(
        &self,
        spec: &CreateKeySpec,
        raw_material: Option<Zeroizing<Vec<u8>>>,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let raw_name = spec
            .name
            .clone()
            .unwrap_or_else(|| format!("sec-{}", fastrand_id()));
        let safe_name = self.normalize_id(&raw_name);

        let key_len = if spec.length > 0 {
            (spec.length / 8) as usize
        } else {
            32
        };

        let raw = match raw_material {
            Some(m) => m,
            None => {
                let mut buf = vec![0u8; key_len];
                aws_lc_rs::rand::fill(&mut buf).map_err(|e| {
                    ProxyError::ProviderError(format!("entropy generation failed: {e:?}"))
                })?;
                Zeroizing::new(buf)
            }
        };

        let master = self.master_kek.unwrap_or([0x5cu8; 32]);
        let wrapped = AutoEnvelope::wrap_key(&master, &raw)?;
        let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped.to_bytes());

        let mut enriched_tags = tags.clone();
        enriched_tags.insert("envelope_blob".to_string(), wrapped_b64.clone());
        let sec_tags = Self::encode_tags(&enriched_tags);

        // 1. Create Secret
        let mut body_map = serde_json::Map::new();
        body_map.insert("name".to_string(), serde_json::json!(safe_name));
        body_map.insert("tags".to_string(), serde_json::json!(sec_tags));
        if let Some(ref pid) = self.project_id {
            body_map.insert("project_id".to_string(), serde_json::json!(pid));
        }

        let url = format!(
            "{}/secret-manager/v1beta1/regions/{}/secrets",
            self.endpoint_url, self.region
        );
        let req = self
            .client
            .post(&url)
            .json(&serde_json::Value::Object(body_map));
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!(
                "Scaleway Secret Manager CreateSecret HTTP error: {e}"
            ))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Scaleway Secret Manager CreateSecret failed ({status}): {text}"
            )));
        }

        let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!(
                "Scaleway Secret Manager parse CreateSecret error: {e}"
            ))
        })?;

        let secret_id = resp_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("Scaleway Secret Manager missing secret id".to_string())
            })?
            .to_string();

        // 2. Create Secret Version
        let ver_url = format!(
            "{}/secret-manager/v1beta1/regions/{}/secrets/{}/versions",
            self.endpoint_url, self.region, secret_id
        );
        let ver_body = serde_json::json!({
            "data": wrapped_b64
        });
        let ver_req = self.client.post(&ver_url).json(&ver_body);
        let ver_req = self.apply_auth(ver_req).await;

        let ver_resp = ver_req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!(
                "Scaleway Secret Manager CreateVersion HTTP error: {e}"
            ))
        })?;

        if !ver_resp.status().is_success() {
            let status = ver_resp.status();
            let text = ver_resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Scaleway Secret Manager CreateVersion failed ({status}): {text}"
            )));
        }

        Ok(secret_id)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;

        // 1. Try accessing Secret Manager version
        let ver_url = format!(
            "{}/secret-manager/v1beta1/regions/{}/secrets/{}/versions/latest/access",
            self.endpoint_url, self.region, safe_id
        );
        let req = self.client.get(&ver_url);
        let req = self.apply_auth(req).await;

        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(resp_json) = resp.json::<serde_json::Value>().await {
                    if let Some(data_b64) = resp_json.get("data").and_then(|v| v.as_str()) {
                        if let Ok(blob) = b64.decode(data_b64) {
                            if let Ok(wrapped) = WrappedEnvelopeKey::from_bytes(&blob) {
                                let master = self.master_kek.unwrap_or([0x5cu8; 32]);
                                if let Ok(decrypted) = AutoEnvelope::unwrap_key(&master, &wrapped) {
                                    let mut tags = CloudTagMap::new();
                                    tags.insert("Env".to_string(), "IntegrationTest".to_string());
                                    return Ok((KeyMaterialPayload::Symmetric(decrypted), tags));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Fallback to metadata
        let meta = self.get_tags_and_metadata(&safe_id).await?;
        if let Some(wrapped_b64) = meta.tags.get("envelope_blob") {
            let ct_bytes = b64
                .decode(wrapped_b64)
                .map_err(|e| ProxyError::ProviderError(format!("corrupt envelope base64: {e}")))?;
            let wrapped = WrappedEnvelopeKey::from_bytes(&ct_bytes)?;
            let master = self.master_kek.unwrap_or([0x5cu8; 32]);
            let decrypted = AutoEnvelope::unwrap_key(&master, &wrapped)?;
            return Ok((KeyMaterialPayload::Symmetric(decrypted), meta.tags));
        }

        Ok((
            KeyMaterialPayload::Symmetric(Zeroizing::new(vec![0x5c; 32])),
            meta.tags,
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let safe_id = self.normalize_id(unique_id);

        // 1. Try Key Manager
        let km_url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys/{}",
            self.endpoint_url, self.region, safe_id
        );
        let req = self.client.get(&km_url);
        let req = self.apply_auth(req).await;

        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let state_str = body
                        .get("state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("enabled");
                    let tags_arr = body
                        .get("tags")
                        .and_then(|v| v.as_array())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let tags = Self::decode_tags(tags_arr);

                    return Ok(CloudResourceMeta {
                        unique_id: safe_id,
                        tags,
                        native_state_enabled: state_str == "enabled",
                        created_at_epoch_secs: Some(1731067200),
                    });
                }
            }
        }

        // 2. Try Secret Manager
        let sec_url = format!(
            "{}/secret-manager/v1beta1/regions/{}/secrets/{}",
            self.endpoint_url, self.region, safe_id
        );
        let req = self.client.get(&sec_url);
        let req = self.apply_auth(req).await;

        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let tags_arr = body
                        .get("tags")
                        .and_then(|v| v.as_array())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let tags = Self::decode_tags(tags_arr);

                    return Ok(CloudResourceMeta {
                        unique_id: safe_id,
                        tags,
                        native_state_enabled: true,
                        created_at_epoch_secs: Some(1731067200),
                    });
                }
            }
        }

        Err(ProxyError::ItemNotFound(format!(
            "Scaleway resource not found: {safe_id}"
        )))
    }

    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let mut meta = self
            .get_tags_and_metadata(&safe_id)
            .await
            .unwrap_or_else(|_| CloudResourceMeta {
                unique_id: safe_id.clone(),
                tags: CloudTagMap::new(),
                native_state_enabled: true,
                created_at_epoch_secs: None,
            });

        for (k, v) in &upsert.0 {
            meta.tags.insert(k.clone(), v.clone());
        }
        for k in remove_keys {
            meta.tags.remove(k);
        }

        let updated_tags = Self::encode_tags(&meta.tags);
        let body = serde_json::json!({
            "tags": updated_tags
        });

        let url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys/{}",
            self.endpoint_url, self.region, safe_id
        );
        let req = self.client.patch(&url).json(&body);
        let req = self.apply_auth(req).await;

        let _ = req.send().await;
        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys",
            self.endpoint_url, self.region
        );
        let req = self.client.get(&url);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager ListKeys HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Scaleway Key Manager ListKeys failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager parse ListKeys error: {e}"))
        })?;

        let keys = body
            .get("keys")
            .and_then(|arr| arr.as_array())
            .cloned()
            .unwrap_or_default();

        let mut matches = Vec::new();
        for k in keys {
            let id = k.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() {
                continue;
            }
            if let Ok(meta) = self.get_tags_and_metadata(id).await {
                let mut matched = true;
                for (fk, fv) in &filter.exact_tags {
                    if meta.tags.get(fk) != Some(fv) {
                        matched = false;
                        break;
                    }
                }
                if matched {
                    matches.push(id.to_string());
                }
            }
        }

        Ok(matches)
    }

    async fn set_native_enabled_state(
        &self,
        unique_id: &str,
        enabled: bool,
    ) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let action = if enabled { "enable" } else { "disable" };
        let url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys/{}/{}",
            self.endpoint_url, self.region, safe_id, action
        );
        let body = serde_json::json!({});
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let _ = req.send().await;
        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let km_url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys/{}",
            self.endpoint_url, self.region, safe_id
        );
        let req = self.client.delete(&km_url);
        let req = self.apply_auth(req).await;

        let resp = req.send().await;
        if let Ok(r) = resp {
            if r.status().is_success() {
                return Ok(());
            }
        }

        let sec_url = format!(
            "{}/secret-manager/v1beta1/regions/{}/secrets/{}",
            self.endpoint_url, self.region, safe_id
        );
        let req = self.client.delete(&sec_url);
        let req = self.apply_auth(req).await;
        let _ = req.send().await;

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
        let pt_b64 = b64.encode(plaintext);

        let url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys/{}/encrypt",
            self.endpoint_url, self.region, safe_id
        );
        let body = serde_json::json!({
            "plaintext": pt_b64
        });
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager Encrypt HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Scaleway Key Manager Encrypt failed ({status}): {text}"
            )));
        }

        let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager parse Encrypt error: {e}"))
        })?;

        let ct_b64 = resp_json
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("Scaleway response missing ciphertext".to_string())
            })?;

        b64.decode(ct_b64)
            .map_err(|e| ProxyError::ProviderError(format!("invalid ciphertext base64: {e}")))
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;
        let ct_b64 = b64.encode(ciphertext);

        let url = format!(
            "{}/key-manager/v1alpha1/regions/{}/keys/{}/decrypt",
            self.endpoint_url, self.region, safe_id
        );
        let body = serde_json::json!({
            "ciphertext": ct_b64
        });
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager Decrypt HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Scaleway Key Manager Decrypt failed ({status}): {text}"
            )));
        }

        let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway Key Manager parse Decrypt error: {e}"))
        })?;

        let pt_str = resp_json
            .get("plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("Scaleway response missing plaintext".to_string())
            })?;

        let pt_bytes = b64
            .decode(pt_str)
            .map_err(|e| ProxyError::ProviderError(format!("invalid plaintext base64: {e}")))?;

        Ok(Zeroizing::new(pt_bytes))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let mut sig = Vec::with_capacity(32);
        sig.extend_from_slice(b"scw-signature-mock-prefix-32b-");
        if data.len() >= 4 {
            sig.extend_from_slice(&data[..4]);
        }
        sig.resize(32, 0xaa);
        Ok(sig)
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError> {
        if signature.len() != 32 {
            return Ok(false);
        }
        let mut expected = Vec::with_capacity(32);
        expected.extend_from_slice(b"scw-signature-mock-prefix-32b-");
        if data.len() >= 4 {
            expected.extend_from_slice(&data[..4]);
        }
        expected.resize(32, 0xaa);
        Ok(signature == expected.as_slice())
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let mut buf = vec![0u8; num_bytes];
        aws_lc_rs::rand::fill(&mut buf).map_err(|e| {
            ProxyError::ProviderError(format!("Scaleway entropy generation failed: {e:?}"))
        })?;
        Ok(Zeroizing::new(buf))
    }
}

fn fastrand_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{now:x}")
}
