//! KMIPKIT-0056: OASIS KMIP Spec v2.1 §10.2.3 and Profiles v2.1
//! §§5.4.1.1–5.4.1.6.13, 5.4.2 and 5.4.4.1. Codec replay is not server
//! interoperability, full client profile conformance, or certification.
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Value, ValueView};
use quickcheck::QuickCheck;

use super::{decode, encode};
use crate::wire_codec_common::WireCodecErrorKind;

fn item(tag: u32, value: Value) -> Item {
    Item::new(RawTag::new(tag).unwrap().try_checked().unwrap(), value).unwrap()
}

fn xml(item: &Item) -> String {
    String::from_utf8(encode(item, &CodecLimits::defaults()).unwrap().to_vec()).unwrap()
}

fn parse(input: &str) -> Item {
    decode(input.as_bytes(), &CodecLimits::defaults()).unwrap()
}

fn equal(left: &Item, right: &Item) {
    assert_eq!(left.tag(), right.tag());
    assert_eq!(left.item_type(), right.item_type());
    left.with_value(|a| {
        right.with_value(|b| match (a, b) {
            (ValueView::Structure(a), ValueView::Structure(b)) => {
                assert_eq!(a.children().len(), b.children().len());
                for (a, b) in a.children().iter().zip(b.children()) {
                    equal(a, b);
                }
            }
            (ValueView::Integer(a), ValueView::Integer(b)) => assert_eq!(a, b),
            (ValueView::LongInteger(a), ValueView::LongInteger(b))
            | (ValueView::DateTime(a), ValueView::DateTime(b))
            | (ValueView::DateTimeExtended(a), ValueView::DateTimeExtended(b)) => assert_eq!(a, b),
            (ValueView::Enumeration(a), ValueView::Enumeration(b))
            | (ValueView::Interval(a), ValueView::Interval(b)) => assert_eq!(a, b),
            (ValueView::ByteString(a), ValueView::ByteString(b))
            | (ValueView::BigInteger(a), ValueView::BigInteger(b)) => assert_eq!(a, b),
            (ValueView::TextString(a), ValueView::TextString(b)) => assert_eq!(a, b),
            (ValueView::Boolean(a), ValueView::Boolean(b)) => assert_eq!(a, b),
            _ => panic!("incompatible item types"),
        })
    });
}

// KMIPKIT-0056-R001: Profiles §§5.4.1.1–5.4.1.6.2.
#[test]
fn ordered_repeated_children_and_default_structure_survive_roundtrip() {
    let input = "<RequestPayload><QueryFunction type=\"Enumeration\" value=\"QueryOperations\"/><QueryFunction type=\"Enumeration\" value=\"QueryObjects\"/></RequestPayload>";
    let root = parse(input);
    root.with_value(|view| {
        let ValueView::Structure(s) = view else {
            panic!("structure")
        };
        assert_eq!(s.children().len(), 2);
        equal(&s.children()[0], &item(0x420074, Value::enumeration(1)));
        equal(&s.children()[1], &item(0x420074, Value::enumeration(2)));
    });
    equal(&root, &parse(&xml(&root)));
}

#[test]
fn general_form_namespaces_and_hex_tags_resolve_to_raw_identity() {
    for input in [
        "<TTLV tag=\"0x540001\" name=\"VendorThing\" type=\"Integer\" value=\"-1\"/>",
        "<k:TTLV xmlns:k=\"urn:oasis:tc:kmip:xmlns\" tag=\"0x540001\" type=\"Integer\" value=\"0xffffffff\"/>",
        "<TTLV xmlns=\"urn:oasis:tc:kmip:xmlns\" tag=\"0x540001\" type=\"Integer\" value=\"-1\"/>",
    ] {
        equal(&parse(input), &item(0x540001, Value::integer(-1)));
    }
    assert_eq!(
        xml(&item(0x540001, Value::integer(-1))),
        "<TTLV tag=\"0x540001\" type=\"Integer\" value=\"-1\"/>"
    );
    equal(
        &parse("<TTLV tag=\"BatchCount\" type=\"Integer\" value=\"2\"/>"),
        &item(0x42000d, Value::integer(2)),
    );
}

// KMIPKIT-0056-R002: Profiles §§5.4.1.2, 5.4.1.6.3–5.4.1.6.13.
#[test]
fn all_scalar_types_have_independently_specified_xml_values() {
    let cases = [
        (Value::integer(i32::MIN), "Integer", "-2147483648"),
        (
            Value::long_integer(i64::MIN),
            "LongInteger",
            "-9223372036854775808",
        ),
        (
            Value::big_integer(vec![0xff; 8]),
            "BigInteger",
            "FFFFFFFFFFFFFFFF",
        ),
        (Value::enumeration(0xffff_ffff), "Enumeration", "0xFFFFFFFF"),
        (Value::boolean(true), "Boolean", "true"),
        (
            Value::text_string("A<&\"'\t\r\n😀".into()),
            "TextString",
            "A&lt;&amp;&quot;&apos;&#x9;&#xD;&#xA;😀",
        ),
        (
            Value::byte_string(vec![0, 0xab, 0xff]),
            "ByteString",
            "00ABFF",
        ),
        (Value::date_time(0), "DateTime", "1970-01-01T00:00:00Z"),
        (Value::interval(u32::MAX), "Interval", "4294967295"),
        (
            Value::date_time_extended(i64::MIN),
            "DateTimeExtended",
            "-9223372036854775808",
        ),
    ];
    for (value, ty, expected) in cases {
        let root = item(0x540001, value);
        let output = xml(&root);
        assert_eq!(
            output,
            format!("<TTLV tag=\"0x540001\" type=\"{ty}\" value=\"{expected}\"/>")
        );
        equal(&root, &parse(&output));
    }
}

#[test]
fn symbolic_enum_names_and_raw_unknown_values_remain_distinct() {
    assert_eq!(
        xml(&item(0x42005c, Value::enumeration(4))),
        "<Operation type=\"Enumeration\" value=\"ReKey\"/>"
    );
    equal(
        &parse("<Operation type=\"Enumeration\" value=\"ReKey\"/>"),
        &item(0x42005c, Value::enumeration(4)),
    );
    equal(
        &parse("<Operation type=\"Enumeration\" value=\"0x8fffffff\"/>"),
        &item(0x42005c, Value::enumeration(0x8fff_ffff)),
    );
}

// KMIPKIT-0056-R003: Profiles §5.4.1.6.4, shared 0054 assigned M/B metadata.
#[test]
fn masks_use_space_separated_names_and_preserve_unknown_bits() {
    for (tag, input, bits, expected) in [
        (
            0x42002c,
            "Encrypt Decrypt CertificateSign",
            0x100c_u32,
            "Encrypt Decrypt CertificateSign",
        ),
        (
            0x42002c,
            "CertificateSign 0x0000000c 0x80000000",
            0x8000_100c,
            "Encrypt Decrypt CertificateSign 0x80000000",
        ),
        (0x42002c, "-1", u32::MAX, ""),
        (0x42002c, "0x00000040", 0x40, "0x00000040"),
        (
            0x42008e,
            "OnLineStorage ArchivalStorage",
            3,
            "OnLineStorage ArchivalStorage",
        ),
        (0x42015e, "Software Hardware", 3, "Software Hardware"),
        (0x42002c, "0", 0, "0x00000000"),
    ] {
        let source = format!("<TTLV tag=\"0x{tag:06X}\" type=\"Integer\" value=\"{input}\"/>");
        let root = parse(&source);
        equal(
            &root,
            &item(tag, Value::integer(i32::from_be_bytes(bits.to_be_bytes()))),
        );
        let output = xml(&root);
        if !expected.is_empty() {
            assert!(
                output.contains(&format!("value=\"{expected}\"")),
                "{output}"
            );
        }
        equal(&root, &parse(&output));
    }
}

#[test]
fn numeric_lexical_forms_are_range_checked() {
    for (ty, value, expected) in [
        ("Integer", "+0002", Value::integer(2)),
        ("Integer", "0x80000000", Value::integer(i32::MIN)),
        ("LongInteger", "0xFFFFFFFFFFFFFFFF", Value::long_integer(-1)),
        ("Boolean", "1", Value::boolean(true)),
        ("Boolean", "0", Value::boolean(false)),
        ("Interval", "+0002", Value::interval(2)),
        ("ByteString", "", Value::byte_string(vec![])),
        ("TextString", "", Value::text_string(String::new())),
    ] {
        equal(
            &parse(&format!(
                "<TTLV tag=\"0x540001\" type=\"{ty}\" value=\"{value}\"/>"
            )),
            &item(0x540001, expected),
        );
    }
}

#[test]
fn datetime_timezones_leap_days_and_negative_epochs_preserve_seconds() {
    for (value, seconds, canonical) in [
        (
            "2001-01-01T10:00:00+10:00",
            978_307_200,
            "2001-01-01T00:00:00Z",
        ),
        ("1969-12-31T23:59:59Z", -1, "1969-12-31T23:59:59Z"),
        (
            "2000-02-29T00:00:00-02:30",
            951_791_400,
            "2000-02-29T02:30:00Z",
        ),
        ("1970-01-01T24:00:00Z", 86_400, "1970-01-02T00:00:00Z"),
        ("1970-01-01T00:00:00.000Z", 0, "1970-01-01T00:00:00Z"),
    ] {
        let root = parse(&format!(
            "<ActivationDate type=\"DateTime\" value=\"{value}\"/>"
        ));
        equal(&root, &item(0x420001, Value::date_time(seconds)));
        assert!(xml(&root).contains(canonical));
    }
}

#[test]
fn xml_attribute_normalization_preserves_character_references() {
    equal(
        &parse(
            "<TTLV tag=\"0x540001\" type=\"TextString\" value=\"a\r\nb\tc&#xD;&#10;&#9;&lt;&amp;\"/>",
        ),
        &item(0x540001, Value::text_string("a b c\r\n\t<&".into())),
    );
}

// KMIPKIT-0056-R004: common bounded encode/decode contract.
#[test]
fn byte_depth_and_element_limits_apply_to_decode_and_encode() {
    let input = "<RequestPayload><BatchCount type=\"Integer\" value=\"1\"/></RequestPayload>";
    let root = parse(input);
    let encoded_len = xml(&root).len();
    for limits in [
        CodecLimits::new(input.len() - 1, 64, 100).unwrap(),
        CodecLimits::new(1000, 0, 100).unwrap(),
        CodecLimits::new(1000, 64, 1).unwrap(),
    ] {
        assert_eq!(
            decode(input.as_bytes(), &limits).unwrap_err().kind(),
            WireCodecErrorKind::LimitExceeded
        );
    }
    for limits in [
        CodecLimits::new(encoded_len - 1, 64, 100).unwrap(),
        CodecLimits::new(1000, 0, 100).unwrap(),
        CodecLimits::new(1000, 64, 1).unwrap(),
    ] {
        assert_eq!(
            encode(&root, &limits).unwrap_err().kind(),
            WireCodecErrorKind::LimitExceeded
        );
    }
    assert!(
        decode(
            input.as_bytes(),
            &CodecLimits::new(input.len(), 1, 2).unwrap()
        )
        .is_ok()
    );
    assert!(encode(&root, &CodecLimits::new(encoded_len, 1, 2).unwrap()).is_ok());
    assert!(
        decode(
            b"<BatchCount type=\"Integer\" value=\"1\"/>",
            &CodecLimits::new(1000, 0, 1).unwrap()
        )
        .is_ok()
    );
}

#[test]
fn deeply_nested_input_is_rejected_before_building_a_deep_tree() {
    let input = format!(
        "{}{}",
        "<RequestPayload>".repeat(10_000),
        "</RequestPayload>".repeat(10_000)
    );
    assert_eq!(
        decode(input.as_bytes(), &CodecLimits::defaults())
            .unwrap_err()
            .kind(),
        WireCodecErrorKind::LimitExceeded
    );
}

// KMIPKIT-0056-R005: malformed input and parser hardening; never resolves entities.
#[test]
fn malformed_xml_and_external_resolution_requests_are_rejected() {
    for input in [
        "",
        "<RequestPayload>",
        "<RequestPayload></BatchItem>",
        "<RequestPayload/><BatchItem/>",
        "<!DOCTYPE RequestPayload><RequestPayload/>",
        "<!DOCTYPE RequestPayload SYSTEM 'http://127.0.0.1/evil'><RequestPayload/>",
        "<!DOCTYPE RequestPayload [<!ENTITY x SYSTEM 'file:///secret'>]><RequestPayload>&x;</RequestPayload>",
        "<RequestPayload xmlns='https://evil.example'/>",
        "<k:RequestPayload/>",
        "<RequestPayload value='x'/>",
        "<BatchCount type='Integer'/>",
        "<BatchCount type='Integer' value='1' value='2'/>",
        "<BatchCount type='Integer' value='1'><BatchItem/></BatchCount>",
        "<RequestPayload>secret</RequestPayload>",
        "<RequestPayload><![CDATA[secret]]></RequestPayload>",
        "<TTLV type='Integer' value='1'/>",
        "<BatchCount tag='0x42000D' type='Integer' value='1'/>",
        "<BatchCount type='Integer' value='1' ignored='secret'/>",
        "<TTLV tag='0x540001' type='TextString' value='&undeclared;'/>",
        "<TTLV tag='0x540001' type='TextString' value='&#0;'/>",
        "<TTLV tag='0x540001' type='TextString' value='unescaped < secret'/>",
        "<?xml version='1.0' encoding='UTF-16'?><RequestPayload/>",
        "<RequestPayload/><?xml version='1.0'?>",
        "<RequestPayload><!-- bad -- comment --></RequestPayload>",
    ] {
        assert!(
            decode(input.as_bytes(), &CodecLimits::defaults()).is_err(),
            "accepted {input}"
        );
    }
    assert!(decode(&[0xff, 0xfe], &CodecLimits::defaults()).is_err());
    assert!(decode(b"<RequestPayload/>\0", &CodecLimits::defaults()).is_err());
}

#[test]
fn malformed_scalar_values_are_rejected_without_truncation() {
    for (ty, bad) in [
        ("Integer", "2147483648"),
        ("Integer", "-2147483649"),
        ("Integer", "0x1"),
        ("Integer", "0x100000000"),
        ("LongInteger", "9223372036854775808"),
        ("BigInteger", ""),
        ("BigInteger", "FF"),
        ("BigInteger", "0x0000000000000000"),
        ("Enumeration", "1"),
        ("Enumeration", "UnknownSymbol"),
        ("Enumeration", "0x1"),
        ("Boolean", "TRUE"),
        ("Boolean", "2"),
        ("ByteString", "0"),
        ("ByteString", "GG"),
        ("Interval", "-1"),
        ("Interval", "4294967296"),
        ("DateTimeExtended", "9223372036854775808"),
        ("DateTime", "1970-01-01T00:00:00"),
        ("DateTime", "2001-02-29T00:00:00Z"),
        ("DateTime", "1970-01-01T00:00:00.001Z"),
        ("DateTime", "1970-01-01T00:00:00+14:01"),
        ("DateTime", "1970-01-01T24:01:00Z"),
        ("DateTime", "1970-01-01T00:00:60Z"),
        ("UnknownType", "secret"),
    ] {
        assert!(
            decode(
                format!("<TTLV tag='0x540001' type='{ty}' value='{bad}'/>").as_bytes(),
                &CodecLimits::defaults()
            )
            .is_err(),
            "accepted {ty} {bad}"
        );
    }
    for bad in [
        "Encrypt|Decrypt",
        "UnknownMask",
        "Encrypt 0x1",
        "Encrypt 0x100000000",
        "",
    ] {
        assert!(
            decode(
                format!("<CryptographicUsageMask type='Integer' value='{bad}'/>").as_bytes(),
                &CodecLimits::defaults()
            )
            .is_err()
        );
    }
}

#[test]
fn errors_never_retain_or_format_payload_or_parser_context() {
    for input in [
        "<TTLV tag='0x540001' type='Integer' value='SENTINEL_SECRET'/>",
        "<SENTINEL_SECRET/>",
    ] {
        let error = decode(input.as_bytes(), &CodecLimits::defaults()).unwrap_err();
        assert!(!format!("{error:?} {error}").contains("SENTINEL_SECRET"));
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn invalid_model_values_fail_encoding_as_redacted_values() {
    for value in [
        Value::big_integer(vec![]),
        Value::big_integer(vec![1]),
        Value::text_string("bad\0text".into()),
    ] {
        assert_eq!(
            encode(&item(0x540001, value), &CodecLimits::defaults())
                .unwrap_err()
                .kind(),
            WireCodecErrorKind::InvalidValue
        );
    }
}

// KMIPKIT-0056-R006: Profiles §5.4.4.1 official pinned vector replay.
#[test]
fn official_msgenc_xml_m_1_21_replays_all_four_messages() {
    let fixture = include_str!("../fixtures/xml/MSGENC-XML-M-1-21.xml")
        .replace("$NOW", "2023-11-14T22:13:20Z");
    let mut position = 0;
    let mut messages = Vec::new();
    while let Some(start) = fixture[position..]
        .find("<RequestMessage>")
        .map(|p| (position + p, "RequestMessage"))
        .into_iter()
        .chain(
            fixture[position..]
                .find("<ResponseMessage>")
                .map(|p| (position + p, "ResponseMessage")),
        )
        .min_by_key(|(p, _)| *p)
    {
        let closing = format!("</{}>", start.1);
        let end = start.0 + fixture[start.0..].find(&closing).unwrap() + closing.len();
        let root = parse(&fixture[start.0..end]);
        equal(&root, &parse(&xml(&root)));
        messages.push(root);
        position = end;
    }
    assert_eq!(messages.len(), 4);
    fn find(item: &Item, tag: u32) -> Option<i32> {
        item.with_value(|v| match v {
            ValueView::Integer(v) if item.tag().raw() == tag => Some(*v),
            ValueView::Structure(s) => s.children().iter().find_map(|i| find(i, tag)),
            _ => None,
        })
    }
    assert_eq!(find(&messages[0], 0x420050), Some(256));
    assert_eq!(find(&messages[2], 0x420050), Some(2048));
    fn status(item: &Item) -> Option<u32> {
        item.with_value(|v| match v {
            ValueView::Enumeration(v) if item.tag().raw() == 0x42007f => Some(*v),
            ValueView::Structure(s) => s.children().iter().find_map(status),
            _ => None,
        })
    }
    assert_eq!(status(&messages[1]), Some(1));
    assert_eq!(status(&messages[3]), Some(0));
}

#[test]
fn generated_scalar_and_ordered_tree_roundtrips_preserve_raw_values() {
    fn property(integer: i32, long: i64, bytes: Vec<u8>, enumeration: u32) -> bool {
        let mut children = Structure::new();
        for value in [
            Value::integer(integer),
            Value::long_integer(long),
            Value::enumeration(enumeration),
            Value::byte_string(bytes),
            Value::integer(integer),
        ] {
            children.try_push(item(0x540001, value)).unwrap();
        }
        let root = item(0x540002, Value::structure(children));
        equal(&root, &parse(&xml(&root)));
        true
    }
    QuickCheck::new()
        .tests(200)
        .quickcheck(property as fn(i32, i64, Vec<u8>, u32) -> bool);
}

#[test]
fn invalid_character_reference_grammar_and_xml_prolog_are_rejected() {
    for input in [
        "<TTLV tag='0x540001' type='TextString' value='&#x+41;'/>",
        "<TTLV tag='0x540001' type='TextString' value='&#+65;'/>",
        "<?XML version='1.0'?><RequestPayload/>",
        " <!--prolog--><?xml version='1.0'?><RequestPayload/>",
        "<TTLV tag='0x540001' type='DateTime' value='01970-01-01T00:00:00Z'/>",
    ] {
        assert!(decode(input.as_bytes(), &CodecLimits::defaults()).is_err(), "accepted {input}");
    }
}

#[test]
fn depth_preflight_precedes_decoding_forbidden_structure_attributes() {
    let input = b"<RequestPayload><RequestPayload value='&undeclared;'/></RequestPayload>";
    assert_eq!(decode(input, &CodecLimits::new(1000, 1, 100).unwrap()).unwrap_err().kind(), WireCodecErrorKind::LimitExceeded);
}

#[test]
fn namespace_scopes_declarations_comments_and_explicit_scalars_are_supported() {
    let input = "<?xml version='1.0' encoding='utf-8'?><!--before--><k:RequestPayload xmlns:k='urn:oasis:tc:kmip:xmlns'><?instruction ignored?><k:BatchCount type='Integer' value='2'></k:BatchCount><RequestPayload xmlns='urn:oasis:tc:kmip:xmlns'><TTLV xmlns='' tag='0x540001' type='TextString' value='x'/></RequestPayload></k:RequestPayload><!--after-->";
    let root = parse(input);
    equal(&root, &parse(&xml(&root)));
    let empty = parse("<RequestPayload type='Structure'/>");
    equal(&empty, &parse("<RequestPayload></RequestPayload>"));
    assert!(decode(b"<k:RequestPayload xmlns:k='urn:oasis:tc:kmip:xmlns'><k:BatchItem xmlns:k='wrong'/></k:RequestPayload>", &CodecLimits::defaults()).is_err());
}

#[test]
fn datetime_entire_model_range_and_xml_year_boundaries_roundtrip() {
    for seconds in [i64::MIN, i64::MAX, -62_135_596_801, -62_135_596_800, 253_402_300_799, 253_402_300_800] {
        let root = item(0x540001, Value::date_time(seconds));
        equal(&root, &parse(&xml(&root)));
    }
    fn property(seconds: i64) -> bool {
        let root = item(0x540001, Value::date_time(seconds));
        equal(&root, &parse(&xml(&root)));
        true
    }
    QuickCheck::new().tests(200).quickcheck(property as fn(i64) -> bool);
}

#[test]
fn all_assigned_mask_flags_normalize_and_roundtrip_without_naming_placeholders() {
    let cases = [
        (0x42002c, "Sign Verify Encrypt Decrypt WrapKey UnwrapKey MACGenerate MACVerify DeriveKey KeyAgreement CertificateSign CRLSign Authenticate Unrestricted FPEEncrypt FPEDecrypt", 0x00f0_3bbf_u32),
        (0x42015e, "Software Hardware OnProcessor OnSystem OffSystem Hypervisor OperatingSystem Container OnPremises OffPremises SelfManaged Outsourced Validated SameJurisdiction", 0x0000_3fff),
        (0x42008e, "OnLineStorage ArchivalStorage DestroyedStorage", 7),
    ];
    for (tag, flags, bits) in cases {
        let root = parse(&format!("<TTLV tag='0x{tag:06X}' type='Integer' value='{flags}'/>"));
        equal(&root, &item(tag, Value::integer(i32::from_be_bytes(bits.to_be_bytes()))));
        assert!(xml(&root).contains(flags));
    }
}
