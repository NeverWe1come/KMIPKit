use super::*;
use std::cell::Cell;

#[test]
fn validation_kinds_have_stable_open_numeric_codes() {
    assert_eq!(
        1,
        credential_validation_kind_code(CredentialValidationErrorKind::EmptyAuthentication)
    );
    assert_eq!(
        2,
        credential_validation_kind_code(CredentialValidationErrorKind::DuplicateField)
    );
    assert_eq!(
        3,
        credential_validation_kind_code(CredentialValidationErrorKind::MissingField)
    );
    assert_eq!(
        4,
        credential_validation_kind_code(CredentialValidationErrorKind::WrongFieldType)
    );
    assert_eq!(
        5,
        credential_validation_kind_code(CredentialValidationErrorKind::FieldOutOfOrder)
    );
    assert_eq!(
        6,
        credential_validation_kind_code(CredentialValidationErrorKind::InvalidTtlvStructure)
    );
}

#[test]
fn validation_error_detail_is_thread_local() {
    record_credential_validation_error_kind(3);
    let child = std::thread::spawn(|| {
        let initial = LAST_ERROR.with(Cell::get);
        record_credential_validation_error_kind(5);
        (
            initial.credential_validation_kind,
            LAST_ERROR.with(|last_error| last_error.get().credential_validation_kind),
        )
    });
    let (child_initial, child_recorded) = child.join().expect("test thread should complete");
    assert_eq!(0, child_initial);
    assert_eq!(5, child_recorded);
    assert_eq!(
        3,
        LAST_ERROR.with(|last_error| last_error.get().credential_validation_kind)
    );
}
