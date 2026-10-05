//! Read-only typed views over KMIP batch items and opaque Message Extensions.

use kmipkit_ttlv::{StructureView, ValueView};

use crate::{ResultReason, ResultStatus};

use super::{RequestMessage, ResponseMessage};

const ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0042_0006;
const EPHEMERAL: u32 = 0x0042_0154;
const MESSAGE_EXTENSION: u32 = 0x0042_0051;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const RESPONSE_PAYLOAD: u32 = 0x0042_007C;
const RESULT_MESSAGE: u32 = 0x0042_007D;
const RESULT_REASON: u32 = 0x0042_007E;
const RESULT_STATUS: u32 = 0x0042_007F;
const UNIQUE_BATCH_ITEM_ID: u32 = 0x0042_0093;
const CRITICALITY_INDICATOR: u32 = 0x0042_0026;
const VENDOR_EXTENSION: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION: u32 = 0x0042_009D;

/// Read-only access to a validated Request Batch Item.
#[derive(Clone, Copy)]
pub struct RequestBatchItemView<'a> {
    message: &'a RequestMessage,
    root_index: usize,
}

impl<'a> RequestBatchItemView<'a> {
    pub(super) const fn new(message: &'a RequestMessage, root_index: usize) -> Self {
        Self {
            message,
            root_index,
        }
    }

    fn with_batch<R>(self, callback: impl for<'b> FnOnce(StructureView<'b>) -> R) -> Option<R> {
        self.message
            .with_root_item_structure(self.root_index, callback)
    }

    /// Returns the raw Operation Enumeration when it can be copied.
    #[must_use]
    pub fn operation(self) -> Option<u32> {
        self.with_batch(|batch| enumeration_field(&batch, OPERATION))
            .flatten()
    }

    /// Returns the optional Ephemeral Boolean.
    #[must_use]
    pub fn ephemeral(self) -> Option<bool> {
        self.with_batch(|batch| boolean_field(&batch, EPHEMERAL))
            .flatten()
    }

    /// Lends the optional Unique Batch Item ID bytes to a callback.
    pub fn with_unique_batch_item_id<R>(
        self,
        callback: impl for<'b> FnOnce(&'b [u8]) -> R,
    ) -> Option<R> {
        self.with_batch(|batch| byte_string_field(&batch, UNIQUE_BATCH_ITEM_ID, callback))
            .flatten()
    }

    /// Lends the required Request Payload Structure for callback-scoped access.
    pub fn with_request_payload<R>(
        self,
        callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
    ) -> Option<R> {
        self.with_batch(|batch| structure_field(&batch, REQUEST_PAYLOAD, callback))
            .flatten()
    }

    /// Returns the number of Message Extension Structures in source order.
    #[must_use]
    pub fn message_extension_count(self) -> usize {
        self.with_batch(|batch| {
            batch
                .children()
                .iter()
                .filter(|child| child.tag().raw() == MESSAGE_EXTENSION)
                .count()
        })
        .unwrap_or(0)
    }

    /// Returns the indexed Message Extension view, if present.
    #[must_use]
    pub fn message_extension(self, extension_index: usize) -> Option<MessageExtensionView<'a>> {
        (extension_index < self.message_extension_count()).then_some(MessageExtensionView {
            owner: MessageOwner::Request(self.message),
            batch_index: self.root_index,
            extension_index,
        })
    }
}

/// Read-only access to a validated Response Batch Item.
#[derive(Clone, Copy)]
pub struct ResponseBatchItemView<'a> {
    message: &'a ResponseMessage,
    root_index: usize,
}

impl<'a> ResponseBatchItemView<'a> {
    pub(super) const fn new(message: &'a ResponseMessage, root_index: usize) -> Self {
        Self {
            message,
            root_index,
        }
    }

    fn with_batch<R>(self, callback: impl for<'b> FnOnce(StructureView<'b>) -> R) -> Option<R> {
        self.message
            .with_root_item_structure(self.root_index, callback)
    }

    /// Returns the optional raw Operation Enumeration.
    #[must_use]
    pub fn operation(self) -> Option<u32> {
        self.with_batch(|batch| enumeration_field(&batch, OPERATION))
            .flatten()
    }

    /// Lends the optional echoed Unique Batch Item ID bytes to a callback.
    pub fn with_unique_batch_item_id<R>(
        self,
        callback: impl for<'b> FnOnce(&'b [u8]) -> R,
    ) -> Option<R> {
        self.with_batch(|batch| byte_string_field(&batch, UNIQUE_BATCH_ITEM_ID, callback))
            .flatten()
    }

    /// Returns the exact Result Status value.
    #[must_use]
    pub fn result_status(self) -> Option<ResultStatus> {
        self.with_batch(|batch| {
            enumeration_field(&batch, RESULT_STATUS).map(ResultStatus::from_raw)
        })
        .flatten()
    }

    /// Returns the optional exact Result Reason value.
    #[must_use]
    pub fn result_reason(self) -> Option<ResultReason> {
        self.with_batch(|batch| {
            enumeration_field(&batch, RESULT_REASON).map(ResultReason::from_raw)
        })
        .flatten()
    }

    /// Lends the optional Result Message text for explicit callback-scoped inspection.
    pub fn with_result_message<R>(self, callback: impl for<'b> FnOnce(&'b str) -> R) -> Option<R> {
        self.with_batch(|batch| text_field(&batch, RESULT_MESSAGE, callback))
            .flatten()
    }

    /// Lends the optional Asynchronous Correlation Value bytes to a callback.
    pub fn with_asynchronous_correlation_value<R>(
        self,
        callback: impl for<'b> FnOnce(&'b [u8]) -> R,
    ) -> Option<R> {
        self.with_batch(|batch| byte_string_field(&batch, ASYNCHRONOUS_CORRELATION_VALUE, callback))
            .flatten()
    }

    /// Lends the optional Response Payload Structure to a callback.
    pub fn with_response_payload<R>(
        self,
        callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
    ) -> Option<R> {
        self.with_batch(|batch| structure_field(&batch, RESPONSE_PAYLOAD, callback))
            .flatten()
    }

    /// Returns the number of Message Extension Structures in source order.
    #[must_use]
    pub fn message_extension_count(self) -> usize {
        self.with_batch(|batch| {
            batch
                .children()
                .iter()
                .filter(|child| child.tag().raw() == MESSAGE_EXTENSION)
                .count()
        })
        .unwrap_or(0)
    }

    /// Returns the indexed Message Extension view, if present.
    #[must_use]
    pub fn message_extension(self, extension_index: usize) -> Option<MessageExtensionView<'a>> {
        (extension_index < self.message_extension_count()).then_some(MessageExtensionView {
            owner: MessageOwner::Response(self.message),
            batch_index: self.root_index,
            extension_index,
        })
    }
}

/// Read-only, opaque access to one structurally valid Message Extension.
#[derive(Clone, Copy)]
pub struct MessageExtensionView<'a> {
    owner: MessageOwner<'a>,
    batch_index: usize,
    extension_index: usize,
}

#[derive(Clone, Copy)]
enum MessageOwner<'a> {
    Request(&'a RequestMessage),
    Response(&'a ResponseMessage),
}

impl MessageExtensionView<'_> {
    fn with_extension<R>(self, callback: impl for<'b> FnOnce(StructureView<'b>) -> R) -> Option<R> {
        match self.owner {
            MessageOwner::Request(message) => message
                .with_root_item_structure(self.batch_index, |batch| {
                    extension_structure(&batch, self.extension_index, callback)
                }),
            MessageOwner::Response(message) => message
                .with_root_item_structure(self.batch_index, |batch| {
                    extension_structure(&batch, self.extension_index, callback)
                }),
        }
        .flatten()
    }

    /// Lends the complete extension tree for callback-scoped generic access.
    pub fn with_ttlv<R>(self, callback: impl for<'b> FnOnce(StructureView<'b>) -> R) -> Option<R> {
        self.with_extension(callback)
    }

    /// Lends the required Vendor Identification text for explicit inspection.
    pub fn with_vendor_identification<R>(
        self,
        callback: impl for<'b> FnOnce(&'b str) -> R,
    ) -> Option<R> {
        self.with_extension(|extension| text_field(&extension, VENDOR_IDENTIFICATION, callback))
            .flatten()
    }

    /// Returns the required Criticality Indicator Boolean.
    #[must_use]
    pub fn criticality_indicator(self) -> Option<bool> {
        self.with_extension(|extension| boolean_field(&extension, CRITICALITY_INDICATOR))
            .flatten()
    }

    /// Lends the opaque Vendor Extension Structure to a callback.
    pub fn with_vendor_extension<R>(
        self,
        callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
    ) -> Option<R> {
        self.with_extension(|extension| structure_field(&extension, VENDOR_EXTENSION, callback))
            .flatten()
    }
}

fn extension_structure<R>(
    batch: &StructureView<'_>,
    extension_index: usize,
    callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
) -> Option<R> {
    batch
        .children()
        .iter()
        .filter(|child| child.tag().raw() == MESSAGE_EXTENSION)
        .nth(extension_index)
        .and_then(|child| {
            child.with_value(|value| match value {
                ValueView::Structure(extension) => Some(callback(extension)),
                _ => None,
            })
        })
}

fn enumeration_field(view: &StructureView<'_>, tag: u32) -> Option<u32> {
    field(view, tag, |value| match value {
        ValueView::Enumeration(raw) => Some(*raw),
        _ => None,
    })
}

fn boolean_field(view: &StructureView<'_>, tag: u32) -> Option<bool> {
    field(view, tag, |value| match value {
        ValueView::Boolean(raw) => Some(*raw),
        _ => None,
    })
}

fn field<R>(
    view: &StructureView<'_>,
    tag: u32,
    callback: impl for<'b> FnOnce(ValueView<'b>) -> Option<R>,
) -> Option<R> {
    view.children()
        .iter()
        .find(|child| child.tag().raw() == tag)
        .and_then(|child| child.with_value(callback))
}

fn byte_string_field<R>(
    view: &StructureView<'_>,
    tag: u32,
    callback: impl for<'b> FnOnce(&'b [u8]) -> R,
) -> Option<R> {
    view.children()
        .iter()
        .find(|child| child.tag().raw() == tag)
        .and_then(|child| {
            child.with_value(|value| match value {
                ValueView::ByteString(bytes) => Some(callback(bytes)),
                _ => None,
            })
        })
}

fn text_field<R>(
    view: &StructureView<'_>,
    tag: u32,
    callback: impl for<'b> FnOnce(&'b str) -> R,
) -> Option<R> {
    view.children()
        .iter()
        .find(|child| child.tag().raw() == tag)
        .and_then(|child| {
            child.with_value(|value| match value {
                ValueView::TextString(text) => Some(callback(text)),
                _ => None,
            })
        })
}

fn structure_field<R>(
    view: &StructureView<'_>,
    tag: u32,
    callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
) -> Option<R> {
    view.children()
        .iter()
        .find(|child| child.tag().raw() == tag)
        .and_then(|child| {
            child.with_value(|value| match value {
                ValueView::Structure(structure) => Some(callback(structure)),
                _ => None,
            })
        })
}
