//! Typed KMIP 2.1 RNG Retrieve request and response models.

use kmipkit_ttlv::{Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    CryptographicOperationError, CryptographicOperationErrorKind, KmipOperationResult,
    ProtocolError, ResponseBatchItemView, SecretBytes,
};

const OPERATION: u32 = 0x0000_0025;
const DATA_LENGTH: u32 = 0x0042_00D5;
const DATA: u32 = 0x0042_00C2;
const SUCCESS: u32 = 0;

/// A KMIP v2.1 RNG Retrieve request from §6.1.49, Table 316.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RngRetrieveRequest {
    data_length: i32,
}

impl RngRetrieveRequest {
    /// Creates a request with the caller-selected Data Length.
    #[must_use]
    pub const fn new(data_length: i32) -> Self {
        Self { data_length }
    }

    /// Returns the caller-selected number of bytes requested from the server.
    #[must_use]
    pub const fn data_length(&self) -> i32 {
        self.data_length
    }

    /// Encodes the required Table 316 Data Length Integer.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the TTLV item cannot be
    /// represented. The request does not infer server policy or enforce an
    /// unstated range for the signed KMIP Integer.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        common::push(&mut payload, DATA_LENGTH, Value::integer(self.data_length))?;
        Ok(payload)
    }
}

/// The typed result of one KMIP v2.1 RNG Retrieve response.
#[derive(Debug)]
pub struct RngRetrieveResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    data: Option<SecretBytes>,
}

impl RngRetrieveResponse {
    /// Converts one completed Table 317 response item.
    ///
    /// The Data bytes are held in a zeroizing secret owner. Unknown response
    /// members remain available through [`Self::with_ttlv`].
    ///
    /// # Errors
    ///
    /// Returns [`RngRetrieveError`] when the operation result or a successful
    /// Table 317 payload is malformed, or when the result is Pending.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, RngRetrieveError> {
        Self::convert(item, Some(false))
    }

    /// Converts a caller-managed Pending Table 318/async result without
    /// issuing a Poll request.
    ///
    /// # Errors
    ///
    /// Returns [`RngRetrieveError`] if the item is not this operation or is
    /// not Pending, or if common result metadata is malformed.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, RngRetrieveError> {
        Self::convert(item, Some(true))
    }

    fn convert(
        item: ResponseBatchItemView<'_>,
        pending_expected: Option<bool>,
    ) -> Result<Self, RngRetrieveError> {
        let result = common::parse_result(item, OPERATION, "RNG Retrieve", pending_expected)?;
        let response_ttlv = common::clone_response_item(item, "RNG Retrieve")?;
        let data = if result.status().raw() == SUCCESS {
            let parsed = item
                .with_response_payload(|payload| parse_success_payload(&payload))
                .ok_or_else(|| {
                    response_error(CryptographicOperationErrorKind::MissingSuccessPayload)
                })??;
            Some(parsed)
        } else {
            None
        };

        Ok(Self {
            result,
            response_ttlv,
            data,
        })
    }

    /// Returns the exact server-reported KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the required RNG output on success; bytes are zeroized when
    /// the returned secret owner is dropped.
    #[must_use]
    pub const fn data(&self) -> Option<&SecretBytes> {
        self.data.as_ref()
    }

    /// Lends the complete ordered generic TTLV response item, preserving
    /// unknown members and values that typed accessors do not interpret.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A sanitized error converting an RNG Retrieve response item.
pub type RngRetrieveError = CryptographicOperationError;

fn parse_success_payload(payload: &StructureView<'_>) -> Result<SecretBytes, RngRetrieveError> {
    let mut data = None;
    for field in payload.children() {
        if field.tag().raw() == DATA {
            if data.is_some() {
                return Err(malformed_response());
            }
            data = Some(
                field
                    .with_value(|value| match value {
                        ValueView::ByteString(bytes) => Some(SecretBytes::new(bytes.to_vec())),
                        _ => None,
                    })
                    .ok_or_else(malformed_response)?,
            );
        }
    }
    data.ok_or_else(malformed_response)
}

fn malformed_response() -> RngRetrieveError {
    response_error(CryptographicOperationErrorKind::MalformedPayload)
}

fn response_error(kind: CryptographicOperationErrorKind) -> RngRetrieveError {
    CryptographicOperationError::new("RNG Retrieve", kind)
}
