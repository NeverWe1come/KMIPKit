use kmipkit_ttlv::codec::{CodecLimits, DecodeErrorKind, LimitsError, decode, decode_with_limits};

fn empty_structure() -> [u8; 8] {
    [0x42, 0x01, 0x73, 0x01, 0, 0, 0, 0]
}

fn integer_item() -> [u8; 16] {
    [0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0, 0]
}

fn structure_with_one_empty_structure_child() -> Vec<u8> {
    [[0x42, 0x01, 0x73, 0x01, 0, 0, 0, 8], empty_structure()].concat()
}

fn assert_kind<T>(result: &Result<T, kmipkit_ttlv::codec::DecodeError>, expected: DecodeErrorKind) {
    assert_eq!(
        result
            .as_ref()
            .err()
            .map(kmipkit_ttlv::codec::DecodeError::kind),
        Some(expected)
    );
}

#[test]
fn public_limits_expose_default_and_explicit_read_only_values() {
    // KMIPKit per-call resource policy: KMIPKIT-0005-FR-007.
    let defaults = CodecLimits::defaults();
    assert_eq!(
        defaults.max_message_bytes(),
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES
    );
    assert_eq!(
        defaults.max_structure_depth(),
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH
    );
    assert_eq!(defaults.max_elements(), CodecLimits::DEFAULT_MAX_ELEMENTS);

    let configured = CodecLimits::new(4096, 12, 37).expect("configured depth is within the model");
    assert_eq!(configured.max_message_bytes(), 4096);
    assert_eq!(configured.max_structure_depth(), 12);
    assert_eq!(configured.max_elements(), 37);
}

#[test]
fn constructor_accepts_zero_and_model_depth_ceiling_and_rejects_one_over() {
    // Zero message/count limits and Structure-depth boundaries are project API policy: FR-007.
    let zero = CodecLimits::new(0, 0, 0).expect("zero limits are valid restrictive settings");
    assert_eq!(zero.max_message_bytes(), 0);
    assert_eq!(zero.max_structure_depth(), 0);
    assert_eq!(zero.max_elements(), 0);
    assert_kind(
        &decode_with_limits(&empty_structure(), &zero),
        DecodeErrorKind::MessageTooLarge,
    );

    let no_items = CodecLimits::new(8, 0, 0).expect("zero item limit is valid");
    assert_kind(
        &decode_with_limits(&empty_structure(), &no_items),
        DecodeErrorKind::ElementLimitExceeded,
    );

    let maximum = CodecLimits::new(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect("the model maximum depth is valid");
    assert_eq!(
        maximum.max_structure_depth(),
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH
    );

    let error = CodecLimits::new(
        CodecLimits::DEFAULT_MAX_MESSAGE_BYTES,
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH + 1,
        CodecLimits::DEFAULT_MAX_ELEMENTS,
    )
    .expect_err("depth one above the model limit must be rejected");
    let _: &LimitsError = &error;
    assert_eq!(error.to_string(), "Structure depth cannot exceed 64");
    assert!(!format!("{error:?}").contains("0x42"));
}

#[test]
fn decode_with_limits_enforces_configured_byte_boundary() {
    // Project resource policy only: KMIPKit FR-007.
    let limits = CodecLimits::new(8, 64, 100_000).expect("limit depth is valid");
    assert!(decode_with_limits(&empty_structure(), &limits).is_ok());
    assert_kind(
        &decode_with_limits(&integer_item(), &limits),
        DecodeErrorKind::MessageTooLarge,
    );
}

#[test]
fn decode_with_limits_enforces_configured_structure_depth_boundary() {
    // Project resource policy only: KMIPKit FR-007; root Structure depth is one.
    let limits = CodecLimits::new(128, 1, 100).expect("limit depth is valid");
    assert!(decode_with_limits(&empty_structure(), &limits).is_ok());
    assert_kind(
        &decode_with_limits(&structure_with_one_empty_structure_child(), &limits),
        DecodeErrorKind::StructureDepthExceeded,
    );

    let zero_depth = CodecLimits::new(128, 0, 100).expect("zero Structure depth is valid");
    assert!(decode_with_limits(&integer_item(), &zero_depth).is_ok());
    assert_kind(
        &decode_with_limits(&empty_structure(), &zero_depth),
        DecodeErrorKind::StructureDepthExceeded,
    );
}

#[test]
fn decode_with_limits_enforces_configured_item_count_boundary() {
    // Project resource policy only: KMIPKit FR-007; the root counts as an Item.
    let limits = CodecLimits::new(128, 64, 1).expect("limit depth is valid");
    assert!(decode_with_limits(&empty_structure(), &limits).is_ok());
    assert_kind(
        &decode_with_limits(&structure_with_one_empty_structure_child(), &limits),
        DecodeErrorKind::ElementLimitExceeded,
    );
}

#[test]
fn raised_message_limit_decodes_an_input_above_the_default_cap() {
    // Project resource policy only: KMIPKit FR-007. The bounded fixture is default + 8 bytes.
    let max_message = CodecLimits::DEFAULT_MAX_MESSAGE_BYTES;
    let value_length = max_message - 8 + 1;
    let padding_length = (8 - value_length % 8) % 8;
    let mut wire = Vec::with_capacity(8 + value_length + padding_length);
    let value_length_u32 = u32::try_from(value_length).expect("fixture fits the U32 Item Length");
    wire.extend_from_slice(&[0x42, 0x01, 0x73, 0x08]);
    wire.extend_from_slice(&value_length_u32.to_be_bytes());
    wire.resize(8 + value_length + padding_length, 0x5a);
    assert_eq!(wire.len(), max_message + 8);

    assert_kind(&decode(&wire), DecodeErrorKind::MessageTooLarge);
    let raised = CodecLimits::new(max_message + 8, 64, 100_000)
        .expect("raising the message limit is supported");
    assert!(decode_with_limits(&wire, &raised).is_ok());
}

#[test]
fn raised_item_limit_decodes_more_than_the_default_number_of_items() {
    // Project resource policy only: KMIPKit FR-007. The 800,008-byte fixture
    // contains one root plus 100,000 empty Structure children.
    let child_count = CodecLimits::DEFAULT_MAX_ELEMENTS;
    let structure_value_length = child_count * 8;
    let structure_value_length_u32 =
        u32::try_from(structure_value_length).expect("bounded fixture fits the U32 Item Length");
    let mut wire = Vec::with_capacity(8 + structure_value_length);
    wire.extend_from_slice(&[0x42, 0x01, 0x73, 0x01, 0, 0, 0, 0]);
    wire[4..8].copy_from_slice(&structure_value_length_u32.to_be_bytes());
    for _ in 0..child_count {
        wire.extend_from_slice(&empty_structure());
    }
    assert_eq!(wire.len(), 8 + structure_value_length);

    assert_kind(&decode(&wire), DecodeErrorKind::ElementLimitExceeded);
    let raised = CodecLimits::new(
        wire.len(),
        CodecLimits::DEFAULT_MAX_STRUCTURE_DEPTH,
        CodecLimits::DEFAULT_MAX_ELEMENTS + 1,
    )
    .expect("raising the Item count limit is supported");
    assert!(decode_with_limits(&wire, &raised).is_ok());
}
