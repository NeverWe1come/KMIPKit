//! Schema-validated extension values retaining their original generic tree.

use std::collections::HashMap;
use std::fmt;

use kmipkit_ttlv::{Item, Structure, StructureView, ValueView, codec::CodecLimits};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};

use super::definition::{ExtensionDefinition, ExtensionIdentity};
use super::schema::{
    Cardinality, CompiledOrderEdge, CompiledOrderEdges, ExtensionSchema, SchemaKind,
    checked_usize_counter_add,
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
    let ((), _) = validate_inner(
        definition,
        &value,
        limits,
        ValidationMode::RequireDiscriminator,
    )?;
    Ok(ValidatedExtensionValue {
        identity: definition.identity.clone(),
        generic_value: value,
    })
}

/// Schema-validation outcome that deliberately makes no recognition claim.
///
/// A `SchemaValid` result proves only that the payload matches the definition's
/// data-only schema. It does not prove that the discriminator matched. This
/// hidden cross-crate outcome is consumed by client inspection only after its
/// bounded index establishes one exact discriminator match.
#[doc(hidden)]
pub enum SchemaValidationOutcome {
    /// The complete definition schema accepted the original subtree.
    SchemaValid(ValidatedExtensionValue),
    /// The schema rejected the original subtree; it remains generic and owned.
    SchemaInvalid(Structure),
}

/// Validates only the data-only schema and retains schema-invalid input generically.
///
/// This cross-crate helper makes no assertion that the discriminator matches.
/// Client recognition calls it only after its bounded index has proved one
/// exact discriminator match. Callers that need a complete standalone check
/// must use [`validate`]. The original subtree is moved through either outcome
/// without cloning secret-bearing payloads or exposing a partial typed value.
///
/// # Errors
///
/// Returns a sanitized resource-limit error when the configured TTLV bounds
/// are exceeded. A schema mismatch is represented as
/// `SchemaValidationOutcome::SchemaInvalid`.
#[doc(hidden)]
pub fn validate_schema_only(
    definition: &ExtensionDefinition,
    value: Structure,
    limits: &CodecLimits,
) -> Result<SchemaValidationOutcome, ProtocolError> {
    match validate_inner(definition, &value, limits, ValidationMode::SchemaOnly) {
        Ok(((), _)) => Ok(SchemaValidationOutcome::SchemaValid(
            ValidatedExtensionValue {
                identity: definition.identity.clone(),
                generic_value: value,
            },
        )),
        Err(error) if error.kind() == ProtocolErrorKind::InvalidSchema => {
            Ok(SchemaValidationOutcome::SchemaInvalid(value))
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
fn validate_with_metrics(
    definition: &ExtensionDefinition,
    value: Structure,
    limits: &CodecLimits,
) -> Result<(ValidatedExtensionValue, ValidationMetrics), ProtocolError> {
    let ((), metrics) = validate_inner(
        definition,
        &value,
        limits,
        ValidationMode::RequireDiscriminator,
    )?;
    Ok((
        ValidatedExtensionValue {
            identity: definition.identity.clone(),
            generic_value: value,
        },
        metrics,
    ))
}

#[derive(Clone, Copy)]
enum ValidationMode {
    RequireDiscriminator,
    SchemaOnly,
}

fn validate_inner(
    definition: &ExtensionDefinition,
    value: &Structure,
    limits: &CodecLimits,
    mode: ValidationMode,
) -> Result<((), ValidationMetrics), ProtocolError> {
    let mut metrics = ValidationMetrics::default();
    let mut path = Vec::new();
    path.try_reserve(limits.max_structure_depth().min(64))
        .map_err(|_| resource_limit())?;
    validate_ttlv_limits(value, limits, &mut path)?;

    let root_view = value.view();
    if matches!(mode, ValidationMode::RequireDiscriminator) {
        validate_discriminator(&root_view, definition.discriminator(), &mut path)?;
        path.clear();
    }
    validate_structure(&definition.schema, &root_view, &mut path, &mut metrics)?;

    Ok(((), metrics))
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
    order_edges: &CompiledOrderEdges,
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
    // when that requires fewer bounded operations.
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
                    work = checked_usize_counter_add(work, 1).ok_or_else(resource_limit)?;
                    let edge_position = order_edges.position(before, after);
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
#[path = "../../tests/unit/extension_value_amplification_tests.rs"]
mod value_amplification_tests;

#[cfg(test)]
#[path = "../../tests/unit/extension_value_work_bound_tests.rs"]
mod t018_work_bound_tests;
