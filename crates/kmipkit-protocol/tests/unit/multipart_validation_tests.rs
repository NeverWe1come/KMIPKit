//! Encrypt/Decrypt Data and multipart request-builder matrix from OASIS KMIP
//! v2.1 §6.1, Decrypt Table 196 (§6.1.11), Encrypt Table 214 (§6.1.17), and
//! §§7.8, 7.14, and 7.17. Tests exercise `to_ttlv_payload` as the local
//! pre-encoding gate, not client dispatch or a complete OASIS Test Case.

use crate::{
    DecryptRequest, EncryptRequest, OperationData, ProtocolCauseCategory, ProtocolError,
    ProtocolErrorKind, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::Structure;

const SENSITIVE_SENTINEL: &[u8] = b"T026-sensitive-sentinel";

#[derive(Clone, Copy, Debug)]
struct RequestShape {
    has_data: bool,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
    has_correlation_value: bool,
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Encrypt,
    Decrypt,
}

impl Operation {
    const ALL: [Self; 2] = [Self::Encrypt, Self::Decrypt];

    const fn name(self) -> &'static str {
        match self {
            Self::Encrypt => "Encrypt",
            Self::Decrypt => "Decrypt",
        }
    }
}

const fn shape(
    has_data: bool,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
    has_correlation_value: bool,
) -> RequestShape {
    RequestShape {
        has_data,
        init_indicator,
        final_indicator,
        has_correlation_value,
    }
}

fn request_data(shape: RequestShape) -> Option<OperationData> {
    shape
        .has_data
        .then(|| OperationData::ByteString(SecretBytes::new(SENSITIVE_SENTINEL.to_vec())))
}

fn apply_encrypt_shape(mut request: EncryptRequest, shape: RequestShape) -> EncryptRequest {
    if let Some(value) = shape.init_indicator {
        request = request.with_init_indicator(value);
    }
    if let Some(value) = shape.final_indicator {
        request = request.with_final_indicator(value);
    }
    if shape.has_correlation_value {
        request = request.with_correlation_value(SecretBytes::new(vec![0xA1, 0xB2]));
    }
    request
}

fn apply_decrypt_shape(mut request: DecryptRequest, shape: RequestShape) -> DecryptRequest {
    if let Some(value) = shape.init_indicator {
        request = request.with_init_indicator(value);
    }
    if let Some(value) = shape.final_indicator {
        request = request.with_final_indicator(value);
    }
    if shape.has_correlation_value {
        request = request.with_correlation_value(SecretBytes::new(vec![0xA1, 0xB2]));
    }
    request
}

fn build_request(operation: Operation, shape: RequestShape) -> Result<Structure, ProtocolError> {
    let unique_identifier = Some(UniqueIdentifier::TextString("matrix-key".to_owned()));
    match operation {
        Operation::Encrypt => apply_encrypt_shape(
            EncryptRequest::new(unique_identifier, request_data(shape)),
            shape,
        )
        .to_ttlv_payload(),
        Operation::Decrypt => apply_decrypt_shape(
            DecryptRequest::new(unique_identifier, request_data(shape)),
            shape,
        )
        .to_ttlv_payload(),
    }
}

fn assert_sanitized_validation_error(error: &ProtocolError) {
    assert_eq!(error.kind(), ProtocolErrorKind::InvalidValue);
    assert_eq!(error.cause_category(), ProtocolCauseCategory::InvalidValue);
    assert_eq!(error.to_string(), "invalid protocol value (invalid value)");
    assert!(
        !error
            .to_string()
            .contains(std::str::from_utf8(SENSITIVE_SENTINEL).expect("test sentinel is UTF-8")),
        "local validation errors must not include request Data"
    );
}

#[test]
fn both_operations_accept_valid_single_initial_middle_and_final_shapes() {
    let valid_cases = [
        (
            "unframed single part with Data",
            shape(true, None, None, false),
        ),
        (
            "single request marked initial and final with Data",
            shape(true, Some(true), Some(true), false),
        ),
        (
            "initial part with Data and Final absent",
            shape(true, Some(true), None, false),
        ),
        (
            "initial part without Data and Final absent",
            shape(false, Some(true), None, false),
        ),
        (
            "initial part with Data and Final false",
            shape(true, Some(true), Some(false), false),
        ),
        (
            "initial part without Data and Final false",
            shape(false, Some(true), Some(false), false),
        ),
        (
            "middle part with indicators absent",
            shape(true, None, None, true),
        ),
        (
            "middle part with Init false",
            shape(true, Some(false), None, true),
        ),
        (
            "middle part with Final false",
            shape(true, None, Some(false), true),
        ),
        (
            "middle part with both indicators false",
            shape(true, Some(false), Some(false), true),
        ),
        (
            "final part with indicators absent except Final true and Data",
            shape(true, None, Some(true), true),
        ),
        (
            "final part without Data",
            shape(false, None, Some(true), true),
        ),
        (
            "final part with Init false and Data",
            shape(true, Some(false), Some(true), true),
        ),
        (
            "final part with Init false and without Data",
            shape(false, Some(false), Some(true), true),
        ),
    ];

    for operation in Operation::ALL {
        for (case, request_shape) in valid_cases {
            assert!(
                build_request(operation, request_shape).is_ok(),
                "{} should accept valid matrix case: {case}",
                operation.name()
            );
        }
    }
}

#[test]
fn both_operations_reject_invalid_data_multipart_shapes_before_encoding() {
    let invalid_cases = [
        (
            "unframed single part without required Data",
            shape(false, None, None, false),
        ),
        (
            "true/true single request without Data while DISC-045 is open",
            shape(false, Some(true), Some(true), false),
        ),
        (
            "initial multipart part carrying Correlation Value",
            shape(true, Some(true), None, true),
        ),
        (
            "true/true single request carrying Correlation Value",
            shape(true, Some(true), Some(true), true),
        ),
        (
            "middle part without Correlation Value",
            shape(true, None, None, false),
        ),
        (
            "final part without Correlation Value",
            shape(true, None, Some(true), false),
        ),
        (
            "middle part without required Data",
            shape(false, None, None, true),
        ),
    ];

    let mut accepted_invalid_cases = Vec::new();
    for operation in Operation::ALL {
        for (case, request_shape) in invalid_cases {
            match build_request(operation, request_shape) {
                Ok(_) => accepted_invalid_cases.push(format!("{}: {case}", operation.name())),
                Err(error) => assert_sanitized_validation_error(&error),
            }
        }
    }

    assert!(
        accepted_invalid_cases.is_empty(),
        "to_ttlv_payload accepted invalid matrix cases before encoding: {accepted_invalid_cases:?}"
    );
}
