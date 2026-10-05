#![cfg(test)]

use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};
use static_assertions::assert_not_impl_any;

use super::{
    BorrowedLimitsView, EncodeError, EncodedOwner, EncodingPlan, check_plan,
    encode_item_with_identity_observer, encode_item_with_limits, plan_item_length,
    reserve_output_buffer,
};

const TEST_TAG_RAW: u32 = 0x0042_0173;
const DEFAULT_MAX_MESSAGE_BYTES: u64 = 16 * 1024 * 1024;
const MODEL_MAX_STRUCTURE_DEPTH: u64 = 64;
const DEFAULT_MAX_ELEMENTS: u64 = 100_000;

// All golden vectors use this allocated Tag. OASIS KMIP Specification v2.1
// §§10.1.1 and 11.56, KMIPKIT-0005-NR-001 and KMIPKIT-0005-NR-006;
// KMIPKIT-REQ-SPEC-11.56-001: Tags are 3-byte big-endian values with a 0x42
// or 0x54 prefix.
fn tag() -> Tag {
    RawTag::new(TEST_TAG_RAW)
        .expect("the test Tag fits the 24-bit field")
        .try_checked()
        .expect("the test Tag is allocated")
}

fn item(value: Value) -> Item {
    Item::new(tag(), value).expect("checked test Tag and owned value form an Item")
}

fn item_with_tag(tag: Tag, value: Value) -> Item {
    Item::new(tag, value).expect("checked test Tag and owned value form an Item")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LimitsView {
    message_bytes: u64,
    structure_depth: u64,
    elements: u64,
}

impl LimitsView {
    const fn new(max_message_bytes: u64, max_structure_depth: u64, max_elements: u64) -> Self {
        Self {
            message_bytes: max_message_bytes,
            structure_depth: max_structure_depth,
            elements: max_elements,
        }
    }

    const fn defaults() -> Self {
        Self::new(
            DEFAULT_MAX_MESSAGE_BYTES,
            MODEL_MAX_STRUCTURE_DEPTH,
            DEFAULT_MAX_ELEMENTS,
        )
    }
}

impl BorrowedLimitsView for LimitsView {
    fn max_message_bytes(&self) -> u128 {
        u128::from(self.message_bytes)
    }

    fn max_structure_depth(&self) -> u128 {
        u128::from(self.structure_depth)
    }

    fn max_elements(&self) -> u128 {
        u128::from(self.elements)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SyntheticPlan {
    encoded_bytes: u64,
    structure_depth: u64,
    elements: u64,
}

impl SyntheticPlan {
    const fn new(encoded_bytes: u64, structure_depth: u64, elements: u64) -> Self {
        Self {
            encoded_bytes,
            structure_depth,
            elements,
        }
    }

    const fn as_plan(self) -> EncodingPlan {
        EncodingPlan::new(self.encoded_bytes, self.structure_depth, self.elements)
    }
}

fn assert_plan_boundary(
    limits: &impl BorrowedLimitsView,
    exact: SyntheticPlan,
    one_over: SyntheticPlan,
) {
    assert_eq!(check_plan(&exact.as_plan(), limits), Ok(()));
    assert_eq!(
        check_plan(&one_over.as_plan(), limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[derive(Default)]
pub(super) struct EncodingObserver {
    calls: Cell<usize>,
    output_reservation_calls: Cell<usize>,
}

impl EncodingObserver {
    pub(super) fn calls(&self) -> usize {
        self.calls.get()
    }

    pub(super) fn record_copy(&self) {
        self.calls.set(self.calls.get() + 1);
    }

    fn output_reservation_calls(&self) -> usize {
        self.output_reservation_calls.get()
    }

    pub(super) fn record_output_reservation(&self) {
        self.output_reservation_calls
            .set(self.output_reservation_calls.get() + 1);
    }
}

pub(super) struct LimitsIdentityObserver<'a> {
    expected: &'a CodecLimits,
    calls: Cell<usize>,
    same_instance: Cell<bool>,
}

impl<'a> LimitsIdentityObserver<'a> {
    pub(super) fn new(expected: &'a CodecLimits) -> Self {
        Self {
            expected,
            calls: Cell::new(0),
            same_instance: Cell::new(true),
        }
    }

    pub(super) fn record(&self, actual: &CodecLimits) {
        self.calls.set(self.calls.get().saturating_add(1));
        self.same_instance
            .set(self.same_instance.get() && std::ptr::eq(self.expected, actual));
    }

    fn calls(&self) -> usize {
        self.calls.get()
    }

    fn always_same_instance(&self) -> bool {
        self.same_instance.get()
    }
}

#[derive(Clone)]
pub(super) struct ZeroizationObserver(Rc<Cell<Option<(usize, bool)>>>);

impl ZeroizationObserver {
    pub(super) fn new() -> Self {
        Self(Rc::new(Cell::new(None)))
    }

    pub(super) fn observe_before_deallocation(&self, initialized_bytes: &[u8]) {
        self.0.set(Some((
            initialized_bytes.len(),
            initialized_bytes.iter().all(|byte| *byte == 0),
        )));
    }

    fn result(&self) -> Option<(usize, bool)> {
        self.0.get()
    }
}

fn encode_item(item: &Item) -> Result<Vec<u8>, EncodeError> {
    super::encode_item(item).map(|owner| owner.as_bytes().to_vec())
}

assert_not_impl_any!(
    EncodedOwner:
        Clone,
        Copy,
        fmt::Debug,
        fmt::Display,
        serde::Serialize,
        AsMut<[u8]>,
        Into<Vec<u8>>
);

fn expected_item(type_code: u8, item_length: u32, value_and_padding: &[u8]) -> Vec<u8> {
    let mut expected = vec![0x42, 0x01, 0x73, type_code];
    expected.extend_from_slice(&item_length.to_be_bytes());
    expected.extend_from_slice(value_and_padding);
    expected
}

#[test]
fn encodes_empty_structure_golden_vector() {
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, 10.1.5, and 11.23;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005:
    // Structure is Item Type 0x01 and its empty value has Item Length zero.
    let encoded = encode_item(&item(Value::structure(Structure::new())))
        .expect("an empty Structure is encodable");

    assert_eq!(encoded, [0x42, 0x01, 0x73, 0x01, 0, 0, 0, 0]);
}

#[test]
fn encodes_integer_golden_vector_with_signed_big_endian_value() {
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, 10.1.5, and 11.56;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-002: Integer is signed big-endian and has
    // four following padding bytes. Zero padding octets are the FR-002
    // canonical output; OASIS does not prescribe their values.
    // KMIPKIT-REQ-SPEC-11.56-001: the allocated test Tag starts with 0x42.
    let encoded =
        encode_item(&item(Value::integer(i32::MIN))).expect("Integer vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0x80, 0, 0, 0, 0, 0, 0, 0,
        ]
    );
}

#[test]
fn encodes_extension_tag_with_oasis_0x54_prefix() {
    // OASIS KMIP Specification v2.1 §11.56;
    // KMIPKIT-REQ-SPEC-11.56-001: extension Tags use the 0x54 first byte.
    let extension_tag = RawTag::new(0x0054_1234)
        .expect("the extension Tag fits the 24-bit field")
        .try_checked()
        .expect("the Tag is in the allocated extension range");
    let encoded = encode_item(&item_with_tag(extension_tag, Value::integer(7)))
        .expect("extension Tag item is encodable");

    assert_eq!(
        encoded.as_slice(),
        [0x54, 0x12, 0x34, 0x02, 0, 0, 0, 4, 0, 0, 0, 7, 0, 0, 0, 0]
    );
}

#[test]
fn encodes_long_integer_golden_vector_with_signed_big_endian_value() {
    // OASIS KMIP Specification v2.1 §§10.1.2 and 10.1.3;
    // KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004: Long Integer is a signed
    // 64-bit big-endian value with Item Length eight.
    let encoded = encode_item(&item(Value::long_integer(i64::MAX)))
        .expect("Long Integer vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x03, 0, 0, 0, 8, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ]
    );
}

#[test]
fn encodes_big_integer_golden_vector_with_sign_extension_in_item_length() {
    // OASIS KMIP Specification v2.1 §§10.1.2 and 10.1.3;
    // KMIPKIT-0005-NR-003 and KMIPKIT-0005-NR-004;
    // KMIPKIT-REQ-SPEC-10.1.2-002-001 and KMIPKIT-REQ-SPEC-10.1.2-002-002:
    // prepend minimum sign-extension bytes to an eight-byte boundary and
    // include them in Item Length.
    let encoded = encode_item(&item(Value::big_integer(vec![0x80])))
        .expect("negative Big Integer vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x04, 0, 0, 0, 8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x80,
        ]
    );
}

#[test]
fn rejects_empty_big_integer_as_a_project_validity_rule() {
    // Project-only empty-value rejection under KMIPKIT-0005-FR-002.
    let result = encode_item(&item(Value::big_integer(Vec::new())));

    assert_eq!(result, Err(EncodeError::EmptyBigInteger));
}

#[test]
fn encodes_enumeration_golden_vector_with_unsigned_big_endian_value() {
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, and 10.1.5;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-002: Enumeration is unsigned big-endian and
    // has four following padding bytes. Zero padding octets are the FR-002
    // canonical output; OASIS does not prescribe their values.
    let encoded =
        encode_item(&item(Value::enumeration(u32::MAX))).expect("Enumeration vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x05, 0, 0, 0, 4, 0xff, 0xff, 0xff, 0xff, 0, 0, 0, 0,
        ]
    );
}

#[test]
fn encodes_boolean_golden_vector_with_exact_eight_byte_value() {
    // OASIS KMIP Specification v2.1 §§10.1.2 and 10.1.3;
    // KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004: Boolean uses its defined
    // eight-byte value representation.
    let encoded = encode_item(&item(Value::boolean(true))).expect("Boolean vector is encodable");

    assert_eq!(
        encoded,
        [0x42, 0x01, 0x73, 0x06, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 1]
    );
}

#[test]
fn encodes_text_string_golden_vector_with_minimum_following_padding() {
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, and 10.1.5;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-001: Text String padding is minimal. Exact
    // UTF-8 bytes and zero fill are KMIPKit's canonical FR-002 vector.
    let encoded = encode_item(&item(Value::text_string("é".to_owned())))
        .expect("Text String vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x07, 0, 0, 0, 2, 0xc3, 0xa9, 0, 0, 0, 0, 0, 0,
        ]
    );
}

#[test]
fn encodes_byte_string_golden_vector_with_minimum_following_padding() {
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, and 10.1.5;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-001: Byte String padding is minimal. Exact
    // payload and zero fill are KMIPKit's canonical FR-002 vector.
    let encoded = encode_item(&item(Value::byte_string(vec![
        0xa1, 0xb2, 0xc3, 0xd4, 0xe5,
    ])))
    .expect("Byte String vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x08, 0, 0, 0, 5, 0xa1, 0xb2, 0xc3, 0xd4, 0xe5, 0, 0, 0,
        ]
    );
}

#[test]
fn encodes_date_time_golden_vector_with_signed_big_endian_value() {
    // OASIS KMIP Specification v2.1 §§10.1.2 and 10.1.3;
    // KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004: Date Time is a signed
    // 64-bit big-endian value with Item Length eight.
    let encoded =
        encode_item(&item(Value::date_time(i64::MIN))).expect("Date Time vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x09, 0, 0, 0, 8, 0x80, 0, 0, 0, 0, 0, 0, 0,
        ]
    );
}

#[test]
fn encodes_interval_golden_vector_with_unsigned_big_endian_value() {
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, and 10.1.5;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-002: Interval is unsigned big-endian and is
    // followed by four padding bytes. Zero padding octets are the FR-002
    // canonical output; OASIS does not prescribe their values.
    let encoded =
        encode_item(&item(Value::interval(0x0102_0304))).expect("Interval vector is encodable");

    assert_eq!(
        encoded,
        [0x42, 0x01, 0x73, 0x0a, 0, 0, 0, 4, 1, 2, 3, 4, 0, 0, 0, 0,]
    );
}

#[test]
fn encodes_date_time_extended_golden_vector_with_signed_big_endian_value() {
    // OASIS KMIP Specification v2.1 §§10.1.2 and 10.1.3;
    // KMIPKIT-0005-NR-002 and KMIPKIT-0005-NR-004: Date Time Extended is a
    // signed 64-bit big-endian value with Item Length eight.
    let encoded = encode_item(&item(Value::date_time_extended(i64::MAX)))
        .expect("Date Time Extended vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x0b, 0, 0, 0, 8, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ]
    );
}

#[test]
fn fixed_width_four_byte_values_have_four_following_padding_bytes() {
    // OASIS KMIP Specification v2.1 §§10.1.3 and 10.1.5;
    // KMIPKIT-0005-NR-004 and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-002: Integer, Enumeration, and Interval
    // receive four following padding bytes; this checks all three families.
    // Zero padding octets are FR-002 canonical output, not an OASIS byte-value
    // requirement.
    let cases = [
        (Value::integer(7), 0x02, [0, 0, 0, 7, 0, 0, 0, 0]),
        (Value::enumeration(7), 0x05, [0, 0, 0, 7, 0, 0, 0, 0]),
        (Value::interval(7), 0x0a, [0, 0, 0, 7, 0, 0, 0, 0]),
    ];

    for (value, type_code, value_and_padding) in cases {
        let encoded = encode_item(&item(value)).expect("fixed-width vector is encodable");
        assert_eq!(
            encoded,
            expected_item(type_code, 4, &value_and_padding),
            "Item Type 0x{type_code:02x} must carry four following padding bytes"
        );
    }
}

#[test]
fn text_and_byte_string_padding_is_minimal_at_eight_byte_boundaries() {
    // OASIS KMIP Specification v2.1 §§10.1.3 and 10.1.5;
    // KMIPKIT-0005-NR-004 and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-001: string and byte padding reaches the
    // next eight-byte boundary with the minimum following bytes. Zero fill is
    // FR-002 canonical output, not an OASIS byte-value requirement. This
    // bounded table covers empty, one-byte, seven-byte, and aligned values.
    for length in [0_usize, 1, 7, 8] {
        let text = "a".repeat(length);
        let text_value = text.as_bytes();
        let text_padding = (8 - (length % 8)) % 8;
        let mut text_value_and_padding = text_value.to_vec();
        text_value_and_padding.resize(length + text_padding, 0);
        let text_expected = expected_item(
            0x07,
            u32::try_from(length).expect("bounded fixture fits"),
            &text_value_and_padding,
        );
        assert_eq!(
            encode_item(&item(Value::text_string(text)))
                .expect("Text String boundary vector is encodable"),
            text_expected,
            "Text String length {length}"
        );

        let bytes = vec![0x5a; length];
        let byte_padding = (8 - (length % 8)) % 8;
        let mut byte_value_and_padding = bytes.clone();
        byte_value_and_padding.resize(length + byte_padding, 0);
        let byte_expected = expected_item(
            0x08,
            u32::try_from(length).expect("bounded fixture fits"),
            &byte_value_and_padding,
        );
        assert_eq!(
            encode_item(&item(Value::byte_string(bytes)))
                .expect("Byte String boundary vector is encodable"),
            byte_expected,
            "Byte String length {length}"
        );
    }
}

#[test]
fn big_integer_padding_is_minimal_and_sign_extended() {
    // OASIS KMIP Specification v2.1 §§10.1.2 and 10.1.3;
    // KMIPKIT-0005-NR-003 and KMIPKIT-0005-NR-004;
    // KMIPKIT-REQ-SPEC-10.1.2-002-001 and KMIPKIT-REQ-SPEC-10.1.2-002-002:
    // padding is minimal sign extension to an eight-byte length and is part of
    // Item Length. These fixtures stay below eight initialized input bytes.
    let cases = [
        (vec![0x7f], [0, 0, 0, 0, 0, 0, 0, 0x7f]),
        (vec![0x80], [0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x80]),
    ];

    for (value, expected_value) in cases {
        assert_eq!(
            encode_item(&item(Value::big_integer(value)))
                .expect("Big Integer boundary vector is encodable"),
            expected_item(0x04, 8, &expected_value)
        );
    }
}

#[test]
fn structure_preserves_child_order_and_repeated_tags() {
    // Child-order and repeated-tag preservation is KMIPKit-only under
    // KMIPKIT-0005-FR-003. This does not verify schema-defined field order;
    // the exact Integer child encodings and parent Structure Length follow
    // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, and 10.1.5;
    // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005;
    // KMIPKIT-REQ-SPEC-10.1.5-001-002.
    let mut children = Structure::new();
    children
        .try_push(item_with_tag(tag(), Value::integer(1)))
        .expect("first shallow child fits the model depth");
    children
        .try_push(item_with_tag(tag(), Value::integer(2)))
        .expect("second repeated-tag child fits the model depth");

    let encoded = encode_item(&item(Value::structure(children)))
        .expect("ordered Structure vector is encodable");

    assert_eq!(
        encoded,
        [
            0x42, 0x01, 0x73, 0x01, 0, 0, 0, 32, 0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0, 0, 0, 1, 0,
            0, 0, 0, 0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 0,
        ]
    );
}

#[test]
fn default_byte_limit_accepts_exact_boundary_and_rejects_one_over() {
    // Project-only resource policy under KMIPKIT-0005-FR-007. Synthetic
    // planning avoids allocating a 16 MiB payload or output.
    let limits = LimitsView::defaults();
    let exact = SyntheticPlan::new(DEFAULT_MAX_MESSAGE_BYTES, 1, 1);
    let over = SyntheticPlan::new(DEFAULT_MAX_MESSAGE_BYTES + 1, 1, 1);

    assert_eq!(check_plan(&exact.as_plan(), &limits), Ok(()));
    assert_eq!(
        check_plan(&over.as_plan(), &limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn configured_lowered_byte_limit_accepts_exact_boundary_and_rejects_one_over() {
    // Project-only configured per-call limit behavior under
    // KMIPKIT-0005-FR-007. No payload or output allocation is made.
    let limits = LimitsView::new(64, MODEL_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
    let exact = SyntheticPlan::new(64, 1, 1);
    let over = SyntheticPlan::new(65, 1, 1);

    assert_eq!(check_plan(&exact.as_plan(), &limits), Ok(()));
    assert_eq!(
        check_plan(&over.as_plan(), &limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn configured_raised_byte_limit_allows_a_plan_above_default() {
    // Project-only configurable byte limit behavior under
    // KMIPKIT-0005-FR-007. The raised plan is synthetic and allocates nothing.
    let limits = LimitsView::new(
        DEFAULT_MAX_MESSAGE_BYTES + 32,
        MODEL_MAX_STRUCTURE_DEPTH,
        DEFAULT_MAX_ELEMENTS,
    );
    let above_default = SyntheticPlan::new(DEFAULT_MAX_MESSAGE_BYTES + 1, 1, 1);

    assert_eq!(check_plan(&above_default.as_plan(), &limits), Ok(()));
}

#[test]
fn default_depth_limit_accepts_64_and_rejects_synthetic_depth_65() {
    // Project-only resource policy under KMIPKIT-0005-FR-007. The depth-65
    // value is a planning fixture; no nested model tree is constructed.
    let limits = LimitsView::defaults();
    let exact = SyntheticPlan::new(8, MODEL_MAX_STRUCTURE_DEPTH, 1);
    let over = SyntheticPlan::new(8, MODEL_MAX_STRUCTURE_DEPTH + 1, 1);

    assert_eq!(check_plan(&exact.as_plan(), &limits), Ok(()));
    assert_eq!(
        check_plan(&over.as_plan(), &limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn configured_lowered_depth_limit_accepts_exact_boundary_and_rejects_one_over() {
    // Project-only configured per-call depth behavior under
    // KMIPKIT-0005-FR-007. Synthetic plans avoid deep recursive trees.
    let limits = LimitsView::new(DEFAULT_MAX_MESSAGE_BYTES, 12, DEFAULT_MAX_ELEMENTS);
    let exact = SyntheticPlan::new(8, 12, 1);
    let over = SyntheticPlan::new(8, 13, 1);

    assert_eq!(check_plan(&exact.as_plan(), &limits), Ok(()));
    assert_eq!(
        check_plan(&over.as_plan(), &limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn configured_depth_cannot_raise_the_model_hard_maximum() {
    // Project-only model-bound policy under KMIPKIT-0005-FR-007. Depth 65 is
    // synthetic and no Structure allocation is performed.
    let limits = LimitsView::new(DEFAULT_MAX_MESSAGE_BYTES, 65, DEFAULT_MAX_ELEMENTS);
    let plan = SyntheticPlan::new(8, 65, 1);

    assert_eq!(
        check_plan(&plan.as_plan(), &limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn default_element_limit_accepts_exact_boundary_and_rejects_100001() {
    // Project-only resource policy under KMIPKIT-0005-FR-007. Element counts
    // are synthetic; this test creates no 100,001-item Structure.
    let limits = LimitsView::defaults();
    let exact = SyntheticPlan::new(8, 1, DEFAULT_MAX_ELEMENTS);
    let over = SyntheticPlan::new(8, 1, DEFAULT_MAX_ELEMENTS + 1);

    assert_eq!(check_plan(&exact.as_plan(), &limits), Ok(()));
    assert_eq!(
        check_plan(&over.as_plan(), &limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn configured_lowered_element_limit_accepts_exact_boundary_and_rejects_one_over() {
    // Project-only configured per-call count behavior under
    // KMIPKIT-0005-FR-007. Element counts are synthetic.
    let limits = LimitsView::new(DEFAULT_MAX_MESSAGE_BYTES, MODEL_MAX_STRUCTURE_DEPTH, 24);
    let exact = SyntheticPlan::new(8, 1, 24);
    let over = SyntheticPlan::new(8, 1, 25);

    assert_eq!(check_plan(&exact.as_plan(), &limits), Ok(()));
    assert_eq!(
        check_plan(&over.as_plan(), &limits),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn configured_raised_element_limit_allows_a_plan_above_default() {
    // Project-only configurable count behavior under KMIPKIT-0005-FR-007.
    // The 100,001 count is a number only, with no corresponding tree.
    let limits = LimitsView::new(
        DEFAULT_MAX_MESSAGE_BYTES,
        MODEL_MAX_STRUCTURE_DEPTH,
        DEFAULT_MAX_ELEMENTS + 1,
    );
    let above_default = SyntheticPlan::new(8, 1, DEFAULT_MAX_ELEMENTS + 1);

    assert_eq!(check_plan(&above_default.as_plan(), &limits), Ok(()));
}

#[test]
fn codec_limits_adapter_enforces_default_and_configured_exact_boundaries() {
    // Resource limits are project policy under KMIPKIT-0005-FR-007. These
    // bounded plans verify the real borrowed adapter without constructing
    // 16 MiB output, 65 nested Structures, or 100,001 Items.
    let defaults = CodecLimits::defaults();
    assert_plan_boundary(
        &defaults,
        SyntheticPlan::new(DEFAULT_MAX_MESSAGE_BYTES, 1, 1),
        SyntheticPlan::new(DEFAULT_MAX_MESSAGE_BYTES + 1, 1, 1),
    );
    assert_plan_boundary(
        &defaults,
        SyntheticPlan::new(8, MODEL_MAX_STRUCTURE_DEPTH, 1),
        SyntheticPlan::new(8, MODEL_MAX_STRUCTURE_DEPTH + 1, 1),
    );
    assert_plan_boundary(
        &defaults,
        SyntheticPlan::new(8, 1, DEFAULT_MAX_ELEMENTS),
        SyntheticPlan::new(8, 1, DEFAULT_MAX_ELEMENTS + 1),
    );

    let configured = CodecLimits::new(64, 12, 24)
        .expect("the configured depth does not exceed the model maximum");
    assert_plan_boundary(
        &configured,
        SyntheticPlan::new(64, 1, 1),
        SyntheticPlan::new(65, 1, 1),
    );
    assert_plan_boundary(
        &configured,
        SyntheticPlan::new(8, 12, 1),
        SyntheticPlan::new(8, 13, 1),
    );
    assert_plan_boundary(
        &configured,
        SyntheticPlan::new(8, 1, 24),
        SyntheticPlan::new(8, 1, 25),
    );
}

#[test]
fn borrowed_per_call_limits_do_not_share_mutable_state() {
    // Project-only per-call behavior under KMIPKIT-0005-FR-007. Both views
    // remain immutable and the plan is reused without global state.
    let restrictive = LimitsView::new(32, 8, 10);
    let permissive = LimitsView::new(128, 8, 10);
    let plan = SyntheticPlan::new(64, 4, 5);

    assert_eq!(check_plan(&plan.as_plan(), &permissive), Ok(()));
    assert_eq!(
        check_plan(&plan.as_plan(), &restrictive),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn writer_uses_the_same_borrowed_codec_limits_instance_for_each_operation() {
    // KMIPKit's private per-operation configuration boundary: FR-007.
    let limits = CodecLimits::new(16, 0, 1).expect("limits fit the model boundary");
    let observer = LimitsIdentityObserver::new(&limits);
    let copy_observer = EncodingObserver::default();
    let input = item(Value::byte_string(vec![0x5a]));

    for _ in 0..2 {
        let output = encode_item_with_identity_observer(&input, &limits, &copy_observer, &observer)
            .expect("16 encoded bytes fit the exact configured byte limit");
        assert_eq!(output.as_bytes().len(), limits.max_message_bytes());
    }

    assert_eq!(observer.calls(), 2);
    assert!(observer.always_same_instance());
    assert_eq!(copy_observer.calls(), 2);
    assert_eq!(copy_observer.output_reservation_calls(), 2);
}

#[test]
fn item_length_planner_accepts_u32_max_without_allocating() {
    // OASIS KMIP Specification v2.1 §10.1.3, KMIPKIT-0005-NR-004: Item Length
    // is unsigned 32-bit. This synthetic maximum is never materialized.
    assert_eq!(plan_item_length(u64::from(u32::MAX)), Ok(u32::MAX));
}

#[test]
fn item_length_planner_rejects_u32_max_plus_one_without_allocating() {
    // OASIS KMIP Specification v2.1 §10.1.3, KMIPKIT-0005-NR-004: Item Length
    // cannot represent this synthetic one-over value; no buffer is created.
    assert_eq!(
        plan_item_length(u64::from(u32::MAX) + 1),
        Err(EncodeError::ItemLengthOverflow)
    );
}

#[test]
fn allocation_error_exposes_its_reservation_source() {
    let result = reserve_output_buffer(usize::MAX, None);
    let Err(error) = result else {
        panic!("impossible output capacity was accepted");
    };
    let source = std::error::Error::source(&error)
        .expect("allocation error retains its original reservation source");

    assert!(source.is::<std::collections::TryReserveError>());
    assert!(!format!("{error}").contains(&source.to_string()));
    assert!(!format!("{error:?}").contains(&source.to_string()));
}

#[test]
fn preflight_rejection_performs_zero_payload_copies() {
    // Project-only security/error behavior under KMIPKIT-0005-FR-009. The
    // observer sits at the production payload-copy boundary. The item is
    // rejected by preflight before the output buffer or any payload copy.
    let limits = CodecLimits::new(4, 64, 100_000).expect("depth is within the model");
    let copy_observer = EncodingObserver::default();
    let payload = vec![0xa5, 0x5a, 0xa5];
    let input = item(Value::byte_string(payload));
    let result = encode_item_with_limits(&input, &limits, &copy_observer);

    assert_eq!(result.err(), Some(EncodeError::LimitExceeded));
    assert_eq!(copy_observer.output_reservation_calls(), 0);
    assert_eq!(copy_observer.calls(), 0);
}

#[test]
fn preflight_error_does_not_format_payload_bytes() {
    // Project-only payload-redaction behavior under KMIPKIT-0005-FR-009.
    let limits = CodecLimits::new(4, 64, 100_000).expect("depth is within the model");
    let payload = vec![0xde, 0xad, 0xbe, 0xef];
    let input = item(Value::byte_string(payload));
    let copy_observer = EncodingObserver::default();
    let result = encode_item_with_limits(&input, &limits, &copy_observer);
    let Err(error) = result else {
        panic!("oversized input was accepted");
    };
    let formatted = format!("{error:?}");

    assert!(!formatted.contains("deadbeef"));
    assert!(!formatted.contains("[222, 173, 190, 239]"));
}

#[test]
fn owner_drop_zeroizes_initialized_bytes_before_backing_allocation_deallocation() {
    // Project-only owner policy under KMIPKIT-0005-FR-013. The observer reads
    // only the initialized slice from Drop, before Vec deallocation; it never
    // uses unsafe code or inspects freed memory.
    let (owner, observer) = EncodedOwner::with_observer(vec![0xa5, 0x5a, 0xc3, 0x3c]);

    drop(owner);

    let (observed_length, all_zero) = observer.result().expect("Drop reports initialized bytes");

    assert!(observed_length > 0, "observer must see initialized bytes");
    assert!(
        all_zero,
        "initialized bytes must be zeroized before deallocation"
    );
}

const GENERATED_ROUNDTRIP_CASES: usize = 88;
const GENERATED_MAX_STRUCTURE_DEPTH: usize = 4;
const GENERATED_MAX_CHILDREN: usize = 4;
const GENERATED_MAX_PAYLOAD_BYTES: usize = 12;

struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.state = value;
        value
    }

    fn bounded(&mut self, exclusive_upper_bound: usize) -> usize {
        assert!(
            exclusive_upper_bound > 0,
            "generator bounds must be nonzero"
        );
        let bound = u64::try_from(exclusive_upper_bound).expect("small generator bound fits u64");
        usize::try_from(self.next_u64() % bound).expect("bounded generator result fits usize")
    }

    fn next_i32(&mut self) -> i32 {
        let bytes = self.next_u64().to_be_bytes();
        i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    fn next_i64(&mut self) -> i64 {
        i64::from_be_bytes(self.next_u64().to_be_bytes())
    }

    fn next_u32(&mut self) -> u32 {
        let bytes = self.next_u64().to_be_bytes();
        u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]])
    }

    fn next_byte(&mut self) -> u8 {
        self.next_u64().to_be_bytes()[7]
    }
}

fn generated_bytes(rng: &mut DeterministicRng, length: usize) -> Vec<u8> {
    (0..length).map(|_| rng.next_byte()).collect()
}

fn generated_item(rng: &mut DeterministicRng, structure_depth: usize, type_index: usize) -> Item {
    let value = match type_index {
        0 => {
            let mut structure = Structure::new();
            let child_count = 2 + rng.bounded(GENERATED_MAX_CHILDREN - 1);
            let first_leaf_type = rng.bounded(10);
            for child_index in 0..child_count {
                let child_type =
                    if child_index == 0 && structure_depth + 1 < GENERATED_MAX_STRUCTURE_DEPTH {
                        0
                    } else {
                        1 + ((first_leaf_type + child_index) % 10)
                    };
                structure
                    .try_push(generated_item(rng, structure_depth + 1, child_type))
                    .expect("bounded generated Structure fits the model depth");
            }
            Value::structure(structure)
        }
        1 => Value::integer(rng.next_i32()),
        2 => Value::long_integer(rng.next_i64()),
        3 => {
            // OASIS KMIP Specification v2.1 §10.1.2, KMIPKIT-0005-NR-003;
            // KMIPKIT-0005-FR-002 excludes empty project test values.
            let length = 1 + rng.bounded(GENERATED_MAX_PAYLOAD_BYTES);
            Value::big_integer(generated_bytes(rng, length))
        }
        4 => Value::enumeration(rng.next_u32()),
        5 => Value::boolean(rng.bounded(2) == 1),
        6 => {
            const TEXT_ALPHABET: [char; 5] = ['a', 'Z', 'é', 'Ω', '中'];
            let max_bytes = rng.bounded(GENERATED_MAX_PAYLOAD_BYTES + 1);
            let mut text = String::new();
            while text.len() < max_bytes {
                let character = TEXT_ALPHABET[rng.bounded(TEXT_ALPHABET.len())];
                if text.len() + character.len_utf8() > max_bytes {
                    break;
                }
                text.push(character);
            }
            Value::text_string(text)
        }
        7 => {
            let length = rng.bounded(GENERATED_MAX_PAYLOAD_BYTES + 1);
            Value::byte_string(generated_bytes(rng, length))
        }
        8 => Value::date_time(rng.next_i64()),
        9 => Value::interval(rng.next_u32()),
        10 => Value::date_time_extended(rng.next_i64()),
        _ => panic!("generated Item Type index is outside the eleven supported types"),
    };
    item(value)
}

fn canonicalized_item(item: &Item) -> Item {
    let value = item.with_value(|view| match view {
        kmipkit_ttlv::ValueView::Structure(structure_view) => {
            let mut structure = Structure::new();
            for child in structure_view.children() {
                structure
                    .try_push(canonicalized_item(child))
                    .expect("bounded expected Structure fits the model depth");
            }
            Value::structure(structure)
        }
        kmipkit_ttlv::ValueView::Integer(value) => Value::integer(*value),
        kmipkit_ttlv::ValueView::LongInteger(value) => Value::long_integer(*value),
        kmipkit_ttlv::ValueView::BigInteger(value) => {
            Value::big_integer(canonical_big_integer_octets(value))
        }
        kmipkit_ttlv::ValueView::Enumeration(value) => Value::enumeration(*value),
        kmipkit_ttlv::ValueView::Boolean(value) => Value::boolean(*value),
        kmipkit_ttlv::ValueView::TextString(value) => Value::text_string(value.to_owned()),
        kmipkit_ttlv::ValueView::ByteString(value) => Value::byte_string(value.to_vec()),
        kmipkit_ttlv::ValueView::DateTime(value) => Value::date_time(*value),
        kmipkit_ttlv::ValueView::Interval(value) => Value::interval(*value),
        kmipkit_ttlv::ValueView::DateTimeExtended(value) => Value::date_time_extended(*value),
        _ => panic!("generic model exposed an unsupported generated value"),
    });
    item_with_tag(item.tag(), value)
}

fn canonical_big_integer_octets(value: &[u8]) -> Vec<u8> {
    assert!(
        !value.is_empty(),
        "generated Big Integer values are nonempty"
    );
    let padding_length = (8 - (value.len() % 8)) % 8;
    if padding_length == 0 {
        return value.to_vec();
    }

    let sign_extension = if value[0] & 0x80 == 0 { 0x00 } else { 0xff };
    let mut canonical = Vec::with_capacity(value.len() + padding_length);
    canonical.resize(padding_length, sign_extension);
    canonical.extend_from_slice(value);
    canonical
}

fn assert_items_equal(actual: &Item, expected: &Item) {
    assert!(
        actual.tag() == expected.tag(),
        "decoder changed an Item Tag"
    );
    assert!(
        actual.item_type() == expected.item_type(),
        "decoder changed an Item Type"
    );
    actual.with_value(|actual_view| {
        expected.with_value(|expected_view| match (actual_view, expected_view) {
            (
                kmipkit_ttlv::ValueView::Structure(actual),
                kmipkit_ttlv::ValueView::Structure(expected),
            ) => {
                let actual_children = actual.children();
                let expected_children = expected.children();
                assert_eq!(
                    actual_children.len(),
                    expected_children.len(),
                    "decoder changed the Structure child count"
                );
                for (index, (actual_child, expected_child)) in
                    actual_children.iter().zip(expected_children).enumerate()
                {
                    assert!(
                        actual_child.tag() == expected_child.tag(),
                        "decoder changed the Tag at child index {index}"
                    );
                    assert_items_equal(actual_child, expected_child);
                }
            }
            (
                kmipkit_ttlv::ValueView::Integer(actual),
                kmipkit_ttlv::ValueView::Integer(expected),
            ) => assert!(actual == expected, "Integer value changed"),
            (
                kmipkit_ttlv::ValueView::LongInteger(actual),
                kmipkit_ttlv::ValueView::LongInteger(expected),
            ) => assert!(actual == expected, "Long Integer value changed"),
            (
                kmipkit_ttlv::ValueView::BigInteger(actual),
                kmipkit_ttlv::ValueView::BigInteger(expected),
            ) => assert!(actual == expected, "Big Integer octets changed"),
            (
                kmipkit_ttlv::ValueView::Enumeration(actual),
                kmipkit_ttlv::ValueView::Enumeration(expected),
            ) => assert!(actual == expected, "Enumeration value changed"),
            (
                kmipkit_ttlv::ValueView::Boolean(actual),
                kmipkit_ttlv::ValueView::Boolean(expected),
            ) => assert!(actual == expected, "Boolean value changed"),
            (
                kmipkit_ttlv::ValueView::TextString(actual),
                kmipkit_ttlv::ValueView::TextString(expected),
            ) => assert!(actual == expected, "Text String value changed"),
            (
                kmipkit_ttlv::ValueView::ByteString(actual),
                kmipkit_ttlv::ValueView::ByteString(expected),
            ) => assert!(actual == expected, "Byte String octets changed"),
            (
                kmipkit_ttlv::ValueView::DateTime(actual),
                kmipkit_ttlv::ValueView::DateTime(expected),
            ) => assert!(actual == expected, "Date Time value changed"),
            (
                kmipkit_ttlv::ValueView::Interval(actual),
                kmipkit_ttlv::ValueView::Interval(expected),
            ) => assert!(actual == expected, "Interval value changed"),
            (
                kmipkit_ttlv::ValueView::DateTimeExtended(actual),
                kmipkit_ttlv::ValueView::DateTimeExtended(expected),
            ) => assert!(actual == expected, "Date Time Extended value changed"),
            _ => panic!("decoder changed a generated Item Type"),
        });
    });
}

#[test]
fn deterministic_generated_models_round_trip_through_private_writer_and_public_decoder() {
    // OASIS KMIP Specification v2.1 §§10.1.1–10.1.5 and §11.23;
    // KMIPKIT-0005-NR-001, KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-003,
    // KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005.
    // Stable records: KMIPKIT-REQ-SPEC-10.1.2-002-001,
    // KMIPKIT-REQ-SPEC-10.1.2-002-002, KMIPKIT-REQ-SPEC-10.1.5-001-001,
    // and KMIPKIT-REQ-SPEC-10.1.5-001-002.
    // Structure order is KMIPKit's generic-model policy under
    // KMIPKIT-0005-FR-003; this does not assert schema-defined order.
    // Eight fixed-seed roots cover each of the eleven represented types.
    // The local xorshift generator and all tree/payload limits are fixed.
    let mut rng = DeterministicRng::new(0x6d69_706b_6974_0005);

    for sample in 0..GENERATED_ROUNDTRIP_CASES {
        let root_type = sample % 11;
        let source = generated_item(&mut rng, 0, root_type);
        let expected = canonicalized_item(&source);
        let encoded = encode_item(&source).expect("bounded generated model is encodable");
        let decoded = kmipkit_ttlv::codec::decode(&encoded)
            .expect("private writer output is accepted by the public decoder");

        assert_items_equal(&decoded, &expected);
    }
}

#[test]
fn big_integer_round_trips_sign_extension_and_aligned_octets_exactly() {
    // OASIS KMIP Specification v2.1 §§10.1.2 and 10.1.3;
    // KMIPKIT-0005-NR-003 and KMIPKIT-0005-NR-004;
    // KMIPKIT-REQ-SPEC-10.1.2-002-001 and KMIPKIT-REQ-SPEC-10.1.2-002-002
    // require minimum leading two's-complement sign extension to an eight-byte
    // Item Value. Aligned octets remain exact. Empty-value rejection is KMIPKit's
    // project rule under KMIPKIT-0005-FR-002, not an OASIS minimum-length rule.
    let cases = [
        (
            vec![0x7f, 0x00],
            vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7f, 0x00],
        ),
        (
            vec![0x80, 0x01],
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x80, 0x01],
        ),
        (
            vec![0x80, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
            vec![0x80, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06],
        ),
    ];

    for (input_octets, expected_octets) in cases {
        let source = item(Value::big_integer(input_octets));
        let expected = item(Value::big_integer(expected_octets));
        let encoded = encode_item(&source).expect("nonempty Big Integer is encodable");
        let decoded = kmipkit_ttlv::codec::decode(&encoded)
            .expect("Big Integer writer output is accepted by the public decoder");

        assert_items_equal(&decoded, &expected);
    }
}
