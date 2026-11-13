use super::{ResponseEncodingError, map_response_encoding_error, private_wire_writer};
use crate::wire_codec_common::{WireCodecError, WireCodecErrorKind};

#[test]
fn every_private_response_encoding_failure_maps_to_a_payload_free_error() {
    let allocation_failure = Vec::<u8>::new()
        .try_reserve(usize::MAX)
        .expect_err("an impossible capacity deterministically yields a reserve error");
    let cases = [
        (
            private_wire_writer::EncodeError::LimitExceeded,
            ResponseEncodingError::LimitExceeded,
        ),
        (
            private_wire_writer::EncodeError::AllocationFailed(allocation_failure),
            ResponseEncodingError::AllocationFailed,
        ),
        (
            private_wire_writer::EncodeError::WireCodec(WireCodecError::new(
                WireCodecErrorKind::LimitExceeded,
            )),
            ResponseEncodingError::LimitExceeded,
        ),
        (
            private_wire_writer::EncodeError::WireCodec(WireCodecError::new(
                WireCodecErrorKind::AllocationFailed,
            )),
            ResponseEncodingError::AllocationFailed,
        ),
        (
            private_wire_writer::EncodeError::WireCodec(WireCodecError::new(
                WireCodecErrorKind::InvalidDocument,
            )),
            ResponseEncodingError::EncodingFailed,
        ),
        (
            private_wire_writer::EncodeError::WireCodec(WireCodecError::new(
                WireCodecErrorKind::InvalidValue,
            )),
            ResponseEncodingError::EncodingFailed,
        ),
        (
            private_wire_writer::EncodeError::UnsupportedValueType,
            ResponseEncodingError::EncodingFailed,
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(map_response_encoding_error(&source), expected);
    }
}
