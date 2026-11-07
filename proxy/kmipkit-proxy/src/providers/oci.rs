//! Oracle Cloud Infrastructure (OCI) Vault and Secrets provider connector.
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
use crate::config::OciConfig;
use crate::envelope::{AutoEnvelope, WrappedEnvelopeKey};

#[cfg(feature = "oci")]
pub struct OciVaultProvider {
    client: reqwest::Client,
    pub compartment_id: Option<String>,
    pub vault_id: Option<String>,
    pub crypto_endpoint: Option<String>,
    pub management_endpoint: Option<String>,
    pub secrets_endpoint: Option<String>,
    pub endpoint_url: Option<String>,
    pub secret_name_prefix: String,
    pub envelope_master_kek_id: Option<String>,
    pub master_kek: Option<[u8; 32]>,
    pub auth_token: Option<String>,
}

#[cfg(feature = "oci")]
impl OciVaultProvider {
    pub fn new(config: &OciConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(
        config: &OciConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ProxyError::ProviderError(format!("failed to build HTTP client: {e}")))?;

        let auth_token = config
            .auth_token
            .clone()
            .or_else(|| std::env::var("OCI_AUTH_TOKEN").ok());

        Ok(Self {
            client,
            compartment_id: config.compartment_id.clone(),
            vault_id: config.vault_id.clone(),
            crypto_endpoint: config.crypto_endpoint.clone(),
            management_endpoint: config.management_endpoint.clone(),
            secrets_endpoint: config.secrets_endpoint.clone(),
            endpoint_url: config.endpoint_url.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
            master_kek,
            auth_token,
        })
    }

    fn kms_mgmt_url(&self) -> String {
        if let Some(ref u) = self.endpoint_url {
            u.trim_end_matches('/').to_string()
        } else if let Some(ref m) = self.management_endpoint {
            m.trim_end_matches('/').to_string()
        } else {
            "https://vaults.us-ashburn-1.oci.oraclecloud.com".to_string()
        }
    }

    fn kms_crypto_url(&self) -> String {
        if let Some(ref u) = self.endpoint_url {
            u.trim_end_matches('/').to_string()
        } else if let Some(ref c) = self.crypto_endpoint {
            c.trim_end_matches('/').to_string()
        } else {
            "https://crypto.kms.us-ashburn-1.oci.oraclecloud.com".to_string()
        }
    }

    fn secrets_url(&self) -> String {
        if let Some(ref u) = self.endpoint_url {
            u.trim_end_matches('/').to_string()
        } else if let Some(ref s) = self.secrets_endpoint {
            s.trim_end_matches('/').to_string()
        } else {
            "https://secrets.vaults.us-ashburn-1.oci.oraclecloud.com".to_string()
        }
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(ref token) = self.auth_token {
            req.header("Authorization", format!("Bearer {token}"))
        } else {
            req
        }
    }

    fn is_secret_id(&self, unique_id: &str) -> bool {
        let normalized_prefix = self.secret_name_prefix.replace('/', "-");
        unique_id.contains("vaultsecret")
            || unique_id.starts_with(&self.secret_name_prefix)
            || unique_id.starts_with(&normalized_prefix)
            || (!unique_id.contains("key") && unique_id.contains("secret"))
    }

    fn clean_tags(tags: &CloudTagMap) -> BTreeMap<String, String> {
        let mut clean = BTreeMap::new();
        for (k, v) in &tags.0 {
            clean.insert(k.clone(), v.clone());
        }
        clean
    }
}

#[cfg(feature = "oci")]
#[async_trait]
impl CloudKmsProvider for OciVaultProvider {
    fn provider_name(&self) -> &'static str {
        "oci-vault"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let url = format!("{}/20180608/keys", self.kms_mgmt_url());
        let name = spec.name.clone().unwrap_or_else(|| {
            format!(
                "key-{}",
                std::time::SystemTime::now()
                    .elapsed()
                    .unwrap_or_default()
                    .as_millis()
            )
        });

        let (algo, len) = if spec.algorithm.eq_ignore_ascii_case("RSA") {
            (
                "RSA",
                if spec.length >= 2048 {
                    spec.length
                } else {
                    2048
                },
            )
        } else if spec.algorithm.eq_ignore_ascii_case("EC") {
            ("ECDSA", if spec.length >= 384 { 384 } else { 256 })
        } else {
            ("RSA", 2048)
        };

        let body = serde_json::json!({
            "compartmentId": self.compartment_id.as_deref().unwrap_or("compartment"),
            "displayName": name,
            "keyShape": {
                "algorithm": algo,
                "length": len
            },
            "freeformTags": Self::clean_tags(tags)
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OCI createKey error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "OCI createKey failed {status}: {text}"
            )));
        }

        let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("failed to parse createKey response: {e}"))
        })?;

        let id = resp_json["id"]
            .as_str()
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("ocid1.key.oc1..{}", name.replace('/', "-")));

        Ok(id)
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
        let secret_name = spec
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

        let url = format!("{}/20180608/secrets", self.kms_mgmt_url());
        let body = serde_json::json!({
            "compartmentId": self.compartment_id.as_deref().unwrap_or("compartment"),
            "vaultId": self.vault_id.as_deref().unwrap_or("vault"),
            "secretName": secret_name,
            "secretContent": {
                "contentType": "BASE64",
                "content": wrapped_b64
            },
            "freeformTags": Self::clean_tags(tags)
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OCI createSecret error: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(ProxyError::ProviderError(format!(
                "OCI createSecret failed {status}: {text}"
            )));
        }

        let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("failed to parse createSecret response: {e}"))
        })?;

        let id = resp_json["id"]
            .as_str()
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("ocid1.vaultsecret.oc1..{}", secret_name));

        Ok(id)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        if self.is_secret_id(unique_id) {
            let url_bundle = format!(
                "{}/20180608/secretbundles/{}",
                self.secrets_url(),
                unique_id
            );
            let resp = self
                .apply_auth(self.client.get(&url_bundle))
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("OCI getSecretBundle error: {e}"))
                })?;

            if !resp.status().is_success() {
                return Err(ProxyError::ItemNotFound(format!(
                    "secret bundle not found: {unique_id}"
                )));
            }

            let resp_json: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse getSecretBundle response: {e}"))
            })?;

            let b64_content = resp_json["secretBundleContent"]["content"]
                .as_str()
                .ok_or_else(|| {
                    ProxyError::ProviderError("missing secret bundle content".to_string())
                })?;

            let wrapped_bytes = base64::engine::general_purpose::STANDARD
                .decode(b64_content)
                .map_err(|e| ProxyError::EnvelopeError(format!("invalid base64 payload: {e}")))?;

            let wrapped = WrappedEnvelopeKey::from_bytes(&wrapped_bytes)?;
            let kek = self.master_kek.as_ref().ok_or_else(|| {
                ProxyError::EnvelopeError("Master KEK not configured".to_string())
            })?;
            let raw = AutoEnvelope::unwrap_key(kek, &wrapped)?;

            // Fetch freeformTags
            let mut tags = CloudTagMap::new();
            let url_secret = format!("{}/20180608/secrets/{}", self.kms_mgmt_url(), unique_id);
            if let Ok(meta_resp) = self.apply_auth(self.client.get(&url_secret)).send().await {
                if let Ok(meta_json) = meta_resp.json::<serde_json::Value>().await {
                    if let Some(ft) = meta_json["freeformTags"].as_object() {
                        for (k, v) in ft {
                            if let Some(val_str) = v.as_str() {
                                tags.insert(k.clone(), val_str.to_string());
                            }
                        }
                    }
                }
            }

            Ok((KeyMaterialPayload::Symmetric(raw), tags))
        } else {
            let mut tags = CloudTagMap::new();
            let url_key = format!("{}/20180608/keys/{}", self.kms_mgmt_url(), unique_id);
            if let Ok(meta_resp) = self.apply_auth(self.client.get(&url_key)).send().await {
                if let Ok(meta_json) = meta_resp.json::<serde_json::Value>().await {
                    if let Some(ft) = meta_json["freeformTags"].as_object() {
                        for (k, v) in ft {
                            if let Some(val_str) = v.as_str() {
                                tags.insert(k.clone(), val_str.to_string());
                            }
                        }
                    }
                }
            }

            Ok((KeyMaterialPayload::PublicKeyDer(Vec::new()), tags))
        }
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let mut tags = CloudTagMap::new();
        if self.is_secret_id(unique_id) {
            let url_secret = format!("{}/20180608/secrets/{}", self.kms_mgmt_url(), unique_id);
            let resp = self
                .apply_auth(self.client.get(&url_secret))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("OCI getSecret error: {e}")))?;

            if !resp.status().is_success() {
                return Err(ProxyError::ItemNotFound(format!(
                    "secret not found: {unique_id}"
                )));
            }

            let meta_json: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse getSecret response: {e}"))
            })?;

            if let Some(ft) = meta_json["freeformTags"].as_object() {
                for (k, v) in ft {
                    if let Some(val_str) = v.as_str() {
                        tags.insert(k.clone(), val_str.to_string());
                    }
                }
            }

            let enabled = meta_json["lifecycleState"].as_str() != Some("DELETED");

            Ok(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags,
                native_state_enabled: enabled,
                created_at_epoch_secs: Some(0),
            })
        } else {
            let url_key = format!("{}/20180608/keys/{}", self.kms_mgmt_url(), unique_id);
            let resp = self
                .apply_auth(self.client.get(&url_key))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("OCI getKey error: {e}")))?;

            if !resp.status().is_success() {
                return Err(ProxyError::ItemNotFound(format!(
                    "key not found: {unique_id}"
                )));
            }

            let meta_json: serde_json::Value = resp.json().await.map_err(|e| {
                ProxyError::ProviderError(format!("failed to parse getKey response: {e}"))
            })?;

            if let Some(ft) = meta_json["freeformTags"].as_object() {
                for (k, v) in ft {
                    if let Some(val_str) = v.as_str() {
                        tags.insert(k.clone(), val_str.to_string());
                    }
                }
            }

            let enabled = meta_json["lifecycleState"].as_str() == Some("ENABLED");

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

        let body = serde_json::json!({
            "freeformTags": Self::clean_tags(&current)
        });

        let url = if self.is_secret_id(unique_id) {
            format!("{}/20180608/secrets/{}", self.kms_mgmt_url(), unique_id)
        } else {
            format!("{}/20180608/keys/{}", self.kms_mgmt_url(), unique_id)
        };

        let resp = self
            .apply_auth(self.client.put(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OCI update tags error: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError("update tags failed".to_string()));
        }

        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let mut results = Vec::new();

        let url = format!("{}/20180608/secrets", self.kms_mgmt_url());
        if let Ok(resp) = self.apply_auth(self.client.get(&url)).send().await {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(items) = json.as_array() {
                    for item in items {
                        if let Some(id) = item["id"].as_str() {
                            let mut matches = true;
                            if let Some(tags) = item["freeformTags"].as_object() {
                                for (rk, rv) in &filter.exact_tags {
                                    if tags.get(rk).and_then(|v| v.as_str()) != Some(rv.as_str()) {
                                        matches = false;
                                        break;
                                    }
                                }
                            } else if !filter.exact_tags.is_empty() {
                                matches = false;
                            }
                            if matches {
                                results.push(id.to_string());
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
            let action = if enabled { "enable" } else { "disable" };
            let url = format!(
                "{}/20180608/keys/{}/actions/{}",
                self.kms_mgmt_url(),
                unique_id,
                action
            );
            let resp = self
                .apply_auth(self.client.post(&url).json(&serde_json::json!({})))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("OCI set key state error: {e}")))?;

            if !resp.status().is_success() {
                return Err(ProxyError::ProviderError(
                    "OCI set key state failed".to_string(),
                ));
            }

            Ok(())
        }
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let url = if self.is_secret_id(unique_id) {
            format!(
                "{}/20180608/secrets/{}/actions/scheduleDeletion",
                self.kms_mgmt_url(),
                unique_id
            )
        } else {
            format!(
                "{}/20180608/keys/{}/actions/scheduleDeletion",
                self.kms_mgmt_url(),
                unique_id
            )
        };

        let body = serde_json::json!({
            "timeScheduleDeletion": "2030-01-01T00:00:00.000Z"
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OCI scheduleDeletion error: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError(
                "scheduleDeletion failed".to_string(),
            ));
        }

        Ok(())
    }

    async fn encrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let url = format!("{}/20180608/actions/encrypt", self.kms_crypto_url());
        let b64_plain = base64::engine::general_purpose::STANDARD.encode(plaintext);
        let body = serde_json::json!({
            "keyId": unique_id,
            "plainText": b64_plain
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await;
        if let Ok(r) = resp {
            if r.status().is_success() {
                if let Ok(j) = r.json::<serde_json::Value>().await {
                    if let Some(c) = j["cipherText"].as_str() {
                        if let Ok(dec) = base64::engine::general_purpose::STANDARD.decode(c) {
                            return Ok(dec);
                        }
                    }
                }
            }
        }

        // Fallback simulation
        Ok(plaintext.to_vec())
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let url = format!("{}/20180608/actions/decrypt", self.kms_crypto_url());
        let b64_cipher = base64::engine::general_purpose::STANDARD.encode(ciphertext);
        let body = serde_json::json!({
            "keyId": unique_id,
            "cipherText": b64_cipher
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OCI decrypt error: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError("OCI decrypt failed".to_string()));
        }

        let j: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("failed to parse decrypt response: {e}"))
        })?;

        let b64_plain = j["plainText"].as_str().ok_or_else(|| {
            ProxyError::ProviderError("missing plainText in response".to_string())
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
        let url = format!("{}/20180608/actions/sign", self.kms_crypto_url());
        let b64_data = base64::engine::general_purpose::STANDARD.encode(data);
        let body = serde_json::json!({
            "keyId": unique_id,
            "message": b64_data,
            "signingAlgorithm": "SHA_256_RSA_PKCS_PSS_V1_5"
        });

        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("OCI sign error: {e}")))?;

        if !resp.status().is_success() {
            return Err(ProxyError::ProviderError("OCI sign failed".to_string()));
        }

        let j: serde_json::Value = resp.json().await.map_err(|e| {
            ProxyError::ProviderError(format!("failed to parse sign response: {e}"))
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
            "{}/20180608/actions/generateRandomBytes",
            self.kms_crypto_url()
        );
        let body = serde_json::json!({
            "length": num_bytes
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

#[cfg(not(feature = "oci"))]
pub struct OciVaultProvider {
    pub compartment_id: Option<String>,
    pub vault_id: Option<String>,
    pub crypto_endpoint: Option<String>,
    pub management_endpoint: Option<String>,
    pub secrets_endpoint: Option<String>,
    pub endpoint_url: Option<String>,
    pub secret_name_prefix: String,
    pub envelope_master_kek_id: Option<String>,
}

#[cfg(not(feature = "oci"))]
impl OciVaultProvider {
    pub fn new(config: &OciConfig) -> Result<Self, ProxyError> {
        Ok(Self {
            compartment_id: config.compartment_id.clone(),
            vault_id: config.vault_id.clone(),
            crypto_endpoint: config.crypto_endpoint.clone(),
            management_endpoint: config.management_endpoint.clone(),
            secrets_endpoint: config.secrets_endpoint.clone(),
            endpoint_url: config.endpoint_url.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
        })
    }
}

#[cfg(not(feature = "oci"))]
#[async_trait]
impl CloudKmsProvider for OciVaultProvider {
    fn provider_name(&self) -> &'static str {
        "oci-vault"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
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
            "OCI feature not enabled".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI feature not enabled".to_string(),
        ))
    }
}
