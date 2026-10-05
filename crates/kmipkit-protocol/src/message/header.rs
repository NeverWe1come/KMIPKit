//! Read-only typed views over KMIP request and response headers.

use kmipkit_ttlv::{StructureView, ValueView};

use super::validation::{ValidatedRequestHeader, ValidatedResponseHeader};
use super::{RequestMessage, ResponseMessage};

const ASYNCHRONOUS_INDICATOR: u32 = 0x0042_0007;
const ATTESTATION_CAPABLE_INDICATOR: u32 = 0x0042_00D3;
const ATTESTATION_TYPE: u32 = 0x0042_00C7;
const AUTHENTICATION: u32 = 0x0042_000C;
const BATCH_ERROR_CONTINUATION_OPTION: u32 = 0x0042_000E;
const BATCH_ORDER_OPTION: u32 = 0x0042_0010;
const CLIENT_CORRELATION_VALUE: u32 = 0x0042_0105;
const MAXIMUM_RESPONSE_SIZE: u32 = 0x0042_0050;
const NONCE: u32 = 0x0042_00C8;
const SERVER_CORRELATION_VALUE: u32 = 0x0042_0106;
const SERVER_HASHED_PASSWORD: u32 = 0x0042_0155;
const TIME_STAMP: u32 = 0x0042_0092;

/// Read-only typed access to a validated KMIP Request Header.
#[derive(Clone, Copy)]
pub struct RequestHeaderView<'a> {
    message: &'a RequestMessage,
}

impl<'a> RequestHeaderView<'a> {
    pub(super) const fn new(message: &'a RequestMessage) -> Self {
        Self { message }
    }

    fn validated(self) -> ValidatedRequestHeader {
        self.message.validated_header()
    }

    fn with_header<R>(self, callback: impl for<'b> FnOnce(StructureView<'b>) -> R) -> Option<R> {
        self.message
            .with_root_item_structure(self.message.header_index(), callback)
    }

    /// Returns the raw signed Protocol Version Major/Minor pair.
    #[must_use]
    pub fn protocol_version(self) -> super::ProtocolVersion {
        self.validated().protocol_version
    }

    /// Returns the validated positive Batch Count.
    #[must_use]
    pub fn batch_count(self) -> i32 {
        self.validated().batch_count
    }

    /// Returns the optional Maximum Response Size Integer.
    #[must_use]
    pub fn maximum_response_size(self) -> Option<i32> {
        self.with_header(|header| integer_field(&header, MAXIMUM_RESPONSE_SIZE))
            .flatten()
    }

    /// Lends the optional Client Correlation Value as callback-scoped text.
    pub fn with_client_correlation_value<R>(
        self,
        callback: impl for<'b> FnOnce(&'b str) -> R,
    ) -> Option<R> {
        self.with_header(|header| text_field(&header, CLIENT_CORRELATION_VALUE, callback))
            .flatten()
    }

    /// Returns the optional raw Asynchronous Indicator Enumeration.
    #[must_use]
    pub fn asynchronous_indicator(self) -> Option<u32> {
        self.with_header(|header| enumeration_field(&header, ASYNCHRONOUS_INDICATOR))
            .flatten()
    }

    /// Returns the OASIS effective default Prohibited (`3`) when absent.
    #[must_use]
    pub fn effective_asynchronous_indicator(self) -> u32 {
        self.asynchronous_indicator().unwrap_or(3)
    }

    /// Returns the optional Attestation Capable Indicator field as represented.
    #[must_use]
    pub fn attestation_capable_indicator(self) -> Option<bool> {
        self.with_header(|header| boolean_field(&header, ATTESTATION_CAPABLE_INDICATOR))
            .flatten()
    }

    /// Returns its effective OASIS default `False` when the field is absent.
    #[must_use]
    pub fn effective_attestation_capable_indicator(self) -> bool {
        self.attestation_capable_indicator().unwrap_or(false)
    }

    /// Returns the raw Attestation Type values in their source order.
    #[must_use]
    pub fn attestation_types(self) -> Vec<u32> {
        self.with_header(|header| repeated_enumerations(&header, ATTESTATION_TYPE))
            .unwrap_or_default()
    }

    /// Lends the optional Authentication Structure for callback-scoped access.
    pub fn with_authentication<R>(
        self,
        callback: impl for<'b> FnOnce(StructureView<'b>) -> R,
    ) -> Option<R> {
        self.with_header(|header| structure_field(&header, AUTHENTICATION, callback))
            .flatten()
    }

    /// Returns the optional raw Batch Error Continuation Enumeration.
    #[must_use]
    pub fn batch_error_continuation_option(self) -> Option<u32> {
        self.with_header(|header| enumeration_field(&header, BATCH_ERROR_CONTINUATION_OPTION))
            .flatten()
    }

    /// Returns the OASIS effective default Stop (`2`) when absent.
    #[must_use]
    pub fn effective_batch_error_continuation_option(self) -> u32 {
        self.batch_error_continuation_option().unwrap_or(2)
    }

    /// Returns the optional Batch Order Option Boolean.
    #[must_use]
    pub fn batch_order_option(self) -> Option<bool> {
        self.with_header(|header| boolean_field(&header, BATCH_ORDER_OPTION))
            .flatten()
    }

    /// Returns the OASIS effective default `True` when the field is absent.
    #[must_use]
    pub fn effective_batch_order_option(self) -> bool {
        self.batch_order_option().unwrap_or(true)
    }

    /// Returns the optional Request Header Date-Time value unchanged.
    #[must_use]
    pub fn time_stamp(self) -> Option<i64> {
        self.with_header(|header| date_time_field(&header, TIME_STAMP))
            .flatten()
    }
}

/// Read-only typed access to a validated KMIP Response Header.
#[derive(Clone, Copy)]
pub struct ResponseHeaderView<'a> {
    message: &'a ResponseMessage,
}

impl<'a> ResponseHeaderView<'a> {
    pub(super) const fn new(message: &'a ResponseMessage) -> Self {
        Self { message }
    }

    fn validated(self) -> ValidatedResponseHeader {
        self.message.validated_header()
    }

    fn with_header<R>(self, callback: impl for<'b> FnOnce(StructureView<'b>) -> R) -> Option<R> {
        self.message
            .with_root_item_structure(self.message.header_index(), callback)
    }

    /// Returns the raw signed Protocol Version Major/Minor pair.
    #[must_use]
    pub fn protocol_version(self) -> super::ProtocolVersion {
        self.validated().protocol_version
    }

    /// Returns the required Response Header Date-Time unchanged.
    #[must_use]
    pub fn time_stamp(self) -> i64 {
        self.validated().time_stamp
    }

    /// Returns the validated positive Batch Count.
    #[must_use]
    pub fn batch_count(self) -> i32 {
        self.validated().batch_count
    }

    /// Lends the optional Nonce Structure for callback-scoped access.
    pub fn with_nonce<R>(self, callback: impl for<'b> FnOnce(StructureView<'b>) -> R) -> Option<R> {
        self.with_header(|header| structure_field(&header, NONCE, callback))
            .flatten()
    }

    /// Lends the optional Server Hashed Password bytes for callback-scoped access.
    pub fn with_server_hashed_password<R>(
        self,
        callback: impl for<'b> FnOnce(&'b [u8]) -> R,
    ) -> Option<R> {
        self.with_header(|header| byte_string_field(&header, SERVER_HASHED_PASSWORD, callback))
            .flatten()
    }

    /// Returns raw Attestation Type values in their source order.
    #[must_use]
    pub fn attestation_types(self) -> Vec<u32> {
        self.with_header(|header| repeated_enumerations(&header, ATTESTATION_TYPE))
            .unwrap_or_default()
    }

    /// Lends the optional Client Correlation Value as callback-scoped text.
    pub fn with_client_correlation_value<R>(
        self,
        callback: impl for<'b> FnOnce(&'b str) -> R,
    ) -> Option<R> {
        self.with_header(|header| text_field(&header, CLIENT_CORRELATION_VALUE, callback))
            .flatten()
    }

    /// Lends the optional Server Correlation Value as callback-scoped text.
    pub fn with_server_correlation_value<R>(
        self,
        callback: impl for<'b> FnOnce(&'b str) -> R,
    ) -> Option<R> {
        self.with_header(|header| text_field(&header, SERVER_CORRELATION_VALUE, callback))
            .flatten()
    }
}

fn integer_field(view: &StructureView<'_>, tag: u32) -> Option<i32> {
    field(view, tag, |value| match value {
        ValueView::Integer(raw) => Some(*raw),
        _ => None,
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

fn date_time_field(view: &StructureView<'_>, tag: u32) -> Option<i64> {
    field(view, tag, |value| match value {
        ValueView::DateTime(raw) => Some(*raw),
        _ => None,
    })
}

fn repeated_enumerations(view: &StructureView<'_>, tag: u32) -> Vec<u32> {
    view.children()
        .iter()
        .filter(|child| child.tag().raw() == tag)
        .filter_map(|child| {
            child.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        })
        .collect()
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
