//! Open Telekom Cloud KMS / DEW, TMS, and CSMS Provider Bridge (KMIPKIT-0082).
#![forbid(unsafe_code)]

#[cfg(feature = "otc")]
use std::sync::Arc;
#[cfg(feature = "otc")]
use async_trait::async_trait;
#[cfg(feature = "otc")]
use zeroize::Zeroizing;

#[cfg(feature = "otc")]
use crate::ProxyError;
#[cfg(feature = "otc")]
use crate::config::OtcConfig;
#[cfg(feature = "otc")]
use crate::providers::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};

#[cfg(feature = "otc")]
pub struct OtcKmsProvider {
    _config: OtcConfig,
}

#[cfg(feature = "otc")]
impl OtcKmsProvider {
    pub fn new(config: &OtcConfig) -> Result<Self, ProxyError> {
        Self::from_config(config, None)
    }

    pub fn from_config(config: &OtcConfig, _master_kek: Option<[u8; 32]>) -> Result<Self, ProxyError> {
        Ok(Self {
            _config: config.clone(),
        })
    }
}

#[cfg(feature = "otc")]
#[async_trait]
impl CloudKmsProvider for OtcKmsProvider {
    fn provider_name(&self) -> &'static str {
        "otc"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, ProxyError> {
        Err(ProxyError::ProviderError("unimplemented in RED phase".to_string()))
    }

    async fn random_bytes(&self, num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        crate::envelope::AutoEnvelope::generate_random_bytes(num_bytes)
    }
}

#[cfg(not(feature = "otc"))]
#[derive(Debug, Clone, Default)]
pub struct OtcKmsProvider;

#[cfg(not(feature = "otc"))]
impl OtcKmsProvider {
    pub fn new(_config: &crate::config::OtcConfig) -> Result<Self, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported(
            "OTC KMS provider is disabled in this build".to_string(),
        ))
    }
}

#[cfg(not(feature = "otc"))]
#[async_trait::async_trait]
impl crate::providers::CloudKmsProvider for OtcKmsProvider {
    fn provider_name(&self) -> &'static str {
        "otc"
    }

    async fn create_native_key(
        &self,
        _spec: &crate::providers::CreateKeySpec,
        _tags: &crate::providers::CloudTagMap,
    ) -> Result<String, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn create_envelope_key(
        &self,
        _spec: &crate::providers::CreateKeySpec,
        _raw_material: Option<zeroize::Zeroizing<Vec<u8>>>,
        _tags: &crate::providers::CloudTagMap,
    ) -> Result<String, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<
        (
            crate::providers::KeyMaterialPayload,
            crate::providers::CloudTagMap,
        ),
        crate::ProxyError,
    > {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<crate::providers::CloudResourceMeta, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &crate::providers::CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn locate_by_tags(
        &self,
        _filter: &crate::providers::CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &crate::providers::CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &crate::providers::CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &crate::providers::CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn verify(
        &self,
        _unique_id: &str,
        _params: &crate::providers::CryptoParams,
        _data: &[u8],
        _signature: &[u8],
    ) -> Result<bool, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }

    async fn random_bytes(
        &self,
        _num_bytes: usize,
    ) -> Result<zeroize::Zeroizing<Vec<u8>>, crate::ProxyError> {
        Err(crate::ProxyError::Unsupported("OTC disabled".to_string()))
    }
}
