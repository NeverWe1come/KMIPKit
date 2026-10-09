//! Project-derived secret-redaction regressions for attribute payloads.
//!
//! OASIS KMIP v2.1 §§5.6–5.7, Tables 162–163, define the Current/New Attribute
//! wrappers around one direct attribute Item. Redaction is a `KMIPKit` security
//! requirement from KMIPKIT-0016-FR-013 and AGENTS.md §8, not an OASIS
//! conformance requirement; these tests do not claim official OASIS coverage.

use std::error::Error;
use std::fmt::Write as _;

use kmipkit_protocol::{AttributeSet, AttributeSetError, CurrentAttribute, NewAttribute};
use kmipkit_ttlv::{Item, RawTag, Tag, Value, ValueView};

const DIRECT_ATTRIBUTE_TAG: u32 = 0x0054_1234;
const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;
const KNOWN_ENUM_ATTRIBUTE_TAG: u32 = 0x0042_0028;

const DIRECT_SENTINEL: &str = "KMIPKIT_ATTRIBUTE_DIRECT_SECRET_7f20a8";
const CURRENT_SENTINEL: &str = "KMIPKIT_ATTRIBUTE_CURRENT_SECRET_b4196e";
const NEW_SENTINEL: &str = "KMIPKIT_ATTRIBUTE_NEW_SECRET_c251d3";
const INVALID_DIRECT_SENTINEL: &str = "KMIPKIT_ATTRIBUTE_INVALID_DIRECT_SECRET_0a98ef";
const WRAPPED_ERROR_SENTINEL: &str = "KMIPKIT_ATTRIBUTE_WRAPPED_ERROR_SECRET_d6431b";

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("attribute fixture tag fits the KMIP Tag width")
        .try_checked()
        .expect("attribute fixture tag is assigned or in an accepted extension allocation")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("checked fixture tag and value form a TTLV Item")
}

fn diagnostics(error: &dyn Error) -> String {
    let mut output = format!("debug={error:?}; display={error}");
    let mut source = error.source();
    while let Some(cause) = source {
        let _ = write!(output, "; source-debug={cause:?}; source-display={cause}");
        source = cause.source();
    }
    output
}

#[test]
fn direct_attribute_value_remains_typed_while_item_and_set_debug_redact_it() {
    let direct = item(
        DIRECT_ATTRIBUTE_TAG,
        Value::text_string(DIRECT_SENTINEL.to_owned()),
    );
    let attributes = AttributeSet::try_new([direct])
        .expect("an extension-tagged direct attribute remains a generic typed Item");
    let retained = attributes.as_items()[0].with_value(|value| match value {
        ValueView::TextString(text) => Some(text.to_owned()),
        _ => None,
    });

    assert_eq!(retained.as_deref(), Some(DIRECT_SENTINEL));

    let diagnostic = format!("item={:?}; set={:?}", attributes.as_items()[0], attributes);
    assert!(!diagnostic.contains(DIRECT_SENTINEL));
}

#[test]
fn current_and_new_attribute_values_remain_typed_while_wrapper_debug_redacts_them() {
    // KMIP v2.1 §§5.6–5.7, Tables 162–163, define each wrapper as one direct
    // attribute Item. The exact values remain inspectable through the typed API.
    let current = CurrentAttribute::new(item(
        DIRECT_ATTRIBUTE_TAG,
        Value::text_string(CURRENT_SENTINEL.to_owned()),
    ));
    let new = NewAttribute::new(item(
        DIRECT_ATTRIBUTE_TAG,
        Value::text_string(NEW_SENTINEL.to_owned()),
    ));

    let current_value = current.item().with_value(|value| match value {
        ValueView::TextString(text) => Some(text.to_owned()),
        _ => None,
    });
    let new_value = new.item().with_value(|value| match value {
        ValueView::TextString(text) => Some(text.to_owned()),
        _ => None,
    });
    assert_eq!(current_value.as_deref(), Some(CURRENT_SENTINEL));
    assert_eq!(new_value.as_deref(), Some(NEW_SENTINEL));

    let diagnostic = format!(
        "current={current:?}; current-item={:?}; new={new:?}; new-item={:?}",
        current.item(),
        new.item()
    );
    assert!(!diagnostic.contains(CURRENT_SENTINEL));
    assert!(!diagnostic.contains(NEW_SENTINEL));
}

#[test]
fn direct_and_wrapped_attribute_validation_errors_redact_values_in_diagnostics() {
    // FR-013 and AGENTS.md §8 require diagnostics to omit secret-bearing
    // attribute values. The OASIS structure sources are §4 and §4.60 Table 150.
    let invalid_direct = item(
        KNOWN_ENUM_ATTRIBUTE_TAG,
        Value::text_string(INVALID_DIRECT_SENTINEL.to_owned()),
    );
    let direct_error = AttributeSet::try_new([invalid_direct])
        .expect_err("Cryptographic Algorithm requires Enumeration, not Text String");
    assert_eq!(direct_error, AttributeSetError::AttributeTtlvTypeMismatch);
    let direct_diagnostic = diagnostics(&direct_error);
    assert!(!direct_diagnostic.contains(INVALID_DIRECT_SENTINEL));

    // Table 150 puts Attribute Name before Attribute Value. This intentionally
    // malformed Vendor Attribute carries a synthetic secret in its nested
    // Attribute Value while violating that source-defined order.
    let wrapped = item(
        VENDOR_ATTRIBUTE_TAG,
        Value::structure(
            [
                item(
                    VENDOR_IDENTIFICATION_TAG,
                    Value::text_string("KMIPKit.TestVendor".to_owned()),
                ),
                item(
                    ATTRIBUTE_VALUE_TAG,
                    Value::text_string(WRAPPED_ERROR_SENTINEL.to_owned()),
                ),
                item(
                    ATTRIBUTE_NAME_TAG,
                    Value::text_string("Opaque.Attribute".to_owned()),
                ),
            ]
            .into_iter()
            .fold(kmipkit_ttlv::Structure::new(), |mut structure, child| {
                structure
                    .try_push(child)
                    .expect("small malformed Vendor Attribute stays within model limits");
                structure
            }),
        ),
    );
    let wrapped_error = AttributeSet::try_new([wrapped])
        .expect_err("Vendor Attribute fields out of Table 150 order are rejected");
    assert_eq!(wrapped_error, AttributeSetError::VendorAttributeFieldOrder);
    let wrapped_diagnostic = diagnostics(&wrapped_error);
    assert!(!wrapped_diagnostic.contains(WRAPPED_ERROR_SENTINEL));
}
