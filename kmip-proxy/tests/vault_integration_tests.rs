//! Integration tests for HashiCorp Vault / OpenBao CloudKmsProvider.
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

#[cfg(feature = "vault")]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use base64::Engine;
    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{ProxyConfig, VaultConfig};
    use kmipkit_proxy::providers::vault::VaultKmsProvider;
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
    struct MockVaultState {
        transit_keys: Arc<Mutex<HashMap<String, serde_json::Value>>>,
        kv_metadata: Arc<Mutex<HashMap<String, serde_json::Value>>>,
        kv_data: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    async fn run_mock_vault_server(listener: TcpListener) {
        let state = MockVaultState::default();

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

                let (status, resp_body) = handle_mock_vault(method, path, body_str, &state);
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp_body}",
                    resp_body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    }

    fn handle_mock_vault(
        method: &str,
        path: &str,
        body: &str,
        state: &MockVaultState,
    ) -> (&'static str, String) {
        let b64 = &base64::engine::general_purpose::STANDARD;

        // 1. Transit create key: POST /v1/transit/keys/{name}
        if method == "POST"
            && path.starts_with("/v1/transit/keys/")
            && !path.ends_with("/config")
            && !path.ends_with("/rotate")
        {
            let key_name = path.trim_start_matches("/v1/transit/keys/").to_string();
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            state.transit_keys.lock().unwrap().insert(key_name, parsed);
            return ("200 OK", serde_json::json!({"data": {}}).to_string());
        }

        // 2. Transit get key: GET /v1/transit/keys/{name}
        if method == "GET" && path.starts_with("/v1/transit/keys/") && !path.contains("/export/") {
            let key_name = path.trim_start_matches("/v1/transit/keys/");
            if let Some(_k) = state.transit_keys.lock().unwrap().get(key_name) {
                let resp = serde_json::json!({
                    "data": {
                        "name": key_name,
                        "type": "rsa-2048",
                        "deletion_allowed": true,
                        "min_decryption_version": 1,
                        "min_encryption_version": 0,
                        "keys": {
                            "1": {
                                "public_key": "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0mockpublickey123456\n-----END PUBLIC KEY-----\n"
                            }
                        }
                    }
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"errors": ["key not found"]}).to_string(),
            );
        }

        // 3. Transit export key: GET /v1/transit/export/{type}/{name}
        if method == "GET" && path.starts_with("/v1/transit/export/") {
            let parts: Vec<&str> = path.split('/').collect();
            let key_name = parts.last().unwrap_or(&"");
            let raw_key = vec![0x42u8; 32];
            let raw_b64 = b64.encode(&raw_key);
            let resp = serde_json::json!({
                "data": {
                    "name": key_name,
                    "keys": {
                        "1": raw_b64
                    }
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 4. Transit config key: POST /v1/transit/keys/{name}/config
        if method == "POST" && path.starts_with("/v1/transit/keys/") && path.ends_with("/config") {
            return ("200 OK", serde_json::json!({"data": {}}).to_string());
        }

        // 5. Transit delete key: DELETE /v1/transit/keys/{name}
        if method == "DELETE" && path.starts_with("/v1/transit/keys/") {
            let key_name = path.trim_start_matches("/v1/transit/keys/");
            state.transit_keys.lock().unwrap().remove(key_name);
            return ("204 No Content", String::new());
        }

        // 6. Transit encrypt: POST /v1/transit/encrypt/{name}
        if method == "POST" && path.starts_with("/v1/transit/encrypt/") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let pt_b64 = parsed
                .get("plaintext")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let ct_val = format!("vault:v1:{}", b64.encode(format!("encrypted:{}", pt_b64)));
            let resp = serde_json::json!({
                "data": {
                    "ciphertext": ct_val
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 7. Transit decrypt: POST /v1/transit/decrypt/{name}
        if method == "POST" && path.starts_with("/v1/transit/decrypt/") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let ct_val = parsed
                .get("ciphertext")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let stripped = ct_val.trim_start_matches("vault:v1:");
            let decoded = b64.decode(stripped).unwrap_or_default();
            let dec_str = String::from_utf8_lossy(&decoded);
            let original_pt = dec_str.trim_start_matches("encrypted:");
            let resp = serde_json::json!({
                "data": {
                    "plaintext": original_pt
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 8. Transit sign: POST /v1/transit/sign/{name}
        if method == "POST" && path.starts_with("/v1/transit/sign/") {
            let resp = serde_json::json!({
                "data": {
                    "signature": format!("vault:v1:{}", b64.encode(b"valid-mock-signature-12345678"))
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 9. Transit verify: POST /v1/transit/verify/{name}
        if method == "POST" && path.starts_with("/v1/transit/verify/") {
            let resp = serde_json::json!({
                "data": {
                    "valid": true
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 10. Transit random: POST /v1/transit/random/{bytes}
        if method == "POST" && path.starts_with("/v1/transit/random/") {
            let bytes_str = path.trim_start_matches("/v1/transit/random/");
            let count: usize = bytes_str.parse().unwrap_or(16);
            let rand_data = vec![0x77u8; count];
            let resp = serde_json::json!({
                "data": {
                    "random_bytes": b64.encode(&rand_data)
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 11. KV v2 metadata: POST/PATCH /v1/secret/metadata/*
        if (method == "POST" || method == "PATCH") && path.starts_with("/v1/secret/metadata/") {
            let key = path.trim_start_matches("/v1/secret/metadata/").to_string();
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            state.kv_metadata.lock().unwrap().insert(key, parsed);
            return ("200 OK", serde_json::json!({"data": {}}).to_string());
        }

        // 12. KV v2 metadata get: GET /v1/secret/metadata/*
        if method == "GET" && path.starts_with("/v1/secret/metadata/") {
            let key = path.trim_start_matches("/v1/secret/metadata/");
            if let Some(meta) = state.kv_metadata.lock().unwrap().get(key) {
                let custom_metadata = meta
                    .get("custom_metadata")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                let resp = serde_json::json!({
                    "data": {
                        "created_time": "2026-11-08T10:00:00Z",
                        "custom_metadata": custom_metadata
                    }
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"errors": ["metadata not found"]}).to_string(),
            );
        }

        // 13. KV v2 metadata list: LIST or GET with ?list=true
        if method == "LIST" || (method == "GET" && path == "/v1/secret/metadata") {
            let keys: Vec<String> = state.kv_metadata.lock().unwrap().keys().cloned().collect();
            let resp = serde_json::json!({
                "data": {
                    "keys": keys
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 14. KV v2 metadata delete: DELETE /v1/secret/metadata/*
        if method == "DELETE" && path.starts_with("/v1/secret/metadata/") {
            let key = path.trim_start_matches("/v1/secret/metadata/");
            state.kv_metadata.lock().unwrap().remove(key);
            return ("204 No Content", String::new());
        }

        // 15. KV v2 data: POST /v1/secret/data/*
        if method == "POST" && path.starts_with("/v1/secret/data/") {
            let key = path.trim_start_matches("/v1/secret/data/").to_string();
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            state.kv_data.lock().unwrap().insert(key, parsed);
            return ("200 OK", serde_json::json!({"data": {}}).to_string());
        }

        // 16. KV v2 data get: GET /v1/secret/data/*
        if method == "GET" && path.starts_with("/v1/secret/data/") {
            let key = path.trim_start_matches("/v1/secret/data/");
            if let Some(val) = state.kv_data.lock().unwrap().get(key) {
                let resp = serde_json::json!({
                    "data": val
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"errors": ["data not found"]}).to_string(),
            );
        }

        // 17. KV v2 data delete: DELETE /v1/secret/data/*
        if method == "DELETE" && path.starts_with("/v1/secret/data/") {
            let key = path.trim_start_matches("/v1/secret/data/");
            state.kv_data.lock().unwrap().remove(key);
            return ("204 No Content", String::new());
        }

        (
            "404 Not Found",
            serde_json::json!({"errors": ["unhandled mock path"]}).to_string(),
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
    async fn test_vault_provider_direct_operations() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_vault_server(listener));

        let config = VaultConfig {
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            namespace: None,
            transit_mount: Some("transit".to_string()),
            kv_v2_mount: Some("secret".to_string()),
            item_prefix: Some("kmipkit/".to_string()),
            allow_transit_export: Some(true),
            auth_method: Some("token".to_string()),
            role_id: None,
            role_cred: None,
            auth_token: Some("root-token".to_string()),
            envelope_master_kek_id: Some("master-kek".to_string()),
        };

        let master_kek = [0x5au8; 32];
        let provider = Arc::new(
            VaultKmsProvider::from_config(&config, Some(master_kek))
                .expect("provider creates cleanly"),
        );

        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());
        tags.insert("kmip_state".to_string(), "active".to_string());

        // 1. Create native key
        let spec = CreateKeySpec {
            algorithm: "RSA".to_string(),
            length: 2048,
            usage_mask: 0x0c,
            name: Some("test-vault-key".to_string()),
        };
        let key_id = provider.create_native_key(&spec, &tags).await.unwrap();
        assert!(!key_id.is_empty());

        // 2. Create envelope key
        let env_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-vault-secret".to_string()),
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
    async fn test_vault_proxy_server_end_to_end() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_vault_server(listener));

        let config = VaultConfig {
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            namespace: None,
            transit_mount: Some("transit".to_string()),
            kv_v2_mount: Some("secret".to_string()),
            item_prefix: Some("kmipkit/".to_string()),
            allow_transit_export: Some(true),
            auth_method: Some("token".to_string()),
            role_id: None,
            role_cred: None,
            auth_token: Some("root-token".to_string()),
            envelope_master_kek_id: Some("master-kek".to_string()),
        };

        let master_kek = [0x42u8; 32];
        let provider = Arc::new(
            VaultKmsProvider::from_config(&config, Some(master_kek))
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
                .expect("discover_versions succeeds against Vault-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
