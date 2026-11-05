//! KMIPKIT-0055: OASIS KMIP Profiles v2.1 §§5.5.1.1–5.5.1.6.13 and §5.5.4.1.
use super::{decode, encode};
use crate::wire_codec_common::WireCodecErrorKind;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView, codec::CodecLimits};

fn item(tag: u32, value: Value) -> Item {
    Item::new(RawTag::new(tag).unwrap().try_checked().unwrap(), value).unwrap()
}
fn parse(text: &str) -> Item { decode(text.as_bytes(), &CodecLimits::defaults()).unwrap() }
fn wire(item: &Item) -> String { String::from_utf8(encode(item, &CodecLimits::defaults()).unwrap().to_vec()).unwrap() }
fn integer(text: &str) -> i32 { parse(text).with_value(|v| match v { ValueView::Integer(n) => *n, _ => panic!("integer") }) }

#[test]
fn structure_default_null_order_and_repeated_tags_survive() {
    let source = r#"{"value":[{"tag":"BatchCount","type":"Integer","value":2},{"tag":"BatchCount","type":"Integer","value":3}],"tag":"RequestMessage"}"#;
    let decoded = parse(source);
    assert_eq!(decoded.tag().raw(), 0x420078);
    decoded.with_value(|v| { let ValueView::Structure(s) = v else { panic!("structure") }; assert_eq!(s.children().len(), 2); });
    assert_eq!(wire(&decoded), r#"{"tag":"RequestMessage","type":"Structure","value":[{"tag":"BatchCount","type":"Integer","value":2},{"tag":"BatchCount","type":"Integer","value":3}]}"#);
    assert_eq!(wire(&parse(r#"{"tag":"RequestMessage","value":null}"#)), r#"{"tag":"RequestMessage","type":"Structure","value":[]}"#);
}

#[test]
fn scalar_profile_forms_and_exact_payloads_roundtrip() {
    let cases = [
        ("Integer", "-2147483648"), ("Integer", "\"0xffffffff\""),
        ("LongInteger", "-4503599627370495"), ("LongInteger", "\"0x8000000000000000\""),
        ("BigInteger", "0"), ("BigInteger", "\"0xffffffffffffffffffffffffffffffff\""),
        ("Enumeration", "4294967295"), ("Enumeration", "\"0xffffffff\""),
        ("Boolean", "true"), ("Boolean", "\"0x0000000000000000\""),
        ("TextString", "\"a\\n\\u0000\\ud83d\\ude00é\""), ("ByteString", "\"00aBFf\""),
        ("DateTime", "\"2001-01-01T10:00:00.999+10:00\""), ("DateTime", "\"0xffffffffffffffff\""),
        ("Interval", "4294967295"), ("Interval", "\"0x0000001b\""),
        ("DateTimeExtended", "-2"), ("DateTimeExtended", "\"0x7fffffffffffffff\""),
    ];
    for (ty, value) in cases {
        let source = format!(r#"{{"tag":"0x540001","type":"{ty}","value":{value}}}"#);
        let first = parse(&source); let encoded = wire(&first); let second = parse(&encoded);
        assert_eq!(wire(&second), encoded, "{ty}");
    }
    assert!(wire(&parse(r#"{"tag":"ObjectType","type":"Enumeration","value":2}"#)).contains("SymmetricKey"));
    assert!(wire(&parse(r#"{"tag":"0x540001","name":"VendorHint","type":"Integer","value":1}"#)).contains("0x540001"));
    parse(r#"{"tag":"ActivationDate","type":"DateTime","value":"2000-02-29T00:00:00Z"}"#).with_value(|v| assert!(matches!(v, ValueView::DateTime(951782400))));
}

#[test]
fn masks_use_assigned_components_and_hex_unknown_bits() {
    assert_eq!(integer(r#"{"tag":"CryptographicUsageMask","type":"Integer","value":"Encrypt|Decrypt|CertificateSign|0x80000000"}"#), i32::from_be_bytes(0x8000100cu32.to_be_bytes()));
    let encoded = wire(&item(0x42002c, Value::integer(i32::from_be_bytes(0x8000100cu32.to_be_bytes()))));
    assert!(encoded.contains("Encrypt|Decrypt|CertificateSign|0x80000000"));
    assert_eq!(integer(r#"{"tag":"CryptographicUsageMask","type":"Integer","value":4108}"#), 4108);
    assert_eq!(integer(r#"{"tag":"StorageStatusMask","type":"Integer","value":"OnLineStorage|ArchivalStorage"}"#), 3);
    assert!(wire(&item(0x42002c, Value::integer(0))).contains("0x00000000"));
}

#[test]
fn malformed_documents_and_values_are_rejected_without_payload_errors() {
    let invalid = [
        "", "[]", "null", "{}", "{", "{\"tag\":\"BatchCount\",\"value\":1} trailing",
        r#"{"tag":"BatchCount","tag":"BatchCount","type":"Integer","value":1}"#,
        r#"{"tag":"BatchCount","type":"Integer","value":1,"value":2}"#,
        r#"{"tag":"BatchCount","type":"Integer","value":01}"#,
        r#"{"tag":"BatchCount","type":"Integer","value":2147483648}"#,
        r#"{"tag":"BatchCount","type":"Integer","value":1.0}"#,
        r#"{"tag":"BatchCount","type":"Integer","value":"0x01"}"#,
        r#"{"tag":"ObjectType","type":"Enumeration","value":"BadSecretName"}"#,
        r#"{"tag":"BadSecretTag","type":"Integer","value":1}"#,
        r#"{"tag":"0x420000","type":"Integer","value":1}"#,
        r#"{"tag":"0x540001","type":"ByteString","value":"0x12"}"#,
        r#"{"tag":"0x540001","type":"ByteString","value":"F"}"#,
        r#"{"tag":"0x540001","type":"BigInteger","value":"0x01"}"#,
        r#"{"tag":"0x540001","type":"LongInteger","value":4503599627370496}"#,
        r#"{"tag":"0x540001","type":"Boolean","value":"0x0000000000000002"}"#,
        r#"{"tag":"0x540001","type":"TextString","value":"\ud800"}"#,
        r#"{"tag":"0x540001","type":"TextString","value":null}"#,
        r#"{"tag":"0x540001","type":"DateTime","value":"2001-01-01T10:00:00"}"#,
        r#"{"tag":"0x540001","type":"DateTime","value":"2001-02-29T00:00:00Z"}"#,
        r#"{"tag":"CryptographicUsageMask","type":"Integer","value":"Encrypt||Decrypt"}"#,
        r#"{"tag":"CryptographicUsageMask","type":"Integer","value":"Encrypt|BadSecretMask"}"#,
        r#"{"tag":"BatchCount","type":"Unknown","value":1}"#,
        r#"{"tag":"RequestMessage","value":[1]}"#,
        r#"{"tag":"RequestMessage","value":[],}"#,
    ];
    for source in invalid {
        let err = decode(source.as_bytes(), &CodecLimits::defaults()).unwrap_err();
        assert!(!format!("{err:?} {err}").contains("Secret"));
    }
}

#[test]
fn limits_apply_to_wire_bytes_structure_depth_and_item_count() {
    let source = br#"{"tag":"RequestMessage","value":[{"tag":"BatchCount","type":"Integer","value":1}]}"#;
    for limits in [CodecLimits::new(source.len()-1,64,10).unwrap(), CodecLimits::new(1024,0,10).unwrap(), CodecLimits::new(1024,64,1).unwrap()] {
        assert_eq!(decode(source, &limits).unwrap_err().kind(), WireCodecErrorKind::LimitExceeded);
    }
    let decoded = decode(source, &CodecLimits::new(source.len(),1,2).unwrap()).unwrap();
    for limits in [CodecLimits::new(1,64,10).unwrap(), CodecLimits::new(1024,0,10).unwrap(), CodecLimits::new(1024,64,1).unwrap()] {
        assert_eq!(encode(&decoded, &limits).unwrap_err().kind(), WireCodecErrorKind::LimitExceeded);
    }
    let mut nested = Structure::new(); nested.try_push(item(0x420078,Value::structure(Structure::new()))).unwrap();
    assert_eq!(encode(&item(0x420078,Value::structure(nested)), &CodecLimits::new(1024,1,10).unwrap()).unwrap_err().kind(), WireCodecErrorKind::LimitExceeded);
}

#[test]
fn official_msgenc_json_m_1_21_replays_all_four_messages() {
    // Exact official Profiles §5.5.4.1 XML test description, normalized to JSON
    // with the sole fixture binding $NOW = 1700000000.
    fn convert(node: roxmltree::Node<'_, '_>) -> serde_json::Value {
        let tag = node.tag_name().name();
        let ty = node.attribute("type").unwrap_or("Structure");
        let raw = node.attribute("value").unwrap_or("");
        let value = match ty {
            "Structure" => serde_json::Value::Array(node.children().filter(roxmltree::Node::is_element).map(convert).collect()),
            "Integer" => serde_json::json!(raw.parse::<i32>().unwrap()),
            "DateTime" => serde_json::json!("0x000000006553f100"),
            _ => serde_json::json!(raw),
        };
        serde_json::json!({"tag":tag,"type":ty,"value":value})
    }
    let fixture = include_str!("../fixtures/json/MSGENC-JSON-M-1-21.xml");
    let doc = roxmltree::Document::parse(fixture).unwrap();
    let messages: Vec<_> = doc.root_element().children().filter(roxmltree::Node::is_element).collect();
    assert_eq!(messages.len(), 4);
    for message in messages { let json = serde_json::to_vec(&convert(message)).unwrap(); let parsed = decode(&json,&CodecLimits::defaults()).unwrap(); let encoded = encode(&parsed,&CodecLimits::defaults()).unwrap(); assert_eq!(wire(&decode(&encoded,&CodecLimits::defaults()).unwrap()), String::from_utf8(encoded.to_vec()).unwrap()); }
}

