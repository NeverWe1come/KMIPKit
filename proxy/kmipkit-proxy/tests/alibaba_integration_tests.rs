//! Integration tests for Alibaba Cloud KMS CloudKmsProvider.
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

#[cfg(feature = "alibaba")]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use base64::Engine;
    use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
    use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest, WireEncoding};
    use kmipkit_proxy::AutoPki;
    use kmipkit_proxy::config::{AlibabaConfig, ProxyConfig};
    use kmipkit_proxy::providers::alibaba::AlibabaKmsProvider;
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
    struct MockAlibabaState {
        keys: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    }

    async fn run_mock_alibaba_server(listener: TcpListener) {
        let state = MockAlibabaState::default();

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

                let (status, resp_body) = handle_mock_alibaba(method, path, body, &state);

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

    fn handle_mock_alibaba(
        method: &str,
        path: &str,
        body: &str,
        state: &MockAlibabaState,
    ) -> (&'static str, String) {
        let b64 = &base64::engine::general_purpose::STANDARD;

        // 1. CreateKey / CreateSecret
        if method == "POST"
            && (path.starts_with("/?Action=CreateKey") || path.contains("Action=CreateKey"))
        {
            let mut parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let key_id = parsed
                .get("KeyId")
                .or_else(|| parsed.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("ali-key-1")
                .to_string();
            parsed["KeyId"] = serde_json::json!(key_id.clone());
            parsed["KeyState"] = serde_json::json!("Enabled");
            parsed["KeySpec"] = serde_json::json!("Aliyun_AES_256");

            state
                .keys
                .lock()
                .unwrap()
                .insert(key_id.clone(), parsed.clone());
            let resp = serde_json::json!({
                "KeyMetadata": parsed
            });
            return ("200 OK", resp.to_string());
        }

        if method == "POST"
            && (path.starts_with("/?Action=CreateSecret") || path.contains("Action=CreateSecret"))
        {
            let mut parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let sec_name = parsed
                .get("SecretName")
                .and_then(|v| v.as_str())
                .unwrap_or("ali-secret-1")
                .to_string();
            parsed["SecretName"] = serde_json::json!(sec_name.clone());

            state
                .keys
                .lock()
                .unwrap()
                .insert(sec_name.clone(), parsed.clone());
            let resp = serde_json::json!({
                "SecretName": sec_name,
                "Arn": format!("acs:kms:cn-hangzhou:123456:secret/{sec_name}")
            });
            return ("200 OK", resp.to_string());
        }

        // 2. DescribeKey / GetSecretValue
        if method == "POST" && (path.contains("Action=DescribeKey")) {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let key_id = parsed.get("KeyId").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(k) = state.keys.lock().unwrap().get(key_id) {
                let resp = serde_json::json!({
                    "KeyMetadata": k
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"Code": "Forbidden.KeyNotFound"}).to_string(),
            );
        }

        if method == "POST" && (path.contains("Action=GetSecretValue")) {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let sec_name = parsed
                .get("SecretName")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if let Some(s) = state.keys.lock().unwrap().get(sec_name) {
                let resp = serde_json::json!({
                    "SecretName": sec_name,
                    "SecretData": s.get("SecretData").unwrap_or(&serde_json::json!(""))
                });
                return ("200 OK", resp.to_string());
            }
            return (
                "404 Not Found",
                serde_json::json!({"Code": "Forbidden.SecretNotFound"}).to_string(),
            );
        }

        // 3. Encrypt / Decrypt
        if method == "POST" && (path.contains("Action=Encrypt")) {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let pt_b64 = parsed
                .get("Plaintext")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let ct = format!("ali:v1:{}", b64.encode(format!("encrypted:{}", pt_b64)));
            let resp = serde_json::json!({
                "CiphertextBlob": ct,
                "KeyId": parsed.get("KeyId").unwrap_or(&serde_json::json!("mock-key"))
            });
            return ("200 OK", resp.to_string());
        }

        if method == "POST" && (path.contains("Action=Decrypt")) {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let ct = parsed
                .get("CiphertextBlob")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let stripped = ct.trim_start_matches("ali:v1:");
            let decoded = b64.decode(stripped).unwrap_or_default();
            let dec_str = String::from_utf8_lossy(&decoded);
            let pt_val = dec_str.trim_start_matches("encrypted:");
            let resp = serde_json::json!({
                "Plaintext": pt_val,
                "KeyId": "mock-key"
            });
            return ("200 OK", resp.to_string());
        }

        // 4. AsymmetricSign / AsymmetricVerify
        if method == "POST" && (path.contains("Action=AsymmetricSign")) {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let digest = parsed.get("Digest").and_then(|v| v.as_str()).unwrap_or("");
            let sig = format!("ali:sig:{}", b64.encode(format!("signed:{}", digest)));
            let resp = serde_json::json!({
                "Value": sig
            });
            return ("200 OK", resp.to_string());
        }

        if method == "POST" && (path.contains("Action=AsymmetricVerify")) {
            let resp = serde_json::json!({
                "Value": true
            });
            return ("200 OK", resp.to_string());
        }

        // 5. UpdateKeyDescription / TagResources
        if method == "POST" && (path.contains("Action=UpdateKeyDescription")) {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let key_id = parsed.get("KeyId").and_then(|v| v.as_str()).unwrap_or("");
            let desc = parsed
                .get("Description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if let Some(k) = state.keys.lock().unwrap().get_mut(key_id) {
                k["Description"] = serde_json::json!(desc);
            }
            return ("200 OK", serde_json::json!({}).to_string());
        }

        // 6. DisableKey / EnableKey
        if method == "POST" && (path.contains("Action=DisableKey")) {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let key_id = parsed.get("KeyId").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(k) = state.keys.lock().unwrap().get_mut(key_id) {
                k["KeyState"] = serde_json::json!("Disabled");
            }
            return ("200 OK", serde_json::json!({}).to_string());
        }

        // 7. ScheduleKeyDeletion / DeleteSecret
        if method == "POST"
            && (path.contains("Action=ScheduleKeyDeletion") || path.contains("Action=DeleteSecret"))
        {
            let parsed: serde_json::Value =
                serde_json::from_str(body).unwrap_or(serde_json::json!({}));
            let key_id = parsed
                .get("KeyId")
                .or_else(|| parsed.get("SecretName"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            state.keys.lock().unwrap().remove(key_id);
            return ("200 OK", serde_json::json!({}).to_string());
        }

        // 8. ListKeys
        if method == "POST" && (path.contains("Action=ListKeys")) {
            let keys: Vec<serde_json::Value> = state
                .keys
                .lock()
                .unwrap()
                .keys()
                .map(|k| serde_json::json!({"KeyId": k, "KeyArn": format!("acs:kms:cn-hangzhou:123456:key/{k}")}))
                .collect();
            let resp = serde_json::json!({
                "Keys": { "Key": keys },
                "TotalCount": keys.len()
            });
            return ("200 OK", resp.to_string());
        }

        (
            "404 Not Found",
            serde_json::json!({"Code": "Forbidden.ActionNotFound"}).to_string(),
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
    async fn test_alibaba_provider_direct_operations() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_alibaba_server(listener));

        let config = AlibabaConfig {
            region_id: Some("cn-hangzhou".to_string()),
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            access_key_id: Some("mock-ali-access-key".to_string()),
            access_key_cred: Some("mock-ali-access-secret".to_string()),
            envelope_master_kek_id: None,
            item_prefix: Some("kmipkit/".to_string()),
            auth_token: None,
        };

        let master_kek = [0x5au8; 32];
        let provider = Arc::new(
            AlibabaKmsProvider::from_config(&config, Some(master_kek))
                .expect("provider creates cleanly"),
        );
        assert_eq!(provider.provider_name(), "alibaba");

        let mut tags = CloudTagMap::new();
        tags.insert("Env".to_string(), "IntegrationTest".to_string());
        tags.insert("kmip_state".to_string(), "active".to_string());

        // 1. Create native key
        let spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-ali-native-key".to_string()),
        };
        let key_id = provider.create_native_key(&spec, &tags).await.unwrap();
        assert!(!key_id.is_empty());

        // 2. Create envelope key
        let env_spec = CreateKeySpec {
            algorithm: "AES".to_string(),
            length: 256,
            usage_mask: 0x0c,
            name: Some("test-ali-secret".to_string()),
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
    async fn test_alibaba_proxy_server_end_to_end() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(run_mock_alibaba_server(listener));

        let config = AlibabaConfig {
            region_id: Some("cn-hangzhou".to_string()),
            endpoint_url: Some(format!("http://127.0.0.1:{port}")),
            access_key_id: Some("mock-ali-access-key".to_string()),
            access_key_cred: Some("mock-ali-access-secret".to_string()),
            envelope_master_kek_id: None,
            item_prefix: Some("kmipkit/".to_string()),
            auth_token: None,
        };

        let master_kek = [0x42u8; 32];
        let provider = Arc::new(
            AlibabaKmsProvider::from_config(&config, Some(master_kek))
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
                .expect("discover_versions succeeds against Alibaba-backed proxy")
        })
        .await
        .unwrap();

        assert_eq!(outcome.len(), 1);
        server.shutdown().await.expect("clean shutdown");
    }
}
