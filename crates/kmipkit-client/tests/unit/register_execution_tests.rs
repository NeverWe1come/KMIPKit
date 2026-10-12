#![cfg(test)]

//! Fake-transport Register contracts from OASIS KMIP v2.1 §6.1.43,
//! Tables 291–294. Traceability: all `KMIPKIT-REQ-SPEC-6.1.43-*` rows.
//! These are derived execution tests, not official fixture passes.

use kmipkit_protocol::{AttributeSet, CheckRequest, ObjectType, RegisterRequest};
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
const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
const PROTECTION_STORAGE_MASKS: u32 = 0x0042_015F;
const PROTECTION_STORAGE_MASK: u32 = 0x0042_0160;
const CERTIFICATE_LENGTH: u32 = 0x0042_00AD;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const OPERATION_PENDING: u32 = 2;
const INVALID_OBJECT_TYPE: u32 = 0x0000_0038;
const PENDING_CORRELATION: &[u8] = b"REGISTER_PENDING_CORRELATION\xFF";

// Byte-for-byte extraction from the pinned OASIS KMIP 2.1 fixture
// TC-PKCS12-1-21.xml, RequestMessage step 2. This local input does not execute
// the official test case or claim a server-side conformance result.
const OASIS_PKCS12_STEP_2_CERTIFICATE_DER: &[u8] =
    include_bytes!("../fixtures/tc-pkcs12-step2-certificate.der");

fn register_request() -> RegisterRequest {
    RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([test_item(CERTIFICATE_LENGTH, Value::integer(512))])
            .expect("Table 293 certificate attribute is correctly typed"),
        test_item(
            CERTIFICATE,
            Value::structure(test_structure([test_item(
                UNKNOWN_VENDOR_TAG,
                Value::byte_string(b"certificate-value".to_vec()),
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

struct RegisterCertificateFields {
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

fn register_certificate_payload_fields(payload: &StructureView<'_>) -> RegisterCertificateFields {
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
                    with_structure(
                        required_field(batch, REQUEST_PAYLOAD, "Register Request Payload"),
                        "Request Payload",
                        register_certificate_payload_fields,
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
fn register_client_omits_signature_algorithm_when_x509_certificate_encodes_it() {
    let request = RegisterRequest::new(
        ObjectType::from_raw(1),
        AttributeSet::try_new([
            test_item(CERTIFICATE_TYPE, Value::enumeration(1)),
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
