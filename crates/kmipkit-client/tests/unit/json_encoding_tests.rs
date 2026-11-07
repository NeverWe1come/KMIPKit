//! KMIPKIT-0055: OASIS KMIP Profiles v2.1 §§5.5.1.1–5.5.1.6.13 and §5.5.4.1.
use super::{decode, encode};
use crate::wire_codec_common::WireCodecErrorKind;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView, codec::CodecLimits};

fn item(tag: u32, value: Value) -> Item {
    Item::new(RawTag::new(tag).unwrap().try_checked().unwrap(), value).unwrap()
}
fn parse(text: &str) -> Item {
    decode(text.as_bytes(), &CodecLimits::defaults()).unwrap()
}
fn wire(item: &Item) -> String {
    String::from_utf8(encode(item, &CodecLimits::defaults()).unwrap().to_vec()).unwrap()
}
fn integer(text: &str) -> i32 {
    parse(text).with_value(|v| match v {
        ValueView::Integer(n) => *n,
        _ => panic!("integer"),
    })
}

#[test]
fn structure_default_null_order_and_repeated_tags_survive() {
    let source = r#"{"value":[{"tag":"BatchCount","type":"Integer","value":2},{"tag":"BatchCount","type":"Integer","value":3}],"tag":"RequestMessage"}"#;
    let decoded = parse(source);
    assert_eq!(decoded.tag().raw(), 0x0042_0078);
    decoded.with_value(|v| {
        let ValueView::Structure(s) = v else {
            panic!("structure")
        };
        assert_eq!(s.children().len(), 2);
    });
    assert_eq!(
        wire(&decoded),
        r#"{"tag":"RequestMessage","type":"Structure","value":[{"tag":"BatchCount","type":"Integer","value":2},{"tag":"BatchCount","type":"Integer","value":3}]}"#
    );
    assert_eq!(
        wire(&parse(r#"{"tag":"RequestMessage","value":null}"#)),
        r#"{"tag":"RequestMessage","type":"Structure","value":[]}"#
    );
}

#[test]
fn scalar_profile_forms_and_exact_payloads_roundtrip() {
    let cases = [
        ("Integer", "-2147483648"),
        ("Integer", "\"0xffffffff\""),
        ("LongInteger", "-4503599627370495"),
        ("LongInteger", "\"0x8000000000000000\""),
        ("BigInteger", "0"),
        ("BigInteger", "\"0xffffffffffffffffffffffffffffffff\""),
        ("Enumeration", "4294967295"),
        ("Enumeration", "\"0xffffffff\""),
        ("Boolean", "true"),
        ("Boolean", "\"0x0000000000000000\""),
        ("TextString", "\"a\\n\\u0000\\ud83d\\ude00é\""),
        ("ByteString", "\"00aBFf\""),
        ("DateTime", "\"2001-01-01T10:00:00.999+10:00\""),
        ("DateTime", "\"0xffffffffffffffff\""),
        ("Interval", "4294967295"),
        ("Interval", "\"0x0000001b\""),
        ("DateTimeExtended", "-2"),
        ("DateTimeExtended", "\"0x7fffffffffffffff\""),
    ];
    for (ty, value) in cases {
        let source = format!(r#"{{"tag":"0x540001","type":"{ty}","value":{value}}}"#);
        let first = parse(&source);
        let encoded = wire(&first);
        let second = parse(&encoded);
        assert_eq!(wire(&second), encoded, "{ty}");
    }
    assert!(
        wire(&parse(
            r#"{"tag":"ObjectType","type":"Enumeration","value":2}"#
        ))
        .contains("SymmetricKey")
    );
    assert!(
        wire(&parse(
            r#"{"tag":"0x540001","name":"VendorHint","type":"Integer","value":1}"#
        ))
        .contains("0x540001")
    );
    parse(r#"{"tag":"ActivationDate","type":"DateTime","value":"2000-02-29T00:00:00Z"}"#)
        .with_value(|v| assert!(matches!(v, ValueView::DateTime(951_782_400))));
}

#[test]
fn masks_use_assigned_components_and_hex_unknown_bits() {
    assert_eq!(
        integer(
            r#"{"tag":"CryptographicUsageMask","type":"Integer","value":"Encrypt|Decrypt|CertificateSign|0x80000000"}"#
        ),
        i32::from_be_bytes(0x8000_100c_u32.to_be_bytes())
    );
    let encoded = wire(&item(
        0x0042_002c,
        Value::integer(i32::from_be_bytes(0x8000_100c_u32.to_be_bytes())),
    ));
    assert!(encoded.contains("Encrypt|Decrypt|CertificateSign|0x80000000"));
    assert_eq!(
        integer(r#"{"tag":"CryptographicUsageMask","type":"Integer","value":4108}"#),
        4108
    );
    assert_eq!(
        integer(
            r#"{"tag":"StorageStatusMask","type":"Integer","value":"OnLineStorage|ArchivalStorage"}"#
        ),
        3
    );
    assert!(wire(&item(0x0042_002c, Value::integer(0))).contains("0x00000000"));
}

#[test]
fn malformed_documents_and_values_are_rejected_without_payload_errors() {
    let invalid = [
        "",
        "[]",
        "null",
        "{}",
        "{",
        "{\"tag\":\"BatchCount\",\"value\":1} trailing",
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
    let source =
        br#"{"tag":"RequestMessage","value":[{"tag":"BatchCount","type":"Integer","value":1}]}"#;
    for limits in [
        CodecLimits::new(source.len() - 1, 64, 10).unwrap(),
        CodecLimits::new(1024, 0, 10).unwrap(),
        CodecLimits::new(1024, 64, 1).unwrap(),
    ] {
        assert_eq!(
            decode(source, &limits).unwrap_err().kind(),
            WireCodecErrorKind::LimitExceeded
        );
    }
    let decoded = decode(source, &CodecLimits::new(source.len(), 1, 2).unwrap()).unwrap();
    for limits in [
        CodecLimits::new(1, 64, 10).unwrap(),
        CodecLimits::new(1024, 0, 10).unwrap(),
        CodecLimits::new(1024, 64, 1).unwrap(),
    ] {
        assert_eq!(
            encode(&decoded, &limits).unwrap_err().kind(),
            WireCodecErrorKind::LimitExceeded
        );
    }
    let mut nested = Structure::new();
    nested
        .try_push(item(0x0042_0078, Value::structure(Structure::new())))
        .unwrap();
    assert_eq!(
        encode(
            &item(0x0042_0078, Value::structure(nested)),
            &CodecLimits::new(1024, 1, 10).unwrap()
        )
        .unwrap_err()
        .kind(),
        WireCodecErrorKind::LimitExceeded
    );
}

#[test]
fn official_xml_descriptor_replays_all_four_semantic_messages() {
    // Exact official Profiles §5.5.4.1 XML test description, normalized to JSON
    // with the sole fixture binding $NOW = 1700000000.
    fn convert(node: roxmltree::Node<'_, '_>) -> serde_json::Value {
        let tag = node.tag_name().name();
        let ty = node.attribute("type").unwrap_or("Structure");
        let raw = node.attribute("value").unwrap_or("");
        let value = match ty {
            "Structure" => serde_json::Value::Array(
                node.children()
                    .filter(roxmltree::Node::is_element)
                    .map(convert)
                    .collect(),
            ),
            "Integer" => serde_json::json!(raw.parse::<i32>().unwrap()),
            "DateTime" => serde_json::json!("0x000000006553f100"),
            _ => serde_json::json!(raw),
        };
        serde_json::json!({"tag":tag,"type":ty,"value":value})
    }
    let fixture = include_str!("../fixtures/json/MSGENC-JSON-M-1-21.xml");
    let doc = roxmltree::Document::parse(fixture).unwrap();
    let messages: Vec<_> = doc
        .root_element()
        .children()
        .filter(roxmltree::Node::is_element)
        .collect();
    assert_eq!(messages.len(), 4);
    for node in doc
        .descendants()
        .filter(|n| n.is_element() && n.attribute("type").is_some())
    {
        let json = serde_json::to_vec(&convert(node)).unwrap();
        assert!(
            decode(&json, &CodecLimits::defaults()).is_ok(),
            "official node {}",
            node.tag_name().name()
        );
    }
    for message in messages {
        let json = serde_json::to_vec(&convert(message)).unwrap();
        let parsed = decode(&json, &CodecLimits::defaults()).unwrap();
        let encoded = encode(&parsed, &CodecLimits::defaults()).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&encoded).unwrap(),
            convert(message)
        );
        assert_eq!(
            wire(&decode(&encoded, &CodecLimits::defaults()).unwrap()),
            String::from_utf8(encoded.to_vec()).unwrap()
        );
    }
}

#[test]
fn malformed_object_nesting_cannot_bypass_structure_depth_limit() {
    let source = format!("{}null{}", "{\"value\":".repeat(150), "}".repeat(150));
    assert_eq!(
        decode(source.as_bytes(), &CodecLimits::defaults())
            .unwrap_err()
            .kind(),
        WireCodecErrorKind::LimitExceeded
    );
}

#[test]
fn semantic_roundtrip_preserves_every_raw_model_value() {
    #[allow(clippy::needless_pass_by_value)] // QuickCheck requires owned Arbitrary arguments.
    fn check(number: i64, enumeration: u32, text: String, bytes: Vec<u8>) -> bool {
        let mut structure = Structure::new();
        for value in [
            Value::integer(enumeration.cast_signed()),
            Value::long_integer(number),
            Value::enumeration(enumeration),
            Value::text_string(text.clone()),
            Value::byte_string(bytes.clone()),
            Value::big_integer(number.to_be_bytes().to_vec()),
            Value::date_time(number),
            Value::interval(enumeration),
            Value::date_time_extended(number),
            Value::boolean(true),
        ] {
            structure.try_push(item(0x0054_0001, value)).unwrap();
        }
        let original = item(0x0042_0078, Value::structure(structure));
        let decoded = parse(&wire(&original));
        decoded.with_value(|value| {
            let ValueView::Structure(s) = value else {
                return false;
            };
            let c = s.children();
            c.len() == 10
                && c[0].with_value(
                    |v| matches!(v,ValueView::Integer(n) if *n==enumeration.cast_signed()),
                )
                && c[1].with_value(|v| matches!(v,ValueView::LongInteger(n) if *n==number))
                && c[2].with_value(|v| matches!(v,ValueView::Enumeration(n) if *n==enumeration))
                && c[3].with_value(|v| matches!(v,ValueView::TextString(n) if n==text))
                && c[4].with_value(|v| matches!(v,ValueView::ByteString(n) if n==bytes))
                && c[5]
                    .with_value(|v| matches!(v,ValueView::BigInteger(n) if n==number.to_be_bytes()))
                && c[6].with_value(|v| matches!(v,ValueView::DateTime(n) if *n==number))
                && c[7].with_value(|v| matches!(v,ValueView::Interval(n) if *n==enumeration))
                && c[8].with_value(|v| matches!(v,ValueView::DateTimeExtended(n) if *n==number))
                && c[9].with_value(|v| matches!(v, ValueView::Boolean(true)))
        })
    }
    quickcheck::QuickCheck::new()
        .tests(100)
        .quickcheck(check as fn(i64, u32, String, Vec<u8>) -> bool);
}

#[test]
fn malformed_scalars_and_lexemes_cover_rejection_boundaries() {
    for (ty, value) in [
        ("Integer", "1e0"),
        ("Integer", "-"),
        ("Integer", "+1"),
        ("Integer", "false"),
        ("Integer", "\"0xGGGGGGGG\""),
        ("Enumeration", "-1"),
        ("Enumeration", "4294967296"),
        ("Boolean", "0"),
        ("Boolean", "null"),
        ("ByteString", "\"xx\""),
        ("BigInteger", "\"\""),
        ("BigInteger", "\"0x\""),
        ("LongInteger", "\"garbage\""),
        ("DateTimeExtended", "4503599627370496"),
        ("Interval", "-1"),
        ("Interval", "4294967296"),
        ("TextString", "\"\\x\""),
        ("TextString", "\"\\uZZZZ\""),
        ("TextString", "\"\\udc00\""),
        ("TextString", "\"\\ud800\\u0041\""),
        ("TextString", "\"\\u00é\""),
        ("TextString", "\"unterminated"),
        ("DateTime", "\"2001-13-01T00:00:00Z\""),
        ("DateTime", "\"2001-01-32T00:00:00Z\""),
        ("DateTime", "\"2001-01-01T24:00:01Z\""),
        ("DateTime", "\"2001-01-01T00:60:00Z\""),
        ("DateTime", "\"2001-01-01T00:00:60Z\""),
        ("DateTime", "\"2001-01-01T00:00:00+14:01\""),
        ("DateTime", "\"2001-01-01T00:00:00+01:60\""),
        ("DateTime", "\"2001-01-01T00:00:00.Z\""),
        ("DateTime", "\"0000-01-01T00:00:00Z\""),
        ("DateTime", "\"999999999999999999-01-01T00:00:00Z\""),
    ] {
        let json = format!(r#"{{"tag":"0x540001","type":"{ty}","value":{value}}}"#);
        assert!(
            decode(json.as_bytes(), &CodecLimits::defaults()).is_err(),
            "{ty}"
        );
    }
    for source in [
        r#"{"tag":"BatchCount","type":"Integer","value":1,"other":2}"#,
        r#"{"tag":"BatchCount","t\u0061g":"BatchCount","value":[]}"#,
        r#"{"tag":null,"value":[]}"#,
        r#"{"tag":"RequestMessage","name":1,"value":[]}"#,
    ] {
        assert!(decode(source.as_bytes(), &CodecLimits::defaults()).is_err());
    }
    assert!(decode(&[0xff], &CodecLimits::defaults()).is_err());
    assert!(
        encode(
            &item(0x0054_0001, Value::big_integer(vec![])),
            &CodecLimits::defaults()
        )
        .is_err()
    );
    assert!(
        encode(
            &item(0x0054_0001, Value::big_integer(vec![1])),
            &CodecLimits::defaults()
        )
        .is_err()
    );
}

#[test]
fn dates_names_and_escaped_fields_keep_profile_semantics() {
    for (text, expected) in [
        ("1970-01-01T00:00:00Z", 0),
        ("1969-12-31T23:59:59Z", -1),
        ("1970-01-01T01:00:00+01:00", 0),
        ("1969-12-31T23:00:00-01:00", 0),
        ("2001-01-01T10:00:00.999+10:00", 978_307_200),
        ("-0001-01-01T00:00:00Z", -62_167_219_200),
    ] {
        let json = format!(r#"{{"tag":"ActivationDate","type":"DateTime","value":"{text}"}}"#);
        parse(&json).with_value(|v| assert!(matches!(v,ValueView::DateTime(n) if *n==expected)));
    }
    assert_eq!(
        integer(r#"{ "t\u0061g" : "BatchCount", "type":"Integer", "value":1 }"#),
        1
    );
    assert_eq!(
        wire(&item(
            0x0054_0001,
            Value::text_string("\"\\\n\r\t\u{8}\u{c}\0é😀".to_owned())
        )),
        r#"{"tag":"0x540001","type":"TextString","value":"\"\\\n\r\t\u0008\u000c\u0000é😀"}"#
    );
    parse(r#"{"tag":"ProtectionStorageMask","type":"Integer","value":"Software|Hardware|0x80000000"}"#).with_value(|v| assert!(matches!(v,ValueView::Integer(n) if n.cast_unsigned()==0x8000_0003)));
}

#[test]
fn xsd_midnight_24_hour_form_is_normalized_to_following_day() {
    parse(r#"{"tag":"ActivationDate","type":"DateTime","value":"2001-01-01T24:00:00Z"}"#)
        .with_value(|v| assert!(matches!(v,ValueView::DateTime(n) if *n==978_393_600)));
}

#[test]
fn profile_edges_cover_null_hex_boolean_and_json_short_escapes() {
    let empty = br#"{"tag":"RequestMessage","value":null}"#;
    assert_eq!(
        decode(empty, &CodecLimits::new(1024, 0, 10).unwrap())
            .unwrap_err()
            .kind(),
        WireCodecErrorKind::LimitExceeded
    );
    parse(r#"{"tag":"0x540001","type":"Boolean","value":"0x0000000000000001"}"#)
        .with_value(|v| assert!(matches!(v, ValueView::Boolean(true))));
    parse(r#"{"tag":"0x540001","type":"TextString","value":"\/\b\f"}"#)
        .with_value(|v| assert!(matches!(v,ValueView::TextString(s) if s=="/\u{8}\u{c}")));
    parse(r#"{"tag":"ActivationDate","type":"DateTime","value":"2001-04-30T00:00:00Z"}"#)
        .with_value(|v| assert!(matches!(v,ValueView::DateTime(n) if *n==988_588_800)));
    for source in [
        "20x0-01-01T00:00:00Z",
        "2001-01-01T0:00:00Z",
        "2001-01-01T00:00:00:00Z",
        "2001-01-01-01T00:00:00Z",
    ] {
        let json = format!(r#"{{"tag":"ActivationDate","type":"DateTime","value":"{source}"}}"#);
        assert!(decode(json.as_bytes(), &CodecLimits::defaults()).is_err());
    }
}

#[test]
fn default_depth_boundary_accepts_64_structures_and_rejects_65() {
    let mut source = r#"{"tag":"BatchCount","type":"Integer","value":1}"#.to_owned();
    for _ in 0..64 {
        source = format!(r#"{{"tag":"RequestMessage","value":[{source}]}}"#);
    }
    let decoded = decode(source.as_bytes(), &CodecLimits::defaults()).unwrap();
    assert!(encode(&decoded, &CodecLimits::defaults()).is_ok());
    let too_deep = format!(r#"{{"tag":"RequestMessage","value":[{source}]}}"#);
    assert_eq!(
        decode(too_deep.as_bytes(), &CodecLimits::defaults())
            .unwrap_err()
            .kind(),
        WireCodecErrorKind::LimitExceeded
    );
}

#[test]
fn empty_structure_and_exact_encoder_byte_cap_are_valid() {
    let decoded = parse(r#"{"tag":"RequestMessage","value":[]}"#);
    let expected = r#"{"tag":"RequestMessage","type":"Structure","value":[]}"#;
    let limits = CodecLimits::new(expected.len(), 1, 1).unwrap();
    assert_eq!(&*encode(&decoded, &limits).unwrap(), expected.as_bytes());
    assert!(decode(br#"{"tag":"RequestMessage","value":[]}"#, &limits).is_ok());
    assert!(
        wire(&parse(
            r#"{"tag":"CryptographicAlgorithm","type":"Enumeration","value":"DES3"}"#
        ))
        .contains(r#""value":"DES3""#)
    );
    assert_eq!(
        wire(&item(0x0042_002c, Value::integer(0x40))),
        r#"{"tag":"CryptographicUsageMask","type":"Integer","value":"0x00000040"}"#
    );
}

#[test]
fn normative_json_wire_samples_decode_and_emit_equivalent_profile_documents() {
    // OASIS KMIP Profiles v2.1 §5.5.4.1, literal normative JSON wire table.
    // Expected projection applies only profile-permitted representation choices:
    // explicit Structure types, numeric Integers, and hexadecimal DateTime.
    fn emitted_form(mut value: serde_json::Value) -> serde_json::Value {
        let object = value.as_object_mut().unwrap();
        let ty = object
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("Structure");
        match ty {
            "Structure" => {
                let children = object.get_mut("value").unwrap().as_array_mut().unwrap();
                for child in children {
                    *child = emitted_form(child.take());
                }
                object.insert("type".to_owned(), serde_json::json!("Structure"));
            }
            "Integer" => {
                let source = object["value"].as_str().unwrap();
                let n = u32::from_str_radix(source.strip_prefix("0x").unwrap(), 16)
                    .unwrap()
                    .cast_signed();
                object.insert("value".to_owned(), serde_json::json!(n));
            }
            "DateTime" => {
                assert_eq!(object["value"], "2016-01-04T11:47:46+00:00");
                object.insert("value".to_owned(), serde_json::json!("0x00000000568a5be2"));
            }
            _ => (),
        }
        value
    }
    for source in [
        include_bytes!("../fixtures/json/request-time-0.json").as_slice(),
        include_bytes!("../fixtures/json/response-time-0.json").as_slice(),
        include_bytes!("../fixtures/json/request-time-1.json").as_slice(),
    ] {
        let reference = serde_json::from_slice(source).unwrap();
        let decoded = decode(source, &CodecLimits::defaults()).unwrap();
        let emitted = encode(&decoded, &CodecLimits::defaults()).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&emitted).unwrap(),
            emitted_form(reference)
        );
    }
}
