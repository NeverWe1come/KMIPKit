use std::fmt;
use std::fmt::Write as _;

use super::{
    CLIENT_REQUIREMENTS, ProfileEvidenceRecord, ProfileReadiness, ProfileReadinessBlocker,
    ProfileTransport, ProfileValidationError, ProfileValidationErrorCode, ProfileValidationResult,
    ProfileValidationSelection, RuleConstraint, RuleEvaluator, RuleTestEvidence, RuleTestKind,
    TYPED_RULE_MAPPINGS, TypedRuleMapping,
    client_add_attribute_random_number_generator_is_well_formed,
    client_create_key_pair_domain_parameters_have_valid_wire_types,
    client_get_or_export_key_compression_type_is_enumeration,
    client_key_value_present_is_not_modified,
    client_key_wrapping_data_has_required_key_information,
    client_opaque_object_context_custom_attributes,
    client_register_does_not_specify_key_value_present,
    client_register_key_value_location_requires_omitted_key_value,
    client_register_opaque_object_has_required_data_fields,
    client_register_random_number_generator_is_well_formed,
    discover_versions_advertises_only_client_supported_versions,
    import_preserves_pgp_key_form_and_opaque_bytes, import_preserves_wrapped_secret_data_shape,
    protocol_version_is_kmip_2_1, reconcile_effective_state,
    type_specific_parameters_do_not_infer_remote_object_type, validate_mapping,
    validate_selection_with_registry,
};
use crate::profiles_generated::{
    GeneratedClientProfile, GeneratedClientRequirement, GeneratedVerificationCategory,
};
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value, ValueView};

const REQUEST_HEADER_TAG: u32 = 0x0042_0077;
const RESPONSE_HEADER_TAG: u32 = 0x0042_007A;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const MAXIMUM_RESPONSE_SIZE_TAG: u32 = 0x0042_0050;
const ATTESTATION_TYPE_TAG: u32 = 0x0042_00C7;
const MESSAGE_EXTENSION_TAG: u32 = 0x0042_0051;
const CRITICALITY_INDICATOR_TAG: u32 = 0x0042_0026;
const VENDOR_EXTENSION_TAG: u32 = 0x0042_009C;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
const BATCH_ERROR_CONTINUATION_OPTION_TAG: u32 = 0x0042_000E;
const ATTESTATION_CAPABLE_INDICATOR_TAG: u32 = 0x0042_00D3;
const BATCH_ORDER_OPTION_TAG: u32 = 0x0042_0010;
const OPERATION_TAG: u32 = 0x0042_005C;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const LOCATE_OPERATION_TAG: u32 = 0x0000_0008;
const DISCOVER_VERSIONS_OPERATION: u32 = 0x0000_001E;
const CREATE_SPLIT_KEY_OPERATION: u32 = 0x0000_0028;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const CLIENT_CORRELATION_VALUE_TAG: u32 = 0x0042_0105;
const UNIQUE_BATCH_ITEM_ID_TAG: u32 = 0x0042_0093;
const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const GET_OPERATION: u32 = 0x0000_000A;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;
const KEY_WRAPPING_SPECIFICATION_TAG: u32 = 0x0042_0047;
const ENCRYPTION_KEY_INFORMATION_TAG: u32 = 0x0042_0036;
const MAC_SIGNATURE_KEY_INFORMATION_TAG: u32 = 0x0042_004E;
const WRAPPING_METHOD_TAG: u32 = 0x0042_009E;
const KEY_VALUE_PRESENT_TAG: u32 = 0x0042_00BB;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const ISSUER_ALTERNATIVE_NAME_TAG: u32 = 0x0042_00B1;
const SUBJECT_ALTERNATIVE_NAME_TAG: u32 = 0x0042_00B3;
const SUBJECT_DISTINGUISHED_NAME_TAG: u32 = 0x0042_00B4;
const VALIDITY_DATE_TAG: u32 = 0x0042_009A;
const DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.14-006-002";
const DERIVE_KEY_ID_PLACEHOLDER_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.14-006";
const DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const DERIVE_KEY_REPEATED_IDENTIFIER_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.14-006-001";
const VARIABLE_IV_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.16-002";
const VARIABLE_IV_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.16-002";
const REGISTER_OPERATION_TAG: u32 = 0x0000_0003;
const DERIVE_KEY_OPERATION: u32 = 0x0000_0005;
const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
const OPAQUE_OBJECT_TAG: u32 = 0x0042_005B;
const OPAQUE_DATA_TYPE_TAG: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE_TAG: u32 = 0x0042_005A;
const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
const ATTRIBUTE_VALUE_TAG: u32 = 0x0042_000B;
const CRYPTOGRAPHIC_USAGE_MASK_TAG: u32 = 0x0042_002C;
const CRYPTOGRAPHIC_ALGORITHM_TAG: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH_TAG: u32 = 0x0042_002A;

struct RejectingWriter;

impl fmt::Write for RejectingWriter {
    fn write_str(&mut self, _value: &str) -> fmt::Result {
        Err(fmt::Error)
    }
}

static CONSTRAINTS_HTTPS: [RuleConstraint; 1] = [RuleConstraint {
    key: "transport",
    value: "https",
}];
static CONSTRAINTS_RAW_TLS: [RuleConstraint; 1] = [RuleConstraint {
    key: "transport",
    value: "raw_tls",
}];
static CONSTRAINTS_TLS: [RuleConstraint; 1] = [RuleConstraint {
    key: "transport_security",
    value: "tls",
}];
static CONSTRAINTS_TLS13: [RuleConstraint; 1] = [RuleConstraint {
    key: "transport_security",
    value: "tls13",
}];
static CONSTRAINTS_TTLV: [RuleConstraint; 1] = [RuleConstraint {
    key: "encoding",
    value: "ttlv",
}];
static UNKNOWN_CONSTRAINT: [RuleConstraint; 1] = [RuleConstraint {
    key: "untyped-policy",
    value: "opaque",
}];
static POSITIVE_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
    test_id: "profile-rule-positive-test",
    kind: RuleTestKind::PositiveBehavior,
}];
static NEGATIVE_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
    test_id: "profile-rule-negative-test",
    kind: RuleTestKind::NegativeBehavior,
}];
static RECOMMENDED_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
    test_id: "profile-rule-recommended-test",
    kind: RuleTestKind::RecommendedBehavior,
}];
static TLS13_RECOMMENDED_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
    test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_recommends_tls_1_3_for_basic_authentication_suite_transport",
    kind: RuleTestKind::RecommendedBehavior,
}];
static TLS_LEGACY_PROHIBITION_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
    test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_tls12_server_hello_before_request_dispatch",
    kind: RuleTestKind::NegativeBehavior,
}];
static TLS13_PROFILE_CIPHER_SUITES: [RuleConstraint; 1] = [RuleConstraint {
    key: "tls13_cipher_suites",
    value: "kmip_profiles_3_1_2",
}];
static TLS13_PROFILE_CIPHER_SUITE_SUPPORT_TESTS: [RuleTestEvidence; 2] = [
    RuleTestEvidence {
        test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_required_tls13_cipher_suite_support",
        kind: RuleTestKind::PositiveBehavior,
    },
    RuleTestEvidence {
        test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_tls12_server_hello_before_request_dispatch",
        kind: RuleTestKind::PositiveBehavior,
    },
];
static TLS13_UNLISTED_CIPHER_SUITE_PROHIBITION_TESTS: [RuleTestEvidence; 2] = [
    RuleTestEvidence {
        test_id: "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_prohibited_tls13_cipher_suites",
        kind: RuleTestKind::PositiveBehavior,
    },
    RuleTestEvidence {
        test_id: "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_tls12_server_hello_before_request_dispatch",
        kind: RuleTestKind::NegativeBehavior,
    },
];
static DISCOURAGED_ABSENCE_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
    test_id: "profile-rule-discouraged-absence-test",
    kind: RuleTestKind::DiscouragedBehaviorAbsent,
}];
static OPTIONAL_CAPABILITY_TEST: [RuleTestEvidence; 1] = [RuleTestEvidence {
    test_id: "profile-rule-optional-capability-test",
    kind: RuleTestKind::OptionalCapabilityUsable,
}];

fn passing_rule(
    _request: Option<&crate::RequestMessage>,
    _response: Option<(&crate::RequestMessage, &crate::ResponseMessage)>,
) -> bool {
    true
}

fn reject_outbound_rule(
    _request: Option<&crate::RequestMessage>,
    response: Option<(&crate::RequestMessage, &crate::ResponseMessage)>,
) -> bool {
    response.is_some()
}

fn reject_response_rule(
    _request: Option<&crate::RequestMessage>,
    response: Option<(&crate::RequestMessage, &crate::ResponseMessage)>,
) -> bool {
    response.is_none()
}

fn tagged(raw_tag: u32, value: Value) -> Item {
    let tag: Tag = RawTag::new(raw_tag)
        .expect("test tag fits the TTLV width")
        .try_checked()
        .expect("test tag is allocated");
    Item::new(tag, value).expect("test item is valid")
}

fn derive_key_id_placeholder_mapping() -> &'static TypedRuleMapping {
    TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID)
        .expect("Derive Key ID Placeholder prohibition has a typed profile mapping")
}

fn derive_key_repeated_identifiers_mapping() -> &'static TypedRuleMapping {
    TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == DERIVE_KEY_REPEATED_IDENTIFIER_REQUIREMENT_ID)
        .expect("Derive Key repeated Unique Identifier capability has a typed profile mapping")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for item in items {
        result
            .try_push(item)
            .expect("test structure depth is valid");
    }
    result
}

fn test_protocol_version() -> Item {
    tagged(
        PROTOCOL_VERSION_TAG,
        Value::structure(structure([
            tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
            tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
        ])),
    )
}

fn request_message() -> crate::RequestMessage {
    request_message_with_unique_batch_item_id(None)
}

fn request_message_with_attestation_capable_indicator(
    indicator: Option<bool>,
) -> crate::RequestMessage {
    let mut header_items = vec![test_protocol_version()];
    if let Some(indicator) = indicator {
        header_items.push(tagged(
            ATTESTATION_CAPABLE_INDICATOR_TAG,
            Value::boolean(indicator),
        ));
    }
    header_items.push(tagged(BATCH_COUNT_TAG, Value::integer(1)));

    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure(header_items)),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Request Message has an optional Attestation Capable Indicator")
}

fn request_message_with_unique_batch_item_id(
    unique_batch_item_id: Option<&[u8]>,
) -> crate::RequestMessage {
    let mut batch_item_fields = vec![tagged(OPERATION_TAG, Value::enumeration(0x1E))];
    if let Some(identifier) = unique_batch_item_id {
        batch_item_fields.push(tagged(
            UNIQUE_BATCH_ITEM_ID_TAG,
            Value::byte_string(identifier.to_vec()),
        ));
    }
    batch_item_fields.push(tagged(
        REQUEST_PAYLOAD_TAG,
        Value::structure(Structure::new()),
    ));

    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure(batch_item_fields)),
        ),
    ]))
    .expect("test single-item Request Message is valid")
}

fn request_message_with_extensions<'a>(
    extensions: impl IntoIterator<Item = (&'a str, bool)>,
) -> crate::RequestMessage {
    let mut batch_item_fields = vec![
        tagged(OPERATION_TAG, Value::enumeration(0x1E)),
        tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
    ];
    batch_item_fields.extend(extensions.into_iter().map(|(vendor, criticality)| {
        tagged(
            MESSAGE_EXTENSION_TAG,
            Value::structure(structure([
                tagged(
                    VENDOR_IDENTIFICATION_TAG,
                    Value::text_string(vendor.to_owned()),
                ),
                tagged(CRITICALITY_INDICATOR_TAG, Value::boolean(criticality)),
                tagged(VENDOR_EXTENSION_TAG, Value::structure(Structure::new())),
            ])),
        )
    }));

    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure(batch_item_fields)),
        ),
    ]))
    .expect("test Request Message with valid Message Extensions is valid")
}

fn request_message_with_batch_item_ids(ids: &[&[u8]]) -> crate::RequestMessage {
    let batch_count = i32::try_from(ids.len()).expect("test batch count fits in an Integer");
    let mut root_items = vec![tagged(
        REQUEST_HEADER_TAG,
        Value::structure(structure([
            test_protocol_version(),
            tagged(BATCH_COUNT_TAG, Value::integer(batch_count)),
        ])),
    )];

    root_items.extend(ids.iter().map(|identifier| {
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(
                    UNIQUE_BATCH_ITEM_ID_TAG,
                    Value::byte_string(identifier.to_vec()),
                ),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        )
    }));

    crate::RequestMessage::try_from_ttlv(structure(root_items))
        .expect("test multi-item Request Message has each required item ID")
}

fn request_message_with_batch_error_continuation_option(
    option: Option<u32>,
) -> crate::RequestMessage {
    let mut header_items = vec![test_protocol_version()];
    if let Some(option) = option {
        header_items.push(tagged(
            BATCH_ERROR_CONTINUATION_OPTION_TAG,
            Value::enumeration(option),
        ));
    }
    header_items.push(tagged(BATCH_COUNT_TAG, Value::integer(2)));

    let mut root_items = vec![tagged(
        REQUEST_HEADER_TAG,
        Value::structure(structure(header_items)),
    )];
    for identifier in [b"first".as_slice(), b"second".as_slice()] {
        root_items.push(tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(
                    UNIQUE_BATCH_ITEM_ID_TAG,
                    Value::byte_string(identifier.to_vec()),
                ),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ));
    }
    crate::RequestMessage::try_from_ttlv(structure(root_items))
        .expect("test multi-item Request Message has valid continuation option shape")
}

fn request_message_with_operation_payload(
    operation: u32,
    payload: Structure,
) -> crate::RequestMessage {
    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(operation)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(payload)),
            ])),
        ),
    ]))
    .expect("test Request Message with operation payload is valid")
}

fn get_request_with_repeated_wrapping_attribute_names() -> crate::RequestMessage {
    let wrapping_specification = structure([
        tagged(WRAPPING_METHOD_TAG, Value::enumeration(1)),
        tagged(
            ENCRYPTION_KEY_INFORMATION_TAG,
            Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("wrapping-key".to_owned()),
            )])),
        ),
        tagged(
            ATTRIBUTE_NAME_TAG,
            Value::text_string("Cryptographic Algorithm".to_owned()),
        ),
        tagged(
            ATTRIBUTE_NAME_TAG,
            Value::text_string("Cryptographic Length".to_owned()),
        ),
    ]);
    request_message_with_operation_payload(
        GET_OPERATION,
        structure([
            tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("wrapped-object".to_owned()),
            ),
            tagged(
                KEY_WRAPPING_SPECIFICATION_TAG,
                Value::structure(wrapping_specification),
            ),
        ]),
    )
}

fn get_request_with_wrapping_key_information(
    include_encryption: bool,
    include_mac_signature: bool,
) -> crate::RequestMessage {
    let mut wrapping_specification = vec![tagged(WRAPPING_METHOD_TAG, Value::enumeration(1))];
    if include_encryption {
        wrapping_specification.push(tagged(
            ENCRYPTION_KEY_INFORMATION_TAG,
            Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("encryption-key".to_owned()),
            )])),
        ));
    }
    if include_mac_signature {
        wrapping_specification.push(tagged(
            MAC_SIGNATURE_KEY_INFORMATION_TAG,
            Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("mac-key".to_owned()),
            )])),
        ));
    }
    request_message_with_operation_payload(
        GET_OPERATION,
        structure([
            tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("wrapped-object".to_owned()),
            ),
            tagged(
                KEY_WRAPPING_SPECIFICATION_TAG,
                Value::structure(structure(wrapping_specification)),
            ),
        ]),
    )
}

fn request_message_with_optional_maximum_response_size(
    operation: u32,
    maximum_response_size: Option<i32>,
) -> crate::RequestMessage {
    let mut header_items = vec![test_protocol_version()];
    if let Some(size) = maximum_response_size {
        header_items.push(tagged(MAXIMUM_RESPONSE_SIZE_TAG, Value::integer(size)));
    }
    header_items.push(tagged(BATCH_COUNT_TAG, Value::integer(1)));

    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure(header_items)),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(operation)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Request Message has an optional Maximum Response Size")
}

fn request_message_with_batch_operations(
    operations: impl IntoIterator<Item = (u32, Structure)>,
) -> crate::RequestMessage {
    request_message_with_batch_operations_and_order(operations, None)
}

fn request_message_with_batch_operations_and_order(
    operations: impl IntoIterator<Item = (u32, Structure)>,
    batch_order_option: Option<bool>,
) -> crate::RequestMessage {
    let operations: Vec<_> = operations.into_iter().collect();
    let batch_count = i32::try_from(operations.len()).expect("test batch count fits in an Integer");
    let mut header_items = vec![test_protocol_version()];
    if let Some(value) = batch_order_option {
        header_items.push(tagged(BATCH_ORDER_OPTION_TAG, Value::boolean(value)));
    }
    header_items.push(tagged(BATCH_COUNT_TAG, Value::integer(batch_count)));
    let mut root_items = vec![tagged(
        REQUEST_HEADER_TAG,
        Value::structure(structure(header_items)),
    )];

    root_items.extend(
        operations
            .into_iter()
            .enumerate()
            .map(|(index, (operation, payload))| {
                tagged(
                    BATCH_ITEM_TAG,
                    Value::structure(structure([
                        tagged(OPERATION_TAG, Value::enumeration(operation)),
                        tagged(
                            UNIQUE_BATCH_ITEM_ID_TAG,
                            Value::byte_string(index.to_be_bytes().to_vec()),
                        ),
                        tagged(REQUEST_PAYLOAD_TAG, Value::structure(payload)),
                    ])),
                )
            }),
    );

    crate::RequestMessage::try_from_ttlv(structure(root_items))
        .expect("test multi-item Request Message has valid batch item identifiers")
}

fn register_key_value_location_request(
    include_location: bool,
    include_key_value: bool,
) -> crate::RequestMessage {
    const KEY_VALUE_LOCATION: u32 = 0x0042_00B8;
    const KEY_VALUE_LOCATION_VALUE: u32 = 0x0042_00B9;
    const KEY_VALUE_LOCATION_TYPE: u32 = 0x0042_00BA;

    let location = include_location.then(|| {
        tagged(
            KEY_VALUE_LOCATION,
            Value::structure(structure([
                tagged(
                    KEY_VALUE_LOCATION_VALUE,
                    Value::text_string("pkcs11:object=key-1".to_owned()),
                ),
                tagged(KEY_VALUE_LOCATION_TYPE, Value::enumeration(2)),
            ])),
        )
    });
    register_request_with_key_value_location(location, include_key_value)
}

fn register_request_with_key_value_location(
    location: Option<Item>,
    include_key_value: bool,
) -> crate::RequestMessage {
    const REGISTER_OPERATION: u32 = 0x0000_0003;
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const SYMMETRIC_KEY: u32 = 0x0042_008F;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_VALUE: u32 = 0x0042_0045;
    const KEY_MATERIAL: u32 = 0x0042_0044;

    let mut key_block = vec![tagged(KEY_FORMAT_TYPE, Value::enumeration(1))];
    if include_key_value {
        key_block.push(tagged(
            KEY_VALUE,
            Value::structure(structure([tagged(
                KEY_MATERIAL,
                Value::byte_string(b"synthetic-key-material".to_vec()),
            )])),
        ));
    }

    let attributes = location.into_iter().collect::<Vec<_>>();

    request_message_with_operation_payload(
        REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE, Value::enumeration(2)),
            tagged(ATTRIBUTES, Value::structure(structure(attributes))),
            tagged(
                SYMMETRIC_KEY,
                Value::structure(structure([tagged(
                    KEY_BLOCK,
                    Value::structure(structure(key_block)),
                )])),
            ),
        ]),
    )
}

fn register_wrapped_key_value_request(encoding_option: Value) -> crate::RequestMessage {
    const REGISTER_OPERATION: u32 = 0x0000_0003;
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const SYMMETRIC_KEY: u32 = 0x0042_008F;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_VALUE: u32 = 0x0042_0045;
    const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
    const WRAPPING_METHOD: u32 = 0x0042_009E;
    const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const ENCODING_OPTION: u32 = 0x0042_00A3;
    const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
    const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;

    request_message_with_operation_payload(
        REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE, Value::enumeration(2)),
            tagged(
                ATTRIBUTES,
                Value::structure(structure([
                    tagged(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                    tagged(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
                ])),
            ),
            tagged(
                SYMMETRIC_KEY,
                Value::structure(structure([tagged(
                    KEY_BLOCK,
                    Value::structure(structure([
                        tagged(KEY_FORMAT_TYPE, Value::enumeration(1)),
                        tagged(
                            KEY_VALUE,
                            Value::byte_string(b"test-only wrapped TTLV bytes".to_vec()),
                        ),
                        tagged(
                            KEY_WRAPPING_DATA,
                            Value::structure(structure([
                                tagged(WRAPPING_METHOD, Value::enumeration(1)),
                                tagged(
                                    ENCRYPTION_KEY_INFORMATION,
                                    Value::structure(structure([tagged(
                                        UNIQUE_IDENTIFIER,
                                        Value::text_string("wrapping-key".to_owned()),
                                    )])),
                                ),
                                tagged(ENCODING_OPTION, encoding_option),
                            ])),
                        ),
                    ])),
                )])),
            ),
        ]),
    )
}

fn discover_versions_payload(versions: &[(i32, i32)]) -> Structure {
    let mut payload = Structure::new();
    for &(major, minor) in versions {
        payload
            .try_push(tagged(
                PROTOCOL_VERSION_TAG,
                Value::structure(structure([
                    tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(major)),
                    tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(minor)),
                ])),
            ))
            .expect("test Discover Versions payload remains valid");
    }
    payload
}

fn response_message() -> crate::ResponseMessage {
    crate::ResponseMessage::try_from_ttlv(structure([
        tagged(
            RESPONSE_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(TIME_STAMP_TAG, Value::date_time(1)),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(RESULT_STATUS_TAG, Value::enumeration(0)),
                tagged(RESPONSE_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Response Message is valid")
}

fn get_attributes_x509_response() -> crate::ResponseMessage {
    crate::ResponseMessage::try_from_ttlv(structure([
        tagged(
            RESPONSE_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(TIME_STAMP_TAG, Value::date_time(1)),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(GET_ATTRIBUTES_OPERATION)),
                tagged(RESULT_STATUS_TAG, Value::enumeration(0)),
                tagged(
                    RESPONSE_PAYLOAD_TAG,
                    Value::structure(structure([
                        tagged(
                            UNIQUE_IDENTIFIER_TAG,
                            Value::text_string("x509-object".to_owned()),
                        ),
                        tagged(
                            ATTRIBUTES_TAG,
                            Value::structure(structure([
                                tagged(
                                    ISSUER_ALTERNATIVE_NAME_TAG,
                                    Value::byte_string(b"issuer-name-a".to_vec()),
                                ),
                                tagged(
                                    SUBJECT_DISTINGUISHED_NAME_TAG,
                                    Value::byte_string(Vec::new()),
                                ),
                                tagged(
                                    SUBJECT_ALTERNATIVE_NAME_TAG,
                                    Value::byte_string(b"subject-name-a".to_vec()),
                                ),
                                tagged(
                                    ISSUER_ALTERNATIVE_NAME_TAG,
                                    Value::byte_string(b"issuer-name-b".to_vec()),
                                ),
                                tagged(
                                    SUBJECT_ALTERNATIVE_NAME_TAG,
                                    Value::byte_string(b"subject-name-b".to_vec()),
                                ),
                            ])),
                        ),
                    ])),
                ),
            ])),
        ),
    ]))
    .expect("test Get Attributes response carries valid X.509 Byte String attributes")
}

fn opaque_context_vendor_attribute(value: &[u8]) -> Item {
    tagged(
        VENDOR_ATTRIBUTE_TAG,
        Value::structure(structure([
            tagged(
                VENDOR_IDENTIFICATION_TAG,
                Value::text_string("example.invalid".to_owned()),
            ),
            tagged(
                ATTRIBUTE_NAME_TAG,
                Value::text_string("KMIPKit.Context".to_owned()),
            ),
            tagged(ATTRIBUTE_VALUE_TAG, Value::byte_string(value.to_vec())),
        ])),
    )
}

fn opaque_object_register_payload(include_context: bool) -> Structure {
    let mut attributes = crate::AttributeSet::new();
    if include_context {
        attributes
            .try_push(opaque_context_vendor_attribute(b"context-a"))
            .expect("the first synthetic Custom Attribute uses a valid Vendor Attribute shape");
        attributes
            .try_push(opaque_context_vendor_attribute(b"context-b"))
            .expect("a repeated synthetic Custom Attribute retains its value");
    }

    crate::RegisterRequest::new(
        crate::ObjectType::from_raw(8),
        attributes,
        tagged(
            OPAQUE_OBJECT_TAG,
            Value::structure(structure([
                tagged(OPAQUE_DATA_TYPE_TAG, Value::enumeration(0xF123_4567)),
                tagged(
                    OPAQUE_DATA_VALUE_TAG,
                    Value::byte_string(b"synthetic-opaque-object-data".to_vec()),
                ),
            ])),
        ),
    )
    .into_ttlv_payload()
    .expect("the Opaque Object and its optional Attributes have valid TTLV shapes")
}

fn raw_opaque_object_register_payload(opaque_object: Value) -> Structure {
    structure([
        tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
        tagged(ATTRIBUTES_TAG, Value::structure(structure([]))),
        tagged(OPAQUE_OBJECT_TAG, opaque_object),
    ])
}

fn managed_opaque_object_register_payload() -> Structure {
    let mut attributes = crate::AttributeSet::new();
    for attribute in [
        tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3)),
        tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::enumeration(3)),
        tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::integer(256)),
    ] {
        attributes
            .try_push(attribute)
            .expect("managed Opaque Object attributes have valid TTLV shapes");
    }

    crate::RegisterRequest::new(
        crate::ObjectType::from_raw(8),
        attributes,
        tagged(
            OPAQUE_OBJECT_TAG,
            Value::structure(structure([
                tagged(OPAQUE_DATA_TYPE_TAG, Value::enumeration(7)),
                tagged(
                    OPAQUE_DATA_VALUE_TAG,
                    Value::byte_string(b"managed-opaque-object-data".to_vec()),
                ),
            ])),
        ),
    )
    .with_opaque_object_context(crate::OpaqueObjectContext::ManagedCryptographicObject)
    .into_ttlv_payload()
    .expect("managed Opaque Object attributes serialize without adding a context marker")
}

fn managed_opaque_register_payload_with_attributes(attributes: Structure) -> Structure {
    structure([
        tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
        tagged(ATTRIBUTES_TAG, Value::structure(attributes)),
        tagged(
            OPAQUE_OBJECT_TAG,
            Value::structure(structure([
                tagged(OPAQUE_DATA_TYPE_TAG, Value::enumeration(7)),
                tagged(
                    OPAQUE_DATA_VALUE_TAG,
                    Value::byte_string(b"managed-opaque-object-data".to_vec()),
                ),
            ])),
        ),
    ])
}

fn opaque_context_get_attributes_response() -> crate::ResponseMessage {
    crate::ResponseMessage::try_from_ttlv(structure([
        tagged(
            RESPONSE_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(TIME_STAMP_TAG, Value::date_time(1)),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(GET_ATTRIBUTES_OPERATION)),
                tagged(RESULT_STATUS_TAG, Value::enumeration(0)),
                tagged(
                    RESPONSE_PAYLOAD_TAG,
                    Value::structure(structure([
                        tagged(
                            UNIQUE_IDENTIFIER_TAG,
                            Value::text_string("opaque-object".to_owned()),
                        ),
                        tagged(
                            ATTRIBUTES_TAG,
                            Value::structure(structure([
                                opaque_context_vendor_attribute(b"context-a"),
                                opaque_context_vendor_attribute(b"context-b"),
                            ])),
                        ),
                    ])),
                ),
            ])),
        ),
    ]))
    .expect("the Get Attributes response preserves repeated Vendor Attributes")
}

fn get_attributes_response_with_payload(
    operation: u32,
    payload: Structure,
) -> crate::ResponseMessage {
    crate::ResponseMessage::try_from_ttlv(structure([
        tagged(
            RESPONSE_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(TIME_STAMP_TAG, Value::date_time(1)),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(operation)),
                tagged(RESULT_STATUS_TAG, Value::enumeration(0)),
                tagged(RESPONSE_PAYLOAD_TAG, Value::structure(payload)),
            ])),
        ),
    ]))
    .expect("test Get Attributes response payload has a valid message envelope")
}

fn opaque_object_attributes_ttlv(attributes: impl IntoIterator<Item = Item>) -> Item {
    tagged(ATTRIBUTES_TAG, Value::structure(structure(attributes)))
}

fn malformed_opaque_register_metadata_payloads() -> Vec<Structure> {
    let valid_attributes =
        || opaque_object_attributes_ttlv([opaque_context_vendor_attribute(b"context")]);
    let invalid_vendor_attribute = tagged(
        VENDOR_ATTRIBUTE_TAG,
        Value::text_string("not a Vendor Attribute structure".to_owned()),
    );

    vec![
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
            valid_attributes(),
            valid_attributes(),
        ]),
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
            tagged(
                ATTRIBUTES_TAG,
                Value::text_string("not a Structure".to_owned()),
            ),
        ]),
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
            opaque_object_attributes_ttlv([invalid_vendor_attribute]),
        ]),
    ]
}

fn vendor_attribute_value_bytes(attribute: &Item) -> Option<Vec<u8>> {
    attribute.with_value(|value| {
        let ValueView::Structure(vendor_attribute) = value else {
            return None;
        };
        vendor_attribute
            .children()
            .iter()
            .find(|field| field.tag().raw() == ATTRIBUTE_VALUE_TAG)
            .and_then(|field| {
                field.with_value(|value| match value {
                    ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                    _ => None,
                })
            })
    })
}

fn profile(
    profile_id: &'static str,
    requirement_ids: &'static [&'static str],
    dependency_profile_ids: &'static [&'static str],
) -> GeneratedClientProfile {
    GeneratedClientProfile {
        profile_id,
        name: "synthetic test profile",
        applicability: "client_1_0",
        claim_state: "not_claimed",
        selected_target: false,
        direct_source_clause_ids: &["TEST-CLAUSE"],
        source_clause_ids: &["TEST-CLAUSE"],
        requirement_ids,
        unresolved_scope_obligations: &[],
        dependency_profile_ids,
        transport_requirements: &[],
        encoding_requirements: &[],
        test_cases: &[],
    }
}

fn requirement(
    requirement_id: &'static str,
    category: GeneratedVerificationCategory,
    has_accepted_deviation: bool,
) -> GeneratedClientRequirement {
    GeneratedClientRequirement {
        requirement_id,
        source_keyword: Some("SHALL"),
        source_force: None,
        source_gap_id: None,
        source_locator: None,
        verification_category: category,
        scope_state: "client_1_0",
        source_clause_ids: &["TEST-CLAUSE"],
        profile_ids: &[],
        official_test_case_ids: &[],
        has_accepted_deviation,
    }
}

fn mapping(
    requirement_id: &'static str,
    category: GeneratedVerificationCategory,
    test_kind: RuleTestKind,
    constraints: &'static [RuleConstraint],
) -> TypedRuleMapping {
    let test_evidence = match test_kind {
        RuleTestKind::PositiveBehavior => &POSITIVE_TEST,
        RuleTestKind::NegativeBehavior => &NEGATIVE_TEST,
        RuleTestKind::RecommendedBehavior => &RECOMMENDED_TEST,
        RuleTestKind::DiscouragedBehaviorAbsent => &DISCOURAGED_ABSENCE_TEST,
        RuleTestKind::OptionalCapabilityUsable => &OPTIONAL_CAPABILITY_TEST,
    };
    TypedRuleMapping {
        requirement_id,
        source_clause_id: "TEST-CLAUSE",
        source_gap_id: None,
        verification_category: category,
        evaluator: Some(passing_rule),
        test_evidence,
        constraints,
    }
}

#[test]
fn each_keyword_category_requires_the_matching_test_form() {
    let cases = [
        (
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
        ),
        (
            GeneratedVerificationCategory::ProhibitedBehavior,
            RuleTestKind::NegativeBehavior,
        ),
        (
            GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
            RuleTestKind::RecommendedBehavior,
        ),
        (
            GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation,
            RuleTestKind::DiscouragedBehaviorAbsent,
        ),
        (
            GeneratedVerificationCategory::OptionalCapability,
            RuleTestKind::OptionalCapabilityUsable,
        ),
    ];

    for (category, test_kind) in cases {
        let requirement = requirement("TEST-REQ", category, false);
        let good_mapping = mapping("TEST-REQ", category, test_kind, &[]);
        assert!(validate_mapping("TEST-PROFILE", &requirement, &good_mapping).is_ok());

        let wrong_test_kind = match test_kind {
            RuleTestKind::PositiveBehavior => RuleTestKind::NegativeBehavior,
            _ => RuleTestKind::PositiveBehavior,
        };
        let mapping = mapping("TEST-REQ", category, wrong_test_kind, &[]);
        let error = validate_mapping("TEST-PROFILE", &requirement, &mapping)
            .expect_err("a different test category cannot cover the rule");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::MissingCategoryTest
        );
    }
}

#[test]
fn production_discover_versions_rule_accepts_only_kmip_21_offers() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.16-001-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Discover Versions capability has a production typed rule mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("Discover Versions capability resolves to generated normative metadata");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the production rule matches its generated clause and optional-test category");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    assert!(!discover_versions_advertises_only_client_supported_versions(None, None));

    let supported_request = request_message_with_operation_payload(
        0x0000_001E,
        crate::DiscoverVersionsRequest::new()
            .to_ttlv_payload()
            .expect("the typed request advertises KMIP 2.1"),
    );
    assert!(result.validate_request(&supported_request).is_ok());

    let omitted_optional_list =
        request_message_with_operation_payload(0x0000_001E, Structure::new());
    assert!(result.validate_request(&omitted_optional_list).is_ok());

    let other_operation = request_message_with_operation_payload(0x0000_0001, Structure::new());
    assert!(result.validate_request(&other_operation).is_ok());

    let malformed_version = request_message_with_operation_payload(
        0x0000_001E,
        structure([tagged(
            PROTOCOL_VERSION_TAG,
            Value::text_string("2.1".to_owned()),
        )]),
    );
    assert!(result.validate_request(&malformed_version).is_err());

    let unsupported_request =
        request_message_with_operation_payload(0x0000_001E, discover_versions_payload(&[(3, 0)]));
    let error = result
        .validate_request(&unsupported_request)
        .expect_err("a 1.0 client must not advertise a protocol version it does not support");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.profile_id(), Some(BASELINE_PROFILE_ID));
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-6.1.16-001")
    );

    let mixed_versions = request_message_with_operation_payload(
        0x0000_001E,
        discover_versions_payload(&[(2, 1), (3, 0)]),
    );
    assert!(result.validate_request(&mixed_versions).is_err());
}

#[test]
fn profile_maps_derive_key_unique_identifiers_exclude_id_placeholder() {
    // KMIP Specification v2.1 §6.1.14 Table 205 prohibits using the current
    // ID Placeholder in place of a Derive Key Unique Identifier.
    let mapping = derive_key_id_placeholder_mapping();
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID)
        .expect("Derive Key ID Placeholder prohibition resolves to normative metadata");
    assert_eq!(
        mapping.source_clause_id,
        DERIVE_KEY_ID_PLACEHOLDER_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::ProhibitedBehavior
    );
    validate_mapping(DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID, requirement, mapping)
        .expect("the prohibition has category-matched negative evidence");

    let result = ProfileValidationResult {
        profile_ids: vec![DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID.to_owned()],
        requirement_ids: vec![DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID.to_owned()],
    };
    let omitted_identifier =
        request_message_with_operation_payload(DERIVE_KEY_OPERATION, Structure::new());
    assert_request_rejected_for_requirement(
        &result,
        &omitted_identifier,
        DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID,
        DERIVE_KEY_ID_PLACEHOLDER_SOURCE_CLAUSE_ID,
        "Derive Key must specify a source identifier instead of using the ID Placeholder",
    );

    let unrelated_operation = request_message_with_operation_payload(0x0000_000A, Structure::new());
    result
        .validate_request(&unrelated_operation)
        .expect("the Derive Key prohibition does not constrain other operations");
}

#[test]
fn profile_maps_optional_repeated_derive_key_unique_identifiers() {
    // KMIP Specification v2.1 §6.1.14 Table 205 permits repeated source IDs.
    let mapping = derive_key_repeated_identifiers_mapping();
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| {
            requirement.requirement_id == DERIVE_KEY_REPEATED_IDENTIFIER_REQUIREMENT_ID
        })
        .expect("the repeated Derive Key identifier requirement resolves to normative metadata");
    assert_eq!(
        mapping.source_clause_id,
        DERIVE_KEY_ID_PLACEHOLDER_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID, requirement, mapping)
        .expect("the repeated identifier capability has usable positive evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id
            == "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_optional_repeated_derive_key_unique_identifiers"
            && evidence.kind == RuleTestKind::OptionalCapabilityUsable
    }));

    let result = ProfileValidationResult {
        profile_ids: vec![DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID.to_owned()],
        requirement_ids: vec![DERIVE_KEY_REPEATED_IDENTIFIER_REQUIREMENT_ID.to_owned()],
    };
    let repeated_identifiers = request_message_with_operation_payload(
        DERIVE_KEY_OPERATION,
        structure([
            tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("source-key-1".to_owned()),
            ),
            tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("source-key-2".to_owned()),
            ),
        ]),
    );
    result
        .validate_request(&repeated_identifiers)
        .expect("repeated explicit source identifiers remain valid");
}

#[test]
fn profile_maps_derive_key_unique_identifiers_rejects_missing_request_context() {
    let mapping = derive_key_id_placeholder_mapping();
    let result = ProfileValidationResult {
        profile_ids: vec![DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID.to_owned()],
        requirement_ids: vec![DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID.to_owned()],
    };

    let error = result
        .evaluate_rules(
            None,
            None,
            std::slice::from_ref(mapping),
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect_err("the mapped rule rejects a missing outbound request");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.profile_id(),
        Some(DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID)
    );
    assert_eq!(
        error.requirement_id(),
        Some(DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(DERIVE_KEY_ID_PLACEHOLDER_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_maps_required_derive_key_cryptographic_length() {
    // KMIP Specification v2.1 §6.1.14 Table 205 requires the desired output
    // length; §4.15 Table 57 defines Cryptographic Length as an Integer.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.14-001-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.14-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the required Derive Key output length has an executable profile mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Derive Key output length resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the production rule has category-matched conformance evidence");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for object_type in [2, 7] {
        let request = request_message_with_operation_payload(
            0x0000_0005,
            derive_key_payload(object_type, [Value::integer(256)]),
        );
        result
            .validate_request(&request)
            .expect("symmetric-key and Secret Data requests include an Integer length");
    }

    for malformed in [
        derive_key_payload(2, []),
        derive_key_payload(2, [Value::enumeration(256)]),
        derive_key_payload(2, [Value::integer(128), Value::integer(256)]),
    ] {
        let request = request_message_with_operation_payload(0x0000_0005, malformed);
        assert_request_rejected_for_requirement(
            &result,
            &request,
            REQUIREMENT_ID,
            SOURCE_CLAUSE_ID,
            "Derive Key cannot omit, mistype, or duplicate Cryptographic Length",
        );
    }

    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    result
        .validate_request(&unrelated_operation)
        .expect("the Derive Key rule does not constrain other operations");
}

#[test]
fn profile_maps_required_derive_key_cryptographic_algorithm_for_key_results() {
    // KMIP Specification v2.1 §6.1.14 requires Cryptographic Algorithm when
    // Derive Key creates a key; this condition does not apply to Secret Data.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.14-001-003";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.14-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Derive Key algorithm requirement has an executable profile mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Derive Key algorithm requirement resolves to normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the production rule has category-matched conformance evidence");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let key_with_algorithm = derive_key_payload_with_attributes(
        2,
        [
            tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::integer(256)),
            tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::enumeration(3)),
        ],
    );
    let valid_key = request_message_with_operation_payload(0x0000_0005, key_with_algorithm);
    result
        .validate_request(&valid_key)
        .expect("key derivation includes the required algorithm enumeration");

    for malformed in [
        derive_key_payload(2, [Value::integer(256)]),
        derive_key_payload_with_attributes(
            2,
            [
                tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::integer(256)),
                tagged(
                    CRYPTOGRAPHIC_ALGORITHM_TAG,
                    Value::text_string("AES".to_owned()),
                ),
            ],
        ),
        derive_key_payload_with_attributes(
            2,
            [
                tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::integer(256)),
                tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::enumeration(3)),
                tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::enumeration(4)),
            ],
        ),
    ] {
        let request = request_message_with_operation_payload(0x0000_0005, malformed);
        assert_request_rejected_for_requirement(
            &result,
            &request,
            REQUIREMENT_ID,
            SOURCE_CLAUSE_ID,
            "key derivation rejects a missing, mistyped, or duplicate algorithm",
        );
    }

    let secret_data = request_message_with_operation_payload(
        0x0000_0005,
        derive_key_payload(7, [Value::integer(512)]),
    );
    result
        .validate_request(&secret_data)
        .expect("the key-only algorithm requirement does not apply to Secret Data");
}

#[test]
fn profile_maps_client_optional_derive_key_secret_data_total_length() {
    // KMIP Specification v2.1 §6.1.14 permits deriving multiple keys and IVs
    // through a Secret Data result whose Cryptographic Length is the total.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.14-001-004";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.14-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional Secret Data derivation capability has a typed profile mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Secret Data derivation capability resolves to normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the production rule has category-matched optional-capability evidence");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let secret_data = request_message_with_operation_payload(
        0x0000_0005,
        derive_key_payload(7, [Value::integer(512)]),
    );
    result
        .validate_request(&secret_data)
        .expect("the client may request a Secret Data result with the total output length");

    let symmetric_key = request_message_with_operation_payload(
        0x0000_0005,
        derive_key_payload_with_attributes(
            2,
            [
                tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::integer(256)),
                tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::enumeration(3)),
            ],
        ),
    );
    result
        .validate_request(&symmetric_key)
        .expect("the optional Secret Data capability does not restrict key derivation");

    let attributes_with_length = || {
        tagged(
            ATTRIBUTES_TAG,
            Value::structure(structure([tagged(
                CRYPTOGRAPHIC_LENGTH_TAG,
                Value::integer(512),
            )])),
        )
    };
    for malformed in [
        structure([attributes_with_length()]),
        structure([
            tagged(
                OBJECT_TYPE_TAG,
                Value::text_string("Secret Data".to_owned()),
            ),
            attributes_with_length(),
        ]),
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(7)),
            tagged(OBJECT_TYPE_TAG, Value::enumeration(7)),
            attributes_with_length(),
        ]),
    ] {
        let request = request_message_with_operation_payload(0x0000_0005, malformed);
        assert_request_rejected_for_requirement(
            &result,
            &request,
            REQUIREMENT_ID,
            SOURCE_CLAUSE_ID,
            "Secret Data derivation needs one well-typed Object Type",
        );
    }

    let secret_data_without_length =
        request_message_with_operation_payload(0x0000_0005, derive_key_payload(7, []));
    assert_request_rejected_for_requirement(
        &result,
        &secret_data_without_length,
        REQUIREMENT_ID,
        SOURCE_CLAUSE_ID,
        "Secret Data derivation must carry its total Cryptographic Length",
    );
}

#[test]
fn profile_maps_required_ttlv_text_and_byte_string_padding() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-10.1.5-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-10.1.5-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    assert_required_configuration_mapping(
        REQUIREMENT_ID,
        SOURCE_CLAUSE_ID,
        "encoding",
        "ttlv",
        &[
            "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_byte_string_golden_vector_with_minimum_following_padding",
            "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_text_string_golden_vector_with_minimum_following_padding",
            "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::text_and_byte_string_padding_is_minimal_at_eight_byte_boundaries",
        ],
    );

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    result
        .validate_request(&request_message())
        .expect("the TTLV encoding constraint does not add request-payload restrictions");
}

fn assert_required_configuration_mapping(
    requirement_id: &str,
    source_clause_id: &str,
    constraint_key: &'static str,
    constraint_value: &'static str,
    expected_test_ids: &[&str],
) {
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == requirement_id)
        .unwrap_or_else(|| panic!("{requirement_id} has a typed profile mapping"));
    assert_eq!(mapping.source_clause_id, source_clause_id);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    assert!(mapping.evaluator.is_none());
    assert_eq!(
        mapping.constraints,
        &[RuleConstraint {
            key: constraint_key,
            value: constraint_value,
        }]
    );
    assert_eq!(
        mapping
            .test_evidence
            .iter()
            .map(|evidence| evidence.test_id)
            .collect::<Vec<_>>(),
        expected_test_ids
    );

    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == requirement_id)
        .unwrap_or_else(|| panic!("{requirement_id} is generated for the client profile"));
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .unwrap_or_else(|error| panic!("{requirement_id} maps to executable evidence: {error}"));
}

#[test]
fn profile_maps_required_ttlv_structure_order_and_numeric_padding() {
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let expected_mappings: &[(&str, &str, &[&str])] = &[
        (
            "KMIPKIT-REQ-SPEC-10.1.2-001",
            "KMIPKIT-CLAUSE-SPEC-10.1.2-001",
            &[
                "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::structure_preserves_child_order_and_repeated_tags",
            ],
        ),
        (
            "KMIPKIT-REQ-SPEC-10.1.2-002-001",
            "KMIPKIT-CLAUSE-SPEC-10.1.2-002",
            &[
                "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_big_integer_golden_vector_with_sign_extension_in_item_length",
                "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::big_integer_padding_is_minimal_and_sign_extended",
            ],
        ),
        (
            "KMIPKIT-REQ-SPEC-10.1.2-002-002",
            "KMIPKIT-CLAUSE-SPEC-10.1.2-002",
            &[
                "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::encodes_big_integer_golden_vector_with_sign_extension_in_item_length",
            ],
        ),
        (
            "KMIPKIT-REQ-SPEC-10.1.5-001-002",
            "KMIPKIT-CLAUSE-SPEC-10.1.5-001",
            &[
                "crates/kmipkit-client/tests/support/wire_encoder_tests.rs::fixed_width_four_byte_values_have_four_following_padding_bytes",
            ],
        ),
    ];

    for (requirement_id, source_clause_id, expected_test_ids) in expected_mappings {
        assert_required_configuration_mapping(
            requirement_id,
            source_clause_id,
            "encoding",
            "ttlv",
            expected_test_ids,
        );
    }

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: expected_mappings
            .iter()
            .map(|(requirement_id, _, _)| (*requirement_id).to_owned())
            .collect(),
    };
    result
        .validate_request(&request_message())
        .expect("TTLV encoding constraints do not add request-payload restrictions");
}

#[test]
fn profile_maps_required_channel_confidentiality_integrity_and_authenticity() {
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let expected_mappings: &[(&str, &[&str])] = &[
        (
            "KMIPKIT-REQ-SPEC-10.4-001-001",
            &[
                "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_sends_exact_bytes_over_tls13_mtls_and_closes_after_one_frame",
                "crates/kmipkit-transport/tests/https.rs::https_default_target_posts_exact_bytes_with_required_headers_and_zeroizes_on_success",
            ],
        ),
        (
            "KMIPKIT-REQ-SPEC-10.4-001-002",
            &[
                "crates/kmipkit-transport/tests/tls_integrity.rs::raw_tls_rejects_a_tampered_post_handshake_application_record",
                "crates/kmipkit-transport/tests/tls_integrity.rs::https_invalidates_a_connection_after_a_tampered_post_handshake_application_record",
            ],
        ),
        (
            "KMIPKIT-REQ-SPEC-10.4-001-003",
            &[
                "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_sends_exact_bytes_over_tls13_mtls_and_closes_after_one_frame",
                "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_unknown_server_ca",
                "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_expired_server_certificate",
                "crates/kmipkit-transport/tests/raw_tls.rs::raw_tls_rejects_server_name_mismatch",
            ],
        ),
    ];

    for (requirement_id, expected_test_ids) in expected_mappings {
        assert_required_configuration_mapping(
            requirement_id,
            "KMIPKIT-CLAUSE-SPEC-10.4-001",
            "transport_security",
            "tls13",
            expected_test_ids,
        );
    }

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: expected_mappings
            .iter()
            .map(|(requirement_id, _)| (*requirement_id).to_owned())
            .collect(),
    };
    result
        .validate_transport(ProfileTransport::RawTtlvTls)
        .expect("raw TTLV/TLS satisfies the channel-security requirement");
    result
        .validate_transport(ProfileTransport::Https)
        .expect("HTTPS satisfies the channel-security requirement");
}

#[test]
fn profile_maps_standard_and_extension_tag_prefixes_for_outbound_requests() {
    // KMIP Specification v2.1 §11.56: defined tags start with 0x42 and
    // extension tags start with 0x54. Applicability is mapped by DEC-018.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-11.56-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-11.56-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the TTLV tag-prefix requirement has an executable profile mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the TTLV tag-prefix requirement resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the production rule has category-matched executable evidence");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request = request_message_with_operation_payload(
        0x0000_0001,
        structure([tagged(
            0x0042_0173,
            Value::structure(structure([tagged(
                0x0054_1234,
                Value::byte_string(vec![0xA5]),
            )])),
        )]),
    );
    result
        .validate_request(&request)
        .expect("nested standard and extension tags use their required prefixes");
}

#[test]
fn profile_maps_reserved_tag_prohibition_for_outbound_requests() {
    // KMIP Specification v2.1 §11 prohibits using Tag Values marked Reserved.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-11-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-11-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the reserved-tag prohibition has an outbound profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the prohibition resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::ProhibitedBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the prohibition has category-appropriate negative evidence");
    let evaluate = mapping
        .evaluator
        .expect("the prohibition has a typed outbound-request evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request = request_message_with_operation_payload(
        0x0000_0001,
        structure([tagged(
            0x0042_0173,
            Value::structure(structure([tagged(
                0x0054_1234,
                Value::byte_string(vec![0xA5]),
            )])),
        )]),
    );
    assert!(evaluate(Some(&request), None));
    result
        .validate_request(&request)
        .expect("allocated standard and extension tags remain usable");
}

fn derive_key_payload(object_type: u32, lengths: impl IntoIterator<Item = Value>) -> Structure {
    let attributes = structure(
        lengths
            .into_iter()
            .map(|length| tagged(CRYPTOGRAPHIC_LENGTH_TAG, length)),
    );
    structure([
        tagged(OBJECT_TYPE_TAG, Value::enumeration(object_type)),
        tagged(ATTRIBUTES_TAG, Value::structure(attributes)),
    ])
}

fn derive_key_payload_with_attributes(
    object_type: u32,
    attributes: impl IntoIterator<Item = Item>,
) -> Structure {
    structure([
        tagged(OBJECT_TYPE_TAG, Value::enumeration(object_type)),
        tagged(ATTRIBUTES_TAG, Value::structure(structure(attributes))),
    ])
}

fn assert_request_rejected_for_requirement(
    result: &ProfileValidationResult,
    request: &crate::RequestMessage,
    requirement_id: &str,
    source_clause_id: &str,
    reason: &str,
) {
    let error = result.validate_request(request).expect_err(reason);
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(requirement_id));
    assert_eq!(error.source_clause_id(), Some(source_clause_id));
}

#[test]
fn result_message_client_access_has_an_optional_typed_rule() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.17-001-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.17-001";

    // OASIS KMIP Specification v2.1 §9.17; KMIPKIT-DEC-016.
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the client-accessible Result Message MAY has a typed optional-capability rule");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    assert_eq!(mapping.constraints, []);
    assert!(
        mapping
            .evaluator
            .is_some_and(|evaluate| evaluate(None, None))
    );
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id
            == "crates/kmipkit-protocol/tests/result_contract.rs::result_message_preserves_presence_and_text"
            && evidence.kind == RuleTestKind::OptionalCapabilityUsable
    }));
}

#[test]
fn profile_maps_client_single_item_unique_batch_item_id_as_optional_capability() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-PROF-4.1.2-007";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-PROF-4.1.2-007";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("single-item Unique Batch Item ID has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional profile capability resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the capability has a category-matched positive test");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for expected_id in [None, Some(b"single-item-id".as_slice())] {
        let request = request_message_with_unique_batch_item_id(expected_id);
        let item = request
            .batch_items()
            .next()
            .expect("the test request has one batch item");
        let actual_id = item.with_unique_batch_item_id(<[u8]>::to_vec);
        assert_eq!(actual_id, expected_id.map(<[u8]>::to_vec));
        result
            .validate_request(&request)
            .expect("the profile capability allows the ID to be included or omitted");
    }
}

#[test]
fn profile_maps_request_batch_item_repetition_as_optional_capability() {
    // KMIP Specification v2.1 §8.1 Table 394 allows Request Batch Item to repeat.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-8.1-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-8.1-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("request Batch Item repetition has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional Batch Item capability resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let single_item_request = request_message();
    result
        .validate_request(&single_item_request)
        .expect("the optional capability does not require repetition in every request");

    let repeated_items_request = request_message_with_batch_item_ids(&[b"first", b"second"]);
    let actual_ids = repeated_items_request
        .batch_items()
        .map(|item| {
            item.with_unique_batch_item_id(<[u8]>::to_vec)
                .expect("multi-item requests carry distinct identifiers")
        })
        .collect::<Vec<_>>();
    assert_eq!(repeated_items_request.header().batch_count(), 2);
    assert_eq!(actual_ids, vec![b"first".to_vec(), b"second".to_vec()]);
    assert!(evaluate(Some(&repeated_items_request), None));
    result
        .validate_request(&repeated_items_request)
        .expect("a client can represent a request with repeated Batch Items");
}

#[test]
fn profile_maps_repeated_get_wrapping_attribute_names_as_optional_capability() {
    // KMIP Specification v2.1 §7.18 Table 370 permits repeated Attribute Name
    // Text Strings in a Key Wrapping Specification carried by Get (§6.1.19).
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-7.18-006";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-7.18-006";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_get_wrapping_attribute_names_as_optional_capability";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("repeated Get wrapping Attribute Names have a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional wrapping Attribute Name capability resolves to metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    assert_eq!(
        mapping.verification_category,
        requirement.verification_category
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id == PROFILE_TEST_ID
            && evidence.kind == RuleTestKind::OptionalCapabilityUsable
    }));

    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let repeated_names_request = get_request_with_repeated_wrapping_attribute_names();
    assert!(evaluate(Some(&repeated_names_request), None));
    result
        .validate_request(&repeated_names_request)
        .expect("a client can send repeated wrapping Attribute Names");

    let request_without_wrapping_specification =
        request_message_with_operation_payload(GET_OPERATION, structure([]));
    result
        .validate_request(&request_without_wrapping_specification)
        .expect("the optional capability does not require wrapping on every Get");
}

#[test]
fn profile_requires_get_wrapping_key_information_pairing() {
    // KMIP Specification v2.1 §7.18 Table 370 requires at least one key-info
    // structure in a Key Wrapping Specification carried by a Get request.
    const REQUIREMENT_IDS: [&str; 2] = ["KMIPKIT-REQ-SPEC-7.18-004", "KMIPKIT-REQ-SPEC-7.18-005"];
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let no_key_information = get_request_with_wrapping_key_information(false, false);
    let encryption_only = get_request_with_wrapping_key_information(true, false);
    let mac_signature_only = get_request_with_wrapping_key_information(false, true);
    let both_key_information = get_request_with_wrapping_key_information(true, true);
    let get_without_wrapping = request_message_with_operation_payload(
        GET_OPERATION,
        structure([tagged(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("plain-object".to_owned()),
        )]),
    );
    let malformed_wrapping_specification = request_message_with_operation_payload(
        GET_OPERATION,
        structure([
            tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("wrapped-object".to_owned()),
            ),
            tagged(
                KEY_WRAPPING_SPECIFICATION_TAG,
                Value::text_string("not-a-structure".to_owned()),
            ),
        ]),
    );
    let unrelated_request =
        request_message_with_operation_payload(DISCOVER_VERSIONS_OPERATION, Structure::new());

    for (requirement_id, source_clause_id) in [
        (REQUIREMENT_IDS[0], "KMIPKIT-CLAUSE-SPEC-7.18-004"),
        (REQUIREMENT_IDS[1], "KMIPKIT-CLAUSE-SPEC-7.18-005"),
    ] {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id)
            .expect("the conditional key-information requirement has a typed profile rule");
        let requirement = CLIENT_REQUIREMENTS
            .iter()
            .find(|requirement| requirement.requirement_id == requirement_id)
            .expect("the conditional key-information requirement resolves to metadata");
        assert_eq!(mapping.source_clause_id, source_clause_id);
        assert_eq!(
            mapping.verification_category,
            GeneratedVerificationCategory::RequiredBehavior
        );
        validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
            .expect("the required key-information pairing has executable evidence");
        let evaluate = mapping
            .evaluator
            .expect("the conditional requirement has a typed evaluator");
        assert!(!evaluate(None, None));
        let result = ProfileValidationResult {
            profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
            requirement_ids: vec![requirement_id.to_owned()],
        };

        for valid in [
            &encryption_only,
            &mac_signature_only,
            &both_key_information,
            &get_without_wrapping,
            &unrelated_request,
        ] {
            assert!(evaluate(Some(valid), None));
            result
                .validate_request(valid)
                .expect("a Get without wrapping or with either key-info structure is valid");
        }
        assert!(!evaluate(Some(&no_key_information), None));
        assert_eq!(
            result
                .validate_request(&no_key_information)
                .expect_err("a Key Wrapping Specification needs one key-info structure")
                .requirement_id(),
            Some(requirement_id)
        );
        assert!(!evaluate(Some(&malformed_wrapping_specification), None));
        assert_eq!(
            result
                .validate_request(&malformed_wrapping_specification)
                .expect_err("a Key Wrapping Specification must be a Structure")
                .requirement_id(),
            Some(requirement_id)
        );
    }
}

#[test]
fn profile_maps_repeated_request_attestation_types_as_optional_capability() {
    // KMIP Specification v2.1 §8.2 Table 395 permits Attestation Type to repeat.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-8.2-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-8.2-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("repeated request Attestation Type has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional Attestation Type capability resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request_without_attestation_types = request_message();
    result
        .validate_request(&request_without_attestation_types)
        .expect("the optional capability does not require Attestation Type in every request");

    let repeated_types_request = request_message_with_attestation_types(&[2, 3]);
    assert_eq!(repeated_types_request.header().attestation_types(), [2, 3]);
    assert!(evaluate(Some(&repeated_types_request), None));
    result
        .validate_request(&repeated_types_request)
        .expect("a client can represent repeated Request Header Attestation Type values");
}

#[test]
fn profile_maps_repeated_request_message_extensions_as_optional_capability() {
    // KMIP Specification v2.1 §8.3 Table 396 permits repeated request Message Extensions.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-8.3-004";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-8.3-004";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_repeated_request_message_extensions_as_optional_capability";
    const MODEL_TEST_ID: &str = "crates/kmipkit-protocol/tests/message_validation.rs::repeated_request_extensions_preserve_order_and_expose_typed_fields";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("repeated request Message Extensions have a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional Message Extension capability resolves to generated metadata");
    let baseline = crate::profiles_generated::CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == BASELINE_PROFILE_ID)
        .expect("generated metadata includes Baseline Client");

    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    assert_eq!(
        mapping.verification_category,
        requirement.verification_category
    );
    assert!(baseline.requirement_ids.contains(&REQUIREMENT_ID));
    assert_eq!(requirement.official_test_case_ids, &[] as &[&str]);
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has category-matched executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id == PROFILE_TEST_ID
            && evidence.kind == RuleTestKind::OptionalCapabilityUsable
    }));
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id == MODEL_TEST_ID && evidence.kind == RuleTestKind::OptionalCapabilityUsable
    }));
    let evaluate = mapping
        .evaluator
        .expect("the optional Message Extension capability has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request_without_extensions = request_message();
    result
        .validate_request(&request_without_extensions)
        .expect("the optional capability does not require an extension in every request");

    let request_with_extensions =
        request_message_with_extensions([("Vendor_A", false), ("Vendor.B", true)]);
    let batch_item = request_with_extensions
        .batch_items()
        .next()
        .expect("the test request contains one Batch Item");
    assert_eq!(batch_item.message_extension_count(), 2);
    let vendors = (0..batch_item.message_extension_count())
        .map(|index| {
            batch_item
                .message_extension(index)
                .expect("each repeated extension is preserved")
                .with_vendor_identification(str::to_owned)
                .expect("each extension has a Vendor Identification")
        })
        .collect::<Vec<_>>();
    assert_eq!(vendors, ["Vendor_A", "Vendor.B"]);
    let first_extension = batch_item
        .message_extension(0)
        .expect("the first extension is present");
    let second_extension = batch_item
        .message_extension(1)
        .expect("the second extension is present");
    assert_eq!(first_extension.criticality_indicator(), Some(false));
    assert_eq!(second_extension.criticality_indicator(), Some(true));
    result
        .validate_request(&request_with_extensions)
        .expect("a client request can carry repeated Message Extensions in source order");
}

#[test]
fn profile_maps_request_time_stamp_as_optional_capability() {
    // KMIP Specification v2.1 §9.20 makes Time Stamp OPTIONAL in a client request.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.20-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.20-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional client request Time Stamp has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional client request Time Stamp resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request_without_time_stamp = request_message();
    result
        .validate_request(&request_without_time_stamp)
        .expect("the OPTIONAL field does not have to appear in every request");

    let raw_seconds = 1_751_100_123_i64;
    let request_with_time_stamp = request_message_with_time_stamp(raw_seconds);
    assert_eq!(
        request_with_time_stamp.header().time_stamp(),
        Some(raw_seconds)
    );
    assert!(evaluate(Some(&request_with_time_stamp), None));
    result
        .validate_request(&request_with_time_stamp)
        .expect("a client can represent an optional Request Header Time Stamp");
}

#[test]
fn profile_maps_validate_validity_date_as_optional_capability() {
    // KMIP Specification v2.1 §6.1.57 paragraph 2 and Table 340 make the
    // Validate request Validity Date optional.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.57-002-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.57-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const VALIDATE_OPERATION: u32 = 0x0000_0017;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional Validate Validity Date has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional date resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let omitted_payload = crate::ValidateRequest::new()
        .with_unique_identifier(crate::UniqueIdentifier::TextString(
            "managed-certificate".to_owned(),
        ))
        .into_ttlv_payload()
        .expect("the Validate request without a date has a valid payload");
    let omitted = request_message_with_operation_payload(VALIDATE_OPERATION, omitted_payload);
    assert!(evaluate(Some(&omitted), None));
    result
        .validate_request(&omitted)
        .expect("the optional Validity Date may be omitted");

    let caller_date = 1_751_100_123_i64;
    let supplied_payload = crate::ValidateRequest::new()
        .with_unique_identifier(crate::UniqueIdentifier::TextString(
            "managed-certificate".to_owned(),
        ))
        .with_validity_date(caller_date)
        .into_ttlv_payload()
        .expect("the Validate request with a Date-Time has a valid payload");
    assert!(matches!(
        supplied_payload.view().children().last().map(|field| {
            field.with_value(
                |value| matches!(value, ValueView::DateTime(date) if *date == caller_date),
            )
        }),
        Some(true)
    ));
    let supplied = request_message_with_operation_payload(VALIDATE_OPERATION, supplied_payload);
    assert!(evaluate(Some(&supplied), None));
    result
        .validate_request(&supplied)
        .expect("a client may include the caller-supplied Validity Date");
}

#[test]
fn profile_maps_validate_repeated_certificates_as_optional_capability() {
    // KMIP Specification v2.1 §6.1.57 Table 340 makes Certificate optional
    // and permits it to be repeated in a Validate request.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.57-004";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.57-004";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const VALIDATE_OPERATION: u32 = 0x0000_0017;
    const CERTIFICATE_TAG: u32 = 0x0042_0013;
    const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;
    const CERTIFICATE_VALUE_TAG: u32 = 0x0042_001E;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("repeated Validate Certificates have a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Certificate MAY resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let certificate = |bytes: &[u8]| {
        structure([
            tagged(CERTIFICATE_TYPE_TAG, Value::enumeration(1)),
            tagged(CERTIFICATE_VALUE_TAG, Value::byte_string(bytes.to_vec())),
        ])
    };
    let validate_request =
        |payload| request_message_with_operation_payload(VALIDATE_OPERATION, payload);
    let identifier = || crate::UniqueIdentifier::TextString("managed-certificate".to_owned());

    let omitted_payload = crate::ValidateRequest::new()
        .with_unique_identifier(identifier())
        .into_ttlv_payload()
        .expect("a Validate request may omit Certificate fields");
    let omitted = validate_request(omitted_payload);
    result
        .validate_request(&omitted)
        .expect("the optional Certificate fields may be omitted");

    let one_payload = crate::ValidateRequest::new()
        .with_certificate(certificate(b"one"))
        .with_unique_identifier(identifier())
        .into_ttlv_payload()
        .expect("one Certificate is a valid Validate request");
    let one = validate_request(one_payload);
    result
        .validate_request(&one)
        .expect("one Certificate remains valid");

    let two_payload = crate::ValidateRequest::new()
        .with_certificate(certificate(b"first"))
        .with_certificate(certificate(b"second"))
        .with_unique_identifier(identifier())
        .into_ttlv_payload()
        .expect("repeated Certificates are represented in the request payload");
    assert_eq!(
        two_payload
            .view()
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [CERTIFICATE_TAG, CERTIFICATE_TAG, UNIQUE_IDENTIFIER_TAG]
    );
    for (field, expected) in two_payload
        .view()
        .children()
        .iter()
        .zip([b"first".as_slice(), b"second".as_slice()])
    {
        field.with_value(|value| {
            let ValueView::Structure(certificate) = value else {
                panic!("Certificate Structure")
            };
            certificate.children()[1].with_value(|value| {
                assert!(matches!(value, ValueView::ByteString(bytes) if bytes == expected));
            });
        });
    }
    let two = validate_request(two_payload);
    result
        .validate_request(&two)
        .expect("two ordered Certificate values are accepted");
}

fn validate_certificate_for_unique_identifier_profile_test() -> Structure {
    structure([
        tagged(0x0042_001D, Value::enumeration(1)),
        tagged(0x0042_001E, Value::byte_string(b"certificate".to_vec())),
    ])
}

fn validate_request_payload_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn assert_ordered_validate_identifier_values(payload: &Structure) {
    let view = payload.view();
    let identifiers = &view.children()[1..];
    identifiers[0].with_value(|value| {
        assert!(matches!(value, ValueView::TextString("first-certificate")));
    });
    identifiers[1].with_value(|value| {
        assert!(matches!(value, ValueView::Enumeration(0xf123_4567)));
    });
    identifiers[2].with_value(|value| assert!(matches!(value, ValueView::Integer(-7))));
}

#[test]
fn profile_maps_validate_repeated_unique_identifiers_as_optional_capability() {
    // KMIP Specification v2.1 §6.1.57 Table 340 makes Validate Unique
    // Identifier fields optional and permits them to be repeated.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.57-005";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.57-005";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const VALIDATE_OPERATION: u32 = 0x0000_0017;
    const CERTIFICATE_TAG: u32 = 0x0042_0013;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("repeated Validate Unique Identifiers have a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Unique Identifier MAY resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let validate_request =
        |payload| request_message_with_operation_payload(VALIDATE_OPERATION, payload);

    let omitted_payload = crate::ValidateRequest::new()
        .with_certificate(validate_certificate_for_unique_identifier_profile_test())
        .into_ttlv_payload()
        .expect("a Validate request may omit the optional Unique Identifier fields");
    assert_eq!(
        validate_request_payload_tags(&omitted_payload),
        [CERTIFICATE_TAG]
    );
    let omitted = validate_request(omitted_payload);
    result
        .validate_request(&omitted)
        .expect("the optional Unique Identifier fields may be omitted");

    let one_payload = crate::ValidateRequest::new()
        .with_certificate(validate_certificate_for_unique_identifier_profile_test())
        .with_unique_identifier(crate::UniqueIdentifier::TextString(
            "first-certificate".to_owned(),
        ))
        .into_ttlv_payload()
        .expect("one Unique Identifier is a valid Validate request");
    assert_eq!(
        validate_request_payload_tags(&one_payload),
        [CERTIFICATE_TAG, UNIQUE_IDENTIFIER_TAG]
    );
    let one = validate_request(one_payload);
    result
        .validate_request(&one)
        .expect("one Unique Identifier remains valid");

    let repeated_payload = crate::ValidateRequest::new()
        .with_certificate(validate_certificate_for_unique_identifier_profile_test())
        .with_unique_identifier(crate::UniqueIdentifier::TextString(
            "first-certificate".to_owned(),
        ))
        .with_unique_identifier(crate::UniqueIdentifier::Enumeration(0xf123_4567))
        .with_unique_identifier(crate::UniqueIdentifier::Integer(-7))
        .into_ttlv_payload()
        .expect("repeated Unique Identifiers are represented in the request payload");
    assert_eq!(
        validate_request_payload_tags(&repeated_payload),
        [
            CERTIFICATE_TAG,
            UNIQUE_IDENTIFIER_TAG,
            UNIQUE_IDENTIFIER_TAG,
            UNIQUE_IDENTIFIER_TAG
        ]
    );
    assert_ordered_validate_identifier_values(&repeated_payload);
    let repeated = validate_request(repeated_payload);
    result
        .validate_request(&repeated)
        .expect("the request preserves multiple ordered Unique Identifier values");
}

#[test]
fn profile_rejects_malformed_validate_unique_identifier_wire_type() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.57-005";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const VALIDATE_OPERATION: u32 = 0x0000_0017;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("repeated Validate Unique Identifiers have a typed profile rule");
    let evaluate = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let malformed = request_message_with_operation_payload(
        VALIDATE_OPERATION,
        structure([tagged(
            UNIQUE_IDENTIFIER_TAG,
            Value::structure(Structure::new()),
        )]),
    );

    assert!(!evaluate(Some(&malformed), None));
    let error = result
        .validate_request(&malformed)
        .expect_err("Unique Identifier fields must use an identifier wire type");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
}

#[test]
fn profile_rejects_malformed_validate_validity_date_when_present() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.57-002-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const VALIDATE_OPERATION: u32 = 0x0000_0017;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional Validate Validity Date has a typed profile rule");
    let evaluate = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let malformed = request_message_with_operation_payload(
        VALIDATE_OPERATION,
        structure([
            tagged(
                UNIQUE_IDENTIFIER_TAG,
                Value::text_string("managed-certificate".to_owned()),
            ),
            tagged(VALIDITY_DATE_TAG, Value::integer(42)),
        ]),
    );

    assert!(!evaluate(Some(&malformed), None));
    let error = result
        .validate_request(&malformed)
        .expect_err("a present Validity Date must retain its Date-Time wire type");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
}

#[test]
fn profile_maps_optional_maximum_response_size_capability() {
    // KMIP Specification v2.1 §9.12 defines Maximum Response Size as optional.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.12-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.12-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional Maximum Response Size has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional field resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let omitted = request_message();
    assert_eq!(omitted.header().maximum_response_size(), None);
    assert!(evaluate(Some(&omitted), None));
    result
        .validate_request(&omitted)
        .expect("the OPTIONAL field may be omitted");

    let supplied = request_message_with_optional_maximum_response_size(
        DISCOVER_VERSIONS_OPERATION,
        Some(4096),
    );
    assert_eq!(supplied.header().maximum_response_size(), Some(4096));
    assert!(evaluate(Some(&supplied), None));
    result
        .validate_request(&supplied)
        .expect("a client may include Maximum Response Size");
}

#[test]
fn profile_maps_maximum_response_size_only_for_possible_large_replies() {
    // KMIP Specification v2.1 §9.12; Discover Versions is not likely-large,
    // while Create Split Key can return repeated key material (§6.1.10).
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.12-001-003";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.12-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the large-response recommendation has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the large-response recommendation resolves to normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RecommendedBehaviorOrDeviation
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the recommendation has executable category-matched evidence");

    let evaluate = mapping
        .evaluator
        .expect("the recommendation has a typed request evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let discover_without_limit =
        request_message_with_optional_maximum_response_size(DISCOVER_VERSIONS_OPERATION, None);
    assert!(evaluate(Some(&discover_without_limit), None));
    result
        .validate_request(&discover_without_limit)
        .expect("Discover Versions may omit the peer-visible field");

    let split_key_without_limit =
        request_message_with_optional_maximum_response_size(CREATE_SPLIT_KEY_OPERATION, None);
    assert!(evaluate(Some(&split_key_without_limit), None));
    result
        .validate_request(&split_key_without_limit)
        .expect("the recommendation does not make Maximum Response Size mandatory");

    let discover_with_limit = request_message_with_optional_maximum_response_size(
        DISCOVER_VERSIONS_OPERATION,
        Some(4096),
    );
    assert!(!evaluate(Some(&discover_with_limit), None));
    let error = result
        .validate_request(&discover_with_limit)
        .expect_err("the recommended field is excluded for non-large Discover Versions replies");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));

    let split_key_with_limit =
        request_message_with_optional_maximum_response_size(CREATE_SPLIT_KEY_OPERATION, Some(4096));
    assert!(evaluate(Some(&split_key_with_limit), None));
    result
        .validate_request(&split_key_with_limit)
        .expect("a potentially large Create Split Key reply may advertise the configured cap");
}

#[test]
fn profile_maps_optional_client_correlation_value_capability() {
    // KMIP Specification v2.1 §9.9 permits clients to add this request string.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.9-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.9-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional client correlation value has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the client correlation MAY resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let omitted = request_message();
    assert_eq!(
        omitted
            .header()
            .with_client_correlation_value(str::to_owned),
        None
    );
    assert!(evaluate(Some(&omitted), None));
    result
        .validate_request(&omitted)
        .expect("the optional Client Correlation Value may be omitted");

    let supplied = request_message_with_client_correlation_value("request-123");
    assert_eq!(
        supplied
            .header()
            .with_client_correlation_value(str::to_owned),
        Some("request-123".to_owned())
    );
    assert!(evaluate(Some(&supplied), None));
    result
        .validate_request(&supplied)
        .expect("the client may include its caller-provided correlation string");
}

#[test]
fn profile_maps_poll_cancel_asynchronous_correlation_requirements() {
    const REQUIREMENTS: [(&str, &str); 4] = [
        (
            "KMIPKIT-REQ-SPEC-6.1.5-001",
            "KMIPKIT-CLAUSE-SPEC-6.1.5-001",
        ),
        (
            "KMIPKIT-REQ-SPEC-6.1.38-001-001",
            "KMIPKIT-CLAUSE-SPEC-6.1.38-001",
        ),
        ("KMIPKIT-REQ-SPEC-9.1-001", "KMIPKIT-CLAUSE-SPEC-9.1-001"),
        ("KMIPKIT-REQ-SPEC-9.19-002", "KMIPKIT-CLAUSE-SPEC-9.19-002"),
    ];
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_poll_cancel_asynchronous_correlation_requirements";
    const ASYNC_CORRELATION_VALUE_TAG: u32 = 0x0042_0006;
    const POLL_OPERATION: u32 = 0x0000_001A;
    const CANCEL_OPERATION: u32 = 0x0000_0019;
    const OTHER_OPERATION: u32 = 0x0000_001E;

    let correlation = b"server-issued\x00correlation\xff";
    let correlation_field = || {
        tagged(
            ASYNC_CORRELATION_VALUE_TAG,
            Value::byte_string(correlation.to_vec()),
        )
    };
    let correlation_payload = || structure([correlation_field()]);
    let poll = request_message_with_operation_payload(POLL_OPERATION, correlation_payload());
    let cancel = request_message_with_operation_payload(CANCEL_OPERATION, correlation_payload());
    let other_operation = request_message_with_operation_payload(OTHER_OPERATION, Structure::new());
    let missing_poll_correlation =
        request_message_with_operation_payload(POLL_OPERATION, Structure::new());
    let missing_cancel_correlation =
        request_message_with_operation_payload(CANCEL_OPERATION, Structure::new());
    let wrong_type_poll_correlation = request_message_with_operation_payload(
        POLL_OPERATION,
        structure([tagged(ASYNC_CORRELATION_VALUE_TAG, Value::integer(7))]),
    );
    let wrong_type_cancel_correlation = request_message_with_operation_payload(
        CANCEL_OPERATION,
        structure([tagged(ASYNC_CORRELATION_VALUE_TAG, Value::integer(7))]),
    );
    let duplicate_poll_correlation = request_message_with_operation_payload(
        POLL_OPERATION,
        structure([correlation_field(), correlation_field()]),
    );
    let duplicate_cancel_correlation = request_message_with_operation_payload(
        CANCEL_OPERATION,
        structure([correlation_field(), correlation_field()]),
    );
    let invalid_requests = [
        &missing_poll_correlation,
        &missing_cancel_correlation,
        &wrong_type_poll_correlation,
        &wrong_type_cancel_correlation,
        &duplicate_poll_correlation,
        &duplicate_cancel_correlation,
    ];

    for (requirement_id, source_clause_id) in REQUIREMENTS {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id)
            .expect("the asynchronous correlation requirement has a typed profile rule");
        let requirement = CLIENT_REQUIREMENTS
            .iter()
            .find(|requirement| requirement.requirement_id == requirement_id)
            .expect("the asynchronous correlation requirement has generated metadata");
        assert_eq!(mapping.source_clause_id, source_clause_id);
        assert_eq!(
            mapping.verification_category,
            GeneratedVerificationCategory::RequiredBehavior
        );
        validate_mapping("KMIPKIT-PROFILE-BASELINE-CLIENT", requirement, mapping)
            .expect("the mandatory correlation rule has executable evidence");
        let evaluate = mapping
            .evaluator
            .expect("the mandatory correlation rule has a typed evaluator");
        assert!(mapping.test_evidence.iter().any(|evidence| {
            evidence.test_id == PROFILE_TEST_ID && evidence.kind == RuleTestKind::PositiveBehavior
        }));
        assert!(!evaluate(None, None));
        assert!(evaluate(Some(&poll), None));
        assert!(evaluate(Some(&cancel), None));
        assert!(evaluate(Some(&other_operation), None));
        assert!(
            invalid_requests
                .iter()
                .all(|request| !evaluate(Some(request), None))
        );
    }
}

#[test]
fn profile_maps_request_unique_batch_item_id_as_optional_capability() {
    // KMIP Specification v2.1 §9.21 makes the ID optional in a client request.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.21-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.21-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional client request ID has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional client request ID resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for expected_id in [None, Some(b"single-item-id".as_slice())] {
        let request = request_message_with_unique_batch_item_id(expected_id);
        let item = request
            .batch_items()
            .next()
            .expect("the test request has one batch item");
        assert_eq!(
            item.with_unique_batch_item_id(<[u8]>::to_vec),
            expected_id.map(<[u8]>::to_vec)
        );
        assert!(evaluate(Some(&request), None));
        result
            .validate_request(&request)
            .expect("the request ID remains optional for a single-item request");
    }
}

#[test]
fn profile_maps_client_get_attributes_optional_attribute_references() {
    // KMIP Specification v2.1 §6.1.20 Table 223 permits repeated references.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.20-004";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.20-004";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ATTRIBUTE_REFERENCE_TAG: u32 = 0x0042_013B;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional Get Attributes references have a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional references resolve to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for references in [
        Vec::new(),
        vec![
            crate::AttributeReference::tag(0x0042_002F),
            crate::AttributeReference::name("KMIPKit.TestVendor_1", "Opaque.Attribute"),
        ],
    ] {
        let expected_reference_count = references.len();
        let request = crate::GetAttributesRequest::try_new(None, references)
            .expect("Table 223 permits zero or more distinct references");
        let payload = request
            .to_ttlv_payload()
            .expect("the Get Attributes request is structurally valid");
        assert_eq!(
            payload
                .view()
                .children()
                .iter()
                .filter(|field| field.tag().raw() == ATTRIBUTE_REFERENCE_TAG)
                .count(),
            expected_reference_count
        );

        let request = request_message_with_operation_payload(GET_ATTRIBUTES_OPERATION, payload);
        assert!(evaluate(Some(&request), None));
        result
            .validate_request(&request)
            .expect("omitted or repeated Get Attributes references remain usable");
    }
}

#[test]
fn profile_rejects_repeated_identical_get_attributes_references() {
    // KMIP Specification v2.1 §6.1.20 prohibits repeating one Attribute Reference.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.20-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.20-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ATTRIBUTE_REFERENCE_TAG: u32 = 0x0042_013B;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the repeated-reference prohibition has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the prohibition resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::ProhibitedBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the prohibited behavior has negative executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let repeated_reference = || {
        crate::AttributeReference::tag(0x0042_002F)
            .to_ttlv_item()
            .expect("the test Attribute Reference is valid")
    };
    let duplicate_payload = structure([repeated_reference(), repeated_reference()]);
    let duplicate_request =
        request_message_with_operation_payload(GET_ATTRIBUTES_OPERATION, duplicate_payload);
    let error = result
        .validate_request(&duplicate_request)
        .expect_err("a Get Attributes request must not repeat an identical reference");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let repeated_named_reference = || {
        crate::AttributeReference::name("KMIPKit.TestVendor_1", "Opaque.Attribute")
            .to_ttlv_item()
            .expect("the test name-form Attribute Reference is valid")
    };
    let duplicate_name_request = request_message_with_operation_payload(
        GET_ATTRIBUTES_OPERATION,
        structure([repeated_named_reference(), repeated_named_reference()]),
    );
    let error = result
        .validate_request(&duplicate_name_request)
        .expect_err("a name-form Attribute Reference must not be repeated either");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let malformed_reference_request = request_message_with_operation_payload(
        GET_ATTRIBUTES_OPERATION,
        structure([tagged(
            ATTRIBUTE_REFERENCE_TAG,
            Value::text_string("not an Attribute Reference".to_owned()),
        )]),
    );
    let error = result
        .validate_request(&malformed_reference_request)
        .expect_err("a malformed Attribute Reference must fail closed");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    for references in [
        Vec::new(),
        vec![
            crate::AttributeReference::tag(0x0042_002F),
            crate::AttributeReference::tag(0x0042_002A),
        ],
    ] {
        let payload = structure(
            references
                .iter()
                .map(|reference| reference.to_ttlv_item().expect("reference is valid")),
        );
        let request = request_message_with_operation_payload(GET_ATTRIBUTES_OPERATION, payload);
        assert!(evaluate(Some(&request), None));
        result
            .validate_request(&request)
            .expect("omitted or distinct Attribute References are permitted");
    }

    let unrelated_operation = request_message_with_operation_payload(
        0x0000_0003,
        structure([repeated_reference(), repeated_reference()]),
    );
    assert!(evaluate(Some(&unrelated_operation), None));
    result
        .validate_request(&unrelated_operation)
        .expect("Get Attributes reference rules do not apply to other operations");
}

#[test]
fn profile_maps_client_query_function_repetition_as_optional_capability() {
    // KMIP Specification v2.1 §6.1.40 Table 282 allows Query Function repetition.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.40-016";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.40-016";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const QUERY_OPERATION: u32 = 0x0000_0018;
    const QUERY_FUNCTION_TAG: u32 = 0x0042_0074;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Query Function repetition has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Query Function MAY resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let repeated_payload = crate::QueryRequest::new([
        crate::QueryFunction::OPERATIONS,
        crate::QueryFunction::OBJECTS,
        crate::QueryFunction::OPERATIONS,
    ])
    .to_ttlv_payload()
    .expect("Table 282 permits repeated Query Function values");
    assert_eq!(
        repeated_payload
            .view()
            .children()
            .iter()
            .filter(|field| field.tag().raw() == QUERY_FUNCTION_TAG)
            .count(),
        3
    );
    let repeated_request =
        request_message_with_operation_payload(QUERY_OPERATION, repeated_payload);
    assert!(evaluate(Some(&repeated_request), None));
    result
        .validate_request(&repeated_request)
        .expect("repeated Query Functions are an allowed client capability");

    let single_function = request_message_with_operation_payload(
        QUERY_OPERATION,
        crate::QueryRequest::new([crate::QueryFunction::OPERATIONS])
            .to_ttlv_payload()
            .expect("a single Query Function is valid"),
    );
    assert!(result.validate_request(&single_function).is_ok());
    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    assert!(result.validate_request(&unrelated_operation).is_ok());
}

#[test]
fn profile_requires_at_least_one_client_query_function() {
    // KMIP Specification v2.1 §6.1.40 paragraph 14 requires one or more.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.40-014";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.40-014";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const QUERY_OPERATION: u32 = 0x0000_0018;
    const QUERY_FUNCTION_TAG: u32 = 0x0042_0074;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the required Query Function has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Query Function SHALL resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let valid_request = request_message_with_operation_payload(
        QUERY_OPERATION,
        crate::QueryRequest::new([crate::QueryFunction::OPERATIONS])
            .to_ttlv_payload()
            .expect("one Query Function satisfies the requirement"),
    );
    assert!(evaluate(Some(&valid_request), None));
    result
        .validate_request(&valid_request)
        .expect("a Query request with one function satisfies the SHALL");

    let empty_query = request_message_with_operation_payload(QUERY_OPERATION, Structure::new());
    assert!(!evaluate(Some(&empty_query), None));
    assert!(result.validate_request(&empty_query).is_err());

    let wrong_type_query = request_message_with_operation_payload(
        QUERY_OPERATION,
        structure([tagged(
            QUERY_FUNCTION_TAG,
            Value::text_string("OPERATIONS".to_owned()),
        )]),
    );
    assert!(!evaluate(Some(&wrong_type_query), None));
    assert!(result.validate_request(&wrong_type_query).is_err());

    let mixed_type_query = request_message_with_operation_payload(
        QUERY_OPERATION,
        structure([
            tagged(
                QUERY_FUNCTION_TAG,
                Value::enumeration(crate::QueryFunction::OPERATIONS.raw()),
            ),
            tagged(QUERY_FUNCTION_TAG, Value::text_string("INVALID".to_owned())),
        ]),
    );
    assert!(!evaluate(Some(&mixed_type_query), None));
    assert!(result.validate_request(&mixed_type_query).is_err());

    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    assert!(result.validate_request(&unrelated_operation).is_ok());
}

#[test]
fn profile_maps_optional_locate_maximum_items_capability() {
    // KMIP Specification v2.1 §6.1.28 and Table 247 make Maximum Items optional.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.28-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.28-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const LOCATE_OPERATION: u32 = 0x0000_0008;
    const MAXIMUM_ITEMS_TAG: u32 = 0x0042_004F;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("optional Locate Maximum Items has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Maximum Items MAY resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let omitted = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(crate::AttributeSet::new())
            .to_ttlv_payload()
            .expect("an omitted Maximum Items value is valid"),
    );
    assert!(evaluate(Some(&omitted), None));
    result
        .validate_request(&omitted)
        .expect("the optional field may be omitted");

    let supplied = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(crate::AttributeSet::new())
            .with_maximum_items(3)
            .to_ttlv_payload()
            .expect("an Integer Maximum Items value is valid"),
    );
    assert!(evaluate(Some(&supplied), None));
    result
        .validate_request(&supplied)
        .expect("a client may supply Maximum Items without asserting server behavior");

    let malformed = request_message_with_operation_payload(
        LOCATE_OPERATION,
        structure([
            tagged(MAXIMUM_ITEMS_TAG, Value::text_string("three".to_owned())),
            tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
        ]),
    );
    assert!(!evaluate(Some(&malformed), None));
    assert!(result.validate_request(&malformed).is_err());

    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    assert!(result.validate_request(&unrelated_operation).is_ok());
}

#[test]
fn profile_maps_optional_locate_offset_items_capability() {
    // KMIP Specification v2.1 §6.1.28 and Table 247 make Offset Items optional.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.28-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.28-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const LOCATE_OPERATION: u32 = 0x0000_0008;
    const OFFSET_ITEMS_TAG: u32 = 0x0042_00D4;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("optional Locate Offset Items has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Offset Items MAY resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let omitted = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(crate::AttributeSet::new())
            .to_ttlv_payload()
            .expect("an omitted Offset Items value is valid"),
    );
    assert!(evaluate(Some(&omitted), None));
    result
        .validate_request(&omitted)
        .expect("Offset Items may be omitted");

    for offset_items in [0, 3] {
        let supplied = request_message_with_operation_payload(
            LOCATE_OPERATION,
            crate::LocateRequest::new(crate::AttributeSet::new())
                .with_offset_items(offset_items)
                .to_ttlv_payload()
                .expect("Offset Items is encoded as an Integer"),
        );
        assert!(evaluate(Some(&supplied), None));
        result
            .validate_request(&supplied)
            .expect("the optional Integer is usable, including explicit zero");
    }

    let malformed = request_message_with_operation_payload(
        LOCATE_OPERATION,
        structure([
            tagged(OFFSET_ITEMS_TAG, Value::text_string("three".to_owned())),
            tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
        ]),
    );
    assert!(!evaluate(Some(&malformed), None));
    assert!(result.validate_request(&malformed).is_err());

    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    assert!(result.validate_request(&unrelated_operation).is_ok());
}

#[test]
fn profile_maps_client_locate_attributes_as_required_behavior() {
    // KMIP Specification v2.1 §6.1.28 Table 247 marks Locate Attributes required.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.28-013-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.28-013";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const LOCATE_OPERATION: u32 = 0x0000_0008;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Locate Attributes has a typed required-behavior rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("required Locate Attributes resolve to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required Attributes Structure has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the required Attributes field has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let valid_request = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(crate::AttributeSet::new())
            .to_ttlv_payload()
            .expect("Locate encodes its required Attributes Structure"),
    );
    assert!(evaluate(Some(&valid_request), None));
    result
        .validate_request(&valid_request)
        .expect("a Locate request with the required empty Attributes Structure is valid");

    let missing_attributes =
        request_message_with_operation_payload(LOCATE_OPERATION, Structure::new());
    assert!(!evaluate(Some(&missing_attributes), None));
    assert!(result.validate_request(&missing_attributes).is_err());

    let malformed_attributes = request_message_with_operation_payload(
        LOCATE_OPERATION,
        structure([tagged(ATTRIBUTES_TAG, Value::integer(1))]),
    );
    assert!(!evaluate(Some(&malformed_attributes), None));
    assert!(result.validate_request(&malformed_attributes).is_err());

    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    assert!(result.validate_request(&unrelated_operation).is_ok());
}

#[test]
fn profile_maps_client_locate_empty_attributes_match_all_capability() {
    // KMIP Specification v2.1 §6.1.28 Table 247 permits empty Attributes.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.28-013-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.28-013";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const LOCATE_OPERATION: u32 = 0x0000_0008;
    const ACTIVATION_DATE_TAG: u32 = 0x0042_0001;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("empty Locate Attributes has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the empty Attributes MAY resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let empty_attributes = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(crate::AttributeSet::new())
            .to_ttlv_payload()
            .expect("the required Attributes structure may be empty"),
    );
    assert!(evaluate(Some(&empty_attributes), None));
    result
        .validate_request(&empty_attributes)
        .expect("an empty required Attributes structure is valid");

    let matching_attributes = request_message_with_operation_payload(
        LOCATE_OPERATION,
        structure([tagged(
            ATTRIBUTES_TAG,
            Value::structure(structure([tagged(
                ACTIVATION_DATE_TAG,
                Value::date_time(100),
            )])),
        )]),
    );
    assert!(evaluate(Some(&matching_attributes), None));

    let missing_attributes =
        request_message_with_operation_payload(LOCATE_OPERATION, Structure::new());
    assert!(!evaluate(Some(&missing_attributes), None));
    assert!(result.validate_request(&missing_attributes).is_err());

    let malformed_attributes = request_message_with_operation_payload(
        LOCATE_OPERATION,
        structure([tagged(
            ATTRIBUTES_TAG,
            Value::text_string("not a structure".to_owned()),
        )]),
    );
    assert!(!evaluate(Some(&malformed_attributes), None));
    assert!(result.validate_request(&malformed_attributes).is_err());

    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    assert!(result.validate_request(&unrelated_operation).is_ok());
}

#[test]
fn profile_maps_client_locate_key_value_present_as_optional_capability() {
    // KMIP Specification v2.1 §4.28 permits Key Value Present in Locate.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-003";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.28-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Key Value Present in Locate has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Locate capability resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional Locate capability has positive executable evidence");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for value in [true, false] {
        let criteria =
            crate::AttributeSet::try_new([tagged(KEY_VALUE_PRESENT_TAG, Value::boolean(value))])
                .expect("Key Value Present is a Boolean Locate criterion");
        let request = request_message_with_operation_payload(
            LOCATE_OPERATION_TAG,
            crate::LocateRequest::new(criteria)
                .to_ttlv_payload()
                .expect("Locate preserves the requested Boolean criterion"),
        );
        result
            .validate_request(&request)
            .expect("the selected client profile can use either Boolean criterion value");
    }

    let without_optional_criterion = request_message_with_operation_payload(
        LOCATE_OPERATION_TAG,
        crate::LocateRequest::new(crate::AttributeSet::new())
            .to_ttlv_payload()
            .expect("Locate still carries its required Attributes Structure"),
    );
    result
        .validate_request(&without_optional_criterion)
        .expect("the optional capability does not become mandatory");
}

#[test]
fn profile_rejects_malformed_locate_key_value_present_criterion() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-003";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let malformed_requests = [
        request_message_with_operation_payload(
            LOCATE_OPERATION_TAG,
            structure([tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    KEY_VALUE_PRESENT_TAG,
                    Value::text_string("not a Boolean".to_owned()),
                )])),
            )]),
        ),
        request_message_with_operation_payload(
            LOCATE_OPERATION_TAG,
            structure([tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([
                    tagged(KEY_VALUE_PRESENT_TAG, Value::boolean(true)),
                    tagged(KEY_VALUE_PRESENT_TAG, Value::boolean(false)),
                ])),
            )]),
        ),
    ];

    for request in malformed_requests {
        let error = result
            .validate_request(&request)
            .expect_err("a Locate Key Value Present criterion must be one Boolean value");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-4.28-001")
        );
    }
}

#[test]
fn profile_maps_client_locate_partial_structured_attribute_fields() {
    // KMIP Specification v2.1 §6.1.28 permits partial structured attributes.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.28-007";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.28-007";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const LOCATE_OPERATION: u32 = 0x0000_0008;
    const LINK_TAG: u32 = 0x0042_004A;
    const LINK_TYPE_TAG: u32 = 0x0042_004B;
    const LINKED_OBJECT_IDENTIFIER_TAG: u32 = 0x0042_004C;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("partial structured Locate attributes have a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("partial structured Locate attributes resolve to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required partial-structure behavior has executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let partial_link = crate::AttributeSet::try_new([tagged(
        LINK_TAG,
        Value::structure(structure([tagged(
            LINKED_OBJECT_IDENTIFIER_TAG,
            Value::text_string("linked-object-7".to_owned()),
        )])),
    )])
    .expect("a Link may contain Linked Object Identifier without Link Type");
    let partial_request = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(partial_link)
            .to_ttlv_payload()
            .expect("the partial Link attribute is retained in Locate"),
    );
    assert!(evaluate(Some(&partial_request), None));
    result
        .validate_request(&partial_request)
        .expect("the partial structured Link is valid client evidence");

    let complete_link = crate::AttributeSet::try_new([tagged(
        LINK_TAG,
        Value::structure(structure([
            tagged(LINK_TYPE_TAG, Value::enumeration(1)),
            tagged(
                LINKED_OBJECT_IDENTIFIER_TAG,
                Value::text_string("linked-object-7".to_owned()),
            ),
        ])),
    )])
    .expect("a complete Link attribute remains valid");
    let complete_request = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(complete_link)
            .to_ttlv_payload()
            .expect("the complete Link attribute is retained in Locate"),
    );
    assert!(evaluate(Some(&complete_request), None));

    let malformed_link = request_message_with_operation_payload(
        LOCATE_OPERATION,
        structure([tagged(
            ATTRIBUTES_TAG,
            Value::structure(structure([tagged(
                LINK_TAG,
                Value::text_string("not a Link Structure".to_owned()),
            )])),
        )]),
    );
    assert!(!evaluate(Some(&malformed_link), None));
    assert!(result.validate_request(&malformed_link).is_err());

    let no_link_criteria = request_message_with_operation_payload(
        LOCATE_OPERATION,
        crate::LocateRequest::new(crate::AttributeSet::new())
            .to_ttlv_payload()
            .expect("Link is an optional matching criterion"),
    );
    assert!(evaluate(Some(&no_link_criteria), None));

    let unrelated_operation = request_message_with_operation_payload(0x0000_0003, Structure::new());
    assert!(result.validate_request(&unrelated_operation).is_ok());
}

#[test]
fn profile_maps_create_split_key_optional_source_identifier() {
    // KMIP Specification v2.1 §6.1.10 permits identifying the split source.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.10-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.10-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const CREATE_SPLIT_KEY_OPERATION: u32 = 0x0000_0028;
    const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the optional Create Split Key source ID has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional source ID resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for include_identifier in [false, true] {
        let request = crate::CreateSplitKeyRequest::new(
            crate::ObjectType::from_raw(7),
            3,
            2,
            crate::SplitKeyMethod::from_raw(1),
            crate::AttributeSet::new(),
        );
        let request = if include_identifier {
            request.with_unique_identifier(crate::UniqueIdentifier::TextString(
                "split-source".to_owned(),
            ))
        } else {
            request
        };
        let request = request_message_with_operation_payload(
            CREATE_SPLIT_KEY_OPERATION,
            request
                .into_ttlv_payload()
                .expect("the Create Split Key request is structurally valid"),
        );
        let item = request
            .batch_items()
            .next()
            .expect("the test request contains one batch item");
        let has_identifier = item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .any(|field| field.tag().raw() == UNIQUE_IDENTIFIER_TAG)
            })
            .expect("the test request contains a Request Payload");
        assert_eq!(has_identifier, include_identifier);
        assert!(evaluate(Some(&request), None));
        result
            .validate_request(&request)
            .expect("the client may omit or provide the optional split source ID");
    }
}
fn request_message_with_time_stamp(seconds: i64) -> crate::RequestMessage {
    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(TIME_STAMP_TAG, Value::date_time(seconds)),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Request Message with a Time Stamp is valid")
}

fn request_message_with_client_correlation_value(value: &str) -> crate::RequestMessage {
    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(
                    CLIENT_CORRELATION_VALUE_TAG,
                    Value::text_string(value.to_owned()),
                ),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Request Message with Client Correlation Value is valid")
}

fn request_message_with_attestation_types(attestation_types: &[u32]) -> crate::RequestMessage {
    let mut header_fields = vec![test_protocol_version()];
    header_fields.extend(
        attestation_types
            .iter()
            .map(|value| tagged(ATTESTATION_TYPE_TAG, Value::enumeration(*value))),
    );
    header_fields.push(tagged(BATCH_COUNT_TAG, Value::integer(1)));

    crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure(header_fields)),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(0x1E)),
                tagged(REQUEST_PAYLOAD_TAG, Value::structure(Structure::new())),
            ])),
        ),
    ]))
    .expect("test Request Message with Attestation Type fields is valid")
}
#[test]
fn profile_maps_required_unique_batch_item_id_for_multi_item_requests() {
    // KMIP Specification v2.1 §8.3 Table 396 requires this field when Batch
    // Count is greater than one; Profiles v2.1 §4.1.2 item 6 corroborates the
    // client-side requirement and permits omission for a single-item request.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-8.3-003";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-8.3-003";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("multi-item Unique Batch Item ID has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the requirement resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));
    let single_item_request = request_message_with_unique_batch_item_id(None);
    assert!(evaluate(Some(&single_item_request), None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request = request_message_with_batch_item_ids(&[b"first-item", b"second-item"]);
    assert_eq!(request.batch_items().len(), 2);
    assert!(
        request
            .batch_items()
            .all(|item| item.with_unique_batch_item_id(|_| ()).is_some())
    );
    result
        .validate_request(&request)
        .expect("a multi-item request with per-item IDs satisfies §8.3");
}

#[test]
fn profile_maps_required_batch_count_to_request_item_count() {
    // KMIP Specification v2.1 §9.5 requires Batch Count to represent the
    // number of Batch Items in the message.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.5-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.5-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Batch Count has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("Batch Count resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let single_item_request = request_message();
    assert_eq!(single_item_request.header().batch_count(), 1);
    assert_eq!(single_item_request.batch_items().len(), 1);
    assert!(evaluate(Some(&single_item_request), None));
    result
        .validate_request(&single_item_request)
        .expect("one Batch Item is accompanied by Batch Count 1");

    let multi_item_request = request_message_with_batch_item_ids(&[b"first", b"second"]);
    assert_eq!(multi_item_request.header().batch_count(), 2);
    assert_eq!(multi_item_request.batch_items().len(), 2);
    assert!(evaluate(Some(&multi_item_request), None));
    result
        .validate_request(&multi_item_request)
        .expect("two Batch Items are accompanied by Batch Count 2");
}

#[test]
fn profile_maps_single_operation_batch_count_to_one() {
    // KMIP Specification v2.1 §9.5 requires count 1 for a single operation.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.5-001-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.5-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the single-operation Batch Count rule has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the single-operation Batch Count rule resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let single_item_request = request_message();
    assert_eq!(single_item_request.batch_items().len(), 1);
    assert_eq!(single_item_request.header().batch_count(), 1);
    assert!(evaluate(Some(&single_item_request), None));
    result
        .validate_request(&single_item_request)
        .expect("one requested operation has Batch Count 1");

    let multi_item_request = request_message_with_batch_item_ids(&[b"first", b"second"]);
    assert_eq!(multi_item_request.batch_items().len(), 2);
    assert_eq!(multi_item_request.header().batch_count(), 2);
    assert!(evaluate(Some(&multi_item_request), None));
    result
        .validate_request(&multi_item_request)
        .expect("the single-operation condition does not constrain a multi-item request");
}

#[test]
fn profile_maps_default_batch_error_continuation_to_stop() {
    // KMIP Specification v2.1 §9.6 defaults an omitted option to Stop.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.6-001-003";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.6-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const STOP: u32 = 2;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the omitted Batch Error Continuation Option default has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Batch Error Continuation Option default resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request_without_option = request_message_with_batch_error_continuation_option(None);
    assert_eq!(
        request_without_option
            .header()
            .batch_error_continuation_option(),
        None
    );
    assert_eq!(
        request_without_option
            .header()
            .effective_batch_error_continuation_option(),
        STOP
    );
    assert!(evaluate(Some(&request_without_option), None));
    result
        .validate_request(&request_without_option)
        .expect("an omitted option has the §9.6 effective value Stop");

    let request_with_explicit_continue =
        request_message_with_batch_error_continuation_option(Some(1));
    assert!(evaluate(Some(&request_with_explicit_continue), None));
}

#[test]
fn profile_maps_assigned_batch_error_continuation_values() {
    // KMIP Specification v2.1 §9.6 and §11.5 Table 435 assign Continue=1,
    // Stop=2, and Undo=3. DEC-002 preserves extension values in generic models
    // while restricting this outbound profile rule to the assigned values.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.6-001-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.6-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("assigned Batch Error Continuation values have a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the assigned Batch Error Continuation values resolve to metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed outbound request evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for value in [1, 2, 3] {
        let request = request_message_with_batch_error_continuation_option(Some(value));
        assert!(evaluate(Some(&request), None));
        result
            .validate_request(&request)
            .expect("Continue, Stop, and Undo are assigned option values");
    }

    for value in [0, 4, 0x8000_0001] {
        let request = request_message_with_batch_error_continuation_option(Some(value));
        assert!(!evaluate(Some(&request), None));
        let error = result
            .validate_request(&request)
            .expect_err("unassigned and extension values are not outbound profile values");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
    }
}

#[test]
fn profile_requires_attestation_capable_indicator_when_client_can_create_credential() {
    // KMIP Specification v2.1 §9.3 requires True exactly when this client can
    // create an Attestation Credential; an absent indicator has effective value False.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.3-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.3-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_requires_attestation_capable_indicator_when_client_can_create_credential";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Attestation Capable Indicator behavior has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the indicator behavior resolves to generated requirement metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("required indicator behavior has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id == PROFILE_TEST_ID && evidence.kind == RuleTestKind::PositiveBehavior
    }));
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id
            == "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::execute_advertises_attestation_without_authentication_or_credential_payload"
            && evidence.kind == RuleTestKind::PositiveBehavior
    }));
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id
            == "crates/kmipkit-client/tests/unit/attestation_indicator_tests.rs::async_follow_up_request_advertises_attestation_capability"
            && evidence.kind == RuleTestKind::PositiveBehavior
    }));

    let evaluate = mapping
        .evaluator
        .expect("the indicator requirement has a typed request evaluator");
    assert!(!evaluate(None, None));
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    let advertised = request_message_with_attestation_capable_indicator(Some(true));
    assert_eq!(
        advertised.header().attestation_capable_indicator(),
        Some(true)
    );
    assert!(evaluate(Some(&advertised), None));
    result
        .validate_request(&advertised)
        .expect("a capable client advertises the true indicator");

    for indicator in [None, Some(false)] {
        let request = request_message_with_attestation_capable_indicator(indicator);
        assert!(!evaluate(Some(&request), None));
        let error = result
            .validate_request(&request)
            .expect_err("a capable client cannot omit or clear the indicator");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
    }
}

#[test]
fn profile_maps_batch_error_continuation_option_only_for_multi_item_batches() {
    // KMIP Specification v2.1 §9.6 permits this option only when Batch Count > 1.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.6-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.6-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the Batch Error Continuation Option cardinality rule has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect(
            "the Batch Error Continuation Option cardinality rule resolves to generated metadata",
        );
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the requirement has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let single_item_request = request_message();
    assert_eq!(single_item_request.header().batch_count(), 1);
    assert_eq!(
        single_item_request
            .header()
            .batch_error_continuation_option(),
        None
    );
    assert!(evaluate(Some(&single_item_request), None));
    result
        .validate_request(&single_item_request)
        .expect("a single-item request omits the option");

    let multi_item_request_without_option =
        request_message_with_batch_error_continuation_option(None);
    assert_eq!(multi_item_request_without_option.header().batch_count(), 2);
    assert_eq!(
        multi_item_request_without_option
            .header()
            .batch_error_continuation_option(),
        None
    );
    assert!(evaluate(Some(&multi_item_request_without_option), None));
    result
        .validate_request(&multi_item_request_without_option)
        .expect("a multi-item request may omit the optional field");

    let multi_item_request_with_option =
        request_message_with_batch_error_continuation_option(Some(1));
    assert_eq!(multi_item_request_with_option.header().batch_count(), 2);
    assert_eq!(
        multi_item_request_with_option
            .header()
            .batch_error_continuation_option(),
        Some(1)
    );
    assert!(evaluate(Some(&multi_item_request_with_option), None));
    result
        .validate_request(&multi_item_request_with_option)
        .expect("a multi-item request may include the option");
}

#[test]
fn profile_maps_client_get_key_compression_type_selector() {
    // KMIP Specification v2.1 §3.1 permits Key Compression Type in a Key Block;
    // §6.1.19 Table 220 exposes the caller-selected Get request field.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-3.1-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const GET_OPERATION: u32 = 0x0000_000A;
    const KEY_COMPRESSION_TYPE_TAG: u32 = 0x0042_0041;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Key Compression Type has an explicit production profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("Key Compression Type resolves to generated normative metadata");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the rule is linked to this capability and its positive test");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    assert!(!client_get_or_export_key_compression_type_is_enumeration(
        None, None
    ));
    let selected_compression = request_message_with_operation_payload(
        GET_OPERATION,
        crate::GetRequest::new()
            .with_unique_identifier(crate::UniqueIdentifier::TextString("ec-key-id".to_owned()))
            .with_key_compression_type(crate::KeyCompressionType::from_raw(1))
            .to_ttlv_payload()
            .expect("the typed Get request encodes the caller-selected value"),
    );
    result
        .validate_request(&selected_compression)
        .expect("the selected client profile permits an explicit compression selector");
    assert!(client_get_or_export_key_compression_type_is_enumeration(
        Some(&selected_compression),
        None
    ));

    let omitted_optional_selector = request_message_with_operation_payload(
        GET_OPERATION,
        crate::GetRequest::new()
            .with_unique_identifier(crate::UniqueIdentifier::TextString("ec-key-id".to_owned()))
            .to_ttlv_payload()
            .expect("the optional compression selector remains absent by default"),
    );
    result
        .validate_request(&omitted_optional_selector)
        .expect("profile validation does not synthesize or require this optional selector");
    assert!(client_get_or_export_key_compression_type_is_enumeration(
        Some(&omitted_optional_selector),
        None
    ));

    let malformed_selector = request_message_with_operation_payload(
        GET_OPERATION,
        structure([
            tagged(0x0042_0094, Value::text_string("ec-key-id".to_owned())),
            tagged(
                KEY_COMPRESSION_TYPE_TAG,
                Value::text_string("compressed".to_owned()),
            ),
        ]),
    );
    assert!(result.validate_request(&malformed_selector).is_err());
    assert!(!client_get_or_export_key_compression_type_is_enumeration(
        Some(&malformed_selector),
        None
    ));

    let duplicate_selector = request_message_with_operation_payload(
        GET_OPERATION,
        structure([
            tagged(0x0042_0094, Value::text_string("ec-key-id".to_owned())),
            tagged(KEY_COMPRESSION_TYPE_TAG, Value::enumeration(1)),
            tagged(KEY_COMPRESSION_TYPE_TAG, Value::enumeration(2)),
        ]),
    );
    assert!(result.validate_request(&duplicate_selector).is_err());
    assert!(!client_get_or_export_key_compression_type_is_enumeration(
        Some(&duplicate_selector),
        None
    ));

    let unrelated_operation = request_message_with_operation_payload(0x0000_0001, Structure::new());
    result
        .validate_request(&unrelated_operation)
        .expect("the retrieval selector rule leaves other operations unchanged");
    assert!(client_get_or_export_key_compression_type_is_enumeration(
        Some(&unrelated_operation),
        None
    ));
}

#[test]
fn profile_maps_client_export_key_compression_type_selector() {
    // KMIP Specification v2.1 §3.1 permits Key Compression Type in a Key Block;
    // §6.1.18 Table 217 exposes the caller-selected Export request field.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-3.1-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const EXPORT_OPERATION: u32 = 0x0000_002B;
    const KEY_COMPRESSION_TYPE_TAG: u32 = 0x0042_0041;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Key Compression Type has an explicit production profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("Key Compression Type resolves to generated normative metadata");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the rule links the Get and Export selector evidence");

    let selected_compression = request_message_with_operation_payload(
        EXPORT_OPERATION,
        crate::ExportRequest::new()
            .with_unique_identifier(crate::UniqueIdentifier::TextString("ec-key-id".to_owned()))
            .with_key_compression_type(crate::KeyCompressionType::from_raw(0xFFFF_FF01))
            .into_ttlv_payload()
            .expect("the typed Export request encodes the caller-selected value"),
    );
    result
        .validate_request(&selected_compression)
        .expect("the profile permits an explicit Export compression selector");
    assert!(client_get_or_export_key_compression_type_is_enumeration(
        Some(&selected_compression),
        None
    ));

    let omitted_optional_selector = request_message_with_operation_payload(
        EXPORT_OPERATION,
        crate::ExportRequest::new()
            .with_unique_identifier(crate::UniqueIdentifier::TextString("ec-key-id".to_owned()))
            .into_ttlv_payload()
            .expect("the optional Export selector remains absent by default"),
    );
    result
        .validate_request(&omitted_optional_selector)
        .expect("profile validation does not synthesize or require the optional selector");
    assert!(client_get_or_export_key_compression_type_is_enumeration(
        Some(&omitted_optional_selector),
        None
    ));

    let malformed_selector = request_message_with_operation_payload(
        EXPORT_OPERATION,
        structure([
            tagged(0x0042_0094, Value::text_string("ec-key-id".to_owned())),
            tagged(
                KEY_COMPRESSION_TYPE_TAG,
                Value::text_string("compressed".to_owned()),
            ),
        ]),
    );
    assert!(result.validate_request(&malformed_selector).is_err());

    let duplicate_selector = request_message_with_operation_payload(
        EXPORT_OPERATION,
        structure([
            tagged(0x0042_0094, Value::text_string("ec-key-id".to_owned())),
            tagged(KEY_COMPRESSION_TYPE_TAG, Value::enumeration(1)),
            tagged(KEY_COMPRESSION_TYPE_TAG, Value::enumeration(2)),
        ]),
    );
    assert!(result.validate_request(&duplicate_selector).is_err());

    let unrelated_operation = request_message_with_operation_payload(0x0000_0001, Structure::new());
    result
        .validate_request(&unrelated_operation)
        .expect("the selector rule does not constrain unrelated operations");
}

#[test]
fn profile_maps_client_pgp_key_forms_and_versions_as_opaque_bytes() {
    // KMIP Specification v2.1 §2.4/Table 5 permits PGP Key versions 3 and 4;
    // §2.4 also recommends preserving its Key Block as caller-owned opaque data.
    const REQUIREMENT_OBJECT_FORM: &str = "KMIPKIT-REQ-SPEC-2.4-001-001";
    const REQUIREMENT_VERSION: &str = "KMIPKIT-REQ-SPEC-2.4-001-002";
    const REQUIREMENT_OPAQUE_KEY_BLOCK: &str = "KMIPKIT-REQ-SPEC-2.4-002-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let requirement_ids = [
        REQUIREMENT_OBJECT_FORM,
        REQUIREMENT_VERSION,
        REQUIREMENT_OPAQUE_KEY_BLOCK,
    ];
    for requirement_id in requirement_ids {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id)
            .expect("the PGP Key capability has an explicit production profile rule");
        let requirement = CLIENT_REQUIREMENTS
            .iter()
            .find(|requirement| requirement.requirement_id == requirement_id)
            .expect("the PGP Key capability resolves to normative metadata");
        validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
            .expect("the PGP Key rule has source-linked positive evidence");
    }

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: requirement_ids.map(str::to_owned).to_vec(),
    };
    let public_key_only = pgp_key_import_request(3, b"synthetic-pgp-public-key-only");
    let public_and_private_key = pgp_key_import_request(4, b"synthetic-pgp-public-and-private-key");

    result
        .validate_request(&public_key_only)
        .expect("profile validation preserves caller-supplied PGP version 3 key bytes");
    result
        .validate_request(&public_and_private_key)
        .expect("profile validation preserves caller-supplied PGP version 4 key bytes");

    let malformed_key_value = request_message_with_operation_payload(
        0x0000_002A,
        structure([
            tagged(0x0042_0057, Value::enumeration(9)),
            tagged(
                0x0042_00BD,
                Value::structure(structure([
                    tagged(0x0042_00BE, Value::integer(3)),
                    tagged(
                        0x0042_0040,
                        Value::structure(structure([
                            tagged(0x0042_0042, Value::enumeration(1)),
                            tagged(0x0042_0045, Value::text_string("not bytes".to_owned())),
                        ])),
                    ),
                ])),
            ),
        ]),
    );
    assert!(result.validate_request(&malformed_key_value).is_err());
    assert!(!import_preserves_pgp_key_form_and_opaque_bytes(
        Some(&malformed_key_value),
        None
    ));
    assert!(!import_preserves_pgp_key_form_and_opaque_bytes(None, None));
    assert!(import_preserves_pgp_key_form_and_opaque_bytes(
        Some(&request_message_with_operation_payload(
            0x0000_0003,
            Structure::new()
        )),
        None
    ));
}

#[test]
fn profile_rejects_malformed_pgp_key_import_shapes() {
    let valid_version = || pgp_key_version_item(Value::integer(3));
    let valid_block = || valid_pgp_key_block_item();
    let key_without_version = pgp_key_item([valid_block()]);
    let key_with_wrong_version = pgp_key_item([
        pgp_key_version_item(Value::text_string("three".to_owned())),
        valid_block(),
    ]);
    let key_with_duplicate_versions =
        pgp_key_item([valid_version(), valid_version(), valid_block()]);
    let key_without_block = pgp_key_item([valid_version()]);
    let key_with_wrong_block = pgp_key_item([
        valid_version(),
        tagged(
            0x0042_0040,
            Value::text_string("not a structure".to_owned()),
        ),
    ]);
    let key_with_missing_format = pgp_key_item([
        valid_version(),
        pgp_key_block_item([pgp_key_value_item(Value::byte_string(Vec::new()))]),
    ]);
    let key_with_wrong_format = pgp_key_item([
        valid_version(),
        pgp_key_block_item([
            pgp_key_format_item(Value::integer(1)),
            pgp_key_value_item(Value::byte_string(Vec::new())),
        ]),
    ]);
    let key_with_duplicate_formats = pgp_key_item([
        valid_version(),
        pgp_key_block_item([
            pgp_key_format_item(Value::enumeration(1)),
            pgp_key_format_item(Value::enumeration(1)),
            pgp_key_value_item(Value::byte_string(Vec::new())),
        ]),
    ]);
    let key_with_missing_value = pgp_key_item([
        valid_version(),
        pgp_key_block_item([pgp_key_format_item(Value::enumeration(1))]),
    ]);
    let key_with_duplicate_values = pgp_key_item([
        valid_version(),
        pgp_key_block_item([
            pgp_key_format_item(Value::enumeration(1)),
            pgp_key_value_item(Value::byte_string(Vec::new())),
            pgp_key_value_item(Value::byte_string(Vec::new())),
        ]),
    ]);

    let malformed_requests = [
        request_message_with_operation_payload(0x0000_002A, Structure::new()),
        request_message_with_operation_payload(
            0x0000_002A,
            structure([tagged(
                0x0042_0057,
                Value::text_string("not an enumeration".to_owned()),
            )]),
        ),
        request_message_with_operation_payload(
            0x0000_002A,
            structure([
                tagged(0x0042_0057, Value::enumeration(9)),
                tagged(0x0042_0057, Value::enumeration(9)),
            ]),
        ),
        pgp_import_request_with_fields([]),
        pgp_import_request_with_fields([tagged(
            0x0042_00BD,
            Value::text_string("not a structure".to_owned()),
        )]),
        pgp_import_request_with_fields([valid_pgp_key_item(), valid_pgp_key_item()]),
        pgp_import_request_with_fields([key_without_version]),
        pgp_import_request_with_fields([key_with_wrong_version]),
        pgp_import_request_with_fields([key_with_duplicate_versions]),
        pgp_import_request_with_fields([key_without_block]),
        pgp_import_request_with_fields([key_with_wrong_block]),
        pgp_import_request_with_fields([key_with_missing_format]),
        pgp_import_request_with_fields([key_with_wrong_format]),
        pgp_import_request_with_fields([key_with_duplicate_formats]),
        pgp_import_request_with_fields([key_with_missing_value]),
        pgp_import_request_with_fields([key_with_duplicate_values]),
    ];

    for malformed in malformed_requests {
        assert!(!import_preserves_pgp_key_form_and_opaque_bytes(
            Some(&malformed),
            None
        ));
    }
    assert!(import_preserves_pgp_key_form_and_opaque_bytes(
        Some(&request_message_with_operation_payload(
            0x0000_002A,
            structure([tagged(0x0042_0057, Value::enumeration(2))]),
        )),
        None
    ));
}

#[test]
fn profile_allows_type_specific_cryptographic_parameters_without_remote_object_inference() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.16-001-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    assert!(!type_specific_parameters_do_not_infer_remote_object_type(
        None, None
    ));

    let parameters = structure([tagged(
        CRYPTOGRAPHIC_PARAMETERS_TAG,
        Value::structure(structure([tagged(
            DIGITAL_SIGNATURE_ALGORITHM_TAG,
            Value::enumeration(1),
        )])),
    )]);
    let payload = crate::EncryptRequest::new(
        Some(crate::UniqueIdentifier::TextString(
            "profile-key-id".to_owned(),
        )),
        Some(crate::OperationData::Integer(7)),
    )
    .with_cryptographic_parameters(parameters)
    .to_ttlv_payload()
    .expect("the caller-supplied type-specific parameter is structurally valid");
    let request = request_message_with_operation_payload(0x0000_001F, payload);

    result
        .validate_request(&request)
        .expect("profile validation does not infer a server-held object type");

    let omitted_parameters = request_message_with_operation_payload(
        0x0000_001F,
        crate::EncryptRequest::new(
            Some(crate::UniqueIdentifier::TextString(
                "profile-key-id".to_owned(),
            )),
            Some(crate::OperationData::Integer(7)),
        )
        .to_ttlv_payload()
        .expect("omitted optional parameters remain a valid Encrypt request"),
    );
    result
        .validate_request(&omitted_parameters)
        .expect("the optional Cryptographic Parameters structure may be omitted");

    let other_operation = request_message_with_operation_payload(0x0000_0001, Structure::new());
    result
        .validate_request(&other_operation)
        .expect("the rule leaves unrelated operation payloads unaffected");
}

#[test]
fn profile_maps_client_optional_cryptographic_parameters_for_encrypt_and_decrypt() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.16-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.16-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ENCRYPT_OPERATION: u32 = 0x0000_001F;
    const DECRYPT_OPERATION: u32 = 0x0000_0020;
    const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("optional Cryptographic Parameters has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the optional fields resolve to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("omission evidence matches the OPTIONAL category");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let requests = [
        request_message_with_operation_payload(
            ENCRYPT_OPERATION,
            crate::EncryptRequest::new(
                Some(crate::UniqueIdentifier::TextString(
                    "profile-key-id".to_owned(),
                )),
                Some(crate::OperationData::ByteString(crate::SecretBytes::new(
                    vec![0x41, 0x42],
                ))),
            )
            .to_ttlv_payload()
            .expect("Encrypt payload retains omitted Cryptographic Parameters"),
        ),
        request_message_with_operation_payload(
            DECRYPT_OPERATION,
            crate::DecryptRequest::new(
                Some(crate::UniqueIdentifier::TextString(
                    "profile-key-id".to_owned(),
                )),
                Some(crate::OperationData::ByteString(crate::SecretBytes::new(
                    vec![0x24, 0x68],
                ))),
            )
            .to_ttlv_payload()
            .expect("Decrypt payload retains omitted Cryptographic Parameters"),
        ),
    ];

    for request in requests {
        let item = request
            .batch_items()
            .next()
            .expect("the test request contains one batch item");
        let has_parameters = item
            .with_request_payload(|payload| {
                payload
                    .children()
                    .iter()
                    .any(|field| field.tag().raw() == CRYPTOGRAPHIC_PARAMETERS_TAG)
            })
            .expect("the test request contains a Request Payload");
        assert!(!has_parameters);
        result
            .validate_request(&request)
            .expect("profile validation permits caller omission of optional parameters");
    }
}

fn variable_iv_profile_rule() -> (RuleEvaluator, ProfileValidationResult) {
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == VARIABLE_IV_REQUIREMENT_ID)
        .expect("variable-IV Length has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == VARIABLE_IV_REQUIREMENT_ID)
        .expect("the variable-IV rule resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, VARIABLE_IV_SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the SHALL has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("variable-IV modes have a typed evaluator");
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![VARIABLE_IV_REQUIREMENT_ID.to_owned()],
    };
    (evaluate, result)
}

#[test]
fn profile_maps_variable_iv_modes_to_required_iv_length() {
    // KMIP Specification v2.1 §4.16 requires IV Length for variable-IV modes,
    // including CTR and GCM. Existing NIST-based parameter validation also
    // classifies CCM as variable-IV. GCM Tag Length remains a separate rule.
    const ENCRYPT_OPERATION: u32 = 0x0000_001F;
    const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
    const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
    const IV_LENGTH_TAG: u32 = 0x0042_00CD;
    const TAG_LENGTH_TAG: u32 = 0x0042_00CE;
    const CBC_MODE: u32 = 1;
    const CTR_MODE: u32 = 6;
    const CCM_MODE: u32 = 8;
    const GCM_MODE: u32 = 9;

    let (evaluate, result) = variable_iv_profile_rule();

    let request_with_parameters = |mode: u32, include_iv_length: bool, include_tag_length: bool| {
        let mut parameters = vec![tagged(BLOCK_CIPHER_MODE_TAG, Value::enumeration(mode))];
        if include_iv_length {
            parameters.push(tagged(IV_LENGTH_TAG, Value::integer(96)));
        }
        if include_tag_length {
            parameters.push(tagged(TAG_LENGTH_TAG, Value::integer(16)));
        }
        request_message_with_operation_payload(
            ENCRYPT_OPERATION,
            structure([tagged(
                CRYPTOGRAPHIC_PARAMETERS_TAG,
                Value::structure(structure(parameters)),
            )]),
        )
    };

    for (mode, include_tag_length) in [(CTR_MODE, false), (CCM_MODE, false), (GCM_MODE, true)] {
        let valid = request_with_parameters(mode, true, include_tag_length);
        assert!(evaluate(Some(&valid), None));
        result
            .validate_request(&valid)
            .expect("a variable-IV mode passes when IV Length is supplied");

        let invalid = request_with_parameters(mode, false, include_tag_length);
        assert!(!evaluate(Some(&invalid), None));
        let error = result
            .validate_request(&invalid)
            .expect_err("a variable-IV mode without IV Length violates §4.16");
        assert_eq!(error.requirement_id(), Some(VARIABLE_IV_REQUIREMENT_ID));
        assert_eq!(error.source_clause_id(), Some(VARIABLE_IV_SOURCE_CLAUSE_ID));
    }

    let parameters_omitted =
        request_message_with_operation_payload(ENCRYPT_OPERATION, Structure::new());
    assert!(evaluate(Some(&parameters_omitted), None));
    result
        .validate_request(&parameters_omitted)
        .expect("operation-level Cryptographic Parameters may be omitted");

    let fixed_iv_mode = request_with_parameters(CBC_MODE, false, false);
    assert!(evaluate(Some(&fixed_iv_mode), None));
    result
        .validate_request(&fixed_iv_mode)
        .expect("CBC does not require variable IV Length under §4.16");
}

#[test]
fn profile_checks_variable_iv_length_in_nested_register_attributes() {
    const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
    const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
    const IV_LENGTH_TAG: u32 = 0x0042_00CD;
    const TAG_LENGTH_TAG: u32 = 0x0042_00CE;
    const GCM_MODE: u32 = 9;

    let (evaluate, result) = variable_iv_profile_rule();
    let nested_register_request = |include_iv_length: bool| {
        let mut parameters = vec![tagged(BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE))];
        if include_iv_length {
            parameters.push(tagged(IV_LENGTH_TAG, Value::integer(96)));
        }
        parameters.push(tagged(TAG_LENGTH_TAG, Value::integer(16)));
        request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            structure([tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    CRYPTOGRAPHIC_PARAMETERS_TAG,
                    Value::structure(structure(parameters)),
                )])),
            )]),
        )
    };

    let nested_valid = nested_register_request(true);
    assert!(evaluate(Some(&nested_valid), None));
    result
        .validate_request(&nested_valid)
        .expect("nested Register parameters pass when IV Length is supplied");

    let nested_invalid = nested_register_request(false);
    assert!(!evaluate(Some(&nested_invalid), None));
    result
        .validate_request(&nested_invalid)
        .expect_err("nested Register parameters without IV Length violate §4.16");
}

#[test]
fn profile_rejects_invalid_ttlv_types_for_variable_iv_rule() {
    const ENCRYPT_OPERATION: u32 = 0x0000_001F;
    const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
    const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
    const IV_LENGTH_TAG: u32 = 0x0042_00CD;
    const CTR_MODE: u32 = 6;

    let (evaluate, result) = variable_iv_profile_rule();
    let request_with_parameters = |parameters| {
        request_message_with_operation_payload(
            ENCRYPT_OPERATION,
            structure([tagged(
                CRYPTOGRAPHIC_PARAMETERS_TAG,
                Value::structure(structure(parameters)),
            )]),
        )
    };

    let invalid_iv_type = request_with_parameters([
        tagged(BLOCK_CIPHER_MODE_TAG, Value::enumeration(CTR_MODE)),
        tagged(
            IV_LENGTH_TAG,
            Value::text_string("96 is not an Integer".to_owned()),
        ),
    ]);
    assert!(!evaluate(Some(&invalid_iv_type), None));
    assert_eq!(
        result
            .validate_request(&invalid_iv_type)
            .expect_err("IV Length with a non-Integer TTLV type is invalid")
            .requirement_id(),
        Some(VARIABLE_IV_REQUIREMENT_ID)
    );

    let invalid_mode_type = request_with_parameters([
        tagged(
            BLOCK_CIPHER_MODE_TAG,
            Value::text_string("CTR is not an Enumeration".to_owned()),
        ),
        tagged(IV_LENGTH_TAG, Value::integer(96)),
    ]);
    assert!(!evaluate(Some(&invalid_mode_type), None));
    assert_eq!(
        result
            .validate_request(&invalid_mode_type)
            .expect_err("Block Cipher Mode with a non-Enumeration TTLV type is invalid")
            .requirement_id(),
        Some(VARIABLE_IV_REQUIREMENT_ID)
    );
}

#[test]
fn profile_maps_gcm_tag_length_as_required_behavior() {
    // §4.16 requires Tag Length for GCM; §11.6 Table 436 assigns GCM value 9.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.16-003";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.16-003";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ENCRYPT_OPERATION: u32 = 0x0000_001F;
    const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
    const IV_LENGTH_TAG: u32 = 0x0042_00CD;
    const TAG_LENGTH_TAG: u32 = 0x0042_00CE;
    const GCM_MODE: u32 = 9;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("GCM Tag Length has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the conditional GCM rule resolves to generated metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the SHALL has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the required GCM behavior has a typed evaluator");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let gcm_request = request_message_with_operation_payload(
        ENCRYPT_OPERATION,
        crate::EncryptRequest::new(
            Some(crate::UniqueIdentifier::TextString(
                "profile-key-id".to_owned(),
            )),
            Some(crate::OperationData::Integer(7)),
        )
        .with_cryptographic_parameters(structure([
            tagged(BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
            tagged(IV_LENGTH_TAG, Value::integer(96)),
            tagged(TAG_LENGTH_TAG, Value::integer(16)),
        ]))
        .to_ttlv_payload()
        .expect("GCM request has the required IV Length and Tag Length"),
    );
    assert!(evaluate(Some(&gcm_request), None));
    result
        .validate_request(&gcm_request)
        .expect("GCM may be requested when Tag Length is supplied");

    let parameters_omitted = request_message_with_operation_payload(
        ENCRYPT_OPERATION,
        crate::EncryptRequest::new(
            Some(crate::UniqueIdentifier::TextString(
                "profile-key-id".to_owned(),
            )),
            Some(crate::OperationData::Integer(7)),
        )
        .to_ttlv_payload()
        .expect("operation-level Cryptographic Parameters may be omitted"),
    );
    assert!(evaluate(Some(&parameters_omitted), None));
    result
        .validate_request(&parameters_omitted)
        .expect("the conditional GCM rule does not require operation-level parameters");
}

#[test]
fn profile_rejects_non_structure_cryptographic_parameters_for_algorithm_constraints() {
    const ENCRYPT_OPERATION: u32 = 0x0000_001F;
    const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    for requirement_id in ["KMIPKIT-REQ-SPEC-4.16-002", "KMIPKIT-REQ-SPEC-4.16-003"] {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id)
            .expect("the algorithm parameter constraint has a typed mapping");
        let evaluate = mapping
            .evaluator
            .expect("the algorithm parameter constraint has an evaluator");
        let request = request_message_with_operation_payload(
            ENCRYPT_OPERATION,
            structure([tagged(CRYPTOGRAPHIC_PARAMETERS_TAG, Value::integer(1))]),
        );
        let result = ProfileValidationResult {
            profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
            requirement_ids: vec![requirement_id.to_owned()],
        };

        assert!(!evaluate(Some(&request), None));
        assert_eq!(
            result
                .validate_request(&request)
                .expect_err("Cryptographic Parameters must use the Structure TTLV type")
                .requirement_id(),
            Some(requirement_id)
        );
    }
}

#[test]
fn profile_validates_nested_register_gcm_tag_length() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.16-003";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const BLOCK_CIPHER_MODE_TAG: u32 = 0x0042_0011;
    const IV_LENGTH_TAG: u32 = 0x0042_00CD;
    const TAG_LENGTH_TAG: u32 = 0x0042_00CE;
    const CRYPTOGRAPHIC_PARAMETERS_TAG: u32 = 0x0042_002B;
    const GCM_MODE: u32 = 9;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let evaluate = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .and_then(|mapping| mapping.evaluator)
        .expect("GCM Tag Length has a typed profile evaluator");
    let register_request = |include_tag_length: bool| {
        let mut parameters = vec![
            tagged(BLOCK_CIPHER_MODE_TAG, Value::enumeration(GCM_MODE)),
            tagged(IV_LENGTH_TAG, Value::integer(96)),
        ];
        if include_tag_length {
            parameters.push(tagged(TAG_LENGTH_TAG, Value::integer(16)));
        }

        request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            structure([tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    CRYPTOGRAPHIC_PARAMETERS_TAG,
                    Value::structure(structure(parameters)),
                )])),
            )]),
        )
    };

    let valid = register_request(true);
    assert!(evaluate(Some(&valid), None));
    result
        .validate_request(&valid)
        .expect("nested GCM parameters pass when Tag Length is supplied");

    let invalid = register_request(false);
    assert!(!evaluate(Some(&invalid), None));
    let error = result
        .validate_request(&invalid)
        .expect_err("nested GCM parameters without Tag Length violate §4.16");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
}

#[test]
fn profile_maps_client_supplied_create_key_pair_domain_parameters() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.14-001-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
    const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
    const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
    const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
    const QLENGTH: u32 = 0x0042_0073;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let domain_parameters = structure([tagged(QLENGTH, Value::integer(256))]);
    let common_attributes = structure([
        // KMIP Specification v2.1 §11.12, Table 443: DSA is enumeration 5.
        tagged(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(5)),
        tagged(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(domain_parameters),
        ),
    ]);
    let payload = structure([tagged(
        COMMON_ATTRIBUTES,
        Value::structure(common_attributes),
    )]);
    let request = request_message_with_operation_payload(CREATE_KEY_PAIR_OPERATION, payload);

    result
        .validate_request(&request)
        .expect("the optional Qlength value is usable in a Create Key Pair request");

    let omitted =
        request_message_with_operation_payload(CREATE_KEY_PAIR_OPERATION, Structure::new());
    result
        .validate_request(&omitted)
        .expect("the client may omit Cryptographic Domain Parameters");

    let unrelated = request_message_with_operation_payload(0x0000_0001, Structure::new());
    result
        .validate_request(&unrelated)
        .expect("the Create Key Pair rule leaves other operations unaffected");

    let malformed_domain_parameters =
        structure([tagged(CRYPTOGRAPHIC_DOMAIN_PARAMETERS, Value::integer(256))]);
    let malformed_attributes = structure([tagged(
        COMMON_ATTRIBUTES,
        Value::structure(malformed_domain_parameters),
    )]);
    let malformed =
        request_message_with_operation_payload(CREATE_KEY_PAIR_OPERATION, malformed_attributes);
    let error = result
        .validate_request(&malformed)
        .expect_err("Table 55 requires Cryptographic Domain Parameters to be a Structure");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-4.14-001")
    );
}

#[test]
fn profile_maps_client_type_applicable_dsa_q_length_domain_parameter() {
    // KMIP Specification v2.1 §4.14 and Tables 55–56; §11.12, Table 443.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.14-001-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
    const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
    const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
    const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
    const QLENGTH: u32 = 0x0042_0073;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("type-applicable domain parameters have an explicit production rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the type-applicable field resolves to generated normative metadata");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the typed rule links this requirement to its optional-capability evidence");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let parameters = structure([tagged(QLENGTH, Value::integer(256))]);
    let common_attributes = structure([
        // KMIP Specification v2.1 §11.12, Table 443: DSA is enumeration 5.
        tagged(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(5)),
        tagged(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(parameters),
        ),
    ]);
    let request = request_message_with_operation_payload(
        CREATE_KEY_PAIR_OPERATION,
        structure([tagged(
            COMMON_ATTRIBUTES,
            Value::structure(common_attributes),
        )]),
    );
    result
        .validate_request(&request)
        .expect("the caller-selected DSA Qlength Integer is usable in Create Key Pair");

    let omitted =
        request_message_with_operation_payload(CREATE_KEY_PAIR_OPERATION, Structure::new());
    result
        .validate_request(&omitted)
        .expect("the client may omit the optional domain-parameter field");

    let malformed_parameters = structure([tagged(QLENGTH, Value::text_string("256".to_owned()))]);
    let malformed_attributes = structure([
        tagged(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(5)),
        tagged(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(malformed_parameters),
        ),
    ]);
    let malformed = request_message_with_operation_payload(
        CREATE_KEY_PAIR_OPERATION,
        structure([tagged(
            COMMON_ATTRIBUTES,
            Value::structure(malformed_attributes),
        )]),
    );
    let error = result
        .validate_request(&malformed)
        .expect_err("Table 55 encodes Qlength as an Integer");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-4.14-001")
    );
}

#[test]
fn profile_rejects_non_integer_q_length_in_client_supplied_domain_parameters() {
    // KMIP Specification v2.1 §4.14, Table 55; KMIPKIT-REQ-SPEC-4.14-001-001.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.14-001-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const CREATE_KEY_PAIR_OPERATION: u32 = 0x0000_0002;
    const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
    const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
    const CRYPTOGRAPHIC_DOMAIN_PARAMETERS: u32 = 0x0042_0029;
    const QLENGTH: u32 = 0x0042_0073;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let domain_parameters = structure([tagged(QLENGTH, Value::text_string("256".to_owned()))]);
    let common_attributes = structure([
        tagged(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(5)),
        tagged(
            CRYPTOGRAPHIC_DOMAIN_PARAMETERS,
            Value::structure(domain_parameters),
        ),
    ]);
    let payload = structure([tagged(
        COMMON_ATTRIBUTES,
        Value::structure(common_attributes),
    )]);
    let request = request_message_with_operation_payload(CREATE_KEY_PAIR_OPERATION, payload);

    let error = result
        .validate_request(&request)
        .expect_err("Table 55 encodes Qlength as an Integer");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-4.14-001")
    );
}

#[test]
fn public_profile_result_entrypoints_validate_transport_and_responses() {
    // KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1; KMIPKIT-0010 FR-014.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.16-001-001";
    const PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    result
        .validate_transport(ProfileTransport::Https)
        .expect("the Discover Versions rule has no incompatible transport constraint");

    let supported_request = request_message_with_operation_payload(
        0x0000_001E,
        crate::DiscoverVersionsRequest::new()
            .to_ttlv_payload()
            .expect("the typed request advertises KMIP 2.1"),
    );
    result
        .validate_response(&supported_request, &response_message())
        .expect("a KMIP 2.1 Discover Versions response satisfies the selected rule");

    let unsupported_request =
        request_message_with_operation_payload(0x0000_001E, discover_versions_payload(&[(3, 0)]));
    let error = result
        .validate_response(&unsupported_request, &response_message())
        .expect_err("a response cannot validate a request advertising an unsupported version");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ResponseRuleRejected
    );
    assert_eq!(error.profile_id(), Some(PROFILE_ID));
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-6.1.16-001")
    );

    let unmapped = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let error = unmapped
        .validate_transport(ProfileTransport::Https)
        .expect_err("a selected result with a missing typed mapping fails closed");
    assert_eq!(error.code(), ProfileValidationErrorCode::MissingRuleMapping);
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), None);
}

#[test]
fn runtime_rule_evaluation_rejects_missing_and_incomplete_mappings() {
    // KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1; KMIPKIT-0010 FR-014.
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };

    let missing = result
        .evaluate_rules(
            None,
            None,
            &[],
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect_err("runtime evaluation fails closed when a mapping is absent");
    assert_eq!(
        missing.code(),
        ProfileValidationErrorCode::MissingRuleMapping
    );
    assert_eq!(missing.source_clause_id(), None);

    let incomplete = [TypedRuleMapping {
        evaluator: None,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    }];
    let error = result
        .evaluate_rules(
            None,
            None,
            &incomplete,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect_err("runtime evaluation rejects a mapping without executable code");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::IncompleteRuleMapping
    );
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));
}

#[test]
fn selection_rejects_a_requirement_without_a_typed_rule_mapping() {
    // KMIP Specification v2.1 §14.1 and KMIP Profiles v2.1 §6.1; KMIPKIT-0010 FR-014.
    let profiles = [profile("TEST-PROFILE", &["TEST-REQ"], &[])];
    let requirements = [requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        false,
    )];
    let error = validate_selection_with_registry(
        &ProfileValidationSelection::new(["TEST-PROFILE"]),
        &profiles,
        &requirements,
        &[],
    )
    .expect_err("selection fails closed when its applicable rule has no typed mapping");

    assert_eq!(error.code(), ProfileValidationErrorCode::MissingRuleMapping);
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));
}

#[test]
fn discover_versions_protocol_version_requires_one_integer_major_minor_pair() {
    let valid = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(protocol_version_is_kmip_2_1(&valid.view()));

    let missing_minor = structure([tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2))]);
    assert!(!protocol_version_is_kmip_2_1(&missing_minor.view()));

    let duplicate_major = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&duplicate_major.view()));

    let duplicate_minor = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&duplicate_minor.view()));

    let wrong_major_type = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::enumeration(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&wrong_major_type.view()));

    let wrong_minor_type = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::enumeration(1)),
    ]);
    assert!(!protocol_version_is_kmip_2_1(&wrong_minor_type.view()));

    let unknown_version_field = structure([
        tagged(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        tagged(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
        tagged(PROTOCOL_VERSION_TAG, Value::integer(99)),
    ]);
    assert!(protocol_version_is_kmip_2_1(&unknown_version_field.view()));
}

#[test]
fn accepted_deviations_cover_only_recommended_and_discouraged_categories() {
    for category in [
        GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        GeneratedVerificationCategory::DiscouragedAbsenceOrDeviation,
    ] {
        let requirement = requirement("TEST-REQ", category, true);
        let mapping = TypedRuleMapping {
            test_evidence: &[],
            ..mapping("TEST-REQ", category, RuleTestKind::PositiveBehavior, &[])
        };
        assert!(validate_mapping("TEST-PROFILE", &requirement, &mapping).is_ok());
    }

    let requirement = requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        true,
    );
    let mapping = TypedRuleMapping {
        test_evidence: &[],
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    };
    let error = validate_mapping("TEST-PROFILE", &requirement, &mapping)
        .expect_err("a deviation cannot replace required behavior verification");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::MissingCategoryTest
    );
}

#[test]
fn a_complete_synthetic_mapping_allows_selection_without_target_membership() {
    let profiles = [profile("TEST-PROFILE", &["TEST-REQ"], &[])];
    let requirements = [requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        false,
    )];
    let mappings = [mapping(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        RuleTestKind::PositiveBehavior,
        &CONSTRAINTS_HTTPS,
    )];
    let selection = ProfileValidationSelection::new(["TEST-PROFILE"]);
    assert_eq!(&["TEST-PROFILE".to_owned()], selection.profile_ids());
    assert_eq!(
        format!("{selection:?}"),
        "ProfileValidationSelection { profile_count: 1 }"
    );

    let result = validate_selection_with_registry(&selection, &profiles, &requirements, &mappings)
        .expect("complete synthetic code and verification evidence is selectable");

    assert_eq!(result.profile_ids(), &["TEST-PROFILE"]);
    assert_eq!(result.requirement_ids(), &["TEST-REQ"]);
}

#[test]
fn mapped_request_and_response_rejections_keep_stable_source_metadata() {
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let required = requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        false,
    );
    let outbound_mapping = [TypedRuleMapping {
        evaluator: Some(reject_outbound_rule),
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    }];
    assert!(validate_mapping("TEST-PROFILE", &required, &outbound_mapping[0]).is_ok());
    let request = request_message();
    let error = result
        .evaluate_rules(
            Some(&request),
            None,
            &outbound_mapping,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect_err("an outbound rule can reject the request before transport");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));

    let response_mapping = [TypedRuleMapping {
        evaluator: Some(reject_response_rule),
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    }];
    assert!(validate_mapping("TEST-PROFILE", &required, &response_mapping[0]).is_ok());
    let error = result
        .evaluate_rules(
            Some(&request),
            Some((&request, &response_message())),
            &response_mapping,
            ProfileValidationErrorCode::ResponseRuleRejected,
        )
        .expect_err("a response rule can reject a received response");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ResponseRuleRejected
    );
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));
}

#[test]
fn minimum_capability_rules_do_not_reject_an_extra_base_protocol_operation() {
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let mapping = [mapping(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        RuleTestKind::PositiveBehavior,
        &[],
    )];
    let request = request_message();
    assert_eq!(
        request
            .batch_items()
            .next()
            .and_then(crate::RequestBatchItemView::operation),
        Some(0x1E),
        "the synthetic extra operation is Discover Versions"
    );

    result
        .evaluate_rules(
            Some(&request),
            None,
            &mapping,
            ProfileValidationErrorCode::RequestRuleRejected,
        )
        .expect("an operation outside a profile minimum is not an allowlist violation");
}

#[test]
fn profile_requires_tls_for_basic_authentication_suite_transport() {
    // KMIP Profiles v2.1 §3.1; Baseline Client inherits this condition through §6.1.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-PROF-3.1-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    for transport in [ProfileTransport::RawTtlvTls, ProfileTransport::Https] {
        result
            .validate_transport(transport)
            .expect("both configured KMIPKit transport classes satisfy the TLS requirement");
    }
    let request = request_message();
    result
        .validate_request(&request)
        .expect("a transport rule does not add a request-payload restriction");
    result
        .validate_response(&request, &response_message())
        .expect("a transport rule does not add a response-payload restriction");
}

#[test]
fn profile_recommends_tls_1_3_for_basic_authentication_suite_transport() {
    // KMIP Profiles v2.1 §3.1.1 paragraph 2; Baseline inherits §3.1 via §6.1 item 2.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-PROF-3.1.1-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    for transport in [ProfileTransport::RawTtlvTls, ProfileTransport::Https] {
        result
            .validate_transport(transport)
            .expect("both configured KMIPKit transport classes satisfy the TLS 1.3 recommendation");
    }
    let request = request_message();
    result
        .validate_request(&request)
        .expect("a transport recommendation does not add a request-payload restriction");
    result
        .validate_response(&request, &response_message())
        .expect("a transport recommendation does not add a response-payload restriction");
}

#[test]
fn configured_tls13_constraint_accepts_both_tls_client_transports() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-PROF-3.1.1-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let mapping = [TypedRuleMapping {
        requirement_id: REQUIREMENT_ID,
        source_clause_id: "KMIPKIT-CLAUSE-PROF-3.1.1-002",
        source_gap_id: None,
        verification_category: GeneratedVerificationCategory::RecommendedBehaviorOrDeviation,
        evaluator: None,
        test_evidence: &TLS13_RECOMMENDED_TEST,
        constraints: &CONSTRAINTS_TLS13,
    }];

    for transport in [ProfileTransport::RawTtlvTls, ProfileTransport::Https] {
        result
            .validate_transport_with_mappings(transport, &mapping)
            .expect("TLS 1.3 is configured for both supported client transports");
    }
    assert!(!ProfileTransport::Https.satisfies_transport_security("tls12"));
}

#[test]
fn profile_maps_prohibited_legacy_tls_and_ssl_protocols_to_tls13_transport() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-PROF-3.1.1-005";
    const CLAUSE_ID: &str = "KMIPKIT-CLAUSE-PROF-3.1.1-005";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("legacy TLS and SSL protocol prohibition has a typed rule");

    assert_eq!(mapping.source_clause_id, CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::ProhibitedBehavior
    );
    assert!(mapping.evaluator.is_none());
    assert_eq!(mapping.test_evidence, &TLS_LEGACY_PROHIBITION_TEST);
    assert_eq!(mapping.constraints, &CONSTRAINTS_TLS13);

    for transport in [ProfileTransport::RawTtlvTls, ProfileTransport::Https] {
        result
            .validate_transport_with_mappings(transport, std::slice::from_ref(mapping))
            .expect("both client transports use TLS 1.3 and exclude legacy protocols");
    }
}

#[test]
fn profile_maps_required_tls13_cipher_suite_support() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-PROF-3.1.2-002";
    const CLAUSE_ID: &str = "KMIPKIT-CLAUSE-PROF-3.1.2-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("TLS 1.3 cipher-suite support has a typed rule");

    assert_eq!(mapping.source_clause_id, CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    assert!(mapping.evaluator.is_none());
    assert_eq!(
        mapping.test_evidence,
        &TLS13_PROFILE_CIPHER_SUITE_SUPPORT_TESTS
    );
    assert_eq!(mapping.constraints, &TLS13_PROFILE_CIPHER_SUITES);
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the TLS 1.3 cipher-suite support requirement is generated");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the typed TLS 1.3 cipher-suite support rule is valid");

    for transport in [ProfileTransport::RawTtlvTls, ProfileTransport::Https] {
        result
            .validate_transport_with_mappings(transport, std::slice::from_ref(mapping))
            .expect("both configured transports use the KMIP Profiles TLS 1.3 suites");
    }
    result
        .validate_request(&request_message())
        .expect("cipher-suite constraints do not add a request-payload restriction");
}

#[test]
fn profile_maps_prohibited_tls13_cipher_suites() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-PROF-3.1.2-005";
    const CLAUSE_ID: &str = "KMIPKIT-CLAUSE-PROF-3.1.2-005";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("unlisted TLS cipher-suite prohibition has a typed rule");

    assert_eq!(mapping.source_clause_id, CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::ProhibitedBehavior
    );
    assert!(mapping.evaluator.is_none());
    assert_eq!(
        mapping.test_evidence,
        &TLS13_UNLISTED_CIPHER_SUITE_PROHIBITION_TESTS
    );
    assert_eq!(mapping.constraints, &TLS13_PROFILE_CIPHER_SUITES);
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the cipher-suite prohibition requirement is generated");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the typed TLS 1.3 cipher-suite prohibition rule is valid");

    for transport in [ProfileTransport::RawTtlvTls, ProfileTransport::Https] {
        result
            .validate_transport_with_mappings(transport, std::slice::from_ref(mapping))
            .expect("both configured transports exclude unlisted TLS 1.3 suites");
    }
    result
        .validate_request(&request_message())
        .expect("cipher-suite prohibitions do not add a request-payload restriction");
}

#[test]
fn selected_transport_constraints_match_or_reject_with_source_ids() {
    let result = ProfileValidationResult {
        profile_ids: vec!["TEST-PROFILE".to_owned()],
        requirement_ids: vec!["TEST-REQ".to_owned()],
    };
    let mappings = [mapping(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        RuleTestKind::PositiveBehavior,
        &CONSTRAINTS_HTTPS,
    )];

    result
        .validate_transport_with_mappings(ProfileTransport::Https, &mappings)
        .expect("HTTPS satisfies its selected typed constraint");
    let error = result
        .validate_transport_with_mappings(ProfileTransport::RawTtlvTls, &mappings)
        .expect_err("raw TTLV/TLS violates the selected HTTPS constraint");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::TransportConstraintRejected
    );
    assert_eq!(error.profile_id(), Some("TEST-PROFILE"));
    assert_eq!(error.requirement_id(), Some("TEST-REQ"));
    assert_eq!(error.source_clause_id(), Some("TEST-CLAUSE"));

    let encoding_mapping = [mapping(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        RuleTestKind::PositiveBehavior,
        &CONSTRAINTS_TTLV,
    )];
    result
        .validate_transport_with_mappings(ProfileTransport::Https, &encoding_mapping)
        .expect("an encoding constraint does not reject a selected transport");
}

#[test]
fn tls_transport_rule_can_be_validated_without_a_message_evaluator() {
    assert!(!ProfileTransport::Https.satisfies_transport_security("cleartext"));
    let profiles = [profile("TEST-PROFILE", &["TEST-REQ"], &[])];
    let requirements = [requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        false,
    )];
    let mapping = TypedRuleMapping {
        evaluator: None,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &CONSTRAINTS_TLS,
        )
    };
    let result = validate_selection_with_registry(
        &ProfileValidationSelection::new(["TEST-PROFILE"]),
        &profiles,
        &requirements,
        &[mapping],
    )
    .expect("the selected TLS transport constraint is an executable rule mapping");

    for transport in [ProfileTransport::RawTtlvTls, ProfileTransport::Https] {
        result
            .validate_transport_with_mappings(transport, &[mapping])
            .expect("both KMIPKit transport variants provide TLS");
    }
}

#[test]
fn dependency_cycles_and_constraint_conflicts_fail_deterministically() {
    let cyclic_profiles = [
        profile("TEST-A", &[], &["TEST-B"]),
        profile("TEST-B", &[], &["TEST-A"]),
    ];
    let cycle_error = validate_selection_with_registry(
        &ProfileValidationSelection::new(["TEST-A"]),
        &cyclic_profiles,
        &[],
        &[],
    )
    .expect_err("cyclic dependencies are rejected");
    assert_eq!(
        cycle_error.code(),
        ProfileValidationErrorCode::DependencyCycle
    );

    let profiles = [
        profile("TEST-A", &["TEST-REQ-A"], &[]),
        profile("TEST-B", &["TEST-REQ-B"], &[]),
    ];
    let requirements = [
        requirement(
            "TEST-REQ-A",
            GeneratedVerificationCategory::RequiredBehavior,
            false,
        ),
        requirement(
            "TEST-REQ-B",
            GeneratedVerificationCategory::RequiredBehavior,
            false,
        ),
    ];
    let mappings = [
        mapping(
            "TEST-REQ-A",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &CONSTRAINTS_HTTPS,
        ),
        mapping(
            "TEST-REQ-B",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &CONSTRAINTS_RAW_TLS,
        ),
    ];
    let error = validate_selection_with_registry(
        &ProfileValidationSelection::new(["TEST-B", "TEST-A"]),
        &profiles,
        &requirements,
        &mappings,
    )
    .expect_err("conflicting selected constraints have no order-based resolution");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ConflictingConstraints
    );
}

#[test]
fn missing_evaluator_and_wrong_source_link_fail_closed() {
    static UNSUPPORTED_SECURITY: [RuleConstraint; 1] = [RuleConstraint {
        key: "transport_security",
        value: "cleartext",
    }];
    let requirement = requirement(
        "TEST-REQ",
        GeneratedVerificationCategory::RequiredBehavior,
        false,
    );
    let missing_evaluator = TypedRuleMapping {
        evaluator: None,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    };
    let error = validate_mapping("TEST-PROFILE", &requirement, &missing_evaluator)
        .expect_err("rule metadata without executable code is incomplete");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::IncompleteRuleMapping
    );

    let unsupported_transport_only = TypedRuleMapping {
        evaluator: None,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &UNSUPPORTED_SECURITY,
        )
    };
    let error = validate_mapping("TEST-PROFILE", &requirement, &unsupported_transport_only)
        .expect_err("a transport-only mapping rejects an unsupported security class");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RuleMappingMismatch
    );

    let encoding_constraint = TypedRuleMapping {
        evaluator: None,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &CONSTRAINTS_TTLV,
        )
    };
    validate_mapping("TEST-PROFILE", &requirement, &encoding_constraint)
        .expect("a TTLV-only encoding rule has byte-level positive test evidence");

    let wrong_clause = TypedRuleMapping {
        source_clause_id: "OTHER-CLAUSE",
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    };
    let error = validate_mapping("TEST-PROFILE", &requirement, &wrong_clause)
        .expect_err("rule source links must match the catalog");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RuleMappingMismatch
    );

    let untyped_constraint = TypedRuleMapping {
        constraints: &UNKNOWN_CONSTRAINT,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &[],
        )
    };
    let error = validate_mapping("TEST-PROFILE", &requirement, &untyped_constraint)
        .expect_err("unknown constraint keys cannot appear executable");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RuleMappingMismatch
    );
}

#[test]
fn validation_error_codes_and_readiness_blockers_have_stable_text() {
    let validation_codes = [
        (
            ProfileValidationErrorCode::EmptySelection,
            "empty_selection",
        ),
        (
            ProfileValidationErrorCode::DuplicateProfile,
            "duplicate_profile",
        ),
        (
            ProfileValidationErrorCode::UnknownProfile,
            "unknown_profile",
        ),
        (
            ProfileValidationErrorCode::ProfileUnavailable,
            "profile_unavailable",
        ),
        (
            ProfileValidationErrorCode::ConditionalProfileUnsatisfied,
            "conditional_profile_unsatisfied",
        ),
        (
            ProfileValidationErrorCode::DependencyCycle,
            "dependency_cycle",
        ),
        (
            ProfileValidationErrorCode::MissingRequirementMetadata,
            "missing_requirement_metadata",
        ),
        (
            ProfileValidationErrorCode::UnresolvedRequirementScope,
            "unresolved_requirement_scope",
        ),
        (
            ProfileValidationErrorCode::MissingRuleMapping,
            "missing_rule_mapping",
        ),
        (
            ProfileValidationErrorCode::IncompleteRuleMapping,
            "incomplete_rule_mapping",
        ),
        (
            ProfileValidationErrorCode::RuleMappingMismatch,
            "rule_mapping_mismatch",
        ),
        (
            ProfileValidationErrorCode::MissingCategoryTest,
            "missing_category_test",
        ),
        (
            ProfileValidationErrorCode::ConflictingConstraints,
            "conflicting_constraints",
        ),
        (
            ProfileValidationErrorCode::RequestRuleRejected,
            "request_rule_rejected",
        ),
        (
            ProfileValidationErrorCode::ResponseRuleRejected,
            "response_rule_rejected",
        ),
        (
            ProfileValidationErrorCode::TransportConstraintRejected,
            "transport_constraint_rejected",
        ),
    ];
    for (code, expected) in validation_codes {
        assert_eq!(expected, code.as_str());
    }
}

#[test]
fn readiness_blockers_have_stable_text_and_requirement_ids() {
    let blockers = [
        (
            ProfileReadinessBlocker::MissingRequirementRecord {
                requirement_id: "REQ-MISSING",
            },
            "required requirement evidence is missing",
            Some("REQ-MISSING"),
        ),
        (
            ProfileReadinessBlocker::FailedRequirementRecord {
                requirement_id: "REQ-FAILED",
            },
            "required requirement evidence failed",
            Some("REQ-FAILED"),
        ),
        (
            ProfileReadinessBlocker::DuplicateRequirementRecord {
                requirement_id: "REQ-DUPLICATE",
            },
            "requirement evidence record is duplicated",
            Some("REQ-DUPLICATE"),
        ),
        (
            ProfileReadinessBlocker::UnknownRequirementRecord,
            "requirement evidence record is unknown",
            None,
        ),
        (
            ProfileReadinessBlocker::RuntimeValidationUnavailable,
            "runtime profile validation is unavailable",
            None,
        ),
        (
            ProfileReadinessBlocker::MandatoryFixtureUnavailable {
                test_id: "FIXTURE-1",
            },
            "mandatory profile fixture is unavailable",
            None,
        ),
        (
            ProfileReadinessBlocker::ConditionalProfileUnsatisfied,
            "conditional profile is unsatisfied",
            None,
        ),
        (
            ProfileReadinessBlocker::DependencyEvidenceIncomplete {
                profile_id: "PROFILE-DEPENDENCY",
            },
            "dependency profile evidence is incomplete",
            None,
        ),
        (
            ProfileReadinessBlocker::VerificationManifestUnavailable,
            "current verification manifest is unavailable",
            None,
        ),
        (
            ProfileReadinessBlocker::StaleCatalogEvidenceState,
            "catalog evidence-complete state is stale",
            None,
        ),
        (
            ProfileReadinessBlocker::CatalogEvidenceReviewRequired,
            "catalog evidence state requires human review",
            None,
        ),
        (
            ProfileReadinessBlocker::ProfileUnavailable,
            "profile is outside the KMIPKit 1.0 client scope",
            None,
        ),
    ];
    for (blocker, expected, requirement_id) in blockers {
        assert_eq!(expected, blocker.as_str());
        assert_eq!(requirement_id, blocker.requirement_id());
        assert_eq!(None, blocker.source_clause_id());
    }
    let unresolved = ProfileReadinessBlocker::UnresolvedRequirementScope {
        requirement_id: "REQ-SCOPE-UNCLEAR",
        source_clause_id: "CLAUSE-SCOPE-UNCLEAR",
    };
    assert_eq!(
        "profile requirement applicability is unresolved",
        unresolved.as_str()
    );
    assert_eq!(Some("REQ-SCOPE-UNCLEAR"), unresolved.requirement_id());
    assert_eq!(Some("CLAUSE-SCOPE-UNCLEAR"), unresolved.source_clause_id());
}

#[test]
fn validation_error_display_contains_only_stable_identifiers() {
    let code_only = ProfileValidationError::new(ProfileValidationErrorCode::EmptySelection);
    assert_eq!(
        "profile validation failed: empty_selection",
        code_only.to_string()
    );
    assert_eq!(None, code_only.profile_id());
    assert_eq!(None, code_only.requirement_id());
    assert_eq!(None, code_only.source_clause_id());

    let profile = ProfileValidationError::for_profile(
        ProfileValidationErrorCode::UnknownProfile,
        "TEST-PROFILE",
    );
    assert_eq!(Some("TEST-PROFILE"), profile.profile_id());
    assert_eq!(
        "profile validation failed: unknown_profile; profile TEST-PROFILE",
        profile.to_string()
    );

    let requirement = ProfileValidationError::for_requirement(
        ProfileValidationErrorCode::MissingRuleMapping,
        "TEST-PROFILE",
        "TEST-REQ",
        Some("TEST-CLAUSE"),
    );
    assert_eq!(Some("TEST-REQ"), requirement.requirement_id());
    assert_eq!(Some("TEST-CLAUSE"), requirement.source_clause_id());
    assert_eq!(
        "profile validation failed: missing_rule_mapping; profile TEST-PROFILE; requirement TEST-REQ; source clause TEST-CLAUSE",
        requirement.to_string()
    );

    assert!(write!(&mut RejectingWriter, "{code_only}").is_err());
}

#[test]
fn readiness_records_classify_duplicates_and_unknown_ids() {
    let baseline = super::CLIENT_PROFILES
        .iter()
        .find(|profile| profile.profile_id == "KMIPKIT-PROFILE-BASELINE-CLIENT")
        .expect("generated catalog contains Baseline Client");
    let first = baseline.requirement_ids[0];
    let records: Vec<_> = baseline
        .requirement_ids
        .iter()
        .map(|requirement_id| ProfileEvidenceRecord::passed(*requirement_id))
        .chain([ProfileEvidenceRecord::passed(first)])
        .chain([ProfileEvidenceRecord::passed("UNKNOWN-REQ")])
        .collect();

    let readiness = ProfileReadiness::evaluate(baseline.profile_id, &records)
        .expect("generated Baseline Client resolves");

    assert!(!readiness.requirement_records_complete());
    assert!(
        readiness
            .blockers()
            .contains(&ProfileReadinessBlocker::DuplicateRequirementRecord {
                requirement_id: first
            })
    );
    assert!(
        readiness
            .blockers()
            .contains(&ProfileReadinessBlocker::UnknownRequirementRecord)
    );
    assert!(readiness.target_selected());
    assert!(!readiness.runtime_validation_available());
    assert!(
        readiness
            .blockers()
            .contains(&ProfileReadinessBlocker::RuntimeValidationUnavailable)
    );
    assert_eq!(baseline.profile_id, readiness.profile_id());
    assert_eq!("evidence_incomplete", readiness.effective_state());
    assert!(!readiness.published_claim());
    assert!(ProfileReadiness::evaluate("UNKNOWN-PROFILE", &[]).is_none());

    let failed =
        ProfileReadiness::evaluate(baseline.profile_id, &[ProfileEvidenceRecord::failed(first)])
            .expect("generated Baseline Client resolves");
    assert!(
        failed
            .blockers()
            .contains(&ProfileReadinessBlocker::FailedRequirementRecord {
                requirement_id: first
            })
    );
}

#[test]
fn readiness_blocks_conditional_out_of_scope_and_dependent_profiles() {
    let conditional = ProfileReadiness::evaluate("KMIPKIT-PROFILE-HTTPS-CLIENT", &[])
        .expect("generated HTTPS Client resolves");
    assert!(
        conditional
            .blockers()
            .contains(&ProfileReadinessBlocker::ConditionalProfileUnsatisfied)
    );

    let unavailable = ProfileReadiness::evaluate("KMIPKIT-PROFILE-JSON-CLIENT", &[])
        .expect("generated JSON Client resolves");
    assert!(
        unavailable
            .blockers()
            .contains(&ProfileReadinessBlocker::ProfileUnavailable)
    );

    let dependent = ProfileReadiness::evaluate("KMIPKIT-PROFILE-AES-XTS-CLIENT", &[])
        .expect("generated AES-XTS Client resolves");
    assert!(dependent.blockers().contains(
        &ProfileReadinessBlocker::DependencyEvidenceIncomplete {
            profile_id: "KMIPKIT-PROFILE-BASELINE-CLIENT"
        }
    ));
}

#[test]
fn readiness_reconciles_catalog_states_with_blockers() {
    let mut stale = vec![ProfileReadinessBlocker::RuntimeValidationUnavailable];
    assert_eq!(
        "evidence_incomplete",
        reconcile_effective_state("evidence_complete", &mut stale)
    );
    assert!(stale.contains(&ProfileReadinessBlocker::StaleCatalogEvidenceState));

    let mut review_required = Vec::new();
    assert_eq!(
        "not_claimed",
        reconcile_effective_state("not_claimed", &mut review_required)
    );
    assert_eq!(
        vec![ProfileReadinessBlocker::CatalogEvidenceReviewRequired],
        review_required
    );

    let repeated = ProfileReadinessBlocker::VerificationManifestUnavailable;
    let mut unchanged_with_duplicates = vec![repeated, repeated];
    assert_eq!(
        "evidence_incomplete",
        reconcile_effective_state("evidence_incomplete", &mut unchanged_with_duplicates)
    );
    assert_eq!(vec![repeated], unchanged_with_duplicates);
}

#[test]
fn selection_rejects_missing_catalog_metadata_and_skips_resolved_dependencies() {
    let empty = validate_selection_with_registry(
        &ProfileValidationSelection::new(std::iter::empty::<&str>()),
        &[],
        &[],
        &[],
    )
    .expect_err("an empty explicit profile selection is invalid");
    assert_eq!(ProfileValidationErrorCode::EmptySelection, empty.code());

    let unknown_duplicate = validate_selection_with_registry(
        &ProfileValidationSelection::new(["UNKNOWN", "UNKNOWN"]),
        &[],
        &[],
        &[],
    )
    .expect_err("duplicate unknown identifiers are still duplicates");
    assert_eq!(
        ProfileValidationErrorCode::DuplicateProfile,
        unknown_duplicate.code()
    );
    assert_eq!(None, unknown_duplicate.profile_id());

    let known_duplicate = validate_selection_with_registry(
        &ProfileValidationSelection::new(["TEST-PROFILE", "TEST-PROFILE"]),
        &[profile("TEST-PROFILE", &[], &[])],
        &[],
        &[],
    )
    .expect_err("a known profile cannot be selected twice");
    assert_eq!(
        ProfileValidationErrorCode::DuplicateProfile,
        known_duplicate.code()
    );
    assert_eq!(Some("TEST-PROFILE"), known_duplicate.profile_id());

    let unknown = validate_selection_with_registry(
        &ProfileValidationSelection::new(["UNKNOWN"]),
        &[],
        &[],
        &[],
    )
    .expect_err("unknown profiles do not resolve");
    assert_eq!(ProfileValidationErrorCode::UnknownProfile, unknown.code());

    let empty_requirements = [profile("EMPTY", &[], &[])];
    let missing_list = validate_selection_with_registry(
        &ProfileValidationSelection::new(["EMPTY"]),
        &empty_requirements,
        &[],
        &[],
    )
    .expect_err("a selected profile must list applicable requirements");
    assert_eq!(
        ProfileValidationErrorCode::MissingRequirementMetadata,
        missing_list.code()
    );

    let missing_record_profiles = [profile("MISSING-RECORD", &["TEST-REQ"], &[])];
    let missing_record = validate_selection_with_registry(
        &ProfileValidationSelection::new(["MISSING-RECORD"]),
        &missing_record_profiles,
        &[],
        &[],
    )
    .expect_err("every profile requirement must resolve to metadata");
    assert_eq!(
        ProfileValidationErrorCode::MissingRequirementMetadata,
        missing_record.code()
    );

    let shared_dependency_profiles = [
        profile("SHARED", &[], &[]),
        profile("ROOT-A", &[], &["SHARED"]),
        profile("ROOT-B", &[], &["SHARED"]),
    ];
    let dependency_error = validate_selection_with_registry(
        &ProfileValidationSelection::new(["ROOT-A", "ROOT-B"]),
        &shared_dependency_profiles,
        &[],
        &[],
    )
    .expect_err("empty requirement metadata remains fail-closed after closure resolution");
    assert_eq!(
        ProfileValidationErrorCode::MissingRequirementMetadata,
        dependency_error.code()
    );
}

#[test]
fn passing_evaluator_is_safe_for_absent_and_present_exchange_contexts() {
    assert!(passing_rule(None, None));
}

fn secret_data_import_request(
    key_value: Value,
    wrapping_data: Option<Value>,
) -> crate::RequestMessage {
    const IMPORT_OPERATION: u32 = 0x0000_002A;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const SECRET_DATA: u32 = 0x0042_0085;
    const SECRET_DATA_TYPE: u32 = 0x0042_0086;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_VALUE: u32 = 0x0042_0045;
    const KEY_WRAPPING_DATA: u32 = 0x0042_0046;

    let mut key_block_fields = vec![
        tagged(KEY_FORMAT_TYPE, Value::enumeration(1)),
        tagged(KEY_VALUE, key_value),
    ];
    if let Some(wrapping_data) = wrapping_data {
        key_block_fields.push(tagged(KEY_WRAPPING_DATA, wrapping_data));
    }

    request_message_with_operation_payload(
        IMPORT_OPERATION,
        structure([
            tagged(
                UNIQUE_IDENTIFIER,
                Value::text_string("secret-data".to_owned()),
            ),
            tagged(OBJECT_TYPE, Value::enumeration(7)),
            tagged(ATTRIBUTES, Value::structure(Structure::new())),
            tagged(
                SECRET_DATA,
                Value::structure(structure([
                    tagged(SECRET_DATA_TYPE, Value::enumeration(1)),
                    tagged(KEY_BLOCK, Value::structure(structure(key_block_fields))),
                ])),
            ),
        ]),
    )
}

fn pgp_key_import_request(version: i32, key_bytes: &[u8]) -> crate::RequestMessage {
    const IMPORT_OPERATION: u32 = 0x0000_002A;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const PGP_KEY: u32 = 0x0042_00BD;
    const PGP_KEY_VERSION: u32 = 0x0042_00BE;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_VALUE: u32 = 0x0042_0045;

    request_message_with_operation_payload(
        IMPORT_OPERATION,
        structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string("pgp-key".to_owned())),
            tagged(OBJECT_TYPE, Value::enumeration(9)),
            tagged(ATTRIBUTES, Value::structure(Structure::new())),
            tagged(
                PGP_KEY,
                Value::structure(structure([
                    tagged(PGP_KEY_VERSION, Value::integer(version)),
                    tagged(
                        KEY_BLOCK,
                        Value::structure(structure([
                            tagged(KEY_FORMAT_TYPE, Value::enumeration(1)),
                            tagged(KEY_VALUE, Value::byte_string(key_bytes.to_vec())),
                        ])),
                    ),
                ])),
            ),
        ]),
    )
}

fn pgp_key_item(fields: impl IntoIterator<Item = Item>) -> Item {
    tagged(0x0042_00BD, Value::structure(structure(fields)))
}

fn pgp_key_block_item(fields: impl IntoIterator<Item = Item>) -> Item {
    tagged(0x0042_0040, Value::structure(structure(fields)))
}

fn pgp_import_request_with_fields(fields: impl IntoIterator<Item = Item>) -> crate::RequestMessage {
    let mut payload = vec![tagged(0x0042_0057, Value::enumeration(9))];
    payload.extend(fields);
    request_message_with_operation_payload(0x0000_002A, structure(payload))
}

fn pgp_key_version_item(value: Value) -> Item {
    tagged(0x0042_00BE, value)
}

fn pgp_key_format_item(value: Value) -> Item {
    tagged(0x0042_0042, value)
}

fn pgp_key_value_item(value: Value) -> Item {
    tagged(0x0042_0045, value)
}

fn valid_pgp_key_block_item() -> Item {
    pgp_key_block_item([
        pgp_key_format_item(Value::enumeration(1)),
        pgp_key_value_item(Value::byte_string(b"synthetic-pgp-key".to_vec())),
    ])
}

fn valid_pgp_key_item() -> Item {
    pgp_key_item([
        pgp_key_version_item(Value::integer(3)),
        valid_pgp_key_block_item(),
    ])
}

fn valid_secret_data_wrapping() -> Value {
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
    const WRAPPING_METHOD: u32 = 0x0042_009E;
    const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;

    Value::structure(structure([
        tagged(WRAPPING_METHOD, Value::enumeration(1)),
        tagged(
            ENCRYPTION_KEY_INFORMATION,
            Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER,
                Value::text_string("wrapping-key".to_owned()),
            )])),
        ),
    ]))
}

fn wrapping_data_without_key_information() -> Value {
    Value::structure(structure([tagged(0x0042_009E, Value::enumeration(1))]))
}

fn register_wrapped_key_without_wrapping_key_information() -> crate::RequestMessage {
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const ATTRIBUTES: u32 = 0x0042_0125;
    const SYMMETRIC_KEY: u32 = 0x0042_008F;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_VALUE: u32 = 0x0042_0045;
    const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
    const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
    const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;

    request_message_with_operation_payload(
        0x0000_0003,
        structure([
            tagged(OBJECT_TYPE, Value::enumeration(2)),
            tagged(
                ATTRIBUTES,
                Value::structure(structure([
                    tagged(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                    tagged(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
                ])),
            ),
            tagged(
                SYMMETRIC_KEY,
                Value::structure(structure([tagged(
                    KEY_BLOCK,
                    Value::structure(structure([
                        tagged(KEY_FORMAT_TYPE, Value::enumeration(1)),
                        tagged(
                            KEY_VALUE,
                            Value::byte_string(b"synthetic-wrapped-key".to_vec()),
                        ),
                        tagged(KEY_WRAPPING_DATA, wrapping_data_without_key_information()),
                    ])),
                )])),
            ),
        ]),
    )
}

#[test]
fn typed_rule_evaluators_fail_closed_without_a_request() {
    // KMIP Specification v2.1 §§2.7, 4.14, and 4.27; the rules must fail closed
    // when no request is available to evaluate.
    assert!(!import_preserves_wrapped_secret_data_shape(None, None));
    assert!(!client_create_key_pair_domain_parameters_have_valid_wire_types(None, None));
    assert!(!client_register_key_value_location_requires_omitted_key_value(None, None));
    assert!(!client_key_wrapping_data_has_required_key_information(
        None, None
    ));
}

#[test]
fn profile_maps_client_import_of_wrapped_secret_data() {
    // KMIP Specification v2.1 §2.7/Table 8 permits a wrapped Secret Data Key Value;
    // §3.1/Table 12 selects its encoding, and §3.3/Table 14 requires key information.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.7-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let wrapped_secret_data = secret_data_import_request(
        Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
        Some(valid_secret_data_wrapping()),
    );

    result
        .validate_request(&wrapped_secret_data)
        .expect("the selected profile accepts a wrapped Secret Data Import request");

    let unwrapped_secret_data =
        secret_data_import_request(Value::structure(Structure::new()), None);
    result.validate_request(&unwrapped_secret_data).expect(
        "the optional wrapping capability does not require every Secret Data value to be wrapped",
    );

    let invalid_unwrapped_secret_data = secret_data_import_request(
        Value::byte_string(b"synthetic-plaintext-secret-data".to_vec()),
        None,
    );
    let error = result
        .validate_request(&invalid_unwrapped_secret_data)
        .expect_err("plaintext Key Value must use its Structure encoding");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-2.7-001")
    );

    let malformed_wrapping_data = secret_data_import_request(
        Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
        Some(Value::integer(1)),
    );
    let error = result
        .validate_request(&malformed_wrapping_data)
        .expect_err("present Key Wrapping Data must keep its structure type");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-2.7-001")
    );

    let incomplete_wrapping_data = secret_data_import_request(
        Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
        Some(Value::structure(Structure::new())),
    );
    let error = result
        .validate_request(&incomplete_wrapping_data)
        .expect_err("wrapped Key Value requires Encryption or MAC/Signature Key Information");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-2.7-001")
    );
}

#[test]
fn typed_rule_evaluators_tolerate_missing_context() {
    for mapping in TYPED_RULE_MAPPINGS {
        if let Some(evaluator) = mapping.evaluator {
            assert!(
                std::panic::catch_unwind(|| evaluator(None, None)).is_ok(),
                "{} must tolerate absent request and response context",
                mapping.requirement_id
            );
        }
    }
}

#[test]
fn profile_maps_required_key_information_when_wrapping_data_is_supplied() {
    // KMIP Specification v2.1 §3.3/Table 14 requires Encryption Key Information,
    // MAC/Signature Key Information, or both whenever Key Wrapping Data is used.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-3.3-004";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    result
        .validate_request(&register_wrapped_key_value_request(Value::enumeration(2)))
        .expect("the client can Register wrapped key data with key information");
    result
        .validate_request(&secret_data_import_request(
            Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
            Some(valid_secret_data_wrapping()),
        ))
        .expect("the client can Import wrapped Secret Data with key information");
    result
        .validate_request(&secret_data_import_request(
            Value::structure(Structure::new()),
            None,
        ))
        .expect("the conditional key-information requirement does not require wrapping");
    result
        .validate_request(&request_message_with_operation_payload(
            0x0000_0008,
            Structure::new(),
        ))
        .expect("the key-wrapping requirement does not constrain unrelated operations");

    let malformed_import = secret_data_import_request(
        Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
        Some(wrapping_data_without_key_information()),
    );
    let error = result
        .validate_request(&malformed_import)
        .expect_err("wrapped Secret Data requires Encryption or MAC/Signature Key Information");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-3.3-004")
    );

    let wrong_type_import = secret_data_import_request(
        Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
        Some(Value::integer(1)),
    );
    let error = result
        .validate_request(&wrong_type_import)
        .expect_err("Key Wrapping Data must remain a Structure");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-3.3-004")
    );

    let malformed_register = register_wrapped_key_without_wrapping_key_information();
    let error = result
        .validate_request(&malformed_register)
        .expect_err("wrapped Register Key Block requires key information");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-3.3-004")
    );
}

#[test]
fn wrapped_secret_data_rule_fails_closed_for_malformed_payload_shapes() {
    // KMIP Specification v2.1 §§2.7, 3.1, and 3.3; KMIPKIT-REQ-SPEC-2.7-001.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.7-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const IMPORT_OPERATION: u32 = 0x0000_002A;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    for (label, payload) in malformed_secret_data_import_payloads() {
        let error = result
            .validate_request(&request_message_with_operation_payload(
                IMPORT_OPERATION,
                payload,
            ))
            .expect_err(label);
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected,
            "{label}"
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID), "{label}");
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-2.7-001"),
            "{label}"
        );
    }

    result
        .validate_request(&request_message_with_operation_payload(
            0x0000_0003,
            Structure::new(),
        ))
        .expect("an unrelated Register request is outside the Secret Data Import rule");
}

fn malformed_secret_data_import_payloads() -> Vec<(&'static str, Structure)> {
    const OBJECT_TYPE: u32 = 0x0042_0057;
    const SECRET_DATA: u32 = 0x0042_0085;
    const SECRET_DATA_TYPE: u32 = 0x0042_0086;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
    const KEY_VALUE: u32 = 0x0042_0045;
    const KEY_WRAPPING_DATA: u32 = 0x0042_0046;

    let import_payload = |extra_fields: Vec<Item>| {
        let mut fields = vec![tagged(OBJECT_TYPE, Value::enumeration(7))];
        fields.extend(extra_fields);
        structure(fields)
    };
    let secret_data = |fields: Vec<Item>| tagged(SECRET_DATA, Value::structure(structure(fields)));
    let key_block = |fields: Vec<Item>| tagged(KEY_BLOCK, Value::structure(structure(fields)));

    vec![
        ("missing Object Type", structure([secret_data(Vec::new())])),
        (
            "wrong Object Type value type",
            structure([tagged(
                OBJECT_TYPE,
                Value::text_string("Secret Data".to_owned()),
            )]),
        ),
        ("missing Secret Data", import_payload(Vec::new())),
        (
            "wrong Secret Data value type",
            import_payload(vec![tagged(SECRET_DATA, Value::integer(1))]),
        ),
        (
            "missing Secret Data Type",
            import_payload(vec![secret_data(vec![key_block(Vec::new())])]),
        ),
        (
            "wrong Secret Data Type value type",
            import_payload(vec![secret_data(vec![
                tagged(SECRET_DATA_TYPE, Value::text_string("Password".to_owned())),
                key_block(Vec::new()),
            ])]),
        ),
        (
            "missing Key Block",
            import_payload(vec![secret_data(vec![tagged(
                SECRET_DATA_TYPE,
                Value::enumeration(1),
            )])]),
        ),
        (
            "wrong Key Block value type",
            import_payload(vec![secret_data(vec![
                tagged(SECRET_DATA_TYPE, Value::enumeration(1)),
                tagged(KEY_BLOCK, Value::integer(1)),
            ])]),
        ),
        (
            "missing Key Value",
            import_payload(vec![secret_data(vec![
                tagged(SECRET_DATA_TYPE, Value::enumeration(1)),
                key_block(vec![tagged(KEY_FORMAT_TYPE, Value::enumeration(1))]),
            ])]),
        ),
        (
            "duplicate Key Wrapping Data",
            import_payload(vec![secret_data(vec![
                tagged(SECRET_DATA_TYPE, Value::enumeration(1)),
                key_block(vec![
                    tagged(
                        KEY_VALUE,
                        Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
                    ),
                    tagged(KEY_WRAPPING_DATA, valid_secret_data_wrapping()),
                    tagged(KEY_WRAPPING_DATA, valid_secret_data_wrapping()),
                ]),
            ])]),
        ),
    ]
}

#[test]
fn profile_rejects_wrapped_secret_data_without_one_wrapping_method() {
    // KMIP Specification v2.1 §3.3/Table 15 requires one Wrapping Method Enumeration.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.7-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const WRAPPING_METHOD: u32 = 0x0042_009E;
    const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let encryption_key_information = || {
        tagged(
            ENCRYPTION_KEY_INFORMATION,
            Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER,
                Value::text_string("wrapping-key".to_owned()),
            )])),
        )
    };
    let malformed_wrapping_data = [
        Value::structure(structure([encryption_key_information()])),
        Value::structure(structure([
            tagged(WRAPPING_METHOD, Value::integer(1)),
            encryption_key_information(),
        ])),
        Value::structure(structure([
            tagged(WRAPPING_METHOD, Value::enumeration(1)),
            tagged(WRAPPING_METHOD, Value::enumeration(2)),
            encryption_key_information(),
        ])),
    ];

    for wrapping_data in malformed_wrapping_data {
        let request = secret_data_import_request(
            Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
            Some(wrapping_data),
        );
        let error = result
            .validate_request(&request)
            .expect_err("wrapped Key Value must have exactly one Enumeration Wrapping Method");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-2.7-001")
        );
    }
}

#[test]
fn profile_rejects_wrapping_key_information_without_one_text_identifier() {
    // KMIP Specification v2.1 §3.3 Tables 15–17 define the required key-info identifier.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.7-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const WRAPPING_METHOD: u32 = 0x0042_009E;
    const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
    const MAC_SIGNATURE_KEY_INFORMATION: u32 = 0x0042_004E;
    const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let wrapping_data = |information: Vec<Item>| {
        let mut fields = vec![tagged(WRAPPING_METHOD, Value::enumeration(1))];
        fields.extend(information);
        Value::structure(structure(fields))
    };
    let encryption_information = |value: Value| tagged(ENCRYPTION_KEY_INFORMATION, value);
    let mac_signature_information = |value: Value| tagged(MAC_SIGNATURE_KEY_INFORMATION, value);
    let valid_mac_only = secret_data_import_request(
        Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
        Some(wrapping_data(vec![mac_signature_information(
            Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER,
                Value::text_string("mac-key".to_owned()),
            )])),
        )])),
    );
    result
        .validate_request(&valid_mac_only)
        .expect("MAC/Signature Key Information alone is permitted");

    let malformed_wrapping_data = [
        wrapping_data(vec![encryption_information(Value::structure(
            Structure::new(),
        ))]),
        wrapping_data(vec![encryption_information(Value::structure(structure([
            tagged(UNIQUE_IDENTIFIER, Value::integer(7)),
        ])))]),
        wrapping_data(vec![encryption_information(Value::structure(structure([
            tagged(UNIQUE_IDENTIFIER, Value::text_string("one".to_owned())),
            tagged(UNIQUE_IDENTIFIER, Value::text_string("two".to_owned())),
        ])))]),
        wrapping_data(vec![encryption_information(Value::integer(1))]),
        wrapping_data(vec![
            encryption_information(Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER,
                Value::text_string("one".to_owned()),
            )]))),
            encryption_information(Value::structure(structure([tagged(
                UNIQUE_IDENTIFIER,
                Value::text_string("two".to_owned()),
            )]))),
        ]),
    ];

    for data in malformed_wrapping_data {
        let request = secret_data_import_request(
            Value::byte_string(b"synthetic-wrapped-secret-data".to_vec()),
            Some(data),
        );
        let error = result
            .validate_request(&request)
            .expect_err("key information needs one Text String Unique Identifier");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-2.7-001")
        );
    }
}

#[test]
fn profile_maps_client_register_key_value_location_only_without_key_value() {
    // KMIP Specification v2.1 §4.27/Table 82 and §6.1.43/Table 291.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.27-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    let external_key = register_key_value_location_request(true, false);
    result
        .validate_request(&external_key)
        .expect("Register may carry Key Value Location when its Key Block omits Key Value");

    let ordinary_key = register_key_value_location_request(false, true);
    result
        .validate_request(&ordinary_key)
        .expect("Key Value Location remains optional for a request containing Key Value");

    let conflicting_location = register_key_value_location_request(true, true);
    let error = result
        .validate_request(&conflicting_location)
        .expect_err("Key Value Location is scoped to a Register object with no Key Value");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-4.27-001")
    );
}

const TABLE12_REGISTER_OPERATION: u32 = 0x0000_0003;
const TABLE12_IMPORT_OPERATION: u32 = 0x0000_002A;
const TABLE12_GET_OPERATION: u32 = 0x0000_000A;
const TABLE12_OBJECT_TYPE: u32 = 0x0042_0057;
const TABLE12_UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const TABLE12_ATTRIBUTES: u32 = 0x0042_0125;
const TABLE12_KEY_BLOCK: u32 = 0x0042_0040;
const TABLE12_KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const TABLE12_KEY_VALUE: u32 = 0x0042_0045;
const TABLE12_KEY_MATERIAL: u32 = 0x0042_0043;
const TABLE12_CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const TABLE12_CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const DIGITAL_SIGNATURE_ALGORITHM_TAG: u32 = 0x0042_00AE;
const TABLE12_CERTIFICATE: u32 = 0x0042_0013;
const TABLE12_CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const TABLE12_SYMMETRIC_KEY: u32 = 0x0042_008F;
const TABLE12_SYMMETRIC_KEY_OBJECT_TYPE: u32 = 2;
const TABLE12_PUBLIC_KEY: u32 = 0x0042_006D;
const TABLE12_PRIVATE_KEY: u32 = 0x0042_0064;
const TABLE12_SPLIT_KEY: u32 = 0x0042_0089;
const TABLE12_SECRET_DATA: u32 = 0x0042_0085;
const TABLE12_SECRET_DATA_TYPE: u32 = 0x0042_0086;
const TABLE12_OPAQUE_OBJECT: u32 = 0x0042_005B;
const TABLE12_PGP_KEY: u32 = 0x0042_00BD;
const TABLE12_PGP_KEY_VERSION: u32 = 0x0042_00BE;

fn table12_key_block_object(
    object_tag: u32,
    has_algorithm: bool,
    has_length: bool,
    nested_key_value_attribute: Option<Item>,
) -> Item {
    let mut key_value_fields = vec![tagged(
        TABLE12_KEY_MATERIAL,
        Value::byte_string(b"synthetic-key-material".to_vec()),
    )];
    if let Some(attribute) = nested_key_value_attribute {
        key_value_fields.push(tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure([attribute])),
        ));
    }

    let mut key_block_fields = vec![
        tagged(TABLE12_KEY_FORMAT_TYPE, Value::enumeration(1)),
        tagged(
            TABLE12_KEY_VALUE,
            Value::structure(structure(key_value_fields)),
        ),
    ];
    if has_algorithm {
        key_block_fields.push(tagged(
            TABLE12_CRYPTOGRAPHIC_ALGORITHM,
            Value::enumeration(3),
        ));
    }
    if has_length {
        key_block_fields.push(tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)));
    }

    let mut object_fields = vec![tagged(
        TABLE12_KEY_BLOCK,
        Value::structure(structure(key_block_fields)),
    )];
    match object_tag {
        TABLE12_SYMMETRIC_KEY => {}
        TABLE12_SECRET_DATA => {
            object_fields.push(tagged(TABLE12_SECRET_DATA_TYPE, Value::enumeration(1)));
        }
        TABLE12_PGP_KEY => {
            object_fields.push(tagged(TABLE12_PGP_KEY_VERSION, Value::integer(4)));
        }
        _ => unreachable!("Table 12 Key Block fixture uses a supported object type"),
    }
    tagged(object_tag, Value::structure(structure(object_fields)))
}

fn table12_opaque_object() -> Item {
    tagged(
        TABLE12_OPAQUE_OBJECT,
        Value::structure(structure([
            // KMIP Specification v2.1 §11.35 Table 469 reserves 8XXXXXXX for extensions.
            tagged(OPAQUE_DATA_TYPE_TAG, Value::enumeration(0x8000_0001)),
            tagged(
                OPAQUE_DATA_VALUE_TAG,
                Value::byte_string(b"opaque-object-value".to_vec()),
            ),
        ])),
    )
}

fn table12_request_payload(
    operation: u32,
    object: Item,
    outer_attributes: impl IntoIterator<Item = Item>,
) -> Structure {
    let object_type = match object.tag().raw() {
        TABLE12_SYMMETRIC_KEY => TABLE12_SYMMETRIC_KEY_OBJECT_TYPE,
        TABLE12_SECRET_DATA => 7,
        TABLE12_OPAQUE_OBJECT => 8,
        TABLE12_PGP_KEY => 9,
        _ => unreachable!("Table 12 test fixture uses a known object tag"),
    };
    let mut payload = Vec::new();
    if operation == TABLE12_IMPORT_OPERATION {
        payload.push(tagged(
            TABLE12_UNIQUE_IDENTIFIER,
            Value::text_string("imported-object".to_owned()),
        ));
    }
    payload.extend([
        tagged(TABLE12_OBJECT_TYPE, Value::enumeration(object_type)),
        tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure(outer_attributes)),
        ),
        object,
    ]);
    structure(payload)
}

fn table12_profile_result(requirement_ids: &[&str]) -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec!["KMIPKIT-PROFILE-BASELINE-CLIENT".to_owned()],
        requirement_ids: requirement_ids
            .iter()
            .map(|requirement_id| (*requirement_id).to_owned())
            .collect(),
    }
}

fn assert_table12_request_rejected(
    request: &crate::RequestMessage,
    requirement_id: &str,
    source_clause_id: &str,
) {
    let error = table12_profile_result(&[requirement_id])
        .validate_request(request)
        .expect_err("a one-sided direct Key Block pair is rejected");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(requirement_id));
    assert_eq!(error.source_clause_id(), Some(source_clause_id));
}

fn assert_table12_malformed_shape_rejected(request: &crate::RequestMessage) {
    const REQUIREMENT_IDS: [&str; 2] = [
        "KMIPKIT-REQ-SPEC-3.1-004-002",
        "KMIPKIT-REQ-SPEC-3.1-005-002",
    ];
    const ALGORITHM_CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-3.1-004";
    const LENGTH_CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-3.1-005";

    let error = table12_profile_result(&REQUIREMENT_IDS)
        .validate_request(request)
        .expect_err("malformed recognized Key Block shapes fail closed");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert!(matches!(
        (error.requirement_id(), error.source_clause_id()),
        (Some("KMIPKIT-REQ-SPEC-3.1-004-002"), Some(ALGORITHM_CLAUSE))
            | (Some("KMIPKIT-REQ-SPEC-3.1-005-002"), Some(LENGTH_CLAUSE))
    ));
}

#[test]
fn profile_rejects_non_structure_table12_key_block_objects() {
    // KMIP Specification v2.1 §3.1 Table 12 applies to recognized Key Block
    // objects; §6.1.43 Table 291 carries the Register Any Object.
    let malformed_object = tagged(TABLE12_SYMMETRIC_KEY, Value::integer(2));
    let request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(TABLE12_REGISTER_OPERATION, malformed_object, []),
    );

    assert_table12_malformed_shape_rejected(&request);
}

#[test]
fn profile_rejects_non_structure_present_table12_key_blocks() {
    // KMIP Specification v2.1 §3.1 Table 12 checks fields within a Structure Key Block.
    let malformed_key_block_object = tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([tagged(TABLE12_KEY_BLOCK, Value::integer(1))])),
    );
    let request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(TABLE12_REGISTER_OPERATION, malformed_key_block_object, []),
    );

    assert_table12_malformed_shape_rejected(&request);
}

#[test]
fn profile_rejects_duplicate_present_table12_key_blocks() {
    // KMIP Specification v2.1 §3.1 Table 12 applies to the unique direct Key Block.
    let duplicate_key_block_object = tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([
            tagged(TABLE12_KEY_BLOCK, Value::structure(Structure::new())),
            tagged(TABLE12_KEY_BLOCK, Value::structure(Structure::new())),
        ])),
    );
    let request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(TABLE12_REGISTER_OPERATION, duplicate_key_block_object, []),
    );

    assert_table12_malformed_shape_rejected(&request);
}

#[test]
fn profile_rejects_duplicate_direct_table12_pair_fields() {
    // KMIP Specification v2.1 §3.1 Table 12 pairs unique direct Key Block fields.
    let request_with_members = |members: [Item; 3]| {
        let mut key_block_fields = vec![
            tagged(TABLE12_KEY_FORMAT_TYPE, Value::enumeration(1)),
            tagged(
                TABLE12_KEY_VALUE,
                Value::structure(structure([tagged(
                    TABLE12_KEY_MATERIAL,
                    Value::byte_string(b"synthetic-key-material".to_vec()),
                )])),
            ),
        ];
        key_block_fields.extend(members);
        let object = tagged(
            TABLE12_SYMMETRIC_KEY,
            Value::structure(structure([tagged(
                TABLE12_KEY_BLOCK,
                Value::structure(structure(key_block_fields)),
            )])),
        );
        request_message_with_operation_payload(
            TABLE12_REGISTER_OPERATION,
            table12_request_payload(TABLE12_REGISTER_OPERATION, object, []),
        )
    };
    let algorithm = || tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3));
    let length = || tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256));

    for request in [
        request_with_members([algorithm(), algorithm(), length()]),
        request_with_members([algorithm(), length(), length()]),
    ] {
        assert_table12_malformed_shape_rejected(&request);
    }
}

#[test]
fn profile_allows_table12_register_key_value_location_without_key_block() {
    // KMIP Specification v2.1 §4.27/Table 82 permits Key Value Location;
    // §6.1.43/Table 291 scopes Register. Table 12 does not require a Key Block.
    const KEY_VALUE_LOCATION: u32 = 0x0042_00B8;
    const KEY_VALUE_LOCATION_VALUE: u32 = 0x0042_00B9;
    const KEY_VALUE_LOCATION_TYPE: u32 = 0x0042_00BA;
    const REQUIREMENT_IDS: [&str; 2] = [
        "KMIPKIT-REQ-SPEC-3.1-004-002",
        "KMIPKIT-REQ-SPEC-3.1-005-002",
    ];

    let request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(TABLE12_OBJECT_TYPE, Value::enumeration(2)),
            tagged(
                TABLE12_ATTRIBUTES,
                Value::structure(structure([tagged(
                    KEY_VALUE_LOCATION,
                    Value::structure(structure([
                        tagged(
                            KEY_VALUE_LOCATION_VALUE,
                            Value::text_string("pkcs11:object=key-1".to_owned()),
                        ),
                        tagged(KEY_VALUE_LOCATION_TYPE, Value::enumeration(2)),
                    ])),
                )])),
            ),
            tagged(TABLE12_SYMMETRIC_KEY, Value::structure(Structure::new())),
        ]),
    );

    table12_profile_result(&REQUIREMENT_IDS)
        .validate_request(&request)
        .expect("Table 12 pairing is inapplicable when Register has no Key Block");
}

#[test]
fn profile_maps_key_block_algorithm_length_pairing_as_required_behavior() {
    // KMIP Specification v2.1 §3.1 Table 12 defines the pair; §§6.1.25 Table 238
    // and 6.1.43 Table 291 establish Import and Register request applicability.
    const CASES: [(&str, &str); 2] = [
        (
            "KMIPKIT-REQ-SPEC-3.1-004-002",
            "KMIPKIT-CLAUSE-SPEC-3.1-004",
        ),
        (
            "KMIPKIT-REQ-SPEC-3.1-005-002",
            "KMIPKIT-CLAUSE-SPEC-3.1-005",
        ),
    ];

    for (requirement_id, source_clause_id) in CASES {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id)
            .expect("Table 12 pairing has a typed profile rule");
        let requirement = CLIENT_REQUIREMENTS
            .iter()
            .find(|requirement| requirement.requirement_id == requirement_id)
            .expect("Table 12 pairing resolves to generated normative metadata");
        assert_eq!(mapping.source_clause_id, source_clause_id);
        assert_eq!(
            mapping.verification_category,
            GeneratedVerificationCategory::RequiredBehavior
        );
        validate_mapping("KMIPKIT-PROFILE-BASELINE-CLIENT", requirement, mapping)
            .expect("the reciprocal-presence rule has category-matched evidence");
        assert!(mapping.evaluator.is_some());
    }
}

#[test]
fn profile_rejects_unpaired_direct_key_block_fields_in_register_and_import() {
    // KMIP Specification v2.1 §3.1 Table 12; request scope is §§6.1.25 and 6.1.43.
    const ALGORITHM_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-3.1-004-002";
    const ALGORITHM_CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-3.1-004";
    const LENGTH_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-3.1-005-002";
    const LENGTH_CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-3.1-005";

    for operation in [TABLE12_REGISTER_OPERATION, TABLE12_IMPORT_OPERATION] {
        let algorithm_without_length = request_message_with_operation_payload(
            operation,
            table12_request_payload(
                operation,
                table12_key_block_object(TABLE12_SYMMETRIC_KEY, true, false, None),
                [],
            ),
        );
        assert_table12_request_rejected(
            &algorithm_without_length,
            ALGORITHM_REQUIREMENT,
            ALGORITHM_CLAUSE,
        );

        let length_without_algorithm = request_message_with_operation_payload(
            operation,
            table12_request_payload(
                operation,
                table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, true, None),
                [],
            ),
        );
        assert_table12_request_rejected(
            &length_without_algorithm,
            LENGTH_REQUIREMENT,
            LENGTH_CLAUSE,
        );
    }

    let pgp_key_without_length = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_PGP_KEY, true, false, None),
            [],
        ),
    );
    assert_table12_request_rejected(
        &pgp_key_without_length,
        ALGORITHM_REQUIREMENT,
        ALGORITHM_CLAUSE,
    );
}

#[test]
fn profile_accepts_absent_or_present_direct_key_block_pairs() {
    // KMIP Specification v2.1 §3.1 Table 12 defines the pair; §§6.1.25 Table 238
    // and 6.1.43 Table 291 establish Import and Register request applicability.
    // Both fields may be absent together under these reciprocal-presence SHALLs.
    const REQUIREMENT_IDS: [&str; 2] = [
        "KMIPKIT-REQ-SPEC-3.1-004-002",
        "KMIPKIT-REQ-SPEC-3.1-005-002",
    ];
    let result = table12_profile_result(&REQUIREMENT_IDS);

    for operation in [TABLE12_REGISTER_OPERATION, TABLE12_IMPORT_OPERATION] {
        for (has_algorithm, has_length) in [(false, false), (true, true)] {
            let request = request_message_with_operation_payload(
                operation,
                table12_request_payload(
                    operation,
                    table12_key_block_object(
                        TABLE12_SYMMETRIC_KEY,
                        has_algorithm,
                        has_length,
                        None,
                    ),
                    [],
                ),
            );
            result
                .validate_request(&request)
                .expect("the reciprocal-presence requirements allow both absent or both present");
        }
    }

    let unrelated_operation =
        request_message_with_operation_payload(TABLE12_GET_OPERATION, Structure::new());
    result
        .validate_request(&unrelated_operation)
        .expect("Table 12 does not constrain unrelated operations");
}

#[test]
fn profile_does_not_merge_register_or_key_value_attributes_into_table12_pairing() {
    // KMIP Specification v2.1 §3.1 Table 12 checks direct Key Block fields;
    // §6.1.43 Table 291 places Register Attributes outside that Key Block.
    const ALGORITHM_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-3.1-004-002";
    const ALGORITHM_CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-3.1-004";
    const LENGTH_REQUIREMENT: &str = "KMIPKIT-REQ-SPEC-3.1-005-002";
    const LENGTH_CLAUSE: &str = "KMIPKIT-CLAUSE-SPEC-3.1-005";

    let algorithm_only_with_nested_and_outer_length = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(
                TABLE12_SYMMETRIC_KEY,
                true,
                false,
                Some(tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256))),
            ),
            [tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256))],
        ),
    );
    assert_table12_request_rejected(
        &algorithm_only_with_nested_and_outer_length,
        ALGORITHM_REQUIREMENT,
        ALGORITHM_CLAUSE,
    );

    let length_only_with_nested_and_outer_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(
                TABLE12_SYMMETRIC_KEY,
                false,
                true,
                Some(tagged(
                    TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                    Value::enumeration(3),
                )),
            ),
            [tagged(
                TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(3),
            )],
        ),
    );
    assert_table12_request_rejected(
        &length_only_with_nested_and_outer_algorithm,
        LENGTH_REQUIREMENT,
        LENGTH_CLAUSE,
    );
}

#[test]
fn profile_maps_register_object_type_and_attributes_as_required() {
    // KMIP Specification v2.1 §6.1.43 Table 291 and the Register text require
    // Object Type and the Attributes object for each applicable request.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-001";
    const ATTRIBUTE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-007";
    const ATTRIBUTE_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-007";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const REGISTER_OPERATION: u32 = 0x0000_0003;
    const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
    const ATTRIBUTES_TAG: u32 = 0x0042_0125;

    let mut evaluate = None;
    for (requirement_id, source_clause_id) in [
        (REQUIREMENT_ID, SOURCE_CLAUSE_ID),
        (ATTRIBUTE_REQUIREMENT_ID, ATTRIBUTE_SOURCE_CLAUSE_ID),
    ] {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id)
            .expect("Register request requirements have typed profile rules");
        let requirement = CLIENT_REQUIREMENTS
            .iter()
            .find(|requirement| requirement.requirement_id == requirement_id)
            .expect("Register request requirements resolve to generated metadata");
        assert_eq!(mapping.source_clause_id, source_clause_id);
        assert_eq!(
            mapping.verification_category,
            GeneratedVerificationCategory::RequiredBehavior
        );
        validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
            .expect("Register request requirements have positive executable evidence");
        let rule = mapping
            .evaluator
            .expect("Register request requirements have typed evaluators");
        assert!(!rule(None, None));
        evaluate = Some(rule);
    }
    let evaluate = evaluate.expect("the required Register fields have a typed evaluator");

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![
            REQUIREMENT_ID.to_owned(),
            ATTRIBUTE_REQUIREMENT_ID.to_owned(),
        ],
    };
    let valid = request_message_with_operation_payload(
        REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(0x8000_0001)),
            tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
        ]),
    );
    assert!(evaluate(Some(&valid), None));
    result
        .validate_request(&valid)
        .expect("a future Object Type Enumeration and empty Attributes structure are preserved");
    result
        .validate_response(&valid, &response_message())
        .expect("the outbound Register field rule does not reject a response");
    let unrelated_operation = request_message();
    assert!(evaluate(Some(&unrelated_operation), None));

    for malformed in [
        request_message_with_operation_payload(
            REGISTER_OPERATION,
            structure([tagged(ATTRIBUTES_TAG, Value::structure(Structure::new()))]),
        ),
        request_message_with_operation_payload(
            REGISTER_OPERATION,
            structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(2))]),
        ),
        request_message_with_operation_payload(
            REGISTER_OPERATION,
            structure([
                tagged(
                    OBJECT_TYPE_TAG,
                    Value::text_string("symmetric-key".to_owned()),
                ),
                tagged(ATTRIBUTES_TAG, Value::integer(7)),
            ]),
        ),
    ] {
        assert!(!evaluate(Some(&malformed), None));
        let error = result.validate_request(&malformed).expect_err(
            "Register requires one Enumeration Object Type and one Attributes Structure",
        );
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    }
}

#[test]
fn profile_maps_register_conditional_algorithm_length_and_signature_omissions() {
    // KMIP Specification v2.1 §6.1.43 Table 293 permits these omissions only
    // when the corresponding information is carried by the Key Block/Certificate.
    const ALGORITHM_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-001";
    const ALGORITHM_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-009";
    const LENGTH_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-010-001";
    const LENGTH_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-010";
    const SIGNATURE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-011";
    const SIGNATURE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-011";
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_conditional_algorithm_length_and_signature_omissions";
    for (requirement_id, source_clause_id) in [
        (ALGORITHM_REQUIREMENT_ID, ALGORITHM_CLAUSE_ID),
        (LENGTH_REQUIREMENT_ID, LENGTH_CLAUSE_ID),
        (SIGNATURE_REQUIREMENT_ID, SIGNATURE_CLAUSE_ID),
    ] {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id)
            .expect("the conditional Register metadata omission has a typed profile rule");
        let requirement = CLIENT_REQUIREMENTS
            .iter()
            .find(|requirement| requirement.requirement_id == requirement_id)
            .expect("the conditional Register requirement resolves to generated metadata");
        assert_eq!(mapping.source_clause_id, source_clause_id);
        assert_eq!(
            mapping.verification_category,
            GeneratedVerificationCategory::OptionalCapability
        );
        validate_mapping("KMIPKIT-PROFILE-BASELINE-CLIENT", requirement, mapping)
            .expect("the conditional omission has usable positive evidence");
        assert!(mapping.evaluator.is_some());
        assert!(mapping.test_evidence.iter().any(|evidence| {
            evidence.test_id == PROFILE_TEST_ID
                && evidence.kind == RuleTestKind::OptionalCapabilityUsable
        }));
    }

    assert_register_key_block_omission_paths(ALGORITHM_REQUIREMENT_ID, LENGTH_REQUIREMENT_ID);
    assert_register_certificate_signature_omission(SIGNATURE_REQUIREMENT_ID);
}

#[test]
fn profile_maps_register_omission_exemptions_and_malformed_containers() {
    // KMIP Specification v2.1 §6.1.43 Table 293 exempts Secret Data and only
    // permits omission when the required information is structurally present.
    const ALGORITHM_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-001";
    const LENGTH_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-010-001";
    const SIGNATURE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-011";
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_exemptions_and_malformed_containers";

    for requirement_id in [
        ALGORITHM_REQUIREMENT_ID,
        LENGTH_REQUIREMENT_ID,
        SIGNATURE_REQUIREMENT_ID,
    ] {
        let mapping = register_omission_mapping(requirement_id);
        assert!(mapping.test_evidence.iter().any(|evidence| {
            evidence.test_id == PROFILE_TEST_ID
                && evidence.kind == RuleTestKind::OptionalCapabilityUsable
        }));
    }

    assert_register_secret_data_omission_is_allowed();
    assert_register_duplicate_key_block_algorithm_is_rejected();
    assert_register_signature_omission_rejects_malformed_certificates();
}

#[test]
fn profile_maps_register_omission_pair_metadata_branches() {
    const REQUIREMENT_IDS: [&str; 3] = [
        "KMIPKIT-REQ-SPEC-6.1.43-009-001",
        "KMIPKIT-REQ-SPEC-6.1.43-010-001",
        "KMIPKIT-REQ-SPEC-6.1.43-011",
    ];
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_register_omission_pair_metadata_branches";

    for requirement_id in REQUIREMENT_IDS {
        let mapping = register_omission_mapping(requirement_id);
        assert!(mapping.test_evidence.iter().any(|evidence| {
            evidence.test_id == PROFILE_TEST_ID
                && evidence.kind == RuleTestKind::OptionalCapabilityUsable
        }));
        assert!(!(mapping.evaluator.expect("the rule has an evaluator"))(
            None, None
        ));
    }

    assert_register_table293_outer_pair_metadata_branches();
    assert_register_signature_algorithm_presence_branches();
}

fn assert_register_table293_outer_pair_metadata_branches() {
    const ALGORITHM_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-001";
    const LENGTH_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-010-001";

    let paired = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(2)),
            tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([
                    tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                    tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
                ])),
            ),
        ]),
    );
    for requirement_id in [ALGORITHM_REQUIREMENT_ID, LENGTH_REQUIREMENT_ID] {
        assert!(register_omission_rule_accepts(requirement_id, &paired));
    }

    let one_sided_pair = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(2)),
            tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                    Value::enumeration(3),
                )])),
            ),
        ]),
    );
    for requirement_id in [ALGORITHM_REQUIREMENT_ID, LENGTH_REQUIREMENT_ID] {
        assert!(!register_omission_rule_accepts(
            requirement_id,
            &one_sided_pair
        ));
    }

    let invalid_object_type = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([tagged(
            OBJECT_TYPE_TAG,
            Value::text_string("symmetric-key".to_owned()),
        )]),
    );
    assert!(!register_omission_rule_accepts(
        ALGORITHM_REQUIREMENT_ID,
        &invalid_object_type
    ));

    let missing_object_type = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([tagged(ATTRIBUTES_TAG, Value::structure(Structure::new()))]),
    );
    assert!(!register_omission_rule_accepts(
        ALGORITHM_REQUIREMENT_ID,
        &missing_object_type
    ));

    let missing_key_block_object = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(2)),
            tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
        ]),
    );
    assert!(!register_omission_rule_accepts(
        ALGORITHM_REQUIREMENT_ID,
        &missing_key_block_object
    ));
}

fn assert_register_signature_algorithm_presence_branches() {
    const SIGNATURE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-011";
    let signature_algorithm_present = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(1)),
            tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    DIGITAL_SIGNATURE_ALGORITHM_TAG,
                    Value::enumeration(4),
                )])),
            ),
        ]),
    );
    assert!(register_omission_rule_accepts(
        SIGNATURE_REQUIREMENT_ID,
        &signature_algorithm_present
    ));

    let non_certificate_object = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(2))]),
    );
    assert!(register_omission_rule_accepts(
        SIGNATURE_REQUIREMENT_ID,
        &non_certificate_object
    ));

    let invalid_signature_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(1)),
            tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    DIGITAL_SIGNATURE_ALGORITHM_TAG,
                    Value::text_string("RSA".to_owned()),
                )])),
            ),
        ]),
    );
    assert!(!register_omission_rule_accepts(
        SIGNATURE_REQUIREMENT_ID,
        &invalid_signature_algorithm
    ));
}

fn register_omission_rule_accepts(requirement_id: &str, request: &crate::RequestMessage) -> bool {
    register_omission_mapping(requirement_id)
        .evaluator
        .expect("the rule has an evaluator")(Some(request), None)
}

fn register_omission_mapping(requirement_id: &str) -> &'static TypedRuleMapping {
    TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == requirement_id)
        .expect("Register omission requirements have typed profile rules")
}

fn assert_register_secret_data_omission_is_allowed() {
    const ALGORITHM_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-001";
    const LENGTH_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-010-001";
    let secret_data = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SECRET_DATA, false, false, None),
            [],
        ),
    );
    for requirement_id in [ALGORITHM_REQUIREMENT_ID, LENGTH_REQUIREMENT_ID] {
        let mapping = register_omission_mapping(requirement_id);
        assert!(mapping.evaluator.expect("the rule has an evaluator")(
            Some(&secret_data),
            None
        ));
    }
}

fn assert_register_duplicate_key_block_algorithm_is_rejected() {
    const ALGORITHM_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-001";
    const KEY_BLOCK_TAG: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE_TAG: u32 = 0x0042_0027;
    const KEY_VALUE_TAG: u32 = 0x0042_0045;
    const KEY_MATERIAL_TAG: u32 = 0x0042_0043;
    let duplicate_algorithm_key_block = tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([tagged(
            KEY_BLOCK_TAG,
            Value::structure(structure([
                tagged(KEY_FORMAT_TYPE_TAG, Value::enumeration(1)),
                tagged(
                    KEY_VALUE_TAG,
                    Value::structure(structure([tagged(
                        KEY_MATERIAL_TAG,
                        Value::byte_string(b"synthetic-key-material".to_vec()),
                    )])),
                ),
                tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(4)),
                tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            ])),
        )])),
    );
    let duplicate_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            duplicate_algorithm_key_block,
            [],
        ),
    );
    let algorithm_mapping = register_omission_mapping(ALGORITHM_REQUIREMENT_ID);
    assert!(!algorithm_mapping
        .evaluator
        .expect("the rule has an evaluator")(
        Some(&duplicate_algorithm),
        None
    ));
}

fn assert_register_signature_omission_rejects_malformed_certificates() {
    const SIGNATURE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-011";
    const CERTIFICATE_TAG: u32 = 0x0042_0013;
    const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;
    const CERTIFICATE_VALUE_TAG: u32 = 0x0042_001E;
    let invalid_certificate_value = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(1)),
            tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
            tagged(
                CERTIFICATE_TAG,
                Value::structure(structure([
                    tagged(CERTIFICATE_TYPE_TAG, Value::enumeration(1)),
                    tagged(CERTIFICATE_VALUE_TAG, Value::integer(1)),
                ])),
            ),
        ]),
    );
    let duplicate_certificate = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(1)),
            tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
            tagged(
                CERTIFICATE_TAG,
                Value::structure(structure([tagged(
                    CERTIFICATE_VALUE_TAG,
                    Value::byte_string(b"certificate".to_vec()),
                )])),
            ),
            tagged(
                CERTIFICATE_TAG,
                Value::structure(structure([tagged(
                    CERTIFICATE_VALUE_TAG,
                    Value::byte_string(b"duplicate".to_vec()),
                )])),
            ),
        ]),
    );
    let signature_mapping = register_omission_mapping(SIGNATURE_REQUIREMENT_ID);
    let signature_rule = signature_mapping
        .evaluator
        .expect("the rule has an evaluator");
    assert!(!signature_rule(Some(&invalid_certificate_value), None));
    assert!(!signature_rule(Some(&duplicate_certificate), None));
}

fn assert_register_key_block_omission_paths(
    algorithm_requirement_id: &str,
    length_requirement_id: &str,
) {
    let key_block_request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, true, true, None),
            [tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3))],
        ),
    );
    for requirement_id in [algorithm_requirement_id, length_requirement_id] {
        table12_profile_result(&[requirement_id])
            .validate_request(&key_block_request)
            .expect("Algorithm and Length may be omitted from Attributes when in the Key Block");
    }

    let nested_key_value = tagged(
        TABLE12_KEY_VALUE,
        Value::structure(structure([
            tagged(
                TABLE12_KEY_MATERIAL,
                Value::byte_string(b"nested-key-material".to_vec()),
            ),
            tagged(
                TABLE12_ATTRIBUTES,
                Value::structure(structure([
                    tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                    tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
                ])),
            ),
        ])),
    );
    let nested_key_block = tagged(
        TABLE12_KEY_BLOCK,
        Value::structure(structure([
            tagged(TABLE12_KEY_FORMAT_TYPE, Value::enumeration(1)),
            nested_key_value,
        ])),
    );
    let nested_object = tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([nested_key_block])),
    );
    let nested_key_block_request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            nested_object,
            [tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3))],
        ),
    );
    for requirement_id in [algorithm_requirement_id, length_requirement_id] {
        table12_profile_result(&[requirement_id])
            .validate_request(&nested_key_block_request)
            .expect("Key Value Attributes also carry the omitted Algorithm and Length");
    }

    let incomplete_key_block_request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, true, false, None),
            [tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3))],
        ),
    );
    let error = table12_profile_result(&[length_requirement_id])
        .validate_request(&incomplete_key_block_request)
        .expect_err("omitted Key Block Length must be represented");
    assert_eq!(error.requirement_id(), Some(length_requirement_id));
}

fn assert_register_certificate_signature_omission(signature_requirement_id: &str) {
    const CERTIFICATE_TAG: u32 = 0x0042_0013;
    const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;
    const CERTIFICATE_VALUE_TAG: u32 = 0x0042_001E;
    const CERTIFICATE_LENGTH_TAG: u32 = 0x0042_00AD;

    let certificate_request = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(TABLE12_OBJECT_TYPE, Value::enumeration(1)),
            tagged(
                TABLE12_ATTRIBUTES,
                Value::structure(structure([
                    tagged(CERTIFICATE_TYPE_TAG, Value::enumeration(1)),
                    tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(4)),
                    tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(2048)),
                    tagged(CERTIFICATE_LENGTH_TAG, Value::integer(4)),
                ])),
            ),
            tagged(
                CERTIFICATE_TAG,
                Value::structure(structure([
                    tagged(CERTIFICATE_TYPE_TAG, Value::enumeration(1)),
                    tagged(
                        CERTIFICATE_VALUE_TAG,
                        Value::byte_string(b"caller-supplied-certificate".to_vec()),
                    ),
                ])),
            ),
        ]),
    );
    table12_profile_result(&[signature_requirement_id])
        .validate_request(&certificate_request)
        .expect("Digital Signature Algorithm may be omitted when carried by Certificate");

    let certificate_without_value = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(TABLE12_OBJECT_TYPE, Value::enumeration(1)),
            tagged(TABLE12_ATTRIBUTES, Value::structure(Structure::new())),
        ]),
    );
    let error = table12_profile_result(&[signature_requirement_id])
        .validate_request(&certificate_without_value)
        .expect_err("Digital Signature Algorithm cannot be omitted without a Certificate");
    assert_eq!(error.requirement_id(), Some(signature_requirement_id));
}

#[test]
fn profile_maps_client_register_algorithm_requires_length_in_attributes() {
    // KMIP Specification v2.1 §6.1.43 Table 293 pairs the outer Register
    // Attributes. §§3.1–3.2 separately describe Key Block and Key Value data.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-009";
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_algorithm_requires_length_in_attributes";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Register Algorithm/Length has a typed profile rule");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    assert!(mapping.evaluator.is_some());
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id == PROFILE_TEST_ID && evidence.kind == RuleTestKind::PositiveBehavior
    }));

    let result = table12_profile_result(&[REQUIREMENT_ID]);
    assert!(!super::client_register_algorithm_requires_length(
        None, None
    ));
    let algorithm_and_length = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
            [
                tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            ],
        ),
    );
    assert!(algorithm_and_length.batch_items().all(|batch_item| {
        batch_item
            .with_request_payload(|payload| {
                let Some(attributes) = payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == TABLE12_ATTRIBUTES)
                else {
                    return false;
                };
                attributes.with_value(|value| match value {
                    ValueView::Structure(attributes) => {
                        attributes
                            .children()
                            .iter()
                            .any(|field| field.tag().raw() == TABLE12_CRYPTOGRAPHIC_ALGORITHM)
                            && attributes
                                .children()
                                .iter()
                                .any(|field| field.tag().raw() == TABLE12_CRYPTOGRAPHIC_LENGTH)
                    }
                    _ => false,
                })
            })
            .unwrap_or(false)
    }));
    assert!(super::client_register_algorithm_requires_length(
        Some(&algorithm_and_length),
        None,
    ));
    result
        .validate_request(&algorithm_and_length)
        .expect("Register Attributes may provide Algorithm and Length together");

    let no_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
            [],
        ),
    );
    result
        .validate_request(&no_algorithm)
        .expect("the conditional Table 293 rule allows Algorithm to be absent");

    let no_attributes = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(
                TABLE12_OBJECT_TYPE,
                Value::enumeration(TABLE12_SYMMETRIC_KEY_OBJECT_TYPE),
            ),
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
        ]),
    );
    result
        .validate_request(&no_attributes)
        .expect("omitting Register Attributes also omits the conditional Algorithm");

    let unrelated_operation =
        request_message_with_operation_payload(TABLE12_GET_OPERATION, Structure::new());
    result
        .validate_request(&unrelated_operation)
        .expect("the Register Attribute rule does not constrain Get");
}

#[test]
fn profile_rejects_client_register_algorithm_without_length_in_attributes() {
    // KMIP Specification v2.1 §6.1.43 Table 293, Register Attributes column.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-009";

    let algorithm_without_length = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
            [tagged(
                TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(3),
            )],
        ),
    );
    let error = table12_profile_result(&[REQUIREMENT_ID])
        .validate_request(&algorithm_without_length)
        .expect_err("a Register Algorithm requires Cryptographic Length");
    assert!(!super::client_register_algorithm_requires_length(
        Some(&algorithm_without_length),
        None,
    ));
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
}

#[test]
fn profile_does_not_use_key_block_length_as_register_attribute_pairing() {
    // KMIP Specification v2.1 §6.1.43 Table 293 makes the Register Attributes
    // pair direct; §3.1 Table 12 independently governs fields inside Key Block.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-009";

    let result = table12_profile_result(&[REQUIREMENT_ID]);
    let key_block_length = [
        table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, true, None),
        table12_key_block_object(
            TABLE12_SYMMETRIC_KEY,
            false,
            false,
            Some(tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256))),
        ),
    ];

    for object in key_block_length {
        let request = request_message_with_operation_payload(
            TABLE12_REGISTER_OPERATION,
            table12_request_payload(
                TABLE12_REGISTER_OPERATION,
                object,
                [tagged(
                    TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                    Value::enumeration(3),
                )],
            ),
        );
        let error = result
            .validate_request(&request)
            .expect_err("Key Block Length does not pair an outer Register Algorithm");
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
    }
}

#[test]
fn profile_maps_client_register_length_requires_algorithm_in_attributes() {
    // KMIP Specification v2.1 §6.1.43 Table 293's Length row requires the
    // Algorithm row above in the outer Register Attributes.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-010-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-010";
    const PROFILE_TEST_ID: &str = "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_maps_client_register_length_requires_algorithm_in_attributes";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Register Length/Algorithm has a typed profile rule");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    assert!(mapping.evaluator.is_some());
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.test_id == PROFILE_TEST_ID && evidence.kind == RuleTestKind::PositiveBehavior
    }));

    let result = table12_profile_result(&[REQUIREMENT_ID]);
    let length_without_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
            [tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256))],
        ),
    );
    let error = result
        .validate_request(&length_without_algorithm)
        .expect_err("an outer Register Length requires an outer Algorithm");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));

    let length_with_key_block_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, true, true, None),
            [tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256))],
        ),
    );
    let error = result
        .validate_request(&length_with_key_block_algorithm)
        .expect_err("Key Block Algorithm does not replace the outer Register Algorithm");
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));

    let direct_pair = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
            [
                tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            ],
        ),
    );
    result
        .validate_request(&direct_pair)
        .expect("the outer Algorithm and Length attributes satisfy the pair");

    let opaque_key_format = tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([tagged(
            TABLE12_KEY_BLOCK,
            Value::structure(structure([
                tagged(TABLE12_KEY_FORMAT_TYPE, Value::enumeration(2)),
                tagged(
                    TABLE12_KEY_VALUE,
                    Value::byte_string(b"opaque key format bytes".to_vec()),
                ),
            ])),
        )])),
    );
    let length_on_symmetric_key_with_opaque_key_format = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            opaque_key_format,
            [tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256))],
        ),
    );
    let error = result
        .validate_request(&length_on_symmetric_key_with_opaque_key_format)
        .expect_err("Key Format Type Opaque does not make a Symmetric Key an Opaque Object");
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
}

#[test]
fn profile_does_not_apply_register_managed_crypto_pairing_to_context_unknown_opaque() {
    // KMIP Specification v2.1 §2.3 makes Opaque's managed-crypto status depend
    // on local client context, which is not encoded in the §6.1.43 Table 291
    // request. A wire-only evaluator cannot infer that context.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";

    let opaque_with_algorithm_only = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_opaque_object(),
            [tagged(
                TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(3),
            )],
        ),
    );

    table12_profile_result(&[REQUIREMENT_ID])
        .validate_request(&opaque_with_algorithm_only)
        .expect("wire-only validation leaves context-dependent Opaque applicability neutral");
}

#[test]
fn profile_does_not_count_key_block_length_from_a_non_object_register_structure() {
    // KMIP Specification v2.1 §6.1.43 Table 291 places the object in its
    // object-specific structure; an unrelated payload structure is not evidence.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";

    let decoy = tagged(
        0x0042_0001,
        Value::structure(structure([tagged(
            TABLE12_KEY_BLOCK,
            Value::structure(structure([tagged(
                TABLE12_CRYPTOGRAPHIC_LENGTH,
                Value::integer(256),
            )])),
        )])),
    );
    let payload = structure([
        tagged(
            TABLE12_OBJECT_TYPE,
            Value::enumeration(TABLE12_SYMMETRIC_KEY_OBJECT_TYPE),
        ),
        tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure([tagged(
                TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(3),
            )])),
        ),
        decoy,
        table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
    ]);
    let request = request_message_with_operation_payload(TABLE12_REGISTER_OPERATION, payload);

    table12_profile_result(&[REQUIREMENT_ID])
        .validate_request(&request)
        .expect_err("a non-object structure cannot supply Register Cryptographic Length");
}

#[test]
fn profile_rejects_malformed_register_object_key_block_length_sources() {
    // KMIP Specification v2.1 §6.1.43 Table 293; §§3.1–3.2 Tables 12–13.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-009";
    let result = table12_profile_result(&[REQUIREMENT_ID]);
    let request_for_object = |object: Option<Item>| {
        let mut fields = vec![
            tagged(
                TABLE12_OBJECT_TYPE,
                Value::enumeration(TABLE12_SYMMETRIC_KEY_OBJECT_TYPE),
            ),
            tagged(
                TABLE12_ATTRIBUTES,
                Value::structure(structure([tagged(
                    TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                    Value::enumeration(3),
                )])),
            ),
        ];
        fields.extend(object);
        request_message_with_operation_payload(TABLE12_REGISTER_OPERATION, structure(fields))
    };
    let assert_rejected = |request: &crate::RequestMessage| {
        assert!(!super::client_register_algorithm_requires_length(
            Some(request),
            None,
        ));
        let error = result
            .validate_request(request)
            .expect_err("malformed Key Block evidence cannot satisfy Table 293");
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
    };

    assert_rejected(&request_for_object(None));
    assert_rejected(&request_for_object(Some(tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::integer(1),
    ))));
    assert_rejected(&request_for_object(Some(tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([tagged(TABLE12_KEY_BLOCK, Value::integer(1))])),
    ))));
    assert_rejected(&request_for_object(Some(tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([tagged(
            TABLE12_KEY_BLOCK,
            Value::structure(structure([
                tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(128)),
                tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            ])),
        )])),
    ))));
    assert_rejected(&request_for_object(Some(tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([tagged(
            TABLE12_KEY_BLOCK,
            Value::structure(structure([
                tagged(TABLE12_KEY_FORMAT_TYPE, Value::enumeration(1)),
                tagged(
                    TABLE12_KEY_VALUE,
                    Value::byte_string(b"opaque wrapped value".to_vec()),
                ),
            ])),
        )])),
    ))));
    assert_rejected(&request_for_object(Some(tagged(
        TABLE12_SYMMETRIC_KEY,
        Value::structure(structure([tagged(
            TABLE12_KEY_BLOCK,
            Value::structure(structure([tagged(
                TABLE12_KEY_VALUE,
                Value::structure(structure([tagged(
                    TABLE12_ATTRIBUTES,
                    Value::structure(Structure::new()),
                )])),
            )])),
        )])),
    ))));
}

#[test]
fn profile_applies_register_algorithm_length_rule_to_standard_object_types() {
    // KMIP Specification v2.1 §§2.1–2.9 and §6.1.43 Table 293.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";
    let cases = [
        (1, TABLE12_CERTIFICATE, true),
        (2, TABLE12_SYMMETRIC_KEY, true),
        (3, TABLE12_PUBLIC_KEY, true),
        (4, TABLE12_PRIVATE_KEY, true),
        (5, TABLE12_SPLIT_KEY, true),
        (6, 0x0042_0001, false),
        (7, TABLE12_SECRET_DATA, false),
        (8, TABLE12_OPAQUE_OBJECT, false),
        (9, TABLE12_PGP_KEY, true),
        (10, TABLE12_CERTIFICATE_REQUEST, true),
        (0x8000_0001, 0x0042_0001, false),
    ];

    for (object_type, object_tag, requires_length) in cases {
        let request = request_message_with_operation_payload(
            TABLE12_REGISTER_OPERATION,
            structure([
                tagged(TABLE12_OBJECT_TYPE, Value::enumeration(object_type)),
                tagged(
                    TABLE12_ATTRIBUTES,
                    Value::structure(structure([tagged(
                        TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                        Value::enumeration(3),
                    )])),
                ),
                tagged(object_tag, Value::structure(Structure::new())),
            ]),
        );
        let validation = table12_profile_result(&[REQUIREMENT_ID]).validate_request(&request);

        if requires_length {
            validation.expect_err("a standard Managed Cryptographic Object requires Length");
        } else {
            validation.expect("Secret Data, Opaque, reserved and vendor values stay neutral");
        }
    }
}

#[test]
fn profile_rejects_malformed_register_attributes_for_algorithm_length_requirement() {
    // KMIP Specification v2.1 §6.1.43 Table 293 requires readable Register
    // Attributes to evaluate the conditional Cryptographic Algorithm pair.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.43-009";
    let result = table12_profile_result(&[REQUIREMENT_ID]);
    let object = || table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None);
    let assert_rejected = |payload: Structure| {
        let request = request_message_with_operation_payload(TABLE12_REGISTER_OPERATION, payload);
        let error = result
            .validate_request(&request)
            .expect_err("ambiguous Register Attributes fail closed");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
    };

    let algorithm = || tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3));
    let length = || tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256));

    assert_rejected(structure([
        tagged(
            TABLE12_OBJECT_TYPE,
            Value::enumeration(TABLE12_SYMMETRIC_KEY_OBJECT_TYPE),
        ),
        tagged(TABLE12_ATTRIBUTES, Value::integer(1)),
        object(),
    ]));

    assert_rejected(structure([
        tagged(
            TABLE12_OBJECT_TYPE,
            Value::enumeration(TABLE12_SYMMETRIC_KEY_OBJECT_TYPE),
        ),
        tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure([algorithm(), length()])),
        ),
        tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure([algorithm(), length()])),
        ),
        object(),
    ]));

    assert_rejected(table12_request_payload(
        TABLE12_REGISTER_OPERATION,
        object(),
        [algorithm(), algorithm(), length()],
    ));
    assert_rejected(table12_request_payload(
        TABLE12_REGISTER_OPERATION,
        object(),
        [algorithm(), length(), length()],
    ));
}

#[test]
fn profile_exempts_secret_data_from_register_algorithm_length_requirement() {
    // KMIP Specification v2.1 §6.1.43 Table 293 explicitly excludes Secret Data.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-009-002";

    let secret_data_with_algorithm_only = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SECRET_DATA, false, false, None),
            [tagged(
                TABLE12_CRYPTOGRAPHIC_ALGORITHM,
                Value::enumeration(3),
            )],
        ),
    );
    table12_profile_result(&[REQUIREMENT_ID])
        .validate_request(&secret_data_with_algorithm_only)
        .expect("Table 293 Algorithm/Length requirements do not apply to Secret Data");
}

#[test]
fn profile_exempts_secret_data_and_ignores_valid_opaque_objects_without_key_blocks() {
    // KMIP Specification v2.1 §3.1 Table 12 exempts Secret Data and Opaque;
    // §2.3 Table 4 defines Opaque Data Type/Value, §11.35 Table 469 gives the
    // extension range, and §6.1.43 Table 291 carries Register's Any Object.
    // Opaque Objects have no Key Block under Table 4.
    const REQUIREMENT_IDS: [&str; 2] = [
        "KMIPKIT-REQ-SPEC-3.1-004-002",
        "KMIPKIT-REQ-SPEC-3.1-005-002",
    ];
    let result = table12_profile_result(&REQUIREMENT_IDS);

    let secret_data_with_unpaired_key_block = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SECRET_DATA, true, false, None),
            [],
        ),
    );
    result
        .validate_request(&secret_data_with_unpaired_key_block)
        .expect("Table 12 reciprocal presence does not apply to Secret Data");

    let valid_opaque_object = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(TABLE12_REGISTER_OPERATION, table12_opaque_object(), []),
    );
    result
        .validate_request(&valid_opaque_object)
        .expect("a valid Opaque Object has no Key Block for Table 12 pairing to inspect");
}

#[test]
fn profile_checks_each_mixed_register_import_batch_item() {
    // KMIP Specification v2.1 §3.1 Table 12 defines the pair; §§6.1.25 Table 238
    // and 6.1.43 Table 291 carry one Any Object per Import or Register Batch Item.
    const REQUIREMENT_IDS: [&str; 2] = [
        "KMIPKIT-REQ-SPEC-3.1-004-002",
        "KMIPKIT-REQ-SPEC-3.1-005-002",
    ];
    let result = table12_profile_result(&REQUIREMENT_IDS);
    let valid_import = table12_request_payload(
        TABLE12_IMPORT_OPERATION,
        table12_key_block_object(TABLE12_SYMMETRIC_KEY, true, true, None),
        [],
    );
    let permitted = request_message_with_batch_operations([
        (TABLE12_GET_OPERATION, Structure::new()),
        (
            TABLE12_REGISTER_OPERATION,
            table12_request_payload(
                TABLE12_REGISTER_OPERATION,
                table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
                [],
            ),
        ),
        (TABLE12_IMPORT_OPERATION, valid_import),
    ]);
    result
        .validate_request(&permitted)
        .expect("unrelated, Register, and Import items with valid pairs remain accepted");

    let invalid_import = table12_request_payload(
        TABLE12_IMPORT_OPERATION,
        table12_key_block_object(TABLE12_SYMMETRIC_KEY, true, false, None),
        [],
    );
    let invalid_batch = request_message_with_batch_operations([
        (TABLE12_GET_OPERATION, Structure::new()),
        (
            TABLE12_REGISTER_OPERATION,
            table12_request_payload(
                TABLE12_REGISTER_OPERATION,
                table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
                [],
            ),
        ),
        (TABLE12_IMPORT_OPERATION, invalid_import),
    ]);
    assert_table12_request_rejected(
        &invalid_batch,
        "KMIPKIT-REQ-SPEC-3.1-004-002",
        "KMIPKIT-CLAUSE-SPEC-3.1-004",
    );
}

#[test]
fn profile_maps_client_register_key_value_present_as_prohibited_behavior() {
    // KMIP Specification v2.1 §4.28 prohibits this client-supplied Register attribute.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.28-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Key Value Present in Register has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the Register prohibition resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::ProhibitedBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the SHALL NOT has negative executable evidence");
    assert!(mapping.evaluator.is_some());
}

#[test]
fn profile_rejects_key_value_present_in_register_but_preserves_locate_criterion() {
    // KMIP Specification v2.1 §4.28 prohibits Register use and permits Locate use.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const REGISTER_OPERATION: u32 = 0x0000_0003;
    const LOCATE_OPERATION: u32 = 0x0000_0008;
    const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
    const ATTRIBUTES_TAG: u32 = 0x0042_0125;
    const SYMMETRIC_KEY_TAG: u32 = 0x0042_008F;
    const KEY_BLOCK_TAG: u32 = 0x0042_0040;
    const KEY_FORMAT_TYPE_TAG: u32 = 0x0042_0042;
    const KEY_VALUE_TAG: u32 = 0x0042_0045;
    const KEY_MATERIAL_TAG: u32 = 0x0042_0044;
    const KEY_VALUE_PRESENT_TAG: u32 = 0x0042_00BB;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let register_request = |key_value_present: Option<bool>| {
        let attributes = key_value_present
            .map(|value| vec![tagged(KEY_VALUE_PRESENT_TAG, Value::boolean(value))])
            .unwrap_or_default();
        request_message_with_operation_payload(
            REGISTER_OPERATION,
            structure([
                tagged(OBJECT_TYPE_TAG, Value::enumeration(2)),
                tagged(ATTRIBUTES_TAG, Value::structure(structure(attributes))),
                tagged(
                    SYMMETRIC_KEY_TAG,
                    Value::structure(structure([tagged(
                        KEY_BLOCK_TAG,
                        Value::structure(structure([
                            tagged(KEY_FORMAT_TYPE_TAG, Value::enumeration(1)),
                            tagged(
                                KEY_VALUE_TAG,
                                Value::structure(structure([tagged(
                                    KEY_MATERIAL_TAG,
                                    Value::byte_string(b"synthetic-key-material".to_vec()),
                                )])),
                            ),
                        ])),
                    )])),
                ),
            ]),
        )
    };

    for value in [true, false] {
        let error = result
            .validate_request(&register_request(Some(value)))
            .expect_err("a client must not specify Key Value Present in Register");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-4.28-001")
        );
    }

    result
        .validate_request(&register_request(None))
        .expect("Register without Key Value Present is permitted");

    let locate_request = request_message_with_operation_payload(
        LOCATE_OPERATION,
        structure([tagged(KEY_VALUE_PRESENT_TAG, Value::boolean(true))]),
    );
    result
        .validate_request(&locate_request)
        .expect("the Register prohibition does not reject a Locate criterion");
}

fn key_value_present_register_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec!["KMIPKIT-PROFILE-BASELINE-CLIENT".to_owned()],
        requirement_ids: vec!["KMIPKIT-REQ-SPEC-4.28-001-001".to_owned()],
    }
}

fn assert_key_value_present_register_request_rejected(request: &crate::RequestMessage) {
    let error = key_value_present_register_profile_result()
        .validate_request(request)
        .expect_err("a Register request with malformed or prohibited Attributes is rejected");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some("KMIPKIT-REQ-SPEC-4.28-001-001")
    );
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-4.28-001")
    );
}

#[test]
fn profile_rejects_malformed_attributes_type_for_register_key_value_present() {
    // KMIP Specification v2.1 §4.28; malformed request representation fails closed.
    let request = request_message_with_operation_payload(
        REGISTER_OPERATION_TAG,
        structure([tagged(ATTRIBUTES_TAG, Value::integer(1))]),
    );

    assert_key_value_present_register_request_rejected(&request);
}

#[test]
fn profile_rejects_duplicate_attributes_for_register_key_value_present() {
    // KMIP Specification v2.1 §4.28; duplicate Attributes cannot establish compliance.
    let request = request_message_with_operation_payload(
        REGISTER_OPERATION_TAG,
        structure([
            tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    KEY_VALUE_PRESENT_TAG,
                    Value::boolean(true),
                )])),
            ),
            tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
        ]),
    );

    assert_key_value_present_register_request_rejected(&request);
}

#[test]
fn profile_checks_register_key_value_present_for_each_batch_item() {
    // KMIP Specification v2.1 §4.28; the prohibition applies only to Register items.
    let locate_with_key_value_present = || {
        structure([tagged(
            ATTRIBUTES_TAG,
            Value::structure(structure([tagged(
                KEY_VALUE_PRESENT_TAG,
                Value::boolean(true),
            )])),
        )])
    };
    let register_without_attributes = Structure::new();
    let permitted_batch = request_message_with_batch_operations([
        (LOCATE_OPERATION_TAG, locate_with_key_value_present()),
        (REGISTER_OPERATION_TAG, register_without_attributes),
    ]);
    key_value_present_register_profile_result()
        .validate_request(&permitted_batch)
        .expect("non-Register items remain unaffected in a mixed batch");

    let register_with_key_value_present = structure([tagged(
        ATTRIBUTES_TAG,
        Value::structure(structure([tagged(
            KEY_VALUE_PRESENT_TAG,
            Value::boolean(false),
        )])),
    )]);
    let prohibited_batch = request_message_with_batch_operations([
        (LOCATE_OPERATION_TAG, locate_with_key_value_present()),
        (REGISTER_OPERATION_TAG, register_with_key_value_present),
    ]);
    assert_key_value_present_register_request_rejected(&prohibited_batch);
}

#[test]
fn profile_maps_key_value_present_register_malformed_duplicate_and_batch_rejections() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-001";
    const EXPECTED_TEST_IDS: [&str; 3] = [
        "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_malformed_attributes_type_for_register_key_value_present",
        "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_rejects_duplicate_attributes_for_register_key_value_present",
        "crates/kmipkit-protocol/tests/support/profile_unit.rs::profile_checks_register_key_value_present_for_each_batch_item",
    ];

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Key Value Present in Register has a typed profile rule");
    for expected_test_id in EXPECTED_TEST_IDS {
        assert!(
            mapping.test_evidence.iter().any(|evidence| {
                evidence.test_id == expected_test_id
                    && evidence.kind == RuleTestKind::NegativeBehavior
            }),
            "profile mapping must link negative test evidence {expected_test_id}"
        );
    }
}

#[test]
fn client_register_key_value_present_evaluator_fails_closed_without_request_context() {
    assert!(!client_register_does_not_specify_key_value_present(
        None, None
    ));
}

#[test]
fn profile_maps_key_value_present_modification_as_prohibited_behavior() {
    // KMIP Specification v2.1 §4.28 and Table 85 prohibit client mutation.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-002";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.28-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Key Value Present mutation has a typed profile rule");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("the mutation prohibition resolves to generated normative metadata");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::ProhibitedBehavior
    );
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the client SHALL NOT has negative executable evidence");
    assert!(mapping.evaluator.is_some());
}

#[test]
fn profile_rejects_key_value_present_attribute_mutations() {
    // KMIP Specification v2.1 §4.28/Table 85 prohibit client mutation; operation payloads are §§6.1.2, 6.1.3, 6.1.13, 6.1.34, 6.1.51 and their attribute wrappers/references are §§5.5–5.7.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_000D;
    const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
    const DELETE_ATTRIBUTE_OPERATION: u32 = 0x0000_000F;
    const MODIFY_ATTRIBUTE_OPERATION: u32 = 0x0000_000E;
    const SET_ATTRIBUTE_OPERATION: u32 = 0x0000_0031;
    const CURRENT_ATTRIBUTE_TAG: u32 = 0x0042_013C;
    const NEW_ATTRIBUTE_TAG: u32 = 0x0042_013D;
    const ATTRIBUTE_REFERENCE_TAG: u32 = 0x0042_013B;
    const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
    const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let key_value_present = || tagged(KEY_VALUE_PRESENT_TAG, Value::boolean(true));
    let current_key_value_present = || {
        tagged(
            CURRENT_ATTRIBUTE_TAG,
            Value::structure(structure([key_value_present()])),
        )
    };
    let new_key_value_present = || {
        tagged(
            NEW_ATTRIBUTE_TAG,
            Value::structure(structure([key_value_present()])),
        )
    };
    let raw_tag_reference = || {
        tagged(
            ATTRIBUTE_REFERENCE_TAG,
            Value::enumeration(KEY_VALUE_PRESENT_TAG),
        )
    };
    let name_reference = || {
        tagged(
            ATTRIBUTE_REFERENCE_TAG,
            Value::structure(structure([
                tagged(VENDOR_IDENTIFICATION_TAG, Value::text_string(String::new())),
                tagged(
                    ATTRIBUTE_NAME_TAG,
                    Value::text_string("Key Value Present".to_owned()),
                ),
            ])),
        )
    };
    let requests = [
        (
            "Add Attribute",
            ADD_ATTRIBUTE_OPERATION,
            structure([new_key_value_present()]),
        ),
        (
            "Adjust Attribute by tag reference",
            ADJUST_ATTRIBUTE_OPERATION,
            structure([raw_tag_reference()]),
        ),
        (
            "Delete Attribute by current value",
            DELETE_ATTRIBUTE_OPERATION,
            structure([current_key_value_present()]),
        ),
        (
            "Delete Attribute by name reference",
            DELETE_ATTRIBUTE_OPERATION,
            structure([name_reference()]),
        ),
        (
            "Modify Attribute current value",
            MODIFY_ATTRIBUTE_OPERATION,
            structure([current_key_value_present()]),
        ),
        (
            "Modify Attribute replacement value",
            MODIFY_ATTRIBUTE_OPERATION,
            structure([new_key_value_present()]),
        ),
        (
            "Set Attribute",
            SET_ATTRIBUTE_OPERATION,
            structure([new_key_value_present()]),
        ),
    ];

    for (case, operation, payload) in requests {
        let request = request_message_with_operation_payload(operation, payload);
        let error = result.validate_request(&request).expect_err(case);
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID), "{case}");
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-4.28-001")
        );
    }
}

#[test]
fn profile_rejects_key_value_present_mutation_in_mixed_batch() {
    // KMIP Specification v2.1 §4.28/Table 85; batch operations are §8.1 and the payloads are §§6.1.28, 6.1.51.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ATTRIBUTES_TAG: u32 = 0x0042_0125;
    const SET_ATTRIBUTE_OPERATION: u32 = 0x0000_0031;
    const NEW_ATTRIBUTE_TAG: u32 = 0x0042_013D;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let key_value_present = || tagged(KEY_VALUE_PRESENT_TAG, Value::boolean(true));
    let locate = structure([tagged(
        ATTRIBUTES_TAG,
        Value::structure(structure([key_value_present()])),
    )]);
    let set_attribute = structure([tagged(
        NEW_ATTRIBUTE_TAG,
        Value::structure(structure([key_value_present()])),
    )]);
    let mixed_batch = request_message_with_batch_operations([
        (0x0000_0008, locate),
        (SET_ATTRIBUTE_OPERATION, set_attribute),
    ]);

    let error = result
        .validate_request(&mixed_batch)
        .expect_err("a prohibited mutation in any batch item must reject the request");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
}

#[test]
fn profile_rejects_malformed_key_value_present_attribute_reference() {
    // KMIP Specification v2.1 §4.28/Table 85, §5.5, and §6.1.3 define the policy and reference shape.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
    const ATTRIBUTE_REFERENCE_TAG: u32 = 0x0042_013B;
    const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let malformed_reference = structure([tagged(
        ATTRIBUTE_REFERENCE_TAG,
        Value::structure(structure([tagged(
            ATTRIBUTE_NAME_TAG,
            Value::text_string("Key Value Present".to_owned()),
        )])),
    )]);
    let request =
        request_message_with_operation_payload(ADJUST_ATTRIBUTE_OPERATION, malformed_reference);

    let error = result
        .validate_request(&request)
        .expect_err("a malformed reference must fail closed");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
}

#[test]
fn client_key_value_present_mutation_rule_fails_closed_without_request_context() {
    assert!(!client_key_value_present_is_not_modified(None, None));
}

#[test]
fn profile_preserves_other_attribute_mutations_and_locate_criteria() {
    // The §4.28 mutation prohibition is specific to Key Value Present.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.28-001-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const ADD_ATTRIBUTE_OPERATION: u32 = 0x0000_000D;
    const ADJUST_ATTRIBUTE_OPERATION: u32 = 0x0000_0030;
    const LOCATE_OPERATION: u32 = 0x0000_0008;
    const NEW_ATTRIBUTE_TAG: u32 = 0x0042_013D;
    const ATTRIBUTE_REFERENCE_TAG: u32 = 0x0042_013B;
    const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
    const ATTRIBUTE_NAME_TAG: u32 = 0x0042_000A;
    const CRYPTOGRAPHIC_LENGTH_TAG: u32 = 0x0042_002A;
    const ATTRIBUTES_TAG: u32 = 0x0042_0125;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let add_other_attribute = structure([tagged(
        NEW_ATTRIBUTE_TAG,
        Value::structure(structure([tagged(
            CRYPTOGRAPHIC_LENGTH_TAG,
            Value::integer(256),
        )])),
    )]);
    let locate_key_value_present = structure([tagged(
        ATTRIBUTES_TAG,
        Value::structure(structure([tagged(
            KEY_VALUE_PRESENT_TAG,
            Value::boolean(true),
        )])),
    )]);
    let mixed_batch = request_message_with_batch_operations([
        (ADD_ATTRIBUTE_OPERATION, add_other_attribute),
        (LOCATE_OPERATION, locate_key_value_present),
    ]);

    result
        .validate_request(&mixed_batch)
        .expect("other attribute mutations and the permitted Locate criterion remain valid");

    let vendor_attribute_reference = structure([tagged(
        ATTRIBUTE_REFERENCE_TAG,
        Value::structure(structure([
            tagged(
                VENDOR_IDENTIFICATION_TAG,
                Value::text_string("vendor.example".to_owned()),
            ),
            tagged(
                ATTRIBUTE_NAME_TAG,
                Value::text_string("Key Value Present".to_owned()),
            ),
        ])),
    )]);
    result
        .validate_request(&request_message_with_operation_payload(
            ADJUST_ATTRIBUTE_OPERATION,
            vendor_attribute_reference,
        ))
        .expect("a vendor attribute with the same name is not the standard attribute");
}

#[test]
fn profile_maps_client_register_wrapped_key_value_with_embedded_attributes() {
    // KMIP Specification v2.1 §§3.2–3.3, Tables 13–15; §11.18 Table 450; §6.1.43 Table 291.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.43-005";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    result
        .validate_request(&register_wrapped_key_value_request(Value::enumeration(2)))
        .expect(
            "caller-supplied TTLV-encoded wrapped Key Value with embedded attributes is usable",
        );
    result
        .validate_request(&register_request_with_key_value_location(None, true))
        .expect("wrapping remains optional for a Register request");

    let malformed = register_wrapped_key_value_request(Value::integer(2));
    let error = result
        .validate_request(&malformed)
        .expect_err("Encoding Option must use its source-defined Enumeration wire type");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(
        error.source_clause_id(),
        Some("KMIPKIT-CLAUSE-SPEC-6.1.43-005")
    );
}

#[test]
fn profile_maps_required_fields_of_opaque_register_object() {
    // KMIP Specification v2.1 §2.3, Table 4 requires both typed child fields.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-003";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == REQUIREMENT_ID)
        .expect("Opaque Object requirement has generated source metadata");
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("Opaque Object requirement has an explicit typed mapping");
    validate_mapping(BASELINE_PROFILE_ID, requirement, mapping)
        .expect("the typed mapping preserves its catalogued source gap and locator");
    let mismatched_mapping = TypedRuleMapping {
        source_clause_id: "KMIPKIT-CLAUSE-SPEC-2.3-003",
        ..*mapping
    };
    let mismatch = validate_mapping(BASELINE_PROFILE_ID, requirement, &mismatched_mapping)
        .expect_err("a source gap cannot be relabeled as a clause");
    assert_eq!(mismatch.source_clause_id(), None);
    assert_eq!(
        mismatch.source_gap_id(),
        Some("KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED")
    );
    assert_eq!(
        mismatch.source_gap_locator(),
        Some("Section 2.3, table object_definition, row Opaque Data Type, HTML lines 6927-6945")
    );
    validation
        .validate_request(&request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            opaque_object_register_payload(false),
        ))
        .expect("a valid Opaque Object satisfies its required Table 4 fields");

    for malformed_object in [
        Value::structure(structure([])),
        Value::text_string("not-an-Opaque-Object-structure".to_owned()),
        Value::structure(structure([tagged(
            OPAQUE_DATA_TYPE_TAG,
            Value::enumeration(0xF123_4567),
        )])),
        Value::structure(structure([tagged(
            OPAQUE_DATA_VALUE_TAG,
            Value::byte_string(b"opaque-data".to_vec()),
        )])),
        Value::structure(structure([
            tagged(OPAQUE_DATA_TYPE_TAG, Value::integer(1)),
            tagged(
                OPAQUE_DATA_VALUE_TAG,
                Value::byte_string(b"opaque-data".to_vec()),
            ),
        ])),
        Value::structure(structure([
            tagged(OPAQUE_DATA_TYPE_TAG, Value::enumeration(0xF123_4567)),
            tagged(
                OPAQUE_DATA_VALUE_TAG,
                Value::text_string("not-opaque-bytes".to_owned()),
            ),
        ])),
    ] {
        let request = request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            raw_opaque_object_register_payload(malformed_object),
        );
        let error = validation
            .validate_request(&request)
            .expect_err("Opaque Object violates the required §2.3/Table 4 child shape");
        assert_eq!(
            error.source_clause_id(),
            None,
            "a table-body requirement must not be reported as a clause"
        );
        assert_eq!(
            error.source_gap_id(),
            Some("KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED")
        );
        assert_eq!(
            error.source_gap_locator(),
            Some(
                "Section 2.3, table object_definition, row Opaque Data Type, HTML lines 6927-6945"
            )
        );
        let display = error.to_string();
        assert!(
            display.contains("source gap KMIPKIT-SOURCE-GAP-SPEC-2.3-OPAQUE-DATA-TYPE-REQUIRED")
        );
        assert!(display.contains("HTML lines 6927-6945"));
    }
}

#[test]
fn profile_opaque_rule_accepts_non_applicable_and_response_contexts() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-003";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    assert!(!client_register_opaque_object_has_required_data_fields(
        None, None
    ));

    for request in [
        request_message_with_operation_payload(0x0000_000A, Structure::new()),
        request_message_with_operation_payload(REGISTER_OPERATION_TAG, Structure::new()),
        request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            structure([tagged(
                OBJECT_TYPE_TAG,
                Value::text_string("opaque".to_owned()),
            )]),
        ),
        request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(2))]),
        ),
    ] {
        validation
            .validate_request(&request)
            .expect("non-applicable Object Types are evaluated by their own requirements");
    }

    let missing_opaque_object = request_message_with_operation_payload(
        REGISTER_OPERATION_TAG,
        structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(8))]),
    );
    validation
        .validate_request(&missing_opaque_object)
        .expect_err("an Opaque Object Register request requires its object structure");

    let valid_request = request_message_with_operation_payload(
        REGISTER_OPERATION_TAG,
        opaque_object_register_payload(false),
    );
    validation
        .validate_response(&valid_request, &response_message())
        .expect("response validation preserves the already checked request rule");
}

#[test]
fn profile_validation_error_rejects_conflicting_source_references() {
    let conflicting = ProfileValidationError::for_rule_requirement(
        ProfileValidationErrorCode::RuleMappingMismatch,
        "KMIPKIT-PROFILE-BASELINE-CLIENT",
        "KMIPKIT-REQ-SPEC-2.3-003",
        "KMIPKIT-CLAUSE-SPEC-2.3-001",
        Some("KMIPKIT-SOURCE-GAP-UNKNOWN"),
    );
    assert_eq!(conflicting.source_clause_id(), None);
    assert_eq!(conflicting.source_gap_id(), None);
    assert_eq!(conflicting.source_gap_locator(), None);

    let unresolved_gap = ProfileValidationError::for_rule_requirement(
        ProfileValidationErrorCode::RuleMappingMismatch,
        "KMIPKIT-PROFILE-BASELINE-CLIENT",
        "KMIPKIT-REQ-SPEC-2.3-003",
        "",
        Some("KMIPKIT-SOURCE-GAP-UNKNOWN"),
    );
    assert_eq!(unresolved_gap.source_clause_id(), None);
    assert_eq!(
        unresolved_gap.source_gap_id(),
        Some("KMIPKIT-SOURCE-GAP-UNKNOWN")
    );
    assert_eq!(unresolved_gap.source_gap_locator(), None);
    assert!(!unresolved_gap.to_string().contains(" at "));
}

#[test]
fn profile_accepts_ttlv_mask_integer_values() {
    // KMIP Specification v2.1 §12 requires every TTLV mask value to use Integer encoding.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-12-001-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const PROTECTION_STORAGE_MASKS_TAG: u32 = 0x0042_015F;
    const PROTECTION_STORAGE_MASK_TAG: u32 = 0x0042_015E;
    const STORAGE_STATUS_MASK_TAG: u32 = 0x0042_008E;
    const GET_OPERATION_TAG: u32 = 0x0000_000A;
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-12-001";

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REQUIREMENT_ID)
        .expect("the client bit-mask encoding requirement has a typed profile mapping");
    assert_eq!(mapping.source_clause_id, SOURCE_CLAUSE_ID);

    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let valid_payload = structure([
        tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(-1)),
        tagged(STORAGE_STATUS_MASK_TAG, Value::integer(i32::MIN)),
        tagged(
            PROTECTION_STORAGE_MASKS_TAG,
            Value::structure(structure([tagged(
                PROTECTION_STORAGE_MASK_TAG,
                Value::integer(i32::MAX),
            )])),
        ),
    ]);
    let valid_request = request_message_with_operation_payload(GET_OPERATION_TAG, valid_payload);
    validation
        .validate_request(&valid_request)
        .expect("all request mask values use TTLV Integer items");

    let valid_response =
        response_message_with_mask_value(GET_OPERATION_TAG, Value::integer(i32::MIN));
    validation
        .validate_response(&valid_request, &valid_response)
        .expect("Integer mask values are valid in received responses");
}

#[test]
fn profile_rejects_non_integer_ttlv_mask_values() {
    // KMIP Specification v2.1 §12 requires every TTLV mask value to use Integer encoding.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-12-001-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const PROTECTION_STORAGE_MASKS_TAG: u32 = 0x0042_015F;
    const PROTECTION_STORAGE_MASK_TAG: u32 = 0x0042_015E;
    const STORAGE_STATUS_MASK_TAG: u32 = 0x0042_008E;
    const GET_OPERATION_TAG: u32 = 0x0000_000A;
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-12-001";

    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let valid_request = request_message_with_operation_payload(
        GET_OPERATION_TAG,
        structure([tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(1))]),
    );
    for invalid_payload in [
        structure([tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::enumeration(1))]),
        structure([tagged(STORAGE_STATUS_MASK_TAG, Value::enumeration(1))]),
        structure([tagged(
            PROTECTION_STORAGE_MASKS_TAG,
            Value::structure(structure([tagged(
                PROTECTION_STORAGE_MASK_TAG,
                Value::enumeration(1),
            )])),
        )]),
    ] {
        let request = request_message_with_operation_payload(GET_OPERATION_TAG, invalid_payload);
        let error = validation
            .validate_request(&request)
            .expect_err("a non-Integer mask value violates KMIP Specification §12");
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
    }

    let invalid_response =
        response_message_with_mask_value(GET_OPERATION_TAG, Value::enumeration(1));
    let error = validation
        .validate_response(&valid_request, &invalid_response)
        .expect_err("a non-Integer response mask violates the TTLV encoding rule");
    assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    assert_eq!(error.source_clause_id(), Some(SOURCE_CLAUSE_ID));
}

fn response_message_with_mask_value(
    operation_tag: u32,
    mask_value: Value,
) -> crate::ResponseMessage {
    crate::ResponseMessage::try_from_ttlv(structure([
        tagged(
            RESPONSE_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(TIME_STAMP_TAG, Value::date_time(1)),
                tagged(BATCH_COUNT_TAG, Value::integer(1)),
            ])),
        ),
        tagged(
            BATCH_ITEM_TAG,
            Value::structure(structure([
                tagged(OPERATION_TAG, Value::enumeration(operation_tag)),
                tagged(RESULT_STATUS_TAG, Value::enumeration(0)),
                tagged(
                    RESPONSE_PAYLOAD_TAG,
                    Value::structure(structure([tagged(
                        CRYPTOGRAPHIC_USAGE_MASK_TAG,
                        mask_value,
                    )])),
                ),
            ])),
        ),
    ]))
    .expect("response with a structurally valid mask value")
}

#[test]
fn profile_maps_client_register_opaque_object_context_custom_attributes() {
    // KMIP Specification v2.1 §2.3, §4.60/Table 150, and §6.1.43/Table 291.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let payload = opaque_object_register_payload(true);
    let payload_view = payload.view();
    let fields = payload_view.children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [OBJECT_TYPE_TAG, ATTRIBUTES_TAG, OPAQUE_OBJECT_TAG]
    );
    let context_values = fields[1].with_value(|value| match value {
        ValueView::Structure(attributes) => attributes
            .children()
            .iter()
            .map(vendor_attribute_value_bytes)
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    });
    assert_eq!(
        context_values,
        [Some(b"context-a".to_vec()), Some(b"context-b".to_vec())],
        "Register preserves repeated caller-supplied Opaque Object context values in order"
    );

    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    validation
        .validate_request(&request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            opaque_object_register_payload(true),
        ))
        .expect("the client can Register an Opaque Object with legacy Custom Attribute context");
    validation
        .validate_request(&request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            opaque_object_register_payload(false),
        ))
        .expect("using the optional context capability does not make it mandatory");
}

#[test]
fn profile_maps_client_register_opaque_object_context_managed_attributes() {
    // KMIP Specification v2.1 §2.3 and §6.1.43, Tables 291/293.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let managed_payload = managed_opaque_object_register_payload();
    let managed_payload_view = managed_payload.view();
    let fields = managed_payload_view.children();
    assert_eq!(
        fields
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [OBJECT_TYPE_TAG, ATTRIBUTES_TAG, OPAQUE_OBJECT_TAG],
        "managed client context is not transmitted as a wire marker"
    );

    validation
        .validate_request(&request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            managed_payload,
        ))
        .expect("the client can Register an Opaque Object using managed cryptographic attributes");
    validation
        .validate_request(&request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            opaque_object_register_payload(false),
        ))
        .expect("the optional managed context does not make attributes mandatory");
}

#[test]
fn profile_rejects_malformed_managed_opaque_attribute_wire_types() {
    // KMIP Specification v2.1 §2.3 and §6.1.43, Tables 291/293.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    for managed_attributes in [
        structure([
            tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3)),
            tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::integer(3)),
            tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::integer(256)),
        ]),
        structure([
            tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::enumeration(3)),
            tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::enumeration(3)),
            tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::integer(256)),
        ]),
        structure([
            tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3)),
            tagged(CRYPTOGRAPHIC_ALGORITHM_TAG, Value::enumeration(3)),
            tagged(CRYPTOGRAPHIC_LENGTH_TAG, Value::enumeration(256)),
        ]),
    ] {
        let error = validation
            .validate_request(&request_message_with_operation_payload(
                REGISTER_OPERATION_TAG,
                managed_opaque_register_payload_with_attributes(managed_attributes),
            ))
            .expect_err("managed cryptographic attributes use their required TTLV wire types");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-2.3-002")
        );
    }
}

#[test]
fn profile_managed_opaque_rule_is_scoped_to_register_request_metadata() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    for payload in [
        Structure::new(),
        structure([tagged(
            OBJECT_TYPE_TAG,
            Value::text_string("not an Enumeration".to_owned()),
        )]),
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
        ]),
    ] {
        let error = validation
            .validate_request(&request_message_with_operation_payload(
                REGISTER_OPERATION_TAG,
                payload,
            ))
            .expect_err("Register requires one well-formed Object Type selector");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-2.3-002")
        );
    }

    for payload in [
        structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(2))]),
        structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(8))]),
    ] {
        validation
            .validate_request(&request_message_with_operation_payload(
                REGISTER_OPERATION_TAG,
                payload,
            ))
            .expect("non-Opaque and attribute-free Register metadata remain outside this capability check");
    }

    let malformed_attributes = [
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
            tagged(ATTRIBUTES_TAG, Value::integer(1)),
        ]),
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
            opaque_object_attributes_ttlv([
                tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3)),
                tagged(CRYPTOGRAPHIC_USAGE_MASK_TAG, Value::integer(3)),
            ]),
        ]),
        structure([
            tagged(OBJECT_TYPE_TAG, Value::enumeration(8)),
            opaque_object_attributes_ttlv([]),
            opaque_object_attributes_ttlv([]),
        ]),
    ];

    for payload in malformed_attributes {
        let error = validation
            .validate_request(&request_message_with_operation_payload(
                REGISTER_OPERATION_TAG,
                payload,
            ))
            .expect_err("malformed managed Opaque Object attribute metadata fails closed");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    }

    validation
        .validate_request(&request_message())
        .expect("the managed Opaque Object rule does not constrain other client operations");

    let register_request = request_message_with_operation_payload(
        REGISTER_OPERATION_TAG,
        managed_opaque_object_register_payload(),
    );
    validation
        .validate_response(
            &register_request,
            &get_attributes_response_with_payload(REGISTER_OPERATION_TAG, Structure::new()),
        )
        .expect("the client-side capability rule does not revalidate server responses");

    assert!(!crate::profile::client_opaque_object_context_managed_attributes(None, None));
}

#[test]
fn profile_opaque_context_capability_does_not_require_or_revalidate_other_register_fields() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };

    let out_of_scope_register_payloads = [
        Structure::new(),
        structure([tagged(
            OBJECT_TYPE_TAG,
            Value::text_string("not an Enumeration".to_owned()),
        )]),
        structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(8))]),
    ];

    for payload in out_of_scope_register_payloads {
        result
            .validate_request(&request_message_with_operation_payload(
                REGISTER_OPERATION_TAG,
                payload,
            ))
            .expect("the optional context rule only checks an Opaque Register's Attributes");
    }
}

#[test]
fn profile_maps_client_get_attributes_opaque_object_custom_context() {
    // KMIP Specification v2.1 §2.3 and §6.1.20/Table 224.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let request = request_message_with_operation_payload(
        GET_ATTRIBUTES_OPERATION,
        structure([tagged(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("opaque-object".to_owned()),
        )]),
    );
    let response = opaque_context_get_attributes_response();
    let response_item = response
        .batch_items()
        .next()
        .expect("the Get Attributes response contains one result item");
    let typed_response = crate::GetAttributesResponse::try_from_response_item(response_item)
        .expect("Get Attributes retains generic Vendor Attribute values");
    let attributes = typed_response
        .attributes()
        .expect("the successful response contains its ordered attributes");
    assert_eq!(
        attributes
            .as_items()
            .iter()
            .map(vendor_attribute_value_bytes)
            .collect::<Vec<_>>(),
        [Some(b"context-a".to_vec()), Some(b"context-b".to_vec())]
    );

    let validation = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    validation
        .validate_response(&request, &response)
        .expect("the client can retrieve the same ordered Opaque Object context attributes");
}

#[test]
fn profile_rejects_malformed_opaque_object_context_shapes() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let invalid_payloads = malformed_opaque_register_metadata_payloads();

    for payload in invalid_payloads {
        let error = result
            .validate_request(&request_message_with_operation_payload(
                REGISTER_OPERATION_TAG,
                payload,
            ))
            .expect_err("malformed Opaque Object context must fail closed");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-2.3-001")
        );
    }

    result
        .validate_request(&request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            structure([tagged(OBJECT_TYPE_TAG, Value::enumeration(2))]),
        ))
        .expect("the Opaque Object capability rule does not constrain other object types");
    result
        .validate_request(&request_message())
        .expect("the Opaque Object capability rule does not constrain unrelated operations");
    assert!(!client_opaque_object_context_custom_attributes(None, None));
}

#[test]
fn profile_rejects_malformed_opaque_context_get_attributes_responses() {
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-2.3-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let request = request_message_with_operation_payload(
        GET_ATTRIBUTES_OPERATION,
        structure([tagged(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("opaque-object".to_owned()),
        )]),
    );

    result
        .validate_response(
            &request,
            &get_attributes_response_with_payload(
                GET_ATTRIBUTES_OPERATION,
                structure([tagged(
                    UNIQUE_IDENTIFIER_TAG,
                    Value::text_string("opaque-object".to_owned()),
                )]),
            ),
        )
        .expect("the optional Attributes field may be absent from a Get Attributes response");

    let malformed_responses = [
        get_attributes_response_with_payload(
            GET_ATTRIBUTES_OPERATION,
            structure([
                tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
                tagged(ATTRIBUTES_TAG, Value::structure(Structure::new())),
            ]),
        ),
        get_attributes_response_with_payload(
            GET_ATTRIBUTES_OPERATION,
            structure([tagged(
                ATTRIBUTES_TAG,
                Value::text_string("not a Structure".to_owned()),
            )]),
        ),
        get_attributes_response_with_payload(
            GET_ATTRIBUTES_OPERATION,
            structure([tagged(
                ATTRIBUTES_TAG,
                Value::structure(structure([tagged(
                    VENDOR_ATTRIBUTE_TAG,
                    Value::text_string("not a Vendor Attribute structure".to_owned()),
                )])),
            )]),
        ),
    ];

    for response in malformed_responses {
        let error = result
            .validate_response(&request, &response)
            .expect_err("malformed returned Vendor Attributes must fail closed");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::ResponseRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-2.3-001")
        );
    }

    result
        .validate_response(
            &request_message(),
            &opaque_context_get_attributes_response(),
        )
        .expect("a response to another operation is outside this optional capability");
    let mismatched_operation = get_attributes_response_with_payload(0x0000_001E, Structure::new());
    let error = result
        .validate_response(&request, &mismatched_operation)
        .expect_err("a Get Attributes request requires its matching response operation");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::ResponseRuleRejected
    );
}

#[test]
fn profile_maps_client_get_attributes_response_x509_values() {
    let request =
        request_message_with_operation_payload(GET_ATTRIBUTES_OPERATION, Structure::new());
    let response = get_attributes_x509_response();
    let response_item = response
        .batch_items()
        .next()
        .expect("test response contains one Get Attributes batch item");
    let typed_response = crate::GetAttributesResponse::try_from_response_item(response_item)
        .expect("the typed response model accepts the X.509 Byte String attributes");
    let attributes = typed_response
        .attributes()
        .expect("successful Get Attributes exposes its attributes");
    let retained_values = attributes
        .as_items()
        .iter()
        .map(|attribute| {
            let value = attribute.with_value(|value| match value {
                ValueView::ByteString(bytes) => Some(bytes.to_vec()),
                _ => None,
            });
            (attribute.tag().raw(), value)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        retained_values,
        [
            (ISSUER_ALTERNATIVE_NAME_TAG, Some(b"issuer-name-a".to_vec())),
            (SUBJECT_DISTINGUISHED_NAME_TAG, Some(Vec::new())),
            (
                SUBJECT_ALTERNATIVE_NAME_TAG,
                Some(b"subject-name-a".to_vec())
            ),
            (ISSUER_ALTERNATIVE_NAME_TAG, Some(b"issuer-name-b".to_vec())),
            (
                SUBJECT_ALTERNATIVE_NAME_TAG,
                Some(b"subject-name-b".to_vec())
            ),
        ],
        "the response model preserves empty and repeated X.509 values in source order"
    );

    let selected = ProfileValidationResult {
        profile_ids: vec!["KMIPKIT-PROFILE-TEST-CLIENT".to_owned()],
        requirement_ids: [
            "KMIPKIT-REQ-SPEC-4.62-003",
            "KMIPKIT-REQ-SPEC-4.63-004",
            "KMIPKIT-REQ-SPEC-4.63-005",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    };
    selected
        .validate_request(&request)
        .expect("response-only optional rules do not reject the matching outbound request");
    selected
        .validate_response(&request, &response)
        .expect("mapped client response rules accept the retained optional X.509 values");
    selected
        .validate_response(&request_message(), &response_message())
        .expect("X.509 response rules do not constrain unrelated operations");
}

#[test]
fn profile_rejects_malformed_register_key_value_location_members() {
    // KMIP Specification v2.1 §4.27/Table 82 requires both typed members.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.27-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const KEY_VALUE_LOCATION: u32 = 0x0042_00B8;
    const KEY_VALUE_LOCATION_VALUE: u32 = 0x0042_00B9;
    const KEY_VALUE_LOCATION_TYPE: u32 = 0x0042_00BA;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let malformed_attributes = [
        structure([tagged(
            KEY_VALUE_LOCATION_VALUE,
            Value::text_string("pkcs11:object=key-1".to_owned()),
        )]),
        structure([tagged(KEY_VALUE_LOCATION_TYPE, Value::enumeration(2))]),
        structure([
            tagged(
                KEY_VALUE_LOCATION_VALUE,
                Value::byte_string(b"pkcs11:object=key-1".to_vec()),
            ),
            tagged(KEY_VALUE_LOCATION_TYPE, Value::enumeration(2)),
        ]),
        structure([
            tagged(
                KEY_VALUE_LOCATION_VALUE,
                Value::text_string("pkcs11:object=key-1".to_owned()),
            ),
            tagged(KEY_VALUE_LOCATION_TYPE, Value::integer(2)),
        ]),
    ];

    for malformed in malformed_attributes {
        let location = tagged(KEY_VALUE_LOCATION, Value::structure(malformed));
        let request = register_request_with_key_value_location(Some(location), false);
        let error = result
            .validate_request(&request)
            .expect_err("Key Value Location must contain one required field of each type");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
        assert_eq!(
            error.source_clause_id(),
            Some("KMIPKIT-CLAUSE-SPEC-4.27-001")
        );
    }
}

#[test]
fn profile_rejects_malformed_register_object_key_block_shapes_for_key_value_location() {
    // KMIP Specification v2.1 §4.27/Table 82 and §6.1.43/Table 291.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.27-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const KEY_VALUE_LOCATION: u32 = 0x0042_00B8;
    const KEY_VALUE_LOCATION_VALUE: u32 = 0x0042_00B9;
    const KEY_VALUE_LOCATION_TYPE: u32 = 0x0042_00BA;
    const SYMMETRIC_KEY: u32 = 0x0042_008F;
    const KEY_BLOCK: u32 = 0x0042_0040;
    const KEY_VALUE: u32 = 0x0042_0045;

    let result = ProfileValidationResult {
        profile_ids: vec![BASELINE_PROFILE_ID.to_owned()],
        requirement_ids: vec![REQUIREMENT_ID.to_owned()],
    };
    let key_value_location = || {
        tagged(
            KEY_VALUE_LOCATION,
            Value::structure(structure([
                tagged(
                    KEY_VALUE_LOCATION_VALUE,
                    Value::text_string("pkcs11:object=key-1".to_owned()),
                ),
                tagged(KEY_VALUE_LOCATION_TYPE, Value::enumeration(2)),
            ])),
        )
    };
    let valid_key_block = || tagged(KEY_BLOCK, Value::structure(Structure::new()));
    let malformed_objects = [
        tagged(SYMMETRIC_KEY, Value::structure(Structure::new())),
        tagged(
            SYMMETRIC_KEY,
            Value::structure(structure([valid_key_block(), valid_key_block()])),
        ),
        tagged(
            SYMMETRIC_KEY,
            Value::structure(structure([tagged(
                KEY_BLOCK,
                Value::structure(structure([
                    tagged(KEY_VALUE, Value::byte_string(b"key-a".to_vec())),
                    tagged(KEY_VALUE, Value::byte_string(b"key-b".to_vec())),
                ])),
            )])),
        ),
        tagged(
            SYMMETRIC_KEY,
            Value::structure(structure([tagged(
                KEY_BLOCK,
                Value::text_string("not a Structure".to_owned()),
            )])),
        ),
        tagged(
            SYMMETRIC_KEY,
            Value::text_string("not an Object Structure".to_owned()),
        ),
    ];

    for object in malformed_objects {
        let request = request_message_with_operation_payload(
            REGISTER_OPERATION_TAG,
            structure([
                tagged(OBJECT_TYPE_TAG, Value::enumeration(2)),
                tagged(
                    ATTRIBUTES_TAG,
                    Value::structure(structure([key_value_location()])),
                ),
                object,
            ]),
        );
        let error = result
            .validate_request(&request)
            .expect_err("Key Value Location requires one well-formed Key Block without Key Value");
        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(error.requirement_id(), Some(REQUIREMENT_ID));
    }
}

const REVOKE_OPERATION: u32 = 0x0000_0013;
const REVOCATION_REASON: u32 = 0x0042_0081;
const REVOCATION_REASON_CODE: u32 = 0x0042_0082;
const REVOCATION_MESSAGE: u32 = 0x0042_0080;
const COMPROMISE_OCCURRENCE_DATE: u32 = 0x0042_0021;
const REVOKE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.44-003-002";
const REVOKE_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.44-003";
const REVOKE_MESSAGE_REQUIREMENT_IDS: [&str; 2] = [
    "KMIPKIT-REQ-SPEC-4.47-001-001",
    "KMIPKIT-REQ-SPEC-4.47-001-002",
];
const REVOKE_MESSAGE_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.47-001";

fn revoke_reason_item(reason_code: Value) -> Item {
    tagged(
        REVOCATION_REASON,
        Value::structure(structure([tagged(REVOCATION_REASON_CODE, reason_code)])),
    )
}

fn revoke_profile_request(
    reason_fields: impl IntoIterator<Item = Item>,
    include_compromise_occurrence_date: bool,
) -> crate::RequestMessage {
    let mut payload: Vec<_> = reason_fields.into_iter().collect();
    if include_compromise_occurrence_date {
        payload.push(tagged(COMPROMISE_OCCURRENCE_DATE, Value::date_time(0)));
    }
    request_message_with_operation_payload(REVOKE_OPERATION, structure(payload))
}

fn revoke_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec!["KMIPKIT-PROFILE-BASELINE-CLIENT".to_owned()],
        requirement_ids: vec![REVOKE_REQUIREMENT_ID.to_owned()],
    }
}

fn revoke_message_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec!["KMIPKIT-PROFILE-BASELINE-CLIENT".to_owned()],
        requirement_ids: REVOKE_MESSAGE_REQUIREMENT_IDS
            .iter()
            .map(|id| (*id).to_owned())
            .collect(),
    }
}

fn assert_revoke_profile_request_rejected(request: &crate::RequestMessage) {
    let error = revoke_profile_result()
        .validate_request(request)
        .expect_err("a date with a non-compromise or malformed reason is rejected");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(REVOKE_REQUIREMENT_ID));
    assert_eq!(error.source_clause_id(), Some(REVOKE_SOURCE_CLAUSE_ID));
}

#[test]
fn profile_maps_revoke_compromise_occurrence_date_shall_not_rule() {
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == REVOKE_REQUIREMENT_ID);
    assert!(
        mapping.is_some(),
        "the Revoke SHALL NOT has a typed profile mapping"
    );
    if let Some(mapping) = mapping {
        assert_eq!(mapping.source_clause_id, REVOKE_SOURCE_CLAUSE_ID);
        assert_eq!(
            mapping.verification_category,
            GeneratedVerificationCategory::ProhibitedBehavior
        );
        assert!(mapping.test_evidence.iter().any(|evidence| {
            evidence.kind == RuleTestKind::NegativeBehavior
                && evidence.test_id.ends_with(
                    "profile_rejects_revoke_compromise_occurrence_date_for_other_reasons",
                )
        }));
        assert!(mapping.test_evidence.iter().any(|evidence| {
            evidence.kind == RuleTestKind::NegativeBehavior
                && evidence.test_id.ends_with(
                    "profile_rejects_revoke_compromise_occurrence_date_when_reason_is_malformed",
                )
        }));
    }
}

#[test]
// Normative sources: KMIP Specification v2.1 §4.47 Table 123 and §6.1.44 Table 295.
fn profile_maps_revoke_revocation_message_as_optional_capability() {
    for requirement_id in REVOKE_MESSAGE_REQUIREMENT_IDS {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id);
        assert!(
            mapping.is_some(),
            "the Revoke Revocation Message permission has a typed profile mapping"
        );
        if let Some(mapping) = mapping {
            assert_eq!(mapping.source_clause_id, REVOKE_MESSAGE_SOURCE_CLAUSE_ID);
            assert_eq!(
                mapping.verification_category,
                GeneratedVerificationCategory::OptionalCapability
            );
            assert!(mapping.test_evidence.iter().any(|evidence| {
                evidence.kind == RuleTestKind::OptionalCapabilityUsable
                    && evidence
                        .test_id
                        .ends_with("profile_accepts_optional_revoke_revocation_message")
            }));
        }
    }
}

// KMIP Specification v2.1 §4.47 Table 123 makes Revocation Message OPTIONAL
// and permits caller-supplied audit context; §6.1.44 Table 295 places
// Revocation Reason in the client-originated Revoke request.
#[test]
fn profile_accepts_optional_revoke_revocation_message() {
    let result = revoke_message_profile_result();
    let request = revoke_profile_request(
        [tagged(
            REVOCATION_REASON,
            Value::structure(structure([
                tagged(REVOCATION_REASON_CODE, Value::enumeration(1)),
                tagged(
                    REVOCATION_MESSAGE,
                    Value::text_string("audit detail".to_owned()),
                ),
            ])),
        )],
        false,
    );

    assert!(result.validate_request(&request).is_ok());
}

#[test]
// KMIP Specification v2.1 §4.47 Table 123 marks the field OPTIONAL.
fn profile_accepts_revoke_without_revocation_message() {
    let request = revoke_profile_request([revoke_reason_item(Value::enumeration(1))], false);

    assert!(
        revoke_message_profile_result()
            .validate_request(&request)
            .is_ok()
    );
}

#[test]
// KMIP Specification v2.1 §4.47 Table 123 defines Revocation Message as Text String.
fn profile_rejects_malformed_revoke_revocation_message() {
    let result = revoke_message_profile_result();
    let request = revoke_profile_request(
        [tagged(
            REVOCATION_REASON,
            Value::structure(structure([
                tagged(REVOCATION_REASON_CODE, Value::enumeration(1)),
                tagged(REVOCATION_MESSAGE, Value::integer(7)),
            ])),
        )],
        false,
    );

    let error = result
        .validate_request(&request)
        .expect_err("Revocation Message uses a Text String value");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(REVOKE_MESSAGE_REQUIREMENT_IDS[0])
    );
    assert_eq!(
        error.source_clause_id(),
        Some(REVOKE_MESSAGE_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_accepts_revoke_compromise_occurrence_date_for_key_and_ca_compromise() {
    // KMIP Specification v2.1 §6.1.44 Table 295; §11.48 Table 481 assigns 2 and 3.
    for reason in [2, 3] {
        let request =
            revoke_profile_request([revoke_reason_item(Value::enumeration(reason))], true);
        assert!(
            revoke_profile_result().validate_request(&request).is_ok(),
            "the SHALL NOT rule must not enforce the adjacent SHOULD for reason {reason}"
        );
    }
}

#[test]
fn profile_rejects_revoke_compromise_occurrence_date_for_other_reasons() {
    // Standard non-compromise reasons and an unknown value remain distinct inputs.
    for reason in [1, 4, 6, 0x8000_0000] {
        let request =
            revoke_profile_request([revoke_reason_item(Value::enumeration(reason))], true);
        assert_revoke_profile_request_rejected(&request);
    }
}

#[test]
fn profile_accepts_revoke_without_compromise_occurrence_date_for_any_reason() {
    // The SHALL NOT rule does not require the adjacent SHOULD field for reasons 2 or 3.
    for reason in [1, 2, 3, 4, 6, 0x8000_0000] {
        let request =
            revoke_profile_request([revoke_reason_item(Value::enumeration(reason))], false);
        assert!(revoke_profile_result().validate_request(&request).is_ok());
    }
}

#[test]
fn profile_rejects_revoke_compromise_occurrence_date_when_reason_is_malformed() {
    let malformed_reasons = [
        Vec::new(),
        vec![tagged(REVOCATION_REASON, Value::integer(2))],
        vec![tagged(
            REVOCATION_REASON,
            Value::structure(Structure::new()),
        )],
        vec![tagged(
            REVOCATION_REASON,
            Value::structure(structure([tagged(
                REVOCATION_REASON_CODE,
                Value::integer(2),
            )])),
        )],
        vec![tagged(
            REVOCATION_REASON,
            Value::structure(structure([
                tagged(REVOCATION_REASON_CODE, Value::enumeration(2)),
                tagged(REVOCATION_REASON_CODE, Value::enumeration(3)),
            ])),
        )],
        vec![
            revoke_reason_item(Value::enumeration(2)),
            revoke_reason_item(Value::enumeration(3)),
        ],
    ];

    for reason_fields in malformed_reasons {
        let request = revoke_profile_request(reason_fields, true);
        assert_revoke_profile_request_rejected(&request);
    }
}

const CERTIFY_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.6-001";
const CERTIFY_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.6-001";
const CERTIFY_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const CERTIFY_OPERATION: u32 = 0x0000_0006;
const CERTIFY_REQUEST_TYPE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.6-007";
const CERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.6-007";
const RECERTIFY_REQUEST_TYPE_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.45-008";
const RECERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.45-008";
const RECERTIFY_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const RECERTIFY_OPERATION_TAG: u32 = 0x0000_0007;
const CERTIFICATE_REQUEST_TYPE_TAG: u32 = 0x0042_0019;
const CERTIFICATE_REQUEST_VALUE_TAG: u32 = 0x0042_0140;
const CERTIFY_REQUEST_OMISSION_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.6-002-001";
const CERTIFY_REQUEST_OMISSION_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.6-002";
const CERTIFY_GET_PLACEHOLDER_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.6-003";
const CERTIFY_GET_PLACEHOLDER_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.6-003";
const CERTIFY_OMITTED_VALUE_IDENTIFIER_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.6-002-002";
const CERTIFY_OMITTED_VALUE_IDENTIFIER_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.6-002";
const CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_REQUIREMENT_ID: &str =
    "KMIPKIT-REQ-SPEC-6.1.6-002-003";
const CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_SOURCE_CLAUSE_ID: &str =
    "KMIPKIT-CLAUSE-SPEC-6.1.6-002";

fn certify_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec![CERTIFY_PROFILE_ID.to_owned()],
        requirement_ids: vec![CERTIFY_REQUIREMENT_ID.to_owned()],
    }
}

fn certify_omitted_value_identifier_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec![CERTIFY_PROFILE_ID.to_owned()],
        requirement_ids: vec![CERTIFY_OMITTED_VALUE_IDENTIFIER_REQUIREMENT_ID.to_owned()],
    }
}

fn certify_omitted_request_certificate_type_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec![CERTIFY_PROFILE_ID.to_owned()],
        requirement_ids: vec![CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_REQUIREMENT_ID.to_owned()],
    }
}

fn certify_request_type_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec![CERTIFY_PROFILE_ID.to_owned()],
        requirement_ids: vec![CERTIFY_REQUEST_TYPE_REQUIREMENT_ID.to_owned()],
    }
}

fn recertify_request_type_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec![RECERTIFY_PROFILE_ID.to_owned()],
        requirement_ids: vec![RECERTIFY_REQUEST_TYPE_REQUIREMENT_ID.to_owned()],
    }
}

fn certify_get_placeholder_profile_result() -> ProfileValidationResult {
    ProfileValidationResult {
        profile_ids: vec![CERTIFY_PROFILE_ID.to_owned()],
        requirement_ids: vec![CERTIFY_GET_PLACEHOLDER_REQUIREMENT_ID.to_owned()],
    }
}

fn certify_payload_with_omitted_request_fields(attributes: Option<Value>) -> Structure {
    let mut payload = vec![tagged(
        UNIQUE_IDENTIFIER_TAG,
        Value::text_string("public-key".to_owned()),
    )];
    if let Some(attributes) = attributes {
        payload.push(tagged(ATTRIBUTES_TAG, attributes));
    }
    structure(payload)
}

fn certify_payload_with_request_values(count: usize) -> Structure {
    structure((0..count).map(|index| {
        tagged(
            CERTIFICATE_REQUEST_VALUE_TAG,
            Value::byte_string(vec![u8::try_from(index).expect("small test index")]),
        )
    }))
}

#[test]
fn profile_maps_certify_generated_certificate_get_in_same_batch_as_optional_capability() {
    // KMIP Specification v2.1 §6.1.6 permits a later Get to retrieve the
    // generated Certificate through the ID Placeholder. §6.1 makes omission
    // of Batch Order Option effectively true, and §6.1.19 Table 220 defines
    // an omitted Get Unique Identifier as use of that placeholder.
    const GET_OPERATION: u32 = 0x0000_000A;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == CERTIFY_GET_PLACEHOLDER_REQUIREMENT_ID)
        .expect("same-batch Certify and Get have a typed optional-capability mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == CERTIFY_GET_PLACEHOLDER_REQUIREMENT_ID)
        .expect("the Certify/Get MAY resolves to generated normative metadata");

    assert_eq!(
        mapping.source_clause_id,
        CERTIFY_GET_PLACEHOLDER_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(CERTIFY_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.kind == RuleTestKind::OptionalCapabilityUsable
            && evidence.test_id.ends_with(
                "profile_maps_certify_generated_certificate_get_in_same_batch_as_optional_capability",
            )
    }));

    let certify_payload = structure([
        tagged(CERTIFICATE_REQUEST_TYPE_TAG, Value::enumeration(1)),
        tagged(
            CERTIFICATE_REQUEST_VALUE_TAG,
            Value::byte_string(vec![0x30, 0x00]),
        ),
    ]);
    let request = request_message_with_batch_operations([
        (CERTIFY_OPERATION, certify_payload),
        (GET_OPERATION, Structure::new()),
    ]);
    assert_eq!(
        request
            .batch_items()
            .map(crate::RequestBatchItemView::operation)
            .collect::<Vec<_>>(),
        [Some(CERTIFY_OPERATION), Some(GET_OPERATION)]
    );
    assert_eq!(request.header().batch_order_option(), None);
    assert!(request.header().effective_batch_order_option());
    let get_item = request
        .batch_items()
        .nth(1)
        .expect("the second Batch Item is Get");
    let get_has_unique_identifier = get_item
        .with_request_payload(|payload| {
            payload
                .children()
                .iter()
                .any(|field| field.tag().raw() == UNIQUE_IDENTIFIER_TAG)
        })
        .expect("Get has a Request Payload");
    assert!(
        !get_has_unique_identifier,
        "Get uses the ID Placeholder by omission"
    );

    certify_get_placeholder_profile_result()
        .validate_request(&request)
        .expect("the client profile accepts Certify followed by Get without Unique Identifier");

    let request_without_the_optional_capability =
        request_message_with_operation_payload(LOCATE_OPERATION_TAG, Structure::new());
    certify_get_placeholder_profile_result()
        .validate_request(&request_without_the_optional_capability)
        .expect("the MAY does not require every request to use Certify and Get");

    let register_get = request_message_with_batch_operations([
        (REGISTER_OPERATION_TAG, Structure::new()),
        (GET_OPERATION, Structure::new()),
    ]);
    certify_get_placeholder_profile_result()
        .validate_request(&register_get)
        .expect("other valid placeholder producer and Get batches remain permitted");
}

#[test]
fn profile_certify_get_placeholder_evaluator_accepts_direct_and_explicit_identifiers() {
    // KMIP Specification v2.1 §11.58 Table 489 defines Enumeration(1) as
    // the ID Placeholder; a direct identifier remains an ordinary Get.
    const GET_OPERATION: u32 = 0x0000_000A;
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == CERTIFY_GET_PLACEHOLDER_REQUIREMENT_ID)
        .expect("the same-batch Certify/Get rule has a typed evaluator");
    let evaluator = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");

    let direct_get = request_message_with_operation_payload(
        GET_OPERATION,
        structure([tagged(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("issued-certificate".to_owned()),
        )]),
    );
    let direct_identifier_count = direct_get
        .batch_items()
        .next()
        .expect("the direct Get contains one Batch Item")
        .with_request_payload(|payload| {
            payload
                .children()
                .iter()
                .filter(|field| field.tag().raw() == UNIQUE_IDENTIFIER_TAG)
                .count()
        })
        .expect("the direct Get has a Request Payload");
    assert_eq!(direct_identifier_count, 1);
    assert!(evaluator(Some(&direct_get), None));
    certify_get_placeholder_profile_result()
        .validate_request(&direct_get)
        .expect("a direct Get identifier does not consume the batch ID Placeholder");

    let explicit_placeholder_get = request_message_with_batch_operations([
        (
            CERTIFY_OPERATION,
            structure([
                tagged(CERTIFICATE_REQUEST_TYPE_TAG, Value::enumeration(1)),
                tagged(
                    CERTIFICATE_REQUEST_VALUE_TAG,
                    Value::byte_string(vec![0x30, 0x00]),
                ),
            ]),
        ),
        (
            GET_OPERATION,
            structure([tagged(UNIQUE_IDENTIFIER_TAG, Value::enumeration(1))]),
        ),
    ]);
    assert!(evaluator(Some(&explicit_placeholder_get), None));
    certify_get_placeholder_profile_result()
        .validate_request(&explicit_placeholder_get)
        .expect("an explicit ID Placeholder follows a successful-path Certify item");
}

#[test]
fn profile_rejects_get_id_placeholder_without_a_preceding_producer() {
    const GET_OPERATION: u32 = 0x0000_000A;
    let request = request_message_with_operation_payload(GET_OPERATION, Structure::new());

    let error = certify_get_placeholder_profile_result()
        .validate_request(&request)
        .expect_err("a Get that uses the ID Placeholder needs a prior producer");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_GET_PLACEHOLDER_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_GET_PLACEHOLDER_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_rejects_get_id_placeholder_when_batch_order_is_false() {
    const GET_OPERATION: u32 = 0x0000_000A;
    let request = request_message_with_batch_operations_and_order(
        [
            (CERTIFY_OPERATION, Structure::new()),
            (GET_OPERATION, Structure::new()),
        ],
        Some(false),
    );

    let error = certify_get_placeholder_profile_result()
        .validate_request(&request)
        .expect_err("ID Placeholder use requires Batch Order Option true");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_GET_PLACEHOLDER_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_GET_PLACEHOLDER_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_maps_certify_single_certificate_request_value_as_required_behavior() {
    // KMIP Specification v2.1 §6.1.6 paragraph 1 limits each Certify request
    // payload to at most one Certificate Request Value.
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == CERTIFY_REQUIREMENT_ID)
        .expect("the Certify single-certificate SHALL has a typed profile mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == CERTIFY_REQUIREMENT_ID)
        .expect("the Certify SHALL resolves to generated normative metadata");

    assert_eq!(mapping.source_clause_id, CERTIFY_SOURCE_CLAUSE_ID);
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(CERTIFY_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.kind == RuleTestKind::PositiveBehavior
            && evidence.test_id.ends_with(
                "profile_accepts_at_most_one_certify_certificate_request_value_per_batch_item",
            )
    }));
}

#[test]
fn profile_accepts_at_most_one_certify_certificate_request_value_per_batch_item() {
    let result = certify_profile_result();

    for count in [0, 1] {
        let request = request_message_with_operation_payload(
            CERTIFY_OPERATION,
            certify_payload_with_request_values(count),
        );
        result
            .validate_request(&request)
            .expect("an omitted or single Certificate Request Value is permitted");
    }

    let request = request_message_with_batch_operations([
        (CERTIFY_OPERATION, certify_payload_with_request_values(1)),
        (CERTIFY_OPERATION, certify_payload_with_request_values(1)),
    ]);
    result
        .validate_request(&request)
        .expect("each Certify Batch Item may request one certificate");
}

#[test]
fn profile_rejects_duplicate_certify_certificate_request_values_per_batch_item() {
    let request = request_message_with_operation_payload(
        CERTIFY_OPERATION,
        certify_payload_with_request_values(2),
    );
    let error = certify_profile_result()
        .validate_request(&request)
        .expect_err("a Certify Batch Item cannot request two certificates");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(error.requirement_id(), Some(CERTIFY_REQUIREMENT_ID));
    assert_eq!(error.source_clause_id(), Some(CERTIFY_SOURCE_CLAUSE_ID));
}

#[test]
fn profile_maps_certify_request_type_when_request_value_is_present_as_required_behavior() {
    // KMIP Specification v2.1 §6.1.6 Table 179 requires Certificate Request
    // Type when the Certificate Request Value is present.
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == CERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
        .expect("the conditional Certify Certificate Request Type REQUIRED has a typed mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == CERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
        .expect("the conditional Certify REQUIRED resolves to normative metadata");

    assert_eq!(
        mapping.source_clause_id,
        CERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(CERTIFY_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.kind == RuleTestKind::PositiveBehavior
            && evidence
                .test_id
                .ends_with("profile_accepts_certificate_request_type_with_request_value")
    }));

    let evaluate = mapping
        .evaluator
        .expect("the conditional REQUIRED has a typed evaluator");
    assert!(!evaluate(None, None));
    let unrelated_request =
        request_message_with_operation_payload(LOCATE_OPERATION_TAG, Structure::new());
    assert!(evaluate(Some(&unrelated_request), None));
}

#[test]
fn profile_maps_recertify_request_type_when_request_value_is_present_as_required_behavior() {
    // KMIP Specification v2.1 §6.1.45 Table 300 requires Certificate Request
    // Type when Certificate Request Value is present.
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == RECERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
        .expect("the conditional Re-certify Certificate Request Type REQUIRED has a typed mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == RECERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
        .expect("the conditional Re-certify REQUIRED resolves to normative metadata");

    assert_eq!(
        mapping.source_clause_id,
        RECERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(RECERTIFY_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.kind == RuleTestKind::PositiveBehavior
            && evidence
                .test_id
                .ends_with("profile_accepts_recertify_request_type_with_request_value")
    }));

    let evaluate = mapping
        .evaluator
        .expect("the conditional REQUIRED has a typed evaluator");
    assert!(!evaluate(None, None));
    let unrelated_request =
        request_message_with_operation_payload(LOCATE_OPERATION_TAG, Structure::new());
    assert!(evaluate(Some(&unrelated_request), None));
}

#[test]
fn profile_accepts_recertify_request_type_with_request_value() {
    let payload = structure([
        tagged(CERTIFICATE_REQUEST_TYPE_TAG, Value::enumeration(1)),
        tagged(
            CERTIFICATE_REQUEST_VALUE_TAG,
            Value::byte_string(vec![0x30, 0x00]),
        ),
    ]);
    let request = request_message_with_operation_payload(RECERTIFY_OPERATION_TAG, payload);

    recertify_request_type_profile_result()
        .validate_request(&request)
        .expect("Certificate Request Type accompanies the supplied Certificate Request Value");
}

#[test]
fn profile_rejects_recertify_request_value_without_request_type_in_any_batch_item() {
    let valid_payload = structure([
        tagged(CERTIFICATE_REQUEST_TYPE_TAG, Value::enumeration(1)),
        tagged(
            CERTIFICATE_REQUEST_VALUE_TAG,
            Value::byte_string(vec![0x30, 0x00]),
        ),
    ]);
    let missing_type_payload = structure([tagged(
        CERTIFICATE_REQUEST_VALUE_TAG,
        Value::byte_string(vec![0x30, 0x00]),
    )]);
    let request = request_message_with_batch_operations([
        (RECERTIFY_OPERATION_TAG, valid_payload),
        (RECERTIFY_OPERATION_TAG, missing_type_payload),
    ]);
    let error = recertify_request_type_profile_result()
        .validate_request(&request)
        .expect_err("each Re-certify Batch Item with Certificate Request Value needs its type");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(RECERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(RECERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_rejects_malformed_recertify_request_type_when_request_value_is_present() {
    let payload = structure([
        tagged(CERTIFICATE_REQUEST_TYPE_TAG, Value::integer(1)),
        tagged(
            CERTIFICATE_REQUEST_VALUE_TAG,
            Value::byte_string(vec![0x30, 0x00]),
        ),
    ]);
    let request = request_message_with_operation_payload(RECERTIFY_OPERATION_TAG, payload);
    let error = recertify_request_type_profile_result()
        .validate_request(&request)
        .expect_err("Table 300 defines Certificate Request Type as an Enumeration");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(RECERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(RECERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_accepts_recertify_request_without_certificate_request_fields() {
    let request = request_message_with_operation_payload(RECERTIFY_OPERATION_TAG, Structure::new());

    recertify_request_type_profile_result()
        .validate_request(&request)
        .expect("the conditional rule does not apply when Certificate Request Value is omitted");
}

#[test]
fn profile_accepts_certificate_request_type_with_request_value() {
    let payload = crate::CertifyRequest::new()
        .with_certificate_request_type(1)
        .with_certificate_request_value(crate::SecretBytes::new(vec![0x30, 0x00]))
        .into_ttlv_payload()
        .expect("the typed builder emits the required Certificate Request Type");
    let request = request_message_with_operation_payload(CERTIFY_OPERATION, payload);

    certify_request_type_profile_result()
        .validate_request(&request)
        .expect("Certificate Request Type accompanies the supplied Certificate Request Value");
}

#[test]
fn profile_rejects_certificate_request_value_without_request_type() {
    let payload = structure([tagged(
        CERTIFICATE_REQUEST_VALUE_TAG,
        Value::byte_string(vec![0x30, 0x00]),
    )]);
    let request = request_message_with_operation_payload(CERTIFY_OPERATION, payload);
    let error = certify_request_type_profile_result()
        .validate_request(&request)
        .expect_err(
            "Table 179 requires Certificate Request Type with the Certificate Request Value",
        );

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_rejects_malformed_certificate_request_type_when_request_value_is_present() {
    let payload = structure([
        tagged(CERTIFICATE_REQUEST_TYPE_TAG, Value::integer(1)),
        tagged(
            CERTIFICATE_REQUEST_VALUE_TAG,
            Value::byte_string(vec![0x30, 0x00]),
        ),
    ]);
    let request = request_message_with_operation_payload(CERTIFY_OPERATION, payload);
    let error = certify_request_type_profile_result()
        .validate_request(&request)
        .expect_err("Table 179 defines Certificate Request Type as an Enumeration");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_REQUEST_TYPE_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_REQUEST_TYPE_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_accepts_valid_certify_omission_without_request_type() {
    const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;

    let payload = crate::CertifyRequest::new()
        .with_unique_identifier(crate::UniqueIdentifier::TextString("public-key".to_owned()))
        .with_attributes(structure([tagged(
            CERTIFICATE_TYPE_TAG,
            Value::enumeration(1),
        )]))
        .into_ttlv_payload()
        .expect("the omitted request path supplies Unique Identifier and Certificate Type");
    let request = request_message_with_operation_payload(CERTIFY_OPERATION, payload);

    certify_request_type_profile_result()
        .validate_request(&request)
        .expect("the conditional rule does not apply when Certificate Request Value is omitted");
}

#[test]
fn profile_maps_omitted_certify_certificate_request_object_as_optional_capability() {
    // KMIP Specification v2.1 §6.1.6 paragraph 2 permits omitting the
    // Certificate Request object. Its conditional SHALL requirements remain
    // mapped separately.
    const CERTIFICATE_REQUEST_TYPE_TAG: u32 = 0x0042_0019;
    const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;
    const ATTRIBUTES_TAG: u32 = 0x0042_0125;

    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == CERTIFY_REQUEST_OMISSION_REQUIREMENT_ID)
        .expect("omitting Certificate Request has a typed optional-capability mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| requirement.requirement_id == CERTIFY_REQUEST_OMISSION_REQUIREMENT_ID)
        .expect("the Certificate Request MAY resolves to generated normative metadata");

    assert_eq!(
        mapping.source_clause_id,
        CERTIFY_REQUEST_OMISSION_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::OptionalCapability
    );
    validate_mapping(CERTIFY_PROFILE_ID, requirement, mapping)
        .expect("the optional capability has positive executable evidence");
    let evaluate = mapping
        .evaluator
        .expect("the optional capability has a typed evaluator");
    assert!(!evaluate(None, None));

    let result = ProfileValidationResult {
        profile_ids: vec![CERTIFY_PROFILE_ID.to_owned()],
        requirement_ids: vec![CERTIFY_REQUEST_OMISSION_REQUIREMENT_ID.to_owned()],
    };
    let payload = crate::CertifyRequest::new()
        .with_unique_identifier(crate::UniqueIdentifier::TextString("public-key".to_owned()))
        .with_attributes(structure([tagged(
            CERTIFICATE_TYPE_TAG,
            Value::enumeration(1),
        )]))
        .into_ttlv_payload()
        .expect("the omitted-object request includes its required identifier and Certificate Type");

    assert_eq!(
        payload
            .view()
            .children()
            .iter()
            .map(|field| field.tag().raw())
            .collect::<Vec<_>>(),
        [UNIQUE_IDENTIFIER_TAG, ATTRIBUTES_TAG]
    );
    let payload_view = payload.view();
    let attributes = &payload_view.children()[1];
    assert!(attributes.with_value(|value| matches!(value, ValueView::Structure(attributes)
        if attributes.children().len() == 1
            && attributes.children()[0].tag().raw() == CERTIFICATE_TYPE_TAG
            && attributes.children()[0].with_value(|value| matches!(value, ValueView::Enumeration(1))))));
    assert!(!payload.view().children().iter().any(|field| {
        matches!(
            field.tag().raw(),
            CERTIFICATE_REQUEST_TYPE_TAG | CERTIFICATE_REQUEST_VALUE_TAG
        )
    }));

    let request = request_message_with_operation_payload(CERTIFY_OPERATION, payload);
    assert!(evaluate(Some(&request), None));
    result
        .validate_request(&request)
        .expect("the Baseline profile accepts the valid omitted-object path");

    let supplied_request_value = [0x30, 0x00];
    let supplied_payload = crate::CertifyRequest::new()
        .with_certificate_request_type(1)
        .with_certificate_request_value(crate::SecretBytes::new(supplied_request_value.to_vec()))
        .into_ttlv_payload()
        .expect("the supplied Certificate Request Value has its required request type");
    assert!(supplied_payload.view().children().iter().any(|field| {
        field.tag().raw() == CERTIFICATE_REQUEST_VALUE_TAG
            && field.with_value(|value| {
                matches!(value, ValueView::ByteString(bytes) if bytes == supplied_request_value)
            })
    }));
    let supplied_value_request =
        request_message_with_operation_payload(CERTIFY_OPERATION, supplied_payload);
    assert!(evaluate(Some(&supplied_value_request), None));
    result
        .validate_request(&supplied_value_request)
        .expect("the optional omission capability accepts a supplied Certificate Request Value");

    let unrelated_operation_request =
        request_message_with_operation_payload(LOCATE_OPERATION_TAG, Structure::new());
    assert!(evaluate(Some(&unrelated_operation_request), None));
    result
        .validate_request(&unrelated_operation_request)
        .expect("the Certify optional capability does not constrain Locate requests");
}

#[test]
fn profile_maps_omitted_certify_request_identifier_as_required_behavior() {
    // KMIP Specification v2.1 §6.1.6 paragraph 2 requires an explicit Unique
    // Identifier when Certificate Request Value is omitted; KMIPKIT-DEC-013
    // resolves the conflicting Table 179 ID Placeholder sentence.
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| mapping.requirement_id == CERTIFY_OMITTED_VALUE_IDENTIFIER_REQUIREMENT_ID)
        .expect("the conditional Certify Unique Identifier SHALL has a typed mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| {
            requirement.requirement_id == CERTIFY_OMITTED_VALUE_IDENTIFIER_REQUIREMENT_ID
        })
        .expect("the conditional Certify SHALL resolves to normative metadata");

    assert_eq!(
        mapping.source_clause_id,
        CERTIFY_OMITTED_VALUE_IDENTIFIER_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(CERTIFY_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.kind == RuleTestKind::PositiveBehavior
            && evidence.test_id.ends_with(
                "profile_accepts_omitted_certify_request_value_with_explicit_unique_identifier",
            )
    }));
    let evaluate = mapping
        .evaluator
        .expect("the required behavior has a typed evaluator");
    assert!(!evaluate(None, None));

    let unrelated_request =
        request_message_with_operation_payload(LOCATE_OPERATION_TAG, Structure::new());
    assert!(evaluate(Some(&unrelated_request), None));
}

#[test]
fn profile_accepts_omitted_certify_request_value_with_explicit_unique_identifier() {
    let request = request_message_with_operation_payload(
        CERTIFY_OPERATION,
        structure([tagged(
            UNIQUE_IDENTIFIER_TAG,
            Value::text_string("public-key".to_owned()),
        )]),
    );

    certify_omitted_value_identifier_profile_result()
        .validate_request(&request)
        .expect("omitted Certificate Request Value is permitted with explicit Unique Identifier");
}

#[test]
fn profile_rejects_omitted_certify_request_value_without_unique_identifier() {
    // A raw Request Message cannot substitute a preceding batch item's
    // ID Placeholder for the public key identifier required by DEC-013.
    let request = request_message_with_operation_payload(CERTIFY_OPERATION, Structure::new());
    let error = certify_omitted_value_identifier_profile_result()
        .validate_request(&request)
        .expect_err("omitting Certificate Request Value requires explicit Unique Identifier");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_OMITTED_VALUE_IDENTIFIER_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_OMITTED_VALUE_IDENTIFIER_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_rejects_id_placeholder_for_omitted_certify_request_value() {
    // DEC-013 requires an explicit public-key identifier when request material
    // is omitted; Enumeration(1) names the batch ID Placeholder, not the key.
    let request = request_message_with_operation_payload(
        CERTIFY_OPERATION,
        structure([tagged(UNIQUE_IDENTIFIER_TAG, Value::enumeration(1))]),
    );

    let error = certify_omitted_value_identifier_profile_result()
        .validate_request(&request)
        .expect_err("the ID Placeholder is not an explicit public-key identifier");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_OMITTED_VALUE_IDENTIFIER_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_OMITTED_VALUE_IDENTIFIER_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_rejects_malformed_certify_identifier_for_omitted_request_value() {
    let request = request_message_with_operation_payload(
        CERTIFY_OPERATION,
        structure([tagged(UNIQUE_IDENTIFIER_TAG, Value::integer(9))]),
    );

    let error = certify_omitted_value_identifier_profile_result()
        .validate_request(&request)
        .expect_err("an Integer is not a direct Unique Identifier");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_OMITTED_VALUE_IDENTIFIER_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_OMITTED_VALUE_IDENTIFIER_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_accepts_certify_request_value_without_unique_identifier() {
    let request = request_message_with_operation_payload(
        CERTIFY_OPERATION,
        structure([tagged(
            CERTIFICATE_REQUEST_VALUE_TAG,
            Value::byte_string(vec![0x30, 0x00]),
        )]),
    );

    certify_omitted_value_identifier_profile_result()
        .validate_request(&request)
        .expect("the Unique Identifier condition does not apply when request material is present");
}

#[test]
fn profile_maps_omitted_certify_certificate_type_as_required_behavior() {
    // KMIP Specification v2.1 §6.1.6 paragraph 2 requires Certificate Type
    // inside Attributes when both Certificate Request fields are omitted;
    // Table 41 defines the attribute's TTLV type as Enumeration.
    let mapping = TYPED_RULE_MAPPINGS
        .iter()
        .find(|mapping| {
            mapping.requirement_id == CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_REQUIREMENT_ID
        })
        .expect("the conditional Certify Certificate Type SHALL has a typed mapping");
    let requirement = CLIENT_REQUIREMENTS
        .iter()
        .find(|requirement| {
            requirement.requirement_id == CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_REQUIREMENT_ID
        })
        .expect("the conditional Certify SHALL resolves to normative metadata");

    assert_eq!(
        mapping.source_clause_id,
        CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_SOURCE_CLAUSE_ID
    );
    assert_eq!(
        mapping.verification_category,
        GeneratedVerificationCategory::RequiredBehavior
    );
    validate_mapping(CERTIFY_PROFILE_ID, requirement, mapping)
        .expect("the required behavior has positive executable evidence");
    assert!(mapping.test_evidence.iter().any(|evidence| {
        evidence.kind == RuleTestKind::PositiveBehavior
            && evidence
                .test_id
                .ends_with("profile_accepts_omitted_certify_certificate_type_in_attributes")
    }));
    let evaluate = mapping
        .evaluator
        .expect("the required behavior has a typed evaluator");
    assert!(!evaluate(None, None));

    let unrelated_request =
        request_message_with_operation_payload(LOCATE_OPERATION_TAG, Structure::new());
    assert!(evaluate(Some(&unrelated_request), None));
}

#[test]
fn profile_accepts_omitted_certify_certificate_type_in_attributes() {
    const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;

    let attributes = Value::structure(structure([tagged(
        CERTIFICATE_TYPE_TAG,
        Value::enumeration(1),
    )]));
    let request = request_message_with_operation_payload(
        CERTIFY_OPERATION,
        certify_payload_with_omitted_request_fields(Some(attributes)),
    );

    certify_omitted_request_certificate_type_profile_result()
        .validate_request(&request)
        .expect("an omitted Certificate Request pair specifies Certificate Type in Attributes");
}

#[test]
fn profile_rejects_omitted_certify_certificate_type_when_attribute_missing() {
    let request = request_message_with_operation_payload(
        CERTIFY_OPERATION,
        certify_payload_with_omitted_request_fields(None),
    );
    let error = certify_omitted_request_certificate_type_profile_result()
        .validate_request(&request)
        .expect_err("omitting both Certificate Request fields requires Certificate Type");

    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        error.requirement_id(),
        Some(CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_REQUIREMENT_ID)
    );
    assert_eq!(
        error.source_clause_id(),
        Some(CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_SOURCE_CLAUSE_ID)
    );
}

#[test]
fn profile_rejects_malformed_certificate_type_attribute_when_request_fields_are_omitted() {
    const CERTIFICATE_TYPE_TAG: u32 = 0x0042_001D;

    let malformed_attributes = [
        Value::structure(structure([tagged(CERTIFICATE_TYPE_TAG, Value::integer(1))])),
        Value::enumeration(1),
    ];
    for attributes in malformed_attributes {
        let request = request_message_with_operation_payload(
            CERTIFY_OPERATION,
            certify_payload_with_omitted_request_fields(Some(attributes)),
        );
        let error = certify_omitted_request_certificate_type_profile_result()
            .validate_request(&request)
            .expect_err("Certificate Type must be an Enumeration within Attributes");

        assert_eq!(
            error.code(),
            ProfileValidationErrorCode::RequestRuleRejected
        );
        assert_eq!(
            error.requirement_id(),
            Some(CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_REQUIREMENT_ID)
        );
        assert_eq!(
            error.source_clause_id(),
            Some(CERTIFY_OMITTED_REQUEST_CERTIFICATE_TYPE_SOURCE_CLAUSE_ID)
        );
    }
}

#[test]
fn profile_accepts_certificate_request_fields_without_certificate_type_attribute() {
    let payload = crate::CertifyRequest::new()
        .with_certificate_request_type(1)
        .with_certificate_request_value(crate::SecretBytes::new(vec![0x30, 0x00]))
        .into_ttlv_payload()
        .expect("a supplied Certificate Request Value includes its required request type");
    let request = request_message_with_operation_payload(CERTIFY_OPERATION, payload);

    certify_omitted_request_certificate_type_profile_result()
        .validate_request(&request)
        .expect("Certificate Type is conditional on both Certificate Request fields being omitted");
}

const ORIGINAL_CREATION_DATE_TAG: u32 = 0x0042_00BC;
const ORIGINAL_CREATION_DATE_ADD_OPERATION: u32 = 0x0000_000D;
const ORIGINAL_CREATION_DATE_NEW_ATTRIBUTE: u32 = 0x0042_013D;
const ORIGINAL_CREATION_DATE_REQUIREMENTS: [&str; 3] = [
    "KMIPKIT-REQ-SPEC-4.38-001-001",
    "KMIPKIT-REQ-SPEC-4.38-001-002",
    "KMIPKIT-REQ-SPEC-4.38-001-003",
];

fn register_original_creation_date_request(date: Option<Value>) -> crate::RequestMessage {
    let mut attributes = vec![
        tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ];
    if let Some(date) = date {
        attributes.push(tagged(ORIGINAL_CREATION_DATE_TAG, date));
    }

    request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
            attributes,
        ),
    )
}

fn add_original_creation_date_request(value: Value) -> crate::RequestMessage {
    request_message_with_operation_payload(
        ORIGINAL_CREATION_DATE_ADD_OPERATION,
        structure([
            tagged(
                TABLE12_UNIQUE_IDENTIFIER,
                Value::text_string("date-target".to_owned()),
            ),
            tagged(
                ORIGINAL_CREATION_DATE_NEW_ATTRIBUTE,
                Value::structure(structure([tagged(ORIGINAL_CREATION_DATE_TAG, value)])),
            ),
        ]),
    )
}

#[test]
// KMIP Specification v2.1 §4.38 Table 105 and §11.56; §6.1.43 Table 293.
fn profile_maps_original_creation_date_register_and_add_capabilities() {
    let evidence_tests = [
        "profile_accepts_register_without_and_with_original_creation_date",
        "profile_accepts_register_without_and_with_original_creation_date",
        "profile_accepts_add_attribute_original_creation_date",
    ];
    for (requirement_id, evidence_test) in ORIGINAL_CREATION_DATE_REQUIREMENTS
        .into_iter()
        .zip(evidence_tests)
    {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id);
        assert!(
            mapping.is_some(),
            "Original Creation Date has a typed mapping"
        );
        if let Some(mapping) = mapping {
            assert_eq!(mapping.source_clause_id, "KMIPKIT-CLAUSE-SPEC-4.38-001");
            assert_eq!(
                mapping.verification_category,
                GeneratedVerificationCategory::OptionalCapability
            );
            assert!(mapping.test_evidence.iter().any(|evidence| {
                evidence.kind == RuleTestKind::OptionalCapabilityUsable
                    && evidence.test_id.ends_with(evidence_test)
            }));
        }
    }
}

#[test]
// KMIP Specification v2.1 §4.38 Table 105 permits omission and client setting on Register.
fn profile_accepts_register_without_and_with_original_creation_date() {
    let omitted = register_original_creation_date_request(None);
    table12_profile_result(&[ORIGINAL_CREATION_DATE_REQUIREMENTS[0]])
        .validate_request(&omitted)
        .expect("Register may omit Original Creation Date");

    let supplied = register_original_creation_date_request(Some(Value::date_time(1_700_000_000)));
    table12_profile_result(&[ORIGINAL_CREATION_DATE_REQUIREMENTS[1]])
        .validate_request(&supplied)
        .expect("Register may set Original Creation Date");
}

#[test]
// KMIP Specification v2.1 §4.38 Table 105 permits later addition through §6.1.2 Table 167.
fn profile_accepts_add_attribute_original_creation_date() {
    let request = add_original_creation_date_request(Value::date_time(1_700_000_000));
    table12_profile_result(&[ORIGINAL_CREATION_DATE_REQUIREMENTS[2]])
        .validate_request(&request)
        .expect("Add Attribute can carry a previously omitted Original Creation Date");
}

#[test]
// KMIP Specification v2.1 §4.38 Table 105 encodes Original Creation Date as Date-Time.
fn profile_rejects_malformed_original_creation_date() {
    let register = register_original_creation_date_request(Some(Value::integer(7)));
    let register_error = table12_profile_result(&[ORIGINAL_CREATION_DATE_REQUIREMENTS[1]])
        .validate_request(&register)
        .expect_err("Register Original Creation Date is Date-Time");
    assert_eq!(
        register_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        register_error.requirement_id(),
        Some(ORIGINAL_CREATION_DATE_REQUIREMENTS[1])
    );

    let add_attribute = add_original_creation_date_request(Value::integer(7));
    let add_attribute_error = table12_profile_result(&[ORIGINAL_CREATION_DATE_REQUIREMENTS[2]])
        .validate_request(&add_attribute)
        .expect_err("Add Attribute Original Creation Date is Date-Time");
    assert_eq!(
        add_attribute_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
    assert_eq!(
        add_attribute_error.requirement_id(),
        Some(ORIGINAL_CREATION_DATE_REQUIREMENTS[2])
    );
}

const RANDOM_NUMBER_GENERATOR_TAG: u32 = 0x0042_00DE;
const RNG_PARAMETERS_TAG: u32 = 0x0042_00D9;
const RNG_ALGORITHM_TAG: u32 = 0x0042_00DA;
const RANDOM_NUMBER_GENERATOR_ADD_OPERATION: u32 = 0x0000_000D;
const RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE: u32 = 0x0042_013D;
const RANDOM_NUMBER_GENERATOR_REQUIREMENTS: [&str; 2] = [
    "KMIPKIT-REQ-SPEC-4.46-001-001",
    "KMIPKIT-REQ-SPEC-4.46-001-002",
];

fn random_number_generator_structure() -> Value {
    Value::structure(structure([tagged(
        RNG_ALGORITHM_TAG,
        Value::enumeration(1),
    )]))
}

fn register_random_number_generator_request(value: Option<Value>) -> crate::RequestMessage {
    let mut attributes = vec![
        tagged(TABLE12_CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        tagged(TABLE12_CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ];
    if let Some(value) = value {
        attributes.push(tagged(RANDOM_NUMBER_GENERATOR_TAG, value));
    }
    request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        table12_request_payload(
            TABLE12_REGISTER_OPERATION,
            table12_key_block_object(TABLE12_SYMMETRIC_KEY, false, false, None),
            attributes,
        ),
    )
}

fn add_attribute_random_number_generator_request(value: Value) -> crate::RequestMessage {
    request_message_with_operation_payload(
        RANDOM_NUMBER_GENERATOR_ADD_OPERATION,
        structure([
            tagged(
                TABLE12_UNIQUE_IDENTIFIER,
                Value::text_string("rng-target".to_owned()),
            ),
            tagged(
                RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE,
                Value::structure(structure([tagged(RANDOM_NUMBER_GENERATOR_TAG, value)])),
            ),
        ]),
    )
}

#[test]
// KMIP Specification v2.1 §4.46 Table 121 and §7.36 Table 388; §6.1.43 Table 293 and §6.1.2 Table 167.
fn profile_maps_register_and_add_attribute_random_number_generator_capabilities() {
    let evidence_tests = [
        "profile_accepts_register_random_number_generator",
        "profile_accepts_add_attribute_random_number_generator",
    ];
    for (requirement_id, evidence_test) in RANDOM_NUMBER_GENERATOR_REQUIREMENTS
        .into_iter()
        .zip(evidence_tests)
    {
        let mapping = TYPED_RULE_MAPPINGS
            .iter()
            .find(|mapping| mapping.requirement_id == requirement_id);
        assert!(
            mapping.is_some(),
            "Random Number Generator has a typed mapping"
        );
        if let Some(mapping) = mapping {
            assert_eq!(mapping.source_clause_id, "KMIPKIT-CLAUSE-SPEC-4.46-001");
            assert_eq!(
                mapping.verification_category,
                GeneratedVerificationCategory::OptionalCapability
            );
            assert!(mapping.test_evidence.iter().any(|evidence| {
                evidence.kind == RuleTestKind::OptionalCapabilityUsable
                    && evidence.test_id.ends_with(evidence_test)
            }));
        }
    }
}

#[test]
// KMIP Specification v2.1 §4.46 allows setting Random Number Generator on Register.
fn profile_accepts_register_random_number_generator() {
    let omitted = register_random_number_generator_request(None);
    table12_profile_result(&[RANDOM_NUMBER_GENERATOR_REQUIREMENTS[0]])
        .validate_request(&omitted)
        .expect("Register may omit Random Number Generator");

    let supplied =
        register_random_number_generator_request(Some(random_number_generator_structure()));
    table12_profile_result(&[RANDOM_NUMBER_GENERATOR_REQUIREMENTS[0]])
        .validate_request(&supplied)
        .expect("Register may include Random Number Generator");
}

#[test]
// KMIP Specification v2.1 §4.46 allows adding Random Number Generator later through Add Attribute.
fn profile_accepts_add_attribute_random_number_generator() {
    let request =
        add_attribute_random_number_generator_request(random_number_generator_structure());
    table12_profile_result(&[RANDOM_NUMBER_GENERATOR_REQUIREMENTS[1]])
        .validate_request(&request)
        .expect("Add Attribute may carry Random Number Generator");
}

#[test]
// KMIP Specification v2.1 §4.46 Table 121 and §7.36 Table 388 require RNG Parameters structure with RNG Algorithm.
fn profile_rejects_malformed_random_number_generator() {
    let register = register_random_number_generator_request(Some(Value::integer(7)));
    let register_error = table12_profile_result(&[RANDOM_NUMBER_GENERATOR_REQUIREMENTS[0]])
        .validate_request(&register)
        .expect_err("Random Number Generator is an RNG Parameters structure");
    assert_eq!(
        register_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let missing_algorithm =
        add_attribute_random_number_generator_request(Value::structure(structure([])));
    let missing_algorithm_error =
        table12_profile_result(&[RANDOM_NUMBER_GENERATOR_REQUIREMENTS[1]])
            .validate_request(&missing_algorithm)
            .expect_err("RNG Parameters has a mandatory RNG Algorithm");
    assert_eq!(
        missing_algorithm_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );

    let nested_parameters =
        add_attribute_random_number_generator_request(Value::structure(structure([
            tagged(RNG_ALGORITHM_TAG, Value::enumeration(1)),
            tagged(
                RNG_PARAMETERS_TAG,
                Value::structure(structure([tagged(
                    RNG_ALGORITHM_TAG,
                    Value::enumeration(1),
                )])),
            ),
        ])));
    let nested_parameters_error =
        table12_profile_result(&[RANDOM_NUMBER_GENERATOR_REQUIREMENTS[1]])
            .validate_request(&nested_parameters)
            .expect_err("RNG Parameters members are encoded directly on the attribute");
    assert_eq!(
        nested_parameters_error.code(),
        ProfileValidationErrorCode::RequestRuleRejected
    );
}

#[test]
// KMIP Specification v2.1 §§4.46, 6.1.2, 6.1.43, and 7.36: RNG profile rules are request-scoped and operation-specific.
fn rng_profile_evaluators_handle_absent_requests_and_unrelated_operations() {
    assert!(!client_register_random_number_generator_is_well_formed(
        None, None
    ));
    assert!(!client_add_attribute_random_number_generator_is_well_formed(None, None));

    let unrelated = request_message_with_operation_payload(TABLE12_GET_OPERATION, structure([]));
    assert!(client_register_random_number_generator_is_well_formed(
        Some(&unrelated),
        None
    ));
    assert!(client_add_attribute_random_number_generator_is_well_formed(
        Some(&unrelated),
        None
    ));

    let register_without_attributes =
        request_message_with_operation_payload(TABLE12_REGISTER_OPERATION, structure([]));
    assert!(client_register_random_number_generator_is_well_formed(
        Some(&register_without_attributes),
        None
    ));

    let add_without_new_attribute = request_message_with_operation_payload(
        RANDOM_NUMBER_GENERATOR_ADD_OPERATION,
        structure([]),
    );
    assert!(client_add_attribute_random_number_generator_is_well_formed(
        Some(&add_without_new_attribute),
        None
    ));
}

#[test]
// KMIP Specification v2.1 §§4.46, 6.1.43, and 7.36: malformed Register RNG fields fail closed.
fn register_rng_evaluator_rejects_duplicate_and_malformed_fields() {
    let duplicate_register_attributes = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([
            tagged(TABLE12_ATTRIBUTES, Value::structure(structure([]))),
            tagged(TABLE12_ATTRIBUTES, Value::structure(structure([]))),
        ]),
    );
    let malformed_register_attributes = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([tagged(TABLE12_ATTRIBUTES, Value::integer(7))]),
    );
    let duplicate_register_rng = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure([
                tagged(
                    RANDOM_NUMBER_GENERATOR_TAG,
                    random_number_generator_structure(),
                ),
                tagged(
                    RANDOM_NUMBER_GENERATOR_TAG,
                    random_number_generator_structure(),
                ),
            ])),
        )]),
    );
    let malformed_register_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure([tagged(
                RANDOM_NUMBER_GENERATOR_TAG,
                Value::structure(structure([tagged(
                    RNG_ALGORITHM_TAG,
                    Value::text_string("not-an-enumeration".to_owned()),
                )])),
            )])),
        )]),
    );
    let duplicate_register_algorithm = request_message_with_operation_payload(
        TABLE12_REGISTER_OPERATION,
        structure([tagged(
            TABLE12_ATTRIBUTES,
            Value::structure(structure([tagged(
                RANDOM_NUMBER_GENERATOR_TAG,
                Value::structure(structure([
                    tagged(RNG_ALGORITHM_TAG, Value::enumeration(1)),
                    tagged(RNG_ALGORITHM_TAG, Value::enumeration(2)),
                ])),
            )])),
        )]),
    );

    for request in [
        duplicate_register_attributes,
        malformed_register_attributes,
        duplicate_register_rng,
        malformed_register_algorithm,
        duplicate_register_algorithm,
    ] {
        assert!(!client_register_random_number_generator_is_well_formed(
            Some(&request),
            None
        ));
    }
}

#[test]
// KMIP Specification v2.1 §§4.46, 6.1.2, and 7.36: malformed Add Attribute RNG fields fail closed.
fn add_attribute_rng_evaluator_rejects_duplicate_and_malformed_fields() {
    let duplicate_new_attribute = request_message_with_operation_payload(
        RANDOM_NUMBER_GENERATOR_ADD_OPERATION,
        structure([
            tagged(
                RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE,
                Value::structure(structure([])),
            ),
            tagged(
                RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE,
                Value::structure(structure([])),
            ),
        ]),
    );
    let malformed_new_attribute = request_message_with_operation_payload(
        RANDOM_NUMBER_GENERATOR_ADD_OPERATION,
        structure([tagged(
            RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE,
            Value::text_string("not-a-structure".to_owned()),
        )]),
    );
    let duplicate_add_rng = request_message_with_operation_payload(
        RANDOM_NUMBER_GENERATOR_ADD_OPERATION,
        structure([tagged(
            RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE,
            Value::structure(structure([
                tagged(
                    RANDOM_NUMBER_GENERATOR_TAG,
                    random_number_generator_structure(),
                ),
                tagged(
                    RANDOM_NUMBER_GENERATOR_TAG,
                    random_number_generator_structure(),
                ),
            ])),
        )]),
    );
    let malformed_add_algorithm = request_message_with_operation_payload(
        RANDOM_NUMBER_GENERATOR_ADD_OPERATION,
        structure([tagged(
            RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE,
            Value::structure(structure([tagged(
                RANDOM_NUMBER_GENERATOR_TAG,
                Value::structure(structure([tagged(
                    RNG_ALGORITHM_TAG,
                    Value::text_string("not-an-enumeration".to_owned()),
                )])),
            )])),
        )]),
    );
    let duplicate_add_algorithm = request_message_with_operation_payload(
        RANDOM_NUMBER_GENERATOR_ADD_OPERATION,
        structure([tagged(
            RANDOM_NUMBER_GENERATOR_NEW_ATTRIBUTE,
            Value::structure(structure([tagged(
                RANDOM_NUMBER_GENERATOR_TAG,
                Value::structure(structure([
                    tagged(RNG_ALGORITHM_TAG, Value::enumeration(1)),
                    tagged(RNG_ALGORITHM_TAG, Value::enumeration(2)),
                ])),
            )])),
        )]),
    );

    for request in [
        duplicate_new_attribute,
        malformed_new_attribute,
        duplicate_add_rng,
        malformed_add_algorithm,
        duplicate_add_algorithm,
    ] {
        assert!(!client_add_attribute_random_number_generator_is_well_formed(Some(&request), None));
    }
}
