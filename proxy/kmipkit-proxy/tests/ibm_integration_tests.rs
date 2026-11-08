//! Integration tests for IBM Cloud Key Protect CloudKmsProvider.
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

#[cfg(feature = "ibm")]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use base64::Engine;
    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{IbmConfig, ProxyConfig};
    use kmipkit_proxy::providers::ibm::IbmKmsProvider;
    use kmipkit_proxy::providers::{CloudKmsProvider, CloudTagMap, CreateKeySpec, CryptoParams};
    use kmipkit_proxy::server::ProxyServer;
    use kmipkit_transport::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
        TransportConfig, TrustSource,
    };
    use kmipkit_ttlv::codec::CodecLimits;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Default, Clone)]
    struct MockIbmState {
        keys: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    async fn run_mock_ibm_server(listener: TcpListener) {
        let state = MockIbmState::default();

        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(_) => break,
            };
            let state = state.clone();

            tokio::spawn(async move {
                let mut buf = vec![0u8; 16384];
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };

                let req_str = String::from_utf8_lossy(&buf[..n]);
                let mut lines = req_str.lines();
                let req_line = lines.next().unwrap_or("");
                let parts: Vec<&str> = req_line.split_whitespace().collect();
                if parts.len() < 2 {
                    return;
                }
                let method = parts[0];
                let path = parts[1];

                let body = req_str.split("\r\n\r\n").nth(1).unwrap_or("");

                let (status, resp_body) = handle_mock_ibm_request(
                    method, path, body, &state,
                );

                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    resp_body.len(),
                    resp_body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    }

    fn handle_mock_ibm_request(
        method: &str,
        path: &str,
        body: &str,
        state: &MockIbmState,
    ) -> (&'static str, String) {
        let b64 = &base64::engine::general_purpose::STANDARD;

        // 1. IAM Token endpoint: POST /identity/token
        if method == "POST" && path == "/identity/token" {
            let resp = serde_json::json!({
                "access_token": "mock-ibm-bearer-token-xyz",
                "refresh_token": "mock-refresh",
                "token_type": "Bearer",
                "expires_in": 3600,
                "expiration": 1800000000
            });
            return ("200 OK", resp.to_string());
        }

        // 2. Create Key: POST /api/v2/keys
        if method == "POST" && path == "/api/v2/keys" {
            let mut parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));

            let mut resources = parsed
                .get_mut("resources")
                .and_then(|r| r.as_array_mut())
                .cloned()
                .unwrap_or_default();

            if let Some(res) = resources.get_mut(0) {
                let key_id = res
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ibm-key-1")
                    .to_string();

                res["id"] = serde_json::json!(key_id.clone());
                res["state"] = serde_json::json!(1); // Active
                res["crn"] = serde_json::json!(format!(
                    "crn:v1:bluemix:public:kms:us-south:a/test-acc:test-inst:key:{}",
                    key_id
                ));
                if res.get("payload").is_none() {
                    res["payload"] = serde_json::json!(b64.encode(vec![0x42; 32]));
                }
                state
                    .keys
                    .lock()
                    .unwrap()
                    .insert(key_id.clone(), res.clone());
            }

            let resp = serde_json::json!({
                "metadata": {
                    "collectionType": "application/vnd.ibm.kms.key+json",
                    "collectionTotal": 1
                },
                "resources": resources
            });
            return ("201 Created", resp.to_string());
        }

        // 3. Get Key: GET /api/v2/keys/{id}
        if method == "GET" && path.starts_with("/api/v2/keys/") && !path.contains("/actions/") {
            let key_id = path.trim_start_matches("/api/v2/keys/");
            if let Some(k) = state.keys.lock().unwrap().get(key_id) {
                let resp = serde_json::json!({
                    "metadata": {
                        "collectionType": "application/vnd.ibm.kms.key+json",
                        "collectionTotal": 1
                    },
                    "resources": [k]
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"errors": [{"message": "key not found"}]}).to_string(),
            );
        }

        // 4. Wrap Key Action: POST /api/v2/keys/{id}/actions/wrap
        if method == "POST" && path.starts_with("/api/v2/keys/") && path.ends_with("/actions/wrap")
        {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let pt = parsed.get("plaintext").and_then(|v| v.as_str()).unwrap_or("");
            let ct = format!("ibm:wrapped:{}", b64.encode(format!("encrypted:{}", pt)));
            let resp = serde_json::json!({
                "ciphertext": ct
            });
            return ("200 OK", resp.to_string());
        }

        // 5. Unwrap Key Action: POST /api/v2/keys/{id}/actions/unwrap
        if method == "POST" && path.starts_with("/api/v2/keys/") && path.ends_with("/actions/unwrap")
        {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let ct = parsed.get("ciphertext").and_then(|v| v.as_str()).unwrap_or("");
            let stripped = ct.trim_start_matches("ibm:wrapped:");
            let decoded = b64.decode(stripped).unwrap_or_default();
            let dec_str = String::from_utf8_lossy(&decoded);
            let pt_val = dec_str.trim_start_matches("encrypted:");
            let resp = serde_json::json!({
                "plaintext": pt_val
            });
            return ("200 OK", resp.to_string());
        }

        // 6. List keys: GET /api/v2/keys
        if method == "GET" && path.starts_with("/api/v2/keys") {
            let keys: Vec<serde_json::Value> =
                state.keys.lock().unwrap().values().cloned().collect();
            let count = keys.len();
            let resp = serde_json::json!({
                "metadata": {
                    "collectionType": "application/vnd.ibm.kms.key+json",
                    "collectionTotal": count
                },
                "resources": keys
            });
            return ("200 OK", resp.to_string());
        }

        // 7. Delete key: DELETE /api/v2/keys/{id}
        if method == "DELETE" && path.starts_with("/api/v2/keys/") {
            let key_id = path.trim_start_matches("/api/v2/keys/");
            state.keys.lock().unwrap().remove(key_id);
            return (
                "204 No Content",
                String::new(),
            );
        }

        (
            "404 Not Found",
            serde_json::json!({"error": "endpoint not mapped"}).to_string(),
        )
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
    async fn test_ibm_provider_direct_operations() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_ibm_server(listener));

        let config = IbmConfig {
            region: Some("us-south".to_string()),
            instance_id: Some("mock-inst-123".to_string()),
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            tagging_endpoint_url: None,
            search_endpoint_url: None,
            iam_endpoint_url: Some(format!("http://127.0.0.1:{port}/identity/token")),
            iam_api_cred: Some("mock-api-cred-secret".to_string()),
            default_extractable_for_get: Some(true),
            envelope_master_kek_id: None,
            auth_token: None,
        };

        let master_kek = [0x5au8; 32];
        let provider = Arc::new(
            IbmKmsProvider::from_config(&config, Some(master_kek))
                .expect("provider creates cleanly"),
        );
        assert_eq!(provider.provider_name(), "ibm");

        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());
        tags.insert("kmip_state".to_string(), "active".to_string());

        // 1. Create native key
        let spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-ibm-native-key".to_string()),
        };
        let key_id = provider.create_native_key(&spec, &tags).await.unwrap();
        assert!(!key_id.is_empty());

        // 2. Create envelope key
        let env_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-ibm-secret".to_string()),
        };
        let secret_id = provider
            .create_envelope_key(&env_spec, None, &tags)
            .await
            .unwrap();
        assert!(!secret_id.is_empty());

        // 3. Get key material
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
            algorithm: "AES".to_string(),
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
        assert_eq!(plaintext.as_slice(), b"test-data");

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
    async fn test_ibm_proxy_server_end_to_end() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_ibm_server(listener));

        let config = IbmConfig {
            region: Some("us-south".to_string()),
            instance_id: Some("mock-inst-123".to_string()),
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            tagging_endpoint_url: None,
            search_endpoint_url: None,
            iam_endpoint_url: Some(format!("http://127.0.0.1:{port}/identity/token")),
            iam_api_cred: Some("mock-api-cred-secret".to_string()),
            default_extractable_for_get: Some(true),
            envelope_master_kek_id: None,
            auth_token: None,
        };

        let master_kek = [0x42u8; 32];
        let provider = Arc::new(
            IbmKmsProvider::from_config(&config, Some(master_kek))
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
                .expect("discover_versions succeeds against IBM-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
