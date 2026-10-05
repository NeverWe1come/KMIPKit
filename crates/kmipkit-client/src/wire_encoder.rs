//! Private outbound TTLV encoding for typed KMIP items.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, ItemType, ValueView};
#[cfg(test)]
use zeroize::Zeroize;
use zeroize::Zeroizing;

const HEADER_BYTES: u64 = 8;
const PADDING_BYTES: u64 = 8;
const DEFAULT_MAX_MESSAGE_BYTES: u64 = 16 * 1024 * 1024;
const MODEL_MAX_STRUCTURE_DEPTH: u64 = 64;
const DEFAULT_MAX_ELEMENTS: u64 = 100_000;

/// Limits are read through a borrowed private seam, so a future public
/// configuration value can be used without cloning or rebuilding settings.
trait BorrowedLimitsView {
    fn max_message_bytes(&self) -> u64;
    fn max_structure_depth(&self) -> u64;
    fn max_elements(&self) -> u64;
}

struct DefaultLimits;

impl BorrowedLimitsView for DefaultLimits {
    fn max_message_bytes(&self) -> u64 {
        DEFAULT_MAX_MESSAGE_BYTES
    }

    fn max_structure_depth(&self) -> u64 {
        MODEL_MAX_STRUCTURE_DEPTH
    }

    fn max_elements(&self) -> u64 {
        DEFAULT_MAX_ELEMENTS
    }
}

static DEFAULT_LIMITS: DefaultLimits = DefaultLimits;

enum EncodeError {
    EmptyBigInteger,
    LimitExceeded,
    ItemLengthOverflow,
    SizeOverflow,
    AllocationFailed(std::collections::TryReserveError),
    UnsupportedValueType,
}

impl PartialEq for EncodeError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::EmptyBigInteger, Self::EmptyBigInteger)
                | (Self::LimitExceeded, Self::LimitExceeded)
                | (Self::ItemLengthOverflow, Self::ItemLengthOverflow)
                | (Self::SizeOverflow, Self::SizeOverflow)
                | (Self::AllocationFailed(_), Self::AllocationFailed(_))
                | (Self::UnsupportedValueType, Self::UnsupportedValueType)
        )
    }
}

impl Eq for EncodeError {}

impl fmt::Debug for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant = match self {
            Self::EmptyBigInteger => "EmptyBigInteger",
            Self::LimitExceeded => "LimitExceeded",
            Self::ItemLengthOverflow => "ItemLengthOverflow",
            Self::SizeOverflow => "SizeOverflow",
            Self::AllocationFailed(_) => "AllocationFailed",
            Self::UnsupportedValueType => "UnsupportedValueType",
        };
        formatter.write_str(variant)
    }
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBigInteger => formatter.write_str("cannot encode an empty Big Integer"),
            Self::LimitExceeded => formatter.write_str("outbound TTLV limits exceeded"),
            Self::ItemLengthOverflow => formatter.write_str("TTLV Item Length exceeds u32"),
            Self::SizeOverflow => formatter.write_str("TTLV encoded size overflowed"),
            Self::AllocationFailed(_) => {
                formatter.write_str("unable to reserve TTLV output buffer")
            }
            Self::UnsupportedValueType => formatter.write_str("unsupported TTLV Item Type"),
        }
    }
}

impl Error for EncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AllocationFailed(source) => Some(source),
            Self::EmptyBigInteger
            | Self::LimitExceeded
            | Self::ItemLengthOverflow
            | Self::SizeOverflow
            | Self::UnsupportedValueType => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EncodingPlan {
    encoded_bytes: u64,
    structure_depth: u64,
    elements: u64,
}

impl EncodingPlan {
    fn for_item(item: &Item) -> Result<Self, EncodeError> {
        let mut counts = PlanCounts::default();
        let encoded_bytes = measure_item(item, 0, &mut counts)?;

        Ok(Self {
            encoded_bytes,
            structure_depth: counts.structure_depth,
            elements: counts.elements,
        })
    }

    #[cfg(test)]
    const fn new(encoded_bytes: u64, structure_depth: u64, elements: u64) -> Self {
        Self {
            encoded_bytes,
            structure_depth,
            elements,
        }
    }
}

#[derive(Default)]
struct PlanCounts {
    structure_depth: u64,
    elements: u64,
}

fn measure_item(
    item: &Item,
    parent_structure_depth: u64,
    counts: &mut PlanCounts,
) -> Result<u64, EncodeError> {
    counts.elements = counts
        .elements
        .checked_add(1)
        .ok_or(EncodeError::SizeOverflow)?;

    let lengths = item.with_value(|value| measure_value(value, parent_structure_depth, counts))?;
    lengths.encoded_item_bytes()
}

#[derive(Clone, Copy)]
struct ValueLengths {
    item_length: u64,
    encoded_value_bytes: u64,
}

impl ValueLengths {
    const fn new(item_length: u64, encoded_value_bytes: u64) -> Self {
        Self {
            item_length,
            encoded_value_bytes,
        }
    }

    /// Checks the wire Item Length and adds the fixed TTLV header size.
    fn encoded_item_bytes(self) -> Result<u64, EncodeError> {
        plan_item_length(self.item_length)?;
        HEADER_BYTES
            .checked_add(self.encoded_value_bytes)
            .ok_or(EncodeError::SizeOverflow)
    }
}

/// Plans the type-specific Item Length and bytes emitted after the header.
fn measure_value(
    value: ValueView<'_>,
    parent_structure_depth: u64,
    counts: &mut PlanCounts,
) -> Result<ValueLengths, EncodeError> {
    match value {
        ValueView::Structure(structure) => {
            measure_structure(structure.children(), parent_structure_depth, counts)
        }
        ValueView::Integer(_) => Ok(measure_integer()),
        ValueView::LongInteger(_) => Ok(measure_long_integer()),
        ValueView::BigInteger(payload) => measure_big_integer(payload),
        ValueView::Enumeration(_) => Ok(measure_enumeration()),
        ValueView::Boolean(_) => Ok(measure_boolean()),
        ValueView::TextString(payload) => measure_text_string(payload),
        ValueView::ByteString(payload) => measure_byte_string(payload),
        ValueView::DateTime(_) => Ok(measure_date_time()),
        ValueView::Interval(_) => Ok(measure_interval()),
        ValueView::DateTimeExtended(_) => Ok(measure_date_time_extended()),
        _ => Err(EncodeError::UnsupportedValueType),
    }
}

/// Includes every child's full encoding, in the model's original order.
fn measure_structure(
    children: &[Item],
    parent_structure_depth: u64,
    counts: &mut PlanCounts,
) -> Result<ValueLengths, EncodeError> {
    let depth = parent_structure_depth
        .checked_add(1)
        .ok_or(EncodeError::SizeOverflow)?;
    counts.structure_depth = counts.structure_depth.max(depth);

    let mut children_length = 0_u64;
    for child in children {
        let child_length = measure_item(child, depth, counts)?;
        children_length = children_length
            .checked_add(child_length)
            .ok_or(EncodeError::SizeOverflow)?;
    }

    Ok(ValueLengths::new(children_length, children_length))
}

/// Plans a signed 32-bit value followed by four padding bytes.
fn measure_integer() -> ValueLengths {
    ValueLengths::new(4, 8)
}

/// Plans a signed 64-bit value with no additional padding.
fn measure_long_integer() -> ValueLengths {
    ValueLengths::new(8, 8)
}

/// Includes Big Integer sign-extension padding in both lengths.
fn measure_big_integer(payload: &[u8]) -> Result<ValueLengths, EncodeError> {
    if payload.is_empty() {
        return Err(EncodeError::EmptyBigInteger);
    }

    let payload_length = u64::try_from(payload.len()).map_err(|_| EncodeError::SizeOverflow)?;
    let padded_value_length = padded_length(payload_length)?;
    Ok(ValueLengths::new(padded_value_length, padded_value_length))
}

/// Plans an unsigned 32-bit Enumeration followed by four padding bytes.
fn measure_enumeration() -> ValueLengths {
    ValueLengths::new(4, 8)
}

/// Plans the exact eight-byte Boolean representation.
fn measure_boolean() -> ValueLengths {
    ValueLengths::new(8, 8)
}

/// Excludes minimal trailing padding from the Text String Item Length.
fn measure_text_string(payload: &str) -> Result<ValueLengths, EncodeError> {
    measure_padded_string(payload.len())
}

/// Excludes minimal trailing padding from the Byte String Item Length.
fn measure_byte_string(payload: &[u8]) -> Result<ValueLengths, EncodeError> {
    measure_padded_string(payload.len())
}

/// Plans the common value-length and minimal eight-byte padding rule for strings.
fn measure_padded_string(payload_length: usize) -> Result<ValueLengths, EncodeError> {
    let item_length = u64::try_from(payload_length).map_err(|_| EncodeError::SizeOverflow)?;
    Ok(ValueLengths::new(item_length, padded_length(item_length)?))
}

/// Plans a signed 64-bit Date Time value with no additional padding.
fn measure_date_time() -> ValueLengths {
    ValueLengths::new(8, 8)
}

/// Plans an unsigned 32-bit Interval followed by four padding bytes.
fn measure_interval() -> ValueLengths {
    ValueLengths::new(4, 8)
}

/// Plans a signed 64-bit Date Time Extended value with no additional padding.
fn measure_date_time_extended() -> ValueLengths {
    ValueLengths::new(8, 8)
}

/// Returns the minimum encoded length divisible by the TTLV padding quantum.
fn padded_length(value_length: u64) -> Result<u64, EncodeError> {
    let remainder = value_length % PADDING_BYTES;
    if remainder == 0 {
        Ok(value_length)
    } else {
        value_length
            .checked_add(PADDING_BYTES - remainder)
            .ok_or(EncodeError::SizeOverflow)
    }
}

fn plan_item_length(value_length: u64) -> Result<u32, EncodeError> {
    u32::try_from(value_length).map_err(|_| EncodeError::ItemLengthOverflow)
}

fn check_plan(plan: &EncodingPlan, limits: &impl BorrowedLimitsView) -> Result<(), EncodeError> {
    let max_depth = limits.max_structure_depth().min(MODEL_MAX_STRUCTURE_DEPTH);

    if plan.encoded_bytes > limits.max_message_bytes()
        || plan.structure_depth > max_depth
        || plan.elements > limits.max_elements()
    {
        return Err(EncodeError::LimitExceeded);
    }

    Ok(())
}

struct EncodedOwner {
    bytes: Zeroizing<Vec<u8>>,
    #[cfg(test)]
    drop_observer: Option<tests::ZeroizationObserver>,
}

impl EncodedOwner {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: Zeroizing::new(bytes),
            #[cfg(test)]
            drop_observer: None,
        }
    }

    /// Borrows the initialized output bytes immutably for an internal caller.
    fn as_bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    #[cfg(test)]
    fn with_observer(bytes: Vec<u8>) -> (Self, tests::ZeroizationObserver) {
        let observer = tests::ZeroizationObserver::new();
        (
            Self {
                bytes: Zeroizing::new(bytes),
                drop_observer: Some(observer.clone()),
            },
            observer,
        )
    }
}

#[cfg(test)]
impl Drop for EncodedOwner {
    fn drop(&mut self) {
        let bytes: &mut Vec<u8> = &mut self.bytes;
        bytes.zeroize();
        if let Some(observer) = &self.drop_observer {
            observer.observe_before_deallocation(bytes);
        }
    }
}

fn encode_item(item: &Item) -> Result<EncodedOwner, EncodeError> {
    #[cfg(test)]
    let observer = tests::PayloadCopyObserver::default();

    encode_item_with_limits(
        item,
        &DEFAULT_LIMITS,
        #[cfg(test)]
        &observer,
    )
}

fn encode_item_with_limits(
    item: &Item,
    limits: &impl BorrowedLimitsView,
    #[cfg(test)] observer: &tests::PayloadCopyObserver,
) -> Result<EncodedOwner, EncodeError> {
    let plan = EncodingPlan::for_item(item)?;
    check_plan(&plan, limits)?;
    let capacity = usize::try_from(plan.encoded_bytes).map_err(|_| EncodeError::SizeOverflow)?;

    let output = reserve_output_buffer(capacity)?;

    // Own and zeroize the reserved allocation before writing starts. If an
    // invariant is ever violated during writing, Drop still clears the bytes.
    let mut owner = EncodedOwner::new(output);
    {
        let mut writer = Writer {
            bytes: &mut owner.bytes,
            #[cfg(test)]
            observer,
        };
        writer.write_item(item);
    }

    Ok(owner)
}

fn reserve_output_buffer(capacity: usize) -> Result<Vec<u8>, EncodeError> {
    let mut output = Vec::new();
    output
        .try_reserve_exact(capacity)
        .map_err(EncodeError::AllocationFailed)?;
    Ok(output)
}

struct Writer<'a> {
    bytes: &'a mut Vec<u8>,
    #[cfg(test)]
    observer: &'a tests::PayloadCopyObserver,
}

impl Writer<'_> {
    /// Writes an item using only infallible appends after complete preflight.
    fn write_item(&mut self, item: &Item) {
        let raw_tag = item.tag().raw();
        self.bytes.extend_from_slice(&[
            ((raw_tag >> 16) & 0xff) as u8,
            ((raw_tag >> 8) & 0xff) as u8,
            (raw_tag & 0xff) as u8,
            type_code(item.item_type()),
        ]);
        let length_start = self.bytes.len();
        self.bytes.extend_from_slice(&[0; 4]);
        let item_length = item.with_value(|value| self.write_value(value));

        // Planning has checked the same immutable value tree and established
        // that every Item Length fits this field before any payload was copied.
        let item_length_bytes = item_length.to_be_bytes();
        self.bytes[length_start..length_start + 4].copy_from_slice(&item_length_bytes[4..]);
    }

    /// Writes the selected value representation and returns its unpadded Item Length.
    fn write_value(&mut self, value: ValueView<'_>) -> u64 {
        match value {
            ValueView::Structure(structure) => self.write_structure_value(structure.children()),
            ValueView::Integer(value) => self.write_integer_value(*value),
            ValueView::LongInteger(value) => self.write_long_integer_value(*value),
            ValueView::BigInteger(payload) => self.write_big_integer_value(payload),
            ValueView::Enumeration(value) => self.write_enumeration_value(*value),
            ValueView::Boolean(value) => self.write_boolean_value(*value),
            ValueView::TextString(payload) => self.write_text_string_value(payload),
            ValueView::ByteString(payload) => self.write_byte_string_value(payload),
            ValueView::DateTime(value) => self.write_date_time_value(*value),
            ValueView::Interval(value) => self.write_interval_value(*value),
            ValueView::DateTimeExtended(value) => self.write_date_time_extended_value(*value),
            _ => 0,
        }
    }

    /// Writes child items in their original order and measures the resulting value bytes.
    fn write_structure_value(&mut self, children: &[Item]) -> u64 {
        let value_start = self.bytes.len();
        for child in children {
            self.write_item(child);
        }
        (self.bytes.len() - value_start) as u64
    }

    /// Writes a signed 32-bit value and its four following padding bytes.
    fn write_integer_value(&mut self, value: i32) -> u64 {
        self.write_four_byte_value(value.to_be_bytes())
    }

    /// Writes a signed 64-bit value without additional padding.
    fn write_long_integer_value(&mut self, value: i64) -> u64 {
        self.write_eight_byte_value(value.to_be_bytes())
    }

    /// Writes sign extension before the exact Big Integer octets.
    fn write_big_integer_value(&mut self, payload: &[u8]) -> u64 {
        let padding = padding_count(payload.len());
        let extension = match payload.first() {
            Some(first) if first & 0x80 != 0 => 0xff,
            Some(_) | None => 0,
        };
        let item_length = (payload.len() + padding) as u64;
        self.append_sign_padding(extension, padding);
        self.append_payload(payload);
        item_length
    }

    /// Writes an unsigned 32-bit Enumeration and its four padding bytes.
    fn write_enumeration_value(&mut self, value: u32) -> u64 {
        self.write_four_byte_value(value.to_be_bytes())
    }

    /// Writes the Boolean's exact eight-byte value representation.
    fn write_boolean_value(&mut self, value: bool) -> u64 {
        self.bytes
            .extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, u8::from(value)]);
        8
    }

    /// Writes UTF-8 bytes and minimal trailing padding excluded from Item Length.
    fn write_text_string_value(&mut self, payload: &str) -> u64 {
        self.write_padded_bytes(payload.as_bytes())
    }

    /// Writes exact Byte String octets and minimal trailing padding.
    fn write_byte_string_value(&mut self, payload: &[u8]) -> u64 {
        self.write_padded_bytes(payload)
    }

    /// Writes a signed 64-bit Date Time value without additional padding.
    fn write_date_time_value(&mut self, value: i64) -> u64 {
        self.write_eight_byte_value(value.to_be_bytes())
    }

    /// Writes an unsigned 32-bit Interval and its four padding bytes.
    fn write_interval_value(&mut self, value: u32) -> u64 {
        self.write_four_byte_value(value.to_be_bytes())
    }

    /// Writes a signed 64-bit Date Time Extended value without additional padding.
    fn write_date_time_extended_value(&mut self, value: i64) -> u64 {
        self.write_eight_byte_value(value.to_be_bytes())
    }

    /// Writes a four-byte type value followed by its four padding bytes.
    fn write_four_byte_value(&mut self, value: [u8; 4]) -> u64 {
        self.bytes.extend_from_slice(&value);
        self.append_zero_padding(4);
        4
    }

    /// Writes an eight-byte type value with no following padding.
    fn write_eight_byte_value(&mut self, value: [u8; 8]) -> u64 {
        self.bytes.extend_from_slice(&value);
        8
    }

    /// Writes payload octets, then the minimum zero fill needed for an eight-byte boundary.
    fn write_padded_bytes(&mut self, payload: &[u8]) -> u64 {
        let padding = padding_count(payload.len());
        let item_length = payload.len() as u64;
        self.append_payload(payload);
        self.append_zero_padding(padding);
        item_length
    }

    /// Appends zero padding whose count was derived from the preflighted value length.
    fn append_zero_padding(&mut self, count: usize) {
        self.append_repeated(0, count);
    }

    /// Appends sign-extension padding whose count is less than the eight-byte quantum.
    fn append_sign_padding(&mut self, byte: u8, count: usize) {
        self.append_repeated(byte, count);
    }

    /// Appends at most seven copies from a fixed stack buffer without allocation.
    fn append_repeated(&mut self, byte: u8, count: usize) {
        const FILL: [u8; 8] = [0; 8];
        const EXTENSION: [u8; 8] = [0xff; 8];
        let fill = if byte == 0 { &FILL } else { &EXTENSION };
        self.bytes.extend_from_slice(&fill[..count]);
    }

    /// Copies an already measured payload into the fully reserved output buffer.
    fn append_payload(&mut self, payload: &[u8]) {
        #[cfg(test)]
        shared_payload_copy(self.bytes, payload, self.observer);
        #[cfg(not(test))]
        shared_payload_copy(self.bytes, payload);
    }
}

/// Returns the minimum padding count for a value length known to fit `usize`.
const fn padding_count(value_length: usize) -> usize {
    (8 - (value_length % 8)) % 8
}

const fn type_code(item_type: ItemType) -> u8 {
    match item_type {
        ItemType::Structure => 0x01,
        ItemType::Integer => 0x02,
        ItemType::LongInteger => 0x03,
        ItemType::BigInteger => 0x04,
        ItemType::Enumeration => 0x05,
        ItemType::Boolean => 0x06,
        ItemType::TextString => 0x07,
        ItemType::ByteString => 0x08,
        ItemType::DateTime => 0x09,
        ItemType::Interval => 0x0a,
        ItemType::DateTimeExtended => 0x0b,
        _ => 0,
    }
}

#[cfg(test)]
fn shared_payload_copy(
    output: &mut Vec<u8>,
    payload: &[u8],
    observer: &tests::PayloadCopyObserver,
) {
    if !payload.is_empty() {
        observer.record_copy();
    }
    output.extend_from_slice(payload);
}

#[cfg(not(test))]
fn shared_payload_copy(output: &mut Vec<u8>, payload: &[u8]) {
    output.extend_from_slice(payload);
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::fmt;
    use std::rc::Rc;

    use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};
    use static_assertions::assert_not_impl_any;

    use super::{
        BorrowedLimitsView, EncodeError, EncodedOwner, EncodingPlan, check_plan,
        encode_item_with_limits, plan_item_length, reserve_output_buffer,
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
        fn max_message_bytes(&self) -> u64 {
            self.message_bytes
        }

        fn max_structure_depth(&self) -> u64 {
            self.structure_depth
        }

        fn max_elements(&self) -> u64 {
            self.elements
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

    #[derive(Default)]
    pub(super) struct PayloadCopyObserver {
        calls: Cell<usize>,
    }

    impl PayloadCopyObserver {
        pub(super) fn calls(&self) -> usize {
            self.calls.get()
        }

        pub(super) fn record_copy(&self) {
            self.calls.set(self.calls.get() + 1);
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
        // OASIS KMIP Specification v2.1 §§10.1.2, 10.1.3, and 10.1.5;
        // KMIPKIT-0005-NR-002, KMIPKIT-0005-NR-004, and KMIPKIT-0005-NR-005;
        // KMIPKIT-REQ-SPEC-10.1.5-001-002: Integer is signed big-endian and has
        // four following padding bytes. Zero padding octets are the FR-002
        // canonical output; OASIS does not prescribe their values.
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
        let encoded = encode_item(&item(Value::enumeration(u32::MAX)))
            .expect("Enumeration vector is encodable");

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
        let encoded =
            encode_item(&item(Value::boolean(true))).expect("Boolean vector is encodable");

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
                0x42, 0x01, 0x73, 0x01, 0, 0, 0, 32, 0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0, 0, 0,
                1, 0, 0, 0, 0, 0x42, 0x01, 0x73, 0x02, 0, 0, 0, 4, 0, 0, 0, 2, 0, 0, 0, 0,
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
        let result = reserve_output_buffer(usize::MAX);
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
        let limits = LimitsView::new(4, MODEL_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
        let copy_observer = PayloadCopyObserver::default();
        let payload = vec![0xa5, 0x5a, 0xa5];
        let input = item(Value::byte_string(payload));
        let result = encode_item_with_limits(&input, &limits, &copy_observer);

        assert_eq!(result.err(), Some(EncodeError::LimitExceeded));
        assert_eq!(copy_observer.calls(), 0);
    }

    #[test]
    fn preflight_error_does_not_format_payload_bytes() {
        // Project-only payload-redaction behavior under KMIPKIT-0005-FR-009.
        let limits = LimitsView::new(4, MODEL_MAX_STRUCTURE_DEPTH, DEFAULT_MAX_ELEMENTS);
        let payload = vec![0xde, 0xad, 0xbe, 0xef];
        let input = item(Value::byte_string(payload));
        let copy_observer = PayloadCopyObserver::default();
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

        let (observed_length, all_zero) =
            observer.result().expect("Drop reports initialized bytes");

        assert!(observed_length > 0, "observer must see initialized bytes");
        assert!(
            all_zero,
            "initialized bytes must be zeroized before deallocation"
        );
    }
}
