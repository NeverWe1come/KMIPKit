//! Raw-preserving Hashed Password Credential Value.

use std::fmt;

use kmipkit_ttlv::{Structure, StructureView, ValueView};

use super::validation::{FieldKind, FieldRule};
use super::{CredentialValidationError, secret, validation};

const USERNAME: u32 = 0x0042_0099;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const TIME_STAMP: u32 = 0x0042_0092;
const HASHED_PASSWORD: u32 = 0x0042_0157;
const DEFAULT_HASHING_ALGORITHM_SHA256: u32 = 6;

const HASHED_PASSWORD_RULES: &[FieldRule] = &[
    FieldRule {
        tag: USERNAME,
        kind: FieldKind::TextString,
        required: true,
    },
    FieldRule {
        tag: TIME_STAMP,
        kind: FieldKind::DateTimeExtended,
        required: true,
    },
    FieldRule {
        tag: HASHING_ALGORITHM,
        kind: FieldKind::Enumeration,
        required: false,
    },
    FieldRule {
        tag: HASHED_PASSWORD,
        kind: FieldKind::ByteString,
        required: true,
    },
];

/// A caller-supplied Hashed Password Credential Value.
pub struct HashedPasswordCredential {
    tree: Structure,
}

impl HashedPasswordCredential {
    /// Validates the Hashed Password table members and takes ownership of the tree.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for missing, duplicated, incorrectly typed,
    /// or noncanonically ordered known members.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, CredentialValidationError> {
        validation::validate_fields(&tree.view(), HASHED_PASSWORD_RULES, &[])?;
        Ok(Self { tree })
    }

    pub(super) fn field_count(&self) -> usize {
        self.tree.view().children().len()
    }

    /// Returns the explicitly supplied raw Hashing Algorithm, if present.
    #[must_use]
    pub fn hashing_algorithm_raw(&self) -> Option<u32> {
        self.tree.view().children().iter().find_map(|field| {
            if field.tag().raw() != HASHING_ALGORITHM {
                return None;
            }
            field.with_value(|value| match value {
                ValueView::Enumeration(raw) => Some(*raw),
                _ => None,
            })
        })
    }

    /// Returns the effective Hashing Algorithm without inserting an absent field.
    #[must_use]
    pub fn effective_hashing_algorithm_raw(&self) -> u32 {
        self.hashing_algorithm_raw()
            .unwrap_or(DEFAULT_HASHING_ALGORITHM_SHA256)
    }

    /// Lends the original ordered TTLV Structure for callback-scoped access.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.tree.view())
    }

    /// Returns the original ordered TTLV Structure without rebuilding it.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.tree
    }
}

impl fmt::Debug for HashedPasswordCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        secret::format_debug_struct(formatter, "HashedPasswordCredential", |debug| {
            debug.field("field_count", &self.field_count());
            Ok(())
        })
    }
}

impl fmt::Display for HashedPasswordCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        secret::format_redacted_display(formatter, "Hashed Password Credential")
    }
}

/// Caller-owned state for producing increasing Hashed Password Timestamps.
///
/// Keep one sequence for each Username used by a logical credential producer.
/// The caller supplies the current Date Time Extended value at hash-production
/// time and persists [`Self::last_timestamp`] if the sequence must survive a
/// process restart. This type does not read a clock, calculate a hash, or send
/// a Credential.
#[must_use]
pub struct HashedPasswordTimestampSequence {
    last_timestamp: Option<i64>,
}

impl HashedPasswordTimestampSequence {
    /// Creates a sequence restored from its previously accepted timestamp.
    ///
    /// Pass `None` for a new logical client and Username, or pass the last
    /// timestamp persisted by the caller to continue the same sequence.
    pub const fn new(last_timestamp: Option<i64>) -> Self {
        Self { last_timestamp }
    }

    /// Records a caller-supplied current timestamp if it strictly increases.
    ///
    /// Call this before producing the hash that uses `timestamp`. A rejected
    /// value leaves the sequence unchanged. Each instance represents one
    /// Username within one logical credential producer; callers must share or
    /// restore that state wherever credentials for that Username are produced.
    ///
    /// # Errors
    ///
    /// Returns a redacted error when `timestamp` is equal to or earlier than
    /// the previously recorded value.
    pub fn record(&mut self, timestamp: i64) -> Result<(), HashedPasswordTimestampSequenceError> {
        if self
            .last_timestamp
            .is_some_and(|previous| timestamp <= previous)
        {
            return Err(HashedPasswordTimestampSequenceError);
        }

        self.last_timestamp = Some(timestamp);
        Ok(())
    }

    /// Returns the last accepted timestamp for caller-managed persistence.
    #[must_use]
    pub const fn last_timestamp(&self) -> Option<i64> {
        self.last_timestamp
    }
}

impl fmt::Debug for HashedPasswordTimestampSequence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HashedPasswordTimestampSequence")
            .field("has_previous_timestamp", &self.last_timestamp.is_some())
            .finish()
    }
}

/// Redacted rejection of a non-increasing Hashed Password Timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HashedPasswordTimestampSequenceError;

impl fmt::Display for HashedPasswordTimestampSequenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Hashed Password Timestamp must strictly increase")
    }
}

impl std::error::Error for HashedPasswordTimestampSequenceError {}
