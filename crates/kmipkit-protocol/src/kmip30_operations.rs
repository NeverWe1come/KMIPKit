//! Typed KMIP 3.0 system-object operation payload models (KMIPKIT-0050).

use std::{error::Error, fmt};

use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

use crate::cryptographic_operation::OwnedResponseTtlv;
use crate::{
    AttributeSet, Certificate, CredentialType, DeviceCredentialObject,
    HashedPasswordCredentialObject, KmipOperationResult, OneTimePasswordCredentialObject,
    PasswordCredentialObject, ResponseBatchItemView, ResultMessage, ResultValidationError,
    UniqueIdentifier,
};

const CREATE_GROUP_OPERATION: u32 = 0x0000_003C;
const OBLITERATE_OPERATION: u32 = 0x0000_003D;
const CREATE_USER_OPERATION: u32 = 0x0000_003E;
const CREATE_CREDENTIAL_OPERATION: u32 = 0x0000_003F;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const CREDENTIAL_TYPE_TAG: u32 = 0x0042_0024;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;
const SUCCESS: u32 = 0;

fn item(raw_tag: u32, value: Value) -> Result<Item, String> {
    let tag = RawTag::new(raw_tag)
        .map_err(|error| error.to_string())?
        .try_checked()
        .map_err(|error| error.to_string())?;
    Item::new(tag, value).map_err(|error| error.to_string())
}

fn push(payload: &mut Structure, raw_tag: u32, value: Value) -> Result<(), String> {
    payload
        .try_push(item(raw_tag, value)?)
        .map_err(|error| error.to_string())
}

fn unique_identifier_value(identifier: UniqueIdentifier) -> Value {
    match identifier {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    }
}

fn unique_identifier_item(identifier: UniqueIdentifier) -> Result<Item, String> {
    item(UNIQUE_IDENTIFIER_TAG, unique_identifier_value(identifier))
}

pub(crate) fn optional_identifier_payload(
    identifier: Option<UniqueIdentifier>,
) -> Result<Structure, String> {
    let mut payload = Structure::new();
    if let Some(identifier) = identifier {
        payload
            .try_push(unique_identifier_item(identifier)?)
            .map_err(|error| error.to_string())?;
    }
    Ok(payload)
}

pub(crate) fn with_response_ttlv<R>(
    response_ttlv: Option<&OwnedResponseTtlv>,
    callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
) -> Option<R> {
    response_ttlv.map(|response_ttlv| response_ttlv.with_ttlv(callback))
}

fn attributes_item(attributes: AttributeSet) -> Result<Item, String> {
    let mut fields = Structure::new();
    for attribute in attributes.into_items() {
        fields
            .try_push(attribute)
            .map_err(|error| error.to_string())?;
    }
    item(ATTRIBUTES_TAG, Value::structure(fields))
}

fn parse_unique_identifier(field: &Item) -> Option<UniqueIdentifier> {
    if field.tag().raw() != UNIQUE_IDENTIFIER_TAG {
        return None;
    }
    field.with_value(|value| match value {
        ValueView::TextString(value) => Some(UniqueIdentifier::TextString(value.to_owned())),
        ValueView::Enumeration(value) => Some(UniqueIdentifier::Enumeration(*value)),
        ValueView::Integer(value) => Some(UniqueIdentifier::Integer(*value)),
        _ => None,
    })
}

pub(crate) fn parse_success_identifier(
    payload: &StructureView<'_>,
) -> Result<UniqueIdentifier, Kmip30OperationError> {
    let mut identifier = None;
    for field in payload.children() {
        if field.tag().raw() == UNIQUE_IDENTIFIER_TAG {
            if identifier.is_some() {
                return Err(Kmip30OperationError::MalformedSuccessPayload);
            }
            identifier = parse_unique_identifier(field);
            if identifier.is_none() {
                return Err(Kmip30OperationError::MalformedSuccessPayload);
            }
        }
    }
    identifier.ok_or(Kmip30OperationError::MalformedSuccessPayload)
}

pub(crate) fn parse_result(
    item: ResponseBatchItemView<'_>,
    expected_operation: u32,
) -> Result<KmipOperationResult, Kmip30OperationError> {
    if item.operation() != Some(expected_operation) {
        return Err(Kmip30OperationError::UnexpectedOperation);
    }
    let status = item
        .result_status()
        .ok_or(Kmip30OperationError::MissingResultStatus)?;
    let message = item.with_result_message(|text| ResultMessage::new(text.to_owned()));
    KmipOperationResult::new(status, item.result_reason(), message)
        .map_err(Kmip30OperationError::InvalidOperationResult)
}

pub(crate) fn parse_typed_success_response<T>(
    item: ResponseBatchItemView<'_>,
    expected_operation: u32,
    operation_name: &'static str,
    parse_payload: impl for<'a> FnOnce(StructureView<'a>) -> Result<T, Kmip30OperationError>,
) -> Result<(KmipOperationResult, Option<T>, Option<OwnedResponseTtlv>), Kmip30OperationError> {
    let result = parse_result(item, expected_operation)?;
    if result.status().raw() != SUCCESS {
        return Ok((result, None, None));
    }

    let payload = item
        .with_response_payload(parse_payload)
        .ok_or(Kmip30OperationError::MissingSuccessPayload)??;
    let response_ttlv = clone_success_response(item, operation_name)?;
    Ok((result, Some(payload), Some(response_ttlv)))
}

pub(crate) fn parse_typed_success_response_as<T, R>(
    item: ResponseBatchItemView<'_>,
    expected_operation: u32,
    operation_name: &'static str,
    parse_payload: impl for<'a> FnOnce(StructureView<'a>) -> Result<T, Kmip30OperationError>,
    build: impl FnOnce(KmipOperationResult, Option<T>, Option<OwnedResponseTtlv>) -> R,
) -> Result<R, Kmip30OperationError> {
    let (result, payload, response_ttlv) =
        parse_typed_success_response(item, expected_operation, operation_name, parse_payload)?;
    Ok(build(result, payload, response_ttlv))
}

pub(crate) fn clone_success_response(
    item: ResponseBatchItemView<'_>,
    operation: &'static str,
) -> Result<OwnedResponseTtlv, Kmip30OperationError> {
    crate::cryptographic_operation::clone_response_item(item, operation)
        .map_err(|_| Kmip30OperationError::MalformedResponseItem)
}

/// A payload-free error converting a KMIP 3.0 operation response.
///
/// This error never retains server-provided payload values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kmip30OperationError {
    /// The response batch item identifies a different operation.
    UnexpectedOperation,
    /// The validated response item does not expose Result Status.
    MissingResultStatus,
    /// The common status/reason pair is invalid.
    InvalidOperationResult(ResultValidationError),
    /// A successful response has no Response Payload Structure.
    MissingSuccessPayload,
    /// The validated response item could not be retained in the TTLV model.
    MalformedResponseItem,
    /// A successful response has missing, duplicate, mistyped, or forbidden fields.
    MalformedSuccessPayload,
}

impl fmt::Display for Kmip30OperationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::UnexpectedOperation => "response item is for another KMIP operation",
            Self::MissingResultStatus => "KMIP operation result status is missing",
            Self::InvalidOperationResult(error) => {
                return write!(formatter, "KMIP operation result is invalid: {error}");
            }
            Self::MissingSuccessPayload => "successful KMIP response payload is missing",
            Self::MalformedResponseItem => "KMIP response item could not be retained",
            Self::MalformedSuccessPayload => "successful KMIP response payload is malformed",
        };
        formatter.write_str(message)
    }
}

impl Error for Kmip30OperationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(error) => Some(error),
            _ => None,
        }
    }
}

/// One object alternative permitted by KMIP 3.0 §6.1.9, Table 277.
///
/// It intentionally excludes unrelated objects and generic future object tags.
#[non_exhaustive]
pub enum CreateCredentialObject {
    /// Certificate object alternative.
    Certificate(Certificate),
    /// Password Credential system object alternative.
    PasswordCredential(PasswordCredentialObject),
    /// Device Credential system object alternative.
    DeviceCredential(DeviceCredentialObject),
    /// Hashed Password Credential system object alternative.
    HashedPasswordCredential(HashedPasswordCredentialObject),
    /// One Time Password Credential system object alternative.
    OneTimePasswordCredential(OneTimePasswordCredentialObject),
}

impl CreateCredentialObject {
    const fn credential_type(&self) -> CredentialType {
        match self {
            Self::Certificate(_) => CredentialType::Attestation,
            Self::PasswordCredential(_) => CredentialType::UsernameAndPassword,
            Self::DeviceCredential(_) => CredentialType::Device,
            Self::HashedPasswordCredential(_) => CredentialType::HashedPassword,
            Self::OneTimePasswordCredential(_) => CredentialType::OneTimePassword,
        }
    }

    fn into_item(self) -> Item {
        match self {
            Self::Certificate(value) => value.into_item(),
            Self::PasswordCredential(value) => value.into_item(),
            Self::DeviceCredential(value) => value.into_item(),
            Self::HashedPasswordCredential(value) => value.into_item(),
            Self::OneTimePasswordCredential(value) => value.into_item(),
        }
    }

    const fn variant_name(&self) -> &'static str {
        match self {
            Self::Certificate(_) => "Certificate",
            Self::PasswordCredential(_) => "PasswordCredential",
            Self::DeviceCredential(_) => "DeviceCredential",
            Self::HashedPasswordCredential(_) => "HashedPasswordCredential",
            Self::OneTimePasswordCredential(_) => "OneTimePasswordCredential",
        }
    }
}

impl fmt::Debug for CreateCredentialObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.variant_name())
    }
}

/// A KMIP 3.0 Create Credential request from §6.1.9, Table 277.
pub struct CreateCredentialRequest {
    credential_type: CredentialType,
    attributes: AttributeSet,
    object: CreateCredentialObject,
}

impl CreateCredentialRequest {
    /// Creates a request with Attributes and a closed object alternative.
    /// The required Credential Type is derived from the selected alternative.
    #[must_use]
    pub const fn new(attributes: AttributeSet, object: CreateCredentialObject) -> Self {
        let credential_type = object.credential_type();
        Self {
            credential_type,
            attributes,
            object,
        }
    }

    /// Serializes the required Table 277 members in source order.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV model error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut payload = Structure::new();
        push(
            &mut payload,
            CREDENTIAL_TYPE_TAG,
            Value::enumeration(self.credential_type.raw()),
        )?;
        payload
            .try_push(attributes_item(self.attributes)?)
            .map_err(|error| error.to_string())?;
        payload
            .try_push(self.object.into_item())
            .map_err(|error| error.to_string())?;
        Ok(payload)
    }
}

impl fmt::Debug for CreateCredentialRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreateCredentialRequest")
            .field("credential_type", &self.credential_type)
            .field("attributes_count", &self.attributes.len())
            .field("object", &self.object)
            .finish()
    }
}

macro_rules! define_create_request {
    ($request:ident, $response:ident, $operation:expr, $request_doc:literal, $response_doc:literal) => {
        #[doc = $request_doc]
        pub struct $request {
            attributes: AttributeSet,
        }

        impl $request {
            /// Creates a request with the required ordered Attributes Structure.
            #[must_use]
            pub const fn new(attributes: AttributeSet) -> Self {
                Self { attributes }
            }

            /// Serializes the required Attributes member.
            ///
            /// # Errors
            ///
            /// Returns a sanitized TTLV model error.
            pub fn into_ttlv_payload(self) -> Result<Structure, String> {
                let mut payload = Structure::new();
                payload
                    .try_push(attributes_item(self.attributes)?)
                    .map_err(|error| error.to_string())?;
                Ok(payload)
            }
        }

        impl fmt::Debug for $request {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($request))
                    .field("attributes_count", &self.attributes.len())
                    .finish()
            }
        }

        #[doc = $response_doc]
        pub struct $response {
            result: KmipOperationResult,
            unique_identifier: Option<UniqueIdentifier>,
            response_ttlv: Option<OwnedResponseTtlv>,
        }

        impl $response {
            /// Creates a successful response payload with the supplied identifier.
            #[must_use]
            pub const fn new(unique_identifier: UniqueIdentifier) -> Self {
                Self {
                    result: KmipOperationResult::success(),
                    unique_identifier: Some(unique_identifier),
                    response_ttlv: None,
                }
            }

            /// Converts one validated KMIP response item.
            ///
            /// Pending and failed results preserve their common result without
            /// requiring success-only payload fields.
            ///
            /// # Errors
            ///
            /// Returns a payload-free error when operation metadata or a
            /// successful payload is malformed.
            pub fn try_from_response_item(
                item: ResponseBatchItemView<'_>,
            ) -> Result<Self, Kmip30OperationError> {
                parse_typed_success_response_as(
                    item,
                    $operation,
                    stringify!($request),
                    |payload| parse_success_identifier(&payload),
                    |result, unique_identifier, response_ttlv| Self {
                        result,
                        unique_identifier,
                        response_ttlv,
                    },
                )
            }

            /// Returns the exact KMIP result, including Pending and unknown statuses.
            #[must_use]
            pub const fn result(&self) -> &KmipOperationResult {
                &self.result
            }

            /// Returns the successful response's original Unique Identifier form.
            #[must_use]
            pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
                self.unique_identifier.as_ref()
            }

            /// Lends the complete original successful response item, including unknown fields.
            ///
            /// Failure and Pending results do not retain a response item.
            pub fn with_ttlv<R>(
                &self,
                callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
            ) -> Option<R> {
                with_response_ttlv(self.response_ttlv.as_ref(), callback)
            }

            /// Serializes the successful response payload, or an empty payload for non-success.
            ///
            /// # Errors
            ///
            /// Returns a sanitized TTLV model error.
            pub fn into_ttlv_payload(self) -> Result<Structure, String> {
                optional_identifier_payload(self.unique_identifier)
            }
        }
    };
}

define_create_request!(
    CreateGroupRequest,
    CreateGroupResponse,
    CREATE_GROUP_OPERATION,
    "A KMIP 3.0 Create Group request from §6.1.10, Table 280.",
    "The typed response to a KMIP 3.0 Create Group operation from Table 281."
);
define_create_request!(
    CreateUserRequest,
    CreateUserResponse,
    CREATE_USER_OPERATION,
    "A KMIP 3.0 Create User request from §6.1.13, Table 290.",
    "The typed response to a KMIP 3.0 Create User operation from Table 291."
);

/// The typed response to a KMIP 3.0 Create Credential operation from Table 278.
pub struct CreateCredentialResponse {
    result: KmipOperationResult,
    unique_identifier: Option<UniqueIdentifier>,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl CreateCredentialResponse {
    /// Creates a successful response payload with the supplied identifier.
    #[must_use]
    pub const fn new(unique_identifier: UniqueIdentifier) -> Self {
        Self {
            result: KmipOperationResult::success(),
            unique_identifier: Some(unique_identifier),
            response_ttlv: None,
        }
    }

    /// Converts one validated Create Credential response batch item.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error when operation metadata or a successful
    /// payload is malformed.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, Kmip30OperationError> {
        parse_typed_success_response_as(
            item,
            CREATE_CREDENTIAL_OPERATION,
            "Create Credential",
            |payload| parse_success_identifier(&payload),
            |result, unique_identifier, response_ttlv| Self {
                result,
                unique_identifier,
                response_ttlv,
            },
        )
    }

    /// Returns the exact KMIP result, including Pending and unknown statuses.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Returns the successful response's original Unique Identifier form.
    #[must_use]
    pub const fn unique_identifier(&self) -> Option<&UniqueIdentifier> {
        self.unique_identifier.as_ref()
    }

    /// Lends the complete original successful response item, including unknown fields.
    ///
    /// Failure and Pending results do not retain a response item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        with_response_ttlv(self.response_ttlv.as_ref(), callback)
    }

    /// Serializes the successful response payload, or an empty payload for non-success.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV model error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        optional_identifier_payload(self.unique_identifier)
    }
}

/// A KMIP 3.0 Obliterate request from §6.1.41, Table 374.
pub struct ObliterateRequest {
    unique_identifier: UniqueIdentifier,
}

impl ObliterateRequest {
    /// Creates an Obliterate request with its required Unique Identifier.
    #[must_use]
    pub const fn new(unique_identifier: UniqueIdentifier) -> Self {
        Self { unique_identifier }
    }

    /// Returns the supplied Unique Identifier in its original wire form.
    #[must_use]
    pub const fn unique_identifier(&self) -> &UniqueIdentifier {
        &self.unique_identifier
    }

    /// Serializes the required Unique Identifier.
    ///
    /// # Errors
    ///
    /// Returns a sanitized TTLV model error.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        let mut payload = Structure::new();
        payload
            .try_push(unique_identifier_item(self.unique_identifier)?)
            .map_err(|error| error.to_string())?;
        Ok(payload)
    }
}

impl fmt::Debug for ObliterateRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ObliterateRequest")
            .field("unique_identifier", &"[REDACTED]")
            .finish()
    }
}

/// The typed response to a KMIP 3.0 Obliterate operation from Table 375.
pub struct ObliterateResponse {
    result: KmipOperationResult,
    response_ttlv: Option<OwnedResponseTtlv>,
}

impl ObliterateResponse {
    /// Creates the successful empty Obliterate response payload.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            result: KmipOperationResult::success(),
            response_ttlv: None,
        }
    }

    /// Converts one validated Obliterate response batch item.
    ///
    /// Successful Obliterate responses require an empty payload. Failure and
    /// Pending results keep their common result and do not require a payload.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for another operation, malformed result,
    /// missing success payload, or a non-empty success payload.
    pub fn try_from_response_item(
        item: ResponseBatchItemView<'_>,
    ) -> Result<Self, Kmip30OperationError> {
        let (result, _payload, response_ttlv) =
            parse_typed_success_response(item, OBLITERATE_OPERATION, "Obliterate", |payload| {
                if payload.children().is_empty() {
                    Ok(())
                } else {
                    Err(Kmip30OperationError::MalformedSuccessPayload)
                }
            })?;
        Ok(Self {
            result,
            response_ttlv,
        })
    }

    /// Returns the exact KMIP result, including Pending and unknown statuses.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }

    /// Lends the complete original successful response item, including unknown fields.
    ///
    /// Failure and Pending results do not retain a response item.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        with_response_ttlv(self.response_ttlv.as_ref(), callback)
    }

    /// Serializes the empty Obliterate response payload.
    ///
    /// # Errors
    ///
    /// This empty payload is always representable.
    pub fn into_ttlv_payload(self) -> Result<Structure, String> {
        Ok(Structure::new())
    }
}

impl Default for ObliterateResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_identifier_parser_preserves_supported_values_and_rejects_bad_shapes() {
        let text = item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("key-1".to_owned()),
        )
        .expect("valid identifier item");
        let enumeration = item(UNIQUE_IDENTIFIER_TAG, Value::enumeration(0x8000_0001))
            .expect("valid identifier item");
        let integer =
            item(UNIQUE_IDENTIFIER_TAG, Value::integer(42)).expect("valid identifier item");
        let wrong_tag = item(0x0042_0001, Value::integer(42)).expect("valid non-identifier item");
        let wrong_type = item(UNIQUE_IDENTIFIER_TAG, Value::byte_string(vec![0xAA]))
            .expect("valid but unsupported identifier type");

        assert_eq!(
            parse_unique_identifier(&text),
            Some(UniqueIdentifier::TextString("key-1".to_owned()))
        );
        assert_eq!(
            parse_unique_identifier(&enumeration),
            Some(UniqueIdentifier::Enumeration(0x8000_0001))
        );
        assert_eq!(
            parse_unique_identifier(&integer),
            Some(UniqueIdentifier::Integer(42))
        );
        assert_eq!(parse_unique_identifier(&wrong_tag), None);
        assert_eq!(parse_unique_identifier(&wrong_type), None);

        let mut with_extension = Structure::new();
        with_extension
            .try_push(wrong_tag)
            .expect("extension item fits in structure");
        with_extension
            .try_push(integer)
            .expect("identifier item fits in structure");
        assert_eq!(
            parse_success_identifier(&with_extension.view()),
            Ok(UniqueIdentifier::Integer(42))
        );

        assert_eq!(
            parse_success_identifier(&Structure::new().view()),
            Err(Kmip30OperationError::MalformedSuccessPayload)
        );
        let mut malformed = Structure::new();
        malformed
            .try_push(wrong_type)
            .expect("malformed value still fits generic TTLV");
        assert_eq!(
            parse_success_identifier(&malformed.view()),
            Err(Kmip30OperationError::MalformedSuccessPayload)
        );

        let duplicate_identifier = item(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("second".to_owned()),
        )
        .expect("valid duplicate identifier item");
        let mut duplicated = Structure::new();
        duplicated
            .try_push(text)
            .expect("first identifier fits generic TTLV");
        duplicated
            .try_push(duplicate_identifier)
            .expect("repeated tags are representable in generic TTLV");
        assert_eq!(
            parse_success_identifier(&duplicated.view()),
            Err(Kmip30OperationError::MalformedSuccessPayload)
        );
    }

    #[test]
    fn operation_errors_have_payload_free_messages_and_expected_sources() {
        let validation = ResultValidationError::FailureRequiresReason;
        let cases = [
            (
                Kmip30OperationError::UnexpectedOperation,
                "response item is for another KMIP operation",
            ),
            (
                Kmip30OperationError::MissingResultStatus,
                "KMIP operation result status is missing",
            ),
            (
                Kmip30OperationError::InvalidOperationResult(validation),
                "KMIP operation result is invalid: Failure requires a Result Reason",
            ),
            (
                Kmip30OperationError::MissingSuccessPayload,
                "successful KMIP response payload is missing",
            ),
            (
                Kmip30OperationError::MalformedResponseItem,
                "KMIP response item could not be retained",
            ),
            (
                Kmip30OperationError::MalformedSuccessPayload,
                "successful KMIP response payload is malformed",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
        let source = Kmip30OperationError::InvalidOperationResult(validation);
        assert_eq!(
            Error::source(&source).map(ToString::to_string).as_deref(),
            Some("Failure requires a Result Reason")
        );
        assert!(Error::source(&Kmip30OperationError::UnexpectedOperation).is_none());
    }
}
