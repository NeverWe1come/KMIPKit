//! Typed KMIP 2.1 user objects and KMIP 3.0 system credential objects.
#![allow(clippy::missing_errors_doc)] // Fallible constructors share the sanitized schema error documented below.

use std::{error::Error, fmt};

use kmipkit_ttlv::{
    Item, ItemType, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView,
};

use crate::{ObjectType, SplitKeyMethod};

use crate::key_block::KeyBlock;
use crate::system_credential_objects::{
    DeviceCredentialObject, HashedPasswordCredentialObject, OneTimePasswordCredentialObject,
    PasswordCredentialObject,
};

const CERTIFICATE: u32 = 0x0042_0013;
const CERTIFICATE_REQUEST: u32 = 0x0042_0018;
const OPAQUE_OBJECT: u32 = 0x0042_005B;
const PGP_KEY: u32 = 0x0042_00BD;
const PRIVATE_KEY: u32 = 0x0042_0064;
const PUBLIC_KEY: u32 = 0x0042_006D;
const SECRET_DATA: u32 = 0x0042_0085;
const SPLIT_KEY: u32 = 0x0042_0089;
const SYMMETRIC_KEY: u32 = 0x0042_008F;

const CERTIFICATE_TYPE: u32 = 0x0042_001D;
const CERTIFICATE_VALUE: u32 = 0x0042_001E;
const CERTIFICATE_REQUEST_TYPE: u32 = 0x0042_0019;
const CERTIFICATE_REQUEST_VALUE: u32 = 0x0042_0140;
const OPAQUE_DATA_TYPE: u32 = 0x0042_0059;
const OPAQUE_DATA_VALUE: u32 = 0x0042_005A;
const PGP_KEY_VERSION: u32 = 0x0042_00BE;
const KEY_BLOCK: u32 = 0x0042_0040;
const SECRET_DATA_TYPE: u32 = 0x0042_0086;
const SPLIT_KEY_PARTS: u32 = 0x0042_008B;
const KEY_PART_IDENTIFIER: u32 = 0x0042_0044;
const SPLIT_KEY_THRESHOLD: u32 = 0x0042_008C;
const SPLIT_KEY_METHOD: u32 = 0x0042_008A;
const PRIME_FIELD_SIZE: u32 = 0x0042_0062;

/// Sanitized failure while constructing or reading a typed KMIP structure.
///
/// This error never retains a tag value, payload, key material, or raw KMIP
/// body.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManagedObjectError {
    /// The supplied Item has the wrong tag or is not a Structure.
    InvalidStructure,
    /// A source-required child is absent.
    MissingRequiredField,
    /// A singleton child appears more than once.
    DuplicateField,
    /// A known child has the wrong TTLV Item Type.
    IncorrectFieldType,
    /// Known children do not follow their source-defined order.
    IncorrectFieldOrder,
    /// A conditional field combination is invalid.
    InvalidFieldCombination,
    /// Object Type and object-specific Structure tag do not correspond.
    MismatchedObjectType,
    /// A local TTLV model limit prevents construction.
    TtlvModelLimit,
}

impl fmt::Display for ManagedObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidStructure => "managed-object structure is invalid",
            Self::MissingRequiredField => "managed-object structure is missing a required field",
            Self::DuplicateField => "managed-object structure repeats a singleton field",
            Self::IncorrectFieldType => "managed-object field has an incorrect TTLV type",
            Self::IncorrectFieldOrder => "managed-object fields are not in KMIP table order",
            Self::InvalidFieldCombination => "managed-object fields violate a KMIP condition",
            Self::MismatchedObjectType => "Object Type does not match the object structure",
            Self::TtlvModelLimit => "managed-object structure exceeds a local TTLV limit",
        })
    }
}

impl Error for ManagedObjectError {}

impl From<ModelError> for ManagedObjectError {
    fn from(_: ModelError) -> Self {
        Self::TtlvModelLimit
    }
}

pub(crate) fn checked_tag(raw: u32) -> Result<Tag, ManagedObjectError> {
    RawTag::new(raw)
        .map_err(ManagedObjectError::from)?
        .try_checked()
        .map_err(ManagedObjectError::from)
}

pub(crate) fn make_item(raw_tag: u32, value: Value) -> Result<Item, ManagedObjectError> {
    Item::new(checked_tag(raw_tag)?, value).map_err(ManagedObjectError::from)
}

pub(crate) fn make_structure_item(
    raw_tag: u32,
    fields: impl IntoIterator<Item = Item>,
) -> Result<Item, ManagedObjectError> {
    let mut structure = Structure::new();
    for field in fields {
        structure
            .try_push(field)
            .map_err(ManagedObjectError::from)?;
    }
    make_item(raw_tag, Value::structure(structure))
}

/// Replaces known children in-place and inserts new known children by schema order.
///
/// Existing unknown children retain their value, relative order, and location
/// among existing siblings. Clones use the TTLV model's zeroizing copy path.
pub(crate) fn rebuild_ordered_structure(
    source: &Item,
    ordered_tags: &[u32],
    replacements: Vec<Item>,
) -> Result<Item, ManagedObjectError> {
    let mut pending = replacements;
    let mut children = source.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return Err(ManagedObjectError::InvalidStructure);
        };
        let mut children = Vec::with_capacity(structure.children().len());
        for child in structure.children() {
            if let Some(position) = pending
                .iter()
                .position(|replacement| replacement.tag() == child.tag())
            {
                children.push(pending.remove(position));
            } else {
                children.push(child.try_clone().map_err(ManagedObjectError::from)?);
            }
        }
        Ok(children)
    })?;

    pending.sort_by_key(|item| {
        ordered_tags
            .iter()
            .position(|tag| *tag == item.tag().raw())
            .unwrap_or(usize::MAX)
    });
    for replacement in pending {
        let replacement_position = ordered_tags
            .iter()
            .position(|tag| *tag == replacement.tag().raw())
            .unwrap_or(usize::MAX);
        let insertion = children
            .iter()
            .position(|child| {
                ordered_tags
                    .iter()
                    .position(|tag| *tag == child.tag().raw())
                    .is_some_and(|position| position > replacement_position)
            })
            .unwrap_or(children.len());
        children.insert(insertion, replacement);
    }

    let mut fields = Structure::new();
    for child in children {
        fields.try_push(child)?;
    }
    make_item(source.tag().raw(), Value::structure(fields))
}

pub(crate) fn validate_structure(
    item: &Item,
    expected_tag: u32,
    schema: &[(u32, ItemType, bool)],
) -> Result<(), ManagedObjectError> {
    if item.tag().raw() != expected_tag || item.item_type() != ItemType::Structure {
        return Err(ManagedObjectError::InvalidStructure);
    }

    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return Err(ManagedObjectError::InvalidStructure); // COVERAGE-EXCLUDE: ItemType::Structure guarantees this ValueView variant.
        };
        let mut present = vec![false; schema.len()];
        let mut last_position = None;
        for field in structure.children() {
            let Some(position) = schema
                .iter()
                .position(|(tag, _, _)| *tag == field.tag().raw())
            else {
                // Unknown/vendor descendants are retained in their original position.
                continue;
            };
            let (_, expected_type, _) = schema[position];
            if field.item_type() != expected_type {
                return Err(ManagedObjectError::IncorrectFieldType);
            }
            if field.tag().raw() == KEY_BLOCK {
                KeyBlock::validate_item(field)?;
            }
            if present[position] {
                return Err(ManagedObjectError::DuplicateField);
            }
            if last_position.is_some_and(|last| position < last) {
                return Err(ManagedObjectError::IncorrectFieldOrder);
            }
            present[position] = true;
            last_position = Some(position);
        }
        if schema
            .iter()
            .enumerate()
            .any(|(index, (_, _, required))| *required && !present[index])
        {
            return Err(ManagedObjectError::MissingRequiredField);
        }
        validate_object_constraints(item, expected_tag)?;
        Ok(())
    })
}

fn validate_object_constraints(item: &Item, expected_tag: u32) -> Result<(), ManagedObjectError> {
    match expected_tag {
        SECRET_DATA => {
            let has_crypto_properties = with_item(item, KEY_BLOCK, |key_block| {
                KeyBlock::has_cryptographic_properties_item(key_block)
            })
            .ok_or(ManagedObjectError::MissingRequiredField)??;
            if has_crypto_properties {
                return Err(ManagedObjectError::InvalidFieldCombination);
            }
        }
        SPLIT_KEY => {
            let parts = integer_field(item, SPLIT_KEY_PARTS)?;
            let identifier = integer_field(item, KEY_PART_IDENTIFIER)?;
            let threshold = integer_field(item, SPLIT_KEY_THRESHOLD)?;
            let method = enumeration_field(item, SPLIT_KEY_METHOD)?;
            let has_prime_field_size = with_field(item, PRIME_FIELD_SIZE, |_| ()).is_some();
            if parts < 1
                || identifier < 1
                || identifier > parts
                || threshold < 2
                || threshold > parts
                || (method == 3 && !has_prime_field_size)
            {
                return Err(ManagedObjectError::InvalidFieldCombination);
            }
        }
        _ => {}
    }
    Ok(())
}

fn with_item<R>(
    item: &Item,
    field_tag: u32,
    callback: impl for<'a> FnOnce(&'a Item) -> R,
) -> Option<R> {
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return None; // COVERAGE-EXCLUDE: callers pass only typed, validated managed-object Structures.
        };
        structure
            .children()
            .iter()
            .find(|child| child.tag().raw() == field_tag)
            .map(callback)
    })
}

fn integer_field(item: &Item, tag: u32) -> Result<i32, ManagedObjectError> {
    with_field(item, tag, |value| match value {
        ValueView::Integer(raw) => Some(*raw),
        _ => None, // COVERAGE-EXCLUDE: validated schema requires ItemType::Integer for this field.
    })
    .flatten()
    .ok_or(ManagedObjectError::IncorrectFieldType)
}

fn enumeration_field(item: &Item, tag: u32) -> Result<u32, ManagedObjectError> {
    with_field(item, tag, |value| match value {
        ValueView::Enumeration(raw) => Some(*raw),
        _ => None, // COVERAGE-EXCLUDE: validated schema requires ItemType::Enumeration for this field.
    })
    .flatten()
    .ok_or(ManagedObjectError::IncorrectFieldType)
}

pub(crate) fn validate_any_structure(item: &Item) -> Result<(), ManagedObjectError> {
    if item.item_type() == ItemType::Structure {
        Ok(())
    } else {
        Err(ManagedObjectError::InvalidStructure)
    }
}

pub(crate) fn with_structure<R>(
    item: &Item,
    callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
) -> Result<R, ManagedObjectError> {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => Ok(callback(structure)),
        _ => Err(ManagedObjectError::InvalidStructure), // COVERAGE-EXCLUDE: typed callers retain only validated Structure Items.
    })
}

pub(crate) fn with_field<R>(
    item: &Item,
    field_tag: u32,
    callback: impl for<'a> FnOnce(ValueView<'a>) -> R,
) -> Option<R> {
    item.with_value(|value| {
        let ValueView::Structure(structure) = value else {
            return None; // COVERAGE-EXCLUDE: typed callers retain only validated Structure Items.
        };
        structure
            .children()
            .iter()
            .find(|field| field.tag().raw() == field_tag)
            .map(|field| field.with_value(callback))
    })
}

macro_rules! object_wrapper {
    ($name:ident, $tag:expr, $schema:expr) => {
        /// A typed KMIP 2.1 object-specific Structure Item.
        pub struct $name {
            item: Item,
        }

        impl $name {
            /// Reads a generic object Item while preserving unknown children and order.
            ///
            /// # Errors
            /// Returns a sanitized error when the tag, known fields, Item Types,
            /// cardinality, or field order do not match the source table.
            pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
                validate_structure(&item, $tag, $schema)?;
                Ok(Self { item })
            }

            /// Consumes this typed object into its exact object-specific TTLV Item.
            #[must_use]
            pub fn into_item(self) -> Item {
                self.item
            }

            /// Lends the source-ordered object Structure for callback-scoped inspection.
            pub fn with_raw<R>(
                &self,
                callback: impl for<'a> FnOnce(StructureView<'a>) -> R,
            ) -> Result<R, ManagedObjectError> {
                with_structure(&self.item, callback)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("item", &self.item)
                    .finish()
            }
        }
    };
}

const CERTIFICATE_SCHEMA: &[(u32, ItemType, bool)] = &[
    (CERTIFICATE_TYPE, ItemType::Enumeration, true),
    (CERTIFICATE_VALUE, ItemType::ByteString, true),
];
const CERTIFICATE_REQUEST_SCHEMA: &[(u32, ItemType, bool)] = &[
    (CERTIFICATE_REQUEST_TYPE, ItemType::Enumeration, true),
    (CERTIFICATE_REQUEST_VALUE, ItemType::ByteString, true),
];
const OPAQUE_OBJECT_SCHEMA: &[(u32, ItemType, bool)] = &[
    (OPAQUE_DATA_TYPE, ItemType::Enumeration, true),
    (OPAQUE_DATA_VALUE, ItemType::ByteString, true),
];
const PGP_KEY_SCHEMA: &[(u32, ItemType, bool)] = &[
    (PGP_KEY_VERSION, ItemType::Integer, true),
    (KEY_BLOCK, ItemType::Structure, true),
];
const KEYED_OBJECT_SCHEMA: &[(u32, ItemType, bool)] = &[(KEY_BLOCK, ItemType::Structure, true)];
const SECRET_DATA_SCHEMA: &[(u32, ItemType, bool)] = &[
    (SECRET_DATA_TYPE, ItemType::Enumeration, true),
    (KEY_BLOCK, ItemType::Structure, true),
];
const SPLIT_KEY_SCHEMA: &[(u32, ItemType, bool)] = &[
    (SPLIT_KEY_PARTS, ItemType::Integer, true),
    (KEY_PART_IDENTIFIER, ItemType::Integer, true),
    (SPLIT_KEY_THRESHOLD, ItemType::Integer, true),
    (SPLIT_KEY_METHOD, ItemType::Enumeration, true),
    (PRIME_FIELD_SIZE, ItemType::BigInteger, false),
    (KEY_BLOCK, ItemType::Structure, true),
];

object_wrapper!(Certificate, CERTIFICATE, CERTIFICATE_SCHEMA);
object_wrapper!(
    CertificateRequest,
    CERTIFICATE_REQUEST,
    CERTIFICATE_REQUEST_SCHEMA
);
object_wrapper!(OpaqueObject, OPAQUE_OBJECT, OPAQUE_OBJECT_SCHEMA);
object_wrapper!(PgpKey, PGP_KEY, PGP_KEY_SCHEMA);
object_wrapper!(PrivateKey, PRIVATE_KEY, KEYED_OBJECT_SCHEMA);
object_wrapper!(PublicKey, PUBLIC_KEY, KEYED_OBJECT_SCHEMA);
object_wrapper!(SecretData, SECRET_DATA, SECRET_DATA_SCHEMA);
object_wrapper!(SplitKey, SPLIT_KEY, SPLIT_KEY_SCHEMA);
object_wrapper!(SymmetricKey, SYMMETRIC_KEY, KEYED_OBJECT_SCHEMA);

impl Certificate {
    /// Constructs the Table 2 Certificate fields in source order.
    pub fn try_new(
        certificate_type: u32,
        certificate_value: Vec<u8>,
    ) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(make_structure_item(
            CERTIFICATE,
            [
                make_item(CERTIFICATE_TYPE, Value::enumeration(certificate_type))?,
                make_item(CERTIFICATE_VALUE, Value::byte_string(certificate_value))?,
            ],
        )?)
    }

    /// Returns the exact raw Certificate Type Enumeration value.
    #[must_use]
    pub fn certificate_type(&self) -> Option<u32> {
        with_field(&self.item, CERTIFICATE_TYPE, |value| match value {
            ValueView::Enumeration(raw) => Some(*raw),
            _ => None, // COVERAGE-EXCLUDE: Certificate validation fixes this field as Enumeration.
        })
        .flatten()
    }

    /// Lends the Certificate Value bytes without copying them.
    pub fn with_certificate_value<R>(
        &self,
        callback: impl for<'a> FnOnce(&'a [u8]) -> R,
    ) -> Option<R> {
        with_field(&self.item, CERTIFICATE_VALUE, |value| match value {
            ValueView::ByteString(bytes) => Some(callback(bytes)),
            _ => None, // COVERAGE-EXCLUDE: Certificate validation fixes this field as ByteString.
        })
        .flatten()
    }
}

impl CertificateRequest {
    /// Constructs the Table 3 Certificate Request fields in source order.
    pub fn try_new(request_type: u32, request_value: Vec<u8>) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(make_structure_item(
            CERTIFICATE_REQUEST,
            [
                make_item(CERTIFICATE_REQUEST_TYPE, Value::enumeration(request_type))?,
                make_item(CERTIFICATE_REQUEST_VALUE, Value::byte_string(request_value))?,
            ],
        )?)
    }

    /// Returns the exact raw Certificate Request Type value.
    #[must_use]
    pub fn request_type(&self) -> Option<u32> {
        with_field(&self.item, CERTIFICATE_REQUEST_TYPE, |value| match value {
            ValueView::Enumeration(raw) => Some(*raw),
            _ => None, // COVERAGE-EXCLUDE: CertificateRequest validation fixes this field as Enumeration.
        })
        .flatten()
    }
}

impl OpaqueObject {
    /// Constructs the Table 4 Opaque Object fields in source order.
    pub fn try_new(data_type: u32, data_value: Vec<u8>) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(make_structure_item(
            OPAQUE_OBJECT,
            [
                make_item(OPAQUE_DATA_TYPE, Value::enumeration(data_type))?,
                make_item(OPAQUE_DATA_VALUE, Value::byte_string(data_value))?,
            ],
        )?)
    }

    /// Returns the exact raw Opaque Data Type value.
    #[must_use]
    pub fn opaque_data_type(&self) -> Option<u32> {
        with_field(&self.item, OPAQUE_DATA_TYPE, |value| match value {
            ValueView::Enumeration(raw) => Some(*raw),
            _ => None, // COVERAGE-EXCLUDE: OpaqueObject validation fixes this field as Enumeration.
        })
        .flatten()
    }
}

impl PgpKey {
    /// Constructs the Table 5 PGP Key fields in source order.
    pub fn try_new(version: i32, key_block: KeyBlock) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(make_structure_item(
            PGP_KEY,
            [
                make_item(PGP_KEY_VERSION, Value::integer(version))?,
                key_block.try_into_item()?,
            ],
        )?)
    }

    /// Returns the exact PGP Key Version Integer.
    #[must_use]
    pub fn pgp_key_version(&self) -> Option<i32> {
        with_field(&self.item, PGP_KEY_VERSION, |value| match value {
            ValueView::Integer(value) => Some(*value),
            _ => None, // COVERAGE-EXCLUDE: PgpKey validation fixes this field as Integer.
        })
        .flatten()
    }
}

impl PrivateKey {
    /// Constructs a Table 6 Private Key from its required Key Block.
    pub fn try_new(key_block: KeyBlock) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(make_structure_item(
            PRIVATE_KEY,
            [key_block.try_into_item()?],
        )?)
    }
}

impl PublicKey {
    /// Constructs a Table 7 Public Key from its required Key Block.
    pub fn try_new(key_block: KeyBlock) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(make_structure_item(
            PUBLIC_KEY,
            [key_block.try_into_item()?],
        )?)
    }
}

impl SecretData {
    /// Constructs the Table 8 Secret Data fields in source order.
    pub fn try_new(data_type: u32, key_block: KeyBlock) -> Result<Self, ManagedObjectError> {
        if key_block.has_cryptographic_properties() {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        Self::try_from_item(make_structure_item(
            SECRET_DATA,
            [
                make_item(SECRET_DATA_TYPE, Value::enumeration(data_type))?,
                key_block.try_into_item()?,
            ],
        )?)
    }

    /// Returns the exact Secret Data Type Enumeration value.
    #[must_use]
    pub fn secret_data_type(&self) -> Option<u32> {
        with_field(&self.item, SECRET_DATA_TYPE, |value| match value {
            ValueView::Enumeration(raw) => Some(*raw),
            _ => None, // COVERAGE-EXCLUDE: SecretData validation fixes this field as Enumeration.
        })
        .flatten()
    }
}

impl SplitKey {
    /// Constructs Table 9 fields in OASIS order, with Key Block last.
    pub fn try_new(
        key_block: KeyBlock,
        key_part_identifier: i32,
        split_key_method: SplitKeyMethod,
        split_key_parts: i32,
        split_key_threshold: i32,
        prime_field_size: Option<Vec<u8>>,
    ) -> Result<Self, ManagedObjectError> {
        if key_part_identifier < 1
            || split_key_parts < 1
            || key_part_identifier > split_key_parts
            || split_key_threshold < 2
            || split_key_threshold > split_key_parts
            || (split_key_method.raw() == 3 && prime_field_size.is_none())
        {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        let mut fields = vec![
            make_item(SPLIT_KEY_PARTS, Value::integer(split_key_parts))?,
            make_item(KEY_PART_IDENTIFIER, Value::integer(key_part_identifier))?,
            make_item(SPLIT_KEY_THRESHOLD, Value::integer(split_key_threshold))?,
            make_item(SPLIT_KEY_METHOD, Value::enumeration(split_key_method.raw()))?,
        ];
        if let Some(size) = prime_field_size {
            fields.push(make_item(PRIME_FIELD_SIZE, Value::big_integer(size))?);
        }
        fields.push(key_block.try_into_item()?);
        Self::try_from_item(make_structure_item(SPLIT_KEY, fields)?)
    }
}

impl SymmetricKey {
    /// Constructs a Table 10 Symmetric Key from its required Key Block.
    pub fn try_new(key_block: KeyBlock) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(make_structure_item(
            SYMMETRIC_KEY,
            [key_block.try_into_item()?],
        )?)
    }
}

/// A known KMIP object paired with its Object Type, or a lossless unknown fallback.
pub enum ManagedObject {
    /// Certificate object (Object Type 1).
    Certificate(Certificate),
    /// Certificate Request object (Object Type 10).
    CertificateRequest(CertificateRequest),
    /// Password Credential system object (KMIP 3.0 Object Type 13).
    PasswordCredential(PasswordCredentialObject),
    /// Device Credential system object (KMIP 3.0 Object Type 14).
    DeviceCredential(DeviceCredentialObject),
    /// One Time Password Credential system object (KMIP 3.0 Object Type 15).
    OneTimePasswordCredential(OneTimePasswordCredentialObject),
    /// Hashed Password Credential system object (KMIP 3.0 Object Type 16).
    HashedPasswordCredential(HashedPasswordCredentialObject),
    /// Opaque Object (Object Type 8).
    OpaqueObject(OpaqueObject),
    /// PGP Key object (Object Type 9).
    PgpKey(PgpKey),
    /// Private Key object (Object Type 4).
    PrivateKey(PrivateKey),
    /// Public Key object (Object Type 3).
    PublicKey(PublicKey),
    /// Secret Data object (Object Type 7).
    SecretData(SecretData),
    /// Split Key object (Object Type 5).
    SplitKey(SplitKey),
    /// Symmetric Key object (Object Type 2).
    SymmetricKey(SymmetricKey),
    /// Future/vendor object retained in the generic TTLV model.
    Unknown { object_type: ObjectType, item: Item },
}

impl ManagedObject {
    /// Associates an Object Type with its source-defined object Structure tag.
    ///
    /// Unknown values remain generic and preserve their child order and tags.
    ///
    /// # Errors
    /// Returns a sanitized error for a mismatch or malformed known structure.
    pub fn try_from_item(object_type: ObjectType, item: Item) -> Result<Self, ManagedObjectError> {
        match object_type.raw() {
            1 => Certificate::try_from_item(item).map(Self::Certificate),
            2 => SymmetricKey::try_from_item(item).map(Self::SymmetricKey),
            3 => PublicKey::try_from_item(item).map(Self::PublicKey),
            4 => PrivateKey::try_from_item(item).map(Self::PrivateKey),
            5 => SplitKey::try_from_item(item).map(Self::SplitKey),
            7 => SecretData::try_from_item(item).map(Self::SecretData),
            8 => OpaqueObject::try_from_item(item).map(Self::OpaqueObject),
            9 => PgpKey::try_from_item(item).map(Self::PgpKey),
            10 => CertificateRequest::try_from_item(item).map(Self::CertificateRequest),
            13 => PasswordCredentialObject::try_from_item(item).map(Self::PasswordCredential),
            14 => DeviceCredentialObject::try_from_item(item).map(Self::DeviceCredential),
            15 => OneTimePasswordCredentialObject::try_from_item(item)
                .map(Self::OneTimePasswordCredential),
            16 => HashedPasswordCredentialObject::try_from_item(item)
                .map(Self::HashedPasswordCredential),
            _ => {
                validate_any_structure(&item)?;
                if known_object_tag(item.tag().raw()) {
                    return Err(ManagedObjectError::MismatchedObjectType);
                }
                Ok(Self::Unknown { object_type, item })
            }
        }
    }

    /// Creates a generic fallback for an unknown Object Type.
    ///
    /// # Errors
    /// Returns an error if the Object Type or Item tag belongs to a known object.
    pub fn unknown(object_type: ObjectType, item: Item) -> Result<Self, ManagedObjectError> {
        if matches!(
            object_type.raw(),
            1 | 2 | 3 | 4 | 5 | 7 | 8 | 9 | 10 | 13 | 14 | 15 | 16
        ) || known_object_tag(item.tag().raw())
        {
            return Err(ManagedObjectError::MismatchedObjectType);
        }
        validate_any_structure(&item)?;
        Ok(Self::Unknown { object_type, item })
    }

    /// Returns the paired Object Type, preserving unknown raw values.
    #[must_use]
    pub const fn object_type(&self) -> ObjectType {
        match self {
            Self::Certificate(_) => ObjectType::from_raw(1),
            Self::SymmetricKey(_) => ObjectType::from_raw(2),
            Self::PublicKey(_) => ObjectType::from_raw(3),
            Self::PrivateKey(_) => ObjectType::from_raw(4),
            Self::SplitKey(_) => ObjectType::from_raw(5),
            Self::SecretData(_) => ObjectType::from_raw(7),
            Self::OpaqueObject(_) => ObjectType::from_raw(8),
            Self::PgpKey(_) => ObjectType::from_raw(9),
            Self::CertificateRequest(_) => ObjectType::from_raw(10),
            Self::PasswordCredential(_) => ObjectType::PASSWORD_CREDENTIAL,
            Self::DeviceCredential(_) => ObjectType::DEVICE_CREDENTIAL,
            Self::OneTimePasswordCredential(_) => ObjectType::ONE_TIME_PASSWORD_CREDENTIAL,
            Self::HashedPasswordCredential(_) => ObjectType::HASHED_PASSWORD_CREDENTIAL,
            Self::Unknown { object_type, .. } => *object_type,
        }
    }

    /// Consumes the wrapper into the direct object-specific TTLV Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        match self {
            Self::Certificate(value) => value.into_item(),
            Self::CertificateRequest(value) => value.into_item(),
            Self::PasswordCredential(value) => value.into_item(),
            Self::DeviceCredential(value) => value.into_item(),
            Self::OneTimePasswordCredential(value) => value.into_item(),
            Self::HashedPasswordCredential(value) => value.into_item(),
            Self::OpaqueObject(value) => value.into_item(),
            Self::PgpKey(value) => value.into_item(),
            Self::PrivateKey(value) => value.into_item(),
            Self::PublicKey(value) => value.into_item(),
            Self::SecretData(value) => value.into_item(),
            Self::SplitKey(value) => value.into_item(),
            Self::SymmetricKey(value) => value.into_item(),
            Self::Unknown { item, .. } => item,
        }
    }
}

impl fmt::Debug for ManagedObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManagedObject")
            .field("object_type", &self.object_type())
            .field("item_tag", &self.item_tag())
            .finish()
    }
}

impl ManagedObject {
    fn item_tag(&self) -> u32 {
        match self {
            Self::Certificate(value) => value.item.tag().raw(),
            Self::CertificateRequest(value) => value.item.tag().raw(),
            Self::PasswordCredential(value) => value.with_item(|item| item.tag().raw()),
            Self::DeviceCredential(value) => value.with_item(|item| item.tag().raw()),
            Self::OneTimePasswordCredential(value) => value.with_item(|item| item.tag().raw()),
            Self::HashedPasswordCredential(value) => value.with_item(|item| item.tag().raw()),
            Self::OpaqueObject(value) => value.item.tag().raw(),
            Self::PgpKey(value) => value.item.tag().raw(),
            Self::PrivateKey(value) => value.item.tag().raw(),
            Self::PublicKey(value) => value.item.tag().raw(),
            Self::SecretData(value) => value.item.tag().raw(),
            Self::SplitKey(value) => value.item.tag().raw(),
            Self::SymmetricKey(value) => value.item.tag().raw(),
            Self::Unknown { item, .. } => item.tag().raw(),
        }
    }
}

fn known_object_tag(tag: u32) -> bool {
    matches!(
        tag,
        CERTIFICATE
            | CERTIFICATE_REQUEST
            | 0x0042_01A1
            | 0x0042_01A6
            | 0x0042_01A7
            | 0x0042_01AF
            | OPAQUE_OBJECT
            | PGP_KEY
            | PRIVATE_KEY
            | PUBLIC_KEY
            | SECRET_DATA
            | SPLIT_KEY
            | SYMMETRIC_KEY
    )
}
