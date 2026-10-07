//! Extension identity, compatibility, discriminator, and definition models.

use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};

use kmipkit_ttlv::{Item, ItemType, RawTag, Tag, Value, ValueView};

use crate::{ProtocolError, ProtocolErrorKind};

use super::categorized_error;
use super::limits::ExtensionRegistryLimits;
use super::schema::ExtensionSchema;

const MAX_TEXT_BYTES_PER_FIELD: usize = 4_096;
const MAX_DISCRIMINATOR_BYTES: usize = 4_096;
const MAX_PATH_DEPTH: usize = 64;

/// Stable local identity for one vendor extension.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExtensionIdentity {
    pub(crate) vendor_identifier: String,
    pub(crate) name: String,
    pub(crate) version: String,
}

/// Compatibility ranges declared for one extension definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Compatibility {
    pub(crate) kmip_minimum: (u8, u8),
    pub(crate) kmip_maximum: (u8, u8),
    pub(crate) kmipkit_minimum: (u64, u64, u64),
    pub(crate) kmipkit_maximum: (u64, u64, u64),
}

/// An ordered path of child Tags within a vendor extension Structure.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TtlvPath {
    pub(crate) tags: Vec<Tag>,
}

/// An exact scalar match at one declared TTLV path.
pub struct Discriminator {
    pub(crate) path: TtlvPath,
    pub(crate) scalar_value: Item,
}

impl fmt::Debug for Discriminator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Discriminator")
            .field("path", &self.path.tags)
            .field("scalar_value", &self.scalar_value)
            .finish()
    }
}

/// A complete local data-only declaration for one vendor extension.
#[derive(Debug)]
pub struct ExtensionDefinition {
    pub(crate) identity: ExtensionIdentity,
    pub(crate) compatibility: Compatibility,
    pub(crate) discriminator: Discriminator,
    pub(crate) schema: ExtensionSchema,
    pub(crate) information: Option<super::information::ExtensionInformation>,
}

/// Compact immutable resource totals used while assembling a client registry.
///
/// The fields count identity and Extension Information UTF-8 bytes,
/// discriminator scalar bytes, recursive schema nodes, and allowed-enum plus
/// order-edge constraint members, respectively.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExtensionDefinitionAccounting {
    /// Identity and optional Extension Information text bytes.
    pub text_bytes: usize,
    /// Largest individual identity or Extension Information text field.
    pub maximum_text_field_bytes: usize,
    /// Encoded content bytes for the discriminator scalar.
    pub discriminator_bytes: usize,
    /// Number of nodes in the recursive schema, including its root.
    pub schema_nodes: usize,
    /// Allowed-enumeration values plus ordering edges across the schema.
    pub constraint_members: usize,
    /// Number of Tags in the exact discriminator path.
    pub discriminator_path_depth: usize,
}

/// Creates a stable extension identity.
///
/// `vendor_identifier` must contain one or more ASCII letters, digits,
/// underscores, or periods. All identity fields are subject to the default
/// 4,096-byte metadata-field limit.
///
/// # Errors
///
/// Returns `InvalidIdentity` for an empty field or invalid vendor identifier,
/// and `ResourceLimit` when a field exceeds its byte limit.
pub fn extension_identity(
    vendor_identifier: &str,
    name: &str,
    version: &str,
) -> Result<ExtensionIdentity, ProtocolError> {
    if vendor_identifier.is_empty()
        || name.is_empty()
        || version.is_empty()
        || !vendor_identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.')
    {
        return Err(categorized_error(ProtocolErrorKind::InvalidIdentity));
    }
    if [vendor_identifier, name, version]
        .iter()
        .any(|field| field.len() > MAX_TEXT_BYTES_PER_FIELD)
    {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }

    Ok(ExtensionIdentity {
        vendor_identifier: vendor_identifier.to_owned(),
        name: name.to_owned(),
        version: version.to_owned(),
    })
}

/// Creates compatibility ranges that include KMIP 2.1 and this `KMIPKit` build.
///
/// Semantic versions use the three numeric `major.minor.patch` components.
///
/// # Errors
///
/// Returns `CompatibilityMismatch` when either range is reversed, KMIP 2.1 is
/// excluded, a semantic version is malformed, or the current `KMIPKit` version
/// is outside the declared range.
pub fn compatibility(
    kmip_min_major: u8,
    kmip_min_minor: u8,
    kmip_max_major: u8,
    kmip_max_minor: u8,
    kmipkit_minimum: &str,
    kmipkit_maximum: &str,
) -> Result<Compatibility, ProtocolError> {
    let kmip_minimum = (kmip_min_major, kmip_min_minor);
    let kmip_maximum = (kmip_max_major, kmip_max_minor);
    let Some(minimum) = parse_semver(kmipkit_minimum) else {
        return Err(categorized_error(ProtocolErrorKind::CompatibilityMismatch));
    };
    let Some(maximum) = parse_semver(kmipkit_maximum) else {
        return Err(categorized_error(ProtocolErrorKind::CompatibilityMismatch));
    };
    let Some(current) = parse_semver(env!("CARGO_PKG_VERSION")) else {
        return Err(categorized_error(ProtocolErrorKind::CompatibilityMismatch));
    };

    if kmip_minimum > kmip_maximum
        || kmip_minimum > (2, 1)
        || kmip_maximum < (2, 1)
        || minimum > maximum
        || current < minimum
        || current > maximum
    {
        return Err(categorized_error(ProtocolErrorKind::CompatibilityMismatch));
    }

    Ok(Compatibility {
        kmip_minimum,
        kmip_maximum,
        kmipkit_minimum: minimum,
        kmipkit_maximum: maximum,
    })
}

fn parse_semver(value: &str) -> Option<(u64, u64, u64)> {
    let mut components = value.split('.');
    let major = parse_semver_component(components.next()?)?;
    let minor = parse_semver_component(components.next()?)?;
    let patch = parse_semver_component(components.next()?)?;
    if components.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn parse_semver_component(value: &str) -> Option<u64> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return None;
    }
    value.parse().ok()
}

/// Creates a discriminator path beginning at `first_tag`.
///
/// # Errors
///
/// Returns `ResourceLimit` if the one-tag path allocation cannot be reserved.
pub fn ttlv_path(first_tag: Tag) -> Result<TtlvPath, ProtocolError> {
    let mut tags = Vec::new();
    tags.try_reserve(1)
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    tags.push(first_tag);
    Ok(TtlvPath { tags })
}

/// Appends one child Tag to a discriminator path.
///
/// # Errors
///
/// Returns `ResourceLimit` when the 64-step hard path depth would be exceeded
/// or the Tag storage cannot be reserved.
pub fn with_child_tag(mut path: TtlvPath, tag: Tag) -> Result<TtlvPath, ProtocolError> {
    if path.tags.len() >= MAX_PATH_DEPTH {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }
    path.tags
        .try_reserve(1)
        .map_err(|_| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    path.tags.push(tag);
    Ok(path)
}

/// Creates an exact scalar discriminator at `path`.
///
/// # Errors
///
/// Returns `InvalidSchema` when the supplied value is a Structure and
/// `ResourceLimit` when a variable-length scalar exceeds the 4,096-byte
/// discriminator limit.
pub fn discriminator(path: TtlvPath, scalar_value: Value) -> Result<Discriminator, ProtocolError> {
    let value_tag = RawTag::new(0x0042_0001)
        .and_then(|raw| raw.try_checked())
        .map_err(|error| {
            ProtocolError::new(
                ProtocolErrorKind::InvalidSchema,
                crate::ProtocolCauseCategory::InvalidValue,
                error,
            )
        })?;
    let scalar_value = Item::new(value_tag, scalar_value).map_err(|error| {
        ProtocolError::new(
            ProtocolErrorKind::InvalidSchema,
            crate::ProtocolCauseCategory::InvalidValue,
            error,
        )
    })?;
    if scalar_value.item_type() == ItemType::Structure {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }
    if scalar_value.with_value(|value| scalar_byte_len(&value)) > MAX_DISCRIMINATOR_BYTES {
        return Err(categorized_error(ProtocolErrorKind::ResourceLimit));
    }
    Ok(Discriminator { path, scalar_value })
}

/// Creates a definition after checking that its path and scalar type fit the schema.
///
/// # Errors
///
/// Returns `InvalidSchema` when the root is not a Structure, the discriminator
/// path does not resolve through declared Structure children, or the terminal
/// schema Item Type differs from the discriminator scalar type.
pub fn extension_definition(
    identity: ExtensionIdentity,
    compatibility: Compatibility,
    discriminator: Discriminator,
    schema: ExtensionSchema,
) -> Result<ExtensionDefinition, ProtocolError> {
    let Some(terminal_type) = schema_type_at_path(&schema, &discriminator.path.tags) else {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    };
    if terminal_type != discriminator.scalar_value.item_type() {
        return Err(categorized_error(ProtocolErrorKind::InvalidSchema));
    }

    Ok(ExtensionDefinition {
        identity,
        compatibility,
        discriminator,
        schema,
        information: None,
    })
}

/// Creates an owned copy of a complete extension definition for a language binding.
///
/// Variable-length discriminator values are copied into the TTLV model's
/// zeroizing storage. The copy preserves the schema, identity, compatibility,
/// discriminator, and optional Extension Information without exposing any
/// payload formatting.
#[doc(hidden)]
pub fn clone_extension_definition(
    definition: &ExtensionDefinition,
) -> Result<ExtensionDefinition, ProtocolError> {
    let discriminator = clone_extension_discriminator(&definition.discriminator)?;
    let mut cloned = extension_definition(
        definition.identity.clone(),
        definition.compatibility,
        discriminator,
        definition.schema.clone(),
    )?;
    if let Some(information) = &definition.information {
        cloned.information = Some(information.clone());
    }
    Ok(cloned)
}

/// Creates an owned copy of a discriminator with zeroizing scalar storage.
#[doc(hidden)]
pub fn clone_extension_discriminator(
    source: &Discriminator,
) -> Result<Discriminator, ProtocolError> {
    let scalar = source.scalar_value.with_value(clone_value)?;
    discriminator(source.path.clone(), scalar)
}

fn clone_value(value: ValueView<'_>) -> Result<Value, ProtocolError> {
    match value {
        ValueView::Structure(structure) => Ok(Value::structure(clone_structure(&structure)?)),
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
        _ => Err(categorized_error(ProtocolErrorKind::UnsupportedValue)),
    }
}

fn clone_structure(
    structure: &kmipkit_ttlv::StructureView<'_>,
) -> Result<kmipkit_ttlv::Structure, ProtocolError> {
    let mut cloned = kmipkit_ttlv::Structure::new();
    for item in structure.children() {
        let value = item.with_value(clone_value)?;
        let cloned_item = Item::new(item.tag(), value).map_err(|error| {
            ProtocolError::new(
                ProtocolErrorKind::InvalidSchema,
                crate::ProtocolCauseCategory::InvalidValue,
                error,
            )
        })?;
        cloned.try_push(cloned_item).map_err(|error| {
            ProtocolError::new(
                ProtocolErrorKind::InvalidSchema,
                crate::ProtocolCauseCategory::InvalidValue,
                error,
            )
        })?;
    }
    Ok(cloned)
}

/// Returns a compact resource-accounting snapshot for client registry checks.
///
/// This hidden workspace bridge exposes counts only. It does not expose schema
/// traversal or mutable model state.
///
/// # Errors
///
/// Returns `ResourceLimit` if any aggregate count overflows `usize`.
#[doc(hidden)]
pub fn accounting(
    definition: &ExtensionDefinition,
) -> Result<ExtensionDefinitionAccounting, ProtocolError> {
    let identity_text_bytes = definition
        .identity
        .vendor_identifier
        .len()
        .checked_add(definition.identity.name.len())
        .and_then(|bytes| bytes.checked_add(definition.identity.version.len()))
        .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    let information_text_bytes = if let Some(information) = &definition.information {
        information
            .name
            .len()
            .checked_add(information.description.as_ref().map_or(0, String::len))
            .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?
    } else {
        0
    };
    let maximum_text_field_bytes = [
        definition.identity.vendor_identifier.len(),
        definition.identity.name.len(),
        definition.identity.version.len(),
        definition
            .information
            .as_ref()
            .map_or(0, |information| information.name.len()),
        definition
            .information
            .as_ref()
            .and_then(|information| information.description.as_ref())
            .map_or(0, String::len),
    ]
    .into_iter()
    .fold(0, usize::max);
    let text_bytes = identity_text_bytes
        .checked_add(information_text_bytes)
        .ok_or_else(|| categorized_error(ProtocolErrorKind::ResourceLimit))?;
    Ok(ExtensionDefinitionAccounting {
        text_bytes,
        maximum_text_field_bytes,
        discriminator_bytes: definition.discriminator.scalar_byte_len(),
        schema_nodes: definition.schema.node_count(),
        constraint_members: definition.schema.constraint_member_count(),
        discriminator_path_depth: definition.discriminator.path.tags.len(),
    })
}

/// Verifies per-schema configured bounds without allocating or cloning schema
/// state. Aggregate counters are returned separately by [`accounting`].
#[doc(hidden)]
pub fn validate_schema_limits(
    definition: &ExtensionDefinition,
    limits: &ExtensionRegistryLimits,
) -> Result<(), ProtocolError> {
    definition.schema.validate_registry_limits(limits)
}

/// Verifies schema limits against the remaining aggregate registry budgets.
///
/// This bounded preflight stops as soon as either aggregate is exceeded, so
/// callers can reject borrowed definitions before accounting or cloning them.
///
/// # Errors
///
/// Returns `ResourceLimit` when a per-schema bound or either remaining
/// aggregate budget is exceeded.
#[doc(hidden)]
pub fn validate_schema_limits_with_aggregate(
    definition: &ExtensionDefinition,
    limits: &ExtensionRegistryLimits,
    remaining_schema_nodes: u64,
    remaining_constraint_members: u64,
) -> Result<(), ProtocolError> {
    definition.schema.validate_registry_limits_with_aggregate(
        limits,
        remaining_schema_nodes,
        remaining_constraint_members,
    )
}

/// Returns a clone of the stable identity declared by this definition.
#[must_use]
pub fn identity(definition: &ExtensionDefinition) -> ExtensionIdentity {
    definition.identity.clone()
}

fn schema_type_at_path(schema: &ExtensionSchema, path: &[Tag]) -> Option<ItemType> {
    let (tag, remaining) = path.split_first()?;
    let rule = schema.children()?.iter().find(|rule| rule.tag == *tag)?;
    if remaining.is_empty() {
        return Some(rule.schema.item_type());
    }
    schema_type_at_path(&rule.schema, remaining)
}

fn scalar_byte_len(value: &ValueView<'_>) -> usize {
    match value {
        ValueView::TextString(text) => text.len(),
        ValueView::ByteString(bytes) | ValueView::BigInteger(bytes) => bytes.len(),
        _ => 0,
    }
}

impl ExtensionIdentity {
    /// Returns the exact KMIP Vendor Identification string.
    #[must_use]
    pub fn vendor_identifier(&self) -> &str {
        &self.vendor_identifier
    }

    /// Returns the local extension name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the local extension version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl TtlvPath {
    /// Returns the ordered child Tags that form this path.
    #[doc(hidden)]
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
}

impl Discriminator {
    /// Returns the exact path used by this discriminator.
    #[doc(hidden)]
    #[must_use]
    pub fn path(&self) -> &TtlvPath {
        &self.path
    }

    /// Returns an in-process fingerprint for the exact declared scalar.
    ///
    /// The fingerprint lets the client build a bounded candidate index
    /// without cloning the discriminator value. Callers must still confirm
    /// scalar equality after a fingerprint match.
    #[doc(hidden)]
    #[must_use]
    pub fn scalar_fingerprint(&self) -> Option<u64> {
        self.scalar_value
            .with_value(|value| scalar_value_fingerprint(&value))
    }

    /// Returns whether the other discriminator declares the same scalar.
    ///
    /// The value comparison borrows both declarations and never formats or
    /// copies either scalar.
    #[doc(hidden)]
    #[must_use]
    pub fn has_same_scalar(&self, other: &Self) -> bool {
        self.scalar_value.with_value(|expected| {
            other
                .scalar_value
                .with_value(|actual| scalar_values_equal(&expected, &actual))
        })
    }

    /// Returns the encoded text or byte length of this scalar, or zero for a
    /// fixed-width scalar.
    #[must_use]
    pub(crate) fn scalar_byte_len(&self) -> usize {
        self.scalar_value
            .with_value(|value| scalar_byte_len(&value))
    }

    /// Compares an observed scalar without formatting or copying its value.
    #[doc(hidden)]
    #[must_use]
    #[allow(clippy::needless_pass_by_value)] // Existing scoped-view callers lend this value by ownership.
    pub fn matches_value(&self, value: ValueView<'_>) -> bool {
        self.scalar_value
            .with_value(|expected| scalar_values_equal(&expected, &value))
    }

    /// Returns whether a borrowed observed scalar exactly matches this one.
    #[doc(hidden)]
    #[must_use]
    pub fn matches_value_ref(&self, value: &ValueView<'_>) -> bool {
        self.scalar_value
            .with_value(|expected| scalar_values_equal(&expected, value))
    }
}

/// Returns a scalar fingerprint for a borrowed TTLV value, or `None` for a
/// Structure or an Item Type this protocol model cannot compare.
#[doc(hidden)]
#[must_use]
pub fn scalar_value_fingerprint(value: &ValueView<'_>) -> Option<u64> {
    let mut hasher = DefaultHasher::new();
    match value {
        ValueView::Integer(value) => hash_scalar(1_u8, value, &mut hasher),
        ValueView::LongInteger(value) => hash_scalar(2_u8, value, &mut hasher),
        ValueView::BigInteger(value) => hash_scalar(3_u8, value, &mut hasher),
        ValueView::Enumeration(value) => hash_scalar(4_u8, value, &mut hasher),
        ValueView::Boolean(value) => hash_scalar(5_u8, value, &mut hasher),
        ValueView::TextString(value) => hash_scalar(6_u8, value, &mut hasher),
        ValueView::ByteString(value) => hash_scalar(7_u8, value, &mut hasher),
        ValueView::DateTime(value) => hash_scalar(8_u8, value, &mut hasher),
        ValueView::Interval(value) => hash_scalar(9_u8, value, &mut hasher),
        ValueView::DateTimeExtended(value) => hash_scalar(10_u8, value, &mut hasher),
        _ => return None,
    }
    Some(hasher.finish())
}

fn hash_scalar<T: Hash>(kind: u8, value: &T, hasher: &mut impl Hasher) {
    kind.hash(hasher);
    value.hash(hasher);
}

fn scalar_values_equal(expected: &ValueView<'_>, actual: &ValueView<'_>) -> bool {
    match (expected, actual) {
        (ValueView::Integer(expected), ValueView::Integer(actual)) => expected == actual,
        (ValueView::LongInteger(expected), ValueView::LongInteger(actual))
        | (ValueView::DateTimeExtended(expected), ValueView::DateTimeExtended(actual)) => {
            expected == actual
        }
        (ValueView::BigInteger(expected), ValueView::BigInteger(actual))
        | (ValueView::ByteString(expected), ValueView::ByteString(actual)) => expected == actual,
        (ValueView::Enumeration(expected), ValueView::Enumeration(actual))
        | (ValueView::Interval(expected), ValueView::Interval(actual)) => expected == actual,
        (ValueView::Boolean(expected), ValueView::Boolean(actual)) => expected == actual,
        (ValueView::TextString(expected), ValueView::TextString(actual)) => expected == actual,
        (ValueView::DateTime(expected), ValueView::DateTime(actual)) => expected == actual,
        _ => false,
    }
}

impl ExtensionDefinition {
    /// Borrows the stable identity for client-side indexing without copying text.
    #[doc(hidden)]
    #[must_use]
    pub const fn identity_ref(&self) -> &ExtensionIdentity {
        &self.identity
    }

    /// Returns the declared compatibility ranges.
    #[must_use]
    pub const fn compatibility(&self) -> Compatibility {
        self.compatibility
    }

    /// Returns the exact discriminator declaration.
    #[doc(hidden)]
    #[must_use]
    pub const fn discriminator(&self) -> &Discriminator {
        &self.discriminator
    }
}
