use std::fmt;
use std::fmt::Write as _;

use super::{
    CLIENT_REQUIREMENTS, ProfileEvidenceRecord, ProfileReadiness, ProfileReadinessBlocker,
    ProfileTransport, ProfileValidationError, ProfileValidationErrorCode, ProfileValidationResult,
    ProfileValidationSelection, RuleConstraint, RuleEvaluator, RuleTestEvidence, RuleTestKind,
    TYPED_RULE_MAPPINGS, TypedRuleMapping,
    client_create_key_pair_domain_parameters_have_valid_wire_types,
    client_get_or_export_key_compression_type_is_enumeration,
    client_opaque_object_context_custom_attributes,
    client_register_does_not_specify_key_value_present,
    client_register_key_value_location_requires_omitted_key_value,
    discover_versions_advertises_only_client_supported_versions,
    import_preserves_pgp_key_versions_and_opaque_bytes, import_preserves_wrapped_secret_data_shape,
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
const ATTESTATION_TYPE_TAG: u32 = 0x0042_00C7;
const BATCH_ERROR_CONTINUATION_OPTION_TAG: u32 = 0x0042_000E;
const OPERATION_TAG: u32 = 0x0042_005C;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;
const LOCATE_OPERATION_TAG: u32 = 0x0000_0008;
const RESPONSE_PAYLOAD_TAG: u32 = 0x0042_007C;
const RESULT_STATUS_TAG: u32 = 0x0042_007F;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const CLIENT_CORRELATION_VALUE_TAG: u32 = 0x0042_0105;
const UNIQUE_BATCH_ITEM_ID_TAG: u32 = 0x0042_0093;
const GET_ATTRIBUTES_OPERATION: u32 = 0x0000_000B;
const ATTRIBUTES_TAG: u32 = 0x0042_0125;
const KEY_VALUE_PRESENT_TAG: u32 = 0x0042_00BB;
const UNIQUE_IDENTIFIER_TAG: u32 = 0x0042_0094;
const ISSUER_ALTERNATIVE_NAME_TAG: u32 = 0x0042_00B1;
const SUBJECT_ALTERNATIVE_NAME_TAG: u32 = 0x0042_00B3;
const SUBJECT_DISTINGUISHED_NAME_TAG: u32 = 0x0042_00B4;
const DERIVE_KEY_ID_PLACEHOLDER_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-6.1.14-006-002";
const DERIVE_KEY_ID_PLACEHOLDER_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-6.1.14-006";
const DERIVE_KEY_ID_PLACEHOLDER_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
const VARIABLE_IV_REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-4.16-002";
const VARIABLE_IV_SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-4.16-002";
const REGISTER_OPERATION_TAG: u32 = 0x0000_0003;
const DERIVE_KEY_OPERATION: u32 = 0x0000_0005;
const OBJECT_TYPE_TAG: u32 = 0x0042_0057;
const OPAQUE_OBJECT_TAG: u32 = 0x0042_005B;
const OPAQUE_DATA_TYPE_TAG: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE_TAG: u32 = 0x0042_005A;
const VENDOR_ATTRIBUTE_TAG: u32 = 0x0042_0008;
const VENDOR_IDENTIFICATION_TAG: u32 = 0x0042_009D;
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

fn request_message_with_batch_operations(
    operations: impl IntoIterator<Item = (u32, Structure)>,
) -> crate::RequestMessage {
    let operations: Vec<_> = operations.into_iter().collect();
    let batch_count = i32::try_from(operations.len()).expect("test batch count fits in an Integer");
    let mut root_items = vec![tagged(
        REQUEST_HEADER_TAG,
        Value::structure(structure([
            test_protocol_version(),
            tagged(BATCH_COUNT_TAG, Value::integer(batch_count)),
        ])),
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
    let explicit_identifiers = request_message_with_operation_payload(
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
        .validate_request(&explicit_identifiers)
        .expect("repeated explicit source identifiers remain valid");

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
fn profile_maps_optional_maximum_response_size_capability() {
    // KMIP Specification v2.1 §9.12 defines Maximum Response Size as optional.
    const REQUIREMENT_ID: &str = "KMIPKIT-REQ-SPEC-9.12-001-001";
    const SOURCE_CLAUSE_ID: &str = "KMIPKIT-CLAUSE-SPEC-9.12-001";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";
    const MAXIMUM_RESPONSE_SIZE_TAG: u32 = 0x0042_0050;

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

    let supplied = crate::RequestMessage::try_from_ttlv(structure([
        tagged(
            REQUEST_HEADER_TAG,
            Value::structure(structure([
                test_protocol_version(),
                tagged(MAXIMUM_RESPONSE_SIZE_TAG, Value::integer(4096)),
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
    .expect("the optional Maximum Response Size request is valid");
    assert_eq!(supplied.header().maximum_response_size(), Some(4096));
    assert!(evaluate(Some(&supplied), None));
    result
        .validate_request(&supplied)
        .expect("a client may include Maximum Response Size");
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
    // its Key Value is caller-owned opaque ASCII-armored data.
    const REQUIREMENT_OBJECT_FORM: &str = "KMIPKIT-REQ-SPEC-2.4-001-001";
    const REQUIREMENT_VERSION: &str = "KMIPKIT-REQ-SPEC-2.4-001-002";
    const BASELINE_PROFILE_ID: &str = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    let requirement_ids = [REQUIREMENT_OBJECT_FORM, REQUIREMENT_VERSION];
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
    assert!(!import_preserves_pgp_key_versions_and_opaque_bytes(
        Some(&malformed_key_value),
        None
    ));
    assert!(!import_preserves_pgp_key_versions_and_opaque_bytes(
        None, None
    ));
    assert!(import_preserves_pgp_key_versions_and_opaque_bytes(
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
        assert!(!import_preserves_pgp_key_versions_and_opaque_bytes(
            Some(&malformed),
            None
        ));
    }
    assert!(import_preserves_pgp_key_versions_and_opaque_bytes(
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
    const DIGITAL_SIGNATURE_ALGORITHM_TAG: u32 = 0x0042_00AE;

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

    let encoding_only = TypedRuleMapping {
        evaluator: None,
        ..mapping(
            "TEST-REQ",
            GeneratedVerificationCategory::RequiredBehavior,
            RuleTestKind::PositiveBehavior,
            &CONSTRAINTS_TTLV,
        )
    };
    let error = validate_mapping("TEST-PROFILE", &requirement, &encoding_only)
        .expect_err("an encoding-only rule is not a transport-only mapping");
    assert_eq!(
        error.code(),
        ProfileValidationErrorCode::IncompleteRuleMapping
    );

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

#[test]
fn typed_rule_evaluators_fail_closed_without_a_request() {
    // KMIP Specification v2.1 §§2.7, 4.14, and 4.27; the rules must fail closed
    // when no request is available to evaluate.
    assert!(!import_preserves_wrapped_secret_data_shape(None, None));
    assert!(!client_create_key_pair_domain_parameters_have_valid_wire_types(None, None));
    assert!(!client_register_key_value_location_requires_omitted_key_value(None, None));
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
