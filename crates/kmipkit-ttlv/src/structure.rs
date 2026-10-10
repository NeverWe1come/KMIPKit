//! Ordered generic KMIP Structure values.

use std::fmt::{self, Debug};

use crate::{Item, ModelError, ValueView};

// This is a local KMIPKit model policy, not a protocol nesting constraint.
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

impl Debug for Structure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_structure_metadata(formatter, "Structure", self.children.len())
    }
}

impl Debug for StructureView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_structure_metadata(formatter, "StructureView", self.children.len())
    }
}

fn debug_structure_metadata(
    formatter: &mut fmt::Formatter<'_>,
    name: &str,
    child_count: usize,
) -> fmt::Result {
    formatter
        .debug_struct(name)
        .field("child_count", &child_count)
        .finish()
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
    /// Returns [`ModelError::StructureDepthExceeded`] if appending would create
    /// more than 64 nested Structures. This is a local `KMIPKit` model policy,
    /// not a nesting constraint imposed by OASIS KMIP 2.1. The error contains
    /// only its category; it does not retain the rejected item, tags, child
    /// count, or payload.
    pub fn try_push(&mut self, item: Item) -> Result<(), ModelError> {
        if item_structure_depth(&item) >= MAX_STRUCTURE_DEPTH {
            return Err(ModelError::StructureDepthExceeded);
        }

        self.children.push(item);
        Ok(())
    }

    /// Borrows this Structure's child items in their original insertion order.
    ///
    /// The returned view cannot outlive this borrow and exposes no mutable
    /// access or ownership transfer for the child items or their payloads.
    #[must_use]
    pub fn view(&self) -> StructureView<'_> {
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

impl Structure {
    pub(crate) fn zeroize_payloads(&mut self) {
        // Preserve the live tree shape while clearing each boxed payload.
        for child in &mut self.children {
            child.zeroize_payloads();
        }
    }
}

impl StructureView<'_> {
    /// Returns the child items in their original insertion order.
    #[must_use]
    pub const fn children(&self) -> &[Item] {
        self.children
    }

    /// Creates a zeroizing owned copy of this ordered generic Structure.
    ///
    /// The copy preserves repeated tags, unknown values, and source order.
    /// Its payloads are zeroized when the owned Structure is dropped.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::StructureDepthExceeded`] if the source exceeds
    /// the local Structure nesting-depth limit.
    pub fn try_clone(&self) -> Result<Structure, ModelError> {
        let mut clone = Structure::new();
        for item in self.children {
            let value = item.with_value(crate::value::clone_value_view)?;
            clone.try_push(Item::new(item.tag(), value)?)?;
        }
        Ok(clone)
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
