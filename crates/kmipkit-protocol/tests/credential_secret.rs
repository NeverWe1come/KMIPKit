//! Derived KMIPKIT-0008 secret owner contracts (FR-010, SC-003).

use kmipkit_protocol::{SecretBytes, SecretText};
use kmipkit_ttlv::{Item, RawTag, Tag, ValueView};

const TEXT_SENTINEL: &str = "KMIPKIT_SECRET_TEXT_SENTINEL_736563726574";
const BYTES_SENTINEL: &[u8] = b"KMIPKIT_SECRET_BYTES_SENTINEL_736563726574";

fn tag() -> Tag {
    RawTag::new(0x0054_0010)
        .expect("test tag is within the KMIP tag width")
        .try_checked()
        .expect("the test tag is allocated by the KMIP catalog")
}

#[test]
fn secret_text_is_redacted_and_moves_into_a_ttlv_owner() {
    let secret = SecretText::new(TEXT_SENTINEL.to_owned());
    let diagnostics = format!("{secret:?}; {secret}");
    assert!(!diagnostics.contains(TEXT_SENTINEL));
    assert!(secret.with_str(|value| value == TEXT_SENTINEL));

    let value = secret.into_ttlv_value();
    assert!(!format!("{value:?}").contains(TEXT_SENTINEL));
    let item = Item::new(tag(), value).expect("a checked tag and value form an Item");
    assert!(item.with_value(|value| match value {
        ValueView::TextString(text) => text == TEXT_SENTINEL,
        _ => false,
    }));
}

#[test]
fn secret_bytes_are_redacted_and_move_into_a_ttlv_owner() {
    let secret = SecretBytes::new(BYTES_SENTINEL.to_vec());
    let diagnostics = format!("{secret:?}; {secret}");
    assert!(!diagnostics.contains("KMIPKIT_SECRET_BYTES_SENTINEL"));
    assert!(secret.with_bytes(|value| value == BYTES_SENTINEL));

    let value = secret.into_ttlv_value();
    assert!(!format!("{value:?}").contains("KMIPKIT_SECRET_BYTES_SENTINEL"));
    let item = Item::new(tag(), value).expect("a checked tag and value form an Item");
    assert!(item.with_value(|value| match value {
        ValueView::ByteString(bytes) => bytes == BYTES_SENTINEL,
        _ => false,
    }));
}
