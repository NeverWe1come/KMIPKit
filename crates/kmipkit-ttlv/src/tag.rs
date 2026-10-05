//! Raw and allocation-checked KMIP tags.

use crate::error::ModelError;
use crate::generated::{EXACT_TAG_ALLOCATIONS, TAG_ALLOCATION_RANGES, TagAllocationKind};

/// A 24-bit tag value that has not been checked against the KMIP allocation catalog.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RawTag(u32);

impl RawTag {
    /// Creates a raw tag, rejecting values wider than the KMIP 24-bit tag field.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::RawTagOutOfRange`] when `raw` exceeds the 24-bit
    /// field. The error contains only its category; it does not retain or
    /// expose the supplied value.
    pub fn new(raw: u32) -> Result<Self, ModelError> {
        if raw > 0x00FF_FFFF {
            return Err(ModelError::RawTagOutOfRange);
        }

        Ok(Self(raw))
    }

    /// Returns this tag's original 24-bit value.
    #[must_use]
    pub const fn raw(&self) -> u32 {
        self.0
    }

    /// Checks this tag against the generated KMIP 2.1 allocation catalog.
    ///
    /// Exact catalog records take precedence over overlapping aggregate
    /// ranges, following the `KMIPKit` project policy recorded in ADR-0010.
    /// This checks allocation only; it does not establish TTLV wire validity,
    /// Structure schema validity, or extension semantics.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::TagNotAllocated`] when the tag is reserved or
    /// unallocated by the KMIP 2.1 catalog policy. The error contains only its
    /// category; it does not retain or expose the rejected tag value.
    pub fn try_checked(&self) -> Result<Tag, ModelError> {
        match allocation_kind(self.0) {
            Some(TagAllocationKind::Assigned | TagAllocationKind::Extension) => Ok(Tag(self.0)),
            Some(TagAllocationKind::Reserved | TagAllocationKind::Unused) | None => {
                Err(ModelError::TagNotAllocated)
            }
        }
    }

    pub(crate) fn is_reserved(self) -> bool {
        matches!(allocation_kind(self.0), Some(TagAllocationKind::Reserved))
    }
}

/// A tag whose KMIP 2.1 allocation has been checked.
///
/// Allocation follows `KMIPKit`'s project policy for exact records and
/// aggregate ranges as recorded in ADR-0010. A checked tag is not a claim that
/// its TTLV framing, Structure schema, extension semantics, or operation
/// meaning is valid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Tag(u32);

impl Tag {
    /// Returns this tag's original 24-bit value.
    #[must_use]
    pub const fn raw(&self) -> u32 {
        self.0
    }
}

fn allocation_kind(raw: u32) -> Option<TagAllocationKind> {
    if let Ok(index) = EXACT_TAG_ALLOCATIONS.binary_search_by_key(&raw, |(value, _)| *value) {
        return Some(EXACT_TAG_ALLOCATIONS[index].1);
    }

    TAG_ALLOCATION_RANGES
        .iter()
        .find(|(start, end, _)| *start <= raw && raw <= *end)
        .map(|(_, _, kind)| *kind)
}
