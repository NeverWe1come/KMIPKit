//! Schema-validated extension values retaining their original generic tree.

use std::collections::HashMap;
use std::fmt;

use kmipkit_ttlv::{Item, Structure, StructureView, ValueView, codec::CodecLimits};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

use super::definition::{ExtensionDefinition, ExtensionIdentity};
use super::schema::{
    Cardinality, CompiledOrderEdge, ExtensionSchema, SchemaKind, checked_usize_counter_add,
};

#[cfg(test)]
#[derive(Debug, Default)]
struct ValidationMetrics {
    schema_tag_comparisons_per_item: Vec<usize>,
    enum_comparisons_per_item: Vec<usize>,
    order_edge_checks_per_structure: Vec<usize>,
    order_edge_work_per_structure: Vec<usize>,
    occurrence_entries_per_structure: Vec<usize>,
}

#[cfg(not(test))]
#[derive(Default)]
struct ValidationMetrics {
    _marker: std::marker::PhantomData<()>,
}

/// An extension value that can only be created after schema validation.
///
/// The original generic TTLV Structure remains owned and unmodified. Formatting
/// redacts it because extension payloads may contain secrets.
pub struct ValidatedExtensionValue {
    identity: ExtensionIdentity,
    generic_value: Structure,
}

impl fmt::Debug for ValidatedExtensionValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ValidatedExtensionValue([REDACTED])")
    }
}

impl fmt::Display for ValidatedExtensionValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("validated extension value ([REDACTED])")
    }
}

/// Validates an owned generic extension Structure against its definition.
///
/// The input tree is retained without rebuilding, sorting, or normalizing it.
/// Configured TTLV size, nesting-depth, and element-count limits are checked
/// before a validated value is returned.
///
/// # Errors
///
/// Returns `InvalidSchema` with a payload-free Tag path for a schema or
/// discriminator mismatch, and `ResourceLimit` when a configured TTLV limit is
/// exceeded. No partial validated value is produced.
pub fn validate(
    definition: &ExtensionDefinition,
    value: Structure,
    limits: &CodecLimits,
) -> Result<ValidatedExtensionValue, ProtocolError> {
    validate_inner(definition, value, limits).map(|(validated, _)| validated)
}

#[cfg(test)]
fn validate_with_metrics(
    definition: &ExtensionDefinition,
    value: Structure,
    limits: &CodecLimits,
) -> Result<(ValidatedExtensionValue, ValidationMetrics), ProtocolError> {
    validate_inner(definition, value, limits)
}

fn validate_inner(
    definition: &ExtensionDefinition,
    value: Structure,
    limits: &CodecLimits,
) -> Result<(ValidatedExtensionValue, ValidationMetrics), ProtocolError> {
    let mut metrics = ValidationMetrics::default();
    let mut path = Vec::new();
    path.try_reserve(limits.max_structure_depth().min(64))
        .map_err(|_| resource_limit())?;
    validate_ttlv_limits(&value, limits, &mut path)?;

    let root_view = value.view();
    validate_discriminator(&root_view, definition.discriminator(), &mut path)?;
    path.clear();
    validate_structure(&definition.schema, &root_view, &mut path, &mut metrics)?;

    Ok((
        ValidatedExtensionValue {
            identity: definition.identity.clone(),
            generic_value: value,
        },
        metrics,
    ))
}

fn validate_ttlv_limits(
    value: &Structure,
    limits: &CodecLimits,
    path: &mut Vec<kmipkit_ttlv::Tag>,
) -> Result<(), ProtocolError> {
    let mut item_count = 1_usize;
    if item_count > limits.max_elements() {
        return Err(resource_limit_at(path));
    }
    let encoded_size = account_structure(&value.view(), 1, &mut item_count, limits, path)?;
    if encoded_size > limits.max_message_bytes() {
        return Err(resource_limit_at(path));
    }
    Ok(())
}

/// Returns the identity associated with a validated value.
#[must_use]
pub fn validated_extension_value_identity(value: &ValidatedExtensionValue) -> ExtensionIdentity {
    value.identity.clone()
}

/// Borrows the unmodified generic TTLV Structure retained by a validated value.
#[must_use]
pub const fn generic_value(value: &ValidatedExtensionValue) -> &Structure {
    &value.generic_value
}

fn validate_discriminator(
    root: &StructureView<'_>,
    discriminator: &super::definition::Discriminator,
    path: &mut Vec<kmipkit_ttlv::Tag>,
) -> Result<(), ProtocolError> {
    validate_discriminator_at(root, discriminator.path().tags(), 0, discriminator, path)
}

fn validate_discriminator_at(
    current: &StructureView<'_>,
    tags: &[kmipkit_ttlv::Tag],
    index: usize,
    discriminator: &super::definition::Discriminator,
    path: &mut Vec<kmipkit_ttlv::Tag>,
) -> Result<(), ProtocolError> {
    let Some(tag) = tags.get(index).copied() else {
        return Err(invalid_schema(path));
    };
    let mut found: Option<&Item> = None;
    for item in current.children() {
        if item.tag() == tag {
            if found.is_some() {
                path.push(tag);
                return Err(invalid_schema(path));
            }
            found = Some(item);
        }
    }
    path.push(tag);
    let Some(item) = found else {
        return Err(invalid_schema(path));
    };
    if index + 1 == tags.len() {
        let matches = item.with_value(|value| discriminator.matches_value(value));
        return if matches {
            Ok(())
        } else {
            Err(invalid_schema(path))
        };
    }
    item.with_value(|value| match value {
        ValueView::Structure(nested) => {
            validate_discriminator_at(&nested, tags, index + 1, discriminator, path)
        }
        _ => Err(invalid_schema(path)),
    })
}

fn validate_structure(
    schema: &ExtensionSchema,
    value: &StructureView<'_>,
    path: &mut Vec<kmipkit_ttlv::Tag>,
    metrics: &mut ValidationMetrics,
) -> Result<(), ProtocolError> {
    let SchemaKind::Structure {
        children: rules,
        child_tag_index,
        required_child_indices,
        order_edges,
        ..
    } = &schema.kind
    else {
        return Err(invalid_schema(path));
    };
    let mut occurrences = HashMap::<usize, ChildOccurrences>::new();
    let mut present_rule_indices = Vec::new();

    for (position, item) in value.children().iter().enumerate() {
        let (index, comparisons) = find_rule_index(rules, child_tag_index, item.tag());
        record_schema_comparisons(metrics, comparisons)?;
        let Some(index) = index else {
            if schema.preserve_undeclared_children() {
                continue;
            }
            path.push(item.tag());
            return Err(invalid_schema(path));
        };
        let occurrence =
            record_child_occurrence(&mut occurrences, &mut present_rule_indices, index, path)?;
        let rule = &rules[index];
        if rule.cardinality != Cardinality::Repeated && occurrence.count != 0 {
            path.push(rule.tag);
            return Err(invalid_schema(path));
        }
        occurrence.count =
            checked_usize_counter_add(occurrence.count, 1).ok_or_else(resource_limit)?;
        if occurrence.first.is_none() {
            occurrence.first = Some(position);
        }
        occurrence.last = Some(position);
        path.push(rule.tag);
        item.with_value(|child_value| validate_value(&rule.schema, &child_value, path, metrics))?;
        path.pop();
    }

    present_rule_indices.sort_unstable();
    record_occurrence_entries(metrics, present_rule_indices.len())?;

    for index in required_child_indices {
        let rule = rules.get(*index).ok_or_else(|| invalid_schema(path))?;
        if !occurrences.contains_key(index) {
            path.push(rule.tag);
            return Err(invalid_schema(path));
        }
    }

    validate_order_edges(
        order_edges,
        &present_rule_indices,
        &occurrences,
        path,
        metrics,
    )?;
    Ok(())
}

fn validate_value(
    schema: &ExtensionSchema,
    value: &ValueView<'_>,
    path: &mut Vec<kmipkit_ttlv::Tag>,
    metrics: &mut ValidationMetrics,
) -> Result<(), ProtocolError> {
    match value {
        ValueView::Structure(structure)
            if schema.item_type() == kmipkit_ttlv::ItemType::Structure =>
        {
            validate_structure(schema, structure, path, metrics)
        }
        _ if value_item_type(value) == Some(schema.item_type()) => {
            validate_scalar(schema, value, path, metrics)
        }
        _ => Err(invalid_schema(path)),
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ChildOccurrences {
    count: usize,
    first: Option<usize>,
    last: Option<usize>,
}

fn record_child_occurrence<'a>(
    occurrences: &'a mut HashMap<usize, ChildOccurrences>,
    present_rule_indices: &mut Vec<usize>,
    rule_index: usize,
    path: &[kmipkit_ttlv::Tag],
) -> Result<&'a mut ChildOccurrences, ProtocolError> {
    if !occurrences.contains_key(&rule_index) {
        occurrences.try_reserve(1).map_err(|_| resource_limit())?;
        present_rule_indices
            .try_reserve(1)
            .map_err(|_| resource_limit())?;
        occurrences.insert(rule_index, ChildOccurrences::default());
        present_rule_indices.push(rule_index);
    }
    occurrences
        .get_mut(&rule_index)
        .ok_or_else(|| invalid_schema(path))
}

fn find_rule_index(
    rules: &[super::schema::ExtensionChildRule],
    child_tag_index: &[usize],
    tag: kmipkit_ttlv::Tag,
) -> (Option<usize>, usize) {
    let mut low = 0_usize;
    let mut high = child_tag_index.len();
    let mut comparisons = 0_usize;
    while low < high {
        let middle = low + (high - low) / 2;
        let Some(rule_index) = child_tag_index.get(middle).copied() else {
            return (None, comparisons);
        };
        let Some(rule) = rules.get(rule_index) else {
            return (None, comparisons);
        };
        comparisons += 1;
        match rule.tag.raw().cmp(&tag.raw()) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Equal => return (Some(rule_index), comparisons),
            std::cmp::Ordering::Greater => high = middle,
        }
    }
    (None, comparisons)
}

fn check_order_edge(
    edge: &CompiledOrderEdge,
    occurrences: &HashMap<usize, ChildOccurrences>,
    path: &[kmipkit_ttlv::Tag],
) -> Result<bool, ProtocolError> {
    let before = occurrences
        .get(&edge.before_index)
        .copied()
        .ok_or_else(|| invalid_schema(path))?;
    let after = occurrences
        .get(&edge.after_index)
        .copied()
        .ok_or_else(|| invalid_schema(path))?;
    Ok(!(before.last.is_some() && after.first.is_some() && before.last >= after.first))
}

fn validate_order_edges(
    order_edges: &[CompiledOrderEdge],
    present_rule_indices: &[usize],
    occurrences: &HashMap<usize, ChildOccurrences>,
    path: &mut Vec<kmipkit_ttlv::Tag>,
    metrics: &mut ValidationMetrics,
) -> Result<(), ProtocolError> {
    let pair_count = present_rule_indices
        .len()
        .checked_mul(present_rule_indices.len().saturating_sub(1))
        .and_then(|count| count.checked_div(2))
        .ok_or_else(resource_limit)?;
    let pair_search_count = pair_count.checked_mul(2).ok_or_else(resource_limit)?;
    let mut checks = 0_usize;
    let mut work = 0_usize;
    let mut first_invalid_edge: Option<usize> = None;

    // Search present pairs for sparse structures; scan the compiled edge list
    // when that is the smaller bounded candidate set.
    if pair_search_count < order_edges.len() {
        for before_position in 0..present_rule_indices.len() {
            let before_index = *present_rule_indices
                .get(before_position)
                .ok_or_else(|| invalid_schema(path))?;
            for after_position in before_position + 1..present_rule_indices.len() {
                let after_index = *present_rule_indices
                    .get(after_position)
                    .ok_or_else(|| invalid_schema(path))?;
                for (before, after) in [(before_index, after_index), (after_index, before_index)] {
                    let (edge_position, comparisons) = find_order_edge(order_edges, before, after);
                    work =
                        checked_usize_counter_add(work, comparisons).ok_or_else(resource_limit)?;
                    if let Some(edge_position) = edge_position {
                        let edge = order_edges
                            .get(edge_position)
                            .ok_or_else(|| invalid_schema(path))?;
                        if !check_order_edge(edge, occurrences, path)? {
                            first_invalid_edge = Some(
                                first_invalid_edge
                                    .map_or(edge_position, |first| first.min(edge_position)),
                            );
                        }
                        checks = checked_usize_counter_add(checks, 1).ok_or_else(resource_limit)?;
                    }
                }
            }
        }
    } else {
        for (edge_position, edge) in order_edges.iter().enumerate() {
            work = checked_usize_counter_add(work, 1).ok_or_else(resource_limit)?;
            if occurrences.contains_key(&edge.before_index)
                && occurrences.contains_key(&edge.after_index)
            {
                if !check_order_edge(edge, occurrences, path)? {
                    first_invalid_edge = Some(
                        first_invalid_edge.map_or(edge_position, |first| first.min(edge_position)),
                    );
                }
                checks = checked_usize_counter_add(checks, 1).ok_or_else(resource_limit)?;
            }
        }
    }

    if let Some(edge_position) = first_invalid_edge {
        let edge = order_edges
            .get(edge_position)
            .ok_or_else(|| invalid_schema(path))?;
        path.push(edge.after_tag);
        return Err(invalid_schema(path));
    }

    record_order_edge_work(metrics, work)?;
    record_order_edge_checks(metrics, checks)
}

fn find_order_edge(
    order_edges: &[CompiledOrderEdge],
    before_index: usize,
    after_index: usize,
) -> (Option<usize>, usize) {
    let expected = (before_index, after_index);
    let mut low = 0_usize;
    let mut high = order_edges.len();
    let mut comparisons = 0_usize;
    while low < high {
        let middle = low + (high - low) / 2;
        let Some(edge) = order_edges.get(middle) else {
            return (None, comparisons);
        };
        comparisons += 1;
        match (edge.before_index, edge.after_index).cmp(&expected) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Equal => return (Some(middle), comparisons),
            std::cmp::Ordering::Greater => high = middle,
        }
    }
    (None, comparisons)
}

fn validate_scalar(
    schema: &ExtensionSchema,
    value: &ValueView<'_>,
    path: &[kmipkit_ttlv::Tag],
    metrics: &mut ValidationMetrics,
) -> Result<(), ProtocolError> {
    if let Some(length) = scalar_payload_length(value) {
        let length = u64::try_from(length).map_err(|_| resource_limit())?;
        if schema
            .minimum_length
            .is_some_and(|minimum| length < minimum)
            || schema
                .maximum_length
                .is_some_and(|maximum| length > maximum)
        {
            return Err(invalid_schema(path));
        }
    }

    if let Some(value) = signed_scalar(value)
        && schema
            .signed_range
            .is_some_and(|(minimum, maximum)| value < minimum || value > maximum)
    {
        return Err(invalid_schema(path));
    }

    if let Some(value) = unsigned_scalar(value)
        && schema
            .unsigned_range
            .is_some_and(|(minimum, maximum)| value < minimum || value > maximum)
    {
        return Err(invalid_schema(path));
    }

    if let ValueView::Enumeration(value) = value {
        if schema.allowed_enumeration.is_empty() {
            record_enum_comparisons(metrics, 0)?;
        } else {
            let (accepted, comparisons) = enum_contains(&schema.allowed_enumeration, **value);
            record_enum_comparisons(metrics, comparisons)?;
            if !accepted {
                return Err(invalid_schema(path));
            }
        }
    }

    if let ValueView::Integer(value) = value {
        let bits = (**value).cast_unsigned();
        if schema
            .allowed_bit_mask
            .is_some_and(|allowed| bits & !allowed != 0)
            || schema
                .required_bit_mask
                .is_some_and(|required| bits & required != required)
        {
            return Err(invalid_schema(path));
        }
    }
    Ok(())
}

fn signed_scalar(value: &ValueView<'_>) -> Option<i64> {
    match value {
        ValueView::Integer(value) => Some(i64::from(**value)),
        ValueView::LongInteger(value)
        | ValueView::DateTime(value)
        | ValueView::DateTimeExtended(value) => Some(**value),
        _ => None,
    }
}

fn unsigned_scalar(value: &ValueView<'_>) -> Option<u64> {
    match value {
        ValueView::Enumeration(value) | ValueView::Interval(value) => Some(u64::from(**value)),
        _ => None,
    }
}

fn enum_contains(values: &[u32], expected: u32) -> (bool, usize) {
    let mut low = 0_usize;
    let mut high = values.len();
    let mut comparisons = 0_usize;
    while low < high {
        let middle = low + (high - low) / 2;
        let Some(value) = values.get(middle) else {
            return (false, comparisons);
        };
        comparisons += 1;
        match value.cmp(&expected) {
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Equal => return (true, comparisons),
            std::cmp::Ordering::Greater => high = middle,
        }
    }
    (false, comparisons)
}

fn value_item_type(value: &ValueView<'_>) -> Option<kmipkit_ttlv::ItemType> {
    match value {
        ValueView::Structure(_) => Some(kmipkit_ttlv::ItemType::Structure),
        ValueView::Integer(_) => Some(kmipkit_ttlv::ItemType::Integer),
        ValueView::LongInteger(_) => Some(kmipkit_ttlv::ItemType::LongInteger),
        ValueView::BigInteger(_) => Some(kmipkit_ttlv::ItemType::BigInteger),
        ValueView::Enumeration(_) => Some(kmipkit_ttlv::ItemType::Enumeration),
        ValueView::Boolean(_) => Some(kmipkit_ttlv::ItemType::Boolean),
        ValueView::TextString(_) => Some(kmipkit_ttlv::ItemType::TextString),
        ValueView::ByteString(_) => Some(kmipkit_ttlv::ItemType::ByteString),
        ValueView::DateTime(_) => Some(kmipkit_ttlv::ItemType::DateTime),
        ValueView::Interval(_) => Some(kmipkit_ttlv::ItemType::Interval),
        ValueView::DateTimeExtended(_) => Some(kmipkit_ttlv::ItemType::DateTimeExtended),
        _ => None,
    }
}

fn account_structure(
    structure: &StructureView<'_>,
    depth: usize,
    item_count: &mut usize,
    limits: &CodecLimits,
    path: &mut Vec<kmipkit_ttlv::Tag>,
) -> Result<usize, ProtocolError> {
    if depth > limits.max_structure_depth() {
        return Err(resource_limit_at(path));
    }
    let mut encoded_size = 8_usize;
    for item in structure.children() {
        path.push(item.tag());
        *item_count =
            checked_usize_counter_add(*item_count, 1).ok_or_else(|| resource_limit_at(path))?;
        if *item_count > limits.max_elements() {
            return Err(resource_limit_at(path));
        }
        let child_size = item.with_value(|value| match value {
            ValueView::Structure(nested) => {
                let child_depth =
                    checked_usize_counter_add(depth, 1).ok_or_else(|| resource_limit_at(path))?;
                account_structure(&nested, child_depth, item_count, limits, path)
            }
            scalar => {
                let payload_length =
                    scalar_payload_length(&scalar).ok_or_else(|| invalid_schema(path))?;
                padded_item_size(payload_length).map_err(|_| resource_limit_at(path))
            }
        })?;
        encoded_size = checked_usize_counter_add(encoded_size, child_size)
            .ok_or_else(|| resource_limit_at(path))?;
        if encoded_size > limits.max_message_bytes() {
            return Err(resource_limit_at(path));
        }
        path.pop();
    }
    Ok(encoded_size)
}

fn scalar_payload_length(value: &ValueView<'_>) -> Option<usize> {
    match value {
        ValueView::Integer(_)
        | ValueView::LongInteger(_)
        | ValueView::Enumeration(_)
        | ValueView::Boolean(_)
        | ValueView::DateTime(_)
        | ValueView::Interval(_)
        | ValueView::DateTimeExtended(_) => Some(8),
        ValueView::BigInteger(bytes) | ValueView::ByteString(bytes) => Some(bytes.len()),
        ValueView::TextString(text) => Some(text.len()),
        _ => None,
    }
}

fn padded_item_size(payload_length: usize) -> Result<usize, ProtocolError> {
    let padded_length = checked_usize_counter_add(payload_length, 7)
        .map(|length| (length / 8) * 8)
        .ok_or_else(resource_limit)?;
    checked_usize_counter_add(8_usize, padded_length).ok_or_else(resource_limit)
}

fn resource_limit_at(path: &[kmipkit_ttlv::Tag]) -> ProtocolError {
    let mut safe_path = Vec::new();
    if safe_path.try_reserve_exact(path.len()).is_err() {
        return resource_limit();
    }
    safe_path.extend_from_slice(path);
    ProtocolError::with_tag_path(
        ProtocolErrorKind::ResourceLimit,
        ProtocolCauseCategory::InvalidValue,
        safe_path,
    )
}

#[cfg(test)]
fn record_schema_comparisons(
    metrics: &mut ValidationMetrics,
    comparisons: usize,
) -> Result<(), ProtocolError> {
    metrics
        .schema_tag_comparisons_per_item
        .try_reserve(1)
        .map_err(|_| resource_limit())?;
    metrics.schema_tag_comparisons_per_item.push(comparisons);
    Ok(())
}

#[cfg(not(test))]
#[inline]
#[allow(clippy::unnecessary_wraps)] // Test metrics can fail while reserving their bounded traces.
fn record_schema_comparisons(_: &mut ValidationMetrics, _: usize) -> Result<(), ProtocolError> {
    Ok(())
}

#[cfg(test)]
fn record_enum_comparisons(
    metrics: &mut ValidationMetrics,
    comparisons: usize,
) -> Result<(), ProtocolError> {
    metrics
        .enum_comparisons_per_item
        .try_reserve(1)
        .map_err(|_| resource_limit())?;
    metrics.enum_comparisons_per_item.push(comparisons);
    Ok(())
}

#[cfg(not(test))]
#[inline]
#[allow(clippy::unnecessary_wraps)] // Test metrics can fail while reserving their bounded traces.
fn record_enum_comparisons(_: &mut ValidationMetrics, _: usize) -> Result<(), ProtocolError> {
    Ok(())
}

#[cfg(test)]
fn record_order_edge_checks(
    metrics: &mut ValidationMetrics,
    checks: usize,
) -> Result<(), ProtocolError> {
    metrics
        .order_edge_checks_per_structure
        .try_reserve(1)
        .map_err(|_| resource_limit())?;
    metrics.order_edge_checks_per_structure.push(checks);
    Ok(())
}

#[cfg(test)]
fn record_order_edge_work(
    metrics: &mut ValidationMetrics,
    work: usize,
) -> Result<(), ProtocolError> {
    metrics
        .order_edge_work_per_structure
        .try_reserve(1)
        .map_err(|_| resource_limit())?;
    metrics.order_edge_work_per_structure.push(work);
    Ok(())
}

#[cfg(test)]
fn record_occurrence_entries(
    metrics: &mut ValidationMetrics,
    entries: usize,
) -> Result<(), ProtocolError> {
    metrics
        .occurrence_entries_per_structure
        .try_reserve(1)
        .map_err(|_| resource_limit())?;
    metrics.occurrence_entries_per_structure.push(entries);
    Ok(())
}

#[cfg(not(test))]
#[inline]
#[allow(clippy::unnecessary_wraps)] // Test metrics can fail while reserving their bounded traces.
fn record_occurrence_entries(_: &mut ValidationMetrics, _: usize) -> Result<(), ProtocolError> {
    Ok(())
}

#[cfg(not(test))]
#[inline]
#[allow(clippy::unnecessary_wraps)] // Test metrics can fail while reserving their bounded traces.
fn record_order_edge_checks(_: &mut ValidationMetrics, _: usize) -> Result<(), ProtocolError> {
    Ok(())
}

#[cfg(not(test))]
#[inline]
#[allow(clippy::unnecessary_wraps)] // Test metrics can fail while reserving their bounded traces.
fn record_order_edge_work(_: &mut ValidationMetrics, _: usize) -> Result<(), ProtocolError> {
    Ok(())
}

fn invalid_schema(path: &[kmipkit_ttlv::Tag]) -> ProtocolError {
    ProtocolError::with_tag_path(
        ProtocolErrorKind::InvalidSchema,
        ProtocolCauseCategory::InvalidValue,
        path.to_vec(),
    )
}

fn resource_limit() -> ProtocolError {
    ProtocolError::categorized(
        ProtocolErrorKind::ResourceLimit,
        ProtocolCauseCategory::InvalidValue,
    )
}

#[cfg(test)]
#[path = "value_amplification_tests.rs"]
mod value_amplification_tests;

#[cfg(test)]
mod t018_work_bound_tests {
    use super::validate_with_metrics;
    use crate::extension;
    use kmipkit_ttlv::codec::CodecLimits;
    use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value};

    const TAG_BASE: u32 = 0x0054_0000;
    const DISCRIMINATOR_OFFSET: u32 = 1;
    const DISCRIMINATOR: &str = "work-bound-fixture-v1";

    fn tag(offset: u32) -> Tag {
        RawTag::new(TAG_BASE + offset)
            .expect("work-bound fixture tag fits the KMIP Tag width")
            .try_checked()
            .expect("work-bound fixture tag uses the KMIP extension allocation")
    }

    fn item(offset: u32, value: Value) -> Item {
        Item::new(tag(offset), value).expect("checked work-bound tag forms an Item")
    }

    fn structure(items: impl IntoIterator<Item = Item>) -> Structure {
        let mut result = Structure::new();
        for item in items {
            result
                .try_push(item)
                .expect("work-bound fixture stays within the TTLV model depth");
        }
        result
    }

    fn definition(
        payload_rules: Vec<extension::ExtensionChildRule>,
        order: Vec<extension::ExtensionOrderConstraint>,
    ) -> extension::ExtensionDefinition {
        let mut children = vec![
            extension::required(
                tag(DISCRIMINATOR_OFFSET),
                extension::scalar(ItemType::TextString).expect("Text String is supported"),
            )
            .expect("discriminator schema child is valid"),
        ];
        children.extend(payload_rules);
        let schema = extension::structure(children, order, false)
            .expect("work-bound fixture schema is structurally valid");
        let identity = extension::extension_identity("example.vendor", "work-bound", "1")
            .expect("work-bound identity is valid");
        let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
            .expect("work-bound fixture supports the workspace version");
        let path = extension::ttlv_path(tag(DISCRIMINATOR_OFFSET))
            .expect("discriminator path is non-empty");
        let discriminator =
            extension::discriminator(path, Value::text_string(DISCRIMINATOR.into()))
                .expect("discriminator scalar is valid");
        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("discriminator resolves through the fixture schema")
    }

    fn extension_value(payload: impl IntoIterator<Item = Item>) -> Structure {
        let mut items = vec![item(
            DISCRIMINATOR_OFFSET,
            Value::text_string(DISCRIMINATOR.into()),
        )];
        items.extend(payload);
        structure(items)
    }

    #[test]
    fn wide_schema_uses_at_most_thirteen_tag_comparisons_per_input_item() {
        const MAX_RULES: u32 = 4_096;
        const REPEATED_ITEMS: usize = 1_024;

        let rule_capacity =
            usize::try_from(MAX_RULES).expect("the bounded fixture rule count fits usize") - 1;
        let mut rules = Vec::with_capacity(rule_capacity);
        for offset in 2..MAX_RULES {
            rules.push(
                extension::optional(
                    tag(offset),
                    extension::scalar(ItemType::Integer).expect("Integer is supported"),
                )
                .expect("wide optional child rule is valid"),
            );
        }
        rules.push(
            extension::repeated(
                tag(MAX_RULES),
                extension::scalar(ItemType::Integer).expect("Integer is supported"),
            )
            .expect("last indexed child rule is valid"),
        );
        let definition = definition(rules, Vec::new());
        let payload = (0..REPEATED_ITEMS).map(|value| {
            item(
                MAX_RULES,
                Value::integer(i32::try_from(value).expect("fixture value fits i32")),
            )
        });
        let (_, metrics) = validate_with_metrics(
            &definition,
            extension_value(payload),
            &CodecLimits::defaults(),
        )
        .expect("the repeated last-tag field satisfies its schema");

        assert_eq!(
            metrics.schema_tag_comparisons_per_item.len(),
            REPEATED_ITEMS + 1
        );
        assert!(
            metrics
                .schema_tag_comparisons_per_item
                .iter()
                .all(|comparisons| *comparisons <= 13)
        );
    }

    #[test]
    fn repeated_worst_case_enum_lookups_use_at_most_thirteen_comparisons_each() {
        const ENUM_VALUES: u32 = 4_096;
        const REPEATED_ITEMS: usize = 4_096;
        const PAYLOAD_OFFSET: u32 = 2;

        let mut schema =
            extension::scalar(ItemType::Enumeration).expect("Enumeration is a supported scalar");
        for value in (0..ENUM_VALUES).rev() {
            schema = extension::with_allowed_enumeration(schema, value)
                .expect("the enumeration reaches its hard member count");
        }
        let rule = extension::repeated(tag(PAYLOAD_OFFSET), schema)
            .expect("repeated Enumeration rule is valid");
        let definition = definition(vec![rule], Vec::new());
        let payload =
            (0..REPEATED_ITEMS).map(|_| item(PAYLOAD_OFFSET, Value::enumeration(ENUM_VALUES - 1)));
        let (_, metrics) = validate_with_metrics(
            &definition,
            extension_value(payload),
            &CodecLimits::defaults(),
        )
        .expect("every repeated enum value is declared");

        assert_eq!(metrics.enum_comparisons_per_item.len(), REPEATED_ITEMS);
        assert!(
            metrics
                .enum_comparisons_per_item
                .iter()
                .all(|comparisons| *comparisons <= 13)
        );
    }

    #[test]
    fn repeated_ordered_fields_check_each_declared_edge_exactly_once() {
        const EDGE_COUNT: u32 = 32;
        const OCCURRENCES_PER_FIELD: usize = 32;
        const FIRST_OFFSET: u32 = 100;

        let rules = (0..=EDGE_COUNT)
            .map(|index| {
                extension::repeated(
                    tag(FIRST_OFFSET + index),
                    extension::scalar(ItemType::Integer).expect("Integer is supported"),
                )
                .expect("repeated ordered field rule is valid")
            })
            .collect();
        let edges = (0..EDGE_COUNT)
            .map(|index| {
                extension::extension_order_constraint(
                    tag(FIRST_OFFSET + index),
                    tag(FIRST_OFFSET + index + 1),
                )
                .expect("adjacent tags form a directed order edge")
            })
            .collect();
        let definition = definition(rules, edges);
        let payload = (0..=EDGE_COUNT).flat_map(|index| {
            (0..OCCURRENCES_PER_FIELD).map(move |occurrence| {
                item(
                    FIRST_OFFSET + index,
                    Value::integer(i32::try_from(occurrence).expect("fixture value fits i32")),
                )
            })
        });
        let (_, metrics) = validate_with_metrics(
            &definition,
            extension_value(payload),
            &CodecLimits::defaults(),
        )
        .expect("all repeated fields occur in the declared edge order");

        assert_eq!(
            metrics.order_edge_checks_per_structure,
            [usize::try_from(EDGE_COUNT).expect("fixture edge count fits usize")]
        );
    }
}
