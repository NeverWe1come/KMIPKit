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
    let integer = item(UNIQUE_IDENTIFIER_TAG, Value::integer(42)).expect("valid identifier item");
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
