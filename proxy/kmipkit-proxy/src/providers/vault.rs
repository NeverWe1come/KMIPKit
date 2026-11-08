//! HashiCorp Vault and OpenBao Transit and KV v2 provider connector.
#![forbid(unsafe_code)]

#[cfg(feature = "vault")]
use std::collections::BTreeMap;

use async_trait::async_trait;
#[cfg(feature = "vault")]
use base64::Engine;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::VaultConfig;
#[cfg(feature = "vault")]
use crate::envelope::{AutoEnvelope, WrappedEnvelopeKey};

#[cfg(feature = "vault")]
pub struct VaultKmsProvider {
    client: reqwest::Client,
    pub endpoint_url: String,
    pub namespace: Option<String>,
    pub transit_mount: String,
    pub kv_v2_mount: String,
    pub item_prefix: String,
    pub allow_transit_export: bool,
    pub envelope_master_kek_id: Option<String>,
    pub master_kek: Option<[u8; 32]>,
    pub auth_token: Option<String>,
}

#[cfg(feature = "vault")]
impl VaultKmsProvider {
    pub fn new(config: &VaultConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &VaultConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ProxyError::ProviderError(format!("failed to build HTTP client: {e}")))?;

        let endpoint_url = config
            .endpoint_url
            .clone()
            .unwrap_or_else(|| "http://127.0.0.1:8200".to_string())
            .trim_end_matches('/')
            .to_string();

        let auth_token = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("VAULT_TOKEN").ok());

        Ok(Self {
            client,
            endpoint_url,
            namespace: config.namespace.clone(),
            transit_mount: config
                .transit_mount
                .clone()
                .unwrap_or_else(|| "transit".to_string()),
            kv_v2_mount: config
                .kv_v2_mount
                .clone()
                .unwrap_or_else(|| "secret".to_string()),
            item_prefix: config
                .item_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            allow_transit_export: config.allow_transit_export.unwrap_or(true),
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
            master_kek,
            auth_token,
        })
    }

    fn normalize_id(&self, unique_id: &str) -> String {
        unique_id.replace('/', "-")
    }

    fn apply_headers(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut r = req;
        if let Some(ref token) = self.auth_token {
            r = r.header("X-Vault-Token", token);
        }
        if let Some(ref ns) = self.namespace {
            r = r.header("X-Vault-Namespace", ns);
        }
        r
    }

    fn clean_tags(tags: &CloudTagMap) -> BTreeMap<String, String> {
        let mut clean = BTreeMap::new();
        for (k, v) in &tags.0 {
            clean.insert(k.clone(), v.clone());
        }
        clean
    }

    fn transit_key_url(&self, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/keys/{}",
            self.endpoint_url, self.transit_mount, safe_id
        )
    }

    fn transit_export_url(&self, export_type: &str, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/export/{}/{}",
            self.endpoint_url, self.transit_mount, export_type, safe_id
        )
    }

    fn transit_encrypt_url(&self, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/encrypt/{}",
            self.endpoint_url, self.transit_mount, safe_id
        )
    }

    fn transit_decrypt_url(&self, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/decrypt/{}",
            self.endpoint_url, self.transit_mount, safe_id
        )
    }

    fn transit_sign_url(&self, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/sign/{}",
            self.endpoint_url, self.transit_mount, safe_id
        )
    }

    fn transit_verify_url(&self, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/verify/{}",
            self.endpoint_url, self.transit_mount, safe_id
        )
    }

    fn transit_random_url(&self, num_bytes: usize) -> String {
        format!(
            "{}/v1/{}/random/{}",
            self.endpoint_url, self.transit_mount, num_bytes
        )
    }

    fn kv_metadata_url(&self, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/metadata/{}{}",
            self.endpoint_url, self.kv_v2_mount, self.item_prefix, safe_id
        )
    }

    fn kv_metadata_list_url(&self) -> String {
        format!(
            "{}/v1/{}/metadata/{}",
            self.endpoint_url,
            self.kv_v2_mount,
            self.item_prefix.trim_end_matches('/')
        )
    }

    fn kv_data_url(&self, safe_id: &str) -> String {
        format!(
            "{}/v1/{}/data/{}{}",
            self.endpoint_url, self.kv_v2_mount, self.item_prefix, safe_id
        )
    }

    fn map_key_type(spec: &CreateKeySpec) -> &'static str {
        match spec.algorithm.to_ascii_uppercase().as_str() {
            "RSA" => match spec.length {
                3072 => "rsa-3072",
                4096 => "rsa-4096",
                _ => "rsa-2048",
            },
            "ECDSA" | "EC" => match spec.length {
                384 => "ecdsa-p384",
                521 => "ecdsa-p521",
                _ => "ecdsa-p256",
            },
            "ED25519" => "ed25519",
            "HMAC" => "hmac",
            _ => match spec.length {
                128 => "aes128-gcm96",
                _ => "aes256-gcm96",
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

#[cfg(feature = "vault")]
#[async_trait]
impl CloudKmsProvider for VaultKmsProvider {
    fn provider_name(&self) -> &'static str {
        "vault"
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
                format!("vault-key-{}", hex::encode(&buf))
            });

        let key_type = Self::map_key_type(spec);
        let transit_body = serde_json::json!({
            "type": key_type,
            "exportable": self.allow_transit_export,
            "derived": false
        });

        let req = self
            .client
            .post(self.transit_key_url(&safe_id))
            .json(&transit_body);
        let resp = self.apply_headers(req).send().await.map_err(|e| {
            ProxyError::ProviderError(format!("Transit create key HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Transit create key failed ({status}): {text}"
            )));
        }

        // Store tags in KV v2 metadata
        let clean = Self::clean_tags(tags);
        let meta_body = serde_json::json!({
            "custom_metadata": clean
        });

        let req = self
            .client
            .post(self.kv_metadata_url(&safe_id))
            .json(&meta_body);
        let _ = self.apply_headers(req).send().await;

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
                format!("vault-secret-{}", hex::encode(&buf))
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

        let kv_data_body = serde_json::json!({
            "data": {
                "ciphertext": wrapped_b64,
                "algorithm": spec.algorithm,
                "length": spec.length
            }
        });

        let req = self
            .client
            .post(self.kv_data_url(&safe_id))
            .json(&kv_data_body);
        let resp =
            self.apply_headers(req).send().await.map_err(|e| {
                ProxyError::ProviderError(format!("KV v2 data write HTTP error: {e}"))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "KV v2 data write failed ({status}): {text}"
            )));
        }

        // Store tags in KV v2 metadata
        let clean = Self::clean_tags(tags);
        let meta_body = serde_json::json!({
            "custom_metadata": clean
        });

        let req = self
            .client
            .post(self.kv_metadata_url(&safe_id))
            .json(&meta_body);
        let _ = self.apply_headers(req).send().await;

        Ok(safe_id)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let b64 = &base64::engine::general_purpose::STANDARD;

        // Fetch metadata first for tags
        let meta_req = self.client.get(self.kv_metadata_url(&safe_id));
        let meta_resp = self.apply_headers(meta_req).send().await;
        let mut tags = CloudTagMap::new();

        if let Ok(resp) = meta_resp {
            if resp.status().is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    if let Some(custom) = body
                        .pointer("/data/custom_metadata")
                        .and_then(|v| v.as_object())
                    {
                        for (k, v) in custom {
                            if let Some(s) = v.as_str() {
                                tags.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }

        // 1. Try reading from KV v2 data (Envelope Key)
        let data_req = self.client.get(self.kv_data_url(&safe_id));
        if let Ok(data_resp) = self.apply_headers(data_req).send().await {
            if data_resp.status().is_success() {
                if let Ok(body) = data_resp.json::<serde_json::Value>().await {
                    if let Some(ct_b64) = body
                        .pointer("/data/data/ciphertext")
                        .and_then(|v| v.as_str())
                    {
                        let ct_bytes = b64.decode(ct_b64).map_err(|e| {
                            ProxyError::ProviderError(format!("corrupt ciphertext base64: {e}"))
                        })?;
                        let wrapped = WrappedEnvelopeKey::from_bytes(&ct_bytes)?;
                        let master = self.master_kek.unwrap_or([0x5au8; 32]);
                        let decrypted = AutoEnvelope::unwrap_key(&master, &wrapped)?;
                        return Ok((KeyMaterialPayload::Symmetric(decrypted), tags));
                    }
                }
            }
        }

        // 2. Try Transit Export (Direct Exportable Key)
        if self.allow_transit_export {
            for export_type in &["encryption-key", "signing-key", "hmac-key"] {
                let exp_req = self
                    .client
                    .get(self.transit_export_url(export_type, &safe_id));
                if let Ok(exp_resp) = self.apply_headers(exp_req).send().await {
                    if exp_resp.status().is_success() {
                        if let Ok(body) = exp_resp.json::<serde_json::Value>().await {
                            if let Some(keys_map) =
                                body.pointer("/data/keys").and_then(|v| v.as_object())
                            {
                                // Extract the highest version key
                                if let Some((_, val)) = keys_map.iter().next_back() {
                                    if let Some(raw_b64) = val.as_str() {
                                        let raw_bytes = b64.decode(raw_b64).map_err(|e| {
                                            ProxyError::ProviderError(format!(
                                                "export base64 error: {e}"
                                            ))
                                        })?;
                                        return Ok((
                                            KeyMaterialPayload::Symmetric(Zeroizing::new(
                                                raw_bytes,
                                            )),
                                            tags,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Try Transit Get Public Key (Asymmetric)
        let key_req = self.client.get(self.transit_key_url(&safe_id));
        let key_resp =
            self.apply_headers(key_req).send().await.map_err(|e| {
                ProxyError::ProviderError(format!("Transit get key HTTP error: {e}"))
            })?;

        if key_resp.status().is_success() {
            let body: serde_json::Value = key_resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse Transit key JSON: {e}"))
            })?;

            if let Some(keys_map) = body.pointer("/data/keys").and_then(|v| v.as_object()) {
                if let Some((_, val)) = keys_map.iter().next_back() {
                    if let Some(pub_pem) = val.get("public_key").and_then(|v| v.as_str()) {
                        return Ok((
                            KeyMaterialPayload::PublicKeyDer(pub_pem.as_bytes().to_vec()),
                            tags,
                        ));
                    }
                }
            }
        }

        Err(ProxyError::ItemNotFound(format!(
            "Vault resource not found or export denied: {unique_id}"
        )))
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let safe_id = self.normalize_id(unique_id);
        let mut tags = CloudTagMap::new();
        let mut created_at = None;

        let meta_req = self.client.get(self.kv_metadata_url(&safe_id));
        if let Ok(meta_resp) = self.apply_headers(meta_req).send().await {
            if meta_resp.status().is_success() {
                if let Ok(body) = meta_resp.json::<serde_json::Value>().await {
                    if let Some(created_str) =
                        body.pointer("/data/created_time").and_then(|v| v.as_str())
                    {
                        created_at = Self::parse_created_time(created_str);
                    }
                    if let Some(custom) = body
                        .pointer("/data/custom_metadata")
                        .and_then(|v| v.as_object())
                    {
                        for (k, v) in custom {
                            if let Some(s) = v.as_str() {
                                tags.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Determine enabled state
        let mut native_state_enabled = true;
        let key_req = self.client.get(self.transit_key_url(&safe_id));
        if let Ok(key_resp) = self.apply_headers(key_req).send().await {
            if key_resp.status().is_success() {
                if let Ok(body) = key_resp.json::<serde_json::Value>().await {
                    if let Some(min_enc) = body
                        .pointer("/data/min_encryption_version")
                        .and_then(|v| v.as_u64())
                    {
                        if min_enc >= 999_999 {
                            native_state_enabled = false;
                        }
                    }
                }
            }
        }

        if let Some(state_tag) = tags.get("kmip_state") {
            if state_tag == "deactivated" {
                native_state_enabled = false;
            }
        }

        Ok(CloudResourceMeta {
            unique_id: unique_id.to_string(),
            tags,
            native_state_enabled,
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
        let meta_req = self.client.get(self.kv_metadata_url(&safe_id));
        let mut clean = BTreeMap::new();

        if let Ok(meta_resp) = self.apply_headers(meta_req).send().await {
            if meta_resp.status().is_success() {
                if let Ok(body) = meta_resp.json::<serde_json::Value>().await {
                    if let Some(custom) = body
                        .pointer("/data/custom_metadata")
                        .and_then(|v| v.as_object())
                    {
                        for (k, v) in custom {
                            if let Some(s) = v.as_str() {
                                clean.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }

        for (k, v) in &upsert.0 {
            clean.insert(k.clone(), v.clone());
        }
        for k in remove_keys {
            clean.remove(k);
        }

        let meta_body = serde_json::json!({
            "custom_metadata": clean
        });

        let req = self
            .client
            .post(self.kv_metadata_url(&safe_id))
            .json(&meta_body);
        let resp =
            self.apply_headers(req).send().await.map_err(|e| {
                ProxyError::ProviderError(format!("KV v2 update tags HTTP error: {e}"))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "KV v2 update tags failed ({status}): {text}"
            )));
        }

        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let list_url = format!("{}?list=true", self.kv_metadata_list_url());
        let req = self.client.get(list_url);
        let resp = self.apply_headers(req).send().await.map_err(|e| {
            ProxyError::ProviderError(format!("KV v2 list metadata HTTP error: {e}"))
        })?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse list JSON: {e}")))?;

        let keys = match body.pointer("/data/keys").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::new();
        let limit = max_items.unwrap_or(u32::MAX) as usize;

        for key_val in keys {
            if let Some(key_name) = key_val.as_str() {
                let meta = self.get_tags_and_metadata(key_name).await?;
                let mut matched = true;
                for (fk, fv) in &filter.exact_tags {
                    if meta.tags.get(fk) != Some(fv) {
                        matched = false;
                        break;
                    }
                }
                if matched {
                    results.push(key_name.to_string());
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
        let min_enc = if enabled { 0 } else { 999_999 };

        let cfg_body = serde_json::json!({
            "min_encryption_version": min_enc
        });

        let config_url = format!("{}/config", self.transit_key_url(&safe_id));
        let req = self.client.post(config_url).json(&cfg_body);
        let _ = self.apply_headers(req).send().await;

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

        // Enable deletion in transit config
        let cfg_body = serde_json::json!({
            "deletion_allowed": true
        });
        let config_url = format!("{}/config", self.transit_key_url(&safe_id));
        let req = self.client.post(config_url).json(&cfg_body);
        let _ = self.apply_headers(req).send().await;

        // Delete transit key
        let del_transit_req = self.client.delete(self.transit_key_url(&safe_id));
        let _ = self.apply_headers(del_transit_req).send().await;

        // Delete KV v2 data
        let del_data_req = self.client.delete(self.kv_data_url(&safe_id));
        let _ = self.apply_headers(del_data_req).send().await;

        // Delete KV v2 metadata
        let del_meta_req = self.client.delete(self.kv_metadata_url(&safe_id));
        let _ = self.apply_headers(del_meta_req).send().await;

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

        let enc_body = serde_json::json!({
            "plaintext": pt_b64
        });

        let req = self
            .client
            .post(self.transit_encrypt_url(&safe_id))
            .json(&enc_body);
        let resp =
            self.apply_headers(req).send().await.map_err(|e| {
                ProxyError::ProviderError(format!("Transit encrypt HTTP error: {e}"))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Transit encrypt failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse encrypt JSON: {e}")))?;

        let ciphertext_str = body
            .pointer("/data/ciphertext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("missing ciphertext in Transit response".to_string())
            })?;

        Ok(ciphertext_str.as_bytes().to_vec())
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

        let dec_body = serde_json::json!({
            "ciphertext": ct_str
        });

        let req = self
            .client
            .post(self.transit_decrypt_url(&safe_id))
            .json(&dec_body);
        let resp =
            self.apply_headers(req).send().await.map_err(|e| {
                ProxyError::ProviderError(format!("Transit decrypt HTTP error: {e}"))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Transit decrypt failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse decrypt JSON: {e}")))?;

        let pt_b64 = body
            .pointer("/data/plaintext")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("missing plaintext in Transit response".to_string())
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

        let sign_body = serde_json::json!({
            "input": input_b64,
            "hash_algorithm": "sha2-256"
        });

        let req = self
            .client
            .post(self.transit_sign_url(&safe_id))
            .json(&sign_body);
        let resp = self
            .apply_headers(req)
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Transit sign HTTP error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Transit sign failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse sign JSON: {e}")))?;

        let sig_str = body
            .pointer("/data/signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("missing signature in Transit response".to_string())
            })?;

        Ok(sig_str.as_bytes().to_vec())
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
        let sig_str = std::str::from_utf8(signature)
            .map_err(|e| ProxyError::ProviderError(format!("signature not valid UTF-8: {e}")))?;

        let verify_body = serde_json::json!({
            "input": input_b64,
            "signature": sig_str,
            "hash_algorithm": "sha2-256"
        });

        let req = self
            .client
            .post(self.transit_verify_url(&safe_id))
            .json(&verify_body);
        let resp =
            self.apply_headers(req).send().await.map_err(|e| {
                ProxyError::ProviderError(format!("Transit verify HTTP error: {e}"))
            })?;

        if !resp.status().is_success() {
            return Ok(false);
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse verify JSON: {e}")))?;

        Ok(body
            .pointer("/data/valid")
            .and_then(|v| v.as_bool())
            .unwrap_or(false))
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let req = self
            .client
            .post(self.transit_random_url(num_bytes))
            .json(&serde_json::json!({"format": "base64"}));
        let resp =
            self.apply_headers(req).send().await.map_err(|e| {
                ProxyError::ProviderError(format!("Transit random HTTP error: {e}"))
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Transit random failed ({status}): {text}"
            )));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("failed to parse random JSON: {e}")))?;

        let rand_b64 = body
            .pointer("/data/random_bytes")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ProxyError::ProviderError("missing random_bytes in Transit response".to_string())
            })?;

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(rand_b64)
            .map_err(|e| ProxyError::ProviderError(format!("corrupt random_bytes base64: {e}")))?;

        Ok(Zeroizing::new(decoded))
    }
}

#[cfg(not(feature = "vault"))]
pub struct VaultKmsProvider;

#[cfg(not(feature = "vault"))]
impl VaultKmsProvider {
    pub fn new(_config: &VaultConfig) -> Result<Self, ProxyError> {
        Ok(Self)
    }

    pub fn from_config(
        _config: &VaultConfig,
        _master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        Ok(Self)
    }
}

#[cfg(not(feature = "vault"))]
#[async_trait]
impl CloudKmsProvider for VaultKmsProvider {
    fn provider_name(&self) -> &'static str {
        "vault"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
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
            "Vault feature not enabled".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Vault feature not enabled".to_string(),
        ))
    }
}

mod hex {
    #[cfg(feature = "vault")]
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
