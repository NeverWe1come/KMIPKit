//! AWS KMS and Secrets Manager provider connector.
#![forbid(unsafe_code)]

#[cfg(feature = "aws")]
use std::collections::BTreeMap;

use async_trait::async_trait;
use zeroize::Zeroizing;

use super::{
    CloudKmsProvider, CloudResourceMeta, CloudTagFilter, CloudTagMap, CreateKeySpec, CryptoParams,
    KeyMaterialPayload,
};
use crate::ProxyError;
use crate::config::AwsConfig;

#[cfg(feature = "aws")]
pub struct AwsKmsProvider {
    kms: aws_sdk_kms::Client,
    secrets: aws_sdk_secretsmanager::Client,
    pub region: String,
    pub envelope_master_kek_id: Option<String>,
    pub secret_name_prefix: String,
    pub master_kek: Option<[u8; 32]>,
}

#[cfg(feature = "aws")]
impl AwsKmsProvider {
    pub fn new(config: &AwsConfig) -> Result<Self, ProxyError> {
        let region = config
            .region
            .clone()
            .unwrap_or_else(|| "us-east-1".to_string());
        let mut kms_builder = aws_sdk_kms::config::Builder::new()
            .behavior_version(aws_sdk_kms::config::BehaviorVersion::latest())
            .region(aws_sdk_kms::config::Region::new(region.clone()));
        let mut secrets_builder = aws_sdk_secretsmanager::config::Builder::new()
            .behavior_version(aws_sdk_secretsmanager::config::BehaviorVersion::latest())
            .region(aws_sdk_secretsmanager::config::Region::new(region.clone()));

        if let Some(endpoint) = &config.endpoint_url {
            kms_builder = kms_builder.endpoint_url(endpoint);
            secrets_builder = secrets_builder.endpoint_url(endpoint);
        }

        let kms = aws_sdk_kms::Client::from_conf(kms_builder.build());
        let secrets = aws_sdk_secretsmanager::Client::from_conf(secrets_builder.build());

        Ok(Self {
            kms,
            secrets,
            region,
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            master_kek: None,
        })
    }

    pub async fn from_config(
        config: &AwsConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Result<Self, ProxyError> {
        let region = config
            .region
            .clone()
            .unwrap_or_else(|| "us-east-1".to_string());
        let mut loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()));
        if let Some(endpoint) = &config.endpoint_url {
            loader = loader.endpoint_url(endpoint);
        }
        let sdk_config = loader.load().await;

        let kms_config = aws_sdk_kms::config::Builder::from(&sdk_config).build();
        let secrets_config = aws_sdk_secretsmanager::config::Builder::from(&sdk_config).build();

        Ok(Self {
            kms: aws_sdk_kms::Client::from_conf(kms_config),
            secrets: aws_sdk_secretsmanager::Client::from_conf(secrets_config),
            region,
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            master_kek,
        })
    }

    #[must_use]
    pub fn with_clients(
        kms: aws_sdk_kms::Client,
        secrets: aws_sdk_secretsmanager::Client,
        config: &AwsConfig,
        master_kek: Option<[u8; 32]>,
    ) -> Self {
        Self {
            kms,
            secrets,
            region: config
                .region
                .clone()
                .unwrap_or_else(|| "us-east-1".to_string()),
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
            master_kek,
        }
    }

    fn is_secret_id(&self, id: &str) -> bool {
        id.starts_with(&self.secret_name_prefix) || id.contains(":secretsmanager:")
    }
}

#[cfg(feature = "aws")]
#[async_trait]
impl CloudKmsProvider for AwsKmsProvider {
    fn provider_name(&self) -> &'static str {
        "aws-kms"
    }

    async fn create_native_key(
        &self,
        spec: &CreateKeySpec,
        tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        let (key_spec, key_usage) = match spec.algorithm.to_uppercase().as_str() {
            "AES" => (
                aws_sdk_kms::types::KeySpec::SymmetricDefault,
                aws_sdk_kms::types::KeyUsageType::EncryptDecrypt,
            ),
            "RSA" => {
                let rsa_spec = match spec.length {
                    3072 => aws_sdk_kms::types::KeySpec::Rsa3072,
                    4096 => aws_sdk_kms::types::KeySpec::Rsa4096,
                    _ => aws_sdk_kms::types::KeySpec::Rsa2048,
                };
                let usage = if (spec.usage_mask & 0x40) != 0 {
                    aws_sdk_kms::types::KeyUsageType::SignVerify
                } else {
                    aws_sdk_kms::types::KeyUsageType::EncryptDecrypt
                };
                (rsa_spec, usage)
            }
            "EC" => (
                aws_sdk_kms::types::KeySpec::EccNistP256,
                aws_sdk_kms::types::KeyUsageType::SignVerify,
            ),
            other => {
                return Err(ProxyError::Unsupported(format!(
                    "unsupported AWS KMS algorithm: {other}"
                )));
            }
        };

        let mut req = self
            .kms
            .create_key()
            .key_spec(key_spec)
            .key_usage(key_usage);
        if let Some(name) = &spec.name {
            req = req.description(name);
        }

        let output = req
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("AWS KMS create_key error: {e}")))?;

        let key_id = output
            .key_metadata
            .map(|m| m.key_id)
            .ok_or_else(|| ProxyError::ProviderError("missing key_id in create_key".to_string()))?;

        if !tags.is_empty() {
            let mut aws_tags = Vec::new();
            for (k, v) in &tags.0 {
                aws_tags.push(
                    aws_sdk_kms::types::Tag::builder()
                        .tag_key(k)
                        .tag_value(v)
                        .build()
                        .map_err(|e| ProxyError::ProviderError(e.to_string()))?,
                );
            }
            self.kms
                .tag_resource()
                .key_id(&key_id)
                .set_tags(Some(aws_tags))
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS KMS tag_resource error: {e}"))
                })?;
        }

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

        let secret_binary = if let Some(kek) = &self.master_kek {
            crate::envelope::AutoEnvelope::wrap_key(kek, &key_bytes)?.to_bytes()
        } else if let Some(kek_id) = &self.envelope_master_kek_id {
            let enc_out = self
                .kms
                .encrypt()
                .key_id(kek_id)
                .plaintext(aws_sdk_kms::primitives::Blob::new(key_bytes.to_vec()))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("KMS KEK encrypt error: {e}")))?;
            enc_out
                .ciphertext_blob
                .ok_or_else(|| ProxyError::ProviderError("missing ciphertext".to_string()))?
                .into_inner()
        } else {
            return Err(ProxyError::ConfigError(
                "neither master_kek nor envelope_master_kek_id configured for AWS envelope"
                    .to_string(),
            ));
        };

        let rand_hex = crate::codec::datetime::octets_to_hex(&self.random_bytes(16).await?);
        let secret_name = format!("{}{}", self.secret_name_prefix, rand_hex);

        let mut sec_tags = Vec::new();
        for (k, v) in &tags.0 {
            sec_tags.push(
                aws_sdk_secretsmanager::types::Tag::builder()
                    .key(k)
                    .value(v)
                    .build(),
            );
        }

        self.secrets
            .create_secret()
            .name(&secret_name)
            .secret_binary(aws_sdk_secretsmanager::primitives::Blob::new(secret_binary))
            .set_tags(if sec_tags.is_empty() {
                None
            } else {
                Some(sec_tags)
            })
            .send()
            .await
            .map_err(|e| {
                ProxyError::ProviderError(format!("AWS SecretsManager create_secret error: {e}"))
            })?;

        Ok(secret_name)
    }

    async fn get_key_material(
        &self,
        unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        if self.is_secret_id(unique_id) {
            let sec_out = self
                .secrets
                .get_secret_value()
                .secret_id(unique_id)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS SecretsManager get_secret error: {e}"))
                })?;

            let meta = self.get_tags_and_metadata(unique_id).await?;

            let binary = sec_out
                .secret_binary
                .ok_or_else(|| ProxyError::ProviderError("missing secret_binary".to_string()))?
                .into_inner();

            let unwrapped = if let Some(kek) = &self.master_kek {
                let wrapped = crate::envelope::WrappedEnvelopeKey::from_bytes(&binary)?;
                crate::envelope::AutoEnvelope::unwrap_key(kek, &wrapped)?
            } else if let Some(kek_id) = &self.envelope_master_kek_id {
                let dec_out = self
                    .kms
                    .decrypt()
                    .key_id(kek_id)
                    .ciphertext_blob(aws_sdk_kms::primitives::Blob::new(binary))
                    .send()
                    .await
                    .map_err(|e| {
                        ProxyError::ProviderError(format!("KMS KEK decrypt error: {e}"))
                    })?;
                let pt = dec_out
                    .plaintext
                    .ok_or_else(|| ProxyError::ProviderError("missing plaintext".to_string()))?
                    .into_inner();
                Zeroizing::new(pt)
            } else {
                return Err(ProxyError::ConfigError(
                    "no KEK available to unwrap envelope secret".to_string(),
                ));
            };

            Ok((KeyMaterialPayload::Symmetric(unwrapped), meta.tags))
        } else {
            let pk_out = self
                .kms
                .get_public_key()
                .key_id(unique_id)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS KMS get_public_key error: {e}"))
                })?;

            let meta = self.get_tags_and_metadata(unique_id).await?;

            let der = pk_out
                .public_key
                .ok_or_else(|| ProxyError::ProviderError("missing public_key in KMS".to_string()))?
                .into_inner();

            Ok((KeyMaterialPayload::PublicKeyDer(der), meta.tags))
        }
    }

    async fn get_tags_and_metadata(
        &self,
        unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        let mut tag_map = CloudTagMap::new();

        if self.is_secret_id(unique_id) {
            let desc = self
                .secrets
                .describe_secret()
                .secret_id(unique_id)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS SecretsManager describe error: {e}"))
                })?;

            if let Some(tags) = desc.tags {
                for t in tags {
                    if let (Some(k), Some(v)) = (t.key, t.value) {
                        tag_map.insert(k, v);
                    }
                }
            }

            let native_state_enabled = desc.deleted_date.is_none();
            let created_at_epoch_secs = desc.created_date.map(|d| d.secs());

            Ok(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags: tag_map,
                native_state_enabled,
                created_at_epoch_secs,
            })
        } else {
            let desc = self
                .kms
                .describe_key()
                .key_id(unique_id)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS KMS describe_key error: {e}"))
                })?;

            let (native_state_enabled, created_at_epoch_secs) =
                if let Some(meta) = desc.key_metadata {
                    (meta.enabled, meta.creation_date.map(|d| d.secs()))
                } else {
                    (false, None)
                };

            let tags_out = self
                .kms
                .list_resource_tags()
                .key_id(unique_id)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS KMS list_resource_tags error: {e}"))
                })?;

            if let Some(tags) = tags_out.tags {
                for t in tags {
                    tag_map.insert(t.tag_key, t.tag_value);
                }
            }

            Ok(CloudResourceMeta {
                unique_id: unique_id.to_string(),
                tags: tag_map,
                native_state_enabled,
                created_at_epoch_secs,
            })
        }
    }

    async fn update_tags(
        &self,
        unique_id: &str,
        upsert: &CloudTagMap,
        remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        if self.is_secret_id(unique_id) {
            if !upsert.is_empty() {
                let mut sec_tags = Vec::new();
                for (k, v) in &upsert.0 {
                    sec_tags.push(
                        aws_sdk_secretsmanager::types::Tag::builder()
                            .key(k)
                            .value(v)
                            .build(),
                    );
                }
                self.secrets
                    .tag_resource()
                    .secret_id(unique_id)
                    .set_tags(Some(sec_tags))
                    .send()
                    .await
                    .map_err(|e| {
                        ProxyError::ProviderError(format!("AWS SecretsManager tag error: {e}"))
                    })?;
            }

            if !remove_keys.is_empty() {
                self.secrets
                    .untag_resource()
                    .secret_id(unique_id)
                    .set_tag_keys(Some(remove_keys.to_vec()))
                    .send()
                    .await
                    .map_err(|e| {
                        ProxyError::ProviderError(format!("AWS SecretsManager untag error: {e}"))
                    })?;
            }
        } else {
            if !upsert.is_empty() {
                let mut aws_tags = Vec::new();
                for (k, v) in &upsert.0 {
                    aws_tags.push(
                        aws_sdk_kms::types::Tag::builder()
                            .tag_key(k)
                            .tag_value(v)
                            .build()
                            .map_err(|e| ProxyError::ProviderError(e.to_string()))?,
                    );
                }
                self.kms
                    .tag_resource()
                    .key_id(unique_id)
                    .set_tags(Some(aws_tags))
                    .send()
                    .await
                    .map_err(|e| {
                        ProxyError::ProviderError(format!("AWS KMS tag_resource error: {e}"))
                    })?;
            }

            if !remove_keys.is_empty() {
                self.kms
                    .untag_resource()
                    .key_id(unique_id)
                    .set_tag_keys(Some(remove_keys.to_vec()))
                    .send()
                    .await
                    .map_err(|e| {
                        ProxyError::ProviderError(format!("AWS KMS untag_resource error: {e}"))
                    })?;
            }
        }

        Ok(())
    }

    async fn locate_by_tags(
        &self,
        filter: &CloudTagFilter,
        max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        let limit = max_items.unwrap_or(100) as usize;
        let mut results = Vec::new();

        // 1. Search Secrets Manager
        let sec_list = self.secrets.list_secrets().send().await.map_err(|e| {
            ProxyError::ProviderError(format!("AWS SecretsManager list_secrets error: {e}"))
        })?;

        if let Some(entries) = sec_list.secret_list {
            for s in entries {
                let mut matches = true;
                let mut tag_map = BTreeMap::new();
                if let Some(tags) = s.tags {
                    for t in tags {
                        if let (Some(k), Some(v)) = (t.key, t.value) {
                            tag_map.insert(k, v);
                        }
                    }
                }
                for (fk, fv) in &filter.exact_tags {
                    if tag_map.get(fk) != Some(fv) {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    if let Some(name) = s.name {
                        results.push(name);
                        if results.len() >= limit {
                            return Ok(results);
                        }
                    }
                }
            }
        }

        // 2. Search KMS Keys
        let key_list = self
            .kms
            .list_keys()
            .send()
            .await
            .map_err(|e| ProxyError::ProviderError(format!("AWS KMS list_keys error: {e}")))?;

        if let Some(keys) = key_list.keys {
            for k in keys {
                if let Some(kid) = k.key_id {
                    let meta = match self.get_tags_and_metadata(&kid).await {
                        Ok(m) => m,
                        Err(_) => continue,
                    };
                    let mut matches = true;
                    for (fk, fv) in &filter.exact_tags {
                        if meta.tags.get(fk) != Some(fv) {
                            matches = false;
                            break;
                        }
                    }
                    if matches {
                        results.push(kid);
                        if results.len() >= limit {
                            return Ok(results);
                        }
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
        if !self.is_secret_id(unique_id) {
            if enabled {
                self.kms
                    .enable_key()
                    .key_id(unique_id)
                    .send()
                    .await
                    .map_err(|e| {
                        ProxyError::ProviderError(format!("AWS KMS enable_key error: {e}"))
                    })?;
            } else {
                self.kms
                    .disable_key()
                    .key_id(unique_id)
                    .send()
                    .await
                    .map_err(|e| {
                        ProxyError::ProviderError(format!("AWS KMS disable_key error: {e}"))
                    })?;
            }
        }
        Ok(())
    }

    async fn schedule_destruction(&self, unique_id: &str) -> Result<(), ProxyError> {
        if self.is_secret_id(unique_id) {
            self.secrets
                .delete_secret()
                .secret_id(unique_id)
                .recovery_window_in_days(7)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS SecretsManager delete error: {e}"))
                })?;
        } else {
            self.kms
                .schedule_key_deletion()
                .key_id(unique_id)
                .pending_window_in_days(7)
                .send()
                .await
                .map_err(|e| {
                    ProxyError::ProviderError(format!("AWS KMS schedule_deletion error: {e}"))
                })?;
        }
        Ok(())
    }

    async fn encrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
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
            let output = self
                .kms
                .encrypt()
                .key_id(unique_id)
                .plaintext(aws_sdk_kms::primitives::Blob::new(plaintext))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("AWS KMS encrypt error: {e}")))?;

            output
                .ciphertext_blob
                .map(|b| b.into_inner())
                .ok_or_else(|| ProxyError::ProviderError("missing ciphertext".to_string()))
        }
    }

    async fn decrypt(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
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
            let output = self
                .kms
                .decrypt()
                .key_id(unique_id)
                .ciphertext_blob(aws_sdk_kms::primitives::Blob::new(ciphertext))
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("AWS KMS decrypt error: {e}")))?;

            output
                .plaintext
                .map(|b| Zeroizing::new(b.into_inner()))
                .ok_or_else(|| ProxyError::ProviderError("missing plaintext".to_string()))
        }
    }

    async fn sign(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
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
            let output = self
                .kms
                .sign()
                .key_id(unique_id)
                .message(aws_sdk_kms::primitives::Blob::new(data))
                .message_type(aws_sdk_kms::types::MessageType::Raw)
                .signing_algorithm(aws_sdk_kms::types::SigningAlgorithmSpec::RsassaPssSha256)
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("AWS KMS sign error: {e}")))?;

            output
                .signature
                .map(|b| b.into_inner())
                .ok_or_else(|| ProxyError::ProviderError("missing signature".to_string()))
        }
    }

    async fn verify(
        &self,
        unique_id: &str,
        _params: &CryptoParams,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, ProxyError> {
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
            let output = self
                .kms
                .verify()
                .key_id(unique_id)
                .message(aws_sdk_kms::primitives::Blob::new(data))
                .message_type(aws_sdk_kms::types::MessageType::Raw)
                .signature(aws_sdk_kms::primitives::Blob::new(signature))
                .signing_algorithm(aws_sdk_kms::types::SigningAlgorithmSpec::RsassaPssSha256)
                .send()
                .await
                .map_err(|e| ProxyError::ProviderError(format!("AWS KMS verify error: {e}")))?;

            Ok(output.signature_valid)
        }
    }

    async fn random_bytes(&self, length: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        let output = self
            .kms
            .generate_random()
            .number_of_bytes(length as i32)
            .send()
            .await;

        match output {
            Ok(out) => out
                .plaintext
                .map(|b| Zeroizing::new(b.into_inner()))
                .ok_or_else(|| ProxyError::ProviderError("missing random bytes".to_string())),
            Err(_) => crate::envelope::AutoEnvelope::generate_random_bytes(length),
        }
    }
}

#[cfg(not(feature = "aws"))]
pub struct AwsKmsProvider {
    pub region: String,
    pub envelope_master_kek_id: Option<String>,
    pub secret_name_prefix: String,
}

#[cfg(not(feature = "aws"))]
impl AwsKmsProvider {
    pub fn new(config: &AwsConfig) -> Result<Self, ProxyError> {
        Ok(Self {
            region: config
                .region
                .clone()
                .unwrap_or_else(|| "us-east-1".to_string()),
            envelope_master_kek_id: config.envelope_master_kek_id.clone(),
            secret_name_prefix: config
                .secret_name_prefix
                .clone()
                .unwrap_or_else(|| "kmipkit/".to_string()),
        })
    }
}

#[cfg(not(feature = "aws"))]
#[async_trait]
impl CloudKmsProvider for AwsKmsProvider {
    fn provider_name(&self) -> &'static str {
        "aws-kms"
    }

    async fn create_native_key(
        &self,
        _spec: &CreateKeySpec,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn create_envelope_key(
        &self,
        _spec: &CreateKeySpec,
        _raw_material: Option<Zeroizing<Vec<u8>>>,
        _tags: &CloudTagMap,
    ) -> Result<String, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn get_key_material(
        &self,
        _unique_id: &str,
    ) -> Result<(KeyMaterialPayload, CloudTagMap), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn get_tags_and_metadata(
        &self,
        _unique_id: &str,
    ) -> Result<CloudResourceMeta, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn update_tags(
        &self,
        _unique_id: &str,
        _upsert: &CloudTagMap,
        _remove_keys: &[String],
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn locate_by_tags(
        &self,
        _filter: &CloudTagFilter,
        _max_items: Option<u32>,
    ) -> Result<Vec<String>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn set_native_enabled_state(
        &self,
        _unique_id: &str,
        _enabled: bool,
    ) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn schedule_destruction(&self, _unique_id: &str) -> Result<(), ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn encrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _plaintext: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn decrypt(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _ciphertext: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn sign(
        &self,
        _unique_id: &str,
        _params: &CryptoParams,
        _data: &[u8],
    ) -> Result<Vec<u8>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
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
            "AWS feature not enabled".to_string(),
        ))
    }

    async fn random_bytes(&self, _num_bytes: usize) -> Result<Zeroizing<Vec<u8>>, ProxyError> {
        Err(ProxyError::ProviderNotConfigured(
            "AWS feature not enabled".to_string(),
        ))
    }
}
