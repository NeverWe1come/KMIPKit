use std::io::Cursor;

use kmip_lsp::lsp::server::run_stdio_server;
use kmip_lsp::security::redaction::REDACTED_DISPLAY;
use kmip_lsp::spec::catalog;
use kmip_lsp::ttlv::inspector::{
    DecodeFormat, DecodeOptions, EncodeNodeInput, decode_ttlv, encode_ttlv,
};
use serde_json::json;

#[test]
fn encode_and_decode_roundtrip_preserves_offsets_and_enum_names() {
    let input = EncodeNodeInput {
        tag_hex: "0x420078".to_string(),
        item_type: "Structure".to_string(),
        value: None,
        children: vec![
            EncodeNodeInput {
                tag_hex: "0x42005C".to_string(),
                item_type: "Enumeration".to_string(),
                value: Some(json!(0x0000_0001)), // Create
                children: vec![],
            },
            EncodeNodeInput {
                tag_hex: "0x420028".to_string(),
                item_type: "Enumeration".to_string(),
                value: Some(json!(0x0000_0001)), // DES (weak)
                children: vec![],
            },
        ],
    };

    let encoded = encode_ttlv(&input).expect("encode_ttlv succeeds");
    assert_eq!(encoded.byte_length, 8 + 16 + 16);

    let decoded = decode_ttlv(&encoded.raw_hex, &DecodeOptions::default())
        .expect("decode_ttlv succeeds");
    assert_eq!(decoded.root.tag_hex, "0x420078");
    assert_eq!(decoded.root.tag_name, "Request Message");
    assert_eq!(decoded.root.children.len(), 2);
    assert_eq!(decoded.root.children[0].offset, 8);
    assert_eq!(decoded.root.children[0].enum_name.as_deref(), Some("Create"));
    assert_eq!(decoded.root.children[1].offset, 24);
    assert_eq!(decoded.root.children[1].enum_name.as_deref(), Some("DES"));
    assert!(
        decoded.alerts.iter().any(|a| a.code == "WEAK_CIPHER"),
        "expected WEAK_CIPHER alert for DES"
    );
}

#[test]
fn sensitive_key_material_is_redacted_by_default_and_revealable_by_offset() {
    let input = EncodeNodeInput {
        tag_hex: "0x420042".to_string(), // KeyBlock
        item_type: "Structure".to_string(),
        value: None,
        children: vec![EncodeNodeInput {
            tag_hex: "0x420043".to_string(), // KeyMaterial
            item_type: "ByteString".to_string(),
            value: Some(json!("0102030405060708")),
            children: vec![],
        }],
    };

    let encoded = encode_ttlv(&input).expect("encode_ttlv succeeds");

    // Default: redacted
    let redacted = decode_ttlv(&encoded.raw_hex, &DecodeOptions::default())
        .expect("decode redacted succeeds");
    let km_node = &redacted.root.children[0];
    assert!(km_node.sensitive);
    assert!(km_node.redacted);
    assert!(km_node.display_value.contains(REDACTED_DISPLAY));
    assert!(!redacted.raw_hex.contains("0102030405060708"));

    // Explicit reveal for that node's offset
    let revealed = decode_ttlv(
        &encoded.raw_hex,
        &DecodeOptions {
            format: DecodeFormat::Hex,
            reveal_all: false,
            reveal_offsets: vec![km_node.offset],
        },
    )
    .expect("decode revealed succeeds");
    assert!(!revealed.root.children[0].redacted);
    assert!(revealed.raw_hex.contains("0102030405060708"));
}

#[test]
fn catalog_lookup_resolves_hex_and_names() {
    let by_hex = catalog::lookup("0x420028");
    assert_eq!(by_hex.len(), 1);
    assert_eq!(by_hex[0].name, "CryptographicAlgorithm");

    let by_name = catalog::lookup("KemAlgorithm");
    assert_eq!(by_name.len(), 1);
    assert_eq!(by_name[0].versions, "3.0+");
}

fn frame(msg: &serde_json::Value) -> String {
    let s = serde_json::to_string(msg).unwrap();
    format!("Content-Length: {}\r\n\r\n{}", s.len(), s)
}

#[test]
fn lsp_stdio_session_handles_standard_and_custom_kmip_requests() {
    let mut stream = String::new();
    stream.push_str(&frame(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    })));
    stream.push_str(&frame(&json!({
        "jsonrpc": "2.0",
        "method": "textDocument/didOpen",
        "params": {
            "textDocument": {
                "uri": "file:///test.kmip.json",
                "text": "{\n  \"CryptographicAlgorithm\": \"3DES\"\n}"
            }
        }
    })));
    stream.push_str(&frame(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "textDocument/hover",
        "params": {
            "textDocument": { "uri": "file:///test.kmip.json" },
            "position": { "line": 1, "character": 5 }
        }
    })));
    stream.push_str(&frame(&json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "kmip/lookupSpec",
        "params": { "query": "0x420078" }
    })));
    stream.push_str(&frame(&json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "shutdown"
    })));
    stream.push_str(&frame(&json!({
        "jsonrpc": "2.0",
        "method": "exit"
    })));

    let mut out = Vec::new();
    run_stdio_server(Cursor::new(stream.into_bytes()), &mut out)
        .expect("stdio server completes cleanly");

    let output_str = String::from_utf8(out).expect("valid utf8 output");
    assert!(output_str.contains("\"name\":\"kmip-lsp\""));
    assert!(output_str.contains("KMIP_WEAK_ALGORITHM"));
    assert!(output_str.contains("0x420028"));
    assert!(output_str.contains("RequestMessage"));
}

#[test]
fn prod_and_readonly_guardrails_block_destructive_operations() {
    use kmip_lsp::client::live::{Environment, LiveProfile, check_guardrails};

    let prod = LiveProfile {
        name: "prod-hsm".to_string(),
        environment: Environment::Prod,
        host: "kms.prod.local".to_string(),
        port: 5696,
        read_only: false,
        tls: None,
    };

    assert!(check_guardrails(&prod, "Locate", false).is_ok());
    assert!(
        check_guardrails(&prod, "Destroy", false)
            .unwrap_err()
            .contains("GUARDRAIL_PROD_LOCKED")
    );
    assert!(check_guardrails(&prod, "Destroy", true).is_ok());

    let ro = LiveProfile {
        read_only: true,
        ..prod
    };
    assert!(
        check_guardrails(&ro, "Create", true)
            .unwrap_err()
            .contains("GUARDRAIL_READ_ONLY")
    );
}

#[test]
fn sdk_analyzer_detects_deprecated_tags_and_generates_anonymized_tests() {
    use kmip_lsp::sdk::analyzer::{TargetLanguage, analyze_source, generate_test_from_ttlv};

    let rust_diags = analyze_source(
        "file:///app/src/main.rs",
        "let tag = \"TemplateAttribute\"; // 3DES",
    );
    assert!(rust_diags.iter().any(|d| d.code == "KMIP_DEPRECATED_1X_TAG"));
    assert!(rust_diags.iter().any(|d| d.code == "KMIP_WEAK_ALGORITHM"));

    let input = EncodeNodeInput {
        tag_hex: "0x420042".to_string(),
        item_type: "Structure".to_string(),
        value: None,
        children: vec![EncodeNodeInput {
            tag_hex: "0x420043".to_string(),
            item_type: "ByteString".to_string(),
            value: Some(json!("deadbeefcafebabe")),
            children: vec![],
        }],
    };
    let encoded = encode_ttlv(&input).expect("encode succeeds");

    let generated = generate_test_from_ttlv(&encoded.raw_hex, TargetLanguage::Rust)
        .expect("generate_test succeeds");
    assert!(!generated.code.contains("deadbeefcafebabe"));
    assert!(generated.code.contains("generated_kmip_ttlv_fixture_test"));
}
