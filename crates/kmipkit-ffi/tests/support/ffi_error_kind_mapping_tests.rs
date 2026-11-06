use super::*;

#[test]
fn ttlv_decode_error_codes_are_stable_for_every_defined_kind() {
    let cases = [
        (DecodeErrorKind::MessageTooLarge, 1),
        (DecodeErrorKind::TruncatedHeader, 2),
        (DecodeErrorKind::TruncatedValue, 3),
        (DecodeErrorKind::InvalidItemLength, 4),
        (DecodeErrorKind::EmptyBigInteger, 5),
        (DecodeErrorKind::InvalidUtf8, 6),
        (DecodeErrorKind::InvalidBoolean, 7),
        (DecodeErrorKind::UnsupportedItemType, 8),
        (DecodeErrorKind::TrailingBytes, 9),
        (DecodeErrorKind::StructureBoundary, 10),
        (DecodeErrorKind::ReservedTag, 11),
        (DecodeErrorKind::UnallocatedTag, 12),
        (DecodeErrorKind::InvalidPaddingExtent, 13),
        (DecodeErrorKind::StructureDepthExceeded, 14),
        (DecodeErrorKind::ElementLimitExceeded, 15),
        (DecodeErrorKind::AllocationFailed, 16),
        (DecodeErrorKind::ModelConstraint, 17),
    ];

    for (kind, expected_code) in cases {
        assert_eq!(decode_error_kind_code(kind), expected_code);
    }
}

#[test]
fn message_validation_error_codes_are_stable_for_every_defined_kind() {
    let cases = [
        (MessageValidationErrorKind::MissingRequiredField, 1),
        (MessageValidationErrorKind::DuplicateField, 2),
        (MessageValidationErrorKind::FieldOutOfOrder, 3),
        (MessageValidationErrorKind::WrongItemType, 4),
        (MessageValidationErrorKind::InvalidBatchCount, 5),
        (MessageValidationErrorKind::BatchCountMismatch, 6),
        (MessageValidationErrorKind::InvalidSingleItemOption, 7),
        (MessageValidationErrorKind::InvalidResult, 8),
        (MessageValidationErrorKind::InvalidFieldValue, 9),
    ];

    for (kind, expected_code) in cases {
        assert_eq!(message_validation_error_kind_code(kind), expected_code);
    }
}
