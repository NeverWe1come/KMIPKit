//! Oracle Cloud Infrastructure (OCI) Vault provider connector.
#![forbid(unsafe_code)]

use async_trait::async_trait;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::OciConfig;

pub struct OciVaultProvider {
    pub compartment_id: Option<String>,
    pub vault_id: Option<String>,
    pub crypto_endpoint: Option<String>,
    pub management_endpoint: Option<String>,
    pub secrets_endpoint: Option<String>,
    pub envelope_master_kek_id: Option<String>,
}

impl OciVaultProvider {
    pub fn new(config: &OciConfig) -> Result<Self, ProxyError> {
        Ok(Self {
            compartment_id: config.compartment_id.clone(),
            vault_id: config.vault_id.clone(),
            crypto_endpoint: config.crypto_endpoint.clone(),
            management_endpoint: config.management_endpoint.clone(),
            secrets_endpoint: config.secrets_endpoint.clone(),
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
        })
    }
}

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
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
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
            "OCI credentials not configured".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "OCI credentials not configured".to_string(),
        ))
    }
}
