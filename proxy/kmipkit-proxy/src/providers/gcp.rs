//! Google Cloud KMS and Secret Manager provider connector.
#![forbid(unsafe_code)]

use async_trait::async_trait;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::GcpConfig;

pub struct GcpKmsProvider {
    pub project_id: String,
    pub location: String,
    pub key_ring: String,
    pub envelope_master_kek_name: Option<String>,
}

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
            envelope_master_kek_name: config.envelope_master_kek_name.clone(),
        })
    }
}

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
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
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
            "GCP credentials not configured".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "GCP credentials not configured".to_string(),
        ))
    }
}
