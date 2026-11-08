//! Integration tests for Scaleway Key Manager and Secret Manager provider bridge (KMIPKIT-0081).
#![forbid(unsafe_code)]
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

#[cfg(feature = "scaleway")]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{ProxyConfig, ScalewayConfig};
    use kmipkit_proxy::providers::scaleway::ScalewayKmsProvider;
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
    struct MockState {
        keys: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    async fn run_mock_scaleway_server(listener: TcpListener) {
        let state = MockState::default();

        loop {
            let (mut socket, _) = match listener.accept().await {
                Ok(s) => s,
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
                let first_line = req_str.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() < 2 {
                    return;
                }
                let method = parts[0];
                let path = parts[1];

                let body = if let Some(idx) = req_str.find("\r\n\r\n") {
                    &req_str[idx + 4..]
                } else {
                    ""
                };

                let (status, resp_json) = handle_mock_scaleway(method, path, body, &state);
                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    resp_json.len(),
                    resp_json
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    }

    fn handle_mock_scaleway(
        method: &str,
        path: &str,
        body: &str,
        state: &MockState,
    ) -> (&'static str, String) {
        // 1. Create Key
        if method == "POST" && path.ends_with("/keys") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("scw-key");
            let key_id = format!("key-{}", name);
            let tags = parsed.get("tags").cloned().unwrap_or(serde_json::json!([]));
            let record = serde_json::json!({
                "id": key_id,
                "name": name,
                "state": "enabled",
                "tags": tags,
                "created_at": "2026-11-08T12:00:00Z"
            });
            state.keys.lock().unwrap().insert(key_id.clone(), record.clone());
            return ("200 OK", record.to_string());
        }

        // 2. Encrypt
        if method == "POST" && path.ends_with("/encrypt") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let pt = parsed.get("plaintext").and_then(|v| v.as_str()).unwrap_or("");
            let ct = format!("ct_{}", pt);
            return (
                "200 OK",
                serde_json::json!({
                    "ciphertext": BASE64.encode(ct.as_bytes())
                })
                .to_string(),
            );
        }

        // 3. Decrypt
        if method == "POST" && path.ends_with("/decrypt") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let ct_b64 = parsed.get("ciphertext").and_then(|v| v.as_str()).unwrap_or("");
            let ct_bytes = BASE64.decode(ct_b64).unwrap_or_default();
            let ct_str = String::from_utf8_lossy(&ct_bytes);
            let pt = ct_str.strip_prefix("ct_").unwrap_or(&ct_str);
            return (
                "200 OK",
                serde_json::json!({
                    "plaintext": pt
                })
                .to_string(),
            );
        }

        // 4. Get Key Metadata
        if method == "GET" && path.contains("/keys/") {
            let key_id = path.split('/').last().unwrap_or("");
            if let Some(record) = state.keys.lock().unwrap().get(key_id) {
                return ("200 OK", record.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }

        // 5. Update Key Tags
        if method == "PATCH" && path.contains("/keys/") {
            let key_id = path.split('/').last().unwrap_or("");
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            if let Some(record) = state.keys.lock().unwrap().get_mut(key_id) {
                if let Some(tags) = parsed.get("tags") {
                    record["tags"] = tags.clone();
                }
                return ("200 OK", record.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }

        // 6. Enable / Disable
        if method == "POST" && path.ends_with("/enable") {
            let parts: Vec<&str> = path.split('/').collect();
            let key_id = parts.get(parts.len() - 2).unwrap_or(&"");
            if let Some(record) = state.keys.lock().unwrap().get_mut(*key_id) {
                record["state"] = serde_json::json!("enabled");
                return ("200 OK", record.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }
        if method == "POST" && path.ends_with("/disable") {
            let parts: Vec<&str> = path.split('/').collect();
            let key_id = parts.get(parts.len() - 2).unwrap_or(&"");
            if let Some(record) = state.keys.lock().unwrap().get_mut(*key_id) {
                record["state"] = serde_json::json!("disabled");
                return ("200 OK", record.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "key not found"}).to_string(),
            );
        }

        // 7. Delete Key / Delete Secret
        if method == "DELETE" {
            let id = path.split('/').last().unwrap_or("");
            state.keys.lock().unwrap().remove(id);
            return ("204 No Content", String::new());
        }

        // 8. List Keys
        if method == "GET" && path.contains("/keys") {
            let keys: Vec<serde_json::Value> = state
                .keys
                .lock()
                .unwrap()
                .values()
                .cloned()
                .collect();
            return (
                "200 OK",
                serde_json::json!({
                    "keys": keys,
                    "total_count": keys.len()
                })
                .to_string(),
            );
        }

        // 9. Secret Manager: Create Secret
        if method == "POST" && path.ends_with("/secrets") {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("scw-sec");
            let sec_id = format!("secret-{}", name);
            let tags = parsed.get("tags").cloned().unwrap_or(serde_json::json!([]));
            let record = serde_json::json!({
                "id": sec_id,
                "name": name,
                "tags": tags,
                "versions": []
            });
            state.keys.lock().unwrap().insert(sec_id.clone(), record.clone());
            return ("200 OK", record.to_string());
        }

        // 10. Secret Manager: Create Version
        if method == "POST" && path.contains("/versions") {
            let parts: Vec<&str> = path.split('/').collect();
            let sec_id = parts.get(parts.len() - 2).unwrap_or(&"");
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let data = parsed.get("data").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(record) = state.keys.lock().unwrap().get_mut(*sec_id) {
                if let Some(arr) = record["versions"].as_array_mut() {
                    arr.push(serde_json::json!(data));
                }
                return (
                    "200 OK",
                    serde_json::json!({"version": 1, "revision": 1}).to_string(),
                );
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "secret not found"}).to_string(),
            );
        }

        // 11. Secret Manager: Access Version
        if method == "GET" && path.contains("/versions/latest/access") {
            let parts: Vec<&str> = path.split('/').collect();
            let sec_id = parts.get(parts.len() - 4).unwrap_or(&"");
            if let Some(record) = state.keys.lock().unwrap().get(*sec_id) {
                let data = record["versions"]
                    .as_array()
                    .and_then(|arr| arr.last())
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                return (
                    "200 OK",
                    serde_json::json!({
                        "data": data
                    })
                    .to_string(),
                );
            }
            return (
                "404 Not Found",
                serde_json::json!({"message": "secret not found"}).to_string(),
            );
        }

        (
            "404 Not Found",
            serde_json::json!({"message": "endpoint not found"}).to_string(),
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
    async fn test_scaleway_provider_lifecycle() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_scaleway_server(listener));

        let config = ScalewayConfig {
            region: Some("fr-par".to_string()),
            project_id: Some("mock-scw-project".to_string()),
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            envelope_master_kek_id: None,
            item_prefix: Some("kmipkit/".to_string()),
            auth_token: Some("mock-scw-token".to_string()),
        };

        let master_kek = [0x5cu8; 32];
        let provider = Arc::new(
            ScalewayKmsProvider::from_config(&config, Some(master_kek))
                .expect("provider creates cleanly"),
        );
        assert_eq!(provider.provider_name(), "scaleway");

        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());
        tags.insert("kmip_state".to_string(), "active".to_string());

        // 1. Create native key
        let spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-scw-native-key".to_string()),
        };
        let key_id = provider.create_native_key(&spec, &tags).await.unwrap();
        assert!(!key_id.is_empty());

        // 2. Create envelope key
        let env_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-scw-secret".to_string()),
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
    async fn test_scaleway_proxy_server_end_to_end() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_scaleway_server(listener));

        let config = ScalewayConfig {
            region: Some("fr-par".to_string()),
            project_id: Some("mock-scw-project".to_string()),
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            envelope_master_kek_id: None,
            item_prefix: Some("kmipkit/".to_string()),
            auth_token: Some("mock-scw-token".to_string()),
        };

        let master_kek = [0x5cu8; 32];
        let provider = Arc::new(
            ScalewayKmsProvider::from_config(&config, Some(master_kek))
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
                .expect("discover_versions succeeds against Scaleway-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
