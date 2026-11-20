#![no_main]
#![forbid(unsafe_code)]

use std::sync::OnceLock;

#[path = "support/ttlv_clone.rs"]
mod ttlv_clone;

use kmipkit_protocol::extension::{
    self, ExtensionDefinition, ExtensionOrderConstraint, ExtensionSchema,
};
use kmipkit_ttlv::codec::{CodecLimits, decode_with_limits};
use kmipkit_ttlv::{ItemType, RawTag, Tag, Value, ValueView};
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 4 * 1024;
const MAX_STRUCTURE_DEPTH: usize = 64;
const MAX_ITEMS: usize = 512;
const NESTED_STRUCTURE_TAG: u32 = 0x0054_0010;
const DISCRIMINATOR_TAG: u32 = 0x0054_0011;
const INTEGER_TAG: u32 = 0x0054_0012;
const ENUMERATION_TAG: u32 = 0x0054_0013;
static DEFINITION: OnceLock<ExtensionDefinition> = OnceLock::new();

fn tag(raw: u32) -> Option<Tag> {
    RawTag::new(raw).ok()?.try_checked().ok()
}

fn schema() -> Option<ExtensionSchema> {
    let discriminator = extension::required(
        tag(DISCRIMINATOR_TAG)?,
        extension::scalar(ItemType::TextString).ok()?,
    )
    .ok()?;
    let integer = extension::optional(
        tag(INTEGER_TAG)?,
        extension::with_signed_range(
            extension::scalar(ItemType::Integer).ok()?,
            0,
            i64::from(i32::MAX),
        )
        .ok()?,
    )
    .ok()?;
    let mut enumeration_schema = extension::scalar(ItemType::Enumeration).ok()?;
    for value in [1, 2, 5, u32::MAX] {
        enumeration_schema = extension::with_allowed_enumeration(enumeration_schema, value).ok()?;
    }
    let enumeration = extension::repeated(tag(ENUMERATION_TAG)?, enumeration_schema).ok()?;
    let order: Vec<ExtensionOrderConstraint> =
        vec![extension::extension_order_constraint(tag(INTEGER_TAG)?, tag(ENUMERATION_TAG)?).ok()?];
    let nested =
        extension::structure(vec![discriminator, integer, enumeration], order, true).ok()?;
    let child = extension::required(tag(NESTED_STRUCTURE_TAG)?, nested).ok()?;
    extension::structure(vec![child], Vec::new(), false).ok()
}

fn definition() -> Option<ExtensionDefinition> {
    let identity = extension::extension_identity("fuzz.vendor", "bounded-schema", "1").ok()?;
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0").ok()?;
    let path = extension::with_child_tag(
        extension::ttlv_path(tag(NESTED_STRUCTURE_TAG)?).ok()?,
        tag(DISCRIMINATOR_TAG)?,
    )
    .ok()?;
    let discriminator =
        extension::discriminator(path, Value::text_string("fuzz-v1".to_owned())).ok()?;
    extension::extension_definition(identity, compatibility, discriminator, schema()?).ok()
}

fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }

    let definition = DEFINITION.get_or_init(|| {
        definition().expect("the fixed fuzz schema and registry metadata are valid")
    });
    let Ok(limits) = CodecLimits::new(MAX_INPUT_BYTES, MAX_STRUCTURE_DEPTH, MAX_ITEMS) else {
        return;
    };
    let Ok(decoded) = decode_with_limits(data, &limits) else {
        return;
    };
    let Some(payload) = decoded.with_value(|value| match value {
        ValueView::Structure(structure) => ttlv_clone::clone_structure(&structure),
        _ => None,
    }) else {
        return;
    };

    // Schema failures are expected. The target asserts robustness: arbitrary,
    // bounded TTLV input must not panic, over-allocate, or produce partial state.
    let _ = extension::validate(definition, payload, &limits);
});
