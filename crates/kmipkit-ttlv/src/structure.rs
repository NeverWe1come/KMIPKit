//! Ordered generic KMIP Structure values.

use crate::{Item, ModelError};

/// An ordered collection of child items.
///
/// Children retain insertion order and repeated tags. This type does not
/// validate operation-specific field ordering or cardinality.
pub struct Structure {
    children: Vec<Item>,
}

/// A borrowed view of an ordered Structure.
pub struct StructureView<'a> {
    children: &'a [Item],
}

impl Structure {
    /// Creates an empty Structure.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Appends an item to the end of this Structure.
    ///
    /// # Errors
    ///
    /// This signature reserves a local construction error for Structure
    /// constraints. Nesting depth enforcement is added in a later task, so this
    /// implementation currently accepts each item.
    pub fn try_push(&mut self, item: Item) -> Result<(), ModelError> {
        self.children.push(item);
        Ok(())
    }

    pub(crate) fn as_view(&self) -> StructureView<'_> {
        StructureView {
            children: &self.children,
        }
    }
}

impl Default for Structure {
    fn default() -> Self {
        Self::new()
    }
}

impl StructureView<'_> {
    /// Returns the child items in their original insertion order.
    #[must_use]
    pub const fn children(&self) -> &[Item] {
        self.children
    }
}
