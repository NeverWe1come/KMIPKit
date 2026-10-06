//! Non-empty, ordered KMIP Authentication values.

use std::fmt;

use kmipkit_ttlv::{Item, RawTag, Structure, StructureView, Tag, Value, ValueView};

use super::secret;
use super::{Credential, CredentialType, CredentialValidationError, validation};

const CREDENTIAL: u32 = 0x0042_0023;

struct CredentialIndex {
    child_index: usize,
    credential_type: CredentialType,
}

/// A KMIP Authentication Structure containing one or more ordered Credentials.
///
/// The original generic TTLV tree is retained without copying credential
/// payloads. This model does not determine whether a server accepts or
/// satisfies any Credential.
pub struct Authentication {
    tree: Structure,
    credentials: Vec<CredentialIndex>,
}

impl Authentication {
    /// Creates Authentication from one or more Credentials in caller-supplied order.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CredentialValidationErrorKind::EmptyAuthentication`] when
    /// `credentials` is empty, or a sanitized model error if an internally
    /// defined TTLV field cannot be represented.
    pub fn new(credentials: Vec<Credential>) -> Result<Self, CredentialValidationError> {
        if credentials.is_empty() {
            return Err(validation::empty_authentication_error());
        }

        let credential_tag = checked_credential_tag()?;
        let mut tree = Structure::new();
        let mut entries = Vec::with_capacity(credentials.len());

        for credential in credentials {
            let credential_type = credential.credential_type();
            let child_index = entries.len();
            let field = Item::new(credential_tag, Value::structure(credential.into_tree()))
                .map_err(|_| invalid_ttlv_structure())?;
            tree.try_push(field).map_err(|_| invalid_ttlv_structure())?;
            entries.push(CredentialIndex {
                child_index,
                credential_type,
            });
        }

        Ok(Self {
            tree,
            credentials: entries,
        })
    }

    /// Validates and takes ownership of one generic Authentication Structure.
    ///
    /// Repeated Credential fields remain in source order. Unknown children are
    /// retained in the original tree but are not assigned Credential semantics.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for an empty value or a Credential field
    /// with an invalid generic shape.
    pub fn try_from_ttlv(tree: Structure) -> Result<Self, CredentialValidationError> {
        let mut credentials = Vec::new();

        for (child_index, field) in tree.view().children().iter().enumerate() {
            if field.tag().raw() != CREDENTIAL {
                continue;
            }

            let credential_type = field.with_value(|value| match value {
                ValueView::Structure(credential) => {
                    Some(validation::credential_type_from_view(&credential))
                }
                _ => None,
            });
            let Some(credential_type) = credential_type else {
                return Err(validation::wrong_field_type_error());
            };

            credentials.push(CredentialIndex {
                child_index,
                credential_type: credential_type?,
            });
        }

        if credentials.is_empty() {
            return Err(validation::empty_authentication_error());
        }

        Ok(Self { tree, credentials })
    }

    /// Returns Credential entries in source order as callback-scoped views.
    #[must_use = "iterate over the credential views to inspect them"]
    pub fn credentials(&self) -> impl ExactSizeIterator<Item = CredentialView<'_>> + '_ {
        self.credentials.iter().map(|entry| CredentialView {
            authentication: self,
            child_index: entry.child_index,
            credential_type: entry.credential_type,
        })
    }

    /// Lends the original ordered generic Authentication tree for callback-scoped access.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.tree.view())
    }

    pub(super) fn into_tree(self) -> Structure {
        self.tree
    }
}

/// A callback-scoped view of one Credential contained in Authentication.
pub struct CredentialView<'a> {
    authentication: &'a Authentication,
    child_index: usize,
    credential_type: CredentialType,
}

impl CredentialView<'_> {
    /// Returns the typed discriminator while retaining every raw value.
    #[must_use]
    pub const fn credential_type(&self) -> CredentialType {
        self.credential_type
    }

    /// Returns the original raw Credential Type enumeration.
    #[must_use]
    pub const fn credential_type_raw(&self) -> u32 {
        self.credential_type.raw()
    }

    /// Lends the Credential's original TTLV structure during `callback`.
    ///
    /// The `None` case is unreachable for a view produced by a validated
    /// Authentication value; it is retained in the signature so malformed
    /// future Item Type variants cannot be converted into a panic.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> Option<R> {
        self.authentication
            .tree
            .view()
            .children()
            .get(self.child_index)
            .and_then(|field| {
                field.with_value(|value| match value {
                    ValueView::Structure(structure) => Some(callback(structure)),
                    _ => None,
                })
            })
    }
}

impl fmt::Debug for Authentication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        secret::format_debug_struct(formatter, "Authentication", |debug| {
            debug.field("credential_count", &self.credentials.len());
            Ok(())
        })
    }
}

impl fmt::Display for Authentication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "KMIP Authentication ({} Credentials)",
            self.credentials.len()
        )
    }
}

impl fmt::Debug for CredentialView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        secret::format_debug_struct(formatter, "CredentialView", |debug| {
            debug.field("credential_type", &self.credential_type);
            Ok(())
        })
    }
}

fn checked_credential_tag() -> Result<Tag, CredentialValidationError> {
    RawTag::new(CREDENTIAL)
        .and_then(|raw| raw.try_checked())
        .map_err(|_| invalid_ttlv_structure())
}

fn invalid_ttlv_structure() -> CredentialValidationError {
    validation::invalid_ttlv_structure_error()
}
