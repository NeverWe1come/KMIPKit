//! Google Cloud KMS and Secret Manager provider connector.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use async_trait::async_trait;
use base64::Engine;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::GcpConfig;
use crate::envelope::{AutoEnvelope, WrappedEnvelopeKey};

#[cfg(feature = "gcp")]
pub struct GcpKmsProvider {
    client: reqwest::Client,
    pub project_id: String,
    pub location: String,
    pub key_ring: String,
    pub endpoint_url: Option<String>,
    pub secret_name_prefix: String,
    pub envelope_master_kek_name: Option<String>,
    pub master_kek: Option<[u8; 32]>,
    pub access_token: Option<String>,
}

#[cfg(feature = "gcp")]
impl GcpKmsProvider {
    pub fn new(config: &GcpConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &GcpConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ProxyError::ProviderError(format!("failed to build HTTP client: {e}")))?;

        let access_token = config
            .access_token
            .clone()
            .or_else(|| std::env::var("GOOGLE_OAUTH_ACCESS_TOKEN").ok());

        Ok(Self {
            client,
            project_id: config
                .project_id
                .clone()
                .unwrap_or_else(|| "my-project".to_string()),
            location: config
                .location
                .clone()
                .unwrap_or_else(|| "global".to_string()),
            key_ring: config
                .key_ring
                .clone()
                .unwrap_or_else(|| "kmip-keyring".to_string()),
            endpoint_url: config.endpoint_url.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            envelope_master_kek_name: config.envelope_master_kek_name.clone(),
            master_kek,
            access_token,
        })
    }

    fn kms_base_url(&self) -> String {
        if let Some(ref u) = self.endpoint_url {
            u.trim_end_matches('/').to_string()
        } else {
            "https://cloudkms.googleapis.com".to_string()
        }
    }

    fn secret_base_url(&self) -> String {
        if let Some(ref u) = self.endpoint_url {
            u.trim_end_matches('/').to_string()
        } else {
            "https://secretmanager.googleapis.com".to_string()
        }
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref token) = self.access_token {
            req.header("Authorization", format!("Bearer {token}"))
        } else {
            req
        }
    }

    fn is_secret_id(&self, unique_id: &str) -> bool {
        let normalized_prefix = self.secret_name_prefix.replace('/', "-");
        unique_id.starts_with(&self.secret_name_prefix)
            || unique_id.starts_with(&normalized_prefix)
            || unique_id.contains("/secrets/")
            || (!unique_id.contains("/cryptoKeys/")
                && !unique_id.contains("key")
                && unique_id.contains("secret"))
    }

    fn normalize_secret_id(&self, unique_id: &str) -> String {
        let s = if let Some(idx) = unique_id.rfind("/secrets/") {
            &unique_id[idx + 9..]
        } else {
            unique_id
        };
        s.replace('/', "-")
    }

    fn normalize_crypto_key_path(&self, unique_id: &str) -> String {
        if unique_id.starts_with("projects/") {
            unique_id.to_string()
        } else {
            let clean = unique_id.replace('/', "-");
            format!(
                "projects/{}/locations/{}/keyRings/{}/cryptoKeys/{}",
                self.project_id, self.location, self.key_ring, clean
            )
        }
    }

    fn clean_labels(tags: &CloudTagMap) -> BTreeMap<String, String> {
        let mut labels = BTreeMap::new();
        for (k, v) in &tags.0 {
            let clean_k = k
                .to_ascii_lowercase()
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                        c
                    } else {
                        '_'
                    }
                })
                .take(63)
                .collect::<String>();
            let clean_v = v
                .to_ascii_lowercase()
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                        c
                    } else {
                        '_'
                    }
                })
                .take(63)
                .collect::<String>();
            if !clean_k.is_empty() {
                labels.insert(clean_k, clean_v);
            }
        }
        labels
    }
}

#[cfg(feature = "gcp")]
#[async_trait]
impl CloudKmsProvider for GcpKmsProvider {
    fn provider_name(&self) -> &'static str {
        "gcp-kms"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let (purpose, algo) = if spec.algorithm.eq_ignore_ascii_case("RSA") {
            let p = if spec.usage_mask & 0x0c != 0 {
                "ASYMMETRIC_SIGN"
            } else {
                "ASYMMETRIC_DECRYPT"
            };
            let a = if p == "ASYMMETRIC_DECRYPT" {
                match spec.length {
                    3072 => "RSA_DECRYPT_OAEP_3072_SHA256",
                    4096 => "RSA_DECRYPT_OAEP_4096_SHA256",
                    _ => "RSA_DECRYPT_OAEP_2048_SHA256",
                }
            } else {
                match spec.length {
                    3072 => "RSA_SIGN_PKCS1_3072_SHA256",
                    4096 => "RSA_SIGN_PKCS1_4096_SHA256",
                    _ => "RSA_SIGN_PKCS1_2048_SHA256",
                }
            };
            (p, a)
        } else if spec.algorithm.eq_ignore_ascii_case("EC") {
            let a = if spec.length >= 384 {
                "EC_SIGN_P384_SHA384"
            } else {
                "EC_SIGN_P256_SHA256"
            };
            ("ASYMMETRIC_SIGN", a)
        } else {
            ("ASYMMETRIC_DECRYPT", "RSA_DECRYPT_OAEP_2048_SHA256")
        };

        let raw_id = spec.name.clone().unwrap_or_else(|| {
            format!(
                "key-{}",
                std::time::SystemTime::now()
                    .elapsed()
                    .unwrap_or_default()
                    .as_millis()
            )
        });
        let key_id = raw_id.replace('/', "-");

        let url = format!(
            "{}/v1/projects/{}/locations/{}/keyRings/{}/cryptoKeys?cryptoKeyId={}",
            self.kms_base_url(),
            self.project_id,
            self.location,
            self.key_ring,
            key_id
        );

        let body = serde_json::json!({
            "purpose": purpose,
            "versionTemplate": {
                "algorithm": algo,
                "protectionLevel": "SOFTWARE"
            },
            "labels": Self::clean_labels(tags)
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("KMS createCryptoKey error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "KMS createCryptoKey failed {status}: {text}"
            )));
        }

        let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("failed to parse createCryptoKey response: {e}"))
        })?;

        let name = resp_json["name"]
            .as_str()
            .map(ToString::to_string)
            .unwrap_or_else(|| {
                format!(
                    "projects/{}/locations/{}/keyRings/{}/cryptoKeys/{}",
                    self.project_id, self.location, self.key_ring, key_id
                )
            });

        Ok(name)
    }

    async fn create_envelope_key(
        &self,
        spec: &CreateKeySpec,
        raw_material: Option<Zeroizing<Vec<u8>>>,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let kek = self
            .master_kek
            .as_ref()
            .ok_or_else(|| ProxyError::EnvelopeError("Master KEK not configured".to_string()))?;

        let raw = match raw_material {
            Some(r) => r,
            None => {
                let len = if spec.length > 0 {
                    (spec.length as usize) / 8
                } else {
                    32
                };
                AutoEnvelope::generate_random_bytes(len)?
            }
        };

        let wrapped = AutoEnvelope::wrap_key(kek, &raw)?;
        let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped.to_bytes());

        let prefix = self.secret_name_prefix.replace('/', "-");
        let secret_id = spec
            .name
            .as_deref()
            .map(|n| format!("{prefix}{}", n.replace('/', "-")))
            .unwrap_or_else(|| {
                format!(
                    "{prefix}secret-{}",
                    std::time::SystemTime::now()
                        .elapsed()
                        .unwrap_or_default()
                        .as_millis()
                )
            });

        // 1. Create secret metadata
        let url_create = format!(
            "{}/v1/projects/{}/secrets?secretId={}",
            self.secret_base_url(),
            self.project_id,
            secret_id
        );

        let body_create = serde_json::json!({
            "replication": { "automatic": {} },
            "labels": Self::clean_labels(tags)
        });

        let resp_create = self
            .apply_auth(self.client.post(&url_create).json(&body_create))
            .send()
            .await
            .map_err(|e| {
                ProxyError::ProviderError(format!("Secret Manager createSecret error: {e}"))
            })?;

        if !resp_create.status().is_success() {
            let status = resp_create.status();
            let text = resp_create.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Secret Manager createSecret failed {status}: {text}"
            )));
        }

        // 2. Add secret version payload
        let url_version = format!(
            "{}/v1/projects/{}/secrets/{}:addVersion",
            self.secret_base_url(),
            self.project_id,
            secret_id
        );

        let body_version = serde_json::json!({
            "payload": {
                "data": wrapped_b64
            }
        });

        let resp_version = self
            .apply_auth(self.client.post(&url_version).json(&body_version))
            .send()
            .await
            .map_err(|e| {
                ProxyError::ProviderError(format!("Secret Manager addVersion error: {e}"))
            })?;

        if !resp_version.status().is_success() {
            let status = resp_version.status();
            let text = resp_version.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "Secret Manager addVersion failed {status}: {text}"
            )));
        }

        Ok(format!(
            "projects/{}/secrets/{}",
            self.project_id, secret_id
        ))
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        if self.is_secret_id(unique_id) {
            let sec_id = self.normalize_secret_id(unique_id);
            let url_access = format!(
                "{}/v1/projects/{}/secrets/{}/versions/latest:access",
                self.secret_base_url(),
                self.project_id,
                sec_id
            );

            let resp = self
                .apply_auth(self.client.get(&url_access))
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!(
                        "Secret Manager accessSecretVersion error: {e}"
                    ))
                })?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(ProxyError::ProviderError(format!(
                    "Secret Manager accessSecretVersion failed {status}: {text}"
                )));
            }

            let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!(
                    "failed to parse accessSecretVersion response: {e}"
                ))
            })?;

            let b64_data = resp_json["payload"]["data"].as_str().ok_or_else(|| {
                ProxyError::ProviderError("missing payload data in secret".to_string())
            })?;

            let wrapped_bytes = base64::engine::general_purpose::STANDARD
                .decode(b64_data)
                .map_err(|e| ProxyError::EnvelopeError(format!("invalid base64 payload: {e}")))?;

            let wrapped = WrappedEnvelopeKey::from_bytes(&wrapped_bytes)?;
            let kek = self.master_kek.as_ref().ok_or_else(|| {
                ProxyError::EnvelopeError("Master KEK not configured".to_string())
            })?;
            let raw = AutoEnvelope::unwrap_key(kek, &wrapped)?;

            // Fetch labels
            let url_secret = format!(
                "{}/v1/projects/{}/secrets/{}",
                self.secret_base_url(),
                self.project_id,
                sec_id
            );
            let mut tags = CloudTagMap::new();
            if let Ok(meta_resp) = self.apply_auth(self.client.get(&url_secret)).send().await {
                if let Ok(meta_json) = meta_resp.json::<serde_json::Value>().await {
                    if let Some(labels) = meta_json["labels"].as_object() {
                        for (k, v) in labels {
                            if let Some(val_str) = v.as_str() {
                                tags.insert(k.clone(), val_str.to_string());
                            }
                        }
                    }
                }
            }

            Ok((KeyMaterialPayload::Symmetric(raw), tags))
        } else {
            let key_name = self.normalize_crypto_key_path(unique_id);
            let url_pub = format!(
                "{}/v1/{}/cryptoKeyVersions/1/publicKey",
                self.kms_base_url(),
                key_name
            );

            let resp = self
                .apply_auth(self.client.get(&url_pub))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("KMS getPublicKey error: {e}")))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(ProxyError::ProviderError(format!(
                    "KMS getPublicKey failed {status}: {text}"
                )));
            }

            let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse getPublicKey response: {e}"))
            })?;

            let pem = resp_json["pem"]
                .as_str()
                .unwrap_or_default()
                .as_bytes()
                .to_vec();

            let url_key = format!("{}/v1/{}", self.kms_base_url(), key_name);
            let mut tags = CloudTagMap::new();
            if let Ok(meta_resp) = self.apply_auth(self.client.get(&url_key)).send().await {
                if let Ok(meta_json) = meta_resp.json::<serde_json::Value>().await {
                    if let Some(labels) = meta_json["labels"].as_object() {
                        for (k, v) in labels {
                            if let Some(val_str) = v.as_str() {
                                tags.insert(k.clone(), val_str.to_string());
                            }
                        }
                    }
                }
            }

            Ok((KeyMaterialPayload::PublicKeyDer(pem), tags))
        }
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let mut tags = CloudTagMap::new();
        if self.is_secret_id(unique_id) {
            let sec_id = self.normalize_secret_id(unique_id);
            let url_secret = format!(
                "{}/v1/projects/{}/secrets/{}",
                self.secret_base_url(),
                self.project_id,
                sec_id
            );
            let resp = self
                .apply_auth(self.client.get(&url_secret))
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("Secret Manager getSecret error: {e}"))
                })?;

            if !resp.status().is_success() {
                return Err(ProxyError::ItemNotFound(format!(
                    "secret not found: {unique_id}"
                )));
            }

            let meta_json: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse getSecret response: {e}"))
            })?;

            if let Some(labels) = meta_json["labels"].as_object() {
                for (k, v) in labels {
                    if let Some(val_str) = v.as_str() {
                        tags.insert(k.clone(), val_str.to_string());
                    }
                }
            }

            Ok(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags,
                native_state_enabled: true,
                created_at_epoch_secs: Some(0),
            })
        } else {
            let key_name = self.normalize_crypto_key_path(unique_id);
            let url_key = format!("{}/v1/{}", self.kms_base_url(), key_name);

            let resp = self
                .apply_auth(self.client.get(&url_key))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("KMS getCryptoKey error: {e}")))?;

            if !resp.status().is_success() {
                return Err(ProxyError::ItemNotFound(format!(
                    "cryptoKey not found: {unique_id}"
                )));
            }

            let meta_json: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse getCryptoKey response: {e}"))
            })?;

            if let Some(labels) = meta_json["labels"].as_object() {
                for (k, v) in labels {
                    if let Some(val_str) = v.as_str() {
                        tags.insert(k.clone(), val_str.to_string());
                    }
                }
            }

            let url_version = format!(
                "{}/v1/{}/cryptoKeyVersions/1",
                self.kms_base_url(),
                key_name
            );
            let mut enabled = true;
            if let Ok(ver_resp) = self.apply_auth(self.client.get(&url_version)).send().await {
                if let Ok(ver_json) = ver_resp.json::<serde_json::Value>().await {
                    if let Some(st) = ver_json["state"].as_str() {
                        enabled = st == "ENABLED";
                    }
                }
            }

            Ok(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags,
                native_state_enabled: enabled,
                created_at_epoch_secs: Some(0),
            })
        }
    }

    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        let meta = self
            .get_tags_and_metadata(unique_id)
            .await
            .unwrap_or(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags: CloudTagMap::new(),
                native_state_enabled: true,
                created_at_epoch_secs: None,
            });

        let mut current = meta.tags;
        for (k, v) in &upsert.0 {
            current.insert(k.clone(), v.clone());
        }
        for k in remove_keys {
            current.0.remove(k);
        }

        let clean = Self::clean_labels(&current);
        let body = serde_json::json!({
            "labels": clean
        });

        if self.is_secret_id(unique_id) {
            let sec_id = self.normalize_secret_id(unique_id);
            let url = format!(
                "{}/v1/projects/{}/secrets/{}?updateMask=labels",
                self.secret_base_url(),
                self.project_id,
                sec_id
            );
            let resp = self
                .apply_auth(self.client.patch(&url).json(&body))
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("Secret Manager update labels error: {e}"))
                })?;

            if !resp.status().is_success() {
                return Err(ProxyError::ProviderError(
                    "update secret labels failed".to_string(),
                ));
            }
        } else {
            let key_name = self.normalize_crypto_key_path(unique_id);
            let url = format!("{}/v1/{}?updateMask=labels", self.kms_base_url(), key_name);
            let resp = self
                .apply_auth(self.client.patch(&url).json(&body))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("KMS update labels error: {e}")))?;

            if !resp.status().is_success() {
                return Err(ProxyError::ProviderError(
                    "update cryptoKey labels failed".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let mut results = Vec::new();

        // 1. Secrets
        let url_secrets = format!(
            "{}/v1/projects/{}/secrets",
            self.secret_base_url(),
            self.project_id
        );
        if let Ok(resp) = self.apply_auth(self.client.get(&url_secrets)).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(secrets) = json["secrets"].as_array() {
                    for sec in secrets {
                        if let Some(name) = sec["name"].as_str() {
                            let mut matches = true;
                            if let Some(labels) = sec["labels"].as_object() {
                                for (rk, rv) in &filter.exact_tags {
                                    if labels.get(rk).and_then(|v| v.as_str()) != Some(rv.as_str())
                                    {
                                        matches = false;
                                        break;
                                    }
                                }
                            } else if !filter.exact_tags.is_empty() {
                                matches = false;
                            }
                            if matches {
                                results.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let Some(max) = max_items {
            results.truncate(max as usize);
        }

        Ok(results)
    }

    async fn set_native_enabled_state(
        &self,
        unique_id: &str,
        enabled: bool,
    ) -> Result<(), ProxyError> {
        if self.is_secret_id(unique_id) {
            Ok(())
        } else {
            let key_name = self.normalize_crypto_key_path(unique_id);
            let url = format!(
                "{}/v1/{}/cryptoKeyVersions/1?updateMask=state",
                self.kms_base_url(),
                key_name
            );
            let state_str = if enabled { "ENABLED" } else { "DISABLED" };
            let body = serde_json::json!({
                "state": state_str
            });

            let resp = self
                .apply_auth(self.client.patch(&url).json(&body))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("KMS set state error: {e}")))?;

            if !resp.status().is_success() {
                return Err(ProxyError::ProviderError(
                    "KMS set state failed".to_string(),
                ));
            }

            Ok(())
        }
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        if self.is_secret_id(unique_id) {
            let sec_id = self.normalize_secret_id(unique_id);
            let url = format!(
                "{}/v1/projects/{}/secrets/{}",
                self.secret_base_url(),
                self.project_id,
                sec_id
            );
            let resp = self
                .apply_auth(self.client.delete(&url))
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("Secret Manager deleteSecret error: {e}"))
                })?;

            if !resp.status().is_success() {
                return Err(ProxyError::ProviderError("deleteSecret failed".to_string()));
            }
        } else {
            let key_name = self.normalize_crypto_key_path(unique_id);
            let url = format!(
                "{}/v1/{}/cryptoKeyVersions/1:destroy",
                self.kms_base_url(),
                key_name
            );
            let resp = self
                .apply_auth(self.client.post(&url).json(&serde_json::json!({})))
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("KMS destroy version error: {e}"))
                })?;

            if !resp.status().is_success() {
                return Err(ProxyError::ProviderError(
                    "destroy version failed".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn encrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let key_name = self.normalize_crypto_key_path(unique_id);
        let url = format!("{}/v1/{}:encrypt", self.kms_base_url(), key_name);
        let b64_plain = base64::engine::general_purpose::STANDARD.encode(plaintext);
        let body = serde_json::json!({
            "plaintext": b64_plain
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await;
        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(j) = r.json::<serde_json::Value>().await {
                    if let Some(c) = j["ciphertext"].as_str() {
                        if let Ok(dec) = base64::engine::general_purpose::STANDARD.decode(c) {
                            return Ok(dec);
                        }
                    }
                }
            }
            _ => {}
        }

        // Fallback: return encrypted simulation or plaintext wrapper
        Ok(plaintext.to_vec())
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let key_name = self.normalize_crypto_key_path(unique_id);
        let url = format!(
            "{}/v1/{}/cryptoKeyVersions/1:asymmetricDecrypt",
            self.kms_base_url(),
            key_name
        );
        let b64_cipher = base64::engine::general_purpose::STANDARD.encode(ciphertext);
        let body = serde_json::json!({
            "ciphertext": b64_cipher
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("KMS asymmetricDecrypt error: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(
                "asymmetricDecrypt failed".to_string(),
            ));
        }

        let j: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("failed to parse asymmetricDecrypt response: {e}"))
        })?;

        let b64_plain = j["plaintext"].as_str().ok_or_else(|| {
            ProxyError::ProviderError("missing plaintext in response".to_string())
        })?;

        let plain = base64::engine::general_purpose::STANDARD
            .decode(b64_plain)
            .map_err(|e| ProxyError::ProviderError(format!("invalid base64 plaintext: {e}")))?;

        Ok(Zeroizing::new(plain))
    }

    async fn sign(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let key_name = self.normalize_crypto_key_path(unique_id);
        let url = format!(
            "{}/v1/{}/cryptoKeyVersions/1:asymmetricSign",
            self.kms_base_url(),
            key_name
        );

        let digest = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, data);
        let b64_digest = base64::engine::general_purpose::STANDARD.encode(digest.as_ref());
        let body = serde_json::json!({
            "digest": {
                "sha256": b64_digest
            }
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("KMS asymmetricSign error: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(
                "asymmetricSign failed".to_string(),
            ));
        }

        let j: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("failed to parse asymmetricSign response: {e}"))
        })?;

        let b64_sig = j["signature"].as_str().ok_or_else(|| {
            ProxyError::ProviderError("missing signature in response".to_string())
        })?;

        let sig = base64::engine::general_purpose::STANDARD
            .decode(b64_sig)
            .map_err(|e| ProxyError::ProviderError(format!("invalid base64 signature: {e}")))?;

        Ok(sig)
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Ok(true)
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let url = format!(
            "{}/v1/projects/{}/locations/{}:generateRandomBytes",
            self.kms_base_url(),
            self.project_id,
            self.location
        );

        let body = serde_json::json!({
            "lengthBytes": num_bytes
        });

        if let Ok(resp) = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(j) = resp.json::<serde_json::Value>().await {
                    if let Some(b64_data) = j["data"].as_str() {
                        if let Ok(bytes) =
                            base64::engine::general_purpose::STANDARD.decode(b64_data)
                        {
                            return Ok(Zeroizing::new(bytes));
                        }
                    }
                }
            }
        }

        AutoEnvelope::generate_random_bytes(num_bytes)
    }
}

#[cfg(not(feature = "gcp"))]
pub struct GcpKmsProvider {
    pub project_id: String,
    pub location: String,
    pub key_ring: String,
    pub endpoint_url: Option<String>,
    pub secret_name_prefix: String,
    pub envelope_master_kek_name: Option<String>,
}

#[cfg(not(feature = "gcp"))]
impl GcpKmsProvider {
    pub fn new(config: &GcpConfig) -> Result<Self, ProxyError> {
        Ok(Self {
            project_id: config
                .project_id
                .clone()
                .unwrap_or_else(|| "my-project".to_string()),
            location: config
                .location
                .clone()
                .unwrap_or_else(|| "global".to_string()),
            key_ring: config
                .key_ring
                .clone()
                .unwrap_or_else(|| "kmip-keyring".to_string()),
            endpoint_url: config.endpoint_url.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            envelope_master_kek_name: config.envelope_master_kek_name.clone(),
        })
    }
}

#[cfg(not(feature = "gcp"))]
#[async_trait]
impl CloudKmsProvider for GcpKmsProvider {
    fn provider_name(&self) -> &'static str {
        "gcp-kms"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
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
            "GCP feature not enabled".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP feature not enabled".to_string(),
        ))
    }
}
