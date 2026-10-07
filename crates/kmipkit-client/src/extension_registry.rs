//! Immutable client-owned registry snapshots for data-only vendor extensions.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::{DefaultHasher, Hasher};

use crate::ClientError;
use kmipkit_protocol::extension::{
    self, ExtensionDefinition, ExtensionIdentity, ExtensionRegistryLimits,
};
use kmipkit_protocol::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::{Tag, ValueView};

#[cfg(test)]
std::thread_local! {
    static INDEX_COMPILATION_ATTEMPTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Default)]
struct DiscriminatorIndex {
    by_fingerprint: HashMap<u64, Vec<usize>>,
}

#[derive(Default)]
struct IdentityIndex {
    by_fingerprint: HashMap<u64, Vec<usize>>,
}

struct RegistryIndexes {
    identity: IdentityIndex,
    metadata_order: Vec<usize>,
    discriminator: DiscriminatorIndex,
}

impl RegistryIndexes {
    fn compile(definitions: &[ExtensionDefinition]) -> Result<Self, ProtocolError> {
        #[cfg(test)]
        INDEX_COMPILATION_ATTEMPTS.with(|attempts| attempts.set(attempts.get().saturating_add(1)));

        let definition_count = definitions.len();
        let mut metadata_order = Vec::new();
        metadata_order
            .try_reserve_exact(definition_count)
            .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
        metadata_order.extend(0..definition_count);
        metadata_order.sort_unstable_by(|left, right| {
            compare_identity(
                definitions[*left].identity_ref(),
                definitions[*right].identity_ref(),
            )
        });
        if metadata_order
            .windows(2)
            .any(|pair| definitions[pair[0]].identity_ref() == definitions[pair[1]].identity_ref())
        {
            return Err(registry_error(ProtocolErrorKind::DuplicateKey));
        }

        let mut identity = IdentityIndex::default();
        identity
            .by_fingerprint
            .try_reserve(definition_count)
            .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
        for (index, definition) in definitions.iter().enumerate() {
            index_identity(&mut identity, definitions, index, definition)?;
        }

        let mut discriminator = DiscriminatorIndex::default();
        discriminator
            .by_fingerprint
            .try_reserve(definition_count)
            .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
        for (index, definition) in definitions.iter().enumerate() {
            index_discriminator(&mut discriminator, definitions, index, definition)?;
        }

        Ok(Self {
            identity,
            metadata_order,
            discriminator,
        })
    }
}

/// Immutable extension definitions and exact discriminator indexes for one client.
///
/// This snapshot owns its definitions and indexes. It has no global state and
/// exposes no operation that can register or replace a definition.
pub struct ClientExtensionRegistry {
    definitions: Vec<ExtensionDefinition>,
    identity_index: IdentityIndex,
    metadata_order: Vec<usize>,
    discriminator_index: DiscriminatorIndex,
    limits: ExtensionRegistryLimits,
}

/// Immutable client configuration containing one owned extension registry.
///
/// The production transport constructor can consume this configuration
/// without rebuilding or sharing registry state between clients.
pub struct ClientConfiguration {
    extension_registry: ClientExtensionRegistry,
}

impl ClientConfiguration {
    /// Creates a client configuration from one immutable extension registry.
    #[must_use]
    pub fn new(extension_registry: ClientExtensionRegistry) -> Self {
        Self { extension_registry }
    }

    /// Returns the registry owned by this client configuration.
    #[must_use]
    pub const fn extension_registry(&self) -> &ClientExtensionRegistry {
        &self.extension_registry
    }
}

impl fmt::Debug for ClientConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientConfiguration")
            .field("extension_registry", &self.extension_registry)
            .finish()
    }
}

impl fmt::Debug for ClientExtensionRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientExtensionRegistry")
            .field("definition_count", &self.definitions.len())
            .finish_non_exhaustive()
    }
}

/// Constructs one immutable registry snapshot for a client configuration.
///
/// Definitions are stored in caller order for future request attachment;
/// metadata indexes expose a stable identity-sorted view. Discriminator
/// buckets use an in-process fingerprint, and candidate matching verifies the
/// full vendor, path, item-type, and scalar-value tuple.
///
/// # Errors
///
/// Returns a `ClientError::Protocol` with `NotSent` delivery evidence. Its
/// sanitized protocol kind is `DuplicateKey` for duplicate identities or
/// exact discriminator keys and `ResourceLimit` when a configured registry
/// limit or required allocation cannot be satisfied.
pub fn client_extension_registry(
    definitions: Vec<ExtensionDefinition>,
    limits: ExtensionRegistryLimits,
) -> Result<ClientExtensionRegistry, ClientError> {
    build_registry(definitions, limits)
        .map_err(|error| ClientError::protocol(error, RequestDeliveryState::NotSent))
}

fn build_registry(
    definitions: Vec<ExtensionDefinition>,
    limits: ExtensionRegistryLimits,
) -> Result<ClientExtensionRegistry, ProtocolError> {
    validate_totals(&definitions, &limits)?;
    let indexes = RegistryIndexes::compile(&definitions)?;

    Ok(ClientExtensionRegistry {
        definitions,
        identity_index: indexes.identity,
        metadata_order: indexes.metadata_order,
        discriminator_index: indexes.discriminator,
        limits,
    })
}

/// Returns the number of definitions in this immutable snapshot.
#[must_use]
pub fn definition_count(registry: &ClientExtensionRegistry) -> u64 {
    u64::try_from(registry.definitions.len()).unwrap_or(u64::MAX)
}

/// Returns a definition from the identity-sorted metadata view.
#[must_use]
pub fn definition_at(
    registry: &ClientExtensionRegistry,
    index: usize,
) -> Option<&ExtensionDefinition> {
    registry
        .metadata_order
        .get(index)
        .and_then(|definition_index| registry.definitions.get(*definition_index))
}

/// Finds a definition by its exact local identity.
#[must_use]
#[allow(clippy::needless_pass_by_value)] // T010's owned identity API avoids borrowing temporary lookup keys.
pub fn definition_for_identity(
    registry: &ClientExtensionRegistry,
    identity: ExtensionIdentity,
) -> Option<&ExtensionDefinition> {
    let fingerprint = identity_fingerprint(&identity);
    let candidates = registry.identity_index.by_fingerprint.get(&fingerprint)?;
    candidates.iter().find_map(|index| {
        let definition = registry.definitions.get(*index)?;
        (definition.identity_ref() == &identity).then_some(definition)
    })
}

/// Returns the candidate definition indexes for one discriminator fingerprint.
///
/// A fingerprint only selects a candidate bucket. T035 must call
/// [`candidate_matches_discriminator`] for every returned index before
/// accepting a match; a fingerprint collision alone never establishes an
/// exact vendor, path, item-type, and scalar-value match.
#[allow(dead_code)] // The staged inbound recognition consumer is T035.
pub(crate) fn discriminator_candidates<'a>(
    registry: &'a ClientExtensionRegistry,
    vendor_identifier: &str,
    path: &[Tag],
    fingerprint: u64,
) -> Option<&'a [usize]> {
    let key_fingerprint = discriminator_index_fingerprint(vendor_identifier, path, fingerprint);
    registry
        .discriminator_index
        .by_fingerprint
        .get(&key_fingerprint)
        .map(Vec::as_slice)
}

/// Verifies a fingerprint-bucket candidate against the complete discriminator
/// key. An index outside this registry returns `false`.
///
/// T035 must use this predicate for each candidate from
/// [`discriminator_candidates`] before treating an input path as recognized.
#[allow(dead_code)] // The staged inbound recognition consumer is T035.
pub(crate) fn candidate_matches_discriminator(
    registry: &ClientExtensionRegistry,
    candidate_index: usize,
    vendor_identifier: &str,
    path: &[Tag],
    scalar_value: &ValueView<'_>,
) -> bool {
    let Some(definition) = registry.definitions.get(candidate_index) else {
        return false;
    };

    definition.identity_ref().vendor_identifier() == vendor_identifier
        && definition.discriminator().path().tags() == path
        && definition.discriminator().matches_value_ref(scalar_value)
}

/// Returns the immutable limits used to construct this registry.
#[must_use]
pub const fn limits(registry: &ClientExtensionRegistry) -> ExtensionRegistryLimits {
    registry.limits
}

fn validate_totals(
    definitions: &[ExtensionDefinition],
    limits: &ExtensionRegistryLimits,
) -> Result<(), ProtocolError> {
    let count = u64::try_from(definitions.len())
        .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
    if count > limits.max_definitions() {
        return Err(registry_error(ProtocolErrorKind::ResourceLimit));
    }

    let mut text_bytes = 0_u64;
    let mut schema_nodes = 0_u64;
    let mut discriminator_bytes = 0_u64;
    let mut constraint_members = 0_u64;
    for definition in definitions {
        extension::validate_schema_limits(definition, limits)?;
        let accounting = extension::accounting(definition)?;
        if exceeds_limit(
            accounting.maximum_text_field_bytes,
            limits.max_text_bytes_per_field(),
        ) || exceeds_limit(accounting.discriminator_path_depth, limits.max_depth())
        {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }
        let fields = [
            (
                &mut text_bytes,
                accounting.text_bytes,
                limits.max_registry_text_bytes(),
            ),
            (
                &mut schema_nodes,
                accounting.schema_nodes,
                limits.max_schema_nodes(),
            ),
            (
                &mut discriminator_bytes,
                accounting.discriminator_bytes,
                limits.max_total_discriminator_scalar_bytes(),
            ),
            (
                &mut constraint_members,
                accounting.constraint_members,
                limits.max_total_constraint_members(),
            ),
        ];
        for (total, increment, maximum) in fields {
            *total = checked_registry_counter_add(
                *total,
                u64::try_from(increment)
                    .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?,
            )
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            if *total > maximum {
                return Err(registry_error(ProtocolErrorKind::ResourceLimit));
            }
        }
        if exceeds_limit(
            accounting.discriminator_bytes,
            limits.max_discriminator_scalar_bytes(),
        ) {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }
    }
    Ok(())
}

fn exceeds_limit(value: usize, maximum: u64) -> bool {
    u64::try_from(value).map_or(true, |value| value > maximum)
}

fn checked_registry_counter_add(left: u64, right: u64) -> Option<u64> {
    left.checked_add(right)
}

fn index_discriminator(
    index: &mut DiscriminatorIndex,
    definitions: &[ExtensionDefinition],
    definition_index: usize,
    definition: &ExtensionDefinition,
) -> Result<(), ProtocolError> {
    let vendor_identifier = definition.identity_ref().vendor_identifier();
    let path = definition.discriminator().path().tags();
    let scalar_fingerprint = definition
        .discriminator()
        .scalar_fingerprint()
        .ok_or_else(|| registry_error(ProtocolErrorKind::InvalidSchema))?;
    let fingerprint = discriminator_index_fingerprint(vendor_identifier, path, scalar_fingerprint);
    index
        .by_fingerprint
        .try_reserve(1)
        .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
    let candidates = index.by_fingerprint.entry(fingerprint).or_default();
    for candidate_index in candidates.iter().copied() {
        let candidate = definitions
            .get(candidate_index)
            .ok_or_else(|| registry_error(ProtocolErrorKind::InvalidSchema))?;
        if candidate.identity_ref().vendor_identifier() == vendor_identifier
            && candidate.discriminator().path().tags() == path
            && candidate
                .discriminator()
                .has_same_scalar(definition.discriminator())
        {
            return Err(registry_error(ProtocolErrorKind::DuplicateKey));
        }
    }
    candidates
        .try_reserve(1)
        .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
    candidates.push(definition_index);
    Ok(())
}

fn index_identity(
    index: &mut IdentityIndex,
    definitions: &[ExtensionDefinition],
    definition_index: usize,
    definition: &ExtensionDefinition,
) -> Result<(), ProtocolError> {
    let identity = definition.identity_ref();
    let fingerprint = identity_fingerprint(identity);
    index
        .by_fingerprint
        .try_reserve(1)
        .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
    let candidates = index.by_fingerprint.entry(fingerprint).or_default();
    for candidate_index in candidates.iter().copied() {
        let candidate = definitions
            .get(candidate_index)
            .ok_or_else(|| registry_error(ProtocolErrorKind::InvalidSchema))?;
        if candidate.identity_ref() == identity {
            return Err(registry_error(ProtocolErrorKind::DuplicateKey));
        }
    }
    candidates
        .try_reserve(1)
        .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
    candidates.push(definition_index);
    Ok(())
}

fn identity_fingerprint(identity: &ExtensionIdentity) -> u64 {
    let mut hasher = DefaultHasher::new();
    for field in [
        identity.vendor_identifier(),
        identity.name(),
        identity.version(),
    ] {
        hasher.write_usize(field.len());
        hasher.write(field.as_bytes());
    }
    hasher.finish()
}

fn discriminator_index_fingerprint(
    vendor_identifier: &str,
    path: &[Tag],
    scalar_fingerprint: u64,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write_usize(vendor_identifier.len());
    hasher.write(vendor_identifier.as_bytes());
    hasher.write_usize(path.len());
    for tag in path {
        hasher.write_u32(tag.raw());
    }
    hasher.write_u64(scalar_fingerprint);
    hasher.finish()
}

fn compare_identity(left: &ExtensionIdentity, right: &ExtensionIdentity) -> std::cmp::Ordering {
    left.vendor_identifier()
        .cmp(right.vendor_identifier())
        .then_with(|| left.name().cmp(right.name()))
        .then_with(|| left.version().cmp(right.version()))
}

fn registry_error(kind: ProtocolErrorKind) -> ProtocolError {
    ProtocolError::new(
        kind,
        ProtocolCauseCategory::InvalidValue,
        RegistryConstructionFailure,
    )
}

#[derive(Debug)]
struct RegistryConstructionFailure;

impl fmt::Display for RegistryConstructionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("extension registry construction failed")
    }
}

impl Error for RegistryConstructionFailure {}

#[cfg(test)]
mod tests {
    use super::{
        INDEX_COMPILATION_ATTEMPTS, candidate_matches_discriminator, client_extension_registry,
        discriminator_candidates,
    };
    use kmipkit_protocol::extension;
    use kmipkit_ttlv::{Item, ItemType, RawTag, Tag, Value};

    fn vendor_tag() -> Tag {
        RawTag::new(0x42_0001)
            .expect("the test tag fits in the KMIP Tag width")
            .try_checked()
            .expect("the test tag uses the vendor allocation")
    }

    fn definition(name: &str, discriminator_value: &str) -> extension::ExtensionDefinition {
        let identity = extension::extension_identity("example.vendor", name, "1")
            .expect("the test extension identity is valid");
        let compatibility = extension::compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
            .expect("the test compatibility range includes this client");
        let path = extension::ttlv_path(vendor_tag()).expect("the test path is valid");
        let discriminator =
            extension::discriminator(path, Value::text_string(discriminator_value.to_owned()))
                .expect("the test discriminator is a valid scalar");
        let child_schema =
            extension::scalar(ItemType::TextString).expect("Text String is a scalar schema");
        let child = extension::required(vendor_tag(), child_schema)
            .expect("the test discriminator child is valid");
        let schema = extension::structure(vec![child], Vec::new(), false)
            .expect("the test extension schema is valid");

        extension::extension_definition(identity, compatibility, discriminator, schema)
            .expect("the test definition is valid")
    }

    fn limits_with_overrides(overrides: [u64; 12]) -> extension::ExtensionRegistryLimits {
        extension::with_values(
            overrides[0],
            overrides[1],
            overrides[2],
            overrides[3],
            overrides[4],
            overrides[5],
            overrides[6],
            overrides[7],
            overrides[8],
            overrides[9],
            overrides[10],
            overrides[11],
        )
        .expect("the test limit overrides remain under every hard maximum")
    }

    #[test]
    fn registry_limits_reject_before_compiling_or_reserving_indexes() {
        let defaults = extension::defaults();
        let default_values = [
            defaults.max_definitions(),
            defaults.max_schema_nodes(),
            defaults.max_child_rules_per_structure(),
            defaults.max_text_bytes_per_field(),
            defaults.max_registry_text_bytes(),
            defaults.max_discriminator_scalar_bytes(),
            defaults.max_total_discriminator_scalar_bytes(),
            defaults.max_constraint_members_per_rule(),
            defaults.max_total_constraint_members(),
            defaults.max_payload_index_records(),
            defaults.max_lookup_comparisons(),
            defaults.max_depth(),
        ];
        let invalid_cases = [
            (
                vec![definition("count", "value")],
                limits_with_overrides([
                    0,
                    default_values[1],
                    default_values[2],
                    default_values[3],
                    default_values[4],
                    default_values[5],
                    default_values[6],
                    default_values[7],
                    default_values[8],
                    default_values[9],
                    default_values[10],
                    default_values[11],
                ]),
            ),
            (
                vec![definition("name-too-long", "value")],
                limits_with_overrides([
                    default_values[0],
                    default_values[1],
                    default_values[2],
                    4,
                    default_values[4],
                    default_values[5],
                    default_values[6],
                    default_values[7],
                    default_values[8],
                    default_values[9],
                    default_values[10],
                    default_values[11],
                ]),
            ),
            (
                vec![definition("aggregate-text", "value")],
                limits_with_overrides([
                    default_values[0],
                    default_values[1],
                    default_values[2],
                    default_values[3],
                    1,
                    default_values[5],
                    default_values[6],
                    default_values[7],
                    default_values[8],
                    default_values[9],
                    default_values[10],
                    default_values[11],
                ]),
            ),
            (
                vec![definition("scalar", "too-long")],
                limits_with_overrides([
                    default_values[0],
                    default_values[1],
                    default_values[2],
                    default_values[3],
                    default_values[4],
                    3,
                    default_values[6],
                    default_values[7],
                    default_values[8],
                    default_values[9],
                    default_values[10],
                    default_values[11],
                ]),
            ),
            (
                vec![definition("aggregate-scalar", "value")],
                limits_with_overrides([
                    default_values[0],
                    default_values[1],
                    default_values[2],
                    default_values[3],
                    default_values[4],
                    default_values[5],
                    3,
                    default_values[7],
                    default_values[8],
                    default_values[9],
                    default_values[10],
                    default_values[11],
                ]),
            ),
            (
                vec![definition("schema-node", "value")],
                limits_with_overrides([
                    default_values[0],
                    1,
                    default_values[2],
                    default_values[3],
                    default_values[4],
                    default_values[5],
                    default_values[6],
                    default_values[7],
                    default_values[8],
                    default_values[9],
                    default_values[10],
                    default_values[11],
                ]),
            ),
        ];

        let initial = INDEX_COMPILATION_ATTEMPTS.with(std::cell::Cell::get);
        for (definitions, limits) in invalid_cases {
            assert!(
                client_extension_registry(definitions, limits).is_err(),
                "each over-limit registry is rejected"
            );
            assert_eq!(
                INDEX_COMPILATION_ATTEMPTS.with(std::cell::Cell::get),
                initial,
                "rejected registry limits must be checked before index reservations"
            );
        }

        client_extension_registry(vec![definition("valid", "value")], defaults)
            .expect("a valid registry reaches index compilation");
        assert_eq!(
            INDEX_COMPILATION_ATTEMPTS.with(std::cell::Cell::get),
            initial + 1
        );
    }

    #[test]
    fn discriminator_index_selects_by_exact_scalar_after_fingerprinting() {
        let registry = client_extension_registry(
            vec![
                definition("alpha", "alpha-v1"),
                definition("beta", "beta-v1"),
            ],
            extension::defaults(),
        )
        .expect("distinct scalars may share a path");
        let observed = Item::new(vendor_tag(), Value::text_string("beta-v1".to_owned()))
            .expect("the observed discriminator is valid TTLV");

        let matches = observed.with_value(|value| {
            let fingerprint = extension::scalar_value_fingerprint(&value)
                .expect("the observed value is a scalar");
            let candidates =
                discriminator_candidates(&registry, "example.vendor", &[vendor_tag()], fingerprint)
                    .expect("the exact path has a candidate bucket");
            candidates
                .iter()
                .filter_map(|index| registry.definitions.get(*index))
                .filter(|candidate| candidate.discriminator().matches_value_ref(&value))
                .map(|candidate| extension::identity(candidate).name().to_owned())
                .collect::<Vec<_>>()
        });

        assert_eq!(matches, ["beta"]);
    }

    #[test]
    fn candidate_match_checks_the_full_vendor_path_and_scalar_tuple() {
        let registry = client_extension_registry(
            vec![
                definition("alpha", "alpha-v1"),
                definition("beta", "beta-v1"),
            ],
            extension::defaults(),
        )
        .expect("distinct scalars may share a path");
        let observed = Item::new(vendor_tag(), Value::text_string("beta-v1".to_owned()))
            .expect("the observed discriminator is valid TTLV");
        let wrong_type =
            Item::new(vendor_tag(), Value::integer(7)).expect("the observed integer is valid TTLV");
        let wrong_value = Item::new(vendor_tag(), Value::text_string("alpha-v1".to_owned()))
            .expect("the alternate observed discriminator is valid TTLV");
        let wrong_path = RawTag::new(0x42_0002)
            .expect("the alternate test tag fits in the KMIP Tag width")
            .try_checked()
            .expect("the alternate test tag uses the vendor allocation");

        let outcomes = observed.with_value(|value| {
            let exact_match = candidate_matches_discriminator(
                &registry,
                1,
                "example.vendor",
                &[vendor_tag()],
                &value,
            );
            let wrong_vendor = candidate_matches_discriminator(
                &registry,
                1,
                "different.vendor",
                &[vendor_tag()],
                &value,
            );
            let wrong_path = candidate_matches_discriminator(
                &registry,
                1,
                "example.vendor",
                &[wrong_path],
                &value,
            );
            let wrong_scalar_value = wrong_value.with_value(|wrong_value| {
                candidate_matches_discriminator(
                    &registry,
                    1,
                    "example.vendor",
                    &[vendor_tag()],
                    &wrong_value,
                )
            });
            let out_of_range = candidate_matches_discriminator(
                &registry,
                usize::MAX,
                "example.vendor",
                &[vendor_tag()],
                &value,
            );
            let wrong_item_type = wrong_type.with_value(|wrong_value| {
                candidate_matches_discriminator(
                    &registry,
                    1,
                    "example.vendor",
                    &[vendor_tag()],
                    &wrong_value,
                )
            });

            (
                exact_match,
                wrong_vendor,
                wrong_path,
                wrong_scalar_value,
                wrong_item_type,
                out_of_range,
            )
        });

        assert_eq!(outcomes, (true, false, false, false, false, false));
    }
}
