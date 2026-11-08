//! Scaleway Key Manager and Secret Manager Provider Bridge (RED stub).
#![forbid(unsafe_code)]

#[cfg(feature = "scaleway")]
use std::sync::Arc;

use async_trait::async_trait;
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
pub struct ScalewayKmsProvider {
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
        let region = config.region.clone().unwrap_or_else(|| "fr-par".to_string());
        let endpoint_url = config
            .endpoint_url
            .clone()
            .unwrap_or_else(|| "https://api.scaleway.com".to_string());
        Ok(Self {
            endpoint_url,
            region,
            project_id: config.project_id.clone(),
            auth_token: Arc::new(RwLock::new(config.auth_token.clone())),
            item_prefix: config.item_prefix.clone().unwrap_or_else(|| "kmipkit/".to_string()),
            master_kek,
        })
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
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn get_tags_and_metadata(&self, _unique_id: &str) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderError("Unimplemented RED".to_string()))
    }
}
