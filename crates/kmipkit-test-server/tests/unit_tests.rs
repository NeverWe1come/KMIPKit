//! Comprehensive unit tests for kmipkit-test-server modules.
#![allow(
    unknown_lints,
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
    clippy::bool_to_int_with_if,
    clippy::manual_let_else,
    clippy::single_match_else
)]

use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;

use kmipkit_test_server::AutoPki;
use kmipkit_test_server::codec::datetime::*;
use kmipkit_test_server::codec::json::{decode_json, encode_json};
use kmipkit_test_server::codec::ttlv::{decode_ttlv, encode_ttlv};
use kmipkit_test_server::codec::xml::{decode_xml, encode_xml};
use kmipkit_test_server::config::{ServerConfig, help_message, parse_cli_args};
use kmipkit_test_server::ops::crypto::*;
use kmipkit_test_server::ops::dispatcher::*;
pub use kmipkit_test_server::ops::dispatcher::{TAG_ATTRIBUTE_NAME, TAG_ATTRIBUTES};
use kmipkit_test_server::store::{JsonStore, LocateFilter, StoredAttribute, StoredObject};
use kmipkit_test_server::v1_compat::*;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};

fn ttlv_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag).unwrap().try_checked().unwrap();
    Item::new(tag, value).unwrap()
}

fn extract_child(item: &Item, raw_tag: u32) -> Option<Item> {
    let mut found = None;
    item.with_value(|v| {
        if let ValueView::Structure(s) = v {
            for child in s.children() {
                if child.tag().raw() == raw_tag {
                    found = child.try_clone().ok();
                    break;
                }
            }
        }
    });
    found
}

fn extract_payload_child(response: &Item, raw_tag: u32) -> Option<Item> {
    let batch_item = extract_child(response, TAG_BATCH_ITEM)?;
    let payload = extract_child(&batch_item, TAG_RESPONSE_PAYLOAD)?;
    extract_child(&payload, raw_tag)
}

fn extract_text_field(item: &Item, raw_tag: u32) -> Option<String> {
    let target = extract_payload_child(item, raw_tag)?;
    let mut text = None;
    target.with_value(|v| {
        if let ValueView::TextString(s) = v {
            text = Some(s.to_string());
        }
    });
    text
}

fn make_req(major: i32, minor: i32, batch_item: Structure) -> Item {
    let mut req_header = Structure::new();
    let mut req_version = Structure::new();
    let _ = req_version.try_push(ttlv_item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(major)));
    let _ = req_version.try_push(ttlv_item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(minor)));
    let _ = req_header.try_push(ttlv_item(
        TAG_PROTOCOL_VERSION,
        Value::structure(req_version),
    ));
    let _ = req_header.try_push(ttlv_item(TAG_BATCH_COUNT, Value::integer(1)));

    let mut req = Structure::new();
    let _ = req.try_push(ttlv_item(TAG_REQUEST_HEADER, Value::structure(req_header)));
    let _ = req.try_push(ttlv_item(TAG_BATCH_ITEM, Value::structure(batch_item)));
    ttlv_item(TAG_REQUEST_MESSAGE, Value::structure(req))
}

#[test]
fn auto_pki_generates_and_exports_cleanly() {
    let pki = AutoPki::generate().expect("auto pki generation succeeds");
    assert!(pki.ca_cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(pki.server_cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(pki.server_key_pem.contains("BEGIN PRIVATE KEY"));
    assert!(pki.client_cert_pem.contains("BEGIN CERTIFICATE"));
    assert!(pki.client_key_pem.contains("BEGIN PRIVATE KEY"));

    let temp_dir = std::env::temp_dir().join(format!("kmipkit-test-pki-{}", std::process::id()));
    pki.export_to_directory(&temp_dir).expect("pki exports");
    assert!(temp_dir.join("ca.crt").exists());
    assert!(temp_dir.join("client.crt").exists());
    assert!(temp_dir.join("client.key").exists());
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn json_store_persists_and_reloads_objects() {
    let temp_dir = std::env::temp_dir().join(format!("kmip-json-store-{}", std::process::id()));
    let store = JsonStore::new(&temp_dir, true).expect("store inits");

    let mut obj = StoredObject::new("id-1234", "SymmetricKey", "Active", 1_700_000_000);
    obj.cryptographic_algorithm = Some("AES".to_owned());
    obj.cryptographic_length = Some(256);
    obj.cryptographic_usage_mask = Some(12);
    obj.key_material_base64 = Some("AAECAwQFBgcICQoLDA0ODw==".to_owned());
    obj.attributes.insert(
        "test-attr".to_owned(),
        vec![StoredAttribute {
            name: "test-attr".to_owned(),
            index: 0,
            value: serde_json::Value::String("hello".to_owned()),
        }],
    );

    store.save(&obj).expect("save succeeds");

    let disk_file = temp_dir.join("objects/id-1234.json");
    assert!(disk_file.exists());
    let disk_content = fs::read_to_string(&disk_file).expect("read file");
    assert!(disk_content.contains("\"unique_identifier\": \"id-1234\""));
    assert!(disk_content.contains("\"cryptographic_algorithm\": \"AES\""));

    let fetched = store.get("id-1234").expect("found in store");
    assert_eq!(fetched.unique_identifier, "id-1234");
    assert_eq!(fetched.cryptographic_length, Some(256));
    assert_eq!(fetched.attributes.len(), 1);

    let results = store.locate(&LocateFilter {
        cryptographic_algorithm: Some("AES".to_owned()),
        ..Default::default()
    });
    assert_eq!(results, vec!["id-1234".to_owned()]);

    let reloaded = JsonStore::new(&temp_dir, false).expect("reloaded store inits");
    let fetched_reloaded = reloaded.get("id-1234").expect("found after reload");
    assert_eq!(fetched_reloaded.unique_identifier, "id-1234");

    assert!(store.delete("id-1234").expect("delete succeeds"));
    assert!(!disk_file.exists());
    assert!(store.get("id-1234").is_none());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn roundtrip_ttlv_all_types() {
    let tag = RawTag::new(0x0042_0078).unwrap().try_checked().unwrap();
    let int_tag = RawTag::new(0x0042_006A).unwrap().try_checked().unwrap();
    let str_tag = RawTag::new(0x0042_0093).unwrap().try_checked().unwrap();
    let enum_tag = RawTag::new(0x0042_005C).unwrap().try_checked().unwrap();
    let bytes_tag = RawTag::new(0x0042_0080).unwrap().try_checked().unwrap();
    let long_tag = RawTag::new(0x0042_0001).unwrap().try_checked().unwrap();
    let big_tag = RawTag::new(0x0042_0002).unwrap().try_checked().unwrap();
    let bool_tag = RawTag::new(0x0042_0003).unwrap().try_checked().unwrap();
    let dt_tag = RawTag::new(0x0042_0004).unwrap().try_checked().unwrap();
    let interval_tag = RawTag::new(0x0042_0005).unwrap().try_checked().unwrap();
    let dte_tag = RawTag::new(0x0042_0006).unwrap().try_checked().unwrap();

    let mut structure = Structure::new();
    structure
        .try_push(Item::new(int_tag, Value::integer(42)).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(str_tag, Value::text_string("test string".to_string())).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(enum_tag, Value::enumeration(0x0000_0001)).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(bytes_tag, Value::byte_string(vec![0xAA, 0xBB, 0xCC])).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(long_tag, Value::long_integer(123_456_789)).unwrap())
        .unwrap();
    structure
        .try_push(
            Item::new(
                big_tag,
                Value::big_integer(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02]),
            )
            .unwrap(),
        )
        .unwrap();
    structure
        .try_push(Item::new(bool_tag, Value::boolean(true)).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(dt_tag, Value::date_time(1_700_000_000)).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(interval_tag, Value::interval(3600)).unwrap())
        .unwrap();
    structure
        .try_push(Item::new(dte_tag, Value::date_time_extended(1_700_000_000_000)).unwrap())
        .unwrap();

    let root_item = Item::new(tag, Value::structure(structure)).unwrap();
    let encoded = encode_ttlv(&root_item);

    let decoded = decode_ttlv(&encoded).expect("TTLV decode succeeds");
    assert_eq!(decoded.tag().raw(), 0x0042_0078);
    let re_encoded = encode_ttlv(&decoded);
    assert_eq!(encoded, re_encoded);

    // Negative TTLV test
    assert!(decode_ttlv(b"short").is_err());
}

#[test]
fn json_codec_roundtrip() {
    let json_input = br#"{"tag":"ResponseMessage","value":[{"tag":"ResponseHeader","value":[{"tag":"ProtocolVersion","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":2},{"tag":"ProtocolVersionMinor","type":"Integer","value":1}]},{"tag":"TimeStamp","type":"DateTime","value":"1970-01-01T00:00:01Z"},{"tag":"BatchCount","type":"Integer","value":1}]},{"tag":"BatchItem","value":[{"tag":"Operation","type":"Enumeration","value":"DiscoverVersions"},{"tag":"ResultStatus","type":"Enumeration","value":"Success"},{"tag":"ResponsePayload","value":[]}]}]}"#;

    let item = decode_json(json_input).expect("JSON decode succeeds");
    assert_eq!(item.tag().raw(), 0x0042_007B);

    let encoded = encode_json(&item).expect("JSON encode succeeds");
    assert!(encoded.contains("\"tag\":\"ResponseMessage\""));
    assert!(encoded.contains("\"tag\":\"ResponseHeader\""));
    assert!(encoded.contains("\"Operation\""));

    // Negative decode
    assert!(decode_json(b"{invalid").is_err());
    assert!(decode_json(b"{\"unknown_tag\": 1}").is_err());
}

#[test]
fn xml_codec_roundtrip() {
    let xml_input = br#"<ResponseMessage><ResponseHeader><ProtocolVersion><ProtocolVersionMajor type="Integer" value="2"/><ProtocolVersionMinor type="Integer" value="1"/></ProtocolVersion><TimeStamp type="DateTime" value="1970-01-01T00:00:01Z"/><BatchCount type="Integer" value="1"/></ResponseHeader><BatchItem><Operation type="Enumeration" value="DiscoverVersions"/><ResultStatus type="Enumeration" value="Success"/><ResponsePayload/></BatchItem></ResponseMessage>"#;

    let item = decode_xml(xml_input).expect("XML decode succeeds");
    assert_eq!(item.tag().raw(), 0x0042_007B);

    let encoded = encode_xml(&item).expect("XML encode succeeds");
    assert!(encoded.contains("<ResponseMessage>"));
    assert!(encoded.contains("<Operation type=\"Enumeration\" value=\"DiscoverVersions\"/>"));
    assert!(encoded.contains("<ResponsePayload/>"));

    // Negative decode
    assert!(decode_xml(b"<UnclosedTag>").is_err());
}

#[test]
fn crypto_primitives_work() {
    let rng = generate_random_bytes(32).expect("rng works");
    assert_eq!(rng.len(), 32);

    let data = b"KMIPKit test server crypto";
    let h = hash(0x0000_0005, data).expect("hash works");
    assert_eq!(h.len(), 32);
    assert_eq!(hash(0x0000_0006, data).unwrap().len(), 48);
    assert_eq!(hash(0x0000_0007, data).unwrap().len(), 64);

    // Unsupported hash
    assert!(hash(0x9999, data).is_err());

    let key = generate_random_bytes(32).expect("key generation works");
    let tag = hmac(0x0000_0005, &key, data).expect("hmac works");
    assert!(hmac_verify_tag(0x0000_0005, &key, data, &tag));
    assert!(!hmac_verify_tag(0x0000_0005, &key, b"other data", &tag));
    assert!(!hmac_verify_tag(0x9999, &key, data, &tag));

    let tag384 = hmac(0x0000_0006, &key, data).unwrap();
    assert!(hmac_verify_tag(0x0000_0006, &key, data, &tag384));
    let tag512 = hmac(0x0000_0007, &key, data).unwrap();
    assert!(hmac_verify_tag(0x0000_0007, &key, data, &tag512));
    assert!(hmac(0x9999, &key, data).is_err());

    let (encrypted, iv) = encrypt_aes(&key, None, data).expect("encrypt works");
    let decrypted = decrypt_aes(&key, iv.as_deref(), &encrypted).expect("decrypt works");
    assert_eq!(decrypted, data);

    // AES-128 (16-byte key)
    let key16 = &key[..16];
    let (enc16, iv16) = encrypt_aes(key16, None, data).expect("encrypt aes-128");
    let dec16 = decrypt_aes(key16, iv16.as_deref(), &enc16).expect("decrypt aes-128");
    assert_eq!(dec16, data);

    let (enc_iv, _) = encrypt_aes(&key, Some(&[1u8; 12]), data).expect("encrypt with iv");
    let dec_iv = decrypt_aes(&key, Some(&[1u8; 12]), &enc_iv).expect("decrypt with iv");
    assert_eq!(dec_iv, data);

    // Decrypt errors
    assert!(decrypt_aes(&key, None, data).is_err());
    assert!(decrypt_aes(&key, Some(&[1u8; 12]), b"bad ciphertext").is_err());

    // Invalid AES key length
    assert!(encrypt_aes(&[0u8; 10], None, data).is_err());
    assert!(decrypt_aes(&[0u8; 10], None, data).is_err());
}

#[test]
fn config_and_cli_parsing() {
    let cfg = ServerConfig::new()
        .with_listen_addr("127.0.0.1:9090".parse().unwrap())
        .with_data_dir("/tmp/data")
        .with_export_pki("/tmp/pki")
        .with_require_client_cert(false)
        .with_clean(true);

    assert_eq!(
        cfg.listen_addr,
        "127.0.0.1:9090".parse::<SocketAddr>().unwrap()
    );
    assert_eq!(cfg.data_dir.to_str().unwrap(), "/tmp/data");
    assert_eq!(cfg.export_pki.unwrap().to_str().unwrap(), "/tmp/pki");
    assert!(!cfg.require_client_cert);
    assert!(cfg.clean);

    assert!(help_message().contains("USAGE:"));

    // Valid CLI
    let args = vec![
        "kmipkit-test-server".to_owned(),
        "--listen".to_owned(),
        "127.0.0.1:5696".to_owned(),
        "--data-dir".to_owned(),
        "/tmp/kmip".to_owned(),
        "--export-pki".to_owned(),
        "/tmp/pki".to_owned(),
        "--clean".to_owned(),
        "--no-client-auth".to_owned(),
        "--print-env".to_owned(),
        "--json".to_owned(),
    ];
    let opts = parse_cli_args(&args)
        .expect("parses")
        .expect("some options");
    assert_eq!(opts.config.listen_addr.port(), 5696);
    assert!(opts.print_env);
    assert!(opts.json_output);

    // Help flag
    assert!(
        parse_cli_args(&["kmip".into(), "-h".into()])
            .unwrap()
            .is_none()
    );
    assert!(
        parse_cli_args(&["kmip".into(), "--help".into()])
            .unwrap()
            .is_none()
    );

    // Error cases
    assert!(parse_cli_args(&["kmip".into(), "--listen".into()]).is_err());
    assert!(parse_cli_args(&["kmip".into(), "--listen".into(), "bad".into()]).is_err());
    assert!(parse_cli_args(&["kmip".into(), "--data-dir".into()]).is_err());
    assert!(parse_cli_args(&["kmip".into(), "--export-pki".into()]).is_err());
    assert!(parse_cli_args(&["kmip".into(), "--unknown-flag".into()]).is_err());
}

#[test]
fn datetime_and_hex_utilities() {
    let days = days_from_civil(2026, 11, 7);
    let (y, m, d) = civil_from_days(days);
    assert_eq!((y, m, d), (2026, 11, 7));

    let fmt = format_datetime(0);
    assert_eq!(fmt, "1970-01-01T00:00:00Z");
    let parsed = parse_datetime("1970-01-01T00:00:00Z").expect("parse datetime");
    assert_eq!(parsed, 0);

    assert!(parse_datetime("invalid").is_err());
    assert!(parse_datetime("1970-01-01 00:00:00").is_err());

    let hex = octets_to_hex(b"\xde\xad\xbe\xef");
    assert_eq!(hex, "deadbeef");
    let octets = hex_to_octets("deadbeef").expect("hex to octets");
    assert_eq!(octets, b"\xde\xad\xbe\xef");
    let octets_prefixed = hex_to_octets("0xdeadbeef").expect("hex with 0x prefix");
    assert_eq!(octets_prefixed, b"\xde\xad\xbe\xef");

    assert!(hex_to_octets("deadbee").is_err());
    assert!(hex_to_octets("deadbezz").is_err());

    // Negative dates and parsing errors:
    let bce = format_datetime(-62_167_219_200);
    assert!(bce.starts_with('-'));
    let _ = parse_datetime("-0001-01-01T00:00:00Z");

    assert!(parse_datetime("2026-01-01").is_err());
    assert!(parse_datetime("2026T00:00:00").is_err());
    assert!(parse_datetime("bad-01-01T00:00:00Z").is_err());
    assert!(parse_datetime("2026-bad-01T00:00:00Z").is_err());
    assert!(parse_datetime("2026-01-badT00:00:00Z").is_err());
    assert!(parse_datetime("2026-01-01Tbad:00:00Z").is_err());
    assert!(parse_datetime("2026-01-01T00:bad:00Z").is_err());
    assert!(parse_datetime("2026-01-01T00:00:badZ").is_err());
}

#[test]
fn v1_compat_full_roundtrip() {
    assert!(is_version_supported(1, 0));
    assert!(is_version_supported(1, 1));
    assert!(is_version_supported(1, 2));
    assert!(is_version_supported(1, 3));
    assert!(is_version_supported(1, 4));
    assert!(is_version_supported(2, 0));
    assert!(is_version_supported(2, 1));
    assert!(is_version_supported(3, 0));
    assert!(!is_version_supported(4, 0));

    // Convert v2 attributes of each type into v1
    let int_item = ttlv_item(TAG_CRYPTOGRAPHIC_LENGTH, Value::integer(256));
    let long_item = ttlv_item(TAG_CRYPTOGRAPHIC_LENGTH, Value::long_integer(512));
    let big_item = ttlv_item(TAG_CRYPTOGRAPHIC_LENGTH, Value::big_integer(vec![1, 2, 3]));
    let enum_item = ttlv_item(TAG_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(2));
    let bool_item = ttlv_item(TAG_CRYPTOGRAPHIC_LENGTH, Value::boolean(true));
    let str_item = ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("key-name".into()));
    let bytes_item = ttlv_item(TAG_MAC_DATA, Value::byte_string(vec![0xAA, 0xBB]));
    let dt_item = ttlv_item(TAG_TIME_STAMP, Value::date_time(1_700_000_000));
    let interval_item = ttlv_item(TAG_TIME_STAMP, Value::interval(60));
    let dte_item = ttlv_item(TAG_TIME_STAMP, Value::date_time_extended(1_700_000_000_000));

    assert!(convert_v2_to_v1_attribute(&int_item).is_some());
    assert!(convert_v2_to_v1_attribute(&long_item).is_some());
    assert!(convert_v2_to_v1_attribute(&big_item).is_some());
    assert!(convert_v2_to_v1_attribute(&enum_item).is_some());
    assert!(convert_v2_to_v1_attribute(&bool_item).is_some());
    assert!(convert_v2_to_v1_attribute(&str_item).is_some());
    assert!(convert_v2_to_v1_attribute(&bytes_item).is_some());
    assert!(convert_v2_to_v1_attribute(&dt_item).is_some());
    assert!(convert_v2_to_v1_attribute(&interval_item).is_some());
    assert!(convert_v2_to_v1_attribute(&dte_item).is_some());

    // Normalize v1 TemplateAttribute containing Attributes
    let mut template = Structure::new();
    template
        .try_push(convert_v2_to_v1_attribute(&int_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&long_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&big_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&enum_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&bool_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&str_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&bytes_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&dt_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&interval_item).unwrap())
        .unwrap();
    template
        .try_push(convert_v2_to_v1_attribute(&dte_item).unwrap())
        .unwrap();

    let mut inner = Structure::new();
    inner.try_push(int_item.try_clone().unwrap()).unwrap();
    let struct_item = ttlv_item(TAG_KEY_BLOCK, Value::structure(inner));
    assert!(convert_v2_to_v1_attribute(&struct_item).is_some());
    template
        .try_push(convert_v2_to_v1_attribute(&struct_item).unwrap())
        .unwrap();

    let template_item = ttlv_item(TAG_ATTRIBUTES, Value::structure(template));
    let normalized = normalize_attributes_to_v2(&template_item).expect("normalized to v2");
    assert_eq!(normalized.tag().raw(), TAG_ATTRIBUTES);
}

#[test]
fn dispatcher_all_operations_and_error_handling() {
    let store = Arc::new(JsonStore::ephemeral());
    let dispatcher = Dispatcher::new(store);

    // 1. Ping
    let mut ping_item = Structure::new();
    let _ = ping_item.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(PING_OPERATION)));
    let _ = ping_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(Structure::new()),
    ));

    let ping_resp = dispatcher.dispatch(&make_req(2, 1, ping_item)).unwrap();
    assert_eq!(ping_resp.tag().raw(), TAG_RESPONSE_MESSAGE);

    // 2. Query
    let mut query_item = Structure::new();
    let _ = query_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(QUERY_OPERATION),
    ));
    let mut query_payload = Structure::new();
    let _ = query_payload.try_push(ttlv_item(0x0042_0070, Value::enumeration(1))); // QueryFunction
    let _ = query_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(query_payload),
    ));

    let query_resp = dispatcher.dispatch(&make_req(2, 1, query_item)).unwrap();
    assert_eq!(query_resp.tag().raw(), TAG_RESPONSE_MESSAGE);

    // 3. Create Key Pair
    let mut ckp_item = Structure::new();
    let _ = ckp_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(CREATE_KEY_PAIR_OPERATION),
    ));
    let mut ckp_payload = Structure::new();
    let _ = ckp_payload.try_push(ttlv_item(
        TAG_CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(3),
    )); // RSA
    let _ = ckp_payload.try_push(ttlv_item(TAG_CRYPTOGRAPHIC_LENGTH, Value::integer(2048)));
    let _ = ckp_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(ckp_payload),
    ));

    let ckp_resp = dispatcher.dispatch(&make_req(2, 1, ckp_item)).unwrap();
    let priv_id =
        extract_text_field(&ckp_resp, TAG_PRIVATE_KEY_UNIQUE_IDENTIFIER).expect("private key id");
    let pub_id =
        extract_text_field(&ckp_resp, TAG_PUBLIC_KEY_UNIQUE_IDENTIFIER).expect("public key id");
    assert_ne!(priv_id, "");
    assert_ne!(pub_id, "");

    // 4. Register
    let mut reg_item = Structure::new();
    let _ = reg_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(REGISTER_OPERATION),
    ));
    let mut reg_payload = Structure::new();
    let _ = reg_payload.try_push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(2))); // SymmetricKey
    let mut key_block = Structure::new();
    let _ = key_block.try_push(ttlv_item(TAG_KEY_FORMAT_TYPE, Value::enumeration(1))); // Raw
    let _ = key_block.try_push(ttlv_item(
        TAG_CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(2),
    )); // AES
    let mut key_value = Structure::new();
    let _ = key_value.try_push(ttlv_item(
        TAG_KEY_MATERIAL,
        Value::byte_string(vec![0x42; 32]),
    ));
    let _ = key_block.try_push(ttlv_item(TAG_KEY_VALUE, Value::structure(key_value)));
    let _ = reg_payload.try_push(ttlv_item(TAG_KEY_BLOCK, Value::structure(key_block)));
    let _ = reg_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(reg_payload),
    ));

    let reg_resp = dispatcher.dispatch(&make_req(2, 1, reg_item)).unwrap();
    let reg_id = extract_text_field(&reg_resp, TAG_UNIQUE_IDENTIFIER).expect("registered key id");

    // 5. Attributes: AddAttribute, GetAttributes, GetAttributeList, DeleteAttribute
    // AddAttribute
    let mut add_attr_item = Structure::new();
    let _ = add_attr_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(ADD_ATTRIBUTE_OPERATION),
    ));
    let mut add_attr_payload = Structure::new();
    let _ = add_attr_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let mut attr_to_add = Structure::new();
    let _ = attr_to_add.try_push(ttlv_item(
        TAG_ATTRIBUTE_NAME,
        Value::text_string("CustomName".into()),
    ));
    let _ = add_attr_payload.try_push(ttlv_item(TAG_ATTRIBUTES, Value::structure(attr_to_add)));
    let _ = add_attr_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(add_attr_payload),
    ));

    let add_attr_resp = dispatcher.dispatch(&make_req(2, 1, add_attr_item)).unwrap();
    assert_eq!(
        extract_text_field(&add_attr_resp, TAG_UNIQUE_IDENTIFIER).unwrap(),
        reg_id
    );

    // GetAttributes
    let mut get_attr_item = Structure::new();
    let _ = get_attr_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(GET_ATTRIBUTES_OPERATION),
    ));
    let mut get_attr_payload = Structure::new();
    let _ = get_attr_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let _ = get_attr_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(get_attr_payload),
    ));

    let get_attr_resp = dispatcher.dispatch(&make_req(2, 1, get_attr_item)).unwrap();
    assert!(extract_payload_child(&get_attr_resp, TAG_ATTRIBUTES).is_some());

    // GetAttributeList
    let mut get_list_item = Structure::new();
    let _ = get_list_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(GET_ATTRIBUTE_LIST_OPERATION),
    ));
    let mut get_list_payload = Structure::new();
    let _ = get_list_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let _ = get_list_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(get_list_payload),
    ));

    let get_list_resp = dispatcher.dispatch(&make_req(2, 1, get_list_item)).unwrap();
    assert!(extract_payload_child(&get_list_resp, TAG_ATTRIBUTE_NAME).is_some());

    // DeleteAttribute
    let mut del_attr_item = Structure::new();
    let _ = del_attr_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(DELETE_ATTRIBUTE_OPERATION),
    ));
    let mut del_attr_payload = Structure::new();
    let _ = del_attr_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let _ = del_attr_payload.try_push(ttlv_item(
        TAG_ATTRIBUTE_NAME,
        Value::text_string("CustomName".into()),
    ));
    let _ = del_attr_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(del_attr_payload),
    ));

    let del_attr_resp = dispatcher.dispatch(&make_req(2, 1, del_attr_item)).unwrap();
    assert_eq!(
        extract_text_field(&del_attr_resp, TAG_UNIQUE_IDENTIFIER).unwrap(),
        reg_id
    );

    // 6. MAC & MAC Verify
    let mut mac_item = Structure::new();
    let _ = mac_item.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(MAC_OPERATION)));
    let mut mac_payload = Structure::new();
    let _ = mac_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let _ = mac_payload.try_push(ttlv_item(
        TAG_CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(0x0000_0005),
    )); // HMAC-SHA256
    let _ = mac_payload.try_push(ttlv_item(
        TAG_MAC_DATA,
        Value::byte_string(b"Test message".to_vec()),
    ));
    let _ = mac_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(mac_payload),
    ));

    let mac_resp = dispatcher.dispatch(&make_req(2, 1, mac_item)).unwrap();
    let mac_data_child = extract_payload_child(&mac_resp, TAG_MAC_DATA).expect("mac data returned");
    let mut mac_tag_bytes = Vec::new();
    mac_data_child.with_value(|v| {
        if let ValueView::ByteString(b) = v {
            mac_tag_bytes = b.to_vec();
        }
    });
    assert_ne!(mac_tag_bytes.len(), 0);

    // MAC Verify
    let mut mac_verify_item = Structure::new();
    let _ = mac_verify_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(MAC_VERIFY_OPERATION),
    ));
    let mut mac_verify_payload = Structure::new();
    let _ = mac_verify_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let _ = mac_verify_payload.try_push(ttlv_item(
        TAG_CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(0x0000_0005),
    ));
    let _ = mac_verify_payload.try_push(ttlv_item(
        TAG_MAC_DATA,
        Value::byte_string(b"Test message".to_vec()),
    ));
    let _ = mac_verify_payload.try_push(ttlv_item(TAG_MAC_DATA, Value::byte_string(mac_tag_bytes)));
    let _ = mac_verify_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(mac_verify_payload),
    ));

    let mac_verify_resp = dispatcher
        .dispatch(&make_req(2, 1, mac_verify_item))
        .unwrap();
    assert_eq!(
        extract_text_field(&mac_verify_resp, TAG_UNIQUE_IDENTIFIER).unwrap(),
        reg_id
    );

    // 7. Sign & Signature Verify
    let mut sign_item = Structure::new();
    let _ = sign_item.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(SIGN_OPERATION)));
    let mut sign_payload = Structure::new();
    let _ = sign_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(priv_id.clone()),
    ));
    let _ = sign_payload.try_push(ttlv_item(
        TAG_CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(0x0000_0005),
    ));
    let _ = sign_payload.try_push(ttlv_item(
        TAG_DATA,
        Value::byte_string(b"Test sign message".to_vec()),
    ));
    let _ = sign_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(sign_payload),
    ));

    let sign_resp = dispatcher.dispatch(&make_req(2, 1, sign_item)).unwrap();
    let sig_child =
        extract_payload_child(&sign_resp, TAG_SIGNATURE_DATA).expect("signature returned");
    let mut sig_bytes = Vec::new();
    sig_child.with_value(|v| {
        if let ValueView::ByteString(b) = v {
            sig_bytes = b.to_vec();
        }
    });

    // Signature Verify
    let mut sig_ver_item = Structure::new();
    let _ = sig_ver_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(SIGNATURE_VERIFY_OPERATION),
    ));
    let mut sig_ver_payload = Structure::new();
    let _ = sig_ver_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(pub_id.clone()),
    ));
    let _ = sig_ver_payload.try_push(ttlv_item(
        TAG_CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(0x0000_0005),
    ));
    let _ = sig_ver_payload.try_push(ttlv_item(
        TAG_DATA,
        Value::byte_string(b"Test sign message".to_vec()),
    ));
    let _ = sig_ver_payload.try_push(ttlv_item(TAG_SIGNATURE_DATA, Value::byte_string(sig_bytes)));
    let _ = sig_ver_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(sig_ver_payload),
    ));

    let sig_ver_resp = dispatcher.dispatch(&make_req(2, 1, sig_ver_item)).unwrap();
    assert_eq!(
        extract_text_field(&sig_ver_resp, TAG_UNIQUE_IDENTIFIER).unwrap(),
        pub_id
    );

    // 8. Revoke & Destroy
    let mut rev_item = Structure::new();
    let _ = rev_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(REVOKE_OPERATION),
    ));
    let mut rev_payload = Structure::new();
    let _ = rev_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let _ = rev_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(rev_payload),
    ));

    let rev_resp = dispatcher.dispatch(&make_req(2, 1, rev_item)).unwrap();
    assert_eq!(
        extract_text_field(&rev_resp, TAG_UNIQUE_IDENTIFIER).unwrap(),
        reg_id
    );

    // Destroy
    let mut des_item = Structure::new();
    let _ = des_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(DESTROY_OPERATION),
    ));
    let mut des_payload = Structure::new();
    let _ = des_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(reg_id.clone()),
    ));
    let _ = des_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(des_payload),
    ));

    let des_resp = dispatcher.dispatch(&make_req(2, 1, des_item)).unwrap();
    assert_eq!(
        extract_text_field(&des_resp, TAG_UNIQUE_IDENTIFIER).unwrap(),
        reg_id
    );

    // Create (Symmetric Key)
    let mut create_item = Structure::new();
    let _ = create_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(CREATE_OPERATION),
    ));
    let mut create_payload = Structure::new();
    let _ = create_payload.try_push(ttlv_item(TAG_OBJECT_TYPE, Value::enumeration(2)));
    let mut create_attr = Structure::new();
    let _ = create_attr.try_push(ttlv_item(
        TAG_CRYPTOGRAPHIC_ALGORITHM,
        Value::enumeration(2),
    ));
    let _ = create_attr.try_push(ttlv_item(TAG_CRYPTOGRAPHIC_LENGTH, Value::integer(256)));
    let _ = create_payload.try_push(ttlv_item(TAG_ATTRIBUTES, Value::structure(create_attr)));
    let _ = create_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(create_payload),
    ));

    let create_resp = dispatcher.dispatch(&make_req(2, 1, create_item)).unwrap();
    let sym_id = extract_text_field(&create_resp, TAG_UNIQUE_IDENTIFIER).expect("created sym id");

    // Activate
    let mut act_item = Structure::new();
    let _ = act_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(ACTIVATE_OPERATION),
    ));
    let mut act_payload = Structure::new();
    let _ = act_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(sym_id.clone()),
    ));
    let _ = act_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(act_payload),
    ));

    let act_resp = dispatcher.dispatch(&make_req(2, 1, act_item)).unwrap();
    assert_eq!(
        extract_text_field(&act_resp, TAG_UNIQUE_IDENTIFIER).unwrap(),
        sym_id
    );

    // Locate
    let mut loc_item = Structure::new();
    let _ = loc_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(LOCATE_OPERATION),
    ));
    let _ = loc_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(Structure::new()),
    ));
    let loc_resp = dispatcher.dispatch(&make_req(2, 1, loc_item)).unwrap();
    assert!(extract_payload_child(&loc_resp, TAG_UNIQUE_IDENTIFIER).is_some());

    // Encrypt & Decrypt
    let mut enc_item = Structure::new();
    let _ = enc_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(ENCRYPT_OPERATION),
    ));
    let mut enc_payload = Structure::new();
    let _ = enc_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(sym_id.clone()),
    ));
    let _ = enc_payload.try_push(ttlv_item(
        TAG_DATA,
        Value::byte_string(b"secret payload".to_vec()),
    ));
    let _ = enc_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(enc_payload),
    ));

    let enc_resp = dispatcher.dispatch(&make_req(2, 1, enc_item)).unwrap();
    let enc_child = extract_payload_child(&enc_resp, TAG_DATA).unwrap();
    let mut cipher_bytes = Vec::new();
    enc_child.with_value(|v| {
        if let ValueView::ByteString(b) = v {
            cipher_bytes = b.to_vec();
        }
    });
    let iv_child = extract_payload_child(&enc_resp, TAG_IV_COUNTER_NONCE).unwrap();
    let mut iv_bytes = Vec::new();
    iv_child.with_value(|v| {
        if let ValueView::ByteString(b) = v {
            iv_bytes = b.to_vec();
        }
    });

    let mut dec_item = Structure::new();
    let _ = dec_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(DECRYPT_OPERATION),
    ));
    let mut dec_payload = Structure::new();
    let _ = dec_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string(sym_id.clone()),
    ));
    let _ = dec_payload.try_push(ttlv_item(TAG_DATA, Value::byte_string(cipher_bytes)));
    let _ = dec_payload.try_push(ttlv_item(
        TAG_IV_COUNTER_NONCE,
        Value::byte_string(iv_bytes),
    ));
    let _ = dec_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(dec_payload),
    ));

    let dec_resp = dispatcher.dispatch(&make_req(2, 1, dec_item)).unwrap();
    let dec_child = extract_payload_child(&dec_resp, TAG_DATA).unwrap();
    dec_child.with_value(|v| {
        if let ValueView::ByteString(b) = v {
            assert_eq!(b, b"secret payload");
        }
    });

    // Hash & RNG
    let mut hash_item = Structure::new();
    let _ = hash_item.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(HASH_OPERATION)));
    let mut hash_payload = Structure::new();
    let _ = hash_payload.try_push(ttlv_item(
        TAG_DATA,
        Value::byte_string(b"hash input".to_vec()),
    ));
    let _ = hash_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(hash_payload),
    ));

    let hash_resp = dispatcher.dispatch(&make_req(2, 1, hash_item)).unwrap();
    assert!(extract_payload_child(&hash_resp, TAG_DATA).is_some());

    let mut rng_item = Structure::new();
    let _ = rng_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(RNG_RETRIEVE_OPERATION),
    ));
    let mut rng_payload = Structure::new();
    let _ = rng_payload.try_push(ttlv_item(TAG_DATA_LENGTH, Value::integer(32)));
    let _ = rng_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(rng_payload),
    ));

    let rng_resp = dispatcher.dispatch(&make_req(2, 1, rng_item)).unwrap();
    assert!(extract_payload_child(&rng_resp, TAG_DATA).is_some());

    // 9. Error paths: ItemNotFound
    let mut not_found_item = Structure::new();
    let _ = not_found_item.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(GET_OPERATION)));
    let mut not_found_payload = Structure::new();
    let _ = not_found_payload.try_push(ttlv_item(
        TAG_UNIQUE_IDENTIFIER,
        Value::text_string("non-existent-id".into()),
    ));
    let _ = not_found_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(not_found_payload),
    ));

    let not_found_resp = dispatcher
        .dispatch(&make_req(2, 1, not_found_item))
        .unwrap();
    let batch_res = extract_child(&not_found_resp, TAG_BATCH_ITEM).unwrap();
    let result_status = extract_child(&batch_res, TAG_RESULT_STATUS).unwrap();
    result_status.with_value(|v| {
        if let ValueView::Enumeration(e) = v {
            assert_eq!(*e, STATUS_OPERATION_FAILED);
        }
    });

    // 10. Unsupported operation
    let mut unsupported_item = Structure::new();
    let _ = unsupported_item.try_push(ttlv_item(TAG_OPERATION, Value::enumeration(0x9999)));
    let _ = unsupported_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(Structure::new()),
    ));

    let unsup_resp = dispatcher
        .dispatch(&make_req(2, 1, unsupported_item))
        .unwrap();
    let batch_res = extract_child(&unsup_resp, TAG_BATCH_ITEM).unwrap();
    let result_status = extract_child(&batch_res, TAG_RESULT_STATUS).unwrap();
    result_status.with_value(|v| {
        if let ValueView::Enumeration(e) = v {
            assert_eq!(*e, STATUS_OPERATION_FAILED);
        }
    });
}

#[tokio::test]
async fn test_cli_full_execution() {
    assert!(
        kmipkit_test_server::config::run_cli_with_shutdown(&["kmip".into(), "--help".into()], None)
            .await
            .is_ok()
    );
    assert!(
        kmipkit_test_server::config::run_cli_with_shutdown(
            &["kmip".into(), "--invalid".into()],
            None
        )
        .await
        .is_err()
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "kmip-cli-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let pki_dir = temp_dir.join("pki");
    let data_dir = temp_dir.join("data");

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = tx.send(());
    });

    let args = vec![
        "kmip".into(),
        "--listen".into(),
        "127.0.0.1:0".into(),
        "--data-dir".into(),
        data_dir.display().to_string(),
        "--export-pki".into(),
        pki_dir.display().to_string(),
        "--clean".into(),
        "--no-client-auth".into(),
        "--print-env".into(),
        "--json".into(),
    ];
    let res = kmipkit_test_server::config::run_cli_with_shutdown(&args, Some(rx)).await;
    assert!(res.is_ok());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_codec_format_detection_and_encoding() {
    use kmipkit_test_server::codec::Format;

    assert_eq!(Format::detect(Some("application/json"), b""), Format::Json);
    assert_eq!(Format::detect(Some("text/xml"), b""), Format::Xml);
    assert_eq!(Format::detect(Some("application/xml"), b""), Format::Xml);
    assert_eq!(
        Format::detect(Some("application/octet-stream"), b""),
        Format::Ttlv
    );

    assert_eq!(Format::detect(None, b"  {\"tag\":\"foo\"}"), Format::Json);
    assert_eq!(Format::detect(None, b"  <foo/>"), Format::Xml);
    assert_eq!(Format::detect(None, b"\x42\x00\x00"), Format::Ttlv);

    assert_eq!(Format::Ttlv.content_type(), "application/octet-stream");
    assert_eq!(Format::Json.content_type(), "application/json");
    assert_eq!(Format::Xml.content_type(), "text/xml");

    let item = ttlv_item(TAG_UNIQUE_IDENTIFIER, Value::text_string("test-id".into()));
    let ttlv_bytes = Format::Ttlv.encode(&item).unwrap();
    let json_bytes = Format::Json.encode(&item).unwrap();
    let xml_bytes = Format::Xml.encode(&item).unwrap();

    assert_eq!(
        Format::Ttlv.decode(&ttlv_bytes).unwrap().tag().raw(),
        TAG_UNIQUE_IDENTIFIER
    );
    assert_eq!(
        Format::Json.decode(&json_bytes).unwrap().tag().raw(),
        TAG_UNIQUE_IDENTIFIER
    );
    assert_eq!(
        Format::Xml.decode(&xml_bytes).unwrap().tag().raw(),
        TAG_UNIQUE_IDENTIFIER
    );
}

#[test]
fn test_json_codec_all_types_and_errors() {
    let int_tag = RawTag::new(0x0042_006A).unwrap().try_checked().unwrap();
    let str_tag = RawTag::new(0x0042_0093).unwrap().try_checked().unwrap();
    let enum_tag = RawTag::new(0x0042_005C).unwrap().try_checked().unwrap();
    let bytes_tag = RawTag::new(0x0042_0080).unwrap().try_checked().unwrap();
    let long_tag = RawTag::new(0x0042_0001).unwrap().try_checked().unwrap();
    let big_tag = RawTag::new(0x0042_0002).unwrap().try_checked().unwrap();
    let bool_tag = RawTag::new(0x0042_0003).unwrap().try_checked().unwrap();
    let dt_tag = RawTag::new(0x0042_0004).unwrap().try_checked().unwrap();
    let interval_tag = RawTag::new(0x0042_0005).unwrap().try_checked().unwrap();
    let dte_tag = RawTag::new(0x0042_0006).unwrap().try_checked().unwrap();

    let mut s = Structure::new();
    s.try_push(Item::new(int_tag, Value::integer(123)).unwrap())
        .unwrap();
    s.try_push(Item::new(str_tag, Value::text_string("str".into())).unwrap())
        .unwrap();
    s.try_push(Item::new(enum_tag, Value::enumeration(1)).unwrap())
        .unwrap();
    s.try_push(Item::new(bytes_tag, Value::byte_string(vec![0xAA, 0xBB])).unwrap())
        .unwrap();
    s.try_push(Item::new(long_tag, Value::long_integer(999_999)).unwrap())
        .unwrap();
    s.try_push(Item::new(big_tag, Value::big_integer(vec![0, 0, 0, 0, 0, 0, 0, 1])).unwrap())
        .unwrap();
    s.try_push(Item::new(bool_tag, Value::boolean(false)).unwrap())
        .unwrap();
    s.try_push(Item::new(dt_tag, Value::date_time(1_700_000_000)).unwrap())
        .unwrap();
    s.try_push(Item::new(interval_tag, Value::interval(60)).unwrap())
        .unwrap();
    s.try_push(Item::new(dte_tag, Value::date_time_extended(1_700_000_000_000)).unwrap())
        .unwrap();

    let root_tag = RawTag::new(0x0042_0078).unwrap().try_checked().unwrap();
    let item = Item::new(root_tag, Value::structure(s)).unwrap();

    let enc = encode_json(&item).expect("encode json");
    let dec = decode_json(enc.as_bytes()).expect("decode json");
    assert_eq!(dec.tag().raw(), 0x0042_0078);

    // Negative / edge cases in JSON decoding:
    assert!(decode_json(b"[]").is_err());
    assert!(decode_json(br#"{"type":"Integer"}"#).is_err());
    assert!(decode_json(br#"{"tag":"ProtocolVersionMajor"}"#).is_err());
    assert!(decode_json(br#"{"tag":"0x999999","value":1}"#).is_err());
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"Integer","value":"invalid"}"#)
            .is_err()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"LongInteger","value":"invalid"}"#)
            .is_err()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"BigInteger","value":"invalid hex"}"#)
            .is_err()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"Enumeration","value":"invalid"}"#)
            .is_err()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"Boolean","value":"not a bool"}"#)
            .is_err()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"DateTime","value":"bad-date"}"#)
            .is_err()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"Interval","value":"bad"}"#).is_err()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"DateTimeExtended","value":"bad"}"#)
            .is_err()
    );

    // Additional JSON decoding branches:
    assert_eq!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","value":42}"#)
            .unwrap()
            .tag()
            .raw(),
        0x0042_006A
    );
    assert_eq!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","value":true}"#)
            .unwrap()
            .tag()
            .raw(),
        0x0042_006A
    );
    assert_eq!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","value":"hello"}"#)
            .unwrap()
            .tag()
            .raw(),
        0x0042_006A
    );
    assert!(
        decode_json(br#"{"tag":"Operation","type":"Enumeration","value":"0x00000001"}"#).is_ok()
    );
    assert!(decode_json(br#"{"tag":"Operation","type":"Enumeration","value":1}"#).is_ok());
    assert!(decode_json(br#"{"tag":"TimeStamp","type":"DateTime","value":1700000000}"#).is_ok());
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"Integer","value":"42"}"#).is_ok()
    );
    assert!(
        decode_json(br#"{"tag":"ProtocolVersionMajor","type":"LongInteger","value":"123456789"}"#)
            .is_ok()
    );
    assert!(decode_json(br#"{"tag":"0x0042006A","type":"Integer","value":1}"#).is_ok());
}

#[test]
fn test_xml_codec_all_types_and_errors() {
    let int_tag = RawTag::new(0x0042_006A).unwrap().try_checked().unwrap();
    let str_tag = RawTag::new(0x0042_0093).unwrap().try_checked().unwrap();
    let enum_tag = RawTag::new(0x0042_005C).unwrap().try_checked().unwrap();
    let bytes_tag = RawTag::new(0x0042_0080).unwrap().try_checked().unwrap();
    let long_tag = RawTag::new(0x0042_0001).unwrap().try_checked().unwrap();
    let big_tag = RawTag::new(0x0042_0002).unwrap().try_checked().unwrap();
    let bool_tag = RawTag::new(0x0042_0003).unwrap().try_checked().unwrap();
    let dt_tag = RawTag::new(0x0042_0004).unwrap().try_checked().unwrap();
    let interval_tag = RawTag::new(0x0042_0005).unwrap().try_checked().unwrap();
    let dte_tag = RawTag::new(0x0042_0006).unwrap().try_checked().unwrap();

    let mut s = Structure::new();
    s.try_push(Item::new(int_tag, Value::integer(123)).unwrap())
        .unwrap();
    s.try_push(
        Item::new(
            str_tag,
            Value::text_string("<special & \"chars\" '>".into()),
        )
        .unwrap(),
    )
    .unwrap();
    s.try_push(Item::new(enum_tag, Value::enumeration(1)).unwrap())
        .unwrap();
    s.try_push(Item::new(bytes_tag, Value::byte_string(vec![0xAA, 0xBB])).unwrap())
        .unwrap();
    s.try_push(Item::new(long_tag, Value::long_integer(999_999)).unwrap())
        .unwrap();
    s.try_push(Item::new(big_tag, Value::big_integer(vec![0, 0, 0, 0, 0, 0, 0, 1])).unwrap())
        .unwrap();
    s.try_push(Item::new(bool_tag, Value::boolean(true)).unwrap())
        .unwrap();
    s.try_push(Item::new(dt_tag, Value::date_time(1_700_000_000)).unwrap())
        .unwrap();
    s.try_push(Item::new(interval_tag, Value::interval(60)).unwrap())
        .unwrap();
    s.try_push(Item::new(dte_tag, Value::date_time_extended(1_700_000_000_000)).unwrap())
        .unwrap();

    let root_tag = RawTag::new(0x0042_0078).unwrap().try_checked().unwrap();
    let item = Item::new(root_tag, Value::structure(s)).unwrap();

    let enc = encode_xml(&item).expect("encode xml");
    let dec = decode_xml(enc.as_bytes()).expect("decode xml");
    assert_eq!(dec.tag().raw(), 0x0042_0078);

    // Empty structure
    let empty_item = Item::new(root_tag, Value::structure(Structure::new())).unwrap();
    let empty_enc = encode_xml(&empty_item).unwrap();
    assert!(empty_enc.contains("<RequestMessage/>"));
    let empty_dec = decode_xml(empty_enc.as_bytes()).unwrap();
    assert_eq!(empty_dec.tag().raw(), 0x0042_0078);

    // XML errors
    assert!(decode_xml(b"<unclosed").is_err());
    assert!(decode_xml(b"<UnknownTag/>").is_err());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"Integer\" value=\"not-num\"/>").is_err());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"LongInteger\" value=\"not-num\"/>").is_err());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"BigInteger\" value=\"not-hex\"/>").is_err());
    assert!(
        decode_xml(b"<ProtocolVersionMajor type=\"Enumeration\" value=\"not-enum\"/>").is_err()
    );
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"Boolean\" value=\"not-bool\"/>").is_err());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"DateTime\" value=\"not-date\"/>").is_err());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"Interval\" value=\"not-int\"/>").is_err());
    assert!(
        decode_xml(b"<ProtocolVersionMajor type=\"DateTimeExtended\" value=\"not-dte\"/>").is_err()
    );
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"UnknownType\" value=\"val\"/>").is_err());

    // Additional XML decoding branches:
    assert!(decode_xml(b"<Tag_0x0042006A type=\"Integer\" value=\"1\"/>").is_ok());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"Boolean\" value=\"1\"/>").is_ok());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"Boolean\" value=\"0\"/>").is_ok());
    assert!(decode_xml(b"<ProtocolVersionMajor type=\"Boolean\" value=\"false\"/>").is_ok());
    assert!(decode_xml(b"<Operation type=\"Enumeration\" value=\"0x00000001\"/>").is_ok());
}

#[test]
fn test_json_store_locate_filters_and_edge_cases() {
    let store = JsonStore::ephemeral();

    let mut obj1 = StoredObject::new("id-1", "SymmetricKey", "Active", 1_700_000_000);
    obj1.cryptographic_algorithm = Some("AES".into());
    obj1.cryptographic_usage_mask = Some(0x0000_000C);
    obj1.attributes.insert(
        "Name".into(),
        vec![StoredAttribute {
            name: "Name".into(),
            index: 0,
            value: serde_json::Value::String("MyAESKey".into()),
        }],
    );
    store.save(&obj1).unwrap();

    let mut obj2 = StoredObject::new("id-2", "PrivateKey", "PreActive", 1_700_000_100);
    obj2.cryptographic_algorithm = Some("RSA".into());
    obj2.cryptographic_usage_mask = Some(0x0000_0001);
    store.save(&obj2).unwrap();

    let f_type = LocateFilter {
        object_type: Some("SymmetricKey".into()),
        ..Default::default()
    };
    assert_eq!(store.locate(&f_type), vec!["id-1".to_string()]);

    let f_state = LocateFilter {
        state: Some("PreActive".into()),
        ..Default::default()
    };
    assert_eq!(store.locate(&f_state), vec!["id-2".to_string()]);

    let f_algo = LocateFilter {
        cryptographic_algorithm: Some("AES".into()),
        ..Default::default()
    };
    assert_eq!(store.locate(&f_algo), vec!["id-1".to_string()]);

    let f_mask = LocateFilter {
        cryptographic_usage_mask: Some(0x0000_0004),
        ..Default::default()
    };
    assert_eq!(store.locate(&f_mask), vec!["id-1".to_string()]);

    let f_name = LocateFilter {
        name: Some("MyAESKey".into()),
        ..Default::default()
    };
    assert_eq!(store.locate(&f_name), vec!["id-1".to_string()]);

    let f_none = LocateFilter {
        name: Some("NonExistent".into()),
        ..Default::default()
    };
    assert_eq!(store.locate(&f_none).len(), 0);

    assert!(!store.delete("non-existent").unwrap());
    assert!(StoredObject::from_json_value(&serde_json::json!({"bad": 1})).is_none());
}

#[tokio::test]
async fn test_transport_handler_duplex() {
    use kmipkit_test_server::transport::handler::handle_connection;

    let store = Arc::new(JsonStore::ephemeral());
    let dispatcher = Arc::new(Dispatcher::new(store));

    let (client, server) = tokio::io::duplex(64);
    drop(client);
    assert!(
        handle_connection(server, Arc::clone(&dispatcher))
            .await
            .is_ok()
    );

    let (mut client, server) = tokio::io::duplex(64);
    use tokio::io::AsyncWriteExt;
    tokio::spawn(async move {
        let _ = client.write_all(&[0x99]).await;
    });
    assert!(
        handle_connection(server, Arc::clone(&dispatcher))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_server_handle_and_no_client_auth() {
    use kmipkit_test_server::TestServer;

    let config = ServerConfig::new()
        .with_listen_addr("127.0.0.1:0".parse().unwrap())
        .with_require_client_cert(false);

    let handle = TestServer::start(config)
        .await
        .expect("start server with no client auth");
    assert!(handle.port() > 0);
    assert_ne!(handle.pki().server_cert_pem, "");
    assert!(handle.store().get("non-existent").is_none());

    handle.shutdown().await.expect("shutdown cleanly");
}
