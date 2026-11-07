//! Integration tests for KMIPKit Proxy with Oracle OCI Vault and Secrets.
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

#[cfg(feature = "oci")]
mod oci_tests {
    use base64::Engine;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use zeroize::Zeroizing;

    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{OciConfig, ProxyConfig};
    use kmipkit_proxy::providers::oci::OciVaultProvider;
    use kmipkit_proxy::providers::{CloudKmsProvider, CloudTagMap, CreateKeySpec, CryptoParams};
    use kmipkit_proxy::server::ProxyServer;
    use kmipkit_transport::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
        TransportConfig, TrustSource,
    };
    use kmipkit_ttlv::codec::CodecLimits;

    async fn run_mock_oci_server(listener: tokio::net::TcpListener) {
        let master_kek = [0x42u8; 32];
        let raw = Zeroizing::new(vec![0x33u8; 32]);
        let wrapped = kmipkit_proxy::envelope::AutoEnvelope::wrap_key(&master_kek, &raw).unwrap();
        let wrapped_b64 = base64::engine::general_purpose::STANDARD.encode(wrapped.to_bytes());

        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(s) => s,
                Err(_) => break,
            };
            let wrapped_b64 = wrapped_b64.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };
                let req_str = String::from_utf8_lossy(&buf[..n]);
                let first_line = req_str.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() < 2 {
                    return;
                }
                let method = parts[0];
                let path = parts[1];

                let body = if path.contains("secretbundles") {
                    format!(r#"{{"secretId":"test-secret","secretBundleContent":{{"contentType":"BASE64","content":"{wrapped_b64}"}}}}"#)
                } else if path.contains("/actions/sign") {
                    r#"{"signature":"ZHVtbXlfc2lnbmF0dXJl"}"#.to_string()
                } else if path.contains("/actions/decrypt") {
                    r#"{"plainText":"dGVzdC1kYXRh"}"#.to_string()
                } else if path.contains("/actions/generateRandomBytes") {
                    r#"{"data":"MDEyMzQ1Njc4OWFiY2RlZg=="}"#.to_string()
                } else if path.contains("/keys/") && method == "GET" {
                    r#"{"id":"ocid1.key.oc1..test-key","displayName":"test-key","lifecycleState":"ENABLED","freeformTags":{"Env":"IntegrationTest"}}"#.to_string()
                } else if path.contains("/secrets/") && method == "GET" {
                    r#"{"id":"ocid1.vaultsecret.oc1..test-secret","secretName":"test-secret","lifecycleState":"ACTIVE","freeformTags":{"Env":"IntegrationTest"}}"#.to_string()
                } else if method == "POST" && path.contains("/keys") {
                    r#"{"id":"ocid1.key.oc1..test-key","displayName":"test-key","lifecycleState":"ENABLED","freeformTags":{"Env":"IntegrationTest"}}"#.to_string()
                } else if method == "POST" && path.contains("/secrets") {
                    r#"{"id":"ocid1.vaultsecret.oc1..test-secret","secretName":"test-secret","lifecycleState":"ACTIVE","freeformTags":{"Env":"IntegrationTest"}}"#.to_string()
                } else if method == "PUT" {
                    r#"{"id":"updated","freeformTags":{"updated":"true"}}"#.to_string()
                } else {
                    r#"{}"#.to_string()
                };

                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
            });
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
    async fn test_oci_provider_config_initialization() {
        let config = OciConfig {
            compartment_id: Some("ocid1.compartment.oc1..test".to_string()),
            vault_id: Some("ocid1.vault.oc1..test".to_string()),
            crypto_endpoint: Some("https://crypto.oci.test".to_string()),
            management_endpoint: Some("https://mgmt.oci.test".to_string()),
            secrets_endpoint: Some("https://secrets.oci.test".to_string()),
            endpoint_url: Some("http://127.0.0.1:8080".to_string()),
            secret_name_prefix: Some("kmipkit/".to_string()),
            envelope_master_kek_id: Some("test-kek".to_string()),
            auth_token: Some("dummy-token".to_string()),
        };

        let provider = OciVaultProvider::new(&config).expect("provider creates cleanly");
        assert_eq!(provider.provider_name(), "oci-vault");
        assert_eq!(
            provider.compartment_id.as_deref(),
            Some("ocid1.compartment.oc1..test")
        );
        assert_eq!(provider.vault_id.as_deref(), Some("ocid1.vault.oc1..test"));
        assert_eq!(
            provider.endpoint_url.as_deref(),
            Some("http://127.0.0.1:8080")
        );
    }

    #[test]
    fn test_oci_envelope_key_crypto_operations() {
        let master_kek = [0x77u8; 32];
        let raw_material = Zeroizing::new(vec![0x88u8; 32]);

        let wrapped = kmipkit_proxy::envelope::AutoEnvelope::wrap_key(&master_kek, &raw_material)
            .expect("wrap succeeds");
        let serialized = wrapped.to_bytes();
        let deserialized = kmipkit_proxy::envelope::WrappedEnvelopeKey::from_bytes(&serialized)
            .expect("deserialize succeeds");
        assert_eq!(wrapped, deserialized);

        let unwrapped =
            kmipkit_proxy::envelope::AutoEnvelope::unwrap_key(&master_kek, &deserialized)
                .expect("unwrap succeeds");
        assert_eq!(*raw_material, *unwrapped);
    }

    #[tokio::test]
    async fn test_oci_mock_lifecycle() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_oci_server(listener));

        let config = OciConfig {
            compartment_id: Some("ocid1.compartment.oc1..test".to_string()),
            vault_id: Some("ocid1.vault.oc1..test".to_string()),
            crypto_endpoint: None,
            management_endpoint: None,
            secrets_endpoint: None,
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            secret_name_prefix: Some("kmipkit/".to_string()),
            envelope_master_kek_id: Some("test-kek".to_string()),
            auth_token: Some("dummy-token".to_string()),
        };

        let master_kek = [0x42u8; 32];
        let provider = OciVaultProvider::from_config(&config, Some(master_kek))
            .expect("provider creates cleanly");

        // 1. Create native key
        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());
        let spec = CreateKeySpec {
            algorithm: "RSA".to_string(),
            length: 2048,
            usage_mask: 0x0c,
            name: Some("test-key".to_string()),
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
        update_tags.insert("updated".to_string(), "true".to_string());
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
    async fn test_oci_proxy_server_end_to_end() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_oci_server(listener));

        let config = OciConfig {
            compartment_id: Some("ocid1.compartment.oc1..test".to_string()),
            vault_id: Some("ocid1.vault.oc1..test".to_string()),
            crypto_endpoint: None,
            management_endpoint: None,
            secrets_endpoint: None,
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            secret_name_prefix: Some("kmipkit/".to_string()),
            envelope_master_kek_id: Some("test-kek".to_string()),
            auth_token: Some("dummy-token".to_string()),
        };

        let master_kek = [0x42u8; 32];
        let provider = Arc::new(
            OciVaultProvider::from_config(&config, Some(master_kek))
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
                .expect("discover_versions succeeds against OCI-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
