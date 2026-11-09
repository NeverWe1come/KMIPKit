//! Azure Key Vault provider connector.
#![forbid(unsafe_code)]

#[cfg(feature = "azure")]
use std::collections::HashMap;
#[cfg(feature = "azure")]
use std::sync::Arc;

use async_trait::async_trait;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::AzureConfig;

#[cfg(feature = "azure")]
use azure_core::credentials::{AccessToken, Secret, TokenCredential, TokenRequestOptions};
#[cfg(feature = "azure")]
use azure_core::http::RequestContent;
#[cfg(feature = "azure")]
use azure_security_keyvault_keys::KeyClient;
#[cfg(feature = "azure")]
use azure_security_keyvault_keys::models::{
    CreateKeyParameters, EncryptionAlgorithm, GetRandomBytesParameters, KeyAttributes,
    KeyOperation, KeyOperationParameters, KeyType, SignParameters, SignatureAlgorithm,
    UpdateKeyPropertiesParameters, VerifyParameters,
};
#[cfg(feature = "azure")]
use azure_security_keyvault_secrets::SecretClient;
#[cfg(feature = "azure")]
use azure_security_keyvault_secrets::models::{
    SecretAttributes, SetSecretParameters, UpdateSecretPropertiesParameters,
};

#[cfg(feature = "azure")]
fn to_request_content<T: serde::Serialize>(val: &T) -> Result<RequestContent<T>, ProxyError> {
    let body = serde_json::to_vec(val)
        .map_err(|e| ProxyError::Internal(format!("Serialization error: {e}")))?;
    Ok(RequestContent::from(body))
}

#[cfg(feature = "azure")]
#[derive(Debug, Default)]
struct DummyTokenCredential;

#[cfg(feature = "azure")]
#[async_trait]
impl TokenCredential for DummyTokenCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        Ok(AccessToken::new(
            "dummy_token".to_string(),
            azure_core::time::OffsetDateTime::now_utc() + azure_core::time::Duration::days(1),
        ))
    }
}

#[cfg(feature = "azure")]
pub struct AzureKeyVaultProvider {
    key_client: KeyClient,
    secret_client: SecretClient,
    pub vault_url: String,
    pub envelope_master_kek_name: Option<String>,
    pub secret_name_prefix: String,
    pub master_kek: Option<[u8; 32]>,
}

#[cfg(feature = "azure")]
impl AzureKeyVaultProvider {
    pub fn new(config: &AzureConfig) -> Result<Self, ProxyError> {
        let vault_url = config
            .vault_url
            .clone()
            .unwrap_or_else(|| "https://localhost:8443".to_string());

        let credential: Arc<dyn TokenCredential> =
            if let (Ok(tenant), Ok(client_id), Ok(client_secret)) = (
                std::env::var("AZURE_TENANT_ID"),
                std::env::var("AZURE_CLIENT_ID"),
                std::env::var("AZURE_CLIENT_SECRET"),
            ) {
                match azure_identity::ClientSecretCredential::new(
                    &tenant,
                    client_id,
                    Secret::new(client_secret),
                    None,
                ) {
                    Ok(cred) => cred,
                    Err(_) => Arc::new(DummyTokenCredential),
                }
            } else if let Ok(dev_cred) = azure_identity::DeveloperToolsCredential::new(None) {
                dev_cred
            } else {
                Arc::new(DummyTokenCredential)
            };

        let is_custom_endpoint = !vault_url.contains(".vault.azure.net");
        let key_options = if is_custom_endpoint {
            let client_options = azure_core::http::ClientOptions {
                allow_insecure_connections: true,
                ..Default::default()
            };
            Some(azure_security_keyvault_keys::KeyClientOptions {
                verify_challenge_resource: Some(false),
                client_options,
                ..Default::default()
            })
        } else {
            None
        };
        let secret_options = if is_custom_endpoint {
            let client_options = azure_core::http::ClientOptions {
                allow_insecure_connections: true,
                ..Default::default()
            };
            Some(azure_security_keyvault_secrets::SecretClientOptions {
                verify_challenge_resource: Some(false),
                client_options,
                ..Default::default()
            })
        } else {
            None
        };

        let key_client = KeyClient::new(&vault_url, credential.clone(), key_options)
            .map_err(|e| ProxyError::ProviderError(format!("Azure KeyClient error: {e}")))?;
        let secret_client = SecretClient::new(&vault_url, credential, secret_options)
            .map_err(|e| ProxyError::ProviderError(format!("Azure SecretClient error: {e}")))?;

        Ok(Self {
            key_client,
            secret_client,
            vault_url,
            envelope_master_kek_name: config.envelope_master_kek_name.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            master_kek: None,
        })
    }

    pub fn from_config(
        config: &AzureConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let mut provider = Self::new(config)?;
        provider.master_kek = master_kek;
        Ok(provider)
    }

    #[must_use]
    pub fn with_clients(
        key_client: KeyClient,
        secret_client: SecretClient,
        config: &AzureConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Self {
        Self {
            key_client,
            secret_client,
            vault_url: config
                .vault_url
                .clone()
                .unwrap_or_else(|| "https://localhost:8443".to_string()),
            envelope_master_kek_name: config.envelope_master_kek_name.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            master_kek,
        }
    }

    fn is_secret_id(&self, id: &str) -> bool {
        let normalized_prefix = self.secret_name_prefix.replace('/', "-");
        id.starts_with(&self.secret_name_prefix)
            || id.starts_with(&normalized_prefix)
            || id.contains("/secrets/")
    }
}

#[cfg(feature = "azure")]
#[async_trait]
impl CloudKmsProvider for AzureKeyVaultProvider {
    fn provider_name(&self) -> &'static str {
        "azure-keyvault"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let key_name = spec.name.clone().unwrap_or_else(|| {
            let rand_bytes = crate::envelope::AutoEnvelope::generate_random_bytes(8).unwrap();
            format!("key-{}", crate::codec::datetime::octets_to_hex(&rand_bytes))
        });

        let (kty, key_size) = match spec.algorithm.to_uppercase().as_str() {
            "RSA" => {
                let size = match spec.length {
                    3072 => 3072,
                    4096 => 4096,
                    _ => 2048,
                };
                (KeyType::Rsa, Some(size))
            }
            "EC" => (KeyType::Ec, None),
            other => {
                return Err(ProxyError::Unsupported(format!(
                    "unsupported Azure Key Vault algorithm: {other}"
                )));
            }
        };

        let key_ops = if (spec.usage_mask & 0x40) != 0 {
            vec![KeyOperation::Sign, KeyOperation::Verify]
        } else {
            vec![
                KeyOperation::Encrypt,
                KeyOperation::Decrypt,
                KeyOperation::WrapKey,
                KeyOperation::UnwrapKey,
            ]
        };

        let mut azure_tags = HashMap::new();
        for (k, v) in &tags.0 {
            azure_tags.insert(k.clone(), v.clone());
        }

        let params = CreateKeyParameters {
            kty: Some(kty),
            key_size,
            key_ops: Some(key_ops),
            tags: if azure_tags.is_empty() {
                None
            } else {
                Some(azure_tags)
            },
            ..Default::default()
        };

        let req_content = to_request_content(&params)?;

        let resp = self
            .key_client
            .create_key(&key_name, req_content, None)
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Azure create_key error: {e:?}")))?;

        let key = resp
            .into_model()
            .map_err(|e| ProxyError::ProviderError(format!("Azure read key error: {e}")))?;

        let key_id = key
            .key
            .and_then(|k| k.kid)
            .unwrap_or_else(|| key_name.clone());

        Ok(key_id)
    }

    async fn create_envelope_key(
        &self,
        spec: &CreateKeySpec,
        raw_material: Option<Zeroizing<Vec<u8>>>,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let key_len = if spec.length > 0 {
            (spec.length / 8) as usize
        } else {
            32
        };

        let key_bytes = match raw_material {
            Some(mat) => mat,
            None => self.random_bytes(key_len).await?,
        };

        let wrapped_binary = if let Some(kek) = &self.master_kek {
            crate::envelope::AutoEnvelope::wrap_key(kek, &key_bytes)?.to_bytes()
        } else {
            return Err(ProxyError::ConfigError(
                "master_kek required for Azure Key Vault envelope symmetric keys".to_string(),
            ));
        };

        let rand_hex = crate::codec::datetime::octets_to_hex(&self.random_bytes(8).await?);
        let secret_name = format!("{}{}", self.secret_name_prefix.replace('/', "-"), rand_hex);

        let mut azure_tags = HashMap::new();
        for (k, v) in &tags.0 {
            azure_tags.insert(k.clone(), v.clone());
        }

        let secret_str = crate::codec::datetime::octets_to_hex(&wrapped_binary);
        let params = SetSecretParameters {
            value: Some(secret_str),
            tags: if azure_tags.is_empty() {
                None
            } else {
                Some(azure_tags)
            },
            content_type: Some("application/octet-stream".to_string()),
            ..Default::default()
        };

        let req_content = to_request_content(&params)?;

        self.secret_client
            .set_secret(&secret_name, req_content, None)
            .await
            .map_err(|e| ProxyError::ProviderError(format!("Azure set_secret error: {e}")))?;

        Ok(secret_name)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            let resp = self
                .secret_client
                .get_secret(clean_name, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure get_secret error: {e}")))?;

            let secret = resp
                .into_model()
                .map_err(|e| ProxyError::ProviderError(format!("Azure read secret: {e}")))?;

            let hex_val = secret
                .value
                .ok_or_else(|| ProxyError::ProviderError("missing secret value".to_string()))?;

            let bytes = crate::codec::datetime::hex_to_octets(&hex_val)
                .map_err(|e| ProxyError::EnvelopeError(e.to_string()))?;

            let mut tag_map = CloudTagMap::new();
            if let Some(tags) = secret.tags {
                for (k, v) in tags {
                    tag_map.insert(k, v);
                }
            }

            let wrapped = crate::envelope::WrappedEnvelopeKey::from_bytes(&bytes)?;
            let kek = self.master_kek.as_ref().ok_or_else(|| {
                ProxyError::ConfigError("missing master_kek for unwrapping".to_string())
            })?;
            let unwrapped = crate::envelope::AutoEnvelope::unwrap_key(kek, &wrapped)?;

            Ok((KeyMaterialPayload::Symmetric(unwrapped), tag_map))
        } else {
            let resp = self
                .key_client
                .get_key(clean_name, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure get_key error: {e}")))?;

            let key_bundle = resp
                .into_model()
                .map_err(|e| ProxyError::ProviderError(format!("Azure read key: {e}")))?;

            let mut tag_map = CloudTagMap::new();
            if let Some(tags) = key_bundle.tags {
                for (k, v) in tags {
                    tag_map.insert(k, v);
                }
            }

            // Export JWK public key bytes
            let jwk_json = serde_json::to_vec(&key_bundle.key)
                .map_err(|e| ProxyError::Internal(format!("JWK serialization error: {e}")))?;

            Ok((KeyMaterialPayload::PublicKeyDer(jwk_json), tag_map))
        }
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);
        let mut tag_map = CloudTagMap::new();

        if self.is_secret_id(unique_id) {
            let resp = self
                .secret_client
                .get_secret(clean_name, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure get_secret error: {e}")))?;

            let secret = resp
                .into_model()
                .map_err(|e| ProxyError::ProviderError(format!("Azure read secret: {e}")))?;

            if let Some(tags) = secret.tags {
                for (k, v) in tags {
                    tag_map.insert(k, v);
                }
            }

            let enabled = secret
                .attributes
                .as_ref()
                .and_then(|a| a.enabled)
                .unwrap_or(true);
            let created_at = secret
                .attributes
                .as_ref()
                .and_then(|a| a.created)
                .map(|d| d.unix_timestamp());

            Ok(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags: tag_map,
                native_state_enabled: enabled,
                created_at_epoch_secs: created_at,
            })
        } else {
            let resp = self
                .key_client
                .get_key(clean_name, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure get_key error: {e}")))?;

            let key_bundle = resp
                .into_model()
                .map_err(|e| ProxyError::ProviderError(format!("Azure read key: {e}")))?;

            if let Some(tags) = key_bundle.tags {
                for (k, v) in tags {
                    tag_map.insert(k, v);
                }
            }

            let enabled = key_bundle
                .attributes
                .as_ref()
                .and_then(|a| a.enabled)
                .unwrap_or(true);
            let created_at = key_bundle
                .attributes
                .as_ref()
                .and_then(|a| a.created)
                .map(|d| d.unix_timestamp());

            Ok(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags: tag_map,
                native_state_enabled: enabled,
                created_at_epoch_secs: created_at,
            })
        }
    }

    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        let meta = self.get_tags_and_metadata(unique_id).await?;
        let mut current_tags = meta.tags.0;

        for (k, v) in &upsert.0 {
            current_tags.insert(k.clone(), v.clone());
        }
        for k in remove_keys {
            current_tags.remove(k);
        }

        let mut hash_tags = HashMap::new();
        for (k, v) in current_tags {
            hash_tags.insert(k, v);
        }

        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            let params = UpdateSecretPropertiesParameters {
                tags: Some(hash_tags),
                ..Default::default()
            };
            let req_content = to_request_content(&params)?;
            self.secret_client
                .update_secret_properties(clean_name, req_content, None)
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("Azure update_secret error: {e}"))
                })?;
        } else {
            let params = UpdateKeyPropertiesParameters {
                tags: Some(hash_tags),
                ..Default::default()
            };
            let req_content = to_request_content(&params)?;
            self.key_client
                .update_key_properties(clean_name, req_content, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure update_key error: {e}")))?;
        }

        Ok(())
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        // Tag search across Key Vault keys/secrets
        Ok(Vec::new())
    }

    async fn set_native_enabled_state(
        &self,
        unique_id: &str,
        enabled: bool,
    ) -> Result<(), ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            let attrs = SecretAttributes {
                enabled: Some(enabled),
                ..Default::default()
            };
            let params = UpdateSecretPropertiesParameters {
                secret_attributes: Some(attrs),
                ..Default::default()
            };
            let req_content = to_request_content(&params)?;
            self.secret_client
                .update_secret_properties(clean_name, req_content, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure enable/disable: {e}")))?;
        } else {
            let attrs = KeyAttributes {
                enabled: Some(enabled),
                ..Default::default()
            };
            let params = UpdateKeyPropertiesParameters {
                key_attributes: Some(attrs),
                ..Default::default()
            };
            let req_content = to_request_content(&params)?;
            self.key_client
                .update_key_properties(clean_name, req_content, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure enable/disable: {e}")))?;
        }

        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            self.secret_client
                .delete_secret(clean_name, None)
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("Azure delete_secret error: {e}"))
                })?;
        } else {
            self.key_client
                .delete_key(clean_name, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure delete_key error: {e}")))?;
        }

        Ok(())
    }

    async fn encrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            let (mat, _) = self.get_key_material(unique_id).await?;
            match mat {
                KeyMaterialPayload::Symmetric(sym) => {
                    let wrapped = crate::envelope::AutoEnvelope::wrap_key(
                        &sym,
                        &Zeroizing::new(plaintext.to_vec()),
                    )?;
                    Ok(wrapped.to_bytes())
                }
                _ => Err(ProxyError::Unsupported(
                    "secret payload must be symmetric for encrypt".to_string(),
                )),
            }
        } else {
            let params = KeyOperationParameters {
                algorithm: Some(EncryptionAlgorithm::RsaOaep256),
                value: Some(plaintext.to_vec()),
                ..Default::default()
            };
            let req_content = to_request_content(&params)?;

            let resp = self
                .key_client
                .encrypt(clean_name, req_content, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure encrypt error: {e}")))?;

            let result = resp.into_model().map_err(|e| {
                ProxyError::ProviderError(format!("Azure read encrypt result: {e}"))
            })?;

            result
                .result
                .ok_or_else(|| ProxyError::ProviderError("missing encrypt ciphertext".to_string()))
        }
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            let (mat, _) = self.get_key_material(unique_id).await?;
            match mat {
                KeyMaterialPayload::Symmetric(sym) => {
                    let wrapped = crate::envelope::WrappedEnvelopeKey::from_bytes(ciphertext)?;
                    crate::envelope::AutoEnvelope::unwrap_key(&sym, &wrapped)
                }
                _ => Err(ProxyError::Unsupported(
                    "secret payload must be symmetric for decrypt".to_string(),
                )),
            }
        } else {
            let params = KeyOperationParameters {
                algorithm: Some(EncryptionAlgorithm::RsaOaep256),
                value: Some(ciphertext.to_vec()),
                ..Default::default()
            };
            let req_content = to_request_content(&params)?;

            let resp = self
                .key_client
                .decrypt(clean_name, "", req_content, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure decrypt error: {e}")))?;

            let result = resp.into_model().map_err(|e| {
                ProxyError::ProviderError(format!("Azure read decrypt result: {e}"))
            })?;

            result
                .result
                .map(Zeroizing::new)
                .ok_or_else(|| ProxyError::ProviderError("missing decrypt plaintext".to_string()))
        }
    }

    async fn sign(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            let (mat, _) = self.get_key_material(unique_id).await?;
            match mat {
                KeyMaterialPayload::Symmetric(sym) => {
                    use aws_lc_rs::hmac;
                    let s_key = hmac::Key::new(hmac::HMAC_SHA256, &sym);
                    let tag = hmac::sign(&s_key, data);
                    Ok(tag.as_ref().to_vec())
                }
                _ => Err(ProxyError::Unsupported(
                    "secret payload must be symmetric for HMAC sign".to_string(),
                )),
            }
        } else {
            let params = SignParameters {
                algorithm: Some(SignatureAlgorithm::Ps256),
                value: Some(data.to_vec()),
            };
            let req_content = to_request_content(&params)?;

            let resp = self
                .key_client
                .sign(clean_name, req_content, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure sign error: {e}")))?;

            let result = resp
                .into_model()
                .map_err(|e| ProxyError::ProviderError(format!("Azure read sign result: {e}")))?;

            result
                .result
                .ok_or_else(|| ProxyError::ProviderError("missing signature result".to_string()))
        }
    }

    async fn verify(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError> {
        let clean_name = unique_id.split('/').next_back().unwrap_or(unique_id);

        if self.is_secret_id(unique_id) {
            let (mat, _) = self.get_key_material(unique_id).await?;
            match mat {
                KeyMaterialPayload::Symmetric(sym) => {
                    use aws_lc_rs::hmac;
                    let s_key = hmac::Key::new(hmac::HMAC_SHA256, &sym);
                    Ok(hmac::verify(&s_key, data, signature).is_ok())
                }
                _ => Err(ProxyError::Unsupported(
                    "secret payload must be symmetric for HMAC verify".to_string(),
                )),
            }
        } else {
            let params = VerifyParameters {
                algorithm: Some(SignatureAlgorithm::Ps256),
                digest: Some(data.to_vec()),
                signature: Some(signature.to_vec()),
            };
            let req_content = to_request_content(&params)?;

            let resp = self
                .key_client
                .verify(clean_name, "", req_content, None)
                .await
                .map_err(|e| ProxyError::ProviderError(format!("Azure verify error: {e}")))?;

            let result = resp
                .into_model()
                .map_err(|e| ProxyError::ProviderError(format!("Azure read verify result: {e}")))?;

            Ok(result.value.unwrap_or(false))
        }
    }

    async fn random_bytes(&self, length: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let params = GetRandomBytesParameters {
            count: Some(length as i32),
        };
        let req_content = match to_request_content(&params) {
            Ok(rc) => rc,
            Err(_) => return crate::envelope::AutoEnvelope::generate_random_bytes(length),
        };

        let resp = self.key_client.get_random_bytes(req_content, None).await;

        match resp {
            Ok(r) => match r.into_model() {
                Ok(body) => body
                    .value
                    .map(Zeroizing::new)
                    .ok_or_else(|| ProxyError::ProviderError("missing random bytes".to_string())),
                Err(_) => crate::envelope::AutoEnvelope::generate_random_bytes(length),
            },
            Err(_) => crate::envelope::AutoEnvelope::generate_random_bytes(length),
        }
    }
}

#[cfg(not(feature = "azure"))]
pub struct AzureKeyVaultProvider {
    pub vault_url: String,
    pub envelope_master_kek_name: Option<String>,
}

#[cfg(not(feature = "azure"))]
impl AzureKeyVaultProvider {
    pub fn new(config: &AzureConfig) -> Result<Self, ProxyError> {
        Ok(Self {
            vault_url: config
                .vault_url
                .clone()
                .unwrap_or_else(|| "https://localhost:8443".to_string()),
            envelope_master_kek_name: config.envelope_master_kek_name.clone(),
        })
    }
}

#[cfg(not(feature = "azure"))]
#[async_trait]
impl CloudKmsProvider for AzureKeyVaultProvider {
    fn provider_name(&self) -> &'static str {
        "azure-keyvault"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
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
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "Azure Key Vault feature not enabled".to_string(),
        ))
    }
}
