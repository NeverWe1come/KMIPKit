//! Property and boundary tests for generic vendor-extension preservation.
//!
//! OASIS KMIP v2.1 §9.13, Table 418 defines the standard Message Extension
//! fields and Vendor Extension subtree. Unknown vendor payload fields and
//! forward-compatible enum/bitmask values are `KMIPKit` preservation policy
//! (`KMIPKIT-0012-FR-007`, `KMIPKIT-0012-SC-003`), not new OASIS schema rules.
//! TTLV decoding bounds follow KMIPKIT-0005-FR-004 and KMIPKIT-0012-FR-012.

use kmipkit_protocol::{
    ProtocolError, ProtocolErrorKind, RequestMessage,
    extension::{self, ExtensionDefinition, ExtensionRegistryLimits},
};
use kmipkit_ttlv::codec::{CodecLimits, DecodeErrorKind, decode_with_limits};
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, StructureView, Tag, Value, ValueView};
use quickcheck::{Arbitrary, Gen, QuickCheck};

const PROPERTY_SEED: u64 = 0x4B4D_4950_4B49_5431;
const VENDOR_TAG: u32 = 0x0054_1234;
const UNKNOWN_STANDARD_TAG: u32 = 0x0042_0173;
const DISCRIMINATOR_TAG: u32 = 0x0054_0001;
const BEFORE_TAG: u32 = 0x0054_0002;
const ENUMERATION_TAG: u32 = 0x0054_0003;
const BITMASK_TAG: u32 = 0x0054_0004;
const AFTER_TAG: u32 = 0x0054_0005;
const DISCRIMINATOR: &str = "preservation-fixture-v1";

const REQUEST_HEADER_TAG: u32 = 0x0042_0077;
const PROTOCOL_VERSION_TAG: u32 = 0x0042_0069;
const PROTOCOL_VERSION_MAJOR_TAG: u32 = 0x0042_006A;
const PROTOCOL_VERSION_MINOR_TAG: u32 = 0x0042_006B;
const BATCH_COUNT_TAG: u32 = 0x0042_000D;
const BATCH_ITEM_TAG: u32 = 0x0042_000F;
const OPERATION_TAG: u32 = 0x0042_005C;
const REQUEST_PAYLOAD_TAG: u32 = 0x0042_0079;

#[derive(Clone, Debug)]
struct PreservationCase {
    enum_values: Vec<u32>,
    interleaving: Vec<u8>,
    vendor_enum: u32,
    unknown_standard_value: u32,
    bit_mask: u32,
}

impl Arbitrary for PreservationCase {
    fn arbitrary(generator: &mut Gen) -> Self {
        let enum_count = 2 + (usize::arbitrary(generator) % 7);
        let enum_values = (0..enum_count).map(|_| u32::arbitrary(generator)).collect();
        let interleave_count = usize::arbitrary(generator) % 12;
        let interleaving = (0..interleave_count)
            .map(|_| u8::arbitrary(generator))
            .collect();
        Self {
            enum_values,
            interleaving,
            vendor_enum: u32::arbitrary(generator),
            unknown_standard_value: u32::arbitrary(generator),
            bit_mask: u32::arbitrary(generator),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CapturedItem {
    tag: u32,
    value: CapturedValue,
}

#[derive(Debug, Eq, PartialEq)]
enum CapturedValue {
    Structure(Vec<CapturedItem>),
    Integer(i32),
    LongInteger(i64),
    BigInteger(Vec<u8>),
    Enumeration(u32),
    Boolean(bool),
    TextString(String),
    ByteString(Vec<u8>),
    DateTime(i64),
    Interval(u32),
    DateTimeExtended(i64),
}

fn tag(raw: u32) -> Tag {
    RawTag::new(raw)
        .expect("preservation fixture Tag fits the 24-bit field")
        .try_checked()
        .expect("preservation fixture Tag is accepted by the KMIP allocation policy")
}

fn item(raw: u32, value: Value) -> Item {
    Item::new(tag(raw), value).expect("an allocated Tag and Value form a generic Item")
}

fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
    let mut result = Structure::new();
    for item in items {
        result
            .try_push(item)
            .expect("preservation fixture remains within the generic model depth");
    }
    result
}

fn capture(view: &StructureView<'_>) -> Vec<CapturedItem> {
    view.children()
        .iter()
        .map(|child| {
            let value = child.with_value(|value| match value {
                ValueView::Structure(nested) => CapturedValue::Structure(capture(&nested)),
                ValueView::Integer(value) => CapturedValue::Integer(*value),
                ValueView::LongInteger(value) => CapturedValue::LongInteger(*value),
                ValueView::BigInteger(value) => CapturedValue::BigInteger(value.to_vec()),
                ValueView::Enumeration(value) => CapturedValue::Enumeration(*value),
                ValueView::Boolean(value) => CapturedValue::Boolean(*value),
                ValueView::TextString(value) => CapturedValue::TextString(value.to_owned()),
                ValueView::ByteString(value) => CapturedValue::ByteString(value.to_vec()),
                ValueView::DateTime(value) => CapturedValue::DateTime(*value),
                ValueView::Interval(value) => CapturedValue::Interval(*value),
                ValueView::DateTimeExtended(value) => CapturedValue::DateTimeExtended(*value),
                _ => unreachable!("all currently represented ValueView variants are captured"),
            });
            CapturedItem {
                tag: child.tag().raw(),
                value,
            }
        })
        .collect()
}

fn capture_structure(value: &Structure) -> Vec<CapturedItem> {
    capture(&value.view())
}

fn definition() -> ExtensionDefinition {
    let children = vec![
        extension::required(
            tag(DISCRIMINATOR_TAG),
            extension::scalar(ItemType::TextString).expect("Text String is supported"),
        )
        .expect("discriminator child rule is valid"),
        extension::required(
            tag(BEFORE_TAG),
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
        )
        .expect("order-start child rule is valid"),
        extension::repeated(
            tag(ENUMERATION_TAG),
            extension::scalar(ItemType::Enumeration).expect("Enumeration is supported"),
        )
        .expect("repeated Enumeration child rule is valid"),
        extension::required(
            tag(BITMASK_TAG),
            extension::scalar(ItemType::Integer).expect("Integer is supported"),
        )
        .expect("bitmask child rule is valid"),
        extension::required(
            tag(AFTER_TAG),
            extension::scalar(ItemType::Boolean).expect("Boolean is supported"),
        )
        .expect("order-end child rule is valid"),
    ];
    let order = vec![
        extension::extension_order_constraint(tag(BEFORE_TAG), tag(AFTER_TAG))
            .expect("the order edge connects distinct child Tags"),
    ];
    let schema = extension::structure(children, order, true)
        .expect("schema preserves Tags not declared by this extension");
    let identity = extension::extension_identity("example.vendor", "preservation", "1")
        .expect("the preservation identity is valid");
    let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
        .expect("the fixture includes this KMIP version");
    let path =
        extension::ttlv_path(tag(DISCRIMINATOR_TAG)).expect("the discriminator path is non-empty");
    let discriminator = extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
        .expect("the discriminator is a valid scalar");
    extension::extension_definition(identity, compatibility, discriminator, schema)
        .expect("the preservation definition is valid")
}

fn generated_extension(case: &PreservationCase) -> Structure {
    let mut children = vec![
        item(
            DISCRIMINATOR_TAG,
            Value::text_string(DISCRIMINATOR.to_owned()),
        ),
        item(BEFORE_TAG, Value::integer(-7)),
    ];

    let mut enum_values = case.enum_values.iter().copied();
    for selector in &case.interleaving {
        match selector % 3 {
            0 => {
                if let Some(value) = enum_values.next() {
                    children.push(item(ENUMERATION_TAG, Value::enumeration(value)));
                } else {
                    children.push(item(VENDOR_TAG, Value::enumeration(case.vendor_enum)));
                }
            }
            1 => children.push(item(VENDOR_TAG, Value::enumeration(case.vendor_enum))),
            _ => children.push(item(
                UNKNOWN_STANDARD_TAG,
                Value::byte_string(case.unknown_standard_value.to_be_bytes().to_vec()),
            )),
        }
    }
    for value in enum_values {
        children.push(item(ENUMERATION_TAG, Value::enumeration(value)));
    }

    children.extend([
        item(
            BITMASK_TAG,
            Value::integer(i32::from_ne_bytes(case.bit_mask.to_ne_bytes())),
        ),
        item(AFTER_TAG, Value::boolean(true)),
    ]);
    structure(children)
}

fn request_with_payload(payload: Structure) -> Structure {
    let version = structure([
        item(PROTOCOL_VERSION_MAJOR_TAG, Value::integer(2)),
        item(PROTOCOL_VERSION_MINOR_TAG, Value::integer(1)),
    ]);
    let request_header = structure([
        item(PROTOCOL_VERSION_TAG, Value::structure(version)),
        item(BATCH_COUNT_TAG, Value::integer(1)),
    ]);
    let batch_item = structure([
        item(OPERATION_TAG, Value::enumeration(1)),
        item(REQUEST_PAYLOAD_TAG, Value::structure(payload)),
    ]);
    structure([
        item(REQUEST_HEADER_TAG, Value::structure(request_header)),
        item(BATCH_ITEM_TAG, Value::structure(batch_item)),
    ])
}

#[allow(clippy::needless_pass_by_value)]
fn validation_retains_generated_generic_subtrees(case: PreservationCase) -> bool {
    let source = generated_extension(&case);
    let expected = capture_structure(&source);
    let result = extension::validate(&definition(), source, &CodecLimits::defaults());
    result
        .is_ok_and(|validated| capture_structure(extension::generic_value(&validated)) == expected)
}

#[allow(clippy::needless_pass_by_value)]
fn typed_request_round_trip_retains_generated_extension_fields(case: PreservationCase) -> bool {
    let source = request_with_payload(generated_extension(&case));
    let expected = capture_structure(&source);
    let Ok(parsed) = RequestMessage::try_from_ttlv(source) else {
        return false;
    };
    capture_structure(&parsed.into_ttlv()) == expected
}

#[test]
fn property_validation_preserves_unknown_tags_values_multiplicity_and_order() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(256)
        .quickcheck(validation_retains_generated_generic_subtrees as fn(PreservationCase) -> bool);
}

#[test]
fn property_typed_request_round_trip_preserves_extension_content() {
    QuickCheck::new()
        .rng(Gen::from_size_and_seed(64, PROPERTY_SEED))
        .tests(256)
        .quickcheck(
            typed_request_round_trip_retains_generated_extension_fields
                as fn(PreservationCase) -> bool,
        );
}

#[test]
fn repeated_discriminator_tag_is_a_non_match_without_a_typed_value() {
    let duplicate = structure([
        item(
            DISCRIMINATOR_TAG,
            Value::text_string(DISCRIMINATOR.to_owned()),
        ),
        item(
            DISCRIMINATOR_TAG,
            Value::text_string("different-vendor-value".into()),
        ),
        item(BEFORE_TAG, Value::integer(-7)),
        item(ENUMERATION_TAG, Value::enumeration(u32::MAX)),
        item(BITMASK_TAG, Value::integer(-1)),
        item(AFTER_TAG, Value::boolean(true)),
    ]);

    let result = extension::validate(&definition(), duplicate, &CodecLimits::defaults());
    assert_eq!(
        result
            .expect_err("a repeated discriminator path cannot produce a typed value")
            .kind(),
        ProtocolErrorKind::InvalidSchema
    );
}

#[test]
fn schema_only_validation_does_not_claim_discriminator_recognition() {
    let mismatching = || {
        structure([
            item(
                DISCRIMINATOR_TAG,
                Value::text_string("different-vendor-value".into()),
            ),
            item(BEFORE_TAG, Value::integer(-7)),
            item(BITMASK_TAG, Value::integer(3)),
            item(AFTER_TAG, Value::boolean(true)),
        ])
    };

    assert!(
        extension::validate(&definition(), mismatching(), &CodecLimits::defaults()).is_err(),
        "full validation still requires the definition's exact discriminator"
    );
    let outcome =
        extension::validate_schema_only(&definition(), mismatching(), &CodecLimits::defaults())
            .expect("schema-only validation checks its accurately named scope");

    assert!(matches!(
        outcome,
        extension::SchemaValidationOutcome::SchemaValid(value)
            if extension::validated_extension_value_identity(&value).name() == "preservation"
    ));
}

#[test]
fn ttlv_decoder_accepts_exact_byte_depth_and_element_limits() {
    // One Structure and one vendor-range Enumeration Item occupy 24 bytes.
    // The unknown Enumeration value must remain a represented u32.
    let wire = [
        0x42, 0x01, 0x73, 0x01, 0x00, 0x00, 0x00, 0x10, 0x54, 0x12, 0x34, 0x05, 0x00, 0x00, 0x00,
        0x04, 0xde, 0xad, 0xbe, 0xef, 0x00, 0x00, 0x00, 0x00,
    ];
    let exact = CodecLimits::new(wire.len(), 1, 2).expect("exact limits are valid");
    let decoded =
        decode_with_limits(&wire, &exact).expect("exact configured limits accept the tree");
    assert_eq!(decoded.tag().raw(), UNKNOWN_STANDARD_TAG);
    assert!(decoded.with_value(|value| {
        matches!(value, ValueView::Structure(view)
            if view.children().len() == 1
                && view.children()[0].tag().raw() == VENDOR_TAG
                && view.children()[0].with_value(|child| matches!(child, ValueView::Enumeration(raw) if *raw == 0xDEAD_BEEF)))
    }));

    let one_byte_too_small = CodecLimits::new(wire.len() - 1, 1, 2)
        .expect("a lowered byte limit is valid configuration");
    assert_eq!(
        decode_with_limits(&wire, &one_byte_too_small)
            .expect_err("one byte below the encoded tree is rejected")
            .kind(),
        DecodeErrorKind::MessageTooLarge
    );
    let too_shallow =
        CodecLimits::new(wire.len(), 0, 2).expect("zero Structure depth is a valid lowered limit");
    assert_eq!(
        decode_with_limits(&wire, &too_shallow)
            .expect_err("the root Structure exceeds depth zero")
            .kind(),
        DecodeErrorKind::StructureDepthExceeded
    );
    let too_few_elements =
        CodecLimits::new(wire.len(), 1, 1).expect("one element is a valid lowered limit");
    assert_eq!(
        decode_with_limits(&wire, &too_few_elements)
            .expect_err("the root and child count as two Items")
            .kind(),
        DecodeErrorKind::ElementLimitExceeded
    );
}

fn limits_with_overrides(
    max_schema_nodes: u64,
    max_child_rules: u64,
    max_payload_index_records: u64,
    max_lookup_comparisons: u64,
    max_depth: u64,
) -> Result<ExtensionRegistryLimits, ProtocolError> {
    let defaults = extension::defaults();
    extension::with_values(
        defaults.max_definitions(),
        max_schema_nodes,
        max_child_rules,
        defaults.max_text_bytes_per_field(),
        defaults.max_registry_text_bytes(),
        defaults.max_discriminator_scalar_bytes(),
        defaults.max_total_discriminator_scalar_bytes(),
        defaults.max_constraint_members_per_rule(),
        defaults.max_total_constraint_members(),
        max_payload_index_records,
        max_lookup_comparisons,
        max_depth,
    )
}

#[test]
fn configured_schema_width_and_depth_accept_exact_counts_and_reject_one_less() {
    let definition = definition();
    // The fixture has one root plus five scalar child nodes, with a root depth
    // of one and scalar-child depth of two.
    assert_eq!(
        extension::accounting(&definition)
            .expect("fixture accounting fits")
            .schema_nodes,
        6
    );
    assert!(
        extension::validate_schema_limits(
            &definition,
            &limits_with_overrides(6, 5, 200_000, 4_194_304, 2).expect("exact limits are valid"),
        )
        .is_ok()
    );

    for limits in [
        limits_with_overrides(6, 4, 200_000, 4_194_304, 2)
            .expect("one fewer child rule is valid configuration"),
        limits_with_overrides(6, 5, 200_000, 4_194_304, 1)
            .expect("one less depth is valid configuration"),
    ] {
        assert_eq!(
            extension::validate_schema_limits(&definition, &limits)
                .expect_err("the fixture exceeds one lowered schema limit")
                .kind(),
            ProtocolErrorKind::ResourceLimit
        );
    }
}

#[test]
fn configured_payload_index_and_lookup_limits_accept_the_hard_caps() {
    assert!(limits_with_overrides(100_000, 4_096, 200_000, 4_194_304, 64).is_ok());
    assert_eq!(
        limits_with_overrides(100_001, 4_096, 200_000, 4_194_304, 64)
            .expect_err("schema-node cap cannot exceed its hard maximum")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );
    assert_eq!(
        limits_with_overrides(100_000, 4_096, 200_001, 4_194_304, 64)
            .expect_err("payload-index cap cannot exceed its hard maximum")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );
    assert_eq!(
        limits_with_overrides(100_000, 4_096, 200_000, 4_194_305, 64)
            .expect_err("lookup-comparison cap cannot exceed its hard maximum")
            .kind(),
        ProtocolErrorKind::ResourceLimit
    );
}
