//! Red-only private wire-encoder test harness.
//!
//! This module deliberately contains test fixtures and behaviorally incomplete
//! stubs only. T003 replaces the stubs with the private production writer.

use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};
use static_assertions::assert_not_impl_any;

const TEST_TAG_RAW: u32 = 0x0042_0173;
const DEFAULT_MAX_MESSAGE_BYTES: u64 = 16 * 1024 * 1024;
const MODEL_MAX_STRUCTURE_DEPTH: u64 = 64;
const DEFAULT_MAX_ELEMENTS: u64 = 100_000;

// All golden vectors use this allocated Tag. OASIS KMIP Specification v2.1
// §11.56, KMIPKIT-REQ-SPEC-11.56-001: encoded Tags use a 0x42 or 0x54 prefix.
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
enum EncodeError {
    EmptyBigInteger,
    LimitExceeded,
    ItemLengthOverflow,
}

// Deliberately incomplete Red stub: it compiles without declaring a
// production encoder and fails the golden-vector behavior assertions.
fn encode_item(_item: &Item) -> Result<Vec<u8>, EncodeError> {
    Ok(Vec::new())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LimitsView {
    max_message_bytes: u64,
    max_structure_depth: u64,
    max_elements: u64,
}

impl LimitsView {
    const fn new(max_message_bytes: u64, max_structure_depth: u64, max_elements: u64) -> Self {
        Self {
            max_message_bytes,
            max_structure_depth,
            max_elements,
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
}

// Private borrowed-limits seam with no payload or tree allocation. The
// permissive stub lets the tests compile before the production planner exists.
fn check_plan(_plan: &SyntheticPlan, _limits: &LimitsView) -> Result<(), EncodeError> {
    Ok(())
}

fn plan_item_length(_value_length: u64) -> Result<u32, EncodeError> {
    Ok(0)
}

#[derive(Default)]
struct PayloadCopyObserver {
    calls: Cell<usize>,
}

impl PayloadCopyObserver {
    fn calls(&self) -> usize {
        self.calls.get()
    }
}

// This is the sole test-only payload-copy boundary. The bad stub below calls
// it before rejecting, so the Red case fails behaviorally on the observer.
fn shared_payload_copy(payload: &[u8], observer: &PayloadCopyObserver) -> Vec<u8> {
    observer.calls.set(observer.calls.get() + 1);
    payload.to_vec()
}

fn rejected_attempt(
    plan: &SyntheticPlan,
    limits: &LimitsView,
    payload: &[u8],
    observer: &PayloadCopyObserver,
) -> Result<(), EncodeError> {
    let _copied_payload = shared_payload_copy(payload, observer);

    if plan.encoded_bytes > limits.max_message_bytes {
        Err(EncodeError::LimitExceeded)
    } else {
        Ok(())
    }
}

#[derive(Clone)]
struct ZeroizationObserver(Rc<Cell<Option<bool>>>);

impl ZeroizationObserver {
    fn observe_before_deallocation(&self, initialized_bytes: &[u8]) {
        self.0
            .set(Some(initialized_bytes.iter().all(|byte| *byte == 0)));
    }

    fn result(&self) -> Option<bool> {
        self.0.get()
    }
}

struct EncodedOwnerStub {
    bytes: Vec<u8>,
    observer: ZeroizationObserver,
}

impl EncodedOwnerStub {
    fn new(bytes: Vec<u8>) -> (Self, ZeroizationObserver) {
        let observer = ZeroizationObserver(Rc::new(Cell::new(None)));
        (
            Self {
                bytes,
                observer: observer.clone(),
            },
            observer,
        )
    }
}

impl Drop for EncodedOwnerStub {
    fn drop(&mut self) {
        // The Red stub intentionally omits the clear. This is the production
        // observer point after the owner must zeroize and before Vec
        // deallocation; the observer safely sees initialized bytes only.
        self.observer.observe_before_deallocation(&self.bytes);
    }
}

assert_not_impl_any!(
    EncodedOwnerStub:
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
    // Project-only vector under KMIPKIT-0005-FR-002. OASIS KMIP Specification
    // v2.1 §§10.1.2 and 11.23 describe Structure's Item Type, but no directly
    // applicable stable catalog requirement is assigned to this exact vector.
    let encoded = encode_item(&item(Value::structure(Structure::new())))
        .expect("an empty Structure is encodable");

    assert_eq!(encoded, [0x42, 0x01, 0x73, 0x01, 0, 0, 0, 0]);
}

#[test]
fn encodes_integer_golden_vector_with_signed_big_endian_value() {
    // Project-only vector under KMIPKIT-0005-FR-002. OASIS KMIP Specification
    // v2.1 §10.1.2 defines Integer representation; no directly applicable
    // stable catalog requirement is assigned to this exact value vector.
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
fn encodes_long_integer_golden_vector_with_signed_big_endian_value() {
    // Project-only vector under KMIPKIT-0005-FR-002. OASIS KMIP Specification
    // v2.1 §10.1.2 defines Long Integer representation; no directly applicable
    // stable catalog requirement is assigned to this exact value vector.
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
    // OASIS KMIP Specification v2.1 §10.1.2, KMIPKIT-REQ-SPEC-10.1.2-002-001
    // and KMIPKIT-REQ-SPEC-10.1.2-002-002: prepend minimum sign-extension
    // bytes to an eight-byte boundary and include them in Item Length.
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
    // Project-only rejection under KMIPKIT-0005-FR-002. OASIS KMIP
    // Specification v2.1 §10.1.2 does not state an explicit minimum length.
    let result = encode_item(&item(Value::big_integer(Vec::new())));

    assert_eq!(result, Err(EncodeError::EmptyBigInteger));
}

#[test]
fn encodes_enumeration_golden_vector_with_unsigned_big_endian_value() {
    // Project-only vector under KMIPKIT-0005-FR-002. OASIS KMIP Specification
    // v2.1 §10.1.2 defines Enumeration representation; no directly applicable
    // stable catalog requirement is assigned to this exact value vector.
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
    // Project-only vector under KMIPKIT-0005-FR-002. OASIS KMIP Specification
    // v2.1 §10.1.2 defines Boolean representation; no directly applicable
    // stable catalog requirement is assigned to this exact value vector.
    let encoded = encode_item(&item(Value::boolean(true))).expect("Boolean vector is encodable");

    assert_eq!(
        encoded,
        [0x42, 0x01, 0x73, 0x06, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 1]
    );
}

#[test]
fn encodes_text_string_golden_vector_with_minimum_following_padding() {
    // OASIS KMIP Specification v2.1 §10.1.5,
    // KMIPKIT-REQ-SPEC-10.1.5-001-001: Text String padding is the minimum
    // number of following bytes needed to reach an eight-byte value boundary.
    // The exact UTF-8 bytes and zero fill are KMIPKit's FR-002 canonical vector.
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
    // OASIS KMIP Specification v2.1 §10.1.5,
    // KMIPKIT-REQ-SPEC-10.1.5-001-001: Byte String padding is the minimum
    // number of following bytes needed to reach an eight-byte value boundary.
    // The exact payload and zero fill are KMIPKit's FR-002 canonical vector.
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
    // Project-only vector under KMIPKIT-0005-FR-002. OASIS KMIP Specification
    // v2.1 §10.1.2 defines Date Time representation; no directly applicable
    // stable catalog requirement is assigned to this exact value vector.
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
    // OASIS KMIP Specification v2.1 §10.1.5,
    // KMIPKIT-REQ-SPEC-10.1.5-001-002: Interval is followed by four padding
    // bytes. The exact value vector is also covered under KMIPKIT-0005-FR-002.
    let encoded =
        encode_item(&item(Value::interval(0x0102_0304))).expect("Interval vector is encodable");

    assert_eq!(
        encoded,
        [0x42, 0x01, 0x73, 0x0a, 0, 0, 0, 4, 1, 2, 3, 4, 0, 0, 0, 0,]
    );
}

#[test]
fn encodes_date_time_extended_golden_vector_with_signed_big_endian_value() {
    // Project-only vector under KMIPKIT-0005-FR-002. OASIS KMIP Specification
    // v2.1 §10.1.2 defines Date Time Extended representation; no directly
    // applicable stable catalog requirement is assigned to this exact vector.
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
    // OASIS KMIP Specification v2.1 §10.1.5,
    // KMIPKIT-REQ-SPEC-10.1.5-001-002: Integer, Enumeration, and Interval
    // receive four following padding bytes; this checks all three families.
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
    // OASIS KMIP Specification v2.1 §10.1.5,
    // KMIPKIT-REQ-SPEC-10.1.5-001-001: string and byte padding reaches the
    // next eight-byte boundary with the minimum following bytes. This bounded
    // table covers empty, one-byte, seven-byte, and aligned values.
    for length in [0_usize, 1, 7, 8] {
        let text = "a".repeat(length);
        let text_value = text.as_bytes();
        let text_padding = (8 - (length % 8)) % 8;
        let mut text_value_and_padding = text_value.to_vec();
        text_value_and_padding.resize(length + text_padding, 0);
        let text_expected = expected_item(0x07, length as u32, &text_value_and_padding);
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
        let byte_expected = expected_item(0x08, length as u32, &byte_value_and_padding);
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
    // OASIS KMIP Specification v2.1 §10.1.2,
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
    // Project-only generic-tree behavior under KMIPKIT-0005-FR-003. This does
    // not claim OASIS schema field-order conformance for any Structure.
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

    assert_eq!(check_plan(&exact, &limits), Ok(()));
    assert_eq!(check_plan(&over, &limits), Err(EncodeError::LimitExceeded));
}

#[test]
fn configured_lowered_byte_limit_accepts_exact_boundary_and_rejects_one_over() {
    // Project-only configured per-call limit behavior under
    // KMIPKIT-0005-FR-007. No payload or output allocation is made.
    let limits = LimitsView::new(64, MODEL_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
    let exact = SyntheticPlan::new(64, 1, 1);
    let over = SyntheticPlan::new(65, 1, 1);

    assert_eq!(check_plan(&exact, &limits), Ok(()));
    assert_eq!(check_plan(&over, &limits), Err(EncodeError::LimitExceeded));
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

    assert_eq!(check_plan(&above_default, &limits), Ok(()));
}

#[test]
fn default_depth_limit_accepts_64_and_rejects_synthetic_depth_65() {
    // Project-only resource policy under KMIPKIT-0005-FR-007. The depth-65
    // value is a planning fixture; no nested model tree is constructed.
    let limits = LimitsView::defaults();
    let exact = SyntheticPlan::new(8, MODEL_MAX_STRUCTURE_DEPTH, 1);
    let over = SyntheticPlan::new(8, MODEL_MAX_STRUCTURE_DEPTH + 1, 1);

    assert_eq!(check_plan(&exact, &limits), Ok(()));
    assert_eq!(check_plan(&over, &limits), Err(EncodeError::LimitExceeded));
}

#[test]
fn configured_lowered_depth_limit_accepts_exact_boundary_and_rejects_one_over() {
    // Project-only configured per-call depth behavior under
    // KMIPKIT-0005-FR-007. Synthetic plans avoid deep recursive trees.
    let limits = LimitsView::new(DEFAULT_MAX_MESSAGE_BYTES, 12, DEFAULT_MAX_ELEMENTS);
    let exact = SyntheticPlan::new(8, 12, 1);
    let over = SyntheticPlan::new(8, 13, 1);

    assert_eq!(check_plan(&exact, &limits), Ok(()));
    assert_eq!(check_plan(&over, &limits), Err(EncodeError::LimitExceeded));
}

#[test]
fn configured_depth_cannot_raise_the_model_hard_maximum() {
    // Project-only model-bound policy under KMIPKIT-0005-FR-007. Depth 65 is
    // synthetic and no Structure allocation is performed.
    let limits = LimitsView::new(DEFAULT_MAX_MESSAGE_BYTES, 65, DEFAULT_MAX_ELEMENTS);
    let plan = SyntheticPlan::new(8, 65, 1);

    assert_eq!(check_plan(&plan, &limits), Err(EncodeError::LimitExceeded));
}

#[test]
fn default_element_limit_accepts_exact_boundary_and_rejects_100001() {
    // Project-only resource policy under KMIPKIT-0005-FR-007. Element counts
    // are synthetic; this test creates no 100,001-item Structure.
    let limits = LimitsView::defaults();
    let exact = SyntheticPlan::new(8, 1, DEFAULT_MAX_ELEMENTS);
    let over = SyntheticPlan::new(8, 1, DEFAULT_MAX_ELEMENTS + 1);

    assert_eq!(check_plan(&exact, &limits), Ok(()));
    assert_eq!(check_plan(&over, &limits), Err(EncodeError::LimitExceeded));
}

#[test]
fn configured_lowered_element_limit_accepts_exact_boundary_and_rejects_one_over() {
    // Project-only configured per-call count behavior under
    // KMIPKIT-0005-FR-007. Element counts are synthetic.
    let limits = LimitsView::new(DEFAULT_MAX_MESSAGE_BYTES, MODEL_MAX_STRUCTURE_DEPTH, 24);
    let exact = SyntheticPlan::new(8, 1, 24);
    let over = SyntheticPlan::new(8, 1, 25);

    assert_eq!(check_plan(&exact, &limits), Ok(()));
    assert_eq!(check_plan(&over, &limits), Err(EncodeError::LimitExceeded));
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

    assert_eq!(check_plan(&above_default, &limits), Ok(()));
}

#[test]
fn borrowed_per_call_limits_do_not_share_mutable_state() {
    // Project-only per-call behavior under KMIPKIT-0005-FR-007. Both views
    // remain immutable and the plan is reused without global state.
    let restrictive = LimitsView::new(32, 8, 10);
    let permissive = LimitsView::new(128, 8, 10);
    let plan = SyntheticPlan::new(64, 4, 5);

    assert_eq!(check_plan(&plan, &permissive), Ok(()));
    assert_eq!(
        check_plan(&plan, &restrictive),
        Err(EncodeError::LimitExceeded)
    );
}

#[test]
fn item_length_planner_accepts_u32_max_without_allocating() {
    // Project-only arithmetic boundary under KMIPKIT-0005-FR-007, informed by
    // OASIS KMIP Specification v2.1 §10.1.3. The value is never materialized.
    assert_eq!(plan_item_length(u64::from(u32::MAX)), Ok(u32::MAX));
}

#[test]
fn item_length_planner_rejects_u32_max_plus_one_without_allocating() {
    // Project-only arithmetic boundary under KMIPKIT-0005-FR-007, informed by
    // OASIS KMIP Specification v2.1 §10.1.3. No oversized buffer is created.
    assert_eq!(
        plan_item_length(u64::from(u32::MAX) + 1),
        Err(EncodeError::ItemLengthOverflow)
    );
}

#[test]
fn preflight_rejection_performs_zero_payload_copies() {
    // Project-only security/error behavior under KMIPKIT-0005-FR-009. The
    // observer sits at the shared payload-copy helper; the bounded fixture is
    // rejected by the byte limit and must not be copied first.
    let limits = LimitsView::new(4, MODEL_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
    let oversized = SyntheticPlan::new(5, 1, 1);
    let copy_observer = PayloadCopyObserver::default();
    let result = rejected_attempt(&oversized, &limits, &[0xa5, 0x5a, 0xa5], &copy_observer);

    assert_eq!(result, Err(EncodeError::LimitExceeded));
    assert_eq!(copy_observer.calls(), 0);
}

#[test]
fn preflight_error_does_not_format_payload_bytes() {
    // Project-only payload-redaction behavior under KMIPKIT-0005-FR-009.
    let limits = LimitsView::new(4, MODEL_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
    let oversized = SyntheticPlan::new(5, 1, 1);
    let payload = [0xde, 0xad, 0xbe, 0xef];
    let copy_observer = PayloadCopyObserver::default();
    let error = rejected_attempt(&oversized, &limits, &payload, &copy_observer)
        .expect_err("oversized synthetic plan is rejected");
    let formatted = format!("{error:?}");

    assert!(!formatted.contains("deadbeef"));
    assert!(!formatted.contains("[222, 173, 190, 239]"));
}

#[test]
fn owner_drop_zeroizes_initialized_bytes_before_backing_allocation_deallocation() {
    // Project-only owner policy under KMIPKIT-0005-FR-013. The observer reads
    // only the initialized slice from Drop, before Vec deallocation; it never
    // uses unsafe code or inspects freed memory.
    let (owner, observer) = EncodedOwnerStub::new(vec![0xa5, 0x5a, 0xc3, 0x3c]);

    drop(owner);

    assert_eq!(observer.result(), Some(true));
}
