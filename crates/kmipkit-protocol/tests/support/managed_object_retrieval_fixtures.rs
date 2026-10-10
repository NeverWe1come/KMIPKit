//! Generic TTLV fixtures for OASIS KMIP v2.1 Get and Locate payloads.
//!
//! Payload field availability follows §6.1.19 Tables 220–221 and §6.1.28
//! Tables 247–248. These builders keep caller-supplied generic fields intact;
//! they are derived test fixtures, not official conformance vectors.

use kmipkit_ttlv::{Structure, Value};

/// Builds a generic Get Request Payload in the supplied field order.
pub(crate) fn get_request_payload(fields: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    crate::cryptographic_operation_test_support::payload(fields)
}

/// Builds a generic Get Response Payload in the supplied field order.
pub(crate) fn get_response_payload(fields: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    crate::cryptographic_operation_test_support::payload(fields)
}

/// Builds a generic Locate Request Payload in the supplied field order.
pub(crate) fn locate_request_payload(fields: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    crate::cryptographic_operation_test_support::payload(fields)
}

/// Builds a generic Locate Response Payload in the supplied field order.
pub(crate) fn locate_response_payload(fields: impl IntoIterator<Item = (u32, Value)>) -> Structure {
    crate::cryptographic_operation_test_support::payload(fields)
}

#[cfg(test)]
mod tests {
    use crate::cryptographic_operation_test_support::field_tags;
    use crate::managed_object_retrieval_fixtures::{
        get_request_payload, get_response_payload, locate_request_payload, locate_response_payload,
    };
    use kmipkit_ttlv::Value;

    const OBJECT_TYPE: u32 = 0x0042_0057;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;

    fn ordered_repeated_fields() -> [(u32, Value); 3] {
        [
            (OBJECT_TYPE, Value::enumeration(7)),
            (
                UNIQUE_IDENTIFIER,
                Value::text_string("first-identifier".to_owned()),
            ),
            (
                UNIQUE_IDENTIFIER,
                Value::text_string("second-identifier".to_owned()),
            ),
        ]
    }

    #[test]
    fn payload_builders_preserve_field_order_and_repetitions() {
        let expected = [OBJECT_TYPE, UNIQUE_IDENTIFIER, UNIQUE_IDENTIFIER];
        let payloads = [
            get_request_payload(ordered_repeated_fields()),
            get_response_payload(ordered_repeated_fields()),
            locate_request_payload(ordered_repeated_fields()),
            locate_response_payload(ordered_repeated_fields()),
        ];

        for payload in payloads {
            assert_eq!(field_tags(&payload), expected);
        }
    }
}
