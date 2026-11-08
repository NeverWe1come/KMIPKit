//! Alibaba Cloud KMS and Secrets Manager provider connector.
#![forbid(unsafe_code)]

use async_trait::async_trait;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::AlibabaConfig;

#[cfg(feature = "alibaba")]
pub struct AlibabaKmsProvider {
    pub endpoint_url: String,
    pub region_id: String,
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
        let region_id = config
            .region_id
            .clone()
            .unwrap_or_else(|| "cn-hangzhou".to_string());
        let endpoint_url = config
            .endpoint_url
            .clone()
            .unwrap_or_else(|| format!("https://kms.{region_id}.aliyuncs.com"));

        Ok(Self {
            endpoint_url,
            region_id,
            master_kek,
        })
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
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::Unsupported("alibaba stub not yet implemented".into()))
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
