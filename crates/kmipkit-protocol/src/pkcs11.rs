//! Typed opaque PKCS#11 forwarding, OASIS KMIP v2.1 §6.1.37 Tables 273–275.
//! Nested structures follow §§7.26–7.30; no PKCS#11 profile codec is provided.
use crate::{
    KmipOperationResult, ProtocolError, ProtocolErrorKind, ResponseBatchItemView,
    ResultValidationError, SecretBytes, cryptographic_operation as common,
};
use kmipkit_ttlv::{Item, Structure, StructureView, Value, ValueView};
use std::{error::Error, fmt};
const OPERATION: u32 = 0x33;
const INTERFACE: u32 = 0x420159;
const FUNCTION: u32 = 0x42015a;
const INPUT: u32 = 0x42015b;
const OUTPUT: u32 = 0x42015c;
const RETURN: u32 = 0x42015d;
const CORRELATION: u32 = 0x4200d6;
const RESPONSE_PAYLOAD: u32 = 0x42007c;

/// An explicit Table 273 request. Generic owned payloads zeroize on drop.
/// The caller supplies the Function Structure, including its Enumeration or
/// vendor Integer child. No default interface or correlation is synthesized.
#[derive(Debug)]
pub struct Pkcs11Request {
    function: Structure,
    interface: Option<Structure>,
    correlation: Option<SecretBytes>,
    input: Option<Structure>,
    extensions: Vec<Item>,
}
impl Pkcs11Request {
    /// Creates a request with a caller-selected required Function Structure.
    #[must_use]
    pub const fn new(function: Structure) -> Self {
        Self {
            function,
            interface: None,
            correlation: None,
            input: None,
            extensions: Vec::new(),
        }
    }
    /// Retains the explicit opaque Interface Structure without interpreting it.
    #[must_use]
    pub fn with_interface(mut self, interface: Structure) -> Self {
        self.interface = Some(interface);
        self
    }
    /// Supplies the exact previous server correlation bytes chosen by the caller.
    #[must_use]
    pub fn with_correlation_value(mut self, correlation: SecretBytes) -> Self {
        self.correlation = Some(correlation);
        self
    }
    /// Supplies opaque Input Parameters, preserving unknown nested members.
    #[must_use]
    pub fn with_input_parameters(mut self, input: Structure) -> Self {
        self.input = Some(input);
        self
    }
    /// Appends a generic extension. Known fields are validated before transmission.
    #[must_use]
    pub fn with_extension(mut self, extension: Item) -> Self {
        self.extensions.push(extension);
        self
    }
    /// Consumes the request into Table 273 order and validates known fields.
    ///
    /// # Errors
    /// Returns a sanitized error for malformed fields or TTLV resource limits.
    pub fn into_ttlv_payload(self) -> Result<Structure, ProtocolError> {
        let mut payload = Structure::new();
        if let Some(interface) = self.interface {
            common::push(&mut payload, INTERFACE, Value::structure(interface))?;
        }
        common::push(&mut payload, FUNCTION, Value::structure(self.function))?;
        if let Some(correlation) = self.correlation {
            common::push(&mut payload, CORRELATION, correlation.into_ttlv_value())?;
        }
        if let Some(input) = self.input {
            common::push(&mut payload, INPUT, Value::structure(input))?;
        }
        for extension in self.extensions {
            payload.try_push(extension).map_err(|_| invalid_request())?;
        }
        validate_payload(payload.view(), false).map_err(|_| invalid_request())?;
        Ok(payload)
    }
}
fn invalid_request() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::InvalidValue,
        crate::ProtocolCauseCategory::InvalidValue,
    )
}

/// A typed PKCS#11 result retaining the complete zeroizing generic response item.
#[derive(Debug)]
pub struct Pkcs11Response {
    result: KmipOperationResult,
    ttlv: common::OwnedResponseTtlv,
}
impl Pkcs11Response {
    /// Converts Table 274 Success or Table 275/common Failure and Pending results.
    /// Success requires Function and Return Code. Non-success results retain
    /// generic payloads without fabricating success fields.
    ///
    /// # Errors
    /// Returns a payload-free error for operation/result or malformed success fields.
    pub fn try_from_response_item(item: ResponseBatchItemView<'_>) -> Result<Self, Pkcs11Error> {
        if item.operation() != Some(OPERATION) {
            return Err(Pkcs11Error::UnexpectedOperation);
        }
        let result = crate::result::parse_operation_result(item).map_err(|e| match e {
            crate::result::OperationResultParseError::MissingResultStatus => {
                Pkcs11Error::MissingResultStatus
            }
            crate::result::OperationResultParseError::Invalid(cause) => {
                Pkcs11Error::InvalidOperationResult(cause)
            }
        })?;
        if result.status().raw() == 0 {
            item.with_response_payload(|p| validate_payload(p, true))
                .ok_or(Pkcs11Error::MissingResponsePayload)??;
        }
        let ttlv = common::clone_response_item(item, "PKCS#11")
            .map_err(|_| Pkcs11Error::MalformedPayload)?;
        Ok(Self { result, ttlv })
    }
    /// Returns the complete common KMIP result, including open status and reason.
    #[must_use]
    pub const fn result(&self) -> &KmipOperationResult {
        &self.result
    }
    /// Lends the complete generic response item, including unknown extensions.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        self.ttlv.with_ttlv(callback)
    }
    /// Lends the original payload when present, preserving unknown fields and order.
    pub fn with_ttlv_payload<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.ttlv
            .with_ttlv(|tree| with_structure(tree, RESPONSE_PAYLOAD, callback))
    }
    /// Lends the optional opaque Interface Structure. Absence is never defaulted.
    pub fn with_interface<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.with_ttlv_payload(|p| with_structure(p, INTERFACE, callback))
            .flatten()
    }
    /// Lends the Function Structure when supplied by the server.
    pub fn with_function<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.with_ttlv_payload(|p| with_structure(p, FUNCTION, callback))
            .flatten()
    }
    /// Lends exact sensitive correlation bytes without allocating an external copy.
    pub fn with_correlation_value<R>(
        &self,
        callback: impl for<'a> FnOnce(&'a [u8]) -> R,
    ) -> Option<R> {
        self.with_ttlv_payload(|p| {
            p.children()
                .iter()
                .find(|f| f.tag().raw() == CORRELATION)?
                .with_value(|v| match v {
                    ValueView::ByteString(bytes) => Some(callback(bytes)),
                    _ => None,
                })
        })
        .flatten()
    }
    /// Lends optional opaque Output Parameters without interpreting profile data.
    pub fn with_output_parameters<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.with_ttlv_payload(|p| with_structure(p, OUTPUT, callback))
            .flatten()
    }
    /// Lends the complete Return Code Structure, preserving unknown members.
    pub fn with_return_code<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Option<R> {
        self.with_ttlv_payload(|p| with_structure(p, RETURN, callback))
            .flatten()
    }
}
fn with_structure<R>(
    tree: StructureView<'_>,
    tag: u32,
    callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
) -> Option<R> {
    tree.children()
        .iter()
        .find(|f| f.tag().raw() == tag)?
        .with_value(|v| match v {
            ValueView::Structure(s) => Some(callback(s)),
            _ => None,
        })
}
fn validate_payload(payload: StructureView<'_>, response: bool) -> Result<(), Pkcs11Error> {
    let mut seen = [false; 5];
    let mut last = 0;
    for field in payload.children() {
        let tag = field.tag().raw();
        let position = match tag {
            INTERFACE => 0,
            FUNCTION => 1,
            CORRELATION => 2,
            OUTPUT if response => 3,
            INPUT if !response => 3,
            RETURN if response => 4,
            _ => continue,
        };
        if seen[position] || position < last {
            return Err(Pkcs11Error::MalformedPayload);
        }
        seen[position] = true;
        last = position;
        let valid = field.with_value(|v| match (tag, v) {
            (CORRELATION, ValueView::ByteString(_)) => true,
            (FUNCTION, ValueView::Structure(s)) => valid_required_value(s, FUNCTION, true),
            (RETURN, ValueView::Structure(s)) => valid_required_value(s, RETURN, false),
            (INTERFACE | INPUT | OUTPUT, ValueView::Structure(_)) => true,
            _ => false,
        });
        if !valid {
            return Err(Pkcs11Error::MalformedPayload);
        }
    }
    if !seen[1] || (response && !seen[4]) {
        return Err(Pkcs11Error::MalformedPayload);
    }
    Ok(())
}
fn valid_required_value(tree: StructureView<'_>, tag: u32, integer_allowed: bool) -> bool {
    let mut values = tree.children().iter().filter(|f| f.tag().raw() == tag);
    let Some(value) = values.next() else {
        return false;
    };
    values.next().is_none()
        && value.with_value(|v| {
            matches!(v, ValueView::Enumeration(_))
                || (integer_allowed && matches!(v, ValueView::Integer(_)))
        })
}
/// A sanitized PKCS#11 conversion error that never retains payload values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pkcs11Error {
    /// The batch item belongs to another operation.
    UnexpectedOperation,
    /// The batch item omitted Result Status.
    MissingResultStatus,
    /// Common result metadata is inconsistent.
    InvalidOperationResult(ResultValidationError),
    /// A successful result omitted Response Payload.
    MissingResponsePayload,
    /// A known field has invalid type, cardinality, order, or nested required value.
    MalformedPayload,
}
impl fmt::Display for Pkcs11Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedOperation => f.write_str("response item is not PKCS#11"),
            Self::MissingResultStatus => f.write_str("PKCS#11 result status is missing"),
            Self::InvalidOperationResult(e) => write!(f, "invalid PKCS#11 operation result: {e}"),
            Self::MissingResponsePayload => {
                f.write_str("successful PKCS#11 response payload is missing")
            }
            Self::MalformedPayload => f.write_str("PKCS#11 payload has an invalid structure"),
        }
    }
}
impl Error for Pkcs11Error {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidOperationResult(e) => Some(e),
            _ => None,
        }
    }
}
