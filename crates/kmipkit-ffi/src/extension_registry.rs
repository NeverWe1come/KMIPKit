//! C ABI implementation for the manifest-backed extension registry API.
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

use std::ptr;
use std::sync::Arc;

use crate::{
    kmipkit_client_batch_item_t, kmipkit_client_configuration_t,
    kmipkit_client_extension_registry_t, kmipkit_client_request_message_extension_t,
    kmipkit_codec_limits_t, kmipkit_extension_child_rule_t, kmipkit_extension_compatibility_t,
    kmipkit_extension_definition_t, kmipkit_extension_discriminator_t,
    kmipkit_extension_identity_t, kmipkit_extension_information_t,
    kmipkit_extension_order_constraint_t, kmipkit_extension_recognition_t,
    kmipkit_extension_schema_t, kmipkit_raw_tag_t, kmipkit_registered_extension_value_t,
    kmipkit_tag_t, kmipkit_ttlv_item_t, kmipkit_ttlv_item_view_t, kmipkit_ttlv_path_t,
    kmipkit_ttlv_structure_t, kmipkit_ttlv_structure_view_t, kmipkit_ttlv_value_t,
    kmipkit_ttlv_value_view_t, kmipkit_validated_extension_value_t,
};
use kmipkit_client::ClientBatchItem;
use kmipkit_client::extension_registry as client_extension;
use kmipkit_protocol::extension;
use kmipkit_protocol::{ProtocolError, ProtocolErrorKind};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};

#[cfg(test)]
#[path = "../tests/support/registry_preflight_tests.rs"]
mod registry_preflight_tests;
#[cfg(test)]
#[path = "../tests/support/view_ownership_tests.rs"]
mod view_ownership_tests;

#[cfg(test)]
thread_local! {
    static REGISTRY_DEFINITION_CLONE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

const SUCCESS: i32 = 0;
const ERROR_COMPATIBILITY_MISMATCH: i32 = 1;
const ERROR_DUPLICATE_KEY: i32 = 2;
const ERROR_INVALID_IDENTITY: i32 = 3;
const ERROR_INVALID_INPUT: i32 = 4;
const ERROR_INVALID_SCHEMA: i32 = 5;
const ERROR_RESOURCE_LIMIT: i32 = 6;
const MAX_TEXT_BYTES: u64 = 4_096;
const MAX_CHILD_RULES: u64 = 4_096;
const MAX_ORDER_CONSTRAINTS: u64 = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Identity,
    Compatibility,
    Path,
    Discriminator,
    Schema,
    Information,
    Definition,
    Registry,
    Configuration,
    Validated,
    Registered,
    MessageExtension,
    Structure,
    BatchItem,
    ChildRule,
    OrderConstraint,
    Value,
    Recognition,
    RawTag,
    Tag,
    Item,
    CodecLimits,
    StructureView,
    ItemView,
    ValueView,
}

enum HandleValue {
    Identity(extension::ExtensionIdentity),
    Compatibility(extension::Compatibility),
    Path(extension::TtlvPath),
    Discriminator(extension::Discriminator),
    Schema(extension::ExtensionSchema),
    Information(extension::ExtensionInformation),
    Definition(Box<extension::ExtensionDefinition>),
    DefinitionRef {
        registry: Arc<client_extension::ClientExtensionRegistry>,
        index: usize,
    },
    Registry(Arc<client_extension::ClientExtensionRegistry>),
    Configuration(client_extension::ClientConfiguration),
    Validated(kmipkit_protocol::extension::ValidatedExtensionValue),
    ValidatedRef(Arc<Handle>),
    Registered(client_extension::RegisteredExtensionValue),
    MessageExtension(client_extension::ClientRequestMessageExtension),
    Structure(Structure),
    BatchItem(ClientBatchItem),
    ChildRule(Box<extension::ExtensionChildRule>),
    OrderConstraint(extension::ExtensionOrderConstraint),
    Value(Value),
    Recognition(client_extension::ExtensionRecognition),
    RawTag(RawTag),
    Tag(Tag),
    Item(Item),
    CodecLimits(CodecLimits),
    StructureView(TtlvView),
    ItemView(TtlvView),
    ValueView(TtlvView),
}

struct Handle {
    kind: Kind,
    value: HandleValue,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ViewRoot {
    Structure,
    Value,
    RecognitionGeneric,
    ValidatedGeneric,
}

#[derive(Clone, Copy)]
enum ViewStep {
    Item(usize),
    Value,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ViewTarget {
    Structure,
    Item,
    Value,
}

// Each of the 64 supported nesting levels records one item selection and its value.
const MAX_VIEW_PATH_STEPS: usize = 2 * 64;

fn copy_view_steps(steps: &[ViewStep], additional_steps: usize) -> FfiResult<Vec<ViewStep>> {
    let required_len = steps
        .len()
        .checked_add(additional_steps)
        .filter(|length| *length <= MAX_VIEW_PATH_STEPS)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(required_len)
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    copied.extend_from_slice(steps);
    Ok(copied)
}

/// A C read-only view resolves into an immutable, Arc-retained TTLV owner.
///
/// The path stores item indexes rather than tags so repeated tags retain their
/// original order and the same selected item across descendant views.
struct TtlvView {
    owner: Arc<Handle>,
    root: ViewRoot,
    steps: Vec<ViewStep>,
    target: ViewTarget,
}

impl TtlvView {
    fn new(owner: Arc<Handle>, root: ViewRoot, target: ViewTarget) -> Self {
        Self {
            owner,
            root,
            steps: Vec::new(),
            target,
        }
    }

    fn append(&self, step: ViewStep, target: ViewTarget) -> FfiResult<Self> {
        let mut steps = copy_view_steps(&self.steps, 1)?;
        steps.push(step);
        Ok(Self {
            owner: Arc::clone(&self.owner),
            root: self.root,
            steps,
            target,
        })
    }

    fn with_structure_at<R>(
        &self,
        steps: &[ViewStep],
        callback: impl for<'a> FnOnce(&kmipkit_ttlv::StructureView<'a>) -> R,
    ) -> FfiResult<R> {
        match self.root {
            ViewRoot::Structure => {
                let HandleValue::Structure(structure) = &self.owner.value else {
                    return Err(ERROR_INVALID_INPUT);
                };
                resolve_structure_path(&structure.view(), steps, callback)
            }
            ViewRoot::Value => {
                let HandleValue::Value(value) = &self.owner.value else {
                    return Err(ERROR_INVALID_INPUT);
                };
                value.with_value(|value| match value {
                    ValueView::Structure(structure) => {
                        resolve_structure_path(&structure, steps, callback)
                    }
                    _ => Err(ERROR_INVALID_INPUT),
                })
            }
            ViewRoot::RecognitionGeneric => {
                let HandleValue::Recognition(recognition) = &self.owner.value else {
                    return Err(ERROR_INVALID_INPUT);
                };
                resolve_structure_path(
                    &client_extension::generic_value(recognition).view(),
                    steps,
                    callback,
                )
            }
            ViewRoot::ValidatedGeneric => {
                let validated = validated_from_handle(&self.owner)?;
                resolve_structure_path(&extension::generic_value(validated).view(), steps, callback)
            }
        }
    }

    fn with_structure<R>(
        &self,
        callback: impl for<'a> FnOnce(&kmipkit_ttlv::StructureView<'a>) -> R,
    ) -> FfiResult<R> {
        if self.target != ViewTarget::Structure {
            return Err(ERROR_INVALID_INPUT);
        }
        self.with_structure_at(&self.steps, callback)
    }

    fn with_item<R>(&self, callback: impl FnOnce(&Item) -> R) -> FfiResult<R> {
        if self.target != ViewTarget::Item {
            return Err(ERROR_INVALID_INPUT);
        }
        let Some((ViewStep::Item(index), parent_steps)) = self.steps.split_last() else {
            return Err(ERROR_INVALID_INPUT);
        };
        self.with_structure_at(parent_steps, |structure| {
            let item = structure
                .children()
                .get(*index)
                .ok_or(ERROR_INVALID_INPUT)?;
            Ok(callback(item))
        })?
    }

    fn with_value<R>(
        &self,
        callback: impl for<'a> FnOnce(ValueView<'a>) -> FfiResult<R>,
    ) -> FfiResult<R> {
        if self.target != ViewTarget::Value {
            return Err(ERROR_INVALID_INPUT);
        }
        if self.root == ViewRoot::Value && self.steps.is_empty() {
            let HandleValue::Value(value) = &self.owner.value else {
                return Err(ERROR_INVALID_INPUT);
            };
            return value.with_value(callback);
        }

        self.with_structure_at(&[], |structure| {
            resolve_value_path(structure, &self.steps, callback)
        })?
    }

    fn item_at(&self, index: usize) -> FfiResult<Self> {
        let count = self.with_structure(|structure| structure.children().len())?;
        if index >= count {
            return Err(ERROR_INVALID_INPUT);
        }
        self.append(ViewStep::Item(index), ViewTarget::Item)
    }

    fn item_value(&self) -> FfiResult<Self> {
        if self.target != ViewTarget::Item {
            return Err(ERROR_INVALID_INPUT);
        }
        self.append(ViewStep::Value, ViewTarget::Value)
    }

    fn structure_from_value(&self) -> FfiResult<Self> {
        self.as_structure(ERROR_INVALID_INPUT)
    }

    fn nested_structure_from_value(&self) -> FfiResult<Self> {
        self.as_structure(ERROR_INVALID_INPUT)
    }

    fn as_structure(&self, mismatch_status: i32) -> FfiResult<Self> {
        let is_structure = self.with_value(|value| Ok(matches!(value, ValueView::Structure(_))))?;
        if !is_structure {
            return Err(mismatch_status);
        }
        let steps = copy_view_steps(&self.steps, 0)?;
        Ok(Self {
            owner: Arc::clone(&self.owner),
            root: self.root,
            steps,
            target: ViewTarget::Structure,
        })
    }

    fn value_at_tags(&self, tags: &[Tag]) -> FfiResult<Self> {
        if self.target != ViewTarget::Structure {
            return Err(ERROR_INVALID_INPUT);
        }
        if tags.is_empty() {
            return Err(ERROR_INVALID_INPUT);
        }
        let mut selected = self.duplicate()?;
        for (index, tag) in tags.iter().enumerate() {
            let child_index = selected.with_structure(|structure| {
                structure
                    .children()
                    .iter()
                    .position(|item| item.tag() == *tag)
            })?;
            let child_index = child_index.ok_or(ERROR_INVALID_INPUT)?;
            selected = selected.item_at(child_index)?;
            if index + 1 < tags.len() {
                selected = selected.item_value()?.nested_structure_from_value()?;
            } else {
                selected = selected.item_value()?;
            }
        }
        Ok(selected)
    }

    fn duplicate(&self) -> FfiResult<Self> {
        let steps = copy_view_steps(&self.steps, 0)?;
        Ok(Self {
            owner: Arc::clone(&self.owner),
            root: self.root,
            steps,
            target: self.target,
        })
    }
}

fn resolve_structure_path<R>(
    structure: &kmipkit_ttlv::StructureView<'_>,
    steps: &[ViewStep],
    callback: impl for<'a> FnOnce(&kmipkit_ttlv::StructureView<'a>) -> R,
) -> FfiResult<R> {
    if steps.is_empty() {
        return Ok(callback(structure));
    }
    let [ViewStep::Item(index), ViewStep::Value, remaining @ ..] = steps else {
        return Err(ERROR_INVALID_INPUT);
    };
    let item = structure
        .children()
        .get(*index)
        .ok_or(ERROR_INVALID_INPUT)?;
    item.with_value(|value| match value {
        ValueView::Structure(nested) => resolve_structure_path(&nested, remaining, callback),
        _ => Err(ERROR_INVALID_INPUT),
    })
}

fn resolve_value_path<R>(
    structure: &kmipkit_ttlv::StructureView<'_>,
    steps: &[ViewStep],
    callback: impl for<'a> FnOnce(ValueView<'a>) -> FfiResult<R>,
) -> FfiResult<R> {
    let [ViewStep::Item(index), ViewStep::Value, remaining @ ..] = steps else {
        return Err(ERROR_INVALID_INPUT);
    };
    let item = structure
        .children()
        .get(*index)
        .ok_or(ERROR_INVALID_INPUT)?;
    item.with_value(|value| {
        if remaining.is_empty() {
            callback(value)
        } else {
            match value {
                ValueView::Structure(nested) => resolve_value_path(&nested, remaining, callback),
                _ => Err(ERROR_INVALID_INPUT),
            }
        }
    })
}

type FfiResult<T> = Result<T, i32>;

fn make_handle<T>(kind: Kind, value: HandleValue) -> *mut T {
    Arc::into_raw(Arc::new(Handle { kind, value }))
        .cast_mut()
        .cast()
}

fn reference_handle<T>(pointer: &*mut T, expected: Kind) -> FfiResult<&Handle> {
    if pointer.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: non-null opaque handle pointers must be live KMIPKit handles; dangling and foreign
    // pointers are documented caller precondition violations. The envelope discriminator is
    // checked before any payload is accessed.
    let handle = unsafe { &*(*pointer).cast::<Handle>() };
    if handle.kind != expected {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(handle)
}

fn clone_handle_owner<T>(pointer: *mut T, expected: Kind) -> FfiResult<Arc<Handle>> {
    if pointer.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    let handle = reference_handle(&pointer, expected)?;
    let _ = handle;
    // SAFETY: the handle reference was validated above and the C caller keeps its strong reference
    // live for the duration of this call. Incrementing before reconstructing creates one owned copy.
    unsafe {
        Arc::increment_strong_count(pointer.cast::<Handle>());
    }
    // SAFETY: the preceding increment produced exactly one strong reference for this Arc.
    Ok(unsafe { Arc::from_raw(pointer.cast::<Handle>()) })
}

fn consume_owner<T>(pointer: *mut T) -> Arc<Handle> {
    // SAFETY: callers validate that this is a live KMIPKit handle and preflight every handle in
    // the operation before transferring any consumed handle. Dangling and foreign pointers remain
    // caller precondition violations.
    unsafe { Arc::from_raw(pointer.cast::<Handle>()) }
}

fn unwrap_payload(handle: Arc<Handle>) -> FfiResult<HandleValue> {
    match Arc::try_unwrap(handle) {
        Ok(handle) => Ok(handle.value),
        Err(shared) => {
            drop(shared);
            Err(ERROR_INVALID_INPUT)
        }
    }
}

fn owned_payload<T>(pointer: *mut T, expected: Kind) -> FfiResult<HandleValue> {
    let _ = reference_handle(&pointer, expected)?;
    unwrap_payload(consume_owner(pointer))
}

macro_rules! out_slot {
    ($out:expr) => {{
        if $out.is_null() {
            return ERROR_INVALID_INPUT;
        }
        // SAFETY: this is the non-null writable output pointer supplied by the C caller.
        let slot = unsafe { &mut *$out };
        *slot = ptr::null_mut();
        slot
    }};
}

macro_rules! scalar_out {
    ($out:expr) => {{
        if $out.is_null() {
            return ERROR_INVALID_INPUT;
        }
        // SAFETY: this is the non-null writable output pointer supplied by the C caller.
        unsafe { &mut *$out }
    }};
}

fn input_span<'a>(data: *const u8, length: u64, maximum: u64) -> FfiResult<&'a [u8]> {
    if length > maximum {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let length = usize::try_from(length).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if length == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: for a non-zero span, the C caller must provide `length` readable bytes; the length
    // has already been checked against the operation's bound and converted to usize.
    Ok(unsafe { std::slice::from_raw_parts(data, length) })
}

fn input_handle_array<T, U>(
    array: *mut *mut T,
    count: u64,
    maximum: u64,
    kind: Kind,
    mut copy: impl FnMut(&Handle) -> FfiResult<U>,
) -> FfiResult<Vec<U>> {
    if count > maximum {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let count = usize::try_from(count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if count == 0 {
        return Ok(Vec::new());
    }
    if array.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: for nonzero `count`, the C caller must provide a readable array of that many handle
    // pointers. The count is bounded before reading any array element.
    let pointers = unsafe { std::slice::from_raw_parts(array, count) };
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for pointer in pointers {
        let handle = reference_handle(pointer, kind)?;
        result.push(copy(handle)?);
    }
    Ok(result)
}

fn protocol_status(error: &ProtocolError) -> i32 {
    match error.kind() {
        ProtocolErrorKind::CompatibilityMismatch => ERROR_COMPATIBILITY_MISMATCH,
        ProtocolErrorKind::DuplicateKey => ERROR_DUPLICATE_KEY,
        ProtocolErrorKind::InvalidIdentity => ERROR_INVALID_IDENTITY,
        ProtocolErrorKind::InvalidInput => ERROR_INVALID_INPUT,
        ProtocolErrorKind::ResourceLimit => ERROR_RESOURCE_LIMIT,
        _ => ERROR_INVALID_SCHEMA,
    }
}

fn client_status(error: &kmipkit_client::ClientError) -> i32 {
    match error {
        kmipkit_client::ClientError::Protocol { error, .. } => protocol_status(error),
        _ => ERROR_INVALID_INPUT,
    }
}

fn model_status(error: kmipkit_ttlv::ModelError) -> i32 {
    match error {
        kmipkit_ttlv::ModelError::RawTagOutOfRange => ERROR_INVALID_INPUT,
        kmipkit_ttlv::ModelError::StructureDepthExceeded => ERROR_RESOURCE_LIMIT,
        _ => ERROR_INVALID_SCHEMA,
    }
}

fn item_type_from_u32(value: u32) -> FfiResult<ItemType> {
    match value {
        1 => Ok(ItemType::Structure),
        2 => Ok(ItemType::Integer),
        3 => Ok(ItemType::LongInteger),
        4 => Ok(ItemType::BigInteger),
        5 => Ok(ItemType::Enumeration),
        6 => Ok(ItemType::Boolean),
        7 => Ok(ItemType::TextString),
        8 => Ok(ItemType::ByteString),
        9 => Ok(ItemType::DateTime),
        10 => Ok(ItemType::Interval),
        11 => Ok(ItemType::DateTimeExtended),
        _ => Err(ERROR_INVALID_SCHEMA),
    }
}

fn item_type_code(value: ItemType) -> u8 {
    match value {
        ItemType::Structure => 1,
        ItemType::Integer => 2,
        ItemType::LongInteger => 3,
        ItemType::BigInteger => 4,
        ItemType::Enumeration => 5,
        ItemType::Boolean => 6,
        ItemType::TextString => 7,
        ItemType::ByteString => 8,
        ItemType::DateTime => 9,
        ItemType::Interval => 10,
        ItemType::DateTimeExtended => 11,
        _ => 0,
    }
}

fn value_view_type_code(value: &ValueView<'_>) -> u8 {
    match value {
        ValueView::Structure(_) => 1,
        ValueView::Integer(_) => 2,
        ValueView::LongInteger(_) => 3,
        ValueView::BigInteger(_) => 4,
        ValueView::Enumeration(_) => 5,
        ValueView::Boolean(_) => 6,
        ValueView::TextString(_) => 7,
        ValueView::ByteString(_) => 8,
        ValueView::DateTime(_) => 9,
        ValueView::Interval(_) => 10,
        ValueView::DateTimeExtended(_) => 11,
        _ => 0,
    }
}

fn clone_value(value: ValueView<'_>) -> FfiResult<Value> {
    match value {
        ValueView::Structure(structure) => Ok(Value::structure(clone_structure_view(&structure)?)),
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(ERROR_INVALID_SCHEMA),
    }
}

fn clone_item(item: &Item) -> FfiResult<Item> {
    Item::new(item.tag(), item.with_value(clone_value)?).map_err(model_status)
}

fn clone_structure(structure: &Structure) -> FfiResult<Structure> {
    let mut clone = Structure::new();
    for item in structure.view().children() {
        clone.try_push(clone_item(item)?).map_err(model_status)?;
    }
    Ok(clone)
}

fn clone_structure_view(structure: &kmipkit_ttlv::StructureView<'_>) -> FfiResult<Structure> {
    let mut clone = Structure::new();
    for item in structure.children() {
        clone.try_push(clone_item(item)?).map_err(model_status)?;
    }
    Ok(clone)
}

fn limits_from_fields(fields: [u64; 12]) -> FfiResult<extension::ExtensionRegistryLimits> {
    extension::with_values(
        fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6], fields[7],
        fields[8], fields[9], fields[10], fields[11],
    )
    .map_err(|error| protocol_status(&error))
}

fn registry_from_handle(
    handle: &Handle,
) -> FfiResult<&Arc<client_extension::ClientExtensionRegistry>> {
    match &handle.value {
        HandleValue::Registry(registry) => Ok(registry),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn definition_from_handle(handle: &Handle) -> FfiResult<&extension::ExtensionDefinition> {
    match &handle.value {
        HandleValue::Definition(definition) => Ok(definition),
        HandleValue::DefinitionRef { registry, index } => {
            client_extension::definition_at(registry, *index).ok_or(ERROR_INVALID_INPUT)
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn validated_from_handle(
    handle: &Handle,
) -> FfiResult<&kmipkit_protocol::extension::ValidatedExtensionValue> {
    match &handle.value {
        HandleValue::Validated(value) => Ok(value),
        HandleValue::ValidatedRef(owner) => match &owner.value {
            HandleValue::Recognition(recognition) => {
                recognition.validated_value().ok_or(ERROR_INVALID_INPUT)
            }
            _ => Err(ERROR_INVALID_INPUT),
        },
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn reference_registry(
    pointer: &*mut kmipkit_client_extension_registry_t,
) -> FfiResult<&Arc<client_extension::ClientExtensionRegistry>> {
    let handle = reference_handle(pointer, Kind::Registry)?;
    registry_from_handle(handle)
}

fn release<T>(pointer: *mut T, kind: Kind) {
    if reference_handle(&pointer, kind).is_err() {
        return;
    }
    drop(consume_owner(pointer));
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_release(handle: *mut kmipkit_extension_identity_t) {
    release(handle, Kind::Identity);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_compatibility_release(
    handle: *mut kmipkit_extension_compatibility_t,
) {
    release(handle, Kind::Compatibility);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_path_release(handle: *mut kmipkit_ttlv_path_t) {
    release(handle, Kind::Path);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_discriminator_release(
    handle: *mut kmipkit_extension_discriminator_t,
) {
    release(handle, Kind::Discriminator);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_release(handle: *mut kmipkit_extension_schema_t) {
    release(handle, Kind::Schema);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_release(
    handle: *mut kmipkit_extension_information_t,
) {
    release(handle, Kind::Information);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_release(
    handle: *mut kmipkit_extension_definition_t,
) {
    release(handle, Kind::Definition);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_release(
    handle: *mut kmipkit_client_extension_registry_t,
) {
    release(handle, Kind::Registry);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_configuration_release(
    handle: *mut kmipkit_client_configuration_t,
) {
    release(handle, Kind::Configuration);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_release(
    handle: *mut kmipkit_validated_extension_value_t,
) {
    release(handle, Kind::Validated);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_registered_extension_value_release(
    handle: *mut kmipkit_registered_extension_value_t,
) {
    release(handle, Kind::Registered);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_request_message_extension_release(
    handle: *mut kmipkit_client_request_message_extension_t,
) {
    release(handle, Kind::MessageExtension);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_release(handle: *mut kmipkit_ttlv_structure_t) {
    release(handle, Kind::Structure);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_release(handle: *mut kmipkit_client_batch_item_t) {
    release(handle, Kind::BatchItem);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_release(
    handle: *mut kmipkit_extension_child_rule_t,
) {
    release(handle, Kind::ChildRule);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_order_constraint_release(
    handle: *mut kmipkit_extension_order_constraint_t,
) {
    release(handle, Kind::OrderConstraint);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_release(handle: *mut kmipkit_ttlv_value_t) {
    release(handle, Kind::Value);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_release(
    handle: *mut kmipkit_extension_recognition_t,
) {
    release(handle, Kind::Recognition);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_raw_tag_release(handle: *mut kmipkit_raw_tag_t) {
    release(handle, Kind::RawTag);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_tag_release(handle: *mut kmipkit_tag_t) {
    release(handle, Kind::Tag);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_release(handle: *mut kmipkit_ttlv_item_t) {
    release(handle, Kind::Item);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_release(handle: *mut kmipkit_codec_limits_t) {
    release(handle, Kind::CodecLimits);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view_release(handle: *mut kmipkit_ttlv_structure_view_t) {
    release(handle, Kind::StructureView);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_release(handle: *mut kmipkit_ttlv_item_view_t) {
    release(handle, Kind::ItemView);
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_release(handle: *mut kmipkit_ttlv_value_view_t) {
    release(handle, Kind::ValueView);
}

macro_rules! try_ffi {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(status) => return status,
        }
    };
}

macro_rules! read_handle {
    ($pointer:expr, $kind:ident) => {
        try_ffi!(reference_handle(&$pointer, Kind::$kind))
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_create(
    vendor_identifier_data: *const u8,
    vendor_identifier_length: u64,
    name_data: *const u8,
    name_length: u64,
    version_data: *const u8,
    version_length: u64,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    let out = out_slot!(out_identity);
    let vendor = try_ffi!(input_span(
        vendor_identifier_data,
        vendor_identifier_length,
        MAX_TEXT_BYTES
    ));
    let name = try_ffi!(input_span(name_data, name_length, MAX_TEXT_BYTES));
    let version = try_ffi!(input_span(version_data, version_length, MAX_TEXT_BYTES));
    let vendor = try_ffi!(std::str::from_utf8(vendor).map_err(|_| ERROR_INVALID_IDENTITY));
    let name = try_ffi!(std::str::from_utf8(name).map_err(|_| ERROR_INVALID_IDENTITY));
    let version = try_ffi!(std::str::from_utf8(version).map_err(|_| ERROR_INVALID_IDENTITY));
    let value = try_ffi!(
        extension::extension_identity(vendor, name, version).map_err(|e| protocol_status(&e))
    );
    *out = make_handle(Kind::Identity, HandleValue::Identity(value));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_vendor_identifier(
    identity: *mut kmipkit_extension_identity_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    let identity = read_handle!(identity, Identity);
    let HandleValue::Identity(identity) = &identity.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = make_handle(
        Kind::Value,
        HandleValue::Value(Value::text_string(identity.vendor_identifier().to_owned())),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_name(
    identity: *mut kmipkit_extension_identity_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    let identity = read_handle!(identity, Identity);
    let HandleValue::Identity(identity) = &identity.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = make_handle(
        Kind::Value,
        HandleValue::Value(Value::text_string(identity.name().to_owned())),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_version(
    identity: *mut kmipkit_extension_identity_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    let identity = read_handle!(identity, Identity);
    let HandleValue::Identity(identity) = &identity.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = make_handle(
        Kind::Value,
        HandleValue::Value(Value::text_string(identity.version().to_owned())),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_compatibility_create(
    kmip_min_major: u8,
    kmip_min_minor: u8,
    kmip_max_major: u8,
    kmip_max_minor: u8,
    kmipkit_minimum_data: *const u8,
    kmipkit_minimum_length: u64,
    kmipkit_maximum_data: *const u8,
    kmipkit_maximum_length: u64,
    out_compatibility: *mut *mut kmipkit_extension_compatibility_t,
) -> i32 {
    let out = out_slot!(out_compatibility);
    let minimum = try_ffi!(input_span(
        kmipkit_minimum_data,
        kmipkit_minimum_length,
        MAX_TEXT_BYTES
    ));
    let maximum = try_ffi!(input_span(
        kmipkit_maximum_data,
        kmipkit_maximum_length,
        MAX_TEXT_BYTES
    ));
    let minimum = try_ffi!(std::str::from_utf8(minimum).map_err(|_| ERROR_COMPATIBILITY_MISMATCH));
    let maximum = try_ffi!(std::str::from_utf8(maximum).map_err(|_| ERROR_COMPATIBILITY_MISMATCH));
    let compatibility = try_ffi!(
        extension::compatibility(
            kmip_min_major,
            kmip_min_minor,
            kmip_max_major,
            kmip_max_minor,
            minimum,
            maximum,
        )
        .map_err(|e| protocol_status(&e))
    );
    *out = make_handle(
        Kind::Compatibility,
        HandleValue::Compatibility(compatibility),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_path_create(
    first_tag: u32,
    out_path: *mut *mut kmipkit_ttlv_path_t,
) -> i32 {
    let out = out_slot!(out_path);
    let tag = try_ffi!(RawTag::new(first_tag).map_err(model_status));
    let tag = try_ffi!(tag.try_checked().map_err(model_status));
    let path = try_ffi!(extension::ttlv_path(tag).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::Path, HandleValue::Path(path));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_path_with_child_tag(
    path: *mut kmipkit_ttlv_path_t,
    tag: u32,
    out_path: *mut *mut kmipkit_ttlv_path_t,
) -> i32 {
    let out = out_slot!(out_path);
    let path = try_ffi!(owned_payload(path, Kind::Path));
    let HandleValue::Path(path) = path else {
        return ERROR_INVALID_INPUT;
    };
    let tag = try_ffi!(RawTag::new(tag).map_err(model_status));
    let tag = try_ffi!(tag.try_checked().map_err(model_status));
    let path = try_ffi!(extension::with_child_tag(path, tag).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::Path, HandleValue::Path(path));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_discriminator_create(
    path: *mut kmipkit_ttlv_path_t,
    scalar_value: *mut kmipkit_ttlv_value_t,
    out_discriminator: *mut *mut kmipkit_extension_discriminator_t,
) -> i32 {
    let out = out_slot!(out_discriminator);
    let _ = read_handle!(path, Path);
    let _ = read_handle!(scalar_value, Value);
    let path = consume_owner(path);
    let scalar = consume_owner(scalar_value);
    let path = try_ffi!(unwrap_payload(path));
    let scalar = try_ffi!(unwrap_payload(scalar));
    let HandleValue::Path(path) = path else {
        return ERROR_INVALID_INPUT;
    };
    let HandleValue::Value(scalar) = scalar else {
        return ERROR_INVALID_INPUT;
    };
    let discriminator =
        try_ffi!(extension::discriminator(path, scalar).map_err(|e| protocol_status(&e)));
    *out = make_handle(
        Kind::Discriminator,
        HandleValue::Discriminator(discriminator),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_scalar(
    item_type: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    let out = out_slot!(out_schema);
    let item_type = try_ffi!(item_type_from_u32(item_type));
    let schema = try_ffi!(extension::scalar(item_type).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::Schema, HandleValue::Schema(schema));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_required(
    tag: u32,
    schema: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
) -> i32 {
    child_rule_export(tag, schema, out_rule, extension::required)
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_optional(
    tag: u32,
    schema: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
) -> i32 {
    child_rule_export(tag, schema, out_rule, extension::optional)
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_repeated(
    tag: u32,
    schema: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
) -> i32 {
    child_rule_export(tag, schema, out_rule, extension::repeated)
}

fn child_rule_export(
    raw_tag: u32,
    schema_pointer: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
    build: fn(
        Tag,
        extension::ExtensionSchema,
    ) -> Result<extension::ExtensionChildRule, ProtocolError>,
) -> i32 {
    let out = out_slot!(out_rule);
    let schema_handle = read_handle!(schema_pointer, Schema);
    let HandleValue::Schema(schema) = &schema_handle.value else {
        return ERROR_INVALID_INPUT;
    };
    let tag = try_ffi!(RawTag::new(raw_tag).map_err(model_status));
    let tag = try_ffi!(tag.try_checked().map_err(model_status));
    let rule = try_ffi!(build(tag, schema.clone()).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::ChildRule, HandleValue::ChildRule(Box::new(rule)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_order_constraint_create(
    before_tag: u32,
    after_tag: u32,
    out_constraint: *mut *mut kmipkit_extension_order_constraint_t,
) -> i32 {
    let out = out_slot!(out_constraint);
    let before = try_ffi!(RawTag::new(before_tag).map_err(model_status));
    let before = try_ffi!(before.try_checked().map_err(model_status));
    let after = try_ffi!(RawTag::new(after_tag).map_err(model_status));
    let after = try_ffi!(after.try_checked().map_err(model_status));
    let constraint = try_ffi!(
        extension::extension_order_constraint(before, after).map_err(|e| protocol_status(&e))
    );
    *out = make_handle(
        Kind::OrderConstraint,
        HandleValue::OrderConstraint(constraint),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_structure(
    children: *mut *mut kmipkit_extension_child_rule_t,
    child_count: u64,
    order_constraints: *mut *mut kmipkit_extension_order_constraint_t,
    order_constraint_count: u64,
    preserve_undeclared_children: u8,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    let out = out_slot!(out_schema);
    if preserve_undeclared_children > 1 {
        return ERROR_INVALID_INPUT;
    }
    let children = try_ffi!(input_handle_array(
        children,
        child_count,
        MAX_CHILD_RULES,
        Kind::ChildRule,
        |handle| match &handle.value {
            HandleValue::ChildRule(rule) => Ok((**rule).clone()),
            _ => Err(ERROR_INVALID_INPUT),
        },
    ));
    let order_constraints = try_ffi!(input_handle_array(
        order_constraints,
        order_constraint_count,
        MAX_ORDER_CONSTRAINTS,
        Kind::OrderConstraint,
        |handle| match &handle.value {
            HandleValue::OrderConstraint(constraint) => Ok(*constraint),
            _ => Err(ERROR_INVALID_INPUT),
        },
    ));
    let schema = try_ffi!(
        extension::structure(
            children,
            order_constraints,
            preserve_undeclared_children == 1
        )
        .map_err(|e| protocol_status(&e))
    );
    *out = make_handle(Kind::Schema, HandleValue::Schema(schema));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_minimum_length(
    schema: *mut kmipkit_extension_schema_t,
    bytes: u64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    schema_transform(schema, out_schema, |schema| {
        extension::with_minimum_length(schema, bytes)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_maximum_length(
    schema: *mut kmipkit_extension_schema_t,
    bytes: u64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    schema_transform(schema, out_schema, |schema| {
        extension::with_maximum_length(schema, bytes)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_signed_numeric_range(
    schema: *mut kmipkit_extension_schema_t,
    minimum: i64,
    maximum: i64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    schema_transform(schema, out_schema, |schema| {
        extension::with_signed_range(schema, minimum, maximum)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_unsigned_numeric_range(
    schema: *mut kmipkit_extension_schema_t,
    minimum: u64,
    maximum: u64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    schema_transform(schema, out_schema, |schema| {
        extension::with_unsigned_range(schema, minimum, maximum)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_allowed_enumeration(
    schema: *mut kmipkit_extension_schema_t,
    value: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    schema_transform(schema, out_schema, |schema| {
        extension::with_allowed_enumeration(schema, value)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_allowed_bit_mask(
    schema: *mut kmipkit_extension_schema_t,
    value: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    schema_transform(schema, out_schema, |schema| {
        extension::with_allowed_bit_mask(schema, value)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_required_bit_mask(
    schema: *mut kmipkit_extension_schema_t,
    value: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    schema_transform(schema, out_schema, |schema| {
        extension::with_required_bit_mask(schema, value)
    })
}

fn schema_transform(
    pointer: *mut kmipkit_extension_schema_t,
    out_schema: *mut *mut kmipkit_extension_schema_t,
    transform: impl FnOnce(
        extension::ExtensionSchema,
    ) -> Result<extension::ExtensionSchema, ProtocolError>,
) -> i32 {
    let out = out_slot!(out_schema);
    let schema = try_ffi!(owned_payload(pointer, Kind::Schema));
    let HandleValue::Schema(schema) = schema else {
        return ERROR_INVALID_INPUT;
    };
    let schema = try_ffi!(transform(schema).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::Schema, HandleValue::Schema(schema));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_create(
    identity: *mut kmipkit_extension_identity_t,
    compatibility: *mut kmipkit_extension_compatibility_t,
    discriminator: *mut kmipkit_extension_discriminator_t,
    schema: *mut kmipkit_extension_schema_t,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    let out = out_slot!(out_definition);
    let identity = read_handle!(identity, Identity);
    let compatibility = read_handle!(compatibility, Compatibility);
    let discriminator = read_handle!(discriminator, Discriminator);
    let schema = read_handle!(schema, Schema);
    let (
        HandleValue::Identity(identity),
        HandleValue::Compatibility(compatibility),
        HandleValue::Discriminator(discriminator),
        HandleValue::Schema(schema),
    ) = (
        &identity.value,
        &compatibility.value,
        &discriminator.value,
        &schema.value,
    )
    else {
        return ERROR_INVALID_INPUT;
    };
    // Discriminator scalar data is secret-bearing; the protocol helper makes a zeroizing copy.
    let discriminator = try_ffi!(
        extension::clone_extension_discriminator(discriminator).map_err(|e| protocol_status(&e))
    );
    let definition = try_ffi!(
        extension::extension_definition(
            identity.clone(),
            *compatibility,
            discriminator,
            schema.clone(),
        )
        .map_err(|e| protocol_status(&e))
    );
    *out = make_handle(
        Kind::Definition,
        HandleValue::Definition(Box::new(definition)),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_registry_limits_default_values(
    out_max_definitions: *mut u64,
    out_max_schema_nodes: *mut u64,
    out_max_child_rules_per_structure: *mut u64,
    out_max_text_bytes_per_field: *mut u64,
    out_max_registry_text_bytes: *mut u64,
    out_max_discriminator_scalar_bytes: *mut u64,
    out_max_total_discriminator_scalar_bytes: *mut u64,
    out_max_constraint_members_per_rule: *mut u64,
    out_max_total_constraint_members: *mut u64,
    out_max_payload_index_records: *mut u64,
    out_max_lookup_comparisons: *mut u64,
    out_max_depth: *mut u64,
) -> i32 {
    let outputs = [
        out_max_definitions,
        out_max_schema_nodes,
        out_max_child_rules_per_structure,
        out_max_text_bytes_per_field,
        out_max_registry_text_bytes,
        out_max_discriminator_scalar_bytes,
        out_max_total_discriminator_scalar_bytes,
        out_max_constraint_members_per_rule,
        out_max_total_constraint_members,
        out_max_payload_index_records,
        out_max_lookup_comparisons,
        out_max_depth,
    ];
    if outputs.iter().any(|output| output.is_null()) {
        return ERROR_INVALID_INPUT;
    }
    let limits = extension::defaults();
    let values = [
        limits.max_definitions(),
        limits.max_schema_nodes(),
        limits.max_child_rules_per_structure(),
        limits.max_text_bytes_per_field(),
        limits.max_registry_text_bytes(),
        limits.max_discriminator_scalar_bytes(),
        limits.max_total_discriminator_scalar_bytes(),
        limits.max_constraint_members_per_rule(),
        limits.max_total_constraint_members(),
        limits.max_payload_index_records(),
        limits.max_lookup_comparisons(),
        limits.max_depth(),
    ];
    for (output, value) in outputs.into_iter().zip(values) {
        // SAFETY: all output pointers were checked non-null and are writable C output slots.
        unsafe {
            *output = value;
        }
    }
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_registry_limits_validate(
    max_definitions: u64,
    max_schema_nodes: u64,
    max_child_rules_per_structure: u64,
    max_text_bytes_per_field: u64,
    max_registry_text_bytes: u64,
    max_discriminator_scalar_bytes: u64,
    max_total_discriminator_scalar_bytes: u64,
    max_constraint_members_per_rule: u64,
    max_total_constraint_members: u64,
    max_payload_index_records: u64,
    max_lookup_comparisons: u64,
    max_depth: u64,
) -> i32 {
    match limits_from_fields([
        max_definitions,
        max_schema_nodes,
        max_child_rules_per_structure,
        max_text_bytes_per_field,
        max_registry_text_bytes,
        max_discriminator_scalar_bytes,
        max_total_discriminator_scalar_bytes,
        max_constraint_members_per_rule,
        max_total_constraint_members,
        max_payload_index_records,
        max_lookup_comparisons,
        max_depth,
    ]) {
        Ok(_) => SUCCESS,
        Err(status) => status,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_create(
    definitions: *mut *mut kmipkit_extension_definition_t,
    definition_count: u64,
    max_definitions: u64,
    max_schema_nodes: u64,
    max_child_rules_per_structure: u64,
    max_text_bytes_per_field: u64,
    max_registry_text_bytes: u64,
    max_discriminator_scalar_bytes: u64,
    max_total_discriminator_scalar_bytes: u64,
    max_constraint_members_per_rule: u64,
    max_total_constraint_members: u64,
    max_payload_index_records: u64,
    max_lookup_comparisons: u64,
    max_depth: u64,
    out_registry: *mut *mut kmipkit_client_extension_registry_t,
) -> i32 {
    let out = out_slot!(out_registry);
    let limits = try_ffi!(limits_from_fields([
        max_definitions,
        max_schema_nodes,
        max_child_rules_per_structure,
        max_text_bytes_per_field,
        max_registry_text_bytes,
        max_discriminator_scalar_bytes,
        max_total_discriminator_scalar_bytes,
        max_constraint_members_per_rule,
        max_total_constraint_members,
        max_payload_index_records,
        max_lookup_comparisons,
        max_depth,
    ]));
    let copied = try_ffi!(input_handle_array(
        definitions,
        definition_count,
        max_definitions,
        Kind::Definition,
        |handle| {
            let definition = definition_from_handle(handle)?;
            #[cfg(test)]
            REGISTRY_DEFINITION_CLONE_COUNT.with(|count| count.set(count.get() + 1));
            extension::clone_extension_definition(definition)
                .map_err(|error| protocol_status(&error))
        },
    ));
    let registry = try_ffi!(
        client_extension::client_extension_registry(copied, limits)
            .map_err(|error| client_status(&error))
    );
    *out = make_handle(Kind::Registry, HandleValue::Registry(Arc::new(registry)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_configuration_create(
    extension_registry: *mut kmipkit_client_extension_registry_t,
    out_configuration: *mut *mut kmipkit_client_configuration_t,
) -> i32 {
    let out = out_slot!(out_configuration);
    let registry = try_ffi!(owned_payload(extension_registry, Kind::Registry));
    let HandleValue::Registry(registry) = registry else {
        return ERROR_INVALID_INPUT;
    };
    let configuration = client_extension::ClientConfiguration::from_shared_registry(registry);
    *out = make_handle(
        Kind::Configuration,
        HandleValue::Configuration(configuration),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_configuration_extension_registry(
    configuration: *mut kmipkit_client_configuration_t,
    out_registry: *mut *mut kmipkit_client_extension_registry_t,
) -> i32 {
    let out = out_slot!(out_registry);
    let configuration = read_handle!(configuration, Configuration);
    let HandleValue::Configuration(configuration) = &configuration.value else {
        return ERROR_INVALID_INPUT;
    };
    let registry = configuration.shared_extension_registry();
    *out = make_handle(Kind::Registry, HandleValue::Registry(registry));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_definition_count(
    registry: *mut kmipkit_client_extension_registry_t,
    out_count: *mut u64,
) -> i32 {
    let count = scalar_out!(out_count);
    let registry = try_ffi!(reference_registry(&registry));
    *count = client_extension::definition_count(registry);
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_definition_at(
    registry: *mut kmipkit_client_extension_registry_t,
    index: u64,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    let out = out_slot!(out_definition);
    let registry = try_ffi!(reference_registry(&registry));
    let Some(index) = usize::try_from(index).ok() else {
        return SUCCESS;
    };
    if client_extension::definition_at(registry, index).is_some() {
        *out = make_handle(
            Kind::Definition,
            HandleValue::DefinitionRef {
                registry: Arc::clone(registry),
                index,
            },
        );
    }
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_definition_for_identity(
    registry: *mut kmipkit_client_extension_registry_t,
    identity: *mut kmipkit_extension_identity_t,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    let out = out_slot!(out_definition);
    let registry = try_ffi!(reference_registry(&registry));
    let identity_handle = read_handle!(identity, Identity);
    let HandleValue::Identity(identity) = &identity_handle.value else {
        return ERROR_INVALID_INPUT;
    };
    let found = (0..client_extension::definition_count(registry)).find(|index| {
        usize::try_from(*index)
            .ok()
            .and_then(|position| client_extension::definition_at(registry, position))
            .is_some_and(|definition| definition.identity_ref() == identity)
    });
    if let Some(index) = found.and_then(|value| usize::try_from(value).ok()) {
        *out = make_handle(
            Kind::Definition,
            HandleValue::DefinitionRef {
                registry: Arc::clone(registry),
                index,
            },
        );
    }
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_validate(
    registry: *mut kmipkit_client_extension_registry_t,
    identity: *mut kmipkit_extension_identity_t,
    value: *mut kmipkit_ttlv_structure_t,
    limits: *mut kmipkit_codec_limits_t,
    out_value: *mut *mut kmipkit_registered_extension_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    let registry = try_ffi!(reference_registry(&registry));
    let identity_handle = read_handle!(identity, Identity);
    let structure_handle = read_handle!(value, Structure);
    let limits_handle = read_handle!(limits, CodecLimits);
    let (
        HandleValue::Identity(identity),
        HandleValue::Structure(value),
        HandleValue::CodecLimits(limits),
    ) = (
        &identity_handle.value,
        &structure_handle.value,
        &limits_handle.value,
    )
    else {
        return ERROR_INVALID_INPUT;
    };
    let value = try_ffi!(clone_structure(value));
    let registered = try_ffi!(
        client_extension::validate_extension_value(registry, identity.clone(), value, limits,)
            .map_err(|error| client_status(&error))
    );
    *out = make_handle(Kind::Registered, HandleValue::Registered(registered));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_inspect(
    registry: *mut kmipkit_client_extension_registry_t,
    vendor_identifier_data: *const u8,
    vendor_identifier_length: u64,
    value: *mut kmipkit_ttlv_structure_t,
    out_value: *mut *mut kmipkit_extension_recognition_t,
    limits: *mut kmipkit_codec_limits_t,
) -> i32 {
    let out = out_slot!(out_value);
    let registry = try_ffi!(reference_registry(&registry));
    let vendor = try_ffi!(input_span(
        vendor_identifier_data,
        vendor_identifier_length,
        client_extension::limits(registry).max_text_bytes_per_field()
    ));
    let vendor = try_ffi!(std::str::from_utf8(vendor).map_err(|_| ERROR_INVALID_INPUT));
    let structure_handle = read_handle!(value, Structure);
    let limits_handle = read_handle!(limits, CodecLimits);
    let (HandleValue::Structure(value), HandleValue::CodecLimits(limits)) =
        (&structure_handle.value, &limits_handle.value)
    else {
        return ERROR_INVALID_INPUT;
    };
    let value = try_ffi!(clone_structure(value));
    let recognition = try_ffi!(
        client_extension::inspect(registry, vendor, value, limits)
            .map_err(|error| client_status(&error))
    );
    *out = make_handle(Kind::Recognition, HandleValue::Recognition(recognition));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_validate(
    definition: *mut kmipkit_extension_definition_t,
    value: *mut kmipkit_ttlv_structure_t,
    out_value: *mut *mut kmipkit_validated_extension_value_t,
    limits: *mut kmipkit_codec_limits_t,
) -> i32 {
    let out = out_slot!(out_value);
    let definition_handle = read_handle!(definition, Definition);
    let definition = try_ffi!(definition_from_handle(definition_handle));
    let value_handle = read_handle!(value, Structure);
    let limits_handle = read_handle!(limits, CodecLimits);
    let (HandleValue::Structure(value), HandleValue::CodecLimits(limits)) =
        (&value_handle.value, &limits_handle.value)
    else {
        return ERROR_INVALID_INPUT;
    };
    let value = try_ffi!(clone_structure(value));
    let validated = try_ffi!(
        extension::validate(definition, value, limits).map_err(|error| protocol_status(&error))
    );
    *out = make_handle(Kind::Validated, HandleValue::Validated(validated));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_request_message_extension_create(
    value: *mut kmipkit_registered_extension_value_t,
    criticality_indicator: u8,
    out_extension: *mut *mut kmipkit_client_request_message_extension_t,
) -> i32 {
    let out = out_slot!(out_extension);
    let _ = read_handle!(value, Registered);
    let value = try_ffi!(owned_payload(value, Kind::Registered));
    let HandleValue::Registered(value) = value else {
        return ERROR_INVALID_INPUT;
    };
    if criticality_indicator > 1 {
        return ERROR_INVALID_INPUT;
    }
    let extension = try_ffi!(
        client_extension::client_request_message_extension(value, criticality_indicator == 1,)
            .map_err(|error| client_status(&error))
    );
    *out = make_handle(
        Kind::MessageExtension,
        HandleValue::MessageExtension(extension),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_with_extension(
    item: *mut kmipkit_client_batch_item_t,
    extension: *mut kmipkit_client_request_message_extension_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    let out = out_slot!(out_item);
    let _ = read_handle!(item, BatchItem);
    let _ = read_handle!(extension, MessageExtension);
    let item = consume_owner(item);
    let extension = consume_owner(extension);
    let item = try_ffi!(unwrap_payload(item));
    let extension = try_ffi!(unwrap_payload(extension));
    let (HandleValue::BatchItem(item), HandleValue::MessageExtension(extension)) =
        (item, extension)
    else {
        return ERROR_INVALID_INPUT;
    };
    *out = make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(item.with_extension(extension)),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_discover_versions(
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    let out = out_slot!(out_item);
    *out = make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(ClientBatchItem::discover_versions()),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_extension_count(
    item: *mut kmipkit_client_batch_item_t,
    out_count: *mut u64,
) -> i32 {
    let out = scalar_out!(out_count);
    let item = read_handle!(item, BatchItem);
    let HandleValue::BatchItem(item) = &item.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(u64::try_from(item.extension_count()).map_err(|_| ERROR_RESOURCE_LIMIT));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_extension_identity_at(
    item: *mut kmipkit_client_batch_item_t,
    index: u64,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    let out = out_slot!(out_identity);
    let item = read_handle!(item, BatchItem);
    let HandleValue::BatchItem(item) = &item.value else {
        return ERROR_INVALID_INPUT;
    };
    let Some(index) = usize::try_from(index).ok() else {
        return SUCCESS;
    };
    let Some(identity) = item.extension_identity_at(index) else {
        return SUCCESS;
    };
    *out = make_handle(Kind::Identity, HandleValue::Identity(identity));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_extension_criticality_indicator_at(
    item: *mut kmipkit_client_batch_item_t,
    index: u64,
    out_criticality_indicator: *mut u8,
) -> i32 {
    let out = scalar_out!(out_criticality_indicator);
    let item = read_handle!(item, BatchItem);
    let HandleValue::BatchItem(item) = &item.value else {
        return ERROR_INVALID_INPUT;
    };
    let Some(index) = usize::try_from(index).ok() else {
        return ERROR_INVALID_INPUT;
    };
    let Some(criticality_indicator) = item.extension_criticality_indicator_at(index) else {
        return ERROR_INVALID_INPUT;
    };
    *out = u8::from(criticality_indicator);
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_create(
    extension_name_data: *const u8,
    extension_name_length: u64,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    let out = out_slot!(out_information);
    let name = try_ffi!(input_span(
        extension_name_data,
        extension_name_length,
        MAX_TEXT_BYTES
    ));
    let name = try_ffi!(std::str::from_utf8(name).map_err(|_| ERROR_INVALID_IDENTITY));
    let information =
        try_ffi!(extension::extension_information(name).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::Information, HandleValue::Information(information));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_tag(
    information: *mut kmipkit_extension_information_t,
    tag: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    information_transform(information, out_information, |info| {
        extension::with_tag(info, tag).map_err(|error| protocol_status(&error))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_type(
    information: *mut kmipkit_extension_information_t,
    item_type: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    information_transform(information, out_information, |info| {
        let item_type = item_type_from_u32(item_type)?;
        extension::with_type(info, item_type).map_err(|error| protocol_status(&error))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_enumeration(
    information: *mut kmipkit_extension_information_t,
    enumeration: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    information_transform(information, out_information, |info| {
        extension::with_enumeration(info, enumeration).map_err(|error| protocol_status(&error))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_attribute(
    information: *mut kmipkit_extension_information_t,
    attribute: u8,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    information_transform(information, out_information, |info| {
        if attribute > 1 {
            return Err(ERROR_INVALID_INPUT);
        }
        extension::with_attribute(info, attribute == 1).map_err(|error| protocol_status(&error))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_parent_structure_tag(
    information: *mut kmipkit_extension_information_t,
    parent_structure_tag: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    information_transform(information, out_information, |info| {
        extension::with_parent_structure_tag(info, parent_structure_tag)
            .map_err(|error| protocol_status(&error))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_description(
    information: *mut kmipkit_extension_information_t,
    description_data: *const u8,
    description_length: u64,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    let out = out_slot!(out_information);
    let source = try_ffi!(owned_payload(information, Kind::Information));
    let HandleValue::Information(source) = source else {
        return ERROR_INVALID_INPUT;
    };
    let bytes = try_ffi!(input_span(
        description_data,
        description_length,
        MAX_TEXT_BYTES
    ));
    let description = try_ffi!(std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT));
    let information =
        try_ffi!(extension::with_description(source, description).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::Information, HandleValue::Information(information));
    SUCCESS
}

fn information_transform(
    pointer: *mut kmipkit_extension_information_t,
    out_information: *mut *mut kmipkit_extension_information_t,
    transform: impl FnOnce(
        extension::ExtensionInformation,
    ) -> FfiResult<extension::ExtensionInformation>,
) -> i32 {
    let out = out_slot!(out_information);
    let information = try_ffi!(owned_payload(pointer, Kind::Information));
    let HandleValue::Information(information) = information else {
        return ERROR_INVALID_INPUT;
    };
    let information = try_ffi!(transform(information));
    *out = make_handle(Kind::Information, HandleValue::Information(information));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_to_ttlv(
    information: *mut kmipkit_extension_information_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    let out = out_slot!(out_structure);
    let information = read_handle!(information, Information);
    let HandleValue::Information(information) = &information.value else {
        return ERROR_INVALID_INPUT;
    };
    let structure =
        try_ffi!(extension::to_ttlv(information.clone()).map_err(|e| protocol_status(&e)));
    *out = make_handle(Kind::Structure, HandleValue::Structure(structure));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_with_information(
    definition: *mut kmipkit_extension_definition_t,
    information: *mut kmipkit_extension_information_t,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    let out = out_slot!(out_definition);
    let information = read_handle!(information, Information);
    let definition = try_ffi!(owned_payload(definition, Kind::Definition));
    let HandleValue::Definition(definition) = definition else {
        return ERROR_INVALID_INPUT;
    };
    let HandleValue::Information(information) = &information.value else {
        return ERROR_INVALID_INPUT;
    };
    let definition = try_ffi!(
        extension::with_information(*definition, information.clone())
            .map_err(|e| protocol_status(&e))
    );
    *out = make_handle(
        Kind::Definition,
        HandleValue::Definition(Box::new(definition)),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_identity(
    definition: *mut kmipkit_extension_definition_t,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    let out = out_slot!(out_identity);
    let definition = read_handle!(definition, Definition);
    let definition = try_ffi!(definition_from_handle(definition));
    *out = make_handle(
        Kind::Identity,
        HandleValue::Identity(extension::identity(definition)),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_information(
    definition: *mut kmipkit_extension_definition_t,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    let out = out_slot!(out_information);
    let definition = read_handle!(definition, Definition);
    let definition = try_ffi!(definition_from_handle(definition));
    if let Some(information) = extension::information(definition) {
        *out = make_handle(Kind::Information, HandleValue::Information(information));
    }
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_is_recognized(
    recognition: *mut kmipkit_extension_recognition_t,
    out_recognized: *mut u32,
) -> i32 {
    let out = scalar_out!(out_recognized);
    let recognition = read_handle!(recognition, Recognition);
    let HandleValue::Recognition(recognition) = &recognition.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = u32::from(recognition.is_recognized());
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_validated_value(
    recognition: *mut kmipkit_extension_recognition_t,
    out_value: *mut *mut kmipkit_validated_extension_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    let recognition_handle = read_handle!(recognition, Recognition);
    let HandleValue::Recognition(recognition_value) = &recognition_handle.value else {
        return ERROR_INVALID_INPUT;
    };
    if recognition_value.validated_value().is_some() {
        let owner = try_ffi!(clone_handle_owner(recognition, Kind::Recognition));
        *out = make_handle(Kind::Validated, HandleValue::ValidatedRef(owner));
    }
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_generic_value(
    recognition: *mut kmipkit_extension_recognition_t,
    out_value: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    let out = out_slot!(out_value);
    let _ = read_handle!(recognition, Recognition);
    let owner = try_ffi!(clone_handle_owner(recognition, Kind::Recognition));
    if !matches!(owner.value, HandleValue::Recognition(_)) {
        return ERROR_INVALID_INPUT;
    }
    let view = TtlvView::new(owner, ViewRoot::RecognitionGeneric, ViewTarget::Structure);
    *out = make_handle(Kind::StructureView, HandleValue::StructureView(view));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_identity(
    value: *mut kmipkit_validated_extension_value_t,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    let out = out_slot!(out_identity);
    let value = read_handle!(value, Validated);
    let value = try_ffi!(validated_from_handle(value));
    *out = make_handle(
        Kind::Identity,
        HandleValue::Identity(extension::validated_extension_value_identity(value)),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_generic_value(
    value: *mut kmipkit_validated_extension_value_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    let out = out_slot!(out_structure);
    let value_handle = read_handle!(value, Validated);
    let _ = try_ffi!(validated_from_handle(value_handle));
    let owner = try_ffi!(clone_handle_owner(value, Kind::Validated));
    let structure = TtlvView::new(owner, ViewRoot::ValidatedGeneric, ViewTarget::Structure);
    *out = make_handle(Kind::StructureView, HandleValue::StructureView(structure));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_value_at(
    value: *mut kmipkit_validated_extension_value_t,
    path: *mut kmipkit_ttlv_path_t,
    out_value: *mut *mut kmipkit_ttlv_value_view_t,
) -> i32 {
    let out = out_slot!(out_value);
    let _value_handle = read_handle!(value, Validated);
    let path_handle = read_handle!(path, Path);
    let HandleValue::Path(path) = &path_handle.value else {
        return ERROR_INVALID_INPUT;
    };
    let owner = try_ffi!(clone_handle_owner(value, Kind::Validated));
    let structure = TtlvView::new(owner, ViewRoot::ValidatedGeneric, ViewTarget::Structure);
    let value_view = try_ffi!(structure.value_at_tags(path.tags()));
    *out = make_handle(Kind::ValueView, HandleValue::ValueView(value_view));
    SUCCESS
}

fn padded_length(value_length: u64) -> FfiResult<u64> {
    value_length
        .checked_add(7)
        .map(|value| value & !7)
        .ok_or(ERROR_RESOURCE_LIMIT)
}

fn structure_measure(structure: &kmipkit_ttlv::StructureView<'_>) -> FfiResult<(u64, u64, u64)> {
    let mut value_bytes = 0_u64;
    let mut elements = 0_u64;
    let mut max_depth = 1_u64;
    for item in structure.children() {
        let (payload_bytes, nested_elements, nested_depth) = item.with_value(measure_value)?;
        let encoded = 8_u64
            .checked_add(padded_length(payload_bytes)?)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        value_bytes = value_bytes
            .checked_add(encoded)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        elements = elements
            .checked_add(1)
            .and_then(|total| total.checked_add(nested_elements))
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        max_depth = max_depth.max(nested_depth.saturating_add(u64::from(nested_depth > 0)));
    }
    Ok((value_bytes, elements, max_depth))
}

fn measure_value(value: ValueView<'_>) -> FfiResult<(u64, u64, u64)> {
    match value {
        ValueView::Structure(structure) => structure_measure(&structure),
        ValueView::Integer(_) | ValueView::Enumeration(_) | ValueView::Interval(_) => Ok((4, 0, 0)),
        ValueView::LongInteger(_) | ValueView::DateTime(_) | ValueView::DateTimeExtended(_) => {
            Ok((8, 0, 0))
        }
        ValueView::Boolean(_) => Ok((8, 0, 0)),
        ValueView::BigInteger(bytes) | ValueView::ByteString(bytes) => Ok((
            u64::try_from(bytes.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?,
            0,
            0,
        )),
        ValueView::TextString(text) => Ok((
            u64::try_from(text.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?,
            0,
            0,
        )),
        _ => Err(ERROR_INVALID_SCHEMA),
    }
}

fn check_structure_limits(structure: &Structure, limits: &CodecLimits) -> FfiResult<()> {
    let (payload_bytes, nested_elements, depth) = structure_measure(&structure.view())?;
    let encoded_size = 8_u64
        .checked_add(payload_bytes)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    let total_elements = 1_u64
        .checked_add(nested_elements)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    if encoded_size > u64::try_from(limits.max_message_bytes()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || total_elements
            > u64::try_from(limits.max_elements()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || depth > u64::try_from(limits.max_structure_depth()).map_err(|_| ERROR_RESOURCE_LIMIT)?
    {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    Ok(())
}

fn check_item_limits(item: &Item, limits: &CodecLimits) -> FfiResult<()> {
    let (payload_bytes, nested_elements, depth) = item.with_value(measure_value)?;
    let encoded_size = 8_u64
        .checked_add(padded_length(payload_bytes)?)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    let total_elements = 1_u64
        .checked_add(nested_elements)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    if encoded_size > u64::try_from(limits.max_message_bytes()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || total_elements
            > u64::try_from(limits.max_elements()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || depth > u64::try_from(limits.max_structure_depth()).map_err(|_| ERROR_RESOURCE_LIMIT)?
    {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    Ok(())
}

fn codec_limits_from_handle(pointer: &*mut kmipkit_codec_limits_t) -> FfiResult<&CodecLimits> {
    let handle = reference_handle(pointer, Kind::CodecLimits)?;
    match &handle.value {
        HandleValue::CodecLimits(limits) => Ok(limits),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_raw_tag_create(
    raw: u32,
    out_raw_tag: *mut *mut kmipkit_raw_tag_t,
) -> i32 {
    let out = out_slot!(out_raw_tag);
    let raw = try_ffi!(RawTag::new(raw).map_err(model_status));
    *out = make_handle(Kind::RawTag, HandleValue::RawTag(raw));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_raw_tag_value(
    raw_tag: *mut kmipkit_raw_tag_t,
    out_raw: *mut u32,
) -> i32 {
    let out = scalar_out!(out_raw);
    let raw_tag = read_handle!(raw_tag, RawTag);
    let HandleValue::RawTag(raw_tag) = &raw_tag.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = raw_tag.raw();
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_raw_tag_try_checked(
    raw_tag: *mut kmipkit_raw_tag_t,
    out_tag: *mut *mut kmipkit_tag_t,
) -> i32 {
    let out = out_slot!(out_tag);
    let raw_tag = read_handle!(raw_tag, RawTag);
    let HandleValue::RawTag(raw_tag) = &raw_tag.value else {
        return ERROR_INVALID_INPUT;
    };
    let tag = try_ffi!(raw_tag.try_checked().map_err(model_status));
    *out = make_handle(Kind::Tag, HandleValue::Tag(tag));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_tag_value(tag: *mut kmipkit_tag_t, out_raw: *mut u32) -> i32 {
    let out = scalar_out!(out_raw);
    let tag = read_handle!(tag, Tag);
    let HandleValue::Tag(tag) = &tag.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = tag.raw();
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_defaults(
    out_limits: *mut *mut kmipkit_codec_limits_t,
) -> i32 {
    let out = out_slot!(out_limits);
    *out = make_handle(
        Kind::CodecLimits,
        HandleValue::CodecLimits(CodecLimits::defaults()),
    );
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_create(
    max_message_bytes: u64,
    max_structure_depth: u64,
    max_elements: u64,
    out_limits: *mut *mut kmipkit_codec_limits_t,
) -> i32 {
    let out = out_slot!(out_limits);
    let max_message_bytes =
        try_ffi!(usize::try_from(max_message_bytes).map_err(|_| ERROR_RESOURCE_LIMIT));
    let max_structure_depth =
        try_ffi!(usize::try_from(max_structure_depth).map_err(|_| ERROR_RESOURCE_LIMIT));
    let max_elements = try_ffi!(usize::try_from(max_elements).map_err(|_| ERROR_RESOURCE_LIMIT));
    let limits = try_ffi!(
        CodecLimits::new(max_message_bytes, max_structure_depth, max_elements)
            .map_err(|_| ERROR_RESOURCE_LIMIT)
    );
    *out = make_handle(Kind::CodecLimits, HandleValue::CodecLimits(limits));
    SUCCESS
}

fn codec_limit_value(
    pointer: *mut kmipkit_codec_limits_t,
    out_value: *mut u64,
    selector: fn(&CodecLimits) -> usize,
) -> i32 {
    let output = scalar_out!(out_value);
    let limits = try_ffi!(codec_limits_from_handle(&pointer));
    *output = try_ffi!(u64::try_from(selector(limits)).map_err(|_| ERROR_RESOURCE_LIMIT));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_max_message_bytes(
    limits: *mut kmipkit_codec_limits_t,
    max_message_bytes: *mut u64,
) -> i32 {
    codec_limit_value(limits, max_message_bytes, CodecLimits::max_message_bytes)
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_max_structure_depth(
    limits: *mut kmipkit_codec_limits_t,
    max_structure_depth: *mut u64,
) -> i32 {
    codec_limit_value(
        limits,
        max_structure_depth,
        CodecLimits::max_structure_depth,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_max_elements(
    limits: *mut kmipkit_codec_limits_t,
    max_elements: *mut u64,
) -> i32 {
    codec_limit_value(limits, max_elements, CodecLimits::max_elements)
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_integer(
    value: i32,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    *out = make_handle(Kind::Value, HandleValue::Value(Value::integer(value)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_long_integer(
    value: i64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    *out = make_handle(Kind::Value, HandleValue::Value(Value::long_integer(value)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_enumeration(
    value: u32,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    *out = make_handle(Kind::Value, HandleValue::Value(Value::enumeration(value)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_boolean(
    value: u8,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    if value > 1 {
        return ERROR_INVALID_INPUT;
    }
    *out = make_handle(Kind::Value, HandleValue::Value(Value::boolean(value == 1)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_date_time(
    value: i64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    *out = make_handle(Kind::Value, HandleValue::Value(Value::date_time(value)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_interval(
    value: u32,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    *out = make_handle(Kind::Value, HandleValue::Value(Value::interval(value)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_date_time_extended(
    value: i64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    *out = make_handle(
        Kind::Value,
        HandleValue::Value(Value::date_time_extended(value)),
    );
    SUCCESS
}

fn variable_value(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
    build: fn(&[u8]) -> FfiResult<Value>,
) -> i32 {
    let out = out_slot!(out_value);
    let limits = try_ffi!(codec_limits_from_handle(&limits));
    let maximum =
        try_ffi!(u64::try_from(limits.max_message_bytes()).map_err(|_| ERROR_RESOURCE_LIMIT));
    let bytes = try_ffi!(input_span(value, value_length, maximum));
    let value = try_ffi!(build(bytes));
    *out = make_handle(Kind::Value, HandleValue::Value(value));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_big_integer(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    variable_value(limits, value, value_length, out_value, |bytes| {
        Ok(Value::big_integer(bytes.to_vec()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_text_string(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    variable_value(limits, value, value_length, out_value, |bytes| {
        let text = std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)?;
        Ok(Value::text_string(text.to_owned()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_byte_string(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    variable_value(limits, value, value_length, out_value, |bytes| {
        Ok(Value::byte_string(bytes.to_vec()))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_structure(
    structure: *mut kmipkit_ttlv_structure_t,
    limits: *mut kmipkit_codec_limits_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    let out = out_slot!(out_value);
    let limits = try_ffi!(codec_limits_from_handle(&limits));
    let structure = try_ffi!(owned_payload(structure, Kind::Structure));
    let HandleValue::Structure(structure) = structure else {
        return ERROR_INVALID_INPUT;
    };
    try_ffi!(check_structure_limits(&structure, limits));
    *out = make_handle(Kind::Value, HandleValue::Value(Value::structure(structure)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_create(
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    let out = out_slot!(out_structure);
    *out = make_handle(Kind::Structure, HandleValue::Structure(Structure::new()));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_create(
    tag: *mut kmipkit_tag_t,
    value: *mut kmipkit_ttlv_value_t,
    limits: *mut kmipkit_codec_limits_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    let out = out_slot!(out_item);
    let tag_handle = read_handle!(tag, Tag);
    let HandleValue::Tag(tag) = &tag_handle.value else {
        return ERROR_INVALID_INPUT;
    };
    let limits = try_ffi!(codec_limits_from_handle(&limits));
    let value = try_ffi!(owned_payload(value, Kind::Value));
    let HandleValue::Value(value) = value else {
        return ERROR_INVALID_INPUT;
    };
    let item = try_ffi!(Item::new(*tag, value).map_err(model_status));
    try_ffi!(check_item_limits(&item, limits));
    *out = make_handle(Kind::Item, HandleValue::Item(item));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_with_item(
    structure: *mut kmipkit_ttlv_structure_t,
    item: *mut kmipkit_ttlv_item_t,
    limits: *mut kmipkit_codec_limits_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    let out = out_slot!(out_structure);
    let limits = try_ffi!(codec_limits_from_handle(&limits));
    let _ = read_handle!(structure, Structure);
    let _ = read_handle!(item, Item);
    let structure = consume_owner(structure);
    let item = consume_owner(item);
    let structure = try_ffi!(unwrap_payload(structure));
    let item = try_ffi!(unwrap_payload(item));
    let (HandleValue::Structure(mut structure), HandleValue::Item(item)) = (structure, item) else {
        return ERROR_INVALID_INPUT;
    };
    try_ffi!(structure.try_push(item).map_err(model_status));
    try_ffi!(check_structure_limits(&structure, limits));
    *out = make_handle(Kind::Structure, HandleValue::Structure(structure));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view(
    structure: *mut kmipkit_ttlv_structure_t,
    out_view: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    let out = out_slot!(out_view);
    let _ = read_handle!(structure, Structure);
    let owner = try_ffi!(clone_handle_owner(structure, Kind::Structure));
    let view = TtlvView::new(owner, ViewRoot::Structure, ViewTarget::Structure);
    *out = make_handle(Kind::StructureView, HandleValue::StructureView(view));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view_item_count(
    view: *mut kmipkit_ttlv_structure_view_t,
    out_count: *mut u64,
) -> i32 {
    let out = scalar_out!(out_count);
    let view = read_handle!(view, StructureView);
    let HandleValue::StructureView(view) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    let count = try_ffi!(view.with_structure(|structure| structure.children().len()));
    *out = try_ffi!(u64::try_from(count).map_err(|_| ERROR_RESOURCE_LIMIT));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view_item_at(
    view: *mut kmipkit_ttlv_structure_view_t,
    index: u64,
    out_item: *mut *mut kmipkit_ttlv_item_view_t,
) -> i32 {
    let out = out_slot!(out_item);
    let view = read_handle!(view, StructureView);
    let HandleValue::StructureView(view) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_INVALID_INPUT));
    let item = try_ffi!(view.item_at(index));
    *out = make_handle(Kind::ItemView, HandleValue::ItemView(item));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_tag(
    view: *mut kmipkit_ttlv_item_view_t,
    out_tag: *mut *mut kmipkit_tag_t,
) -> i32 {
    let out = out_slot!(out_tag);
    let view = read_handle!(view, ItemView);
    let HandleValue::ItemView(item) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    let tag = try_ffi!(item.with_item(Item::tag));
    *out = make_handle(Kind::Tag, HandleValue::Tag(tag));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_type(
    view: *mut kmipkit_ttlv_item_view_t,
    out_item_type: *mut u8,
) -> i32 {
    let out = scalar_out!(out_item_type);
    let view = read_handle!(view, ItemView);
    let HandleValue::ItemView(item) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    let item_type = try_ffi!(item.with_item(Item::item_type));
    *out = item_type_code(item_type);
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_value(
    view: *mut kmipkit_ttlv_item_view_t,
    out_value: *mut *mut kmipkit_ttlv_value_view_t,
) -> i32 {
    let out = out_slot!(out_value);
    let view = read_handle!(view, ItemView);
    let HandleValue::ItemView(item) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    let value = try_ffi!(item.item_value());
    *out = make_handle(Kind::ValueView, HandleValue::ValueView(value));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view(
    value: *mut kmipkit_ttlv_value_t,
    out_view: *mut *mut kmipkit_ttlv_value_view_t,
) -> i32 {
    let out = out_slot!(out_view);
    let _ = read_handle!(value, Value);
    let owner = try_ffi!(clone_handle_owner(value, Kind::Value));
    let view = TtlvView::new(owner, ViewRoot::Value, ViewTarget::Value);
    *out = make_handle(Kind::ValueView, HandleValue::ValueView(view));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_type(
    view: *mut kmipkit_ttlv_value_view_t,
    out_item_type: *mut u8,
) -> i32 {
    let out = scalar_out!(out_item_type);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(value.with_value(|value| Ok(value_view_type_code(&value))));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_structure(
    view: *mut kmipkit_ttlv_value_view_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    let out = out_slot!(out_structure);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    let structure = try_ffi!(value.structure_from_value());
    *out = make_handle(Kind::StructureView, HandleValue::StructureView(structure));
    SUCCESS
}

fn extract_value<T: Copy>(
    value: &TtlvView,
    extract: impl FnOnce(ValueView<'_>) -> Option<T>,
) -> FfiResult<T> {
    value.with_value(|value| extract(value).ok_or(ERROR_INVALID_INPUT))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_integer(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i32,
) -> i32 {
    let out = scalar_out!(out_value);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::Integer(v) => Some(*v),
        _ => None,
    }));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_long_integer(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i64,
) -> i32 {
    let out = scalar_out!(out_value);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::LongInteger(v) => Some(*v),
        _ => None,
    }));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_enumeration(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut u32,
) -> i32 {
    let out = scalar_out!(out_value);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::Enumeration(v) => Some(*v),
        _ => None,
    }));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_boolean(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut u8,
) -> i32 {
    let out = scalar_out!(out_value);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = u8::from(try_ffi!(extract_value(value, |value| match value {
        ValueView::Boolean(v) => Some(*v),
        _ => None,
    })));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_date_time(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i64,
) -> i32 {
    let out = scalar_out!(out_value);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::DateTime(v) => Some(*v),
        _ => None,
    }));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_interval(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut u32,
) -> i32 {
    let out = scalar_out!(out_value);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::Interval(v) => Some(*v),
        _ => None,
    }));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_date_time_extended(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i64,
) -> i32 {
    let out = scalar_out!(out_value);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::DateTimeExtended(v) => Some(*v),
        _ => None,
    }));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_byte_length(
    view: *mut kmipkit_ttlv_value_view_t,
    out_length: *mut u64,
) -> i32 {
    let out = scalar_out!(out_length);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::BigInteger(bytes) | ValueView::ByteString(bytes) =>
            u64::try_from(bytes.len()).ok(),
        ValueView::TextString(text) => u64::try_from(text.len()).ok(),
        _ => None,
    }));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_byte_at(
    view: *mut kmipkit_ttlv_value_view_t,
    index: u64,
    out_byte: *mut u8,
) -> i32 {
    let out = scalar_out!(out_byte);
    let view = read_handle!(view, ValueView);
    let HandleValue::ValueView(value) = &view.value else {
        return ERROR_INVALID_INPUT;
    };
    let Some(index) = usize::try_from(index).ok() else {
        return ERROR_INVALID_INPUT;
    };
    *out = try_ffi!(extract_value(value, |value| match value {
        ValueView::BigInteger(bytes) | ValueView::ByteString(bytes) => bytes.get(index).copied(),
        ValueView::TextString(text) => text.as_bytes().get(index).copied(),
        _ => None,
    }));
    SUCCESS
}
