//! Canonical KMIP 2.1 response message builder for Server-to-Client responses.

use std::time::{SystemTime, UNIX_EPOCH};

use kmipkit_ttlv::{ModelError, Structure, Value};

use crate::asynchronous::item;
use crate::{
    ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, ProtocolVersion, ResponseMessage,
    ResultReason, ResultStatus,
};

const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const UNIQUE_BATCH_ITEM_ID_TAG: u32 = 0x0042_0094;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const RESULT_REASON_TAG: u32 = 0x0042_007E;
const RESULT_MESSAGE_TAG: u32 = 0x0042_007D;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;

/// Builds canonical KMIP 2.1 response messages from client to server.
#[derive(Debug)]
pub struct ServerResponseBuilder {
    version: ProtocolVersion,
    timestamp: i64,
    maximum_response_size: Option<usize>,
    items: Vec<Structure>,
}

impl Default for ServerResponseBuilder {
    fn default() -> Self {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|d| i64::try_from(d.as_secs()).ok())
            .unwrap_or(0);
        Self {
            version: ProtocolVersion::from_raw(2, 1),
            timestamp: ts,
            maximum_response_size: None,
            items: Vec::new(),
        }
    }
}

impl ServerResponseBuilder {
    /// Creates a new `ServerResponseBuilder` with default KMIP 2.1 protocol version.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `ServerResponseBuilder` with the specified protocol version.
    #[must_use]
    pub fn with_version(version: ProtocolVersion) -> Self {
        Self {
            version,
            ..Self::default()
        }
    }

    /// Sets the optional maximum response size limit in bytes.
    #[must_use]
    pub fn with_maximum_response_size(mut self, max_size: Option<usize>) -> Self {
        self.maximum_response_size = max_size;
        self
    }

    /// Adds a raw pre-assembled batch item Structure to the response.
    pub fn add_item(&mut self, item: Structure) {
        self.items.push(item);
    }

    /// Adds a successful batch item to the response.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if item structure assembly fails.
    pub fn add_success_item(
        &mut self,
        operation: u32,
        unique_id: Option<Vec<u8>>,
        payload: Structure,
    ) -> Result<(), ProtocolError> {
        let mut item_structure = Structure::new();
        item_structure
            .try_push(item(OPERATION_TAG, Value::enumeration(operation))?)
            .map_err(protocol_model_error)?;
        if let Some(uid) = unique_id {
            item_structure
                .try_push(item(UNIQUE_BATCH_ITEM_ID_TAG, Value::byte_string(uid))?)
                .map_err(protocol_model_error)?;
        }
        item_structure
            .try_push(item(RESULT_STATUS_TAG, Value::enumeration(0))?) // Success = 0
            .map_err(protocol_model_error)?;
        item_structure
            .try_push(item(RESPONSE_PAYLOAD_TAG, Value::structure(payload))?)
            .map_err(protocol_model_error)?;

        self.items.push(item_structure);
        Ok(())
    }

    /// Adds a failure batch item to the response.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if item structure assembly fails.
    pub fn add_failure_item(
        &mut self,
        operation: u32,
        unique_id: Option<Vec<u8>>,
        status: ResultStatus,
        reason: Option<ResultReason>,
        message: Option<String>,
    ) -> Result<(), ProtocolError> {
        let mut item_structure = Structure::new();
        item_structure
            .try_push(item(OPERATION_TAG, Value::enumeration(operation))?)
            .map_err(protocol_model_error)?;
        if let Some(uid) = unique_id {
            item_structure
                .try_push(item(UNIQUE_BATCH_ITEM_ID_TAG, Value::byte_string(uid))?)
                .map_err(protocol_model_error)?;
        }
        item_structure
            .try_push(item(RESULT_STATUS_TAG, Value::enumeration(status.raw()))?)
            .map_err(protocol_model_error)?;
        if let Some(r) = reason {
            item_structure
                .try_push(item(RESULT_REASON_TAG, Value::enumeration(r.raw()))?)
                .map_err(protocol_model_error)?;
        }
        if let Some(msg) = message {
            item_structure
                .try_push(item(RESULT_MESSAGE_TAG, Value::text_string(msg))?)
                .map_err(protocol_model_error)?;
        }

        self.items.push(item_structure);
        Ok(())
    }

    /// Builds the complete `ResponseMessage` containing all accumulated batch items.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if response message validation fails.
    pub fn build(self) -> Result<ResponseMessage, ProtocolError> {
        let count = i32::try_from(self.items.len()).map_err(|_| {
            ProtocolError::new(
                ProtocolErrorKind::ResourceLimit,
                ProtocolCauseCategory::InvalidValue,
                std::io::Error::other("batch count exceeds i32 limit"),
            )
        })?;

        let mut header = Structure::new();
        let mut pv = Structure::new();
        pv.try_push(item(
            PROTOCOL_VERSION_MAJOR_TAG,
            Value::integer(self.version.major()),
        )?)
        .map_err(protocol_model_error)?;
        pv.try_push(item(
            PROTOCOL_VERSION_MINOR_TAG,
            Value::integer(self.version.minor()),
        )?)
        .map_err(protocol_model_error)?;
        header
            .try_push(item(PROTOCOL_VERSION_TAG, Value::structure(pv))?)
            .map_err(protocol_model_error)?;
        header
            .try_push(item(TIME_STAMP_TAG, Value::date_time(self.timestamp))?)
            .map_err(protocol_model_error)?;
        header
            .try_push(item(BATCH_COUNT_TAG, Value::integer(count))?)
            .map_err(protocol_model_error)?;

        let mut root = Structure::new();
        root.try_push(item(RESPONSE_HEADER_TAG, Value::structure(header))?)
            .map_err(protocol_model_error)?;
        for item_structure in self.items {
            root.try_push(item(BATCH_ITEM_TAG, Value::structure(item_structure))?)
                .map_err(protocol_model_error)?;
        }

        let response = ResponseMessage::try_from_ttlv(root).map_err(|e| {
            ProtocolError::new(
                ProtocolErrorKind::MalformedMessage,
                ProtocolCauseCategory::InvalidValue,
                e,
            )
        })?;

        Ok(response)
    }

    /// Builds a successful response message for a single operation.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if response assembly or size limits fail.
    pub fn build_success(
        mut self,
        operation: u32,
        payload: Option<Structure>,
    ) -> Result<ResponseMessage, ProtocolError> {
        if let Some(p) = payload {
            self.add_success_item(operation, None, p)?;
        } else {
            let mut item_structure = Structure::new();
            item_structure
                .try_push(item(OPERATION_TAG, Value::enumeration(operation))?)
                .map_err(protocol_model_error)?;
            item_structure
                .try_push(item(RESULT_STATUS_TAG, Value::enumeration(0))?)
                .map_err(protocol_model_error)?;
            self.items.push(item_structure);
        }
        self.build()
    }

    /// Builds an error response message for a failed operation.
    ///
    /// # Errors
    ///
    /// Returns a `ProtocolError` if response assembly fails.
    pub fn build_error(
        mut self,
        operation: u32,
        reason: ResultReason,
        message: Option<&str>,
    ) -> Result<ResponseMessage, ProtocolError> {
        self.add_failure_item(
            operation,
            None,
            ResultStatus::from_raw(1),
            Some(reason),
            message.map(ToString::to_string),
        )?;
        self.build()
    }
}

fn protocol_model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}
