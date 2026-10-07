//! Immutable client-owned registry snapshots for data-only vendor extensions.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;

use crate::ClientError;
use kmipkit_protocol::extension::{
    self, ExtensionDefinition, ExtensionIdentity, ExtensionRegistryLimits, ValidatedExtensionValue,
};
use kmipkit_protocol::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};
use kmipkit_transport::RequestDeliveryState;
use kmipkit_ttlv::{Structure, StructureView, Tag, ValueView, codec::CodecLimits};

const MAX_PAYLOAD_ITEMS: usize = 100_000;
const MAX_TAG_COMPARISONS_PER_STEP: u64 = 35;

#[derive(Clone, Copy, Default)]
struct PayloadEntry {
    tag: u32,
    child_index: usize,
    nested_structure: Option<usize>,
}

struct IndexedStructure {
    entry_start: usize,
    entry_count: usize,
}

struct PayloadTagIndex {
    structures: Vec<IndexedStructure>,
    entries: Vec<PayloadEntry>,
    scratch: Vec<PayloadEntry>,
}

struct LookupBudget<'a> {
    maximum_comparisons: u64,
    comparisons: &'a mut u64,
}

#[derive(Default)]
struct PayloadAccounting {
    item_count: usize,
    structure_count: usize,
    child_count: usize,
    maximum_width: usize,
}

enum RecognitionValue {
    Validated(ValidatedExtensionValue),
    Unrecognized(Structure),
}

/// Result of inspecting a generic vendor extension subtree.
///
/// Recognized results own a completely schema-validated value. Unrecognized
/// results still own and expose the unchanged generic TTLV subtree. Formatting
/// always redacts payload content.
pub struct ExtensionRecognition {
    value: RecognitionValue,
}

impl fmt::Debug for ExtensionRecognition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ExtensionRecognition([REDACTED])")
    }
}

impl ExtensionRecognition {
    /// Returns whether one exact discriminator match passed complete schema validation.
    #[must_use]
    pub const fn is_recognized(&self) -> bool {
        matches!(&self.value, RecognitionValue::Validated(_))
    }

    /// Borrows the complete validated extension value, if recognition succeeded.
    #[must_use]
    pub const fn validated_value(&self) -> Option<&ValidatedExtensionValue> {
        match &self.value {
            RecognitionValue::Validated(value) => Some(value),
            RecognitionValue::Unrecognized(_) => None,
        }
    }

    /// Borrows the original generic TTLV subtree regardless of recognition outcome.
    #[must_use]
    const fn generic_value(&self) -> &Structure {
        match &self.value {
            RecognitionValue::Validated(value) => extension::generic_value(value),
            RecognitionValue::Unrecognized(value) => value,
        }
    }
}

/// Returns whether the generic extension subtree has one exact, schema-valid match.
#[must_use]
pub const fn is_recognized(recognition: &ExtensionRecognition) -> bool {
    recognition.is_recognized()
}

/// Borrows the complete typed value when recognition succeeded.
#[must_use]
pub const fn validated_value(
    recognition: &ExtensionRecognition,
) -> Option<&ValidatedExtensionValue> {
    recognition.validated_value()
}

/// Borrows the original generic subtree for recognized and unrecognized values.
#[must_use]
pub fn generic_value(recognition: &ExtensionRecognition) -> &kmipkit_ttlv::Structure {
    recognition.generic_value()
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
        crate::extension_registry_test_support::record_index_compilation_attempt();

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
    identity: Arc<()>,
}

/// An extension value validated against one exact definition in a client registry.
///
/// This client-owned seal distinguishes registry-validated request data from
/// a `ValidatedExtensionValue` checked against an arbitrary standalone
/// definition. Only [`validate_extension_value`] can create it.
pub struct RegisteredExtensionValue {
    value: ValidatedExtensionValue,
    registry_identity: Arc<()>,
}

impl fmt::Debug for RegisteredExtensionValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RegisteredExtensionValue([REDACTED])")
    }
}

impl RegisteredExtensionValue {
    pub(crate) const fn value(&self) -> &ValidatedExtensionValue {
        &self.value
    }
}

/// Immutable client configuration containing one owned extension registry.
///
/// The production transport constructor can consume this configuration
/// without rebuilding or sharing registry state between clients.
pub struct ClientConfiguration {
    extension_registry: Arc<ClientExtensionRegistry>,
}

/// A registry-validated extension and its caller-selected request criticality.
///
/// This sealed request-use value is the only extension type accepted by
/// [`crate::ClientBatchItem::with_extension`]. It contains no caller-defined
/// conversion, raw wire body, or executable behavior.
pub struct ClientRequestMessageExtension {
    value: RegisteredExtensionValue,
    criticality_indicator: bool,
}

impl fmt::Debug for ClientRequestMessageExtension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ClientRequestMessageExtension([REDACTED])")
    }
}

impl ClientRequestMessageExtension {
    pub(crate) const fn value(&self) -> &RegisteredExtensionValue {
        &self.value
    }

    pub(crate) const fn criticality_indicator(&self) -> bool {
        self.criticality_indicator
    }

    pub(crate) fn identity(&self) -> ExtensionIdentity {
        extension::validated_extension_value_identity(self.value.value())
    }

    /// Checks whether this request value was sealed by the given client configuration.
    pub(crate) fn is_owned_by(&self, configuration: &ClientConfiguration) -> bool {
        Arc::ptr_eq(
            &self.value.registry_identity,
            &configuration.extension_registry.identity,
        )
    }
}

/// Wraps a registry-validated extension for one request with explicit criticality.
///
/// The Boolean is required at each call site; `KMIPKit` does not select a
/// criticality default. The wrapper accepts only a value produced by complete
/// the exact registered definition's schema validation.
///
/// # Errors
///
/// This constructor currently has no error condition. Its fallible return
/// type follows the reviewed cross-language public API contract.
#[allow(clippy::unnecessary_wraps)] // Keep the fallible API contract shared with validating language bindings.
pub fn client_request_message_extension(
    value: RegisteredExtensionValue,
    criticality_indicator: bool,
) -> Result<ClientRequestMessageExtension, ClientError> {
    Ok(ClientRequestMessageExtension {
        value,
        criticality_indicator,
    })
}

impl ClientConfiguration {
    /// Creates a client configuration from one immutable extension registry.
    #[must_use]
    pub fn new(extension_registry: ClientExtensionRegistry) -> Self {
        Self {
            extension_registry: Arc::new(extension_registry),
        }
    }

    /// Creates a configuration that shares one immutable registry snapshot.
    #[doc(hidden)]
    #[must_use]
    pub fn from_shared_registry(extension_registry: Arc<ClientExtensionRegistry>) -> Self {
        Self { extension_registry }
    }

    /// Returns the registry owned by this client configuration.
    #[must_use]
    pub fn extension_registry(&self) -> &ClientExtensionRegistry {
        &self.extension_registry
    }

    /// Returns a shared owner for language-binding accessors.
    #[doc(hidden)]
    #[must_use]
    pub fn shared_extension_registry(&self) -> Arc<ClientExtensionRegistry> {
        Arc::clone(&self.extension_registry)
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
        identity: Arc::new(()),
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

/// Validates a generic extension Structure against an exact registered definition.
///
/// This is the only constructor for [`RegisteredExtensionValue`]. The returned
/// value can enter a typed request only after matching the identity and complete
/// schema of a definition in this immutable registry.
///
/// # Errors
///
/// Returns a redacted `ClientError::Validation` with `NotSent` delivery state
/// when `identity` is absent. Returns a redacted protocol error when the
/// payload violates that registered definition or exceeds `limits`.
pub fn validate_extension_value(
    registry: &ClientExtensionRegistry,
    identity: ExtensionIdentity,
    value: kmipkit_ttlv::Structure,
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<RegisteredExtensionValue, ClientError> {
    let Some(definition) = definition_for_identity(registry, identity) else {
        return Err(ClientError::validation(
            crate::ClientCauseCategory::InvalidInput,
            RequestDeliveryState::NotSent,
            UnregisteredExtension,
        ));
    };
    let value = extension::validate(definition, value, limits)
        .map_err(|error| ClientError::protocol(error, RequestDeliveryState::NotSent))?;
    Ok(RegisteredExtensionValue {
        value,
        registry_identity: Arc::clone(&registry.identity),
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

/// Inspects one generic vendor extension subtree against this immutable registry.
///
/// The original subtree is retained whether it is recognized or not. Zero or
/// multiple exact discriminator matches remain unrecognized; one exact match
/// proceeds to complete schema validation. The lookup uses one bounded index
/// shared across all registered paths and never reorders the source tree.
///
/// # Errors
///
/// Returns a redacted `ResourceLimit` error with `NotSent` delivery evidence
/// when TTLV, payload-index, or tag-comparison bounds are exceeded. No partial
/// recognition result is returned on error.
pub fn inspect(
    registry: &ClientExtensionRegistry,
    vendor_identifier: &str,
    value: kmipkit_ttlv::Structure,
    limits: &kmipkit_ttlv::codec::CodecLimits,
) -> Result<ExtensionRecognition, ClientError> {
    if u64::try_from(vendor_identifier.len()).map_or(true, |length| {
        length > registry.limits.max_text_bytes_per_field()
    }) {
        return Err(ClientError::protocol(
            registry_error(ProtocolErrorKind::ResourceLimit),
            RequestDeliveryState::NotSent,
        ));
    }

    let index = PayloadTagIndex::build(&value, limits, registry.limits.max_payload_index_records())
        .map_err(|error| ClientError::protocol(error, RequestDeliveryState::NotSent))?;

    let mut comparisons = 0_u64;
    let mut exact_match = None;
    for (definition_index, definition) in registry.definitions.iter().enumerate() {
        let path = definition.discriminator().path().tags();
        let mut lookup_budget = LookupBudget {
            maximum_comparisons: registry.limits.max_lookup_comparisons(),
            comparisons: &mut comparisons,
        };
        let matched = index
            .resolve_path(&value.view(), path, &mut lookup_budget, |scalar| {
                let Some(fingerprint) = extension::scalar_value_fingerprint(&scalar) else {
                    return false;
                };
                let Some(candidates) =
                    discriminator_candidates(registry, vendor_identifier, path, fingerprint)
                else {
                    return false;
                };

                let mut exact = false;
                for candidate_index in candidates.iter().copied() {
                    if candidate_matches_discriminator(
                        registry,
                        candidate_index,
                        vendor_identifier,
                        path,
                        &scalar,
                    ) && candidate_index == definition_index
                    {
                        exact = true;
                    }
                }
                exact
            })
            .map_err(|error| ClientError::protocol(error, RequestDeliveryState::NotSent))?
            .unwrap_or(false);

        if matched && exact_match != Some(definition_index) {
            if exact_match.is_some() {
                return Ok(ExtensionRecognition {
                    value: RecognitionValue::Unrecognized(value),
                });
            }
            exact_match = Some(definition_index);
        }
    }

    let Some(definition_index) = exact_match else {
        return Ok(ExtensionRecognition {
            value: RecognitionValue::Unrecognized(value),
        });
    };
    let definition = registry.definitions.get(definition_index).ok_or_else(|| {
        ClientError::protocol(
            registry_error(ProtocolErrorKind::InvalidSchema),
            RequestDeliveryState::NotSent,
        )
    })?;
    let value = match extension::validate_schema_only(definition, value, limits)
        .map_err(|error| ClientError::protocol(error, RequestDeliveryState::NotSent))?
    {
        extension::SchemaValidationOutcome::SchemaValid(value) => {
            RecognitionValue::Validated(value)
        }
        extension::SchemaValidationOutcome::SchemaInvalid(value) => {
            RecognitionValue::Unrecognized(value)
        }
    };
    Ok(ExtensionRecognition { value })
}

impl PayloadTagIndex {
    fn build(
        value: &Structure,
        codec_limits: &CodecLimits,
        max_index_records: u64,
    ) -> Result<Self, ProtocolError> {
        let max_index_records = usize::try_from(max_index_records)
            .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let max_depth = codec_limits.max_structure_depth();
        let mut accounting = PayloadAccounting {
            item_count: 1,
            ..PayloadAccounting::default()
        };
        if accounting.item_count > codec_limits.max_elements() {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }
        let encoded_size = count_payload_structure(
            &value.view(),
            1,
            max_depth,
            codec_limits,
            max_index_records,
            &mut accounting,
        )?;
        if encoded_size > codec_limits.max_message_bytes() {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }

        let child_count = accounting.child_count;
        let structure_count = accounting.structure_count;
        let mut structures = Vec::new();
        structures
            .try_reserve_exact(structure_count)
            .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(child_count)
            .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(accounting.maximum_width)
            .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
        scratch.resize(accounting.maximum_width, PayloadEntry::default());

        let mut index = Self {
            structures,
            entries,
            scratch,
        };
        let root = value.view();
        index.append_structure(&root)?;
        Ok(index)
    }

    fn append_structure(&mut self, view: &StructureView<'_>) -> Result<usize, ProtocolError> {
        let structure_index = self.structures.len();
        let entry_start = self.entries.len();
        let children = view.children();
        self.structures.push(IndexedStructure {
            entry_start,
            entry_count: children.len(),
        });
        for (child_index, child) in children.iter().enumerate() {
            self.entries.push(PayloadEntry {
                tag: child.tag().raw(),
                child_index,
                nested_structure: None,
            });
        }

        for (child_index, child) in children.iter().enumerate() {
            let nested_index = child.with_value(|value| match value {
                ValueView::Structure(nested) => self.append_structure(&nested).map(Some),
                _ => Ok(None),
            })?;
            let entry_index = entry_start
                .checked_add(child_index)
                .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            let entry = self
                .entries
                .get_mut(entry_index)
                .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            entry.nested_structure = nested_index;
        }

        self.radix_sort_structure(entry_start, children.len())?;
        Ok(structure_index)
    }

    fn radix_sort_structure(&mut self, start: usize, count: usize) -> Result<(), ProtocolError> {
        let end = start
            .checked_add(count)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let entries = self
            .entries
            .get_mut(start..end)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let scratch = self
            .scratch
            .get_mut(..count)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        for shift in [0_u32, 8, 16] {
            let mut counts = [0_usize; 256];
            for entry in entries.iter() {
                let bucket = usize::try_from((entry.tag >> shift) & 0xff)
                    .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
                let count = counts
                    .get_mut(bucket)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
                *count = count
                    .checked_add(1)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            }
            let mut next = [0_usize; 256];
            let mut offset = 0_usize;
            for (bucket, count) in counts.iter().copied().enumerate() {
                let position = next
                    .get_mut(bucket)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
                *position = offset;
                offset = offset
                    .checked_add(count)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            }
            for entry in entries.iter().copied() {
                let bucket = usize::try_from((entry.tag >> shift) & 0xff)
                    .map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
                let position = next
                    .get_mut(bucket)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
                let destination = scratch
                    .get_mut(*position)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
                *destination = entry;
                *position = position
                    .checked_add(1)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            }
            entries.copy_from_slice(scratch);
        }
        Ok(())
    }

    fn resolve_path<F>(
        &self,
        root: &StructureView<'_>,
        path: &[Tag],
        budget: &mut LookupBudget<'_>,
        mut terminal: F,
    ) -> Result<Option<bool>, ProtocolError>
    where
        F: for<'value> FnMut(ValueView<'value>) -> bool,
    {
        self.resolve_path_at(0, root, path, 0, budget, &mut terminal)
    }

    fn resolve_path_at<F>(
        &self,
        structure_index: usize,
        current: &StructureView<'_>,
        path: &[Tag],
        path_position: usize,
        budget: &mut LookupBudget<'_>,
        terminal: &mut F,
    ) -> Result<Option<bool>, ProtocolError>
    where
        F: for<'value> FnMut(ValueView<'value>) -> bool,
    {
        let Some(tag) = path.get(path_position) else {
            return Ok(None);
        };
        let Some(entry) = self.unique_child(structure_index, tag.raw(), budget)? else {
            return Ok(None);
        };
        let child = current
            .children()
            .get(entry.child_index)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        if path_position + 1 == path.len() {
            return child.with_value(|value| Ok(Some(terminal(value))));
        }
        let Some(nested_index) = entry.nested_structure else {
            return Ok(None);
        };
        child.with_value(|value| match value {
            ValueView::Structure(nested) => self.resolve_path_at(
                nested_index,
                &nested,
                path,
                path_position + 1,
                budget,
                terminal,
            ),
            _ => Ok(None),
        })
    }

    fn unique_child(
        &self,
        structure_index: usize,
        tag: u32,
        budget: &mut LookupBudget<'_>,
    ) -> Result<Option<PayloadEntry>, ProtocolError> {
        let structure = self
            .structures
            .get(structure_index)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let start = structure.entry_start;
        let end = start
            .checked_add(structure.entry_count)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let mut step_comparisons = 0_u64;
        let mut low = start;
        let mut high = end;
        while low < high {
            let middle = low + (high - low) / 2;
            let entry = self
                .entries
                .get(middle)
                .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            charge_tag_comparison(budget, &mut step_comparisons)?;
            if entry.tag < tag {
                low = middle.saturating_add(1);
            } else {
                high = middle;
            }
        }
        if low >= end {
            return Ok(None);
        }
        let first = self
            .entries
            .get(low)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        charge_tag_comparison(budget, &mut step_comparisons)?;
        if first.tag != tag {
            return Ok(None);
        }

        let first_after_match = low
            .checked_add(1)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        let mut upper = first_after_match;
        let mut upper_end = end;
        while upper < upper_end {
            let middle = upper + (upper_end - upper) / 2;
            let entry = self
                .entries
                .get(middle)
                .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
            charge_tag_comparison(budget, &mut step_comparisons)?;
            if entry.tag <= tag {
                upper = middle.saturating_add(1);
            } else {
                upper_end = middle;
            }
        }
        if upper != first_after_match {
            return Ok(None);
        }
        Ok(Some(*first))
    }
}

fn count_payload_structure(
    structure: &StructureView<'_>,
    depth: usize,
    maximum_depth: usize,
    codec_limits: &CodecLimits,
    max_index_records: usize,
    accounting: &mut PayloadAccounting,
) -> Result<usize, ProtocolError> {
    if depth > maximum_depth {
        return Err(registry_error(ProtocolErrorKind::ResourceLimit));
    }
    accounting.structure_count = accounting
        .structure_count
        .checked_add(1)
        .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
    if accounting
        .structure_count
        .checked_add(accounting.child_count)
        .is_none_or(|records| records > max_index_records)
    {
        return Err(registry_error(ProtocolErrorKind::ResourceLimit));
    }
    let mut encoded_size = 8_usize;
    accounting.maximum_width = accounting.maximum_width.max(structure.children().len());
    for child in structure.children() {
        accounting.item_count = accounting
            .item_count
            .checked_add(1)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        if accounting.item_count > codec_limits.max_elements() {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }
        if accounting.item_count > MAX_PAYLOAD_ITEMS {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }
        accounting.child_count = accounting
            .child_count
            .checked_add(1)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        if accounting
            .structure_count
            .checked_add(accounting.child_count)
            .is_none_or(|records| records > max_index_records)
        {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }

        let child_size = child.with_value(|value| match value {
            ValueView::Structure(nested) => count_payload_structure(
                &nested,
                depth.saturating_add(1),
                maximum_depth,
                codec_limits,
                max_index_records,
                accounting,
            ),
            scalar => {
                let payload_length = scalar_payload_length(&scalar)
                    .ok_or_else(|| registry_error(ProtocolErrorKind::UnsupportedValue))?;
                padded_item_size(payload_length)
            }
        })?;
        encoded_size = encoded_size
            .checked_add(child_size)
            .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
        if encoded_size > codec_limits.max_message_bytes() {
            return Err(registry_error(ProtocolErrorKind::ResourceLimit));
        }
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
    let padded_length = payload_length
        .checked_add(7)
        .map(|length| (length / 8) * 8)
        .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
    8_usize
        .checked_add(padded_length)
        .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))
}

fn charge_tag_comparison(
    budget: &mut LookupBudget<'_>,
    step_comparisons: &mut u64,
) -> Result<(), ProtocolError> {
    *budget.comparisons = budget
        .comparisons
        .checked_add(1)
        .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
    *step_comparisons = step_comparisons
        .checked_add(1)
        .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
    if *step_comparisons > MAX_TAG_COMPARISONS_PER_STEP {
        return Err(registry_error(ProtocolErrorKind::ResourceLimit));
    }
    if *budget.comparisons > budget.maximum_comparisons {
        return Err(registry_error(ProtocolErrorKind::ResourceLimit));
    }
    Ok(())
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
        add_bounded_total(
            &mut text_bytes,
            accounting.text_bytes,
            limits.max_registry_text_bytes(),
        )?;
        add_bounded_total(
            &mut schema_nodes,
            accounting.schema_nodes,
            limits.max_schema_nodes(),
        )?;
        add_bounded_total(
            &mut discriminator_bytes,
            accounting.discriminator_bytes,
            limits.max_total_discriminator_scalar_bytes(),
        )?;
        add_bounded_total(
            &mut constraint_members,
            accounting.constraint_members,
            limits.max_total_constraint_members(),
        )?;
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

fn add_bounded_total(total: &mut u64, increment: usize, maximum: u64) -> Result<(), ProtocolError> {
    let increment =
        u64::try_from(increment).map_err(|_| registry_error(ProtocolErrorKind::ResourceLimit))?;
    *total = total
        .checked_add(increment)
        .ok_or_else(|| registry_error(ProtocolErrorKind::ResourceLimit))?;
    if *total > maximum {
        return Err(registry_error(ProtocolErrorKind::ResourceLimit));
    }
    Ok(())
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

#[derive(Debug)]
struct UnregisteredExtension;

impl fmt::Display for UnregisteredExtension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("extension identity is not registered")
    }
}

impl Error for UnregisteredExtension {}

#[cfg(test)]
#[path = "../tests/unit/extension_registry_tests.rs"]
mod tests;
