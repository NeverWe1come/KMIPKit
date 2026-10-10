//! Typed KMIP items pairing an allocation-checked tag with an opaque value.

use std::fmt::{self, Debug};

use crate::{ItemType, ModelError, Tag, Value, ValueView};

/// A typed KMIP item with an allocation-checked tag.
pub struct Item {
    tag: Tag,
    value: Value,
}

impl Debug for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Item")
            .field("tag", &self.tag)
            .field("item_type", &self.item_type())
            .finish_non_exhaustive()
    }
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

    /// Creates an independently owned, zeroizing copy of this item.
    ///
    /// The copy preserves this item's tag and recursively retains the source
    /// value's order, repeated tags, and unknown allocated values. Secret
    /// payloads are redacted from formatting and zeroized when the copied
    /// value is dropped.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::StructureDepthExceeded`] if copying the nested
    /// value would exceed the local Structure nesting-depth limit.
    pub fn try_clone(&self) -> Result<Self, ModelError> {
        let value = crate::try_clone_value(&self.value)?;
        Self::new(self.tag, value)
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
