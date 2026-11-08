//! Integration tests for Open Telekom Cloud KMS, TMS, and CSMS provider bridge (KMIPKIT-0082).
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

#[cfg(feature = "otc")]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{OtcConfig, ProxyConfig};
    use kmipkit_proxy::providers::otc::OtcKmsProvider;
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
    struct MockState {
        keys: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    async fn run_mock_otc_server(listener: TcpListener) {
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
                let mut lines = req_str.lines();
                let request_line = lines.next().unwrap_or("");
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or("");
                let path = parts.next().unwrap_or("");

                let body = if let Some(pos) = req_str.find("\r\n\r\n") {
                    &req_str[pos + 4..]
                } else {
                    ""
                };

                let (status, resp_body) = handle_mock_otc_request(&state, method, path, body);

                let resp = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    resp_body.len(),
                    resp_body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                let _ = socket.flush().await;
            });
        }
    }

    fn handle_mock_otc_request(
        state: &MockState,
        method: &str,
        path: &str,
        body: &str,
    ) -> (&'static str, String) {
        let mut keys_guard = state.keys.lock().unwrap();

        // 1. KMS Create Key
        if method == "POST" && path.ends_with("/kms/create-key") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let key_id = format!("otc-key-{}", fastrand_id());
            let alias = parsed
                .get("key_alias")
                .and_then(|v| v.as_str())
                .unwrap_or("test-alias")
                .to_string();

            let val = serde_json::json!({
                "key_id": key_id,
                "key_alias": alias,
                "key_state": "2",
                "key_spec": "AES_256",
                "key_description": parsed.get("key_description").and_then(|v| v.as_str()).unwrap_or(""),
                "tags": []
            });
            keys_guard.insert(key_id.clone(), val);

            let resp = serde_json::json!({
                "key_info": {
                    "key_id": key_id,
                    "key_alias": alias,
                    "key_state": "2",
                    "key_spec": "AES_256"
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 2. KMS Describe Key
        if method == "POST" && path.ends_with("/kms/describe-key") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            if let Some(kid) = parsed.get("key_id").and_then(|v| v.as_str()) {
                if let Some(entry) = keys_guard.get(kid) {
                    let resp = serde_json::json!({
                        "key_info": entry
                    });
                    return ("200 OK", resp.to_string());
                }
            }
            return ("404 Not Found", r#"{"error_code":"KMS.0004"}"#.to_string());
        }

        // 3. KMS Encrypt Data
        if method == "POST" && path.ends_with("/kms/encrypt-data") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let kid = parsed.get("key_id").and_then(|v| v.as_str()).unwrap_or("");
            let pt = parsed
                .get("plain_text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let cipher = format!("ENC_{pt}");
            let resp = serde_json::json!({
                "key_id": kid,
                "cipher_text": BASE64.encode(cipher.as_bytes())
            });
            return ("200 OK", resp.to_string());
        }

        // 4. KMS Decrypt Data
        if method == "POST" && path.ends_with("/kms/decrypt-data") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let kid = parsed.get("key_id").and_then(|v| v.as_str()).unwrap_or("");
            let ct = parsed
                .get("cipher_text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let decoded = BASE64.decode(ct).unwrap_or_default();
            let decoded_str = String::from_utf8_lossy(&decoded);
            let pt = decoded_str.strip_prefix("ENC_").unwrap_or(&decoded_str);
            let resp = serde_json::json!({
                "key_id": kid,
                "plain_text": pt.to_string()
            });
            return ("200 OK", resp.to_string());
        }

        // 5. KMS Enable Key
        if method == "POST" && path.ends_with("/kms/enable-key") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            if let Some(kid) = parsed.get("key_id").and_then(|v| v.as_str()) {
                if let Some(entry) = keys_guard.get_mut(kid) {
                    entry["key_state"] = serde_json::json!("2");
                    let resp = serde_json::json!({
                        "key_info": { "key_id": kid, "key_state": "2" }
                    });
                    return ("200 OK", resp.to_string());
                }
            }
            return ("404 Not Found", "{}".to_string());
        }

        // 6. KMS Disable Key
        if method == "POST" && path.ends_with("/kms/disable-key") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            if let Some(kid) = parsed.get("key_id").and_then(|v| v.as_str()) {
                if let Some(entry) = keys_guard.get_mut(kid) {
                    entry["key_state"] = serde_json::json!("3");
                    let resp = serde_json::json!({
                        "key_info": { "key_id": kid, "key_state": "3" }
                    });
                    return ("200 OK", resp.to_string());
                }
            }
            return ("404 Not Found", "{}".to_string());
        }

        // 7. KMS Schedule Key Deletion
        if method == "POST" && path.ends_with("/kms/schedule-key-deletion") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            if let Some(kid) = parsed.get("key_id").and_then(|v| v.as_str()) {
                if let Some(entry) = keys_guard.get_mut(kid) {
                    entry["key_state"] = serde_json::json!("4");
                    let resp = serde_json::json!({
                        "key_id": kid,
                        "key_state": "4"
                    });
                    return ("200 OK", resp.to_string());
                }
            }
            return ("404 Not Found", "{}".to_string());
        }

        // 8. KMS Create Datakey
        if method == "POST" && path.ends_with("/kms/create-datakey") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let kid = parsed.get("key_id").and_then(|v| v.as_str()).unwrap_or("");
            let dek = vec![0x42u8; 32];
            let resp = serde_json::json!({
                "key_id": kid,
                "plain_text": BASE64.encode(&dek),
                "cipher_text": BASE64.encode(format!("CIPHER_{}", BASE64.encode(&dek)).as_bytes())
            });
            return ("200 OK", resp.to_string());
        }

        // 9. KMS Tagging Action: /v1.0/{project_id}/kms/{key_id}/tags/action
        if method == "POST" && path.contains("/kms/") && path.ends_with("/tags/action") {
            let parts: Vec<&str> = path.split('/').collect();
            // path: /v1.0/{project_id}/kms/{key_id}/tags/action
            if parts.len() >= 6 {
                let kid = parts[parts.len() - 3];
                let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
                let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("");
                let tags_arr = parsed.get("tags").and_then(|v| v.as_array());

                if let Some(entry) = keys_guard.get_mut(kid) {
                    if action == "create" {
                        if let Some(new_tags) = tags_arr {
                            let entry_tags = entry["tags"].as_array_mut();
                            if let Some(current) = entry_tags {
                                for nt in new_tags {
                                    current.push(nt.clone());
                                }
                            }
                        }
                    }
                    return ("200 OK", "{}".to_string());
                }
            }
            return ("200 OK", "{}".to_string());
        }

        // 10. KMS Query Tags: GET /v1.0/{project_id}/kms/{key_id}/tags
        if method == "GET" && path.contains("/kms/") && path.ends_with("/tags") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 5 {
                let kid = parts[parts.len() - 2];
                if let Some(entry) = keys_guard.get(kid) {
                    let resp = serde_json::json!({
                        "tags": entry.get("tags").cloned().unwrap_or(serde_json::json!([]))
                    });
                    return ("200 OK", resp.to_string());
                }
            }
            return ("200 OK", r#"{"tags":[]}"#.to_string());
        }

        // 11. Resource instances action: POST /v1.0/{project_id}/kms/resource_instances/action
        if method == "POST" && path.ends_with("/kms/resource_instances/action") {
            let resources: Vec<serde_json::Value> = keys_guard
                .keys()
                .map(|k| serde_json::json!({ "resource_id": k }))
                .collect();
            let resp = serde_json::json!({ "resources": resources });
            return ("200 OK", resp.to_string());
        }

        // 12. CSMS Create Secret: POST /v1/{project_id}/secrets
        if method == "POST" && path.ends_with("/secrets") {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let sec_id = format!("otc-sec-{}", fastrand_id());
            let name = parsed.get("name").and_then(|v| v.as_str()).unwrap_or("sec");
            let sec_bin = parsed
                .get("secret_binary")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let val = serde_json::json!({
                "id": sec_id,
                "name": name,
                "secret_binary": sec_bin,
                "tags": []
            });
            keys_guard.insert(sec_id.clone(), val);

            let resp = serde_json::json!({
                "secret": {
                    "id": sec_id,
                    "name": name,
                    "state": "ENABLED"
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 13. CSMS Create Secret Version: POST /v1/{project_id}/secrets/{item_id}/versions
        if method == "POST" && path.contains("/secrets/") && path.ends_with("/versions") {
            let parts: Vec<&str> = path.split('/').collect();
            let sid = parts[parts.len() - 2];
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            if let Some(entry) = keys_guard.get_mut(sid) {
                if let Some(data) = parsed.get("secret_binary").and_then(|v| v.as_str()) {
                    entry["secret_binary"] = serde_json::json!(data);
                }
            }
            let resp = serde_json::json!({
                "version": {
                    "version_id": "1",
                    "secret_id": sid
                }
            });
            return ("200 OK", resp.to_string());
        }

        // 14. CSMS Get Secret Version: GET /v1/{project_id}/secrets/{item_id}/versions/latest
        if method == "GET" && path.contains("/secrets/") && path.ends_with("/versions/latest") {
            let parts: Vec<&str> = path.split('/').collect();
            let sid = parts[parts.len() - 3];
            if let Some(entry) = keys_guard.get(sid) {
                let bin = entry
                    .get("secret_binary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let resp = serde_json::json!({
                    "version": {
                        "version_id": "1",
                        "secret_binary": bin
                    }
                });
                return ("200 OK", resp.to_string());
            }
            return ("404 Not Found", "{}".to_string());
        }

        // 15. CSMS Delete Secret: DELETE /v1/{project_id}/secrets/{item_id}
        if method == "DELETE" && path.contains("/secrets/") {
            let parts: Vec<&str> = path.split('/').collect();
            let sid = parts.last().unwrap_or(&"");
            keys_guard.remove(*sid);
            return ("200 OK", "{}".to_string());
        }

        ("404 Not Found", "{}".to_string())
    }

    fn fastrand_id() -> u32 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
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
    async fn test_otc_provider_lifecycle() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(run_mock_otc_server(listener));

        let config = OtcConfig {
            region: Some("eu-de".to_string()),
            project_id: Some("0123456789abcdef0123456789abcdef".to_string()),
            endpoint_url: Some(format!("http://{addr}")),
            envelope_master_kek_id: None,
            item_prefix: Some("kmipkit-".to_string()),
            auth_token: Some("mock-token-xyz".to_string()),
            access_key_id: None,
            access_key_cred: None,
        };

        let master_kek = [0x5du8; 32];
        let provider = OtcKmsProvider::from_config(&config, Some(master_kek))
            .expect("failed to create OtcKmsProvider");
        assert_eq!(provider.provider_name(), "otc");

        // 1. Create native key
        let spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-otc-key".to_string()),
        };
        let mut tags = CloudTagMap::new();
        tags.insert("environment", "test");
        tags.insert("owner", "alice");

        let key_id = provider
            .create_native_key(&spec, &tags)
            .await
            .expect("create_native_key failed");
        assert!(key_id.starts_with("otc-key-"));

        // 2. Encrypt & Decrypt
        let pt = b"Hello Open Telekom Cloud KMS!";
        let crypto_params = CryptoParams {
            algorithm: "AES".to_string(),
            padding: None,
            hashing_algorithm: None,
        };
        let ct = provider
            .encrypt(&key_id, &crypto_params, pt)
            .await
            .expect("encrypt failed");
        assert_ne!(&ct[..], pt);

        let decrypted = provider
            .decrypt(&key_id, &crypto_params, &ct)
            .await
            .expect("decrypt failed");
        assert_eq!(&decrypted[..], pt);

        // 3. Metadata & tags
        let meta = provider
            .get_tags_and_metadata(&key_id)
            .await
            .expect("get_tags_and_metadata failed");
        assert!(meta.native_state_enabled);

        // 4. Update tags
        let mut upsert = CloudTagMap::new();
        upsert.insert("environment", "production");
        provider
            .update_tags(&key_id, &upsert, &[])
            .await
            .expect("update_tags failed");

        // 5. Disable and Enable
        provider
            .set_native_enabled_state(&key_id, false)
            .await
            .expect("disable failed");
        provider
            .set_native_enabled_state(&key_id, true)
            .await
            .expect("enable failed");

        // 6. Envelope Key via CSMS
        let raw_key = zeroize::Zeroizing::new(vec![0x77u8; 32]);
        let sec_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("secret-key".to_string()),
        };
        let env_id = provider
            .create_envelope_key(&sec_spec, Some(raw_key.clone()), &tags)
            .await
            .expect("create_envelope_key failed");
        assert!(env_id.starts_with("otc-sec-"));

        // 7. Get key material
        let (payload, _) = provider
            .get_key_material(&env_id)
            .await
            .expect("get_key_material failed");
        match payload {
            kmipkit_proxy::providers::KeyMaterialPayload::Symmetric(mat) => {
                assert_eq!(&mat[..], &raw_key[..]);
            }
            _ => panic!("expected symmetric key"),
        }

        // 8. Schedule destruction
        provider
            .schedule_destruction(&key_id)
            .await
            .expect("schedule_destruction failed");
        provider
            .schedule_destruction(&env_id)
            .await
            .expect("delete secret failed");
    }

    #[tokio::test]
    async fn test_otc_proxy_server_end_to_end() {
        let mock_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mock_addr = mock_listener.local_addr().unwrap();
        tokio::spawn(run_mock_otc_server(mock_listener));

        let config = OtcConfig {
            region: Some("eu-de".to_string()),
            project_id: Some("0123456789abcdef0123456789abcdef".to_string()),
            endpoint_url: Some(format!("http://{mock_addr}")),
            envelope_master_kek_id: None,
            item_prefix: Some("kmip-".to_string()),
            auth_token: Some("mock-token-123".to_string()),
            access_key_id: None,
            access_key_cred: None,
        };

        let master_kek = [0x5du8; 32];
        let provider = Arc::new(
            OtcKmsProvider::from_config(&config, Some(master_kek))
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
                .expect("discover_versions succeeds against OTC-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
