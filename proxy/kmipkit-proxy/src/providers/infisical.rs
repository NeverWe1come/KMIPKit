//! Infisical KMS provider connector.
#![forbid(unsafe_code)]

#[cfg(feature = "infisical")]
use std::sync::Arc;

use async_trait::async_trait;
#[cfg(feature = "infisical")]
use base64::Engine;
#[cfg(feature = "infisical")]
use tokio::sync::RwLock;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::InfisicalConfig;
#[cfg(feature = "infisical")]
use crate::envelope::{AutoEnvelope, WrappedEnvelopeKey};

#[cfg(feature = "infisical")]
pub struct InfisicalKmsProvider {
    client: reqwest::Client,
    pub site_url: String,
    pub project_id: String,
    pub client_id: Option<String>,
    pub client_cred: Option<String>,
    pub auth_token: Arc<RwLock<Option<String>>>,
    pub item_prefix: String,
    pub master_kek: Option<[u8; 32]>,
}

#[cfg(feature = "infisical")]
impl InfisicalKmsProvider {
    pub fn new(config: &InfisicalConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &InfisicalConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ProxyError::ProviderError(format!("failed to build HTTP client: {e}")))?;

        let site_url = config
            .site_url
            .clone()
            .unwrap_or_else(|| "https://eu.infisical.com".to_string())
            .trim_end_matches('/')
            .to_string();

        let project_id = config
            .project_id
            .clone()
            .unwrap_or_else(|| "default".to_string());

        let initial_token = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("INFISICAL_TOKEN").ok());

        Ok(Self {
            client,
            site_url,
            project_id,
            client_id: config.client_id.clone(),
            client_cred: config.client_cred.clone(),
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

    async fn get_bearer_token(&self) -> Result<String, ProxyError> {
        {
            let guard = self.auth_token.read().await;
            if let Some(ref t) = *guard {
                return Ok(t.clone());
            }
        }

        // Try Universal Auth login if client_id and client_cred exist
        if let (Some(cid), Some(csec)) = (&self.client_id, &self.client_cred) {
            let login_url = format!("{}/api/v1/auth/universal-auth/login", self.site_url);
            let body = serde_json::json!({
                "clientId": cid,
                "clientSecret": csec
            });

            let resp = self
                .client
                .post(&login_url)
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("Universal auth login HTTP error: {e}"))
                })?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(ProxyError::ProviderError(format!(
                    "Universal auth login failed ({status}): {text}"
                )));
            }

            let parsed: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse auth login JSON: {e}"))
            })?;

            let token = parsed
                .pointer("/accessToken")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ProxyError::ProviderError(
                        "missing accessToken in auth login response".to_string(),
                    )
                })?
                .to_string();

            let mut guard = self.auth_token.write().await;
            *guard = Some(token.clone());
            return Ok(token);
        }

        Ok(String::new())
    }

    async fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Ok(token) = self.get_bearer_token().await {
            if !token.is_empty() {
                return req.header("Authorization", format!("Bearer {token}"));
            }
        }
        req
    }

    fn keys_base_url(&self) -> String {
        format!("{}/api/v1/kms/keys", self.site_url)
    }

    fn key_url(&self, safe_id: &str) -> String {
        format!("{}/api/v1/kms/keys/{}", self.site_url, safe_id)
    }

    fn encode_tags(tags: &CloudTagMap) -> String {
        let mut parts = Vec::new();
        for (k, v) in &tags.0 {
            parts.push(format!("{k}={v}"));
        }
        format!("kmip:{}", parts.join(";"))
    }

    fn decode_tags(desc: &str) -> CloudTagMap {
        let mut map = CloudTagMap::new();
        if let Some(rest) = desc.strip_prefix("kmip:") {
            for pair in rest.split(';') {
                if let Some((k, v)) = pair.split_once('=') {
                    if !k.is_empty() {
                        map.insert(k.to_string(), v.to_string());
                    }
                }
            }
        }
        map
    }

    fn map_algorithm(spec: &CreateKeySpec) -> &'static str {
        match spec.algorithm.to_ascii_uppercase().as_str() {
            "RSA" => "rsa-4096",
            "ECDSA" | "EC" => "ecc-nist-p256",
            _ => match spec.length {
                128 => "aes-128-gcm",
                _ => "aes-256-gcm",
            },
        }
    }

    fn parse_created_time(time_str: &str) -> Option<i64> {
        let parts: Vec<&str> = time_str.split('T').collect();
        if parts.is_empty() {
            return None;
        }
        let date_parts: Vec<i64> = parts[0].split('-').filter_map(|s| s.parse().ok()).collect();
        if date_parts.len() != 3 {
            return None;
        }
        let year = date_parts[0];
        let month = date_parts[1];
        let day = date_parts[2];
        let days = (year - 1970) * 365 + (year - 1969) / 4 + (month - 1) * 30 + (day - 1);
        Some(days * 86400)
    }
}

#[cfg(feature = "infisical")]
#[async_trait]
impl CloudKmsProvider for InfisicalKmsProvider {
    fn provider_name(&self) -> &'static str {
        "infisical"
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
                format!("infisical-key-{}", hex::encode(&buf))
            });

        let enc_alg = Self::map_algorithm(spec);
        let tag_desc = Self::encode_tags(tags);

        let body = serde_json::json!({
            "projectId": self.project_id,
            "name": safe_id,
            "encryptionAlgorithm": enc_alg,
            "description": tag_desc
        });

        let req = self.client.post(self.keys_base_url()).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Infisical create key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical create key failed ({status}): {text}"
            )));
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
                format!("infisical-secret-{}", hex::encode(&buf))
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
            "projectId": self.project_id,
            "name": safe_id,
            "encryptionAlgorithm": "aes-256-gcm",
            "description": tag_desc
        });

        let req = self.client.post(self.keys_base_url()).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Infisical create envelope key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical create envelope key failed ({status}): {text}"
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

        // Fetch key metadata to read description tags
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

        // 2. Try direct export of private/symmetric key: GET /api/v1/kms/keys/{keyId}/private-key
        let priv_url = format!("{}/private-key", self.key_url(&safe_id));
        let req = self.client.get(&priv_url);
        let req = self.apply_auth(req).await;
        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(priv_b64) = body.get("privateKey").and_then(|v| v.as_str()) {
                        let raw = b64.decode(priv_b64).map_err(|e| {
                            ProxyError::ProviderError(format!("corrupt private key base64: {e}"))
                        })?;
                        return Ok((
                            KeyMaterialPayload::Symmetric(Zeroizing::new(raw)),
                            meta.tags,
                        ));
                    }
                }
            }
        }

        // 3. Try direct export of public key: GET /api/v1/kms/keys/{keyId}/public-key
        let pub_url = format!("{}/public-key", self.key_url(&safe_id));
        let req = self.client.get(&pub_url);
        let req = self.apply_auth(req).await;
        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(pub_pem) = body.get("publicKey").and_then(|v| v.as_str()) {
                        return Ok((
                            KeyMaterialPayload::PublicKeyDer(pub_pem.as_bytes().to_vec()),
                            meta.tags,
                        ));
                    }
                }
            }
        }

        Err(ProxyError::ItemNotFound(format!(
            "Infisical key material not found: {unique_id}"
        )))
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let req = self.client.get(self.key_url(&safe_id));
        let req = self.apply_auth(req).await;
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Infisical get key HTTP error: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ItemNotFound(format!(
                "Infisical key not found: {unique_id}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse key JSON: {e}")))?;

        let key_obj = body.get("key").unwrap_or(&body);
        let desc = key_obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tags = Self::decode_tags(desc);

        let is_disabled = key_obj
            .get("isDisabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let created_at = key_obj
            .get("createdAt")
            .and_then(|v| v.as_str())
            .and_then(Self::parse_created_time);

        Ok(CloudResourceMeta {
            unique_id: unique_id.to_string(),
            tags,
            native_state_enabled: !is_disabled,
            created_at_epoch_secs: created_at,
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

        let new_desc = Self::encode_tags(&meta.tags);
        let body = serde_json::json!({
            "description": new_desc
        });

        let req = self.client.patch(self.key_url(&safe_id)).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Infisical patch key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical update tags failed ({status}): {text}"
            )));
        }

        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let list_url = format!("{}/api/v1/kms/keys", self.site_url);
        let req = self.client.get(list_url).query(&[
            ("projectId", &self.project_id),
            ("limit", &"100".to_string()),
        ]);
        let req = self.apply_auth(req).await;
        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Infisical list keys HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse list JSON: {e}")))?;

        let keys = match body.get("keys").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();
        let limit = max_items.unwrap_or(u32::MAX) as usize;

        for key_val in keys {
            let key_id = key_val
                .get("name")
                .or_else(|| key_val.get("id"))
                .and_then(|v| v.as_str());

            if let Some(kid) = key_id {
                let desc = key_val
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

                if matched {
                    results.push(kid.to_string());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    async fn set_native_enabled_state(
        &self,
        unique_id: &str,
        enabled: bool,
    ) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let body = serde_json::json!({
            "isDisabled": !enabled
        });

        let req = self.client.patch(self.key_url(&safe_id)).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Infisical patch key state HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical set enabled state failed ({status}): {text}"
            )));
        }

        let mut update = CloudTagMap::new();
        update.insert(
            "kmip_state".to_string(),
            if enabled { "active" } else { "deactivated" }.to_string(),
        );
        let _ = self.update_tags(unique_id, &update, &[]).await;

        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let req = self.client.delete(self.key_url(&safe_id));
        let req = self.apply_auth(req).await;
        let resp = req.send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Infisical delete key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::NOT_FOUND {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical delete key failed ({status}): {text}"
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
        let pt_b64 = base64::engine::general_purpose::STANDARD.encode(plaintext);

        let enc_url = format!("{}/encrypt", self.key_url(&safe_id));
        let body = serde_json::json!({
            "plaintext": pt_b64
        });

        let req = self.client.post(&enc_url).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Infisical encrypt HTTP error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical encrypt failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse encrypt JSON: {e}")))?;

        let ct_str = body
            .get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("missing ciphertext in encrypt response".to_string())
            })?;

        Ok(ct_str.as_bytes().to_vec())
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let ct_str = std::str::from_utf8(ciphertext)
            .map_err(|e| ProxyError::ProviderError(format!("ciphertext not valid UTF-8: {e}")))?;

        let dec_url = format!("{}/decrypt", self.key_url(&safe_id));
        let body = serde_json::json!({
            "ciphertext": ct_str
        });

        let req = self.client.post(&dec_url).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Infisical decrypt HTTP error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical decrypt failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse decrypt JSON: {e}")))?;

        let pt_b64 = body
            .get("plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("missing plaintext in decrypt response".to_string())
            })?;

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(pt_b64)
            .map_err(|e| ProxyError::ProviderError(format!("corrupt plaintext base64: {e}")))?;

        Ok(Zeroizing::new(decoded))
    }

    async fn sign(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let input_b64 = base64::engine::general_purpose::STANDARD.encode(data);

        let sign_url = format!("{}/sign", self.key_url(&safe_id));
        let body = serde_json::json!({
            "data": input_b64,
            "signingAlgorithm": "rsa-pss-sha256"
        });

        let req = self.client.post(&sign_url).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Infisical sign HTTP error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Infisical sign failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse sign JSON: {e}")))?;

        let sig_b64 = body
            .get("signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("missing signature in sign response".to_string())
            })?;

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(sig_b64)
            .map_err(|e| ProxyError::ProviderError(format!("corrupt signature base64: {e}")))?;

        Ok(decoded)
    }

    async fn verify(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let input_b64 = base64::engine::general_purpose::STANDARD.encode(data);
        let sig_b64 = base64::engine::general_purpose::STANDARD.encode(signature);

        let verify_url = format!("{}/verify", self.key_url(&safe_id));
        let body = serde_json::json!({
            "data": input_b64,
            "signature": sig_b64,
            "signingAlgorithm": "rsa-pss-sha256"
        });

        let req = self.client.post(&verify_url).json(&body);
        let req = self.apply_auth(req).await;
        let resp = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Infisical verify HTTP error: {e}")))?;

        if !resp.status().is_success() {
            return Ok(false);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse verify JSON: {e}")))?;

        Ok(body
            .get("isValid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        crate::envelope::AutoEnvelope::generate_random_bytes(num_bytes)
    }
}

#[cfg(not(feature = "infisical"))]
pub struct InfisicalKmsProvider;

#[cfg(not(feature = "infisical"))]
impl InfisicalKmsProvider {
    pub fn new(_config: &InfisicalConfig) -> Result<Self, ProxyError> {
        Ok(Self)
    }

    pub fn from_config(
        _config: &InfisicalConfig,
        _master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        Ok(Self)
    }
}

#[cfg(not(feature = "infisical"))]
#[async_trait]
impl CloudKmsProvider for InfisicalKmsProvider {
    fn provider_name(&self) -> &'static str {
        "infisical"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Infisical feature not enabled".to_string(),
        ))
    }
}

mod hex {
    #[cfg(feature = "infisical")]
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
