//! Cloud KMS and Secret Management Provider abstraction.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use zeroize::Zeroizing;

use crate::ProxyError;
use crate::config::BackendConfig;

pub mod aws;
pub mod azure;
pub mod gcp;
pub mod oci;

/// Normalized map of cloud tags/labels.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CloudTagMap(pub BTreeMap<String, String>);

impl CloudTagMap {
    #[must_use]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }
}

/// Key creation specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateKeySpec {
    pub algorithm: String,
    pub length: u32,
    pub usage_mask: u32,
    pub name: Option<String>,
}

/// Metadata and tags associated with a cloud resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudResourceMeta {
    pub unique_id: String,
    pub tags: CloudTagMap,
    pub native_state_enabled: bool,
    pub created_at_epoch_secs: Option<i64>,
}

/// Tag filter for KMIP Locate operation.
#[derive(Debug, Clone, Default)]
pub struct CloudTagFilter {
    pub exact_tags: BTreeMap<String, String>,
}

/// Cryptographic parameters for server-side cryptographic operations.
#[derive(Debug, Clone, Default)]
pub struct CryptoParams {
    pub algorithm: String,
    pub padding: Option<String>,
    pub hashing_algorithm: Option<String>,
}

/// Key material payload retrieved from the provider.
#[derive(Debug, Clone)]
pub enum KeyMaterialPayload {
    Symmetric(Zeroizing<Vec<u8>>),
    PublicKeyDer(Vec<u8>),
    PrivateKeyDer(Zeroizing<Vec<u8>>),
}

/// Universal asynchronous trait for cloud KMS and secret storage backends.
#[async_trait]
pub trait CloudKmsProvider: Send + Sync + 'static {
    /// Provider name identifier (e.g. "aws", "azure", "gcp", "oci", "mock").
    fn provider_name(&self) -> &'static str;

    /// Creates a native key in the cloud KMS.
    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError>;

    /// Creates an envelope-wrapped key stored in cloud secret storage.
    async fn create_envelope_key(
        &self,
        spec: &CreateKeySpec,
        raw_material: Option<Zeroizing<Vec<u8>>>,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError>;

    /// Retrieves key material (decrypted DEK for envelope, public key for asymmetric).
    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError>;

    /// Retrieves tags and metadata for a given resource.
    async fn get_tags_and_metadata(&self, unique_id: &str)
    -> Result<CloudResourceMeta, ProxyError>;

    /// Updates or deletes tags on the cloud resource.
    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        remove_keys: &[String],
    ) -> Result<(), ProxyError>;

    /// Locates resource unique identifiers matching the specified tag filter.
    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError>;

    /// Enables or disables the native key.
    async fn set_native_enabled_state(
        &self,
        unique_id: &str,
        enabled: bool,
    ) -> Result<(), ProxyError>;

    /// Schedules key destruction.
    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError>;

    /// Delegated cryptographic encryption.
    async fn encrypt(
        &self,
        unique_id: &str,
        params: &CryptoParams,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError>;

    /// Delegated cryptographic decryption.
    async fn decrypt(
        &self,
        unique_id: &str,
        params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError>;

    /// Delegated cryptographic signing.
    async fn sign(
        &self,
        unique_id: &str,
        params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError>;

    /// Delegated cryptographic signature verification.
    async fn verify(
        &self,
        unique_id: &str,
        params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError>;

    /// Random bytes generation.
    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError>;
}

/// In-memory mock cloud provider for deterministic foundation testing.
pub struct InMemoryMockProvider {
    kek: [u8; 32],
    keys: RwLock<BTreeMap<String, MockKeyEntry>>,
}

struct MockKeyEntry {
    meta: CloudResourceMeta,
    material: KeyMaterialPayload,
}

impl Default for InMemoryMockProvider {
    fn default() -> Self {
        Self::new([0x5au8; 32])
    }
}

impl InMemoryMockProvider {
    #[must_use]
    pub fn new(kek: [u8; 32]) -> Self {
        Self {
            kek,
            keys: RwLock::new(BTreeMap::new()),
        }
    }
}

#[async_trait]
impl CloudKmsProvider for InMemoryMockProvider {
    fn provider_name(&self) -> &'static str {
        "mock"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let unique_id = format!(
            "mock-native-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let raw = crate::envelope::AutoEnvelope::generate_random_bytes(spec.length as usize / 8)?;
        let meta = CloudResourceMeta {
            unique_id: unique_id.clone(),
            tags: tags.clone(),
            native_state_enabled: true,
            created_at_epoch_secs: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
        };
        let mut guard = self.keys.write().unwrap();
        guard.insert(
            unique_id.clone(),
            MockKeyEntry {
                meta,
                material: KeyMaterialPayload::Symmetric(raw),
            },
        );
        Ok(unique_id)
    }

    async fn create_envelope_key(
        &self,
        spec: &CreateKeySpec,
        raw_material: Option<Zeroizing<Vec<u8>>>,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let unique_id = format!(
            "mock-env-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let raw = match raw_material {
            Some(m) => m,
            None => crate::envelope::AutoEnvelope::generate_random_bytes(spec.length as usize / 8)?,
        };
        // Wrap with KEK
        let _wrapped = crate::envelope::AutoEnvelope::wrap_key(&self.kek, &raw)?;
        let meta = CloudResourceMeta {
            unique_id: unique_id.clone(),
            tags: tags.clone(),
            native_state_enabled: true,
            created_at_epoch_secs: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
        };
        let mut guard = self.keys.write().unwrap();
        guard.insert(
            unique_id.clone(),
            MockKeyEntry {
                meta,
                material: KeyMaterialPayload::Symmetric(raw),
            },
        );
        Ok(unique_id)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        let guard = self.keys.read().unwrap();
        let entry = guard
            .get(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        Ok((entry.material.clone(), entry.meta.tags.clone()))
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let guard = self.keys.read().unwrap();
        let entry = guard
            .get(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        Ok(entry.meta.clone())
    }

    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        let mut guard = self.keys.write().unwrap();
        let entry = guard
            .get_mut(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        for (k, v) in &upsert.0 {
            entry.meta.tags.0.insert(k.clone(), v.clone());
        }
        for k in remove_keys {
            entry.meta.tags.0.remove(k);
        }
        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let guard = self.keys.read().unwrap();
        let mut matched = Vec::new();
        for (id, entry) in guard.iter() {
            let matches = filter
                .exact_tags
                .iter()
                .all(|(k, v)| entry.meta.tags.0.get(k) == Some(v));
            if matches {
                matched.push(id.clone());
                if let Some(limit) = max_items {
                    if matched.len() >= limit as usize {
                        break;
                    }
                }
            }
        }
        Ok(matched)
    }

    async fn set_native_enabled_state(
        &self,
        unique_id: &str,
        enabled: bool,
    ) -> Result<(), ProxyError> {
        let mut guard = self.keys.write().unwrap();
        let entry = guard
            .get_mut(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        entry.meta.native_state_enabled = enabled;
        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let mut guard = self.keys.write().unwrap();
        let entry = guard
            .get_mut(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        entry.meta.native_state_enabled = false;
        entry.meta.tags.0.insert(
            crate::tag_codec::TAG_STATE.to_string(),
            "destroyed".to_string(),
        );
        Ok(())
    }

    async fn encrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let guard = self.keys.read().unwrap();
        let entry = guard
            .get(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        match &entry.material {
            KeyMaterialPayload::Symmetric(key) => {
                let wrapped = crate::envelope::AutoEnvelope::wrap_key(
                    key,
                    &Zeroizing::new(plaintext.to_vec()),
                )?;
                Ok(wrapped.to_bytes())
            }
            _ => Err(ProxyError::Unsupported(
                "asymmetric mock encrypt".to_string(),
            )),
        }
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let guard = self.keys.read().unwrap();
        let entry = guard
            .get(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        match &entry.material {
            KeyMaterialPayload::Symmetric(key) => {
                let wrapped = crate::envelope::WrappedEnvelopeKey::from_bytes(ciphertext)?;
                crate::envelope::AutoEnvelope::unwrap_key(key, &wrapped)
            }
            _ => Err(ProxyError::Unsupported(
                "asymmetric mock decrypt".to_string(),
            )),
        }
    }

    async fn sign(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let guard = self.keys.read().unwrap();
        let entry = guard
            .get(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        match &entry.material {
            KeyMaterialPayload::Symmetric(key) => {
                // Mock HMAC-SHA256 signature
                use aws_lc_rs::hmac;
                let s_key = hmac::Key::new(hmac::HMAC_SHA256, key);
                let tag = hmac::sign(&s_key, data);
                Ok(tag.as_ref().to_vec())
            }
            _ => Err(ProxyError::Unsupported("asymmetric mock sign".to_string())),
        }
    }

    async fn verify(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError> {
        let guard = self.keys.read().unwrap();
        let entry = guard
            .get(unique_id)
            .ok_or_else(|| ProxyError::ItemNotFound(unique_id.to_string()))?;
        match &entry.material {
            KeyMaterialPayload::Symmetric(key) => {
                use aws_lc_rs::hmac;
                let s_key = hmac::Key::new(hmac::HMAC_SHA256, key);
                Ok(hmac::verify(&s_key, data, signature).is_ok())
            }
            _ => Err(ProxyError::Unsupported(
                "asymmetric mock verify".to_string(),
            )),
        }
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        crate::envelope::AutoEnvelope::generate_random_bytes(num_bytes)
    }
}

/// Factory function to create the active CloudKmsProvider based on configuration.
pub fn create_provider(config: &BackendConfig) -> Result<Arc<dyn CloudKmsProvider>, ProxyError> {
    match config.provider.to_ascii_lowercase().as_str() {
        "mock" => {
            let kek = if let Some(ref hex) = config.master_kek_hex {
                let bytes = hex::decode(hex)
                    .map_err(|e| ProxyError::ConfigError(format!("invalid master_kek_hex: {e}")))?;
                if bytes.len() != 32 {
                    return Err(ProxyError::ConfigError(
                        "master_kek_hex must be 32 bytes".to_string(),
                    ));
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            } else {
                [0x5au8; 32]
            };
            Ok(Arc::new(InMemoryMockProvider::new(kek)))
        }
        "aws" => Ok(Arc::new(aws::AwsKmsProvider::new(&config.aws)?)),
        "azure" => Ok(Arc::new(azure::AzureKeyVaultProvider::new(&config.azure)?)),
        "gcp" => Ok(Arc::new(gcp::GcpKmsProvider::new(&config.gcp)?)),
        "oci" => Ok(Arc::new(oci::OciVaultProvider::new(&config.oci)?)),
        other => Err(ProxyError::ConfigError(format!(
            "unknown provider: {other}"
        ))),
    }
}

mod hex {
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("odd hex length".to_string());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}
