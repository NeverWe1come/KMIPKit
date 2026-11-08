//! Integration tests for KMIPKit Proxy with Azure Key Vault and Secrets.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::format_push_string,
    clippy::semicolon_if_nothing_returned,
    clippy::unreadable_literal,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::collapsible_if,
    clippy::items_after_statements,
    clippy::manual_is_multiple_of,
    clippy::return_self_not_must_use,
    clippy::unnecessary_wraps,
    clippy::vec_init_then_push,
    clippy::bool_to_int_with_if
)]

#[cfg(feature = "azure")]
mod azure_tests {
    use std::sync::Arc;
    use std::time::Duration;
    use zeroize::Zeroizing;

    use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
    use azure_core::http::headers::{AUTHORIZATION, CONTENT_TYPE, Headers, WWW_AUTHENTICATE};
    use azure_core::http::{AsyncRawResponse, HttpClient, Request, StatusCode, Transport};
    use azure_security_keyvault_keys::KeyClient;
    use azure_security_keyvault_secrets::SecretClient;

    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{AzureConfig, ProxyConfig};
    use kmipkit_proxy::providers::azure::AzureKeyVaultProvider;
    use kmipkit_proxy::providers::{CloudKmsProvider, CloudTagMap, CreateKeySpec, CryptoParams};
    use kmipkit_proxy::server::ProxyServer;
    use kmipkit_transport::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
        TransportConfig, TrustSource,
    };
    use kmipkit_ttlv::codec::CodecLimits;

    #[derive(Debug, Default)]
    pub struct DummyTokenCredential;

    #[async_trait::async_trait]
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

    #[derive(Debug)]
    struct MockAzureHttpClient;

    #[async_trait::async_trait]
    impl HttpClient for MockAzureHttpClient {
        async fn execute_request(&self, request: &Request) -> azure_core::Result<AsyncRawResponse> {
            let path = request.url().path();
            let method = request.method().as_str();

            // Check authorization header; if absent, trigger 401 challenge
            let auth_hdr = request.headers().get_str(&AUTHORIZATION);
            if auth_hdr.is_err() {
                let mut headers = Headers::new();
                headers.insert(
                    WWW_AUTHENTICATE,
                    "Bearer authorization=\"https://login.microsoftonline.com/dummy\", resource=\"https://vault.azure.net\"",
                );
                return Ok(AsyncRawResponse::from_bytes(
                    StatusCode::Unauthorized,
                    headers,
                    bytes::Bytes::new(),
                ));
            }

            let body: String = if path.contains("/create") {
                r#"{"key":{"kid":"https://myvault.vault.azure.net/keys/test-key/v1","kty":"RSA"},"attributes":{"enabled":true,"created":1700000000},"tags":{"Env":"IntegrationTest"}}"#.to_string()
            } else if path.starts_with("/secrets") && (method == "PUT" || method == "POST") {
                r#"{"id":"https://myvault.vault.azure.net/secrets/test-secret/v1","value":"7b226b6579223a202276616c227d","attributes":{"enabled":true,"created":1700000000},"tags":{"Env":"IntegrationTest"}}"#.to_string()
            } else if path.starts_with("/secrets") && method == "GET" {
                let master_kek = [0x42u8; 32];
                let raw = Zeroizing::new(vec![0x33u8; 32]);
                let wrapped =
                    kmipkit_proxy::envelope::AutoEnvelope::wrap_key(&master_kek, &raw).unwrap();
                let hex_val = kmipkit_proxy::codec::datetime::octets_to_hex(&wrapped.to_bytes());
                format!(
                    r#"{{"id":"https://myvault.vault.azure.net/secrets/test-secret/v1","value":"{hex_val}","attributes":{{"enabled":true,"created":1700000000}},"tags":{{"Env":"IntegrationTest"}}}}"#
                )
            } else if path.starts_with("/keys") && method == "GET" {
                r#"{"key":{"kid":"https://myvault.vault.azure.net/keys/test-key/v1","kty":"RSA","n":"AQAB","e":"AQAB"},"attributes":{"enabled":true,"created":1700000000},"tags":{"Env":"IntegrationTest"}}"#.to_string()
            } else if path.contains("/encrypt")
                || path.contains("/decrypt")
                || path.contains("/sign")
            {
                r#"{"kid":"https://myvault.vault.azure.net/keys/test-key/v1","value":"AAAA"}"#
                    .to_string()
            } else if path.contains("/verify") {
                r#"{"value":true}"#.to_string()
            } else if path.contains("/rng") {
                r#"{"value":"AQIDBAUGBwgJCgsMDQ4PEA=="}"#.to_string()
            } else if method == "PATCH" && path.starts_with("/secrets") {
                r#"{"id":"https://myvault.vault.azure.net/secrets/test-secret/v1","attributes":{"enabled":false},"tags":{"Updated":"true"}}"#.to_string()
            } else if method == "PATCH" && path.starts_with("/keys") {
                r#"{"key":{"kid":"https://myvault.vault.azure.net/keys/test-key/v1"},"attributes":{"enabled":false},"tags":{"Updated":"true"}}"#.to_string()
            } else if method == "DELETE" {
                r#"{"recoveryId":"https://myvault.vault.azure.net/deleted/item"}"#.to_string()
            } else {
                r#"{"status":"ok"}"#.to_string()
            };

            let mut headers = Headers::new();
            headers.insert(CONTENT_TYPE, "application/json");

            Ok(AsyncRawResponse::from_bytes(
                StatusCode::Ok,
                headers,
                bytes::Bytes::from(body),
            ))
        }
    }

    fn build_client(endpoint: Endpoint, pki: &AutoPki, encoding: WireEncoding) -> Client {
        let certificates = vec![pki.client_cert_der.clone()];
        let private_key = pki.client_key_der.to_vec();

        let transport_config = TransportConfig::builder(endpoint)
            .client_identity(ClientIdentity::new(
                CertificateInput::from_der(certificates),
                PrivateKeyInput::from_der(private_key),
            ))
            .trust_source(TrustSource::certificate_authorities(vec![
                CertificateInput::from_der(vec![pki.ca_cert_der.clone()]),
            ]))
            .timeouts(
                TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(Duration::from_secs(5))),
            )
            .build()
            .expect("transport config builds cleanly");

        let registry =
            client_extension_registry(Vec::new(), kmipkit_protocol::extension::defaults())
                .expect("empty registry is valid");
        let client_config = ClientConfiguration::new(registry).with_wire_encoding(encoding);

        Client::new(client_config, transport_config).expect("client builds cleanly")
    }

    #[tokio::test]
    async fn test_azure_provider_config_initialization() {
        let config = AzureConfig {
            vault_url: Some("https://my-test-vault.vault.azure.net".to_string()),
            endpoint_url: None,
            envelope_master_kek_name: Some("test-kek".to_string()),
            secret_name_prefix: Some("kmip-custom/".to_string()),
        };

        let master_kek = [0x55u8; 32];
        let provider = AzureKeyVaultProvider::from_config(&config, Some(master_kek))
            .expect("AzureKeyVaultProvider builds cleanly");

        assert_eq!(provider.provider_name(), "azure-keyvault");
        assert_eq!(provider.vault_url, "https://my-test-vault.vault.azure.net");
        assert_eq!(provider.secret_name_prefix, "kmip-custom/");
        assert_eq!(
            provider.envelope_master_kek_name.as_deref(),
            Some("test-kek")
        );
        assert_eq!(provider.master_kek, Some(master_kek));
    }

    #[tokio::test]
    async fn test_azure_envelope_key_crypto_operations() {
        let master_kek = [0x77u8; 32];
        let raw_key = Zeroizing::new(vec![0x11u8; 32]);

        // Wrap key via AutoEnvelope
        let wrapped = kmipkit_proxy::envelope::AutoEnvelope::wrap_key(&master_kek, &raw_key)
            .expect("wrap_key succeeds");
        let serialized = wrapped.to_bytes();

        // Unwrap key via AutoEnvelope
        let deserialized = kmipkit_proxy::envelope::WrappedEnvelopeKey::from_bytes(&serialized)
            .expect("deserialization succeeds");
        let unwrapped =
            kmipkit_proxy::envelope::AutoEnvelope::unwrap_key(&master_kek, &deserialized)
                .expect("unwrap_key succeeds");

        assert_eq!(*unwrapped, *raw_key);
    }

    #[tokio::test]
    async fn test_azure_keyvault_mock_lifecycle() {
        let mock_transport = Transport::new(Arc::new(MockAzureHttpClient));

        let mut key_opts = azure_security_keyvault_keys::KeyClientOptions::default();
        key_opts.client_options.transport = Some(mock_transport.clone());
        key_opts.verify_challenge_resource = Some(false);

        let mut secret_opts = azure_security_keyvault_secrets::SecretClientOptions::default();
        secret_opts.client_options.transport = Some(mock_transport);
        secret_opts.verify_challenge_resource = Some(false);

        let cred: Arc<dyn TokenCredential> = Arc::new(DummyTokenCredential);
        let key_client = KeyClient::new(
            "https://myvault.vault.azure.net",
            cred.clone(),
            Some(key_opts),
        )
        .unwrap();
        let secret_client =
            SecretClient::new("https://myvault.vault.azure.net", cred, Some(secret_opts)).unwrap();

        let config = AzureConfig {
            vault_url: Some("https://myvault.vault.azure.net".to_string()),
            endpoint_url: None,
            envelope_master_kek_name: Some("test-kek".to_string()),
            secret_name_prefix: Some("kmipkit/".to_string()),
        };

        let master_kek = [0x42u8; 32];
        let provider = AzureKeyVaultProvider::with_clients(
            key_client,
            secret_client,
            &config,
            Some(master_kek),
        );

        // 1. Create native key
        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());
        let spec = CreateKeySpec {
            algorithm: "RSA".to_string(),
            length: 2048,
            usage_mask: 0x0c,
            name: Some("test-rsa-key".to_string()),
        };

        let key_id = provider.create_native_key(&spec, &tags).await.unwrap();
        assert!(!key_id.is_empty());

        // 2. Create envelope key
        let env_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-secret".to_string()),
        };
        let secret_id = provider
            .create_envelope_key(&env_spec, None, &tags)
            .await
            .unwrap();
        assert!(!secret_id.is_empty());

        // 3. Get key material (secret)
        let (mat, meta_tags) = provider.get_key_material(&secret_id).await.unwrap();
        assert_eq!(meta_tags.get("Env"), Some(&"IntegrationTest".to_string()));
        match mat {
            kmipkit_proxy::providers::KeyMaterialPayload::Symmetric(raw) => {
                assert_eq!(raw.len(), 32);
            }
            _ => panic!("Expected symmetric payload"),
        }

        // 4. Get tags and metadata
        let meta = provider.get_tags_and_metadata(&key_id).await.unwrap();
        assert!(meta.native_state_enabled);

        // 5. Update tags
        let mut update_tags = CloudTagMap::new();
        update_tags.insert("Updated".to_string(), "true".to_string());
        provider
            .update_tags(&key_id, &update_tags, &[])
            .await
            .unwrap();

        // 6. Disable state
        provider
            .set_native_enabled_state(&key_id, false)
            .await
            .unwrap();

        // 7. Encrypt / Decrypt
        let dummy_params = CryptoParams {
            algorithm: "RSA".to_string(),
            padding: None,
            hashing_algorithm: None,
        };
        let ciphertext = provider
            .encrypt(&key_id, &dummy_params, b"test-data")
            .await
            .unwrap();
        assert!(!ciphertext.is_empty());
        let plaintext = provider
            .decrypt(&key_id, &dummy_params, &ciphertext)
            .await
            .unwrap();
        assert!(!plaintext.is_empty());

        // 8. Sign / Verify
        let signature = provider
            .sign(&key_id, &dummy_params, b"message")
            .await
            .unwrap();
        assert!(!signature.is_empty());
        let valid = provider
            .verify(&key_id, &dummy_params, b"message", &signature)
            .await
            .unwrap();
        assert!(valid);

        // 9. Random bytes
        let rand = provider.random_bytes(16).await.unwrap();
        assert_eq!(rand.len(), 16);

        // 10. Destruction
        provider.schedule_destruction(&key_id).await.unwrap();
        provider.schedule_destruction(&secret_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_azure_proxy_server_end_to_end() {
        let config = AzureConfig {
            vault_url: Some("https://myvault.vault.azure.net".to_string()),
            endpoint_url: None,
            envelope_master_kek_name: Some("test-kek".to_string()),
            secret_name_prefix: Some("kmipkit/".to_string()),
        };

        let master_kek = [0x42u8; 32];
        let provider = Arc::new(
            AzureKeyVaultProvider::from_config(&config, Some(master_kek))
                .expect("provider creates cleanly"),
        );

        let mut proxy_cfg = ProxyConfig::default();
        proxy_cfg.listener.bind_addr = "127.0.0.1:0".parse().unwrap();

        let server = ProxyServer::start(proxy_cfg, provider)
            .await
            .expect("server creates cleanly");

        let pki = server.pki().expect("pki was generated");
        let endpoint = Endpoint::raw_tls("127.0.0.1", server.port());
        let mut client = build_client(endpoint, pki, WireEncoding::Ttlv);

        let outcome = tokio::task::spawn_blocking(move || {
            let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
            client
                .execute(batch, &CodecLimits::defaults())
                .expect("discover_versions succeeds against Azure-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
