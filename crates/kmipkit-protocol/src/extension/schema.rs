//! Immutable, data-only schemas for vendor TTLV values.

use std::collections::HashMap;
use std::fmt;

use kmipkit_ttlv::{ItemType, Tag};

use crate::{ProtocolError, ProtocolErrorKind};

use super::categorized_error;
use super::limits::ExtensionRegistryLimits;

const MAX_CHILD_RULES: usize = 4_096;
const MAX_CONSTRAINT_MEMBERS: usize = 4_096;
const MAX_SCHEMA_DEPTH: usize = 64;

/// A recursive description of one allowed TTLV value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionSchema {
    pub(crate) kind: SchemaKind,
    pub(crate) minimum_length: Option<u64>,
    pub(crate) maximum_length: Option<u64>,
    pub(crate) signed_range: Option<(i64, i64)>,
    pub(crate) unsigned_range: Option<(u64, u64)>,
    pub(crate) allowed_enumeration: Vec<u32>,
    pub(crate) allowed_bit_mask: Option<u32>,
    pub(crate) required_bit_mask: Option<u32>,
}

/// A child-order constraint compiled to stable child-rule indexes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CompiledOrderEdge {
    pub(crate) before_index: usize,
    pub(crate) after_index: usize,
    pub(crate) before_tag: Tag,
    pub(crate) after_tag: Tag,
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct CompiledOrderEdges {
    edges: Vec<CompiledOrderEdge>,
    edge_index: HashMap<(usize, usize), usize>,
}

impl fmt::Debug for CompiledOrderEdges {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.edges, formatter)
    }
}

impl CompiledOrderEdges {
    fn from_sorted(edges: Vec<CompiledOrderEdge>) -> Result<Self, ProtocolError> {
        let mut edge_index = HashMap::new();
        edge_index
            .try_reserve(edges.len())
            .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
        for (index, edge) in edges.iter().enumerate() {
            edge_index.insert((edge.before_index, edge.after_index), index);
        }

        Ok(Self { edges, edge_index })
    }

    pub(crate) const fn len(&self) -> usize {
        self.edges.len()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, CompiledOrderEdge> {
        self.edges.iter()
    }

    pub(crate) fn get(&self, index: usize) -> Option<&CompiledOrderEdge> {
        self.edges.get(index)
    }

    pub(crate) fn position(&self, before_index: usize, after_index: usize) -> Option<usize> {
        self.edge_index.get(&(before_index, after_index)).copied()
    }
}

struct CompiledStructure {
    child_tag_index: Vec<usize>,
    required_child_indices: Vec<usize>,
    order_edges: CompiledOrderEdges,
}

/// The child cardinality declared for one Structure rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Cardinality {
    Required,
    Optional,
    Repeated,
}

/// A schema rule for one child Tag.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionChildRule {
    pub(crate) tag: Tag,
    pub(crate) schema: ExtensionSchema,
    pub(crate) cardinality: Cardinality,
}

/// A declared ordering edge between two child Tags in a Structure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtensionOrderConstraint {
    pub(crate) before_tag: Tag,
    pub(crate) after_tag: Tag,
}

/// Creates a scalar schema for any represented non-Structure TTLV Item Type.
///
/// Structure schemas are created with [`structure`].
///
/// # Errors
///
/// Returns `InvalidSchema` when `item_type` is Structure or an unsupported
/// future Item Type.
pub fn scalar(item_type: ItemType) -> Result<ExtensionSchema, ProtocolError> {
    if matches!(item_type, ItemType::Structure) {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }

    Ok(ExtensionSchema {
        kind: SchemaKind::Scalar(item_type),
        minimum_length: None,
        maximum_length: None,
        signed_range: None,
        unsigned_range: None,
        allowed_enumeration: Vec::new(),
        allowed_bit_mask: None,
        required_bit_mask: None,
    })
}

/// Creates a Structure schema with ordered child rules and optional order edges.
///
/// Unknown children are rejected when `preserve_undeclared_children` is false
/// and retained without a typed rule when it is true.
///
/// # Errors
///
/// Returns `InvalidSchema` for duplicate child Tags or an order edge whose
/// endpoint has no child rule, and `ResourceLimit` when a local hard bound is
/// exceeded.
pub fn structure(
    children: Vec<ExtensionChildRule>,
    order_constraints: Vec<ExtensionOrderConstraint>,
    preserve_undeclared_children: bool,
) -> Result<ExtensionSchema, ProtocolError> {
    if children.len() > MAX_CHILD_RULES || order_constraints.len() > MAX_CONSTRAINT_MEMBERS {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }

    for child in &children {
        if child.schema.depth()? >= MAX_SCHEMA_DEPTH {
            return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
        }
    }

    let compiled = compile_structure(&children, order_constraints)?;

    Ok(ExtensionSchema {
        kind: SchemaKind::Structure {
            children,
            child_tag_index: compiled.child_tag_index,
            required_child_indices: compiled.required_child_indices,
            order_edges: compiled.order_edges,
            preserve_undeclared_children,
        },
        minimum_length: None,
        maximum_length: None,
        signed_range: None,
        unsigned_range: None,
        allowed_enumeration: Vec::new(),
        allowed_bit_mask: None,
        required_bit_mask: None,
    })
}

/// Creates a required child rule.
///
/// # Errors
///
/// Returns `ResourceLimit` when the child schema exceeds the maximum nesting
/// depth.
pub fn required(tag: Tag, schema: ExtensionSchema) -> Result<ExtensionChildRule, ProtocolError> {
    child_rule(tag, schema, Cardinality::Required)
}

/// Creates an optional child rule.
///
/// # Errors
///
/// Returns `ResourceLimit` when the child schema exceeds the maximum nesting
/// depth.
pub fn optional(tag: Tag, schema: ExtensionSchema) -> Result<ExtensionChildRule, ProtocolError> {
    child_rule(tag, schema, Cardinality::Optional)
}

/// Creates a repeated child rule.
///
/// # Errors
///
/// Returns `ResourceLimit` when the child schema exceeds the maximum nesting
/// depth.
pub fn repeated(tag: Tag, schema: ExtensionSchema) -> Result<ExtensionChildRule, ProtocolError> {
    child_rule(tag, schema, Cardinality::Repeated)
}

fn child_rule(
    tag: Tag,
    schema: ExtensionSchema,
    cardinality: Cardinality,
) -> Result<ExtensionChildRule, ProtocolError> {
    if schema.depth()? > MAX_SCHEMA_DEPTH {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }
    Ok(ExtensionChildRule {
        tag,
        schema,
        cardinality,
    })
}

/// Creates an order edge requiring every `before_tag` occurrence to precede
/// every `after_tag` occurrence when both fields occur.
///
/// # Errors
///
/// Returns `InvalidSchema` when both Tags are identical.
pub fn extension_order_constraint(
    before_tag: Tag,
    after_tag: Tag,
) -> Result<ExtensionOrderConstraint, ProtocolError> {
    if before_tag == after_tag {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    Ok(ExtensionOrderConstraint {
        before_tag,
        after_tag,
    })
}

/// Adds a minimum Text String or Byte String value length.
///
/// # Errors
///
/// Returns `InvalidSchema` for a non-text/binary schema or a minimum above the
/// existing maximum.
pub fn with_minimum_length(
    mut schema: ExtensionSchema,
    value: u64,
) -> Result<ExtensionSchema, ProtocolError> {
    if !schema.supports_length() || schema.maximum_length.is_some_and(|maximum| value > maximum) {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    schema.minimum_length = Some(value);
    Ok(schema)
}

/// Adds a maximum Text String or Byte String value length.
///
/// # Errors
///
/// Returns `InvalidSchema` for a non-text/binary schema or a maximum below the
/// existing minimum.
pub fn with_maximum_length(
    mut schema: ExtensionSchema,
    value: u64,
) -> Result<ExtensionSchema, ProtocolError> {
    if !schema.supports_length() || schema.minimum_length.is_some_and(|minimum| value < minimum) {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    schema.maximum_length = Some(value);
    Ok(schema)
}

/// Adds an inclusive signed numeric range to an Integer, Long Integer,
/// Date Time, or Date Time Extended schema.
///
/// # Errors
///
/// Returns `InvalidSchema` for an incompatible schema type or a reversed range.
pub fn with_signed_range(
    mut schema: ExtensionSchema,
    minimum: i64,
    maximum: i64,
) -> Result<ExtensionSchema, ProtocolError> {
    if !schema.supports_signed_range() || minimum > maximum {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    schema.signed_range = Some((minimum, maximum));
    Ok(schema)
}

/// Adds an inclusive unsigned numeric range to an Enumeration or Interval schema.
///
/// # Errors
///
/// Returns `InvalidSchema` for an incompatible schema type or a reversed range.
pub fn with_unsigned_range(
    mut schema: ExtensionSchema,
    minimum: u64,
    maximum: u64,
) -> Result<ExtensionSchema, ProtocolError> {
    if !schema.supports_unsigned_range() || minimum > maximum {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    schema.unsigned_range = Some((minimum, maximum));
    Ok(schema)
}

/// Adds one accepted value to an Enumeration schema.
///
/// # Errors
///
/// Returns `InvalidSchema` for a non-Enumeration schema and `ResourceLimit`
/// when the per-rule constraint limit is reached or storage cannot be reserved.
pub fn with_allowed_enumeration(
    mut schema: ExtensionSchema,
    value: u32,
) -> Result<ExtensionSchema, ProtocolError> {
    if !matches!(schema.kind, SchemaKind::Scalar(ItemType::Enumeration)) {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    insert_sorted_constraint(&mut schema.allowed_enumeration, value)?;
    Ok(schema)
}

/// Adds bits that may be set on an Integer-backed bit mask schema.
///
/// # Errors
///
/// Returns `InvalidSchema` for a non-Integer schema.
pub fn with_allowed_bit_mask(
    mut schema: ExtensionSchema,
    value: u32,
) -> Result<ExtensionSchema, ProtocolError> {
    if !matches!(schema.kind, SchemaKind::Scalar(ItemType::Integer)) {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    schema.allowed_bit_mask = Some(schema.allowed_bit_mask.unwrap_or(0) | value);
    Ok(schema)
}

/// Adds bits that must be set on an Integer-backed bit mask schema.
///
/// # Errors
///
/// Returns `InvalidSchema` for a non-Integer schema.
pub fn with_required_bit_mask(
    mut schema: ExtensionSchema,
    value: u32,
) -> Result<ExtensionSchema, ProtocolError> {
    if !matches!(schema.kind, SchemaKind::Scalar(ItemType::Integer)) {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    schema.required_bit_mask = Some(schema.required_bit_mask.unwrap_or(0) | value);
    Ok(schema)
}

fn insert_sorted_constraint(values: &mut Vec<u32>, value: u32) -> Result<(), ProtocolError> {
    if values.len() >= MAX_CONSTRAINT_MEMBERS {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }
    values
        .try_reserve(1)
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    let insertion_index = values.partition_point(|existing| *existing < value);
    values.insert(insertion_index, value);
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SchemaKind {
    Scalar(ItemType),
    Structure {
        children: Vec<ExtensionChildRule>,
        /// Child-rule indexes ordered by each rule's Tag.
        child_tag_index: Vec<usize>,
        /// Required child-rule indexes in their original declaration order.
        required_child_indices: Vec<usize>,
        /// Unique, acyclic order edges resolved to child-rule indexes.
        order_edges: CompiledOrderEdges,
        preserve_undeclared_children: bool,
    },
}

fn compile_structure(
    children: &[ExtensionChildRule],
    constraints: Vec<ExtensionOrderConstraint>,
) -> Result<CompiledStructure, ProtocolError> {
    let mut child_tag_index = Vec::new();
    child_tag_index
        .try_reserve_exact(children.len())
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    child_tag_index.extend(0..children.len());
    child_tag_index.sort_unstable_by_key(|index| children[*index].tag.raw());
    if child_tag_index
        .windows(2)
        .any(|pair| children[pair[0]].tag == children[pair[1]].tag)
    {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }

    let mut required_child_indices = Vec::new();
    let required_count = children
        .iter()
        .filter(|child| child.cardinality == Cardinality::Required)
        .count();
    required_child_indices
        .try_reserve_exact(required_count)
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    required_child_indices.extend(children.iter().enumerate().filter_map(|(index, child)| {
        (child.cardinality == Cardinality::Required).then_some(index)
    }));

    let mut order_edges = Vec::new();
    order_edges
        .try_reserve_exact(constraints.len())
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    for constraint in constraints {
        if constraint.before_tag == constraint.after_tag {
            return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
        }
        let before_index = resolve_child_index(children, &child_tag_index, constraint.before_tag)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
        let after_index = resolve_child_index(children, &child_tag_index, constraint.after_tag)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
        order_edges.push(CompiledOrderEdge {
            before_index,
            after_index,
            before_tag: constraint.before_tag,
            after_tag: constraint.after_tag,
        });
    }
    order_edges.sort_unstable_by_key(|edge| (edge.before_index, edge.after_index));
    if order_edges.windows(2).any(|pair| {
        pair[0].before_index == pair[1].before_index && pair[0].after_index == pair[1].after_index
    }) {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    validate_order_acyclic(children.len(), &order_edges)?;

    Ok(CompiledStructure {
        child_tag_index,
        required_child_indices,
        order_edges: CompiledOrderEdges::from_sorted(order_edges)?,
    })
}

fn resolve_child_index(
    children: &[ExtensionChildRule],
    child_tag_index: &[usize],
    tag: Tag,
) -> Option<usize> {
    child_tag_index
        .binary_search_by_key(&tag.raw(), |index| children[*index].tag.raw())
        .ok()
        .and_then(|position| child_tag_index.get(position).copied())
}

fn validate_order_acyclic(
    node_count: usize,
    edges: &[CompiledOrderEdge],
) -> Result<(), ProtocolError> {
    if edges.is_empty() {
        return Ok(());
    }

    let offsets_length = node_count
        .checked_add(1)
        .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    let mut offsets = Vec::new();
    offsets
        .try_reserve_exact(offsets_length)
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    offsets.resize(offsets_length, 0_usize);
    let mut incoming = Vec::new();
    incoming
        .try_reserve_exact(node_count)
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    incoming.resize(node_count, 0_usize);

    for edge in edges {
        let outgoing_count = offsets
            .get_mut(edge.before_index + 1)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
        *outgoing_count = checked_usize_counter_add(*outgoing_count, 1)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
        let incoming_count = incoming
            .get_mut(edge.after_index)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
        *incoming_count = checked_usize_counter_add(*incoming_count, 1)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    }
    for index in 1..offsets.len() {
        let previous = offsets[index - 1];
        offsets[index] = checked_usize_counter_add(previous, offsets[index])
            .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    }

    let mut ready = Vec::new();
    ready
        .try_reserve_exact(node_count)
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    ready.extend(
        incoming
            .iter()
            .enumerate()
            .filter_map(|(index, count)| (*count == 0).then_some(index)),
    );
    let mut visited = 0_usize;
    let mut head = 0_usize;
    while head < ready.len() {
        let node = ready[head];
        head += 1;
        visited = checked_usize_counter_add(visited, 1)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
        let start = *offsets
            .get(node)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
        let end = *offsets
            .get(node + 1)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
        for edge in edges
            .get(start..end)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?
        {
            let count = incoming
                .get_mut(edge.after_index)
                .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
            *count = count
                .checked_sub(1)
                .ok_or_else(|| categorized_error(ProtocolErrorKind::InvalidSchema))?;
            if *count == 0 {
                ready.push(edge.after_index);
            }
        }
    }
    if visited != node_count {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    Ok(())
}

/// Checked addition shared by bounded schema and validation counters.
pub(crate) const fn checked_usize_counter_add(left: usize, right: usize) -> Option<usize> {
    left.checked_add(right)
}

/// Checked addition shared by bounded registry counters.
#[cfg(test)]
pub(crate) const fn checked_u64_counter_add(left: u64, right: u64) -> Option<u64> {
    left.checked_add(right)
}

impl ExtensionSchema {
    /// Returns the TTLV Item Type required at this schema node.
    #[must_use]
    pub(crate) fn item_type(&self) -> ItemType {
        match self.kind {
            SchemaKind::Scalar(item_type) => item_type,
            SchemaKind::Structure { .. } => ItemType::Structure,
        }
    }

    /// Returns this Structure's child rules, or `None` for a scalar schema.
    #[must_use]
    pub(crate) fn children(&self) -> Option<&[ExtensionChildRule]> {
        match &self.kind {
            SchemaKind::Structure { children, .. } => Some(children),
            SchemaKind::Scalar(_) => None,
        }
    }

    /// Returns whether undeclared Structure children are retained without a
    /// typed rule.
    #[must_use]
    pub(crate) const fn preserve_undeclared_children(&self) -> bool {
        match &self.kind {
            SchemaKind::Structure {
                preserve_undeclared_children,
                ..
            } => *preserve_undeclared_children,
            SchemaKind::Scalar(_) => false,
        }
    }

    fn supports_length(&self) -> bool {
        matches!(
            self.kind,
            SchemaKind::Scalar(ItemType::TextString | ItemType::ByteString)
        )
    }

    fn supports_signed_range(&self) -> bool {
        matches!(
            self.kind,
            SchemaKind::Scalar(
                ItemType::Integer
                    | ItemType::LongInteger
                    | ItemType::DateTime
                    | ItemType::DateTimeExtended
            )
        )
    }

    fn supports_unsigned_range(&self) -> bool {
        matches!(
            self.kind,
            SchemaKind::Scalar(ItemType::Enumeration | ItemType::Interval)
        )
    }

    fn depth(&self) -> Result<usize, ProtocolError> {
        match &self.kind {
            SchemaKind::Scalar(_) => Ok(1),
            SchemaKind::Structure { children, .. } => {
                let deepest_child = children.iter().try_fold(0_usize, |depth, child| {
                    Ok::<usize, ProtocolError>(depth.max(child.schema.depth()?))
                })?;
                checked_usize_counter_add(deepest_child, 1)
                    .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))
            }
        }
    }

    /// Returns the number of schema nodes in this subtree.
    pub(crate) fn node_count(&self) -> Result<usize, ProtocolError> {
        match &self.kind {
            SchemaKind::Scalar(_) => Ok(1),
            SchemaKind::Structure { children, .. } => {
                children.iter().try_fold(1_usize, |count, child| {
                    checked_usize_counter_add(count, child.schema.node_count()?)
                        .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))
                })
            }
        }
    }

    /// Returns allowed-enumeration values and order edges across this subtree.
    pub(crate) fn constraint_member_count(&self) -> Result<usize, ProtocolError> {
        let order_members = match &self.kind {
            SchemaKind::Structure {
                children,
                order_edges,
                ..
            } => children
                .iter()
                .try_fold(order_edges.len(), |count, child| {
                    checked_usize_counter_add(count, child.schema.constraint_member_count()?)
                        .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))
                })?,
            SchemaKind::Scalar(_) => 0,
        };
        checked_usize_counter_add(self.allowed_enumeration.len(), order_members)
            .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))
    }

    /// Checks the configurable registry limits before registry-owned indexes
    /// or snapshots reserve or clone schema state.
    pub(crate) fn validate_registry_limits(
        &self,
        limits: &ExtensionRegistryLimits,
    ) -> Result<(), ProtocolError> {
        self.validate_registry_limits_at(limits, 1)
    }

    fn validate_registry_limits_at(
        &self,
        limits: &ExtensionRegistryLimits,
        depth: usize,
    ) -> Result<(), ProtocolError> {
        let configured_depth = u64::try_from(depth)
            .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
        let per_rule_members = u64::try_from(self.allowed_enumeration.len())
            .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
        if configured_depth > limits.max_depth()
            || per_rule_members > limits.max_constraint_members_per_rule()
        {
            return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
        }

        if let SchemaKind::Structure {
            children,
            order_edges,
            ..
        } = &self.kind
        {
            let child_count = u64::try_from(children.len())
                .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
            let order_member_count = u64::try_from(order_edges.len())
                .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
            if child_count > limits.max_child_rules_per_structure()
                || order_member_count > limits.max_constraint_members_per_rule()
            {
                return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
            }
            let child_depth = checked_usize_counter_add(depth, 1)
                .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
            for child in children {
                child
                    .schema
                    .validate_registry_limits_at(limits, child_depth)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/extension_schema_counter_overflow_tests.rs"]
mod t018_counter_overflow_tests;
