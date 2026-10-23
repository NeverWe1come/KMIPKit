#![cfg(test)]

//! Fake-transport Register contracts from OASIS KMIP v2.1 §6.1.43,
//! Tables 291–294. Traceability: all `KMIPKIT-REQ-SPEC-6.1.43-*` rows.
//! These are derived execution tests, not official fixture passes.

use kmipkit_protocol::{
    AttributeSet, CheckRequest, ObjectType, OpaqueObjectContext, RegisterRequest,
};
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::codec::{CodecLimits, decode};
use kmipkit_ttlv::{Item, StructureView, Value, ValueView};

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_item, test_structure};
use crate::{
    ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientErrorCategory, ClientOperation,
    ClientRequest,
};

const REGISTER_OPERATION: u32 = 0x0000_0003;
const CHECK_OPERATION: u32 = 0x0000_0009;
const BATCH_ITEM: u32 = 0x0042_000F;
const OPERATION: u32 = 0x0042_005C;
const REQUEST_PAYLOAD: u32 = 0x0042_0079;
const OBJECT_TYPE: u32 = 0x0042_0057;
const ATTRIBUTES: u32 = 0x0042_0125;
const LINK_ATTRIBUTE: u32 = 0x0042_004A;
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const SYMMETRIC_KEY: u32 = 0x0042_008F;
const KEY_BLOCK: u32 = 0x0042_0040;
const KEY_FORMAT_TYPE: u32 = 0x0042_0042;
const KEY_VALUE: u32 = 0x0042_0045;
const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
const WRAPPING_METHOD: u32 = 0x0042_009E;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const CRYPTOGRAPHIC_LENGTH: u32 = 0x0042_002A;
const CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0042_002C;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const PROTECTION_STORAGE_MASK: u32 = 0x0042_0160;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const OPAQUE_OBJECT_TYPE: u32 = 8;
const OPAQUE_OBJECT: u32 = 0x0042_005B;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE: u32 = 0x0042_005A;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const INVALID_OBJECT_TYPE: u32 = 0x0000_0038;
const PENDING_CORRELATION: &[u8] = b"REGISTER_PENDING_CORRELATION\xFF";
const ENCODING_OPTION: u32 = 0x0042_00A3;
const TTLV_ENCODING_OPTION: u32 = 2;
// Deterministic AES-128-WRAP test fixture. OpenSSL 3.5.5 unwraps this byte string
// to SYNTHETIC_PREWRAP_KEY_VALUE_TTLV using the published RFC 3394 Example 1 KEK.
// The fixture is test-only and is not cryptographic validation by KMIPKit.
const REGISTER_WRAPPED_KEY_VALUE: &[u8] =
    include_bytes!("../fixtures/register-wrapped-key-value.bin");
const SYNTHETIC_PREWRAP_KEY_VALUE_TTLV: &[u8] = &[
    0x42, 0x00, 0x45, 0x01, 0x00, 0x00, 0x00, 0x38, // Key Value Structure, 56-byte body.
    0x42, 0x00, 0x44, 0x08, 0x00, 0x00, 0x00, 0x04, // Key Material Byte String.
    0x01, 0x02, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00, 0x42, 0x01, 0x25, 0x01, 0x00, 0x00, 0x00,
    0x20, // Key Value Attributes Structure.
    0x42, 0x00, 0x28, 0x05, 0x00, 0x00, 0x00, 0x04, // Cryptographic Algorithm Enumeration.
    0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x42, 0x00, 0x2a, 0x02, 0x00, 0x00, 0x00,
    0x04, // Cryptographic Length Integer.
    0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// Byte-for-byte extraction from the pinned OASIS KMIP 2.1 fixture
// TC-PKCS12-1-21.xml, RequestMessage step 2. This local input does not execute
// the official test case or claim a server-side conformance result.
const OASIS_PKCS12_STEP_2_CERTIFICATE_DER: &[u8] =
    include_bytes!("../fixtures/tc-pkcs12-step2-certificate.der");

fn register_request() -> RegisterRequest {
    RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([
            test_item(CERTIFICATE_LENGTH, Value::integer(512)),
            test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(4)),
            test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048)),
        ])
        .expect("Table 293 certificate attributes are correctly typed"),
        test_item(
            CERTIFICATE,
            Value::structure(test_structure([test_item(
                UNKNOWN_VENDOR_TAG,
                Value::byte_string(b"certificate-value".to_vec()),
            )])),
        ),
    )
}

fn register_request_with_one_outer_pair_attribute(
    algorithm: bool,
    length: bool,
) -> RegisterRequest {
    let mut attributes = vec![test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(3))];
    if algorithm {
        attributes.push(test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)));
    }
    if length {
        attributes.push(test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)));
    }

    RegisterRequest::new(
        ObjectType::from_raw(2),
        AttributeSet::try_new(attributes).expect("Register attributes are structurally valid"),
        test_item(
            SYMMETRIC_KEY,
            Value::structure(test_structure([test_item(
                KEY_BLOCK,
                Value::structure(test_structure([
                    test_item(KEY_FORMAT_TYPE, Value::enumeration(1)),
                    test_item(
                        KEY_VALUE,
                        Value::byte_string(b"opaque wrapped key value".to_vec()),
                    ),
                    test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                    test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
                ])),
            )])),
        ),
    )
}

fn register_request_with_protection_storage_masks() -> RegisterRequest {
    register_request().with_protection_storage_masks(test_structure([test_item(
        PROTECTION_STORAGE_MASK,
        Value::integer(1),
    )]))
}

#[test]
fn register_rejects_one_sided_outer_algorithm_length_before_transport() {
    // KMIP Specification v2.1 §6.1.43 Table 293 requires the paired direct
    // Register Attribute even when the Key Block contains a complete pair.
    for (algorithm, length) in [(true, false), (false, true)] {
        let (mut client, fake, captured) = client_for(ExchangeScript::Success {
            response: Vec::new(),
            request_write_chunks: Vec::new(),
        });

        let error = client
            .register(
                register_request_with_one_outer_pair_attribute(algorithm, length),
                &CodecLimits::defaults(),
            )
            .expect_err("a one-sided outer Register pair must fail local validation");

        assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
        assert!(
            captured.borrow().is_none(),
            "no request bytes reach transport"
        );
        assert_eq!(fake.borrow().exchange_count(), 0);
        assert_eq!(fake.borrow().write_call_count(), 0);
    }
}

fn register_pair_attribute(raw_tag: u32) -> Item {
    match raw_tag {
        CRYPTOGRAPHIC_ALGORITHM => test_item(raw_tag, Value::enumeration(3)),
        CRYPTOGRAPHIC_LENGTH => test_item(raw_tag, Value::integer(256)),
        _ => panic!("only the Table 293 pair attributes are used here"),
    }
}

fn assert_register_request_is_not_sent(request: RegisterRequest) {
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .register(request, &CodecLimits::defaults())
        .expect_err("an invalid Table 293 Register pair must fail local validation");

    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(
        captured.borrow().is_none(),
        "no request bytes reach transport"
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

#[test]
fn register_certificate_rejects_one_sided_outer_algorithm_length_before_transport() {
    // KMIP Specification v2.1 §§2.1 and 6.1.43 Table 293: a Certificate is
    // a Managed Cryptographic Object and therefore uses the outer pair rule.
    for raw_tag in [CRYPTOGRAPHIC_ALGORITHM, CRYPTOGRAPHIC_LENGTH] {
        let request = RegisterRequest::new(
            ObjectType::from_raw(1),
            AttributeSet::try_new([
                test_item(CERTIFICATE_LENGTH, Value::integer(512)),
                register_pair_attribute(raw_tag),
            ])
            .expect("Certificate Register attributes are structurally valid"),
            test_item(CERTIFICATE, Value::structure(test_structure([]))),
        );
        assert_register_request_is_not_sent(request);
    }
}

#[test]
fn register_certificate_request_rejects_one_sided_outer_algorithm_length_before_transport() {
    // KMIP Specification v2.1 §§2.2 and 6.1.43 Table 293: a Certificate
    // Request is a Managed Cryptographic Object and uses the same outer pair.
    for raw_tag in [CRYPTOGRAPHIC_ALGORITHM, CRYPTOGRAPHIC_LENGTH] {
        let request = RegisterRequest::new(
            ObjectType::from_raw(10),
            AttributeSet::try_new([register_pair_attribute(raw_tag)])
                .expect("Certificate Request Register attributes are structurally valid"),
            test_item(CERTIFICATE_REQUEST, Value::structure(test_structure([]))),
        );
        assert_register_request_is_not_sent(request);
    }
}

#[test]
fn register_certificate_rejects_omitted_outer_algorithm_length_without_key_block_before_transport()
{
    // KMIP Specification v2.1 §2.1 Table 2 defines Certificate without a Key
    // Block; §6.1.43 Table 293 therefore requires both outer attributes.
    let request = RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([test_item(CERTIFICATE_LENGTH, Value::integer(512))])
            .expect("Certificate Length is structurally valid"),
        test_item(CERTIFICATE, Value::structure(test_structure([]))),
    );
    assert_register_request_is_not_sent(request);
}

#[test]
fn register_certificate_request_rejects_omitted_outer_algorithm_length_without_key_block_before_transport()
 {
    // KMIP Specification v2.1 §2.2 Table 3 defines Certificate Request
    // without a Key Block; §6.1.43 Table 293 therefore requires both outer attributes.
    let request = RegisterRequest::new(
        ObjectType::from_raw(10),
        AttributeSet::try_new([]).expect("an empty attribute set is structurally valid"),
        test_item(CERTIFICATE_REQUEST, Value::structure(test_structure([]))),
    );
    assert_register_request_is_not_sent(request);
}

fn managed_opaque_register_request(attributes: AttributeSet) -> RegisterRequest {
    RegisterRequest::new(
        ObjectType::from_raw(OPAQUE_OBJECT_TYPE),
        attributes,
        test_item(
            OPAQUE_OBJECT,
            Value::structure(test_structure([
                test_item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001)),
                test_item(
                    OPAQUE_DATA_VALUE,
                    Value::byte_string(b"opaque-register-data".to_vec()),
                ),
            ])),
        ),
    )
    .with_opaque_object_context(OpaqueObjectContext::ManagedCryptographicObject)
}

fn request_fields(bytes: &[u8], item_index: usize) -> (Option<u32>, Vec<u32>, Vec<Vec<u32>>) {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(item_index)
            .expect("request contains the selected Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let operation = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == OPERATION)
                .and_then(|field| {
                    field.with_value(|value| match value {
                        ValueView::Enumeration(raw) => Some(*raw),
                        _ => None,
                    })
                });
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("request contains a Request Payload");
            let (tags, nested) = payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let tags = payload
                    .children()
                    .iter()
                    .map(|field| field.tag().raw())
                    .collect();
                let nested = payload
                    .children()
                    .iter()
                    .filter_map(|field| {
                        field.with_value(|value| match value {
                            ValueView::Structure(structure) => Some(
                                structure
                                    .children()
                                    .iter()
                                    .map(|child| child.tag().raw())
                                    .collect(),
                            ),
                            _ => None,
                        })
                    })
                    .collect();
                (tags, nested)
            });
            (operation, tags, nested)
        })
    })
}

#[derive(Debug, Eq, PartialEq)]
enum RegisterAttributeValue {
    Integer(i32),
    Enumeration(u32),
    Other,
}

fn register_attribute_values(
    bytes: &[u8],
    item_index: usize,
) -> Vec<(u32, RegisterAttributeValue)> {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("KMIP request message is a Structure");
        };
        let batch = message
            .children()
            .iter()
            .filter(|field| field.tag().raw() == BATCH_ITEM)
            .nth(item_index)
            .expect("request contains the selected Batch Item");
        batch.with_value(|value| {
            let ValueView::Structure(batch) = value else {
                panic!("Batch Item is a Structure");
            };
            let payload = batch
                .children()
                .iter()
                .find(|field| field.tag().raw() == REQUEST_PAYLOAD)
                .expect("request contains a Request Payload");
            payload.with_value(|value| {
                let ValueView::Structure(payload) = value else {
                    panic!("Request Payload is a Structure");
                };
                let attributes = payload
                    .children()
                    .iter()
                    .find(|field| field.tag().raw() == ATTRIBUTES)
                    .expect("Register Request Payload contains Attributes");
                attributes.with_value(|value| {
                    let ValueView::Structure(attributes) = value else {
                        panic!("Attributes is a Structure");
                    };
                    attributes
                        .children()
                        .iter()
                        .map(|field| {
                            let value = field.with_value(|value| match value {
                                ValueView::Integer(value) => {
                                    RegisterAttributeValue::Integer(*value)
                                }
                                ValueView::Enumeration(value) => {
                                    RegisterAttributeValue::Enumeration(*value)
                                }
                                _ => RegisterAttributeValue::Other,
                            });
                            (field.tag().raw(), value)
                        })
                        .collect()
                })
            })
        })
    })
}

struct RegisterWrappingEvidence {
    direct_attribute_fields: Vec<u32>,
    direct_algorithm: u32,
    direct_length: i32,
    key_block_fields: Vec<u32>,
    key_value_bytes: Vec<u8>,
    wrapping_method: u32,
    encoding_option: u32,
    encryption_key_identifier: String,
}

fn register_wrapping_evidence(bytes: &[u8]) -> RegisterWrappingEvidence {
    let message = decode(bytes).expect("captured Register request has valid TTLV framing");
    message.with_value(|value| {
        let ValueView::Structure(message) = value else {
            panic!("request message is a Structure");
        };
        let batch_item = message
            .children()
            .iter()
            .find(|field| field.tag().raw() == BATCH_ITEM)
            .expect("Register request has a Batch Item");
        with_structure(batch_item, "Register Batch Item", |batch_item| {
            let payload = required_field(batch_item, REQUEST_PAYLOAD, "Register Request Payload");
            with_structure(payload, "Register Request Payload", |payload| {
                let (direct_attribute_fields, direct_algorithm, direct_length) = with_structure(
                    required_field(payload, ATTRIBUTES, "Register Attributes"),
                    "Register Attributes",
                    |attributes| {
                        let fields = attributes
                            .children()
                            .iter()
                            .map(|field| field.tag().raw())
                            .collect();
                        let algorithm = enumeration_field(
                            attributes,
                            CRYPTOGRAPHIC_ALGORITHM,
                            "Register Attributes Cryptographic Algorithm",
                        );
                        let length = integer_field(
                            attributes,
                            CRYPTOGRAPHIC_LENGTH,
                            "Register Attributes Cryptographic Length",
                        );
                        (fields, algorithm, length)
                    },
                );
                let object = required_field(payload, SYMMETRIC_KEY, "Register Symmetric Key");
                with_structure(object, "Register Symmetric Key", |object| {
                    let key_block = required_field(object, KEY_BLOCK, "Register Key Block");
                    with_structure(key_block, "Register Key Block", |key_block| {
                        let key_block_fields = key_block
                            .children()
                            .iter()
                            .map(|field| field.tag().raw())
                            .collect();
                        let key_value_bytes =
                            byte_string_field(key_block, KEY_VALUE, "wrapped Register Key Value");
                        let wrapping = required_field(
                            key_block,
                            KEY_WRAPPING_DATA,
                            "Register Key Wrapping Data",
                        );
                        with_structure(wrapping, "Key Wrapping Data", |wrapping| {
                            let wrapping_method = enumeration_field(
                                wrapping,
                                WRAPPING_METHOD,
                                "Key Wrapping Data Wrapping Method",
                            );
                            let encoding_option = enumeration_field(
                                wrapping,
                                ENCODING_OPTION,
                                "Key Wrapping Data Encoding Option",
                            );
                            let key_information = required_field(
                                wrapping,
                                ENCRYPTION_KEY_INFORMATION,
                                "Key Wrapping Data Encryption Key Information",
                            );
                            with_structure(
                                key_information,
                                "Encryption Key Information",
                                |key_information| {
                                    let identifier = required_field(
                                        key_information,
                                        UNIQUE_IDENTIFIER,
                                        "Encryption Key Information Unique Identifier",
                                    );
                                    let encryption_key_identifier =
                                        identifier.with_value(|value| match value {
                                            ValueView::TextString(value) => value.to_owned(),
                                            _ => panic!("wrapping key identifier is a Text String"),
                                        });
                                    RegisterWrappingEvidence {
                                        direct_attribute_fields,
                                        direct_algorithm,
                                        direct_length,
                                        key_block_fields,
                                        key_value_bytes,
                                        wrapping_method,
                                        encoding_option,
                                        encryption_key_identifier,
                                    }
                                },
                            )
                        })
                    })
                })
            })
        })
    })
}

fn prewrap_key_value_attribute_tags(bytes: &[u8]) -> Vec<u32> {
    let key_value = decode(bytes).expect("test fixture is a valid pre-wrap Key Value TTLV item");
    key_value.with_value(|value| {
        let ValueView::Structure(key_value) = value else {
            panic!("pre-wrap Key Value is a Structure");
        };
        let attributes = key_value
            .children()
            .iter()
            .find(|field| field.tag().raw() == ATTRIBUTES)
            .expect("pre-wrap Key Value contains its associated Attributes Structure");
        attributes.with_value(|value| {
            let ValueView::Structure(attributes) = value else {
                panic!("pre-wrap Key Value Attributes is a Structure");
            };
            attributes
                .children()
                .iter()
                .map(|attribute| attribute.tag().raw())
                .collect()
        })
    })
}

struct RegisterCertificateFields {
    operation: u32,
    object_type: u32,
    attribute_tags: Vec<u32>,
    attribute_certificate_type: u32,
    certificate_length: i32,
    certificate_type: u32,
    certificate_der: Vec<u8>,
}

fn structure_value<R>(
    value: ValueView<'_>,
    description: &str,
    inspect: impl FnOnce(&StructureView<'_>) -> R,
) -> R {
    let ValueView::Structure(structure) = value else {
        panic!("{description} is a Structure");
    };
    inspect(&structure)
}

fn with_structure<R>(
    item: &Item,
    description: &str,
    inspect: impl FnOnce(&StructureView<'_>) -> R,
) -> R {
    item.with_value(|value| structure_value(value, description, inspect))
}

fn required_field<'a>(structure: &'a StructureView<'_>, tag: u32, description: &str) -> &'a Item {
    structure
        .children()
        .iter()
        .find(|field| field.tag().raw() == tag)
        .unwrap_or_else(|| panic!("{description} is present"))
}

fn enumeration_field(structure: &StructureView<'_>, tag: u32, description: &str) -> u32 {
    required_field(structure, tag, description).with_value(|value| match value {
        ValueView::Enumeration(raw) => *raw,
        _ => panic!("{description} is an Enumeration"),
    })
}

fn integer_field(structure: &StructureView<'_>, tag: u32, description: &str) -> i32 {
    required_field(structure, tag, description).with_value(|value| match value {
        ValueView::Integer(integer) => *integer,
        _ => panic!("{description} is an Integer"),
    })
}

fn byte_string_field(structure: &StructureView<'_>, tag: u32, description: &str) -> Vec<u8> {
    required_field(structure, tag, description).with_value(|value| match value {
        ValueView::ByteString(bytes) => bytes.to_vec(),
        _ => panic!("{description} is a Byte String"),
    })
}

fn register_certificate_attribute_fields(attributes: &StructureView<'_>) -> (Vec<u32>, u32, i32) {
    let tags = attributes
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect();
    let certificate_type =
        enumeration_field(attributes, CERTIFICATE_TYPE, "Attributes Certificate Type");
    let certificate_length = integer_field(
        attributes,
        CERTIFICATE_LENGTH,
        "Attributes Certificate Length",
    );
    (tags, certificate_type, certificate_length)
}

fn register_certificate_object_fields(certificate: &StructureView<'_>) -> (u32, Vec<u8>) {
    (
        enumeration_field(
            certificate,
            CERTIFICATE_TYPE,
            "Certificate object Certificate Type",
        ),
        byte_string_field(
            certificate,
            CERTIFICATE_VALUE,
            "Certificate object Certificate Value",
        ),
    )
}

fn register_certificate_payload_fields(
    payload: &StructureView<'_>,
    operation: u32,
) -> RegisterCertificateFields {
    let object_type = enumeration_field(payload, OBJECT_TYPE, "Register Object Type");
    let (attribute_tags, attribute_certificate_type, certificate_length) = with_structure(
        required_field(payload, ATTRIBUTES, "Register Attributes"),
        "Attributes",
        register_certificate_attribute_fields,
    );
    let (certificate_type, certificate_der) = with_structure(
        required_field(payload, CERTIFICATE, "Register Certificate object"),
        "Certificate object",
        register_certificate_object_fields,
    );
    RegisterCertificateFields {
        operation,
        object_type,
        attribute_tags,
        attribute_certificate_type,
        certificate_length,
        certificate_type,
        certificate_der,
    }
}

fn register_certificate_fields(bytes: &[u8]) -> RegisterCertificateFields {
    let message = decode(bytes).expect("captured request has valid TTLV framing");
    message.with_value(|value| {
        structure_value(value, "KMIP request message", |message| {
            with_structure(
                required_field(message, BATCH_ITEM, "Register Batch Item"),
                "Batch Item",
                |batch| {
                    let operation = enumeration_field(batch, OPERATION, "Batch Item Operation");
                    with_structure(
                        required_field(batch, REQUEST_PAYLOAD, "Register Request Payload"),
                        "Request Payload",
                        |payload| register_certificate_payload_fields(payload, operation),
                    )
                },
            )
        })
    })
}

fn certificate_signature_algorithm_oid(certificate_der: &[u8]) -> Vec<u8> {
    fn element(data: &[u8], offset: usize) -> (u8, usize, usize) {
        let tag = *data.get(offset).expect("DER element has a tag");
        let first_length = *data.get(offset + 1).expect("DER element has a length");
        let (length, content_start) = if first_length < 0x80 {
            (usize::from(first_length), offset + 2)
        } else {
            let length_bytes = usize::from(first_length & 0x7f);
            assert!(length_bytes > 0, "DER lengths are definite");
            assert!(length_bytes <= size_of::<usize>(), "DER length fits usize");
            let length_end = offset + 2 + length_bytes;
            let encoded_length = data
                .get(offset + 2..length_end)
                .expect("DER long-form length is complete");
            assert_ne!(encoded_length[0], 0, "DER length uses its shortest form");
            let length = encoded_length.iter().fold(0_usize, |length, byte| {
                length
                    .checked_mul(256)
                    .and_then(|length| length.checked_add(usize::from(*byte)))
                    .expect("DER length does not overflow usize")
            });
            assert!(length >= 0x80, "DER long-form length is canonical");
            (length, length_end)
        };
        let content_end = content_start
            .checked_add(length)
            .filter(|end| *end <= data.len())
            .expect("DER element content is complete");
        (tag, content_start, content_end)
    }

    let (certificate_tag, certificate_start, certificate_end) = element(certificate_der, 0);
    assert_eq!(certificate_tag, 0x30, "X.509 Certificate is a SEQUENCE");
    assert_eq!(
        certificate_end,
        certificate_der.len(),
        "DER certificate has no trailing bytes"
    );
    let (tbs_tag, _, tbs_end) = element(certificate_der, certificate_start);
    assert_eq!(tbs_tag, 0x30, "first Certificate field is TBSCertificate");
    assert!(
        tbs_end <= certificate_end,
        "TBSCertificate ends inside Certificate"
    );
    let (algorithm_tag, algorithm_start, algorithm_end) = element(certificate_der, tbs_end);
    assert_eq!(
        algorithm_tag, 0x30,
        "second Certificate field is signatureAlgorithm"
    );
    assert!(
        algorithm_end <= certificate_end,
        "signatureAlgorithm ends inside Certificate"
    );
    let (oid_tag, oid_start, oid_end) = element(certificate_der, algorithm_start);
    assert_eq!(
        oid_tag, 0x06,
        "AlgorithmIdentifier starts with an OBJECT IDENTIFIER"
    );
    assert!(
        oid_end <= algorithm_end,
        "algorithm OID ends inside AlgorithmIdentifier"
    );
    certificate_der[oid_start..oid_end].to_vec()
}

#[test]
fn register_sends_table_291_and_exposes_table_292_identifier_once() {
    crate::execute_test_support::assert_lifecycle_request_debug(
        &crate::ClientRequest::register(register_request_with_protection_storage_masks()),
        "Register",
    );
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-certificate".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![5, 9],
    });

    let item = client
        .register(
            register_request_with_protection_storage_masks(),
            &CodecLimits::defaults(),
        )
        .expect("valid Table 291 request receives typed Table 292 success");
    let outcome = item.outcome();
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        outcome,
        ClientOperation::Register,
        "Register(",
        "Register(",
    );
    let typed = outcome
        .response()
        .register()
        .expect("result retains its typed Register response");
    assert_eq!(outcome.operation(), ClientOperation::Register);
    assert_eq!(outcome.result().status().raw(), SUCCESS);
    assert_eq!(
        typed.unique_identifier(),
        Some(&kmipkit_protocol::UniqueIdentifier::TextString(
            "registered-certificate".to_owned()
        ))
    );
    let request = captured.borrow();
    let request = request.as_deref().expect("fake transport captured request");
    let (operation, fields, nested) = request_fields(request, 0);
    assert_eq!(
        operation,
        Some(REGISTER_OPERATION),
        "§11.36 assigns Register Operation 0x00000003"
    );
    assert_eq!(
        fields,
        [
            OBJECT_TYPE,
            ATTRIBUTES,
            CERTIFICATE,
            PROTECTION_STORAGE_MASKS
        ]
    );
    assert_eq!(
        nested,
        [
            vec![CERTIFICATE_LENGTH],
            vec![UNKNOWN_VENDOR_TAG],
            vec![PROTECTION_STORAGE_MASK],
        ]
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_preserves_caller_supplied_opaque_key_value_bytes_with_wrapping_data_from_table_291() {
    // OASIS KMIP v2.1 §§3.2–3.3, Tables 13–15; §11.18 Table 450; §6.1.43 Tables 291 and 293.
    // The caller supplies a deterministic AES-128-WRAP fixture with TTLV Encoding Option. KMIPKit
    // forwards it opaquely; this test does not exercise decryption, validate key material, or claim
    // that the fake server accepts or unwraps the request.
    let wrapped_key_value = REGISTER_WRAPPED_KEY_VALUE.to_vec();
    assert_eq!(
        prewrap_key_value_attribute_tags(SYNTHETIC_PREWRAP_KEY_VALUE_TTLV),
        [CRYPTOGRAPHIC_ALGORITHM, CRYPTOGRAPHIC_LENGTH],
        "§3.2 Table 13 places associated attributes in the Key Value Structure"
    );
    let request = RegisterRequest::new(
        ObjectType::from_raw(2),
        AttributeSet::try_new([
            test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
            test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
            test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(3)),
        ])
        .expect("symmetric key Attributes are well formed"),
        test_item(
            SYMMETRIC_KEY,
            Value::structure(test_structure([test_item(
                KEY_BLOCK,
                Value::structure(test_structure([
                    test_item(KEY_FORMAT_TYPE, Value::enumeration(1)),
                    test_item(KEY_VALUE, Value::byte_string(wrapped_key_value.clone())),
                    test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
                    test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
                    test_item(
                        KEY_WRAPPING_DATA,
                        Value::structure(test_structure([
                            test_item(WRAPPING_METHOD, Value::enumeration(1)),
                            test_item(ENCODING_OPTION, Value::enumeration(TTLV_ENCODING_OPTION)),
                            test_item(
                                ENCRYPTION_KEY_INFORMATION,
                                Value::structure(test_structure([test_item(
                                    UNIQUE_IDENTIFIER,
                                    Value::text_string("wrapping-key".to_owned()),
                                )])),
                            ),
                        ])),
                    ),
                ])),
            )])),
        ),
    );
    // This synthetic failure only completes the local fake exchange. It does not assert server acceptance or unwrapping.
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_OBJECT_TYPE),
        None,
        None,
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let outcome = client
        .register(request, &CodecLimits::defaults())
        .expect("the fake OperationFailed response completes one encoded Register exchange");
    assert_eq!(outcome.outcome().result().status().raw(), OPERATION_FAILED);

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("fake transport captured Register request");
    let (operation, fields, _) = request_fields(request, 0);
    assert_eq!(operation, Some(REGISTER_OPERATION));
    assert_eq!(fields, [OBJECT_TYPE, ATTRIBUTES, SYMMETRIC_KEY]);
    let wrapped = register_wrapping_evidence(request);
    assert_eq!(
        wrapped.direct_attribute_fields,
        [
            CRYPTOGRAPHIC_ALGORITHM,
            CRYPTOGRAPHIC_LENGTH,
            CRYPTOGRAPHIC_USAGE_MASK
        ]
    );
    assert_eq!(wrapped.direct_algorithm, 3);
    assert_eq!(wrapped.direct_length, 256);
    assert_eq!(
        wrapped.key_block_fields,
        [
            KEY_FORMAT_TYPE,
            KEY_VALUE,
            CRYPTOGRAPHIC_ALGORITHM,
            CRYPTOGRAPHIC_LENGTH,
            KEY_WRAPPING_DATA,
        ]
    );
    assert_eq!(wrapped.key_value_bytes, wrapped_key_value);
    assert_eq!(wrapped.wrapping_method, 1);
    assert_eq!(wrapped.encoding_option, TTLV_ENCODING_OPTION);
    assert_eq!(wrapped.encryption_key_identifier, "wrapping-key");
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_client_omits_signature_algorithm_when_x509_certificate_encodes_it() {
    let request = RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([
            test_item(CERTIFICATE_TYPE, Value::enumeration(1)),
            test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(4)),
            test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(2048)),
            test_item(
                CERTIFICATE_LENGTH,
                Value::integer(
                    i32::try_from(OASIS_PKCS12_STEP_2_CERTIFICATE_DER.len())
                        .expect("OASIS fixture length fits KMIP Integer"),
                ),
            ),
        ])
        .expect("X.509 certificate attributes are correctly typed"),
        test_item(
            CERTIFICATE,
            Value::structure(test_structure([
                test_item(CERTIFICATE_TYPE, Value::enumeration(1)),
                test_item(
                    CERTIFICATE_VALUE,
                    Value::byte_string(OASIS_PKCS12_STEP_2_CERTIFICATE_DER.to_vec()),
                ),
            ])),
        ),
    );
    // A failure response completes this local exchange without implying that
    // a server accepted or validated the certificate.
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_OBJECT_TYPE),
        None,
        None,
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    client
        .register(request, &CodecLimits::defaults())
        .expect("fake response completes the typed client exchange");

    let request = captured.borrow();
    let request = request.as_deref().expect("fake transport captured request");
    let fields = register_certificate_fields(request);
    assert_eq!(
        fields.operation, REGISTER_OPERATION,
        "Operation is Register"
    );
    assert_eq!(fields.object_type, 1, "Object Type is Certificate");
    assert_eq!(
        fields.attribute_certificate_type, 1,
        "Certificate Type attribute is X.509"
    );
    assert_eq!(
        fields.certificate_type, 1,
        "Certificate object type is X.509"
    );
    assert_eq!(
        fields.certificate_length,
        i32::try_from(OASIS_PKCS12_STEP_2_CERTIFICATE_DER.len())
            .expect("OASIS fixture length fits KMIP Integer")
    );
    assert!(
        fields.attribute_tags.contains(&CERTIFICATE_TYPE),
        "Table 293 Certificate Type is serialized"
    );
    assert!(
        fields.attribute_tags.contains(&CERTIFICATE_LENGTH),
        "Table 293 Certificate Length is serialized"
    );
    assert!(
        !fields.attribute_tags.contains(&DIGITAL_SIGNATURE_ALGORITHM),
        "Digital Signature Algorithm is omitted from Attributes"
    );
    assert!(
        !fields.attribute_tags.contains(&LINK_ATTRIBUTE),
        "the client can represent the certificate without a Link when associated private-key material is unavailable"
    );
    assert_eq!(fields.certificate_der, OASIS_PKCS12_STEP_2_CERTIFICATE_DER);
    assert_eq!(
        certificate_signature_algorithm_oid(&fields.certificate_der),
        [0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0b],
        "X.509 signatureAlgorithm contains SHA256withRSA OID 1.2.840.113549.1.1.11"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_produces_id_placeholder_for_a_later_identifier_omitting_operation() {
    let (mut client, fake, captured) = client_for(ExchangeScript::FailBeforeWrite);
    let batch = ClientBatch::from_items([
        ClientBatchItem::new(ClientRequest::Register(register_request()))
            .with_unique_batch_item_id(b"register-item".to_vec()),
        ClientBatchItem::new(ClientRequest::Check(CheckRequest::new()))
            .with_unique_batch_item_id(b"check-item".to_vec()),
    ]);

    let error = client
        .execute(batch, &CodecLimits::defaults())
        .expect_err("fake transport stops before any request bytes are sent");
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    let request = captured.borrow();
    let request = request.as_deref().expect("request reaches fake transport");
    assert_eq!(
        request_fields(request, 1),
        (Some(CHECK_OPERATION), Vec::new(), Vec::new()),
        "§4.1.2 permits Check to consume Register's returned ID Placeholder"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_failure_preserves_table_294_reason_without_retry() {
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_FAILED,
        Some(INVALID_OBJECT_TYPE),
        None,
        None,
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let item = client
        .register(register_request(), &CodecLimits::defaults())
        .expect("Table 294 Operation Failed is preserved as a KMIP result");
    assert_eq!(item.outcome().result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        item.outcome()
            .result()
            .reason()
            .map(kmipkit_protocol::ResultReason::raw),
        Some(INVALID_OBJECT_TYPE)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_pending_preserves_correlation_without_implicit_poll() {
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        OPERATION_PENDING,
        None,
        Some(PENDING_CORRELATION),
        Some(test_structure([])),
    );
    let (mut client, fake, _) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::Register(
        register_request(),
    )))
    .with_asynchronous_indicator(1);

    let response = client
        .execute(batch, &CodecLimits::defaults())
        .expect("caller-enabled typed batch preserves Register Pending");
    let item = response.get(0).expect("one Register result is associated");
    crate::execute_test_support::assert_lifecycle_outcome_surface(
        item.outcome(),
        ClientOperation::Register,
        "Register(",
        "Pending(",
    );
    let ClientBatchOutcome::Pending(pending) = item.outcome() else {
        panic!("server Operation Pending remains Pending");
    };
    assert_eq!(pending.operation(), ClientOperation::Register);
    assert_eq!(
        pending.asynchronous_correlation_value(),
        PENDING_CORRELATION
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

#[test]
fn register_transport_failure_preserves_partial_write_delivery_without_retry() {
    let (mut client, fake, _) =
        client_for(ExchangeScript::FailAfterPartialWrite { written_bytes: 3 });
    let error = client
        .register(register_request(), &CodecLimits::defaults())
        .expect_err("fake transport fails after partial request write");
    assert_eq!(error.category(), ClientErrorCategory::Transport);
    assert_eq!(
        error.delivery_state(),
        Some(RequestDeliveryState::PossiblySent)
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}

// OASIS KMIP v2.1 §2.3, Table 4; §6.1.43, Tables 291 and 293; §11.34, Table 468.
#[test]
fn managed_opaque_register_missing_direct_table_293_attribute_is_not_sent() {
    let attributes = AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ])
    .expect("the supplied direct attributes are individually well-formed");
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .register(
            managed_opaque_register_request(attributes),
            &CodecLimits::defaults(),
        )
        .expect_err("managed Opaque Object requires direct Cryptographic Usage Mask");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        error.cause_category(),
        Some(crate::ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(captured.borrow().is_none(), "no request bytes are captured");
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

// OASIS KMIP v2.1 §2.3 and §6.1.43, Table 291; local context never changes wire eligibility.
#[test]
fn selected_opaque_context_for_certificate_register_is_not_sent() {
    let request = register_request().with_opaque_object_context(OpaqueObjectContext::Generic);
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .register(request, &CodecLimits::defaults())
        .expect_err("an Opaque Object context cannot be selected for a Certificate");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        error.cause_category(),
        Some(crate::ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(captured.borrow().is_none(), "no request bytes are captured");
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

// OASIS KMIP v2.1 §2.3, Table 4; §6.1.43, Table 291; generic context has no Table 293 override.
#[test]
fn generic_opaque_register_sends_once_without_a_context_wire_marker() {
    let request = RegisterRequest::new(
        ObjectType::from_raw(OPAQUE_OBJECT_TYPE),
        AttributeSet::try_new([]).expect("an empty generic attribute set is valid"),
        test_item(
            OPAQUE_OBJECT,
            Value::structure(test_structure([
                test_item(OPAQUE_DATA_TYPE, Value::enumeration(0x8000_0001)),
                test_item(
                    OPAQUE_DATA_VALUE,
                    Value::byte_string(b"generic-opaque-data".to_vec()),
                ),
            ])),
        ),
    );
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-generic-opaque-object".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: Vec::new(),
    });

    client
        .register(request, &CodecLimits::defaults())
        .expect("default Generic Opaque Object Register is eligible for one exchange");

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("the fake transport captures the encoded Register request");
    let (operation, fields, nested) = request_fields(request, 0);
    assert_eq!(operation, Some(REGISTER_OPERATION));
    assert_eq!(fields, [OBJECT_TYPE, ATTRIBUTES, OPAQUE_OBJECT]);
    assert_eq!(
        nested,
        [vec![], vec![OPAQUE_DATA_TYPE, OPAQUE_DATA_VALUE]],
        "the local context is absent and Table 4 children stay ordered"
    );
    assert_eq!(
        register_attribute_values(request, 0),
        Vec::<(u32, RegisterAttributeValue)>::new()
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
    assert_eq!(fake.borrow().write_call_count(), 1);
}

// OASIS KMIP v2.1 §2.3, Table 4 requires both Opaque Data Type and Opaque Data Value.
#[test]
fn malformed_table_4_opaque_object_is_not_sent() {
    let request = RegisterRequest::new(
        ObjectType::from_raw(OPAQUE_OBJECT_TYPE),
        AttributeSet::try_new([]).expect("an empty generic attribute set is valid"),
        test_item(
            OPAQUE_OBJECT,
            Value::structure(test_structure([test_item(
                OPAQUE_DATA_TYPE,
                Value::enumeration(0x8000_0001),
            )])),
        ),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response: Vec::new(),
        request_write_chunks: Vec::new(),
    });

    let error = client
        .register(request, &CodecLimits::defaults())
        .expect_err("Table 4 requires Opaque Data Value as well as Opaque Data Type");

    assert_eq!(error.category(), ClientErrorCategory::Validation);
    assert_eq!(
        error.cause_category(),
        Some(crate::ClientCauseCategory::InvalidInput)
    );
    assert_eq!(error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert!(
        captured.borrow().is_none(),
        "malformed input is not encoded"
    );
    assert_eq!(fake.borrow().exchange_count(), 0);
    assert_eq!(fake.borrow().write_call_count(), 0);
}

// OASIS KMIP v2.1 §2.3, Table 4; §6.1.43, Tables 291 and 293; §11.34, Table 468.
#[test]
fn managed_opaque_register_preserves_standard_table_291_wire_shape_and_attribute_order() {
    let attributes = AttributeSet::try_new([
        test_item(CRYPTOGRAPHIC_USAGE_MASK, Value::integer(3)),
        test_item(CRYPTOGRAPHIC_ALGORITHM, Value::enumeration(3)),
        test_item(CRYPTOGRAPHIC_LENGTH, Value::integer(256)),
    ])
    .expect("all direct Table 293 attributes are well-formed");
    let response = asynchronous_response_bytes(
        REGISTER_OPERATION,
        SUCCESS,
        None,
        None,
        Some(test_structure([test_item(
            UNIQUE_IDENTIFIER,
            Value::text_string("registered-opaque-object".to_owned()),
        )])),
    );
    let (mut client, fake, captured) = client_for(ExchangeScript::Success {
        response,
        request_write_chunks: vec![4, 7],
    });

    client
        .register(
            managed_opaque_register_request(attributes),
            &CodecLimits::defaults(),
        )
        .expect("managed Opaque Object with direct Table 293 attributes succeeds");

    let request = captured.borrow();
    let request = request
        .as_deref()
        .expect("the successful request is captured by the fake transport");
    let (operation, fields, nested) = request_fields(request, 0);
    assert_eq!(operation, Some(REGISTER_OPERATION));
    assert_eq!(
        fields,
        [OBJECT_TYPE, ATTRIBUTES, OPAQUE_OBJECT],
        "Table 291 remains unchanged and carries no local context marker"
    );
    assert_eq!(
        nested,
        [
            vec![
                CRYPTOGRAPHIC_USAGE_MASK,
                CRYPTOGRAPHIC_ALGORITHM,
                CRYPTOGRAPHIC_LENGTH,
            ],
            vec![OPAQUE_DATA_TYPE, OPAQUE_DATA_VALUE],
        ],
        "direct attributes retain caller order and the object remains Table 4"
    );
    assert_eq!(
        register_attribute_values(request, 0),
        [
            (CRYPTOGRAPHIC_USAGE_MASK, RegisterAttributeValue::Integer(3),),
            (
                CRYPTOGRAPHIC_ALGORITHM,
                RegisterAttributeValue::Enumeration(3),
            ),
            (CRYPTOGRAPHIC_LENGTH, RegisterAttributeValue::Integer(256),),
        ],
        "the Table 293 direct attribute values and TTLV types are retained"
    );
    assert_eq!(fake.borrow().exchange_count(), 1);
}
