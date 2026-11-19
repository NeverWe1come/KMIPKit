//! Shared TTLV item construction for operation protocol tests.

use kmipkit_ttlv::{Item, RawTag, Value};

/// Builds a source-derived test item from its allocated numeric tag and value.
pub(crate) fn item(raw_tag: u32, value: Value) -> Item {
    let tag = RawTag::new(raw_tag)
        .expect("the source-derived tag fits the TTLV tag width")
        .try_checked()
        .expect("the source-derived tag is allocated");
    Item::new(tag, value).expect("the source-derived tag and value form a TTLV item")
}
