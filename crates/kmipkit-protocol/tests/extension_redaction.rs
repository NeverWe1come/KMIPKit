//! Redaction guarantees for validated vendor extension values.
//!
//! Traceability: KMIPKIT-0012-FR-011 and KMIP 2.1 §9.13, Table 418.

use std::error::Error;

use kmipkit_protocol::extension;
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

const DISCRIMINATOR_TAG: u32 = 0x42_0001;
const SECRET_TAG: u32 = 0x42_0002;
const DISCRIMINATOR_SENTINEL: &str = "KMIPKIT_EXTENSION_DISCRIMINATOR_SECRET_SENTINEL";
const PAYLOAD_SENTINEL: &str = "KMIPKIT_EXTENSION_PAYLOAD_SECRET_SENTINEL";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("the redaction fixture tag fits the KMIP Tag width")
        .try_checked()
        .expect("the redaction fixture tag is in the vendor allocation")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut structure = Structure::new();
    for child in items {
        structure
            .try_push(child)
            .expect("the redaction fixture remains within the TTLV depth limit");
    }
    structure
}

fn definition() -> extension::ExtensionDefinition {
    let identity = extension::extension_identity("example.vendor", "redaction", "1")
        .expect("the redaction fixture identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the redaction fixture supports this client");
    let path = extension::ttlv_path(tag(DISCRIMINATOR_TAG))
        .expect("the redaction fixture discriminator path is valid");
    let discriminator =
        extension::discriminator(path, Value::text_string(DISCRIMINATOR_SENTINEL.to_owned()))
            .expect("the redaction fixture discriminator is a valid scalar");
    let schema = extension::structure(
        vec![
            extension::required(
                tag(DISCRIMINATOR_TAG),
                extension::scalar(ItemType::TextString)
                    .expect("Text String is a supported schema type"),
            )
            .expect("the discriminator child rule is valid"),
            extension::required(
                tag(SECRET_TAG),
                extension::scalar(ItemType::ByteString)
                    .expect("Byte String is a supported schema type"),
            )
            .expect("the secret child rule is valid"),
        ],
        Vec::new(),
        false,
    )
    .expect("the redaction fixture schema is valid");
    extension::extension_definition(identity, compatibility, discriminator, schema)
        .expect("the redaction fixture definition is valid")
}

fn valid_value() -> Structure {
    structure([
        Item::new(
            tag(DISCRIMINATOR_TAG),
            Value::text_string(DISCRIMINATOR_SENTINEL.to_owned()),
        )
        .expect("the discriminator item is valid"),
        Item::new(
            tag(SECRET_TAG),
            Value::byte_string(PAYLOAD_SENTINEL.as_bytes().to_vec()),
        )
        .expect("the secret item is valid"),
    ])
}

fn diagnostics(error: &dyn Error) -> String {
    let mut text = format!("debug={error:?}; display={error}");
    let mut source = error.source();
    while let Some(cause) = source {
        text.push_str(&format!("; source-debug={cause:?}; source-display={cause}"));
        source = cause.source();
    }
    text
}

#[test]
fn validated_extension_debug_and_display_redact_discriminator_and_payload_secrets() {
    let validated = extension::validate(&definition(), valid_value(), &CodecLimits::defaults())
        .expect("the valid redaction fixture passes its schema");
    let formatted = format!("{validated:?}; {validated}");

    assert!(!formatted.contains(DISCRIMINATOR_SENTINEL));
    assert!(!formatted.contains(PAYLOAD_SENTINEL));
    assert!(formatted.contains("REDACTED"));
}

#[test]
fn schema_error_debug_display_and_source_chain_redact_payload_secrets() {
    let invalid = structure([
        Item::new(
            tag(DISCRIMINATOR_TAG),
            Value::text_string(DISCRIMINATOR_SENTINEL.to_owned()),
        )
        .expect("the discriminator item is valid"),
        Item::new(
            tag(SECRET_TAG),
            Value::text_string(PAYLOAD_SENTINEL.to_owned()),
        )
        .expect("the deliberately invalid secret item is valid TTLV"),
    ]);
    let error = extension::validate(&definition(), invalid, &CodecLimits::defaults())
        .expect_err("Text String does not satisfy the Byte String schema rule");
    let formatted = diagnostics(&error);

    assert!(!formatted.contains(DISCRIMINATOR_SENTINEL));
    assert!(!formatted.contains(PAYLOAD_SENTINEL));
}
