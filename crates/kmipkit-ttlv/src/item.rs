//! Typed KMIP items pairing an allocation-checked tag with an opaque value.

use crate::{ItemType, ModelError, Tag, Value, ValueView};

/// A typed KMIP item with an allocation-checked tag.
pub struct Item {
    tag: Tag,
    value: Value,
}

impl Item {
    /// Creates an item from an allocation-checked tag and a value.
    ///
    /// The Item Type is derived from the value representation. This checks no
    /// TTLV framing, Structure schema, or operation-specific semantics.
    ///
    /// # Errors
    ///
    /// This signature reserves a local construction error for model
    /// constraints. A checked tag and an owned value currently satisfy all
    /// construction constraints, so this method returns `Ok`.
    pub fn new(tag: Tag, value: Value) -> Result<Self, ModelError> {
        Ok(Self { tag, value })
    }

    /// Returns the Item Type derived from this item's value representation.
    #[must_use]
    pub fn item_type(&self) -> ItemType {
        self.value.item_type()
    }

    /// Returns this item's allocation-checked tag.
    #[must_use]
    pub const fn tag(&self) -> Tag {
        self.tag
    }

    /// Lends a read-only value view for the duration of `callback`.
    ///
    /// The callback's return type is independent of the view's lifetime, so it
    /// cannot return a reference borrowed from the item.
    pub fn with_value<R>(&self, callback: impl for<'a> FnOnce(ValueView<'a>) -> R) -> R {
        callback(self.value.as_view())
    }

    pub(crate) fn zeroize_payloads(&mut self) {
        self.value.zeroize_payloads();
    }
}
