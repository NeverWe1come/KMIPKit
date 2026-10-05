    use kmipkit_ttlv::{RawTag, Structure, Tag, Value};

    fn tag(raw: u32) -> Tag {
        RawTag::new(raw)
            .expect("fixture tag fits")
            .try_checked()
            .expect("fixture tag is allocated")
    }

    fn item(raw: u32, value: Value) -> Item {
        Item::new(tag(raw), value).expect("fixture item is valid")
    }

    fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
        let mut result = Structure::new();
        for child in items {
            result.try_push(child).expect("fixture remains bounded");
        }
        result
    }

    fn version() -> Item {
        item(
            PROTOCOL_VERSION,
            Value::structure(structure([
                item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
                item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
            ])),
        )
    }

    fn request_header(count: i32) -> Structure {
        structure([version(), item(BATCH_COUNT, Value::integer(count))])
    }

    fn response_header(count: i32) -> Structure {
        structure([
            version(),
            item(TIME_STAMP, Value::date_time(1)),
            item(BATCH_COUNT, Value::integer(count)),
        ])
    }

    fn request_batch() -> Structure {
        structure([
            item(OPERATION, Value::enumeration(1)),
            item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![1])),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        ])
    }

    fn response_batch(status: u32) -> Structure {
        let mut fields = vec![item(RESULT_STATUS, Value::enumeration(status))];
        match status {
            1 => fields.push(item(RESULT_REASON, Value::enumeration(1))),
            2 => {
                fields.push(item(
                    ASYNCHRONOUS_CORRELATION_VALUE,
                    Value::byte_string(vec![0xA5]),
                ));
                fields.push(item(RESPONSE_PAYLOAD, Value::structure(Structure::new())));
            }
            _ => fields.push(item(RESPONSE_PAYLOAD, Value::structure(Structure::new()))),
        }
        structure(fields)
    }

    fn message_extension() -> Item {
        item(
            MESSAGE_EXTENSION,
            Value::structure(structure([
                item(
                    VENDOR_IDENTIFICATION,
                    Value::text_string("Vendor".to_owned()),
                ),
                item(CRITICALITY_INDICATOR, Value::boolean(false)),
                item(VENDOR_EXTENSION, Value::structure(Structure::new())),
            ])),
        )
    }

    fn assert_kind<T>(
        result: Result<T, MessageValidationError>,
        expected: MessageValidationErrorKind,
    ) {
        assert_eq!(result.err().expect("validation fails").kind(), expected);
    }

    #[test]
    fn every_validation_error_kind_has_safe_display_text() {
        let cases = [
            (
                MessageValidationErrorKind::MissingRequiredField,
                "required message field is missing",
            ),
            (
                MessageValidationErrorKind::DuplicateField,
                "message field is duplicated",
            ),
            (
                MessageValidationErrorKind::FieldOutOfOrder,
                "message field is out of order",
            ),
            (
                MessageValidationErrorKind::WrongItemType,
                "message field has the wrong TTLV type",
            ),
            (
                MessageValidationErrorKind::InvalidBatchCount,
                "message Batch Count is invalid",
            ),
            (
                MessageValidationErrorKind::BatchCountMismatch,
                "message Batch Count does not match its items",
            ),
            (
                MessageValidationErrorKind::InvalidSingleItemOption,
                "batch option is invalid for a single-item message",
            ),
            (
                MessageValidationErrorKind::InvalidResult,
                "response result fields are inconsistent",
            ),
            (
                MessageValidationErrorKind::InvalidFieldValue,
                "message field value is invalid",
            ),
        ];
        for (kind, expected) in cases {
            assert_eq!(kind.to_string(), expected);
        }

        let error = MessageValidationError::new(
            MessageValidationErrorKind::WrongItemType,
            Some(3),
            Some(7),
        );
        assert_eq!(error.kind(), MessageValidationErrorKind::WrongItemType);
        assert_eq!(error.structure_index(), Some(3));
        assert_eq!(error.field_index(), Some(7));
        assert_eq!(
            error.to_string(),
            "message field has the wrong TTLV type at structure item 3, field 7"
        );
        let unindexed = MessageValidationError::new(
            MessageValidationErrorKind::MissingRequiredField,
            None,
            None,
        );
        assert_eq!(unindexed.structure_index(), None);
        assert_eq!(unindexed.field_index(), None);
        assert_eq!(unindexed.to_string(), "required message field is missing");
    }

    #[test]
    fn field_schema_checks_type_order_singletons_required_fields_and_repetition() {
        let schema = [
            field(OPERATION, ItemType::Enumeration),
            repeated_field(ATTESTATION_TYPE, ItemType::Enumeration),
            field(RESULT_STATUS, ItemType::Enumeration),
        ];
        let valid = [
            item(OPERATION, Value::enumeration(1)),
            item(ATTESTATION_TYPE, Value::enumeration(2)),
            item(ATTESTATION_TYPE, Value::enumeration(3)),
            item(
                RESULT_REASON,
                Value::text_string("unknown schema field".to_owned()),
            ),
            item(RESULT_STATUS, Value::enumeration(0)),
        ];
        assert!(validate_fields(&valid, &schema, &[OPERATION, RESULT_STATUS], Some(4)).is_ok());

        assert_kind(
            validate_fields(
                &[item(OPERATION, Value::text_string("wrong".to_owned()))],
                &schema,
                &[OPERATION],
                Some(4),
            ),
            MessageValidationErrorKind::WrongItemType,
        );
        assert_kind(
            validate_fields(
                &[
                    item(ATTESTATION_TYPE, Value::enumeration(2)),
                    item(OPERATION, Value::enumeration(1)),
                ],
                &schema,
                &[],
                Some(4),
            ),
            MessageValidationErrorKind::FieldOutOfOrder,
        );
        assert_kind(
            validate_fields(
                &[
                    item(OPERATION, Value::enumeration(1)),
                    item(OPERATION, Value::enumeration(2)),
                ],
                &schema,
                &[],
                Some(4),
            ),
            MessageValidationErrorKind::DuplicateField,
        );
        assert_kind(
            validate_fields(&[], &schema, &[RESULT_STATUS], Some(4)),
            MessageValidationErrorKind::MissingRequiredField,
        );

        let type_error = validate_fields(
            &[item(OPERATION, Value::text_string("wrong".to_owned()))],
            &schema,
            &[],
            Some(4),
        )
        .expect_err("the Operation item type is constrained");
        assert_eq!(type_error.structure_index(), Some(4));
        assert_eq!(type_error.field_index(), Some(0));
    }

    #[test]
    fn protocol_version_and_typed_value_helpers_cover_valid_and_invalid_values() {
        let valid = structure([
            item(PROTOCOL_VERSION_MAJOR, Value::integer(2)),
            item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        ]);
        assert_eq!(
            validate_protocol_version(&valid.view())
                .expect("version valid")
                .major(),
            2
        );

        let missing_major = structure([item(PROTOCOL_VERSION_MINOR, Value::integer(1))]);
        assert_kind(
            validate_protocol_version(&missing_major.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_major = structure([
            item(PROTOCOL_VERSION_MAJOR, Value::text_string("2".to_owned())),
            item(PROTOCOL_VERSION_MINOR, Value::integer(1)),
        ]);
        assert_kind(
            validate_protocol_version(&wrong_major.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let missing_minor = structure([item(PROTOCOL_VERSION_MAJOR, Value::integer(2))]);
        assert_kind(
            validate_protocol_version(&missing_minor.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );

        let integer = item(OPERATION, Value::integer(7));
        assert_eq!(integer_value(&integer), Some(7));
        assert_eq!(enumeration_value(&integer), None);
        assert_eq!(byte_string_value(&integer), None);
        let enumeration = item(OPERATION, Value::enumeration(9));
        assert_eq!(enumeration_value(&enumeration), Some(9));
        assert_eq!(integer_value(&enumeration), None);
        let bytes = item(OPERATION, Value::byte_string(vec![1, 2]));
        assert_eq!(byte_string_value(&bytes), Some(vec![1, 2]));
    }

    #[test]
    fn private_field_helpers_return_categories_without_exposing_values() {
        let Err(missing) = required_child(&[], OPERATION, 6) else {
            panic!("required operation is missing");
        };
        assert_eq!(
            missing.kind(),
            MessageValidationErrorKind::MissingRequiredField
        );
        assert_eq!(missing.structure_index(), Some(6));
        assert_eq!(missing.field_index(), None);

        let not_a_structure = item(OPERATION, Value::integer(1));
        assert_kind(
            with_structure(&not_a_structure, 4, Some(2), |_| Ok(())),
            MessageValidationErrorKind::WrongItemType,
        );
    }

    #[test]
    fn request_header_validation_covers_required_fields_types_counts_and_options() {
        assert_eq!(
            validate_request_header(&request_header(2).view(), 0)
                .expect("valid request header")
                .batch_count,
            2
        );
        let missing_version = structure([item(BATCH_COUNT, Value::integer(1))]);
        assert_kind(
            validate_request_header(&missing_version.view(), 0),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let missing_count = structure([version()]);
        assert_kind(
            validate_request_header(&missing_count.view(), 0),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_count = structure([version(), item(BATCH_COUNT, Value::enumeration(1))]);
        assert_kind(
            validate_request_header(&wrong_count.view(), 0),
            MessageValidationErrorKind::WrongItemType,
        );
        let zero_count = request_header(0);
        assert_kind(
            validate_request_header(&zero_count.view(), 0),
            MessageValidationErrorKind::InvalidBatchCount,
        );
        let option = structure([
            version(),
            item(BATCH_ORDER_OPTION, Value::boolean(true)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_request_header(&option.view(), 0),
            MessageValidationErrorKind::InvalidSingleItemOption,
        );
        let duplicate = structure([
            version(),
            item(MAXIMUM_RESPONSE_SIZE, Value::integer(1)),
            item(MAXIMUM_RESPONSE_SIZE, Value::integer(2)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_request_header(&duplicate.view(), 0),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn response_header_validation_covers_required_fields_types_and_counts() {
        assert_eq!(
            validate_response_header(&response_header(1).view(), 0)
                .expect("valid response header")
                .time_stamp,
            1
        );
        let missing_time = structure([version(), item(BATCH_COUNT, Value::integer(1))]);
        assert_kind(
            validate_response_header(&missing_time.view(), 0),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_time = structure([
            version(),
            item(TIME_STAMP, Value::integer(1)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_header(&wrong_time.view(), 0),
            MessageValidationErrorKind::WrongItemType,
        );
        let wrong_count = structure([
            version(),
            item(TIME_STAMP, Value::date_time(1)),
            item(BATCH_COUNT, Value::boolean(true)),
        ]);
        assert_kind(
            validate_response_header(&wrong_count.view(), 0),
            MessageValidationErrorKind::WrongItemType,
        );
        let zero_count = response_header(0);
        assert_kind(
            validate_response_header(&zero_count.view(), 0),
            MessageValidationErrorKind::InvalidBatchCount,
        );
        let duplicate = structure([
            version(),
            item(TIME_STAMP, Value::date_time(1)),
            item(TIME_STAMP, Value::date_time(2)),
            item(BATCH_COUNT, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_header(&duplicate.view(), 0),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn request_batch_validation_covers_required_fields_ids_and_extensions() {
        let valid = request_batch();
        assert_eq!(
            validate_request_batch_item(&valid.view(), 1, 1).expect("batch valid"),
            Some(vec![1])
        );
        let identified = structure([
            item(OPERATION, Value::enumeration(1)),
            item(UNIQUE_BATCH_ITEM_ID, Value::byte_string(vec![3, 4])),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
            message_extension(),
        ]);
        assert_eq!(
            validate_request_batch_item(&identified.view(), 1, 2).expect("identified batch valid"),
            Some(vec![3, 4])
        );
        let missing_operation =
            structure([item(REQUEST_PAYLOAD, Value::structure(Structure::new()))]);
        assert_kind(
            validate_request_batch_item(&missing_operation.view(), 1, 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let missing_payload = structure([item(OPERATION, Value::enumeration(1))]);
        assert_kind(
            validate_request_batch_item(&missing_payload.view(), 1, 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let missing_id = structure([
            item(OPERATION, Value::enumeration(1)),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_request_batch_item(&missing_id.view(), 1, 2),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_id = structure([
            item(OPERATION, Value::enumeration(1)),
            item(UNIQUE_BATCH_ITEM_ID, Value::text_string("id".to_owned())),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_request_batch_item(&wrong_id.view(), 1, 1),
            MessageValidationErrorKind::WrongItemType,
        );

        let malformed_extension = structure([
            item(OPERATION, Value::enumeration(1)),
            item(REQUEST_PAYLOAD, Value::structure(Structure::new())),
            item(MESSAGE_EXTENSION, Value::integer(2)),
        ]);
        assert_kind(
            validate_request_batch_item(&malformed_extension.view(), 1, 1),
            MessageValidationErrorKind::WrongItemType,
        );
    }

    #[test]
    fn response_batch_validation_covers_result_invariants_and_extensions() {
        assert!(validate_response_batch_item(&response_batch(0).view(), 1).is_ok());
        assert!(validate_response_batch_item(&response_batch(1).view(), 1).is_ok());
        assert!(validate_response_batch_item(&response_batch(2).view(), 1).is_ok());
        assert!(validate_response_batch_item(&response_batch(u32::MAX).view(), 1).is_ok());

        let missing_status = Structure::new();
        assert_kind(
            validate_response_batch_item(&missing_status.view(), 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_status = structure([item(
            RESULT_STATUS,
            Value::text_string("success".to_owned()),
        )]);
        assert_kind(
            validate_response_batch_item(&wrong_status.view(), 1),
            MessageValidationErrorKind::WrongItemType,
        );
        let success_with_reason = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESULT_REASON, Value::enumeration(1)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&success_with_reason.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let failure_without_reason = structure([item(RESULT_STATUS, Value::enumeration(1))]);
        assert_kind(
            validate_response_batch_item(&failure_without_reason.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let failure_with_payload = structure([
            item(RESULT_STATUS, Value::enumeration(1)),
            item(RESULT_REASON, Value::enumeration(1)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&failure_with_payload.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let success_with_message = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESULT_MESSAGE, Value::text_string("not allowed".to_owned())),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&success_with_message.view(), 1),
            MessageValidationErrorKind::InvalidResult,
        );
        let pending_without_correlation = structure([
            item(RESULT_STATUS, Value::enumeration(2)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_response_batch_item(&pending_without_correlation.view(), 1),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let extension_on_response = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
            message_extension(),
        ]);
        assert!(validate_response_batch_item(&extension_on_response.view(), 1).is_ok());
        let bad_extension = structure([
            item(RESULT_STATUS, Value::enumeration(0)),
            item(RESPONSE_PAYLOAD, Value::structure(Structure::new())),
            item(MESSAGE_EXTENSION, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_batch_item(&bad_extension.view(), 1),
            MessageValidationErrorKind::WrongItemType,
        );
    }

    #[test]
    fn message_extension_validation_checks_required_fields_order_and_values() {
        let valid = structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("Vendor_1".to_owned()),
            ),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert!(validate_message_extension(&valid.view()).is_ok());

        let missing_vendor = structure([
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&missing_vendor.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let wrong_vendor_type = structure([
            item(VENDOR_IDENTIFICATION, Value::enumeration(1)),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&wrong_vendor_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let wrong_criticality_type = structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("Vendor".to_owned()),
            ),
            item(CRITICALITY_INDICATOR, Value::integer(0)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&wrong_criticality_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let empty_vendor = structure([
            item(VENDOR_IDENTIFICATION, Value::text_string(String::new())),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&empty_vendor.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
        let invalid_characters = structure([
            item(
                VENDOR_IDENTIFICATION,
                Value::text_string("Vendor name".to_owned()),
            ),
            item(CRITICALITY_INDICATOR, Value::boolean(false)),
            item(VENDOR_EXTENSION, Value::structure(Structure::new())),
        ]);
        assert_kind(
            validate_message_extension(&invalid_characters.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
    }

    #[test]
    fn request_envelope_validator_rejects_wrong_kinds_order_duplicates_and_absence() {
        let request = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(BATCH_ITEM, Value::structure(request_batch())),
        ]);
        assert!(validate_request_message(&request.view()).is_ok());
        let no_header = Structure::new();
        assert_kind(
            validate_request_message(&no_header.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let no_items = structure([item(REQUEST_HEADER, Value::structure(request_header(1)))]);
        assert_kind(
            validate_request_message(&no_items.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let batch_first = structure([
            item(BATCH_ITEM, Value::structure(request_batch())),
            item(REQUEST_HEADER, Value::structure(request_header(1))),
        ]);
        assert_kind(
            validate_request_message(&batch_first.view()),
            MessageValidationErrorKind::FieldOutOfOrder,
        );
        let response_header_in_request =
            structure([item(RESPONSE_HEADER, Value::structure(response_header(1)))]);
        assert_kind(
            validate_request_message(&response_header_in_request.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
        let wrong_header_type = structure([item(REQUEST_HEADER, Value::integer(1))]);
        assert_kind(
            validate_request_message(&wrong_header_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let duplicate_header = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(REQUEST_HEADER, Value::structure(request_header(1))),
        ]);
        assert_kind(
            validate_request_message(&duplicate_header.view()),
            MessageValidationErrorKind::DuplicateField,
        );
        let header_after_item = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(BATCH_ITEM, Value::structure(request_batch())),
            item(REQUEST_HEADER, Value::structure(request_header(1))),
        ]);
        assert_kind(
            validate_request_message(&header_after_item.view()),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn response_envelope_validator_rejects_wrong_kinds_order_duplicates_and_absence() {
        let response = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(BATCH_ITEM, Value::structure(response_batch(0))),
        ]);
        assert!(validate_response_message(&response.view()).is_ok());
        let no_header = Structure::new();
        assert_kind(
            validate_response_message(&no_header.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let no_items = structure([item(RESPONSE_HEADER, Value::structure(response_header(1)))]);
        assert_kind(
            validate_response_message(&no_items.view()),
            MessageValidationErrorKind::MissingRequiredField,
        );
        let batch_first = structure([
            item(BATCH_ITEM, Value::structure(response_batch(0))),
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
        ]);
        assert_kind(
            validate_response_message(&batch_first.view()),
            MessageValidationErrorKind::FieldOutOfOrder,
        );
        let request_header_in_response =
            structure([item(REQUEST_HEADER, Value::structure(request_header(1)))]);
        assert_kind(
            validate_response_message(&request_header_in_response.view()),
            MessageValidationErrorKind::InvalidFieldValue,
        );
        let wrong_header_type = structure([item(RESPONSE_HEADER, Value::integer(1))]);
        assert_kind(
            validate_response_message(&wrong_header_type.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let duplicate_header = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
        ]);
        assert_kind(
            validate_response_message(&duplicate_header.view()),
            MessageValidationErrorKind::DuplicateField,
        );
        let header_after_item = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(BATCH_ITEM, Value::structure(response_batch(0))),
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
        ]);
        assert_kind(
            validate_response_message(&header_after_item.view()),
            MessageValidationErrorKind::DuplicateField,
        );
    }

    #[test]
    fn envelope_count_and_batch_structure_edges_have_safe_categories() {
        let wrong_batch = structure([
            item(REQUEST_HEADER, Value::structure(request_header(1))),
            item(BATCH_ITEM, Value::integer(1)),
        ]);
        assert_kind(
            validate_request_message(&wrong_batch.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let wrong_response_batch = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(1))),
            item(BATCH_ITEM, Value::integer(1)),
        ]);
        assert_kind(
            validate_response_message(&wrong_response_batch.view()),
            MessageValidationErrorKind::WrongItemType,
        );
        let mismatch_request = structure([
            item(REQUEST_HEADER, Value::structure(request_header(2))),
            item(BATCH_ITEM, Value::structure(request_batch())),
        ]);
        assert_kind(
            validate_request_message(&mismatch_request.view()),
            MessageValidationErrorKind::BatchCountMismatch,
        );
        let mismatch_response = structure([
            item(RESPONSE_HEADER, Value::structure(response_header(2))),
            item(BATCH_ITEM, Value::structure(response_batch(0))),
        ]);
        assert_kind(
            validate_response_message(&mismatch_response.view()),
            MessageValidationErrorKind::BatchCountMismatch,
        );
        let empty_request = structure([item(REQUEST_HEADER, Value::structure(request_header(0)))]);
        assert_kind(
            validate_request_message(&empty_request.view()),
            MessageValidationErrorKind::InvalidBatchCount,
        );
        let empty_response =
            structure([item(RESPONSE_HEADER, Value::structure(response_header(0)))]);
        assert_kind(
            validate_response_message(&empty_response.view()),
            MessageValidationErrorKind::InvalidBatchCount,
        );
    }
