//! Typed KMIP 2.1 Key Wrapping metadata (§3.3, Tables 14–17).
#![allow(clippy::missing_errors_doc)] // Fallible constructors share the sanitized schema error documented on ManagedObjectError.

use kmipkit_ttlv::{Item, ItemType, Structure, StructureView, Value, ValueView};

use crate::ManagedObjectError;
use crate::UniqueIdentifier;
use crate::managed_object::{make_item, make_structure_item};

const KEY_WRAPPING_DATA: u32 = 0x0042_0046;
const WRAPPING_METHOD: u32 = 0x0042_009E;
const ENCRYPTION_KEY_INFORMATION: u32 = 0x0042_0036;
const MAC_SIGNATURE_KEY_INFORMATION: u32 = 0x0042_004E;
const MAC_SIGNATURE: u32 = 0x0042_004D;
const IV_COUNTER_NONCE: u32 = 0x0042_003D;
const ENCODING_OPTION: u32 = 0x0042_00A3;
const UNIQUE_IDENTIFIER: u32 = 0x0042_0094;
const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;

fn structure_item(
    raw: u32,
    fields: impl IntoIterator<Item = Item>,
) -> Result<Item, ManagedObjectError> {
    make_structure_item(raw, fields)
}

/// Encryption Key Information containing a required Unique Identifier.
pub struct EncryptionKeyInformation {
    item: Item,
}

impl EncryptionKeyInformation {
    /// Creates key information with a text Unique Identifier.
    pub fn try_new(unique_identifier: String) -> Result<Self, ManagedObjectError> {
        Self::try_from_unique_identifier(UniqueIdentifier::TextString(unique_identifier))
    }

    /// Creates key information with any KMIP Unique Identifier alternative.
    pub fn try_from_unique_identifier(
        unique_identifier: UniqueIdentifier,
    ) -> Result<Self, ManagedObjectError> {
        let item = structure_item(
            ENCRYPTION_KEY_INFORMATION,
            [unique_identifier_item(unique_identifier)?],
        )?;
        Ok(Self { item })
    }

    /// Reads a valid Encryption Key Information Structure losslessly.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        validate_key_information(&item, ENCRYPTION_KEY_INFORMATION)?;
        Ok(Self { item })
    }

    /// Lends the Structure for callback-scoped inspection.
    pub fn with_raw<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Result<R, ManagedObjectError> {
        with_structure(&self.item, callback)
    }

    /// Adds optional Cryptographic Parameters after Unique Identifier.
    pub fn with_cryptographic_parameters(
        mut self,
        parameters: Structure,
    ) -> Result<Self, ManagedObjectError> {
        self.item =
            append_cryptographic_parameters(&self.item, ENCRYPTION_KEY_INFORMATION, parameters)?;
        Ok(self)
    }

    /// Adds typed Cryptographic Parameters after the Unique Identifier.
    pub fn with_typed_cryptographic_parameters(
        self,
        parameters: crate::CryptographicParameters,
    ) -> Result<Self, ManagedObjectError> {
        self.with_cryptographic_parameters(parameters.into_ttlv())
    }

    /// Consumes this value into its Structure Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }
}

/// MAC/Signature Key Information containing a required Unique Identifier.
pub struct MacSignatureKeyInformation {
    item: Item,
}

impl MacSignatureKeyInformation {
    /// Creates key information with a text Unique Identifier.
    pub fn try_new(unique_identifier: String) -> Result<Self, ManagedObjectError> {
        Self::try_from_unique_identifier(UniqueIdentifier::TextString(unique_identifier))
    }

    /// Creates key information with any KMIP Unique Identifier alternative.
    pub fn try_from_unique_identifier(
        unique_identifier: UniqueIdentifier,
    ) -> Result<Self, ManagedObjectError> {
        Ok(Self {
            item: structure_item(
                MAC_SIGNATURE_KEY_INFORMATION,
                [unique_identifier_item(unique_identifier)?],
            )?,
        })
    }

    /// Reads a valid MAC/Signature Key Information Structure losslessly.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        validate_key_information(&item, MAC_SIGNATURE_KEY_INFORMATION)?;
        Ok(Self { item })
    }

    /// Lends the Structure for callback-scoped inspection.
    pub fn with_raw<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Result<R, ManagedObjectError> {
        with_structure(&self.item, callback)
    }

    /// Adds optional Cryptographic Parameters after Unique Identifier.
    pub fn with_cryptographic_parameters(
        mut self,
        parameters: Structure,
    ) -> Result<Self, ManagedObjectError> {
        self.item =
            append_cryptographic_parameters(&self.item, MAC_SIGNATURE_KEY_INFORMATION, parameters)?;
        Ok(self)
    }

    /// Adds typed Cryptographic Parameters after the Unique Identifier.
    pub fn with_typed_cryptographic_parameters(
        self,
        parameters: crate::CryptographicParameters,
    ) -> Result<Self, ManagedObjectError> {
        self.with_cryptographic_parameters(parameters.into_ttlv())
    }

    /// Consumes this value into its Structure Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }
}

/// Typed Key Wrapping Data builder; it carries metadata and performs no cryptography.
pub struct KeyWrappingData {
    wrapping_method: u32,
    encryption_key_information: Option<EncryptionKeyInformation>,
    mac_signature_key_information: Option<MacSignatureKeyInformation>,
    mac_signature: Option<Vec<u8>>,
    iv_counter_nonce: Option<Vec<u8>>,
    encoding_option: Option<u32>,
    raw_item: Option<Item>,
}

impl KeyWrappingData {
    /// Starts Key Wrapping Data with its required, open Wrapping Method value.
    pub fn try_new(wrapping_method: u32) -> Result<Self, ManagedObjectError> {
        Ok(Self {
            wrapping_method,
            encryption_key_information: None,
            mac_signature_key_information: None,
            mac_signature: None,
            iv_counter_nonce: None,
            encoding_option: None,
            raw_item: None,
        })
    }

    /// Reads Key Wrapping Data and retains unknown members, repeats, and order.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        let wrapping_method = Self::validate_item(&item)?;
        Ok(Self {
            wrapping_method,
            encryption_key_information: None,
            mac_signature_key_information: None,
            mac_signature: None,
            iv_counter_nonce: None,
            encoding_option: None,
            raw_item: Some(item),
        })
    }

    pub(crate) fn validate_item(item: &Item) -> Result<u32, ManagedObjectError> {
        if item.tag().raw() != KEY_WRAPPING_DATA || item.item_type() != ItemType::Structure {
            return Err(ManagedObjectError::InvalidStructure);
        }
        let mut present = [false; 6];
        let mut last_position = None;
        let mut wrapping_method = None;
        item.with_value(|value| {
            let ValueView::Structure(structure) = value else {
                return Err(ManagedObjectError::InvalidStructure); // COVERAGE-EXCLUDE: ItemType::Structure guarantees this ValueView variant.
            };
            for child in structure.children() {
                let (position, expected) = match child.tag().raw() {
                    WRAPPING_METHOD => (0, ItemType::Enumeration),
                    ENCRYPTION_KEY_INFORMATION => (1, ItemType::Structure),
                    MAC_SIGNATURE_KEY_INFORMATION => (2, ItemType::Structure),
                    MAC_SIGNATURE => (3, ItemType::ByteString),
                    IV_COUNTER_NONCE => (4, ItemType::ByteString),
                    ENCODING_OPTION => (5, ItemType::Enumeration),
                    _ => continue,
                };
                if present[position] {
                    return Err(ManagedObjectError::DuplicateField);
                }
                if last_position.is_some_and(|last| position < last) {
                    return Err(ManagedObjectError::IncorrectFieldOrder);
                }
                if child.item_type() != expected {
                    return Err(ManagedObjectError::IncorrectFieldType);
                }
                if position == 0 {
                    wrapping_method = child.with_value(|value| match value {
                        ValueView::Enumeration(raw) => Some(*raw),
                        _ => None, // COVERAGE-EXCLUDE: ItemType::Enumeration guarantees this ValueView variant.
                    });
                }
                if position == 1 {
                    validate_key_information(child, ENCRYPTION_KEY_INFORMATION)?;
                }
                if position == 2 {
                    validate_key_information(child, MAC_SIGNATURE_KEY_INFORMATION)?;
                }
                present[position] = true;
                last_position = Some(position);
            }
            if present[0] {
                Ok(())
            } else {
                Err(ManagedObjectError::MissingRequiredField)
            }
        })?;
        if !present[1] && !present[2] {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        wrapping_method.ok_or(ManagedObjectError::IncorrectFieldType)
    }

    /// Lends the full structure for callback-scoped inspection.
    pub fn with_raw<R>(
        &self,
        callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
    ) -> Result<R, ManagedObjectError> {
        if let Some(item) = &self.raw_item {
            return with_structure(item, callback);
        }
        Err(ManagedObjectError::InvalidStructure)
    }

    /// Sets Encryption Key Information.
    #[must_use]
    pub fn with_encryption_key_information(mut self, value: EncryptionKeyInformation) -> Self {
        self.encryption_key_information = Some(value);
        self
    }
    /// Sets MAC/Signature Key Information.
    #[must_use]
    pub fn with_mac_signature_key_information(mut self, value: MacSignatureKeyInformation) -> Self {
        self.mac_signature_key_information = Some(value);
        self
    }
    /// Sets MAC/Signature bytes without interpreting or computing them.
    #[must_use]
    pub fn with_mac_signature(mut self, value: Vec<u8>) -> Self {
        self.mac_signature = Some(value);
        self
    }
    /// Sets IV/Counter/Nonce bytes.
    #[must_use]
    pub fn with_iv_counter_nonce(mut self, value: Vec<u8>) -> Self {
        self.iv_counter_nonce = Some(value);
        self
    }
    /// Sets the open Encoding Option value.
    #[must_use]
    pub fn with_encoding_option(mut self, value: u32) -> Self {
        self.encoding_option = Some(value);
        self
    }

    /// Consumes the builder into a source-ordered Key Wrapping Data Item.
    ///
    /// # Errors
    /// Returns a sanitized model error if local TTLV nesting limits are exceeded.
    pub fn into_item(self) -> Result<Item, ManagedObjectError> {
        self.try_into_item()
    }

    /// Consumes the builder into a source-ordered Key Wrapping Data Item.
    pub(crate) fn try_into_item(self) -> Result<Item, ManagedObjectError> {
        if let Some(item) = self.raw_item {
            return Ok(item);
        }
        if self.encryption_key_information.is_none() && self.mac_signature_key_information.is_none()
        {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        let mut fields = vec![make_item(
            WRAPPING_METHOD,
            Value::enumeration(self.wrapping_method),
        )?];
        if let Some(value) = self.encryption_key_information {
            fields.push(value.into_item());
        }
        if let Some(value) = self.mac_signature_key_information {
            fields.push(value.into_item());
        }
        if let Some(value) = self.mac_signature {
            fields.push(make_item(MAC_SIGNATURE, Value::byte_string(value))?);
        }
        if let Some(value) = self.iv_counter_nonce {
            fields.push(make_item(IV_COUNTER_NONCE, Value::byte_string(value))?);
        }
        if let Some(value) = self.encoding_option {
            fields.push(make_item(ENCODING_OPTION, Value::enumeration(value))?);
        }
        structure_item(KEY_WRAPPING_DATA, fields)
    }
}

fn validate_key_information(item: &Item, expected_tag: u32) -> Result<(), ManagedObjectError> {
    if item.tag().raw() != expected_tag || item.item_type() != ItemType::Structure {
        return Err(ManagedObjectError::InvalidStructure);
    }
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return Err(ManagedObjectError::InvalidStructure);
        };
        let mut identifier = false;
        let mut parameters = false;
        let mut last = None;
        for child in structure.children() {
            let position = match child.tag().raw() {
                UNIQUE_IDENTIFIER => 0,
                CRYPTOGRAPHIC_PARAMETERS => 1,
                _ => continue,
            };
            if (position == 0 && identifier) || (position == 1 && parameters) {
                return Err(ManagedObjectError::DuplicateField);
            }
            if last.is_some_and(|previous| position < previous) {
                return Err(ManagedObjectError::IncorrectFieldOrder);
            }
            if position == 0 {
                if !matches!(
                    child.item_type(),
                    ItemType::TextString | ItemType::Enumeration | ItemType::Integer
                ) {
                    return Err(ManagedObjectError::IncorrectFieldType);
                }
                identifier = true;
            } else {
                if child.item_type() != ItemType::Structure {
                    return Err(ManagedObjectError::IncorrectFieldType);
                }
                parameters = true;
            }
            last = Some(position);
        }
        if identifier {
            Ok(())
        } else {
            Err(ManagedObjectError::MissingRequiredField)
        }
    })
}

fn append_cryptographic_parameters(
    item: &Item,
    information_tag: u32,
    parameters: Structure,
) -> Result<Item, ManagedObjectError> {
    let mut fields = item
        .with_value(|value| match value {
            ValueView::Structure(structure) => structure
                .children()
                .iter()
                .map(Item::try_clone)
                .collect::<Result<Vec<_>, _>>(),
            _ => Ok(Vec::new()),
        })
        .map_err(ManagedObjectError::from)?;
    fields.push(make_item(
        CRYPTOGRAPHIC_PARAMETERS,
        Value::structure(parameters),
    )?);
    structure_item(information_tag, fields)
}

fn unique_identifier_item(value: UniqueIdentifier) -> Result<Item, ManagedObjectError> {
    let value = match value {
        UniqueIdentifier::TextString(value) => Value::text_string(value),
        UniqueIdentifier::Enumeration(value) => Value::enumeration(value),
        UniqueIdentifier::Integer(value) => Value::integer(value),
    };
    make_item(UNIQUE_IDENTIFIER, value)
}

fn with_structure<R>(
    item: &Item,
    callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
) -> Result<R, ManagedObjectError> {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => Ok(callback(structure)),
        _ => Err(ManagedObjectError::InvalidStructure), // COVERAGE-EXCLUDE: callers validate or construct the expected Structure tag and type.
    })
}
