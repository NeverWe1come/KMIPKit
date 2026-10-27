//! Re-key Key Pair contracts from OASIS KMIP Specification v2.1 §6.1.47,
//! Tables 308–312. Traceability: `KMIPKIT-REQ-SPEC-6.1.47-001`,
//! `-004-001`, and `-004-002`.

use crate::async_operation_fixtures::{response_message, structure};
use crate::{
    RekeyKeyPairError, RekeyKeyPairRequest, RekeyKeyPairResponse, ResponseBatchItemView,
    ResponseMessage, ResultReason, UniqueIdentifier,
};
use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Value, ValueView};

const REKEY_KEY_PAIR_OPERATION: u32 = 0x0000_001d;
const SUCCESS: u32 = 0;
const OPERATION_FAILED: u32 = 1;
const KEY_VALUE_NOT_PRESENT: u32 = 0x0000_0013;
const PRIVATE_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_0066;
const PUBLIC_KEY_UNIQUE_IDENTIFIER: u32 = 0x0042_006f;
const OFFSET: u32 = 0x0042_0058;
const COMMON_ATTRIBUTES: u32 = 0x0042_0126;
const PRIVATE_KEY_ATTRIBUTES: u32 = 0x0042_0127;
const PUBLIC_KEY_ATTRIBUTES: u32 = 0x0042_0128;
const COMMON_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0163;
const PRIVATE_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0164;
const PUBLIC_PROTECTION_STORAGE_MASKS: u32 = 0x0042_0165;
const UNKNOWN_VENDOR_TAG: u32 = 0x0054_1234;

fn response_item(message: &ResponseMessage) -> ResponseBatchItemView<'_> {
    message
        .batch_items()
        .next()
        .expect("fixture contains one validated response batch item")
}

fn field_tags(payload: &Structure) -> Vec<u32> {
    payload
        .view()
        .children()
        .iter()
        .map(|field| field.tag().raw())
        .collect()
}

fn structure_contains_tag(view: &StructureView<'_>, raw_tag: u32) -> bool {
    view.children().iter().any(|field| {
        field.tag().raw() == raw_tag
            || field.with_value(|value| match value {
                ValueView::Structure(nested) => structure_contains_tag(&nested, raw_tag),
                _ => false,
            })
    })
}

fn item(raw_tag: u32, value: Value) -> Item {
    Item::new(
        RawTag::new(raw_tag)
            .expect("the source-derived tag fits TTLV width")
            .try_checked()
            .expect("the source-derived tag is allocated"),
        value,
    )
    .expect("the source-derived tag and value form a valid TTLV item")
}

fn identifier_item(raw_tag: u32, value: &str) -> Item {
    item(raw_tag, Value::text_string(value.to_owned()))
}

fn deeply_nested_structure() -> Structure {
    let mut nested = Structure::new();
    for _ in 1..64 {
        let mut parent = Structure::new();
        parent
            .try_push(item(UNKNOWN_VENDOR_TAG, Value::structure(nested)))
            .expect("each level remains within the model depth bound");
        nested = parent;
    }
    nested
}

fn typed_response(
    status: u32,
    reason: Option<u32>,
    payload: Option<Structure>,
) -> Result<RekeyKeyPairResponse, RekeyKeyPairError> {
    let message = response_message(REKEY_KEY_PAIR_OPERATION, status, reason, None, payload);
    RekeyKeyPairResponse::try_from_response_item(response_item(&message))
}

#[test]
fn request_emits_table_310_fields_once_in_order_and_preserves_groups() {
    let identifier = UniqueIdentifier::TextString("private-key".to_owned());
    let request = RekeyKeyPairRequest::new()
        .with_private_key_unique_identifier(identifier.clone())
        .with_offset(86_400)
        .with_common_attributes(crate::AttributeSet::new())
        .with_private_key_attributes(crate::AttributeSet::new())
        .with_public_key_attributes(crate::AttributeSet::new())
        .with_common_protection_storage_masks(structure([item(0x0042_0160, Value::integer(1))]))
        .with_private_protection_storage_masks(structure([item(0x0042_0160, Value::integer(2))]))
        .with_public_protection_storage_masks(structure([item(0x0042_0160, Value::integer(3))]));

    assert_eq!(request.private_key_unique_identifier(), Some(&identifier));
    let payload = request
        .into_ttlv_payload()
        .expect("the complete Table 310 request form is valid");
    assert_eq!(
        field_tags(&payload),
        [
            PRIVATE_KEY_UNIQUE_IDENTIFIER,
            OFFSET,
            COMMON_ATTRIBUTES,
            PRIVATE_KEY_ATTRIBUTES,
            PUBLIC_KEY_ATTRIBUTES,
            COMMON_PROTECTION_STORAGE_MASKS,
            PRIVATE_PROTECTION_STORAGE_MASKS,
            PUBLIC_PROTECTION_STORAGE_MASKS,
        ]
    );
    assert_eq!(
        payload.view().children()[1].with_value(|value| match value {
            ValueView::Interval(value) => Some(*value),
            _ => None,
        }),
        Some(86_400)
    );
}

#[test]
fn request_can_omit_private_identifier_for_the_batch_id_placeholder() {
    let payload = RekeyKeyPairRequest::new()
        .into_ttlv_payload()
        .expect("Table 310 permits an omitted Private Key Unique Identifier");
    assert!(payload.view().children().is_empty());
}

#[test]
fn request_rejects_repeated_singletons_in_every_attribute_group() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.47 Table 310.
    let repeated_comment = || {
        crate::AttributeSet::try_new([
            item(0x0042_00FD, Value::text_string("first".to_owned())),
            item(0x0042_00FD, Value::text_string("second".to_owned())),
        ])
        .expect("Comment accepts valid Text String instances before typed encoding")
    };

    assert!(
        RekeyKeyPairRequest::new()
            .with_common_attributes(repeated_comment())
            .into_ttlv_payload()
            .is_err()
    );
    assert!(
        RekeyKeyPairRequest::new()
            .with_private_key_attributes(repeated_comment())
            .into_ttlv_payload()
            .is_err()
    );
    assert!(
        RekeyKeyPairRequest::new()
            .with_public_key_attributes(repeated_comment())
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn request_rejects_repeated_dsa_without_explicit_pgp_object_context() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.47 Table 310.
    let repeated_dsa = crate::AttributeSet::try_new([
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ])
    .expect("Digital Signature Algorithm values have Enumeration encoding");
    assert!(
        RekeyKeyPairRequest::new()
            .with_public_key_attributes(repeated_dsa)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn request_rejects_repeated_dsa_in_common_attributes() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.47 Table 310.
    let attributes = crate::AttributeSet::try_new([
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ])
    .expect("Digital Signature Algorithm values have Enumeration encoding");

    assert!(
        RekeyKeyPairRequest::new()
            .with_common_attributes(attributes)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn request_rejects_repeated_dsa_in_private_key_attributes() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.47 Table 310.
    let attributes = crate::AttributeSet::try_new([
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ])
    .expect("Digital Signature Algorithm values have Enumeration encoding");

    assert!(
        RekeyKeyPairRequest::new()
            .with_private_key_attributes(attributes)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn request_rejects_repeated_dsa_in_public_key_attributes() {
    // KMIP Specification v2.1 §§4, 4.22 Table 72, and §6.1.47 Table 310.
    let attributes = crate::AttributeSet::try_new([
        item(0x0042_00AE, Value::enumeration(1)),
        item(0x0042_00AE, Value::enumeration(2)),
    ])
    .expect("Digital Signature Algorithm values have Enumeration encoding");

    assert!(
        RekeyKeyPairRequest::new()
            .with_public_key_attributes(attributes)
            .into_ttlv_payload()
            .is_err()
    );
}

#[test]
fn request_rejects_protection_mask_groups_beyond_model_depth() {
    let result = RekeyKeyPairRequest::new()
        .with_common_protection_storage_masks(deeply_nested_structure())
        .into_ttlv_payload();

    assert!(result.is_err());
}

#[test]
fn successful_response_returns_both_replacement_identifiers_and_unknown_fields() {
    let payload = structure([
        identifier_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, "replacement-private"),
        identifier_item(PUBLIC_KEY_UNIQUE_IDENTIFIER, "replacement-public"),
        item(UNKNOWN_VENDOR_TAG, Value::integer(-7)),
    ]);
    let response = typed_response(SUCCESS, None, Some(payload))
        .expect("Table 311 success returns both replacement identifiers");

    assert_eq!(response.result().status().raw(), SUCCESS);
    assert_eq!(
        response.private_key_unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "replacement-private".to_owned()
        ))
    );
    assert_eq!(
        response.public_key_unique_identifier(),
        Some(&UniqueIdentifier::TextString(
            "replacement-public".to_owned()
        ))
    );
    assert!(response.with_ttlv(|view| structure_contains_tag(&view, UNKNOWN_VENDOR_TAG)));
}

#[test]
fn successful_response_rejects_missing_or_duplicate_identifiers() {
    for payload in [
        structure([identifier_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, "private")]),
        structure([
            identifier_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, "private-1"),
            identifier_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, "private-2"),
            identifier_item(PUBLIC_KEY_UNIQUE_IDENTIFIER, "public"),
        ]),
        structure([
            identifier_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, "private"),
            identifier_item(PUBLIC_KEY_UNIQUE_IDENTIFIER, "public-1"),
            identifier_item(PUBLIC_KEY_UNIQUE_IDENTIFIER, "public-2"),
        ]),
        structure([
            item(PRIVATE_KEY_UNIQUE_IDENTIFIER, Value::boolean(true)),
            identifier_item(PUBLIC_KEY_UNIQUE_IDENTIFIER, "public"),
        ]),
        structure([
            identifier_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, "private"),
            item(PUBLIC_KEY_UNIQUE_IDENTIFIER, Value::boolean(true)),
        ]),
    ] {
        assert!(
            typed_response(SUCCESS, None, Some(payload)).is_err(),
            "Table 311 success requires exactly one identifier for each key"
        );
    }
}

#[test]
fn successful_response_debug_redacts_both_identifiers_and_unknown_values() {
    let payload = structure([
        identifier_item(PRIVATE_KEY_UNIQUE_IDENTIFIER, "PRIVATE_ID_SENTINEL"),
        identifier_item(PUBLIC_KEY_UNIQUE_IDENTIFIER, "PUBLIC_ID_SENTINEL"),
        item(
            UNKNOWN_VENDOR_TAG,
            Value::text_string("UNKNOWN_RESPONSE_SENTINEL".to_owned()),
        ),
    ]);
    let response = typed_response(SUCCESS, None, Some(payload))
        .expect("the Table 311 identifiers have valid forms");
    let debug = format!("{response:?}");

    assert!(!debug.contains("PRIVATE_ID_SENTINEL"));
    assert!(!debug.contains("PUBLIC_ID_SENTINEL"));
    assert!(!debug.contains("UNKNOWN_RESPONSE_SENTINEL"));
}

#[test]
fn failure_preserves_table_312_reason_without_success_identifiers() {
    let response = typed_response(OPERATION_FAILED, Some(KEY_VALUE_NOT_PRESENT), None)
        .expect("Table 312 Operation Failed remains a common KMIP result");
    assert_eq!(response.result().status().raw(), OPERATION_FAILED);
    assert_eq!(
        response.result().reason().map(ResultReason::raw),
        Some(KEY_VALUE_NOT_PRESENT)
    );
    assert!(response.private_key_unique_identifier().is_none());
    assert!(response.public_key_unique_identifier().is_none());
}

#[test]
fn request_debug_redacts_identifier_and_mask_contents() {
    let request = RekeyKeyPairRequest::new()
        .with_private_key_unique_identifier(UniqueIdentifier::TextString(
            "PRIVATE_KEY_IDENTIFIER_SENTINEL".to_owned(),
        ))
        .with_common_protection_storage_masks(structure([item(
            0x0042_0160,
            Value::text_string("PROTECTION_MASK_SECRET_SENTINEL".to_owned()),
        )]));
    let debug = format!("{request:?}");
    assert!(!debug.contains("PRIVATE_KEY_IDENTIFIER_SENTINEL"));
    assert!(!debug.contains("PROTECTION_MASK_SECRET_SENTINEL"));
}
