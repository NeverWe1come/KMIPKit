//! Typed KMIP 2.1 RNG Seed request and response models.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use crate::cryptographic_operation::{self as common, OwnedResponseTtlv};
use crate::{
    CryptographicOperationError, CryptographicOperationErrorKind, KmipOperationResult,
    ProtocolError, ResponseBatchItemView, SecretBytes,
};

const OPERATION: u32 = 0x0000_0026;
const DATA: u32 = 0x0042_00C2;
const DATA_LENGTH: u32 = 0x0042_00D5;
const SUCCESS: u32 = 0;

/// A KMIP v2.1 RNG Seed request from §6.1.50, Table 319.
pub struct RngSeedRequest {
    data: SecretBytes,
}

impl RngSeedRequest {
    /// Creates a request that transfers caller-owned seed bytes to the server.
    #[must_use]
    pub const fn new(data: SecretBytes) -> Self {
        Self { data }
    }

    /// Consumes the request and encodes its required Table 319 Data field.
    ///
    /// The seed allocation is moved into TTLV storage, whose byte-string
    /// owner zeroizes the bytes when dropped.
    ///
    /// # Errors
    ///
    /// Returns a sanitized protocol error if the TTLV item cannot be
    /// represented.
    pub fn to_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        common::push(&mut payload, DATA, common::secret_value(self.data))?;
        Ok(payload)
    }
}

impl fmt::Debug for RngSeedRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RngSeedRequest")
            .field("data", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}

/// The typed result of one KMIP v2.1 RNG Seed response.
#[derive(Debug)]
pub struct RngSeedResponse {
    result: KmipOperationResult,
    response_ttlv: OwnedResponseTtlv,
    data_length: Option<i32>,
}

impl RngSeedResponse {
    /// Converts one completed Table 320 response item.
    ///
    /// A zero Data Length is preserved as a successful result: KMIP v2.1
    /// §6.1.50 explicitly allows the server to indicate that it did not use
    /// the supplied seed by returning zero.
    ///
    /// # Errors
    ///
    /// Returns [`RngSeedError`] when the operation result or successful
    /// Table 320 payload is malformed, or when the result is Pending.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, RngSeedError> {
        Self::convert(item, Some(false))
    }

    /// Converts a caller-managed Pending response without issuing a Poll.
    ///
    /// # Errors
    ///
    /// Returns [`RngSeedError`] if the item is not this operation or is not
    /// Pending, or if common result metadata is malformed.
    pub fn try_from_pending_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, RngSeedError> {
        Self::convert(item, Some(true))
    }

    fn convert(
        item: ResponseBatchItemView<'_>,
        pending_expected: Option<bool>,
    ) -> Result<Self, RngSeedError> {
        let result = common::parse_result(item, OPERATION, "RNG Seed", pending_expected)?;
        let response_ttlv = common::clone_response_item(item, "RNG Seed")?;
        let data_length = if result.status().raw() == SUCCESS {
            Some(
                item.with_response_payload(|payload| parse_success_payload(&payload))
                    .ok_or_else(|| {
                        response_error(CryptographicOperationErrorKind::MissingSuccessPayload)
                    })??,
            )
        } else {
            None
        };

        Ok(Self {
            result,
            response_ttlv,
            data_length,
        })
    }

    /// Returns the exact server-reported KMIP operation result.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the Data Length from a successful Table 320 response.
    ///
    /// In particular, `Some(0)` is a successful response, not a failure.
    #[must_use]
    pub const fn data_length(&self) -> Option<i32> {
        self.data_length
    }

    /// Lends the complete ordered generic TTLV response item, preserving
    /// unknown members and values that typed accessors do not interpret.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.response_ttlv.with_ttlv(callback)
    }
}

/// A sanitized error converting an RNG Seed response item.
pub type RngSeedError = CryptographicOperationError;

fn parse_success_payload(payload: &StructureView<'_>) -> Result<i32, RngSeedError> {
    let mut data_length = None;
    for field in payload
        .children()
        .iter()
        .filter(|field| field.tag().raw() == DATA_LENGTH)
    {
        if data_length.is_some() {
            return Err(malformed_response());
        }
        data_length = Some(
            field
                .with_value(|value| match value {
                    ValueView::Integer(value) => Some(*value),
                    _ => None,
                })
                .ok_or_else(malformed_response)?,
        );
    }
    data_length.ok_or_else(malformed_response)
}

fn malformed_response() -> RngSeedError {
    common::response_shape_error("RNG Seed")
}

fn response_error(kind: CryptographicOperationErrorKind) -> RngSeedError {
    CryptographicOperationError::new("RNG Seed", kind)
}
