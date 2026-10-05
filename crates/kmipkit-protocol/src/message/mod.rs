//! Validated, ordered KMIP Request and Response Messages.

mod validation;
mod version;

pub use validation::{MessageValidationError, MessageValidationErrorKind};
pub use version::ProtocolVersion;

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView};

use self::validation::{
    ValidatedRequestMessage, ValidatedResponseMessage, validate_request_message,
    validate_response_message,
};

/// A validated Request Message that retains its original ordered TTLV tree.
///
/// Construction checks the KMIP message/header/batch structure and keeps the
/// supplied [`Structure`] without cloning or normalizing its payloads.
pub struct RequestMessage {
    tree: Structure,
    batch_indices: Vec<usize>,
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
        let ValidatedRequestMessage { batch_indices, .. } = validate_request_message(&tree.view())?;
        Ok(Self {
            tree,
            batch_indices,
        })
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
    batch_indices: Vec<usize>,
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
        let ValidatedResponseMessage { batch_indices, .. } =
            validate_response_message(&tree.view())?;
        Ok(Self {
            tree,
            batch_indices,
        })
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
