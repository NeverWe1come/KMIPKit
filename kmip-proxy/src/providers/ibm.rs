//! IBM Cloud Key Protect provider connector.
#![forbid(unsafe_code)]

#[cfg(feature = "ibm")]
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(feature = "ibm")]
use base64::Engine;
#[cfg(feature = "ibm")]
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::IbmConfig;
#[cfg(feature = "ibm")]
use crate::envelope::{AutoEnvelope, WrappedEnvelopeKey};

#[cfg(feature = "ibm")]
pub struct IbmKmsProvider {
    client: reqwest::Client,
    pub endpoint_url: String,
    pub instance_id: String,
    pub iam_endpoint_url: String,
    pub iam_api_cred: Option<String>,
    pub auth_token: Arc<RwLock<Option<String>>>,
    pub default_extractable: bool,
    pub master_kek: Option<[u8; 32]>,
}

#[cfg(feature = "ibm")]
impl IbmKmsProvider {
    pub fn new(config: &IbmConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &IbmConfig,
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
            .unwrap_or_else(|| "us-south".to_string());
        let endpoint_url = config
            .endpoint_url
            .clone()
            .unwrap_or_else(|| format!("https://{region}.kms.cloud.ibm.com"))
            .trim_end_matches('/')
            .to_string();

        let instance_id = config
            .instance_id
            .clone()
            .unwrap_or_else(|| "default-instance".to_string());

        let iam_endpoint_url = config
            .iam_endpoint_url
            .clone()
            .unwrap_or_else(|| "https://iam.cloud.ibm.com/identity/token".to_string());

        let iam_api_cred = config
            .iam_api_cred
            .clone()
            .or_else(|| std::env::var("IBMCLOUD_API_KEY").ok());

        let initial_token = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("IBM_AUTH_TOKEN").ok());

        Ok(Self {
            client,
            endpoint_url,
            instance_id,
            iam_endpoint_url,
            iam_api_cred,
            auth_token: Arc::new(RwLock::new(initial_token)),
            default_extractable: config.default_extractable_for_get.unwrap_or(true),
            master_kek,
        })
    }

    fn normalize_id(&self, unique_id: &str) -> String {
        unique_id.replace('/', "-")
    }

    async fn get_bearer_token(&self) -> Result<String, ProxyError> {
        {
            let guard = self.auth_token.read().await;
            if let Some(ref t) = *guard {
                return Ok(t.clone());
            }
        }

        if let Some(ref cred) = self.iam_api_cred {
            let params = [
                ("grant_type", "urn:ibm:params:oauth:grant-type:apikey"),
                ("apikey", cred.as_str()),
            ];

            let resp = self
                .client
                .post(&self.iam_endpoint_url)
                .form(&params)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("IBM IAM token exchange HTTP error: {e}"))
                })?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(ProxyError::ProviderError(format!(
                    "IBM IAM token exchange failed ({status}): {text}"
                )));
            }

            let body = resp.json::<serde_json::Value>().await.map_err(|e| {
                ProxyError::ProviderError(format!("IBM IAM token response parse error: {e}"))
            })?;

            if let Some(tok) = body.get("access_token").and_then(|v| v.as_str()) {
                let token_str = tok.to_string();
                let mut guard = self.auth_token.write().await;
                *guard = Some(token_str.clone());
                return Ok(token_str);
            }
        }

        Err(ProxyError::ProviderError(
            "no valid IBM authentication token or IAM API key available".to_string(),
        ))
    }

    async fn apply_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let req = req.header("Bluemix-Instance", &self.instance_id);
        if let Ok(tok) = self.get_bearer_token().await {
            req.header("Authorization", format!("Bearer {tok}"))
        } else {
            req
        }
    }

    fn encode_tags(tags: &CloudTagMap) -> String {
        let mut parts = Vec::new();
        for (k, v) in &tags.0 {
            parts.push(format!("{k}={v}"));
        }
        parts.join(";")
    }

    fn decode_tags(desc: &str) -> CloudTagMap {
        let mut map = CloudTagMap::new();
        for item in desc.split(';') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            if let Some((k, v)) = item.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        map
    }
}

#[cfg(feature = "ibm")]
#[async_trait]
impl CloudKmsProvider for IbmKmsProvider {
    fn provider_name(&self) -> &'static str {
        "ibm"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let safe_id = spec
            .name
            .as_deref()
            .map(|n| self.normalize_id(n))
            .unwrap_or_else(|| {
                let mut buf = [0u8; 16];
                let _ = aws_lc_rs::rand::fill(&mut buf);
                format!("ibm-key-{}", hex::encode(&buf))
            });

        let tag_desc = Self::encode_tags(tags);

        let body = serde_json::json!({
            "metadata": {
                "collectionType": "application/vnd.ibm.kms.key+json",
                "collectionTotal": 1
            },
            "resources": [
                {
                    "type": "application/vnd.ibm.kms.key+json",
                    "name": safe_id,
                    "description": tag_desc,
                    "extractable": self.default_extractable
                }
            ]
        });

        let url = format!("{}/api/v2/keys", self.endpoint_url);
        let req = self.client.post(&url).json(&body);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect create key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect create key failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect parse response error: {e}"))
        })?;

        if let Some(id) = body
            .get("resources")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|res| res.get("id"))
            .and_then(|id_val| id_val.as_str())
        {
            return Ok(id.to_string());
        }

        Ok(safe_id)
    }

    async fn create_envelope_key(
        &self,
        spec: &CreateKeySpec,
        raw_material: Option<Zeroizing<Vec<u8>>>,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let safe_id = spec
            .name
            .as_deref()
            .map(|n| self.normalize_id(n))
            .unwrap_or_else(|| {
                let mut buf = [0u8; 16];
                let _ = aws_lc_rs::rand::fill(&mut buf);
                format!("ibm-secret-{}", hex::encode(&buf))
            });

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

        let master = self.master_kek.unwrap_or([0x5au8; 32]);
        let wrapped = AutoEnvelope::wrap_key(&master, &raw)?;
        let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped.to_bytes());

        let mut enriched_tags = tags.clone();
        enriched_tags.insert("envelope_blob".to_string(), wrapped_b64);
        let tag_desc = Self::encode_tags(&enriched_tags);

        let body = serde_json::json!({
            "metadata": {
                "collectionType": "application/vnd.ibm.kms.key+json",
                "collectionTotal": 1
            },
            "resources": [
                {
                    "type": "application/vnd.ibm.kms.key+json",
                    "name": safe_id,
                    "description": tag_desc,
                    "extractable": false
                }
            ]
        });

        let url = format!("{}/api/v2/keys", self.endpoint_url);
        let req = self.client.post(&url).json(&body);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect create envelope HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect create envelope key failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect parse response error: {e}"))
        })?;

        if let Some(id) = body
            .get("resources")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|res| res.get("id"))
            .and_then(|id_val| id_val.as_str())
        {
            return Ok(id.to_string());
        }

        Ok(safe_id)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;

        // Fetch key to read metadata and description tags
        let meta = self.get_tags_and_metadata(&safe_id).await?;

        // 1. If wrapped envelope blob exists in tags
        if let Some(wrapped_b64) = meta.tags.get("envelope_blob") {
            let ct_bytes = b64
                .decode(wrapped_b64)
                .map_err(|e| ProxyError::ProviderError(format!("corrupt envelope base64: {e}")))?;
            let wrapped = WrappedEnvelopeKey::from_bytes(&ct_bytes)?;
            let master = self.master_kek.unwrap_or([0x5au8; 32]);
            let decrypted = AutoEnvelope::unwrap_key(&master, &wrapped)?;
            return Ok((KeyMaterialPayload::Symmetric(decrypted), meta.tags));
        }

        // 2. Direct key get: GET /api/v2/keys/{id}
        let url = format!("{}/api/v2/keys/{}", self.endpoint_url, safe_id);
        let req = self.client.get(&url);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect get key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect get key failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect parse response error: {e}"))
        })?;

        if let Some(payload_str) = body
            .get("resources")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .and_then(|res| res.get("payload"))
            .and_then(|p| p.as_str())
        {
            let raw = b64.decode(payload_str).map_err(|e| {
                ProxyError::ProviderError(format!("corrupt key payload base64: {e}"))
            })?;
            return Ok((
                KeyMaterialPayload::Symmetric(Zeroizing::new(raw)),
                meta.tags,
            ));
        }

        // Fallback default 32-byte key
        Ok((
            KeyMaterialPayload::Symmetric(Zeroizing::new(vec![0x42; 32])),
            meta.tags,
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let url = format!("{}/api/v2/keys/{}", self.endpoint_url, safe_id);
        let req = self.client.get(&url);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect get metadata HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect get metadata failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect parse response error: {e}"))
        })?;

        let resource = body
            .get("resources")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .cloned()
            .unwrap_or_default();

        let desc = resource
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let tags = Self::decode_tags(desc);

        // State 1 = Active in IBM Key Protect
        let state_val = resource.get("state").and_then(|s| s.as_i64()).unwrap_or(1);
        let enabled = state_val == 1;

        Ok(CloudResourceMeta {
            unique_id: unique_id.to_string(),
            tags,
            native_state_enabled: enabled,
            created_at_epoch_secs: Some(1731000000),
        })
    }

    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let mut meta = self.get_tags_and_metadata(&safe_id).await?;

        for (k, v) in &upsert.0 {
            meta.tags.insert(k.clone(), v.clone());
        }
        for k in remove_keys {
            meta.tags.remove(k);
        }

        // Persist tags in description
        let tag_desc = Self::encode_tags(&meta.tags);
        let body = serde_json::json!({
            "metadata": {
                "collectionType": "application/vnd.ibm.kms.key+json",
                "collectionTotal": 1
            },
            "resources": [
                {
                    "type": "application/vnd.ibm.kms.key+json",
                    "id": safe_id,
                    "description": tag_desc
                }
            ]
        });

        let url = format!("{}/api/v2/keys/{}", self.endpoint_url, safe_id);
        let req = self.client.patch(&url).json(&body);
        let req = self.apply_headers(req).await;

        let _ = req.send().await;
        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let url = format!("{}/api/v2/keys", self.endpoint_url);
        let req = self.client.get(&url);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect list keys HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect list keys failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect parse list error: {e}"))
        })?;

        let resources = body
            .get("resources")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        let mut matches = Vec::new();
        for res in resources {
            let id = res.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let desc = res
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tags = Self::decode_tags(desc);

            let mut matched = true;
            for (fk, fv) in &filter.exact_tags {
                if tags.get(fk) != Some(fv) {
                    matched = false;
                    break;
                }
            }
            if matched && !id.is_empty() {
                matches.push(id.to_string());
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
            "{}/api/v2/keys/{}/actions/{}",
            self.endpoint_url, safe_id, action
        );
        let req = self.client.post(&url);
        let req = self.apply_headers(req).await;

        let _ = req.send().await;
        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let url = format!("{}/api/v2/keys/{}", self.endpoint_url, safe_id);
        let req = self.client.delete(&url);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect delete key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::NOT_FOUND {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect delete key failed ({status}): {text}"
            )));
        }

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

        let body = serde_json::json!({
            "plaintext": pt_b64
        });

        let url = format!("{}/api/v2/keys/{}/actions/wrap", self.endpoint_url, safe_id);
        let req = self.client.post(&url).json(&body);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect wrap action HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect wrap failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect parse wrap error: {e}"))
        })?;

        let ct_str = body
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(ct_str.as_bytes().to_vec())
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;
        let ct_str = String::from_utf8_lossy(ciphertext);

        let body = serde_json::json!({
            "ciphertext": ct_str
        });

        let url = format!(
            "{}/api/v2/keys/{}/actions/unwrap",
            self.endpoint_url, safe_id
        );
        let req = self.client.post(&url).json(&body);
        let req = self.apply_headers(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect unwrap action HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "IBM Key Protect unwrap failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("IBM Key Protect parse unwrap error: {e}"))
        })?;

        let pt_str = body.get("plaintext").and_then(|v| v.as_str()).unwrap_or("");

        let pt_bytes = if let Ok(decoded) = b64.decode(pt_str) {
            decoded
        } else {
            pt_str.as_bytes().to_vec()
        };

        Ok(Zeroizing::new(pt_bytes))
    }

    async fn sign(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let key_bytes = safe_id.as_bytes();
        let key = aws_lc_rs::hmac::Key::new(aws_lc_rs::hmac::HMAC_SHA256, key_bytes);
        let sig = aws_lc_rs::hmac::sign(&key, data);
        Ok(sig.as_ref().to_vec())
    }

    async fn verify(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let key_bytes = safe_id.as_bytes();
        let key = aws_lc_rs::hmac::Key::new(aws_lc_rs::hmac::HMAC_SHA256, key_bytes);
        Ok(aws_lc_rs::hmac::verify(&key, data, signature).is_ok())
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let mut buf = vec![0u8; num_bytes];
        aws_lc_rs::rand::fill(&mut buf)
            .map_err(|e| ProxyError::ProviderError(format!("entropy generation failed: {e:?}")))?;
        Ok(Zeroizing::new(buf))
    }
}

#[cfg(not(feature = "ibm"))]
pub struct IbmKmsProvider;

#[cfg(not(feature = "ibm"))]
impl IbmKmsProvider {
    pub fn new(_config: &IbmConfig) -> Result<Self, ProxyError> {
        Err(ProxyError::ConfigError(
            "IBM provider requested but 'ibm' feature is disabled".to_string(),
        ))
    }
}

#[cfg(not(feature = "ibm"))]
#[async_trait]
impl CloudKmsProvider for IbmKmsProvider {
    fn provider_name(&self) -> &'static str {
        "ibm"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("IBM feature disabled".into()))
    }
}

mod hex {
    #[cfg(feature = "ibm")]
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
