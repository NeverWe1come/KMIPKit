//! Unit tests for kmipkit-test-server modules.
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

use std::fs;
use std::sync::Arc;

use kmipkit_test_server::AutoPki;
use kmipkit_test_server::codec::json::{decode_json, encode_json};
use kmipkit_test_server::codec::ttlv::{decode_ttlv, encode_ttlv};
use kmipkit_test_server::codec::xml::{decode_xml, encode_xml};
use kmipkit_test_server::ops::crypto::*;
use kmipkit_test_server::ops::dispatcher::*;
use kmipkit_test_server::store::{JsonStore, LocateFilter, StoredObject};
use kmipkit_test_server::v1_compat::is_version_supported;
use kmipkit_ttlv::{Item, RawTag, Structure, Value};

fn ttlv_item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag).unwrap().try_checked().unwrap();
    Item::new(tag, value).unwrap()
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

    store.save(&obj).expect("save succeeds");

    // Verify readable on disk
    let disk_file = temp_dir.join("objects/id-1234.json");
    assert!(disk_file.exists());
    let disk_content = fs::read_to_string(&disk_file).expect("read file");
    assert!(disk_content.contains("\"unique_identifier\": \"id-1234\""));
    assert!(disk_content.contains("\"cryptographic_algorithm\": \"AES\""));

    // Verify retrieval
    let fetched = store.get("id-1234").expect("found in store");
    assert_eq!(fetched.unique_identifier, "id-1234");
    assert_eq!(fetched.cryptographic_length, Some(256));

    // Test locate
    let results = store.locate(&LocateFilter {
        cryptographic_algorithm: Some("AES".to_owned()),
        ..Default::default()
    });
    assert_eq!(results, vec!["id-1234".to_owned()]);

    // Reload fresh store from existing directory
    let reloaded = JsonStore::new(&temp_dir, false).expect("reloaded store inits");
    let fetched_reloaded = reloaded.get("id-1234").expect("found after reload");
    assert_eq!(fetched_reloaded.unique_identifier, "id-1234");

    // Delete
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

    let root_item = Item::new(tag, Value::structure(structure)).unwrap();
    let encoded = encode_ttlv(&root_item);

    let decoded = decode_ttlv(&encoded).expect("TTLV decode succeeds");
    assert_eq!(decoded.tag().raw(), 0x0042_0078);
    let re_encoded = encode_ttlv(&decoded);
    assert_eq!(encoded, re_encoded);
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
}

#[test]
fn crypto_primitives_work() {
    let rng = generate_random_bytes(32).expect("rng works");
    assert_eq!(rng.len(), 32);

    let data = b"KMIPKit test server crypto";
    let h = hash(0x0000_0005, data).expect("hash works");
    assert_eq!(h.len(), 32);

    let key = generate_random_bytes(32).expect("key generation works");
    let tag = hmac(0x0000_0005, &key, data).expect("hmac works");
    assert!(hmac_verify_tag(0x0000_0005, &key, data, &tag));
    assert!(!hmac_verify_tag(0x0000_0005, &key, b"other data", &tag));

    let (encrypted, iv) = encrypt_aes(&key, None, data).expect("encrypt works");
    let decrypted = decrypt_aes(&key, iv.as_deref(), &encrypted).expect("decrypt works");
    assert_eq!(decrypted, data);
}

#[test]
fn dispatch_discover_versions() {
    let store = Arc::new(JsonStore::ephemeral());
    let dispatcher = Dispatcher::new(store);

    let mut req_version = Structure::new();
    let _ = req_version.try_push(ttlv_item(TAG_PROTOCOL_VERSION_MAJOR, Value::integer(2)));
    let _ = req_version.try_push(ttlv_item(TAG_PROTOCOL_VERSION_MINOR, Value::integer(1)));

    let mut req_header = Structure::new();
    let _ = req_header.try_push(ttlv_item(
        TAG_PROTOCOL_VERSION,
        Value::structure(req_version),
    ));
    let _ = req_header.try_push(ttlv_item(TAG_BATCH_COUNT, Value::integer(1)));

    let mut batch_item = Structure::new();
    let _ = batch_item.try_push(ttlv_item(
        TAG_OPERATION,
        Value::enumeration(DISCOVER_VERSIONS_OPERATION),
    ));
    let _ = batch_item.try_push(ttlv_item(
        TAG_REQUEST_PAYLOAD,
        Value::structure(Structure::new()),
    ));

    let mut req = Structure::new();
    let _ = req.try_push(ttlv_item(TAG_REQUEST_HEADER, Value::structure(req_header)));
    let _ = req.try_push(ttlv_item(TAG_BATCH_ITEM, Value::structure(batch_item)));

    let request = ttlv_item(TAG_REQUEST_MESSAGE, Value::structure(req));
    let response = dispatcher.dispatch(&request).expect("dispatch succeeds");

    assert_eq!(response.tag().raw(), TAG_RESPONSE_MESSAGE);
}

#[test]
fn version_support_checks() {
    assert!(is_version_supported(1, 0));
    assert!(is_version_supported(1, 4));
    assert!(is_version_supported(2, 0));
    assert!(is_version_supported(2, 1));
    assert!(!is_version_supported(3, 0));
}
