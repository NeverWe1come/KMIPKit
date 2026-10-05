//! Private outbound TTLV encoding for typed KMIP items.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, ValueView};
#[cfg(test)]
use zeroize::Zeroize;
use zeroize::Zeroizing;

const HEADER_BYTES: u64 = 8;
const PADDING_BYTES: u64 = 8;
const MODEL_MAX_STRUCTURE_DEPTH: u64 = 64;

/// Private adapter reads the public immutable values without cloning or
/// reconstructing the per-operation limits instance.
trait BorrowedLimitsView {
    fn max_message_bytes(&self) -> u128;
    fn max_structure_depth(&self) -> u128;
    fn max_elements(&self) -> u128;

    #[cfg(test)]
    fn codec_limits(&self) -> Option<&CodecLimits> {
        None
    }
}

impl BorrowedLimitsView for CodecLimits {
    fn max_message_bytes(&self) -> u128 {
        usize_limit_to_u128(self.max_message_bytes())
    }

    fn max_structure_depth(&self) -> u128 {
        usize_limit_to_u128(self.max_structure_depth())
    }

    fn max_elements(&self) -> u128 {
        usize_limit_to_u128(self.max_elements())
    }

    #[cfg(test)]
    fn codec_limits(&self) -> Option<&CodecLimits> {
        Some(self)
    }
}

fn usize_limit_to_u128(value: usize) -> u128 {
    // Planner counts use u64, so a larger configured limit accepts every
    // representable plan and can be saturated without narrowing that plan.
    u128::from(u64::try_from(value).unwrap_or(u64::MAX))
}

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
    // One complete item tree is measured before limits are checked or output is
    // reserved. `structure_depth` counts nested Structures; `elements` counts
    // every Item including the root.
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
    check_plan_inner(
        plan,
        limits,
        #[cfg(test)]
        None,
    )
}

fn check_plan_inner(
    plan: &EncodingPlan,
    limits: &impl BorrowedLimitsView,
    #[cfg(test)] limits_observer: Option<&tests::LimitsIdentityObserver<'_>>,
) -> Result<(), EncodeError> {
    #[cfg(test)]
    if let Some(observer) = limits_observer
        && let Some(codec_limits) = limits.codec_limits()
    {
        observer.record(codec_limits);
    }

    let max_depth = limits
        .max_structure_depth()
        .min(u128::from(MODEL_MAX_STRUCTURE_DEPTH));

    // This is the single preflight gate for all three resource dimensions.
    // It runs after checked planning and before the output reservation or any
    // payload copy in `encode_item_inner`.
    if u128::from(plan.encoded_bytes) > limits.max_message_bytes()
        || u128::from(plan.structure_depth) > max_depth
        || u128::from(plan.elements) > limits.max_elements()
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
        self.bytes.as_mut_slice().zeroize();
        if let Some(observer) = &self.drop_observer {
            observer.observe_before_deallocation(self.bytes.as_slice());
        }
    }
}

fn encode_item(item: &Item) -> Result<EncodedOwner, EncodeError> {
    let limits = CodecLimits::defaults();
    #[cfg(test)]
    let observer = tests::EncodingObserver::default();

    encode_item_with_limits(
        item,
        &limits,
        #[cfg(test)]
        &observer,
    )
}

fn encode_item_with_limits(
    item: &Item,
    limits: &CodecLimits,
    #[cfg(test)] observer: &tests::EncodingObserver,
) -> Result<EncodedOwner, EncodeError> {
    encode_item_inner(
        item,
        limits,
        #[cfg(test)]
        observer,
        #[cfg(test)]
        None,
    )
}

#[cfg(test)]
fn encode_item_with_identity_observer(
    item: &Item,
    limits: &CodecLimits,
    copy_observer: &tests::EncodingObserver,
    limits_observer: &tests::LimitsIdentityObserver<'_>,
) -> Result<EncodedOwner, EncodeError> {
    encode_item_inner(item, limits, copy_observer, Some(limits_observer))
}

fn encode_item_inner(
    item: &Item,
    limits: &CodecLimits,
    #[cfg(test)] copy_observer: &tests::EncodingObserver,
    #[cfg(test)] limits_observer: Option<&tests::LimitsIdentityObserver<'_>>,
) -> Result<EncodedOwner, EncodeError> {
    // Keep the failure-atomic sequence explicit: measure the full tree, apply
    // all resource limits and U32 Item Length checks, reserve once, then write.
    let plan = EncodingPlan::for_item(item)?;
    check_writer_plan(
        &plan,
        limits,
        #[cfg(test)]
        limits_observer,
    )?;
    let capacity = usize::try_from(plan.encoded_bytes).map_err(|_| EncodeError::SizeOverflow)?;

    let output = reserve_output_buffer(
        capacity,
        #[cfg(test)]
        Some(copy_observer),
    )?;

    // Own and zeroize the reserved allocation before writing starts. If an
    // invariant is ever violated during writing, Drop still clears the bytes.
    let mut owner = EncodedOwner::new(output);
    {
        let mut writer = Writer {
            bytes: &mut owner.bytes,
            #[cfg(test)]
            observer: copy_observer,
        };
        writer.write_item(item);
    }

    Ok(owner)
}

fn check_writer_plan(
    plan: &EncodingPlan,
    limits: &CodecLimits,
    #[cfg(test)] limits_observer: Option<&tests::LimitsIdentityObserver<'_>>,
) -> Result<(), EncodeError> {
    #[cfg(test)]
    if let Some(observer) = limits_observer {
        return check_plan_inner(plan, limits, Some(observer));
    }
    check_plan(plan, limits)
}

fn reserve_output_buffer(
    capacity: usize,
    #[cfg(test)] observer: Option<&tests::EncodingObserver>,
) -> Result<Vec<u8>, EncodeError> {
    let mut output = Vec::new();
    #[cfg(test)]
    if let Some(observer) = observer {
        observer.record_output_reservation();
    }
    output
        .try_reserve_exact(capacity)
        .map_err(EncodeError::AllocationFailed)?;
    Ok(output)
}

struct Writer<'a> {
    bytes: &'a mut Vec<u8>,
    #[cfg(test)]
    observer: &'a tests::EncodingObserver,
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
fn shared_payload_copy(output: &mut Vec<u8>, payload: &[u8], observer: &tests::EncodingObserver) {
    if !payload.is_empty() {
        observer.record_copy();
    }
    output.extend_from_slice(payload);
}

#[cfg(not(test))]
fn shared_payload_copy(output: &mut Vec<u8>, payload: &[u8]) {
    output.extend_from_slice(payload);
}

#[path = "../tests/support/wire_encoder_tests.rs"]
mod tests;
