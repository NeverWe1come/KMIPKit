//! Ordered generic KMIP Structure values.

use crate::{Item, ModelError, ValueView};

const MAX_STRUCTURE_DEPTH: usize = 64;

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
    /// Returns an error if appending would exceed the local nesting-depth limit.
    pub fn try_push(&mut self, item: Item) -> Result<(), ModelError> {
        if item_structure_depth(&item) >= MAX_STRUCTURE_DEPTH {
            return Err(ModelError::StructureDepthExceeded);
        }

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

fn item_structure_depth(item: &Item) -> usize {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => structure_depth(&structure),
        _ => 0,
    })
}

fn structure_depth(structure: &StructureView<'_>) -> usize {
    structure
        .children()
        .iter()
        .map(item_structure_depth)
        .fold(0, usize::max)
        .saturating_add(1)
}
