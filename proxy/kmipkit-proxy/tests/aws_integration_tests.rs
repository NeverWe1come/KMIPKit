//! Integration tests for KMIPKit Proxy with AWS KMS and AWS Secrets Manager.
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

#[cfg(feature = "aws")]
mod aws_tests {
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::net::TcpStream;
    use zeroize::Zeroizing;

    use kmipkit_proxy::config::{AwsConfig, ProxyConfig};
    use kmipkit_proxy::providers::aws::AwsKmsProvider;
    use kmipkit_proxy::providers::{CloudKmsProvider, CloudTagMap, CreateKeySpec, CryptoParams};
    use kmipkit_proxy::server::ProxyServer;

    async fn is_localstack_available(host_port: &str) -> bool {
        tokio::time::timeout(Duration::from_millis(500), TcpStream::connect(host_port))
            .await
            .map(|r| r.is_ok())
            .unwrap_or(false)
    }

    #[tokio::test]
    async fn test_aws_kms_provider_lifecycle() {
        let endpoint = std::env::var("AWS_ENDPOINT_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:4566".to_string());
        let host_port = endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        if !is_localstack_available(host_port).await {
            if std::env::var("CI").is_ok() && std::env::var("REQUIRE_AWS_LOCALSTACK").is_ok() {
                panic!("LocalStack is required in CI but not reachable at {endpoint}");
            }
            eprintln!("LocalStack not running at {endpoint}, skipping live AWS KMS tests");
            return;
        }

        let config = AwsConfig {
            region: Some("us-east-1".to_string()),
            endpoint_url: Some(endpoint.clone()),
            envelope_master_kek_id: None,
            secret_name_prefix: Some("kmipkit-test/".to_string()),
        };

        let master_kek = [0x42u8; 32];
        let provider = AwsKmsProvider::from_config(&config, Some(master_kek))
            .await
            .expect("AwsKmsProvider builds cleanly");

        assert_eq!(provider.provider_name(), "aws-kms");

        // 1. Test Native KMS key lifecycle (AES-256 Symmetric)
        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());

        let create_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c, // Encrypt | Decrypt
            name: Some("kmipkit-native-test-key".to_string()),
        };

        let key_id = provider
            .create_native_key(&create_spec, &tags)
            .await
            .expect("create_native_key succeeds");
        assert!(!key_id.is_empty());

        let meta = provider
            .get_tags_and_metadata(&key_id)
            .await
            .expect("get_tags_and_metadata succeeds");
        assert!(meta.native_state_enabled);
        assert_eq!(meta.tags.get("Env"), Some(&"IntegrationTest".to_string()));

        // Native Encrypt / Decrypt
        let plaintext = b"Hello AWS KMS via KMIPKit Proxy!";
        let params = CryptoParams::default();
        let ciphertext = provider
            .encrypt(&key_id, &params, plaintext)
            .await
            .expect("native encrypt succeeds");
        assert_ne!(ciphertext, plaintext);

        let decrypted = provider
            .decrypt(&key_id, &params, &ciphertext)
            .await
            .expect("native decrypt succeeds");
        assert_eq!(*decrypted, plaintext);

        // Clean up native key
        provider
            .schedule_destruction(&key_id)
            .await
            .expect("schedule_destruction succeeds");

        // 2. Test Auto-Envelope Secret Lifecycle (Secrets Manager + AutoEnvelope)
        let env_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("kmipkit-envelope-secret".to_string()),
        };

        let raw_dek = Zeroizing::new(vec![0x77u8; 32]);
        let secret_id = provider
            .create_envelope_key(&env_spec, Some(raw_dek.clone()), &tags)
            .await
            .expect("create_envelope_key succeeds");
        assert!(secret_id.starts_with("kmipkit-test/"));

        let (material, ret_tags) = provider
            .get_key_material(&secret_id)
            .await
            .expect("get_key_material succeeds for envelope secret");
        assert_eq!(ret_tags.get("Env"), Some(&"IntegrationTest".to_string()));

        match material {
            kmipkit_proxy::providers::KeyMaterialPayload::Symmetric(sym) => {
                assert_eq!(*sym, *raw_dek);
            }
            other => panic!("expected symmetric key material, got {other:?}"),
        }

        // Clean up secret
        provider
            .schedule_destruction(&secret_id)
            .await
            .expect("secret deletion succeeds");
    }

    #[tokio::test]
    async fn test_proxy_server_with_aws_provider_end_to_end() {
        let endpoint = std::env::var("AWS_ENDPOINT_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:4566".to_string());
        let host_port = endpoint
            .trim_start_matches("http://")
            .trim_start_matches("https://");

        if !is_localstack_available(host_port).await {
            if std::env::var("CI").is_ok() && std::env::var("REQUIRE_AWS_LOCALSTACK").is_ok() {
                panic!("LocalStack is required in CI but not reachable at {endpoint}");
            }
            eprintln!("LocalStack not running at {endpoint}, skipping proxy live test");
            return;
        }

        let aws_conf = AwsConfig {
            region: Some("us-east-1".to_string()),
            endpoint_url: Some(endpoint.clone()),
            envelope_master_kek_id: None,
            secret_name_prefix: Some("kmipkit-proxy-e2e/".to_string()),
        };

        let provider = Arc::new(
            AwsKmsProvider::from_config(&aws_conf, Some([0x33u8; 32]))
                .await
                .expect("provider builds"),
        );

        let mut config = ProxyConfig::default();
        config.listener.bind_addr = "127.0.0.1:0".parse().unwrap();

        let server = ProxyServer::start(config, provider)
            .await
            .expect("proxy server starts with AWS backend");

        let pki = server.pki().expect("pki was generated");
        let client_endpoint = kmipkit_transport::Endpoint::raw_tls("127.0.0.1", server.port());

        let certificates = vec![pki.client_cert_der.clone()];
        let private_key = pki.client_key_der.to_vec();

        let transport_config = kmipkit_transport::TransportConfig::builder(client_endpoint)
            .client_identity(kmipkit_transport::ClientIdentity::new(
                kmipkit_transport::CertificateInput::from_der(certificates),
                kmipkit_transport::PrivateKeyInput::from_der(private_key),
            ))
            .trust_source(kmipkit_transport::TrustSource::certificate_authorities(
                vec![kmipkit_transport::CertificateInput::from_der(vec![
                    pki.ca_cert_der.clone(),
                ])],
            ))
            .timeouts(kmipkit_transport::TimeoutPolicy::default().with_total(
                kmipkit_transport::TimeoutLimit::Bounded(Duration::from_secs(5)),
            ))
            .build()
            .expect("transport config builds");

        let registry = kmipkit_client::extension_registry::client_extension_registry(
            Vec::new(),
            kmipkit_protocol::extension::defaults(),
        )
        .expect("empty registry");
        let client_config = kmipkit_client::extension_registry::ClientConfiguration::new(registry);
        let mut client =
            kmipkit_client::Client::new(client_config, transport_config).expect("client builds");

        let outcome = tokio::task::spawn_blocking(move || {
            let batch = kmipkit_client::ClientBatch::new(kmipkit_client::ClientBatchItem::new(
                kmipkit_client::ClientRequest::discover_versions(),
            ));
            client
                .execute(batch, &kmipkit_ttlv::codec::CodecLimits::defaults())
                .expect("discover_versions succeeds")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);

        server.shutdown().await.expect("clean shutdown");
    }
}
