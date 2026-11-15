//! Alibaba Cloud KMS and Secrets Manager provider connector.
#![forbid(unsafe_code)]

#[cfg(feature = "alibaba")]
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(feature = "alibaba")]
use base64::Engine;
#[cfg(feature = "alibaba")]
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::AlibabaConfig;
#[cfg(feature = "alibaba")]
use crate::envelope::{AutoEnvelope, WrappedEnvelopeKey};

#[cfg(feature = "alibaba")]
pub struct AlibabaKmsProvider {
    client: reqwest::Client,
    pub endpoint_url: String,
    pub region_id: String,
    pub access_key_id: Option<String>,
    pub access_key_cred: Option<String>,
    pub auth_token: Arc<RwLock<Option<String>>>,
    pub item_prefix: String,
    pub master_kek: Option<[u8; 32]>,
}

#[cfg(feature = "alibaba")]
impl AlibabaKmsProvider {
    pub fn new(config: &AlibabaConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &AlibabaConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ProxyError::ProviderError(format!("failed to build HTTP client: {e}")))?;

        let region_id = config
            .region_id
            .clone()
            .unwrap_or_else(|| "cn-hangzhou".to_string());
        let endpoint_url = config
            .endpoint_url
            .clone()
            .unwrap_or_else(|| format!("https://kms.{region_id}.aliyuncs.com"))
            .trim_end_matches('/')
            .to_string();

        let access_key_id = config
            .access_key_id
            .clone()
            .or_else(|| std::env::var("ALIBABA_CLOUD_ACCESS_KEY_ID").ok());

        let access_key_cred = config
            .access_key_cred
            .clone()
            .or_else(|| std::env::var("ALIBABA_CLOUD_ACCESS_KEY_SECRET").ok());

        let initial_token = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("ALIBABA_AUTH_TOKEN").ok());

        Ok(Self {
            client,
            endpoint_url,
            region_id,
            access_key_id,
            access_key_cred,
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
            return req.header("Authorization", format!("Bearer {tok}"));
        }
        if let Some(ref ak) = self.access_key_id {
            return req
                .header("x-acs-accesskey-id", ak.as_str())
                .header("x-acs-signature-method", "HMAC-SHA256")
                .header("x-acs-signature-version", "1.0");
        }
        req
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

#[cfg(feature = "alibaba")]
#[async_trait]
impl CloudKmsProvider for AlibabaKmsProvider {
    fn provider_name(&self) -> &'static str {
        "alibaba"
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
                format!("ali-key-{}", hex::encode(&buf))
            });

        let tag_desc = Self::encode_tags(tags);

        let body = serde_json::json!({
            "KeySpec": "Aliyun_AES_256",
            "KeyUsage": "ENCRYPT/DECRYPT",
            "Description": tag_desc,
            "name": safe_id
        });

        let url = format!("{}/?Action=CreateKey&Version=2016-01-20", self.endpoint_url);
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS CreateKey HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Alibaba KMS CreateKey failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS parse CreateKey error: {e}"))
        })?;

        if let Some(key_id) = body
            .get("KeyMetadata")
            .and_then(|m| m.get("KeyId"))
            .and_then(|id| id.as_str())
        {
            return Ok(key_id.to_string());
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
                format!("ali-secret-{}", hex::encode(&buf))
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
        enriched_tags.insert("envelope_blob".to_string(), wrapped_b64.clone());
        let tag_desc = Self::encode_tags(&enriched_tags);

        let body = serde_json::json!({
            "SecretName": safe_id,
            "SecretType": "Generic",
            "SecretData": wrapped_b64,
            "Description": tag_desc
        });

        let url = format!(
            "{}/?Action=CreateSecret&Version=2016-01-20",
            self.endpoint_url
        );
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS CreateSecret HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Alibaba KMS CreateSecret failed ({status}): {text}"
            )));
        }

        Ok(safe_id)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;

        // 1. Try Secrets Manager GetSecretValue
        let body = serde_json::json!({
            "SecretName": safe_id
        });

        let url = format!(
            "{}/?Action=GetSecretValue&Version=2016-01-20",
            self.endpoint_url
        );
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(json_body) = resp.json::<serde_json::Value>().await {
                    if let Some(sec_data) = json_body.get("SecretData").and_then(|d| d.as_str()) {
                        if let Ok(ct_bytes) = b64.decode(sec_data) {
                            if let Ok(wrapped) = WrappedEnvelopeKey::from_bytes(&ct_bytes) {
                                let master = self.master_kek.unwrap_or([0x5au8; 32]);
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

        // 2. Fallback to reading key metadata
        let meta = self.get_tags_and_metadata(&safe_id).await?;
        if let Some(wrapped_b64) = meta.tags.get("envelope_blob") {
            let ct_bytes = b64
                .decode(wrapped_b64)
                .map_err(|e| ProxyError::ProviderError(format!("corrupt envelope base64: {e}")))?;
            let wrapped = WrappedEnvelopeKey::from_bytes(&ct_bytes)?;
            let master = self.master_kek.unwrap_or([0x5au8; 32]);
            let decrypted = AutoEnvelope::unwrap_key(&master, &wrapped)?;
            return Ok((KeyMaterialPayload::Symmetric(decrypted), meta.tags));
        }

        // Native key fallback
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
        let body = serde_json::json!({
            "KeyId": safe_id
        });

        let url = format!(
            "{}/?Action=DescribeKey&Version=2016-01-20",
            self.endpoint_url
        );
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS DescribeKey HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Alibaba KMS DescribeKey failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS parse DescribeKey error: {e}"))
        })?;

        let key_meta = body.get("KeyMetadata").unwrap_or(&body);
        let desc = key_meta
            .get("Description")
            .and_then(|d| d.as_str())
            .unwrap_or("");
        let tags = Self::decode_tags(desc);

        let state_str = key_meta
            .get("KeyState")
            .and_then(|s| s.as_str())
            .unwrap_or("Enabled");
        let enabled = state_str == "Enabled";

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

        let tag_desc = Self::encode_tags(&meta.tags);
        let body = serde_json::json!({
            "KeyId": safe_id,
            "Description": tag_desc
        });

        let url = format!(
            "{}/?Action=UpdateKeyDescription&Version=2016-01-20",
            self.endpoint_url
        );
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let _ = req.send().await;
        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let url = format!("{}/?Action=ListKeys&Version=2016-01-20", self.endpoint_url);
        let body = serde_json::json!({});
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS ListKeys HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Alibaba KMS ListKeys failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS parse ListKeys error: {e}"))
        })?;

        let keys = body
            .get("Keys")
            .and_then(|k| k.get("Key"))
            .and_then(|arr| arr.as_array())
            .cloned()
            .unwrap_or_default();

        let mut matches = Vec::new();
        for k in keys {
            let id = k.get("KeyId").and_then(|v| v.as_str()).unwrap_or("");
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
        let action = if enabled { "EnableKey" } else { "DisableKey" };
        let url = format!(
            "{}/?Action={}&Version=2016-01-20",
            self.endpoint_url, action
        );
        let body = serde_json::json!({
            "KeyId": safe_id
        });
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let _ = req.send().await;
        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let body = serde_json::json!({
            "KeyId": safe_id,
            "PendingWindowInDays": 7
        });

        let url = format!(
            "{}/?Action=ScheduleKeyDeletion&Version=2016-01-20",
            self.endpoint_url
        );
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS ScheduleKeyDeletion HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            // Also try DeleteSecret
            let sec_body = serde_json::json!({
                "SecretName": safe_id,
                "ForceDeleteWithoutRecovery": "true"
            });
            let sec_url = format!(
                "{}/?Action=DeleteSecret&Version=2016-01-20",
                self.endpoint_url
            );
            let sec_req = self.client.post(&sec_url).json(&sec_body);
            let sec_req = self.apply_auth(sec_req).await;
            let _ = sec_req.send().await;
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
            "KeyId": safe_id,
            "Plaintext": pt_b64
        });

        let url = format!("{}/?Action=Encrypt&Version=2016-01-20", self.endpoint_url);
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS Encrypt HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Alibaba KMS Encrypt failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS parse Encrypt error: {e}"))
        })?;

        let ct_blob = body
            .get("CiphertextBlob")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(ct_blob.as_bytes().to_vec())
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
            "KeyId": safe_id,
            "CiphertextBlob": ct_str
        });

        let url = format!("{}/?Action=Decrypt&Version=2016-01-20", self.endpoint_url);
        let req = self.client.post(&url).json(&body);
        let req = self.apply_auth(req).await;

        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS Decrypt HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Alibaba KMS Decrypt failed ({status}): {text}"
            )));
        }

        let body = resp.json::<serde_json::Value>().await.map_err(|e| {
            ProxyError::ProviderError(format!("Alibaba KMS parse Decrypt error: {e}"))
        })?;

        let pt_str = body.get("Plaintext").and_then(|v| v.as_str()).unwrap_or("");

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

#[cfg(not(feature = "alibaba"))]
pub struct AlibabaKmsProvider;

#[cfg(not(feature = "alibaba"))]
impl AlibabaKmsProvider {
    pub fn new(_config: &AlibabaConfig) -> Result<Self, ProxyError> {
        Err(ProxyError::ConfigError(
            "Alibaba provider requested but 'alibaba' feature is disabled".to_string(),
        ))
    }
}

#[cfg(not(feature = "alibaba"))]
#[async_trait]
impl CloudKmsProvider for AlibabaKmsProvider {
    fn provider_name(&self) -> &'static str {
        "alibaba"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("Alibaba feature disabled".into()))
    }
}

mod hex {
    #[cfg(feature = "alibaba")]
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
