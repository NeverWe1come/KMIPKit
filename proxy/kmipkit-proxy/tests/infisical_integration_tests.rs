//! Integration tests for Infisical KMS CloudKmsProvider.
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

#[cfg(feature = "infisical")]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use base64::Engine;
    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{InfisicalConfig, ProxyConfig};
    use kmipkit_proxy::providers::infisical::InfisicalKmsProvider;
    use kmipkit_proxy::providers::{
        CloudKmsProvider, CloudTagMap, CreateKeySpec, CryptoParams,
    };
    use kmipkit_proxy::server::ProxyServer;
    use kmipkit_transport::{
        CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
        TransportConfig, TrustSource,
    };
    use kmipkit_ttlv::codec::CodecLimits;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[derive(Default, Clone)]
    struct MockInfisicalState {
        keys: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    async fn run_mock_infisical_server(listener: TcpListener) {
        let state = MockInfisicalState::default();

        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(conn) => conn,
                Err(_) => break,
            };

            let state = state.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };

                let req_str = String::from_utf8_lossy(&buf[..n]);
                let mut lines = req_str.lines();
                let request_line = lines.next().unwrap_or("");
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or("GET");
                let path_and_query = parts.next().unwrap_or("/");
                let path = path_and_query.split('?').next().unwrap_or("/");

                let body_str = if let Some(pos) = req_str.find("\r\n\r\n") {
                    &req_str[pos + 4..]
                } else {
                    ""
                };

                let (status, resp_body) = handle_mock_infisical(method, path, body_str, &state);
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp_body}",
                    resp_body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    }

    fn handle_mock_infisical(
        method: &str,
        path: &str,
        body: &str,
        state: &MockInfisicalState,
    ) -> (&'static str, String) {
        let b64 = &base64::engine::general_purpose::STANDARD;

        // 1. Universal Auth login
        if method == "POST" && path == "/api/v1/auth/universal-auth/login" {
            let resp = serde_json::json!({
                "accessToken": "mock-infisical-access-token-123456",
                "expiresIn": 7200,
                "tokenType": "Bearer"
            });
            return ("200 OK", resp.to_string());
        }

        // 2. Create key: POST /api/v1/kms/keys
        if method == "POST" && path == "/api/v1/kms/keys" {
            let mut parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let key_id = parsed
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("infisical-key-1")
                .to_string();

            parsed["id"] = serde_json::json!(key_id);
            parsed["isDisabled"] = serde_json::json!(false);
            parsed["createdAt"] = serde_json::json!("2026-11-08T10:00:00Z");
            parsed["version"] = serde_json::json!(1);

            state.keys.lock().unwrap().insert(key_id.clone(), parsed.clone());
            let resp = serde_json::json!({
                "key": parsed
            });
            return ("200 OK", resp.to_string());
        }

        // 3. Export private/symmetric key: GET /api/v1/kms/keys/{keyId}/private-key
        if method == "GET" && path.starts_with("/api/v1/kms/keys/") && path.ends_with("/private-key") {
            let key_id = path
                .trim_start_matches("/api/v1/kms/keys/")
                .trim_end_matches("/private-key");

            if let Some(_k) = state.keys.lock().unwrap().get(key_id) {
                let raw_key = vec![0x42u8; 32];
                let resp = serde_json::json!({
                    "keyId": key_id,
                    "privateKey": b64.encode(&raw_key)
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }

        // 4. Export public key: GET /api/v1/kms/keys/{keyId}/public-key
        if method == "GET" && path.starts_with("/api/v1/kms/keys/") && path.ends_with("/public-key") {
            let key_id = path
                .trim_start_matches("/api/v1/kms/keys/")
                .trim_end_matches("/public-key");

            if let Some(_k) = state.keys.lock().unwrap().get(key_id) {
                let resp = serde_json::json!({
                    "keyId": key_id,
                    "publicKey": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0mockpublickey123456\n-----END PUBLIC KEY-----\n"
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }

        // 5. Encrypt: POST /api/v1/kms/keys/{keyId}/encrypt
        if method == "POST" && path.starts_with("/api/v1/kms/keys/") && path.ends_with("/encrypt") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let pt_b64 = parsed.get("plaintext").and_then(|v| v.as_str()).unwrap_or("");
            let ct_val = format!("infisical:v1:{}", b64.encode(format!("encrypted:{}", pt_b64)));
            let resp = serde_json::json!({
                "ciphertext": ct_val
            });
            return ("200 OK", resp.to_string());
        }

        // 6. Decrypt: POST /api/v1/kms/keys/{keyId}/decrypt
        if method == "POST" && path.starts_with("/api/v1/kms/keys/") && path.ends_with("/decrypt") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let ct_val = parsed.get("ciphertext").and_then(|v| v.as_str()).unwrap_or("");
            let stripped = ct_val.trim_start_matches("infisical:v1:");
            let decoded = b64.decode(stripped).unwrap_or_default();
            let dec_str = String::from_utf8_lossy(&decoded);
            let original_pt = dec_str.trim_start_matches("encrypted:");
            let resp = serde_json::json!({
                "plaintext": original_pt
            });
            return ("200 OK", resp.to_string());
        }

        // 7. Sign: POST /api/v1/kms/keys/{keyId}/sign
        if method == "POST" && path.starts_with("/api/v1/kms/keys/") && path.ends_with("/sign") {
            let resp = serde_json::json!({
                "signature": b64.encode(b"valid-mock-signature-12345678")
            });
            return ("200 OK", resp.to_string());
        }

        // 8. Verify: POST /api/v1/kms/keys/{keyId}/verify
        if method == "POST" && path.starts_with("/api/v1/kms/keys/") && path.ends_with("/verify") {
            let resp = serde_json::json!({
                "isValid": true
            });
            return ("200 OK", resp.to_string());
        }

        // 9. Rotate: POST /api/v1/kms/keys/{keyId}/rotate
        if method == "POST" && path.starts_with("/api/v1/kms/keys/") && path.ends_with("/rotate") {
            return ("200 OK", serde_json::json!({"version": 2}).to_string());
        }

        // 10. Get key: GET /api/v1/kms/keys/{keyId}
        if method == "GET" && path.starts_with("/api/v1/kms/keys/") && path != "/api/v1/kms/keys" {
            let key_id = path.trim_start_matches("/api/v1/kms/keys/");
            if let Some(k) = state.keys.lock().unwrap().get(key_id) {
                let resp = serde_json::json!({
                    "key": k
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }

        // 11. Patch key: PATCH /api/v1/kms/keys/{keyId}
        if method == "PATCH" && path.starts_with("/api/v1/kms/keys/") {
            let key_id = path.trim_start_matches("/api/v1/kms/keys/");
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));

            let mut guard = state.keys.lock().unwrap();
            if let Some(k) = guard.get_mut(key_id) {
                if let Some(desc) = parsed.get("description") {
                    k["description"] = desc.clone();
                }
                if let Some(disabled) = parsed.get("isDisabled") {
                    k["isDisabled"] = disabled.clone();
                }
                let resp = serde_json::json!({
                    "key": k.clone()
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }

        // 12. Delete key: DELETE /api/v1/kms/keys/{keyId}
        if method == "DELETE" && path.starts_with("/api/v1/kms/keys/") {
            let key_id = path.trim_start_matches("/api/v1/kms/keys/");
            state.keys.lock().unwrap().remove(key_id);
            return ("204 No Content", String::new());
        }

        // 13. List keys: GET /api/v1/kms/keys
        if method == "GET" && path == "/api/v1/kms/keys" {
            let keys: Vec<serde_json::Value> =
                state.keys.lock().unwrap().values().cloned().collect();
            let resp = serde_json::json!({
                "keys": keys
            });
            return ("200 OK", resp.to_string());
        }

        (
            "404 Not Found",
            serde_json::json!({"message": "unhandled mock path"}).to_string(),
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
    async fn test_infisical_provider_direct_operations() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_infisical_server(listener));

        let config = InfisicalConfig {
            site_url: Some(format!("http://127.0.0.1:{port}")),
            project_id: Some("mock-proj-123".to_string()),
            client_id: Some("mock-client-id".to_string()),
            client_cred: Some("mock-client-secret".to_string()),
            auth_token: None,
            item_prefix: Some("kmipkit/".to_string()),
        };

        let master_kek = [0x5au8; 32];
        let provider = Arc::new(
            InfisicalKmsProvider::from_config(&config, Some(master_kek))
                .expect("provider creates cleanly"),
        );

        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());
        tags.insert("kmip_state".to_string(), "active".to_string());

        // 1. Create native key
        let spec = CreateKeySpec {
            algorithm: "RSA".to_string(),
            length: 4096,
            usage_mask: 0x0c,
            name: Some("test-infisical-key".to_string()),
        };
        let key_id = provider.create_native_key(&spec, &tags).await.unwrap();
        assert!(!key_id.is_empty());

        // 2. Create envelope key
        let env_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-infisical-secret".to_string()),
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
    async fn test_infisical_proxy_server_end_to_end() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_infisical_server(listener));

        let config = InfisicalConfig {
            site_url: Some(format!("http://127.0.0.1:{port}")),
            project_id: Some("mock-proj-123".to_string()),
            client_id: Some("mock-client-id".to_string()),
            client_cred: Some("mock-client-secret".to_string()),
            auth_token: None,
            item_prefix: Some("kmipkit/".to_string()),
        };

        let master_kek = [0x42u8; 32];
        let provider = Arc::new(
            InfisicalKmsProvider::from_config(&config, Some(master_kek))
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
                .expect("discover_versions succeeds against Infisical-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
