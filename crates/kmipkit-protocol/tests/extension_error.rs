//! KMIPKIT-0012 protocol error contract tests.
//!
//! Traceability: KMIPKIT-0012-FR-011 and KMIPKIT-0012-FR-012.

use std::error::Error;
use std::fmt::{Debug, Display};

use kmipkit_protocol::ProtocolErrorKind;
use kmipkit_protocol::extension;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

const EXTENSION_BODY_TAG: u32 = 0x0054_1236;
const DISCRIMINATOR_TAG: u32 = 0x0054_1234;
const PAYLOAD_TAG: u32 = 0x0054_1235;
const DISCRIMINATOR_TEXT_SENTINEL: &str = "KMIPKIT_VENDOR_DISCRIMINATOR_SECRET_SENTINEL";
const PAYLOAD_TEXT_SENTINEL: &str = "KMIPKIT_VENDOR_PAYLOAD_SECRET_SENTINEL";
const PAYLOAD_BYTES_SENTINEL: &[u8] = b"KMIPKIT_VENDOR_BYTES_SENTINEL";
const PAYLOAD_INTEGER_SENTINEL: i32 = 1_907_031_337;

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the extension fixture tag fits the KMIP tag width")
        .try_checked()
        .expect("the extension fixture tag is in the allocated vendor range")
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(tag(raw_tag), value).expect("a checked fixture tag can form an Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the fixture remains within the TTLV depth limit");
    }
    structure
}

fn extension_definition() -> extension::ExtensionDefinition {
    let identity = extension::extension_identity("ExampleVendor", "diagnostic-test", "1")
        .expect("the extension fixture identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the extension fixture supports KMIP 2.1 and the workspace version");
    let discriminator_path = extension::with_child_tag(
        extension::ttlv_path(tag(EXTENSION_BODY_TAG))
            .expect("the discriminator path starts with its containing tag"),
        tag(DISCRIMINATOR_TAG),
    )
    .expect("the discriminator path contains its declared child tag");
    let discriminator = extension::discriminator(
        discriminator_path,
        Value::text_string(DISCRIMINATOR_TEXT_SENTINEL.to_owned()),
    )
    .expect("the discriminator is a scalar text value");
    let body_schema = extension::structure(
        vec![
            extension::required(
                tag(DISCRIMINATOR_TAG),
                extension::scalar(ItemType::TextString)
                    .expect("Text String is a supported scalar item type"),
            )
            .expect("the discriminator schema child is valid"),
            extension::required(
                tag(PAYLOAD_TAG),
                extension::scalar(ItemType::Integer)
                    .expect("Integer is a supported scalar item type"),
            )
            .expect("the payload schema child is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the extension body schema has unique, valid child rules");
    let schema = extension::structure(
        vec![
            extension::required(tag(EXTENSION_BODY_TAG), body_schema)
                .expect("the extension body schema child is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the extension schema has a valid root rule");

    extension::extension_definition(identity, compatibility, discriminator, schema)
        .expect("the extension definition is internally consistent")
}

fn extension_value(payload: Value) -> Structure {
    structure([item(
        EXTENSION_BODY_TAG,
        Value::structure(structure([
            item(
                DISCRIMINATOR_TAG,
                Value::text_string(DISCRIMINATOR_TEXT_SENTINEL.to_owned()),
            ),
            item(PAYLOAD_TAG, payload),
        ])),
    )])
}

fn all_diagnostics(error: &(impl Debug + Display + Error)) -> String {
    let mut diagnostics = format!("debug={error:?}; display={error}");
    let mut source = error.source();
    while let Some(cause) = source {
        diagnostics.push_str("; source-debug=");
        diagnostics.push_str(&format!("{cause:?}"));
        diagnostics.push_str("; source-display=");
        diagnostics.push_str(&cause.to_string());
        source = cause.source();
    }
    diagnostics
}

fn compact_hex(payload: &[u8], uppercase: bool) -> String {
    let encoded = payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if uppercase {
        encoded.to_ascii_uppercase()
    } else {
        encoded
    }
}

fn spaced_hex(payload: &[u8], uppercase: bool) -> String {
    let encoded = payload
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    if uppercase {
        encoded.to_ascii_uppercase()
    } else {
        encoded
    }
}

fn escaped_bytes(payload: &[u8], uppercase: bool) -> String {
    let encoded = payload
        .iter()
        .map(|byte| format!(r"\x{byte:02x}"))
        .collect::<String>();
    if uppercase {
        encoded.to_ascii_uppercase()
    } else {
        encoded
    }
}

fn assert_text_and_byte_payloads_redacted(
    diagnostics: &str,
    text_sentinels: &[&str],
    byte_payloads: &[&[u8]],
) {
    for sentinel in text_sentinels {
        assert!(
            !diagnostics.contains(sentinel),
            "error diagnostics must not contain caller payload text"
        );
    }

    for payload in byte_payloads {
        let visible_text =
            std::str::from_utf8(payload).expect("the synthetic byte payload is valid UTF-8");
        let representations = [
            visible_text.to_owned(),
            compact_hex(payload, false),
            compact_hex(payload, true),
            spaced_hex(payload, false),
            spaced_hex(payload, true),
            escaped_bytes(payload, false),
            escaped_bytes(payload, true),
            format!("{payload:?}"),
        ];
        for representation in representations {
            assert!(
                !diagnostics.contains(&representation),
                "error diagnostics must not contain a caller byte-payload representation"
            );
        }
    }
}

#[test]
fn protocol_extension_errors_use_the_manifest_categories() {
    let invalid_identity = extension::extension_identity("Example/Vendor", "name", "1")
        .expect_err("Vendor Identification rejects characters outside its allowed set");
    assert_eq!(invalid_identity.kind(), ProtocolErrorKind::InvalidIdentity);

    let incompatible_range = extension::compatibility(1, 0, 1, 9, "0.0.0", "99.0.0")
        .expect_err("a KMIP range that excludes 2.1 is incompatible with this client");
    assert_eq!(
        incompatible_range.kind(),
        ProtocolErrorKind::CompatibilityMismatch
    );

    let above_hard_maximum = extension::with_values(
        1_025, 16_384, 256, 4_096, 1_048_576, 4_096, 1_048_576, 256, 16_384, 200_000, 1_048_576, 64,
    )
    .expect_err("configured definition limits cannot exceed the hard maximum");
    assert_eq!(above_hard_maximum.kind(), ProtocolErrorKind::ResourceLimit);
}

fn assert_diagnostics_identify_tag_path(diagnostics: &str) {
    let path = vec![tag(EXTENSION_BODY_TAG), tag(PAYLOAD_TAG)];
    let expected_path = format!("{path:?}");
    let expected_leaf = format!("{:?}", tag(PAYLOAD_TAG));

    assert!(
        diagnostics.contains(&expected_path) && diagnostics.contains(&expected_leaf),
        "validation diagnostics must identify the complete ordered Tag path"
    );
}

#[test]
fn invalid_schema_diagnostics_identify_the_path_without_payload_content() {
    let integer_sentinel = PAYLOAD_INTEGER_SENTINEL.to_string();
    let definition = extension_definition();
    let error = extension::validate(
        definition,
        extension_value(Value::text_string(PAYLOAD_TEXT_SENTINEL.to_owned())),
        &CodecLimits::defaults(),
    )
    .expect_err("a Text String does not satisfy the registered Integer rule");

    assert_eq!(error.kind(), ProtocolErrorKind::InvalidSchema);
    let diagnostics = all_diagnostics(&error);
    assert_diagnostics_identify_tag_path(&diagnostics);
    assert_text_and_byte_payloads_redacted(
        &diagnostics,
        &[
            DISCRIMINATOR_TEXT_SENTINEL,
            PAYLOAD_TEXT_SENTINEL,
            &integer_sentinel,
        ],
        &[],
    );

    let definition = extension_definition();
    let error = extension::validate(
        definition,
        extension_value(Value::byte_string(PAYLOAD_BYTES_SENTINEL.to_vec())),
        &CodecLimits::defaults(),
    )
    .expect_err("a Byte String does not satisfy the registered Integer rule");

    assert_eq!(error.kind(), ProtocolErrorKind::InvalidSchema);
    let diagnostics = all_diagnostics(&error);
    assert_diagnostics_identify_tag_path(&diagnostics);
    assert_text_and_byte_payloads_redacted(
        &diagnostics,
        &[DISCRIMINATOR_TEXT_SENTINEL, &integer_sentinel],
        &[PAYLOAD_BYTES_SENTINEL],
    );
}

#[test]
fn configured_ttlv_limit_returns_resource_limit_without_a_partial_value() {
    let integer_sentinel = PAYLOAD_INTEGER_SENTINEL.to_string();
    let result = extension::validate(
        extension_definition(),
        extension_value(Value::integer(PAYLOAD_INTEGER_SENTINEL)),
        &CodecLimits::new(1_024, 64, 3)
            .expect("the configured element limit is within the TTLV hard maximum"),
    );
    let error = result.expect_err("the four-item tree exceeds the configured element limit");

    assert_eq!(error.kind(), ProtocolErrorKind::ResourceLimit);
    assert_text_and_byte_payloads_redacted(
        &all_diagnostics(&error),
        &[DISCRIMINATOR_TEXT_SENTINEL, &integer_sentinel],
        &[],
    );
}
