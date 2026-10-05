//! Validated, ordered KMIP Request and Response Messages.

mod batch;
mod header;
mod validation;
mod version;

pub use batch::{MessageExtensionView, RequestBatchItemView, ResponseBatchItemView};
pub use header::{RequestHeaderView, ResponseHeaderView};
pub use validation::{MessageValidationError, MessageValidationErrorKind};
pub use version::ProtocolVersion;

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use self::validation::{
    ValidatedRequestHeader, ValidatedRequestMessage, ValidatedResponseHeader,
    ValidatedResponseMessage, validate_request_message, validate_response_message,
};

/// A validated Request Message that retains its original ordered TTLV tree.
///
/// Construction checks the KMIP message/header/batch structure and keeps the
/// supplied [`Structure`] without cloning or normalizing its payloads.
pub struct RequestMessage {
    tree: Structure,
    header_index: usize,
    batch_indices: Vec<usize>,
    header: ValidatedRequestHeader,
}

impl RequestMessage {
    /// Validates and takes ownership of one generic Request Message Structure.
    ///
    /// Unknown allocation-valid fields remain in their original positions.
    /// Protocol-version send policy and operation-specific payload validation
    /// are outside the message model.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`MessageValidationError`] for invalid message
    /// shape, field order, type, counts, or cross-field result constraints.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, MessageValidationError> {
        let ValidatedRequestMessage {
            header_index,
            batch_indices,
            header,
        } = validate_request_message(&tree.view())?;
        Ok(Self {
            tree,
            header_index,
            batch_indices,
            header,
        })
    }

    /// Returns a read-only typed view of the validated Request Header.
    #[must_use]
    pub const fn header(&self) -> RequestHeaderView<'_> {
        RequestHeaderView::new(self)
    }

    /// Returns request batch items in source order.
    pub fn batch_items(&self) -> impl ExactSizeIterator<Item = RequestBatchItemView<'_>> + '_ {
        self.batch_indices
            .iter()
            .copied()
            .map(|index| RequestBatchItemView::new(self, index))
    }

    /// Lends the original ordered generic TTLV tree for callback-scoped access.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.tree.view())
    }

    /// Returns the exact original generic TTLV tree by ownership.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.tree
    }

    pub(in crate::message) const fn validated_header(&self) -> ValidatedRequestHeader {
        self.header
    }

    pub(in crate::message) const fn header_index(&self) -> usize {
        self.header_index
    }

    pub(in crate::message) fn with_root_item_structure<R>(
        &self,
        index: usize,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.tree.view().children().get(index).and_then(|child| {
            child.with_value(|value| match value {
                ValueView::Structure(view) => Some(callback(view)),
                _ => None,
            })
        })
    }
}

impl fmt::Debug for RequestMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestMessage")
            .field("batch_item_count", &self.batch_indices.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for RequestMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "KMIP Request Message ({} batch items)",
            self.batch_indices.len()
        )
    }
}

/// A validated Response Message that retains its original ordered TTLV tree.
pub struct ResponseMessage {
    tree: Structure,
    header_index: usize,
    batch_indices: Vec<usize>,
    header: ValidatedResponseHeader,
}

impl ResponseMessage {
    /// Validates and takes ownership of one generic Response Message Structure.
    ///
    /// Unknown allocation-valid fields remain in their original positions.
    /// Pairing this response with a request and applying client version policy
    /// belong to client execution.
    ///
    /// # Errors
    ///
    /// Returns a payload-free [`MessageValidationError`] for invalid message
    /// shape, field order, type, counts, or cross-field result constraints.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, MessageValidationError> {
        let ValidatedResponseMessage {
            header_index,
            batch_indices,
            header,
        } = validate_response_message(&tree.view())?;
        Ok(Self {
            tree,
            header_index,
            batch_indices,
            header,
        })
    }

    /// Returns a read-only typed view of the validated Response Header.
    #[must_use]
    pub const fn header(&self) -> ResponseHeaderView<'_> {
        ResponseHeaderView::new(self)
    }

    /// Returns response batch items in source order.
    pub fn batch_items(&self) -> impl ExactSizeIterator<Item = ResponseBatchItemView<'_>> + '_ {
        self.batch_indices
            .iter()
            .copied()
            .map(|index| ResponseBatchItemView::new(self, index))
    }

    /// Lends the original ordered generic TTLV tree for callback-scoped access.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.tree.view())
    }

    /// Returns the exact original generic TTLV tree by ownership.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.tree
    }

    pub(in crate::message) const fn validated_header(&self) -> ValidatedResponseHeader {
        self.header
    }

    pub(in crate::message) const fn header_index(&self) -> usize {
        self.header_index
    }

    pub(in crate::message) fn with_root_item_structure<R>(
        &self,
        index: usize,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.tree.view().children().get(index).and_then(|child| {
            child.with_value(|value| match value {
                ValueView::Structure(view) => Some(callback(view)),
                _ => None,
            })
        })
    }
}

impl fmt::Debug for ResponseMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseMessage")
            .field("batch_item_count", &self.batch_indices.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Display for ResponseMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "KMIP Response Message ({} batch items)",
            self.batch_indices.len()
        )
    }
}
