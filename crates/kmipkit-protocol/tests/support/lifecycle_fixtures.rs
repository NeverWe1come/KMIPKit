//! Test-only generic TTLV fixtures for KMIP 2.1 managed-object lifecycle payloads.
//!
//! Request and successful response shapes follow OASIS KMIP Specification v2.1
//! §6.1.1 Tables 164–165, §6.1.4 Tables 173–174, §6.1.15 Tables 208–209, and
//! §6.1.42 Tables 288–289. The Unique Identifier tag assignment is in §11.56.
//! These are derived fixtures, not official KMIP test vectors.

use std::fmt::Debug;

use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

use crate::UniqueIdentifier;

pub(crate) const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;

pub(crate) fn assert_debug_redacts_identifier(value: &impl Debug, identifier: &str) {
    let formatted = format!("{value:?}");

    assert!(
        !formatted.contains(identifier),
        "Debug output must not expose a Unique Identifier value"
    );
}

/// Builds a minimal request payload with its optional Unique Identifier.
pub(crate) fn request_payload(identifier: Option<UniqueIdentifier>) -> Structure {
    let mut payload = Structure::new();
    if let Some(identifier) = identifier {
        payload
            .try_push(identifier_item(identifier))
            .expect("fixture payload fits model depth limits");
    }
    payload
}

/// Builds a successful response payload with the supplied Unique Identifier.
pub(crate) fn successful_response_payload(identifier: UniqueIdentifier) -> Structure {
    let mut payload = Structure::new();
    payload
        .try_push(identifier_item(identifier))
        .expect("fixture payload fits model depth limits");
    payload
}

/// Builds a malformed success payload without its required Unique Identifier.
pub(crate) fn success_payload_missing_identifier() -> Structure {
    Structure::new()
}

/// Builds a malformed success payload with a non-identifier Item Type.
pub(crate) fn success_payload_wrong_identifier_type() -> Structure {
    let item = Item::new(unique_identifier_tag(), Value::date_time(1))
        .expect("fixture identifier tag and wrong Item Type form a valid Item");
    let mut payload = Structure::new();
    payload
        .try_push(item)
        .expect("fixture payload fits model depth limits");
    payload
}

/// Builds a malformed success payload with two Unique Identifier items.
pub(crate) fn success_payload_duplicate_identifier(identifier: UniqueIdentifier) -> Structure {
    let mut payload = Structure::new();
    for identifier in [identifier.clone(), identifier] {
        payload
            .try_push(identifier_item(identifier))
            .expect("fixture payload fits model depth limits");
    }
    payload
}

fn identifier_item(identifier: UniqueIdentifier) -> Item {
    let value = match identifier {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    };
    Item::new(unique_identifier_tag(), value).expect("fixture identifier has a valid tag and value")
}

fn unique_identifier_tag() -> Tag {
    RawTag::new(UNIQUE_IDENTIFIER_TAG)
        .expect("fixture tag fits TTLV width")
        .try_checked()
        .expect("pinned Unique Identifier tag is allocated")
}
