//! Raw-preserving KMIP 3.0 system credential object structures.
#![allow(clippy::missing_errors_doc)] // Errors are sanitized categories shared with managed-object schemas.

use std::fmt;

use kmipkit_ttlv::{Item, ItemType, Value, ValueView};

use crate::{
    ManagedObjectError, SecretBytes, SecretText,
    managed_object::{make_item, make_structure_item, validate_structure},
};

const PASSWORD_CREDENTIAL: u32 = 0x0042_01A1;
const DEVICE_CREDENTIAL: u32 = 0x0042_01A6;
const OTP_CREDENTIAL: u32 = 0x0042_01A7;
const HASHED_PASSWORD_CREDENTIAL: u32 = 0x0042_01AF;

const PASSWORD: u32 = 0x0042_00A1;
const PASSWORD_SALT: u32 = 0x0042_01A2;
const PASSWORD_SALT_ALGORITHM: u32 = 0x0042_01A3;
const SALTED_PASSWORD: u32 = 0x0042_01A4;
const ITERATION_COUNT: u32 = 0x0042_003C;
const DEVICE_SERIAL_NUMBER: u32 = 0x0042_00B0;
const DEVICE_IDENTIFIER: u32 = 0x0042_00A2;
const NETWORK_IDENTIFIER: u32 = 0x0042_00AB;
const MACHINE_IDENTIFIER: u32 = 0x0042_00A9;
const MEDIA_IDENTIFIER: u32 = 0x0042_00AA;
const OTP_ALGORITHM: u32 = 0x0042_01A8;
const OTP_DIGEST: u32 = 0x0042_01A9;
const OTP_SERIAL: u32 = 0x0042_01AA;
const OTP_SEED: u32 = 0x0042_01AB;
const OTP_INTERVAL: u32 = 0x0042_01AC;
const OTP_DIGITS: u32 = 0x0042_01AD;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const HASHED_USERNAME_PASSWORD: u32 = 0x0042_01B0;
const HASHED_PASSWORD_USERNAME: u32 = 0x0042_01B1;
const DEFAULT_HASHING_ALGORITHM_SHA256: u32 = 6;

const PASSWORD_FIELDS: &[(u32, ItemType, bool)] = &[
    (PASSWORD, ItemType::TextString, false),
    (PASSWORD_SALT, ItemType::ByteString, false),
    (PASSWORD_SALT_ALGORITHM, ItemType::Enumeration, false),
    (SALTED_PASSWORD, ItemType::ByteString, false),
    (ITERATION_COUNT, ItemType::Integer, false),
];
const DEVICE_FIELDS: &[(u32, ItemType, bool)] = &[
    (DEVICE_SERIAL_NUMBER, ItemType::TextString, false),
    (DEVICE_IDENTIFIER, ItemType::TextString, false),
    (NETWORK_IDENTIFIER, ItemType::TextString, false),
    (MACHINE_IDENTIFIER, ItemType::TextString, false),
    (MEDIA_IDENTIFIER, ItemType::TextString, false),
];
const OTP_FIELDS: &[(u32, ItemType, bool)] = &[
    (OTP_ALGORITHM, ItemType::Enumeration, true),
    (OTP_DIGEST, ItemType::Enumeration, false),
    (OTP_SERIAL, ItemType::TextString, false),
    (OTP_SEED, ItemType::ByteString, false),
    (OTP_INTERVAL, ItemType::Interval, false),
    (OTP_DIGITS, ItemType::Integer, false),
];
const HASHED_PASSWORD_FIELDS: &[(u32, ItemType, bool)] = &[
    (HASHING_ALGORITHM, ItemType::Enumeration, false),
    (HASHED_USERNAME_PASSWORD, ItemType::ByteString, true),
    (HASHED_PASSWORD_USERNAME, ItemType::ByteString, true),
];

/// Input fields for a KMIP 3.0 Password Credential system object.
#[derive(Debug, Default)]
pub struct PasswordCredentialObjectFields {
    /// Plain password material, when this representation is used.
    pub password: Option<SecretText>,
    /// Optional Password Salt.
    pub password_salt: Option<SecretBytes>,
    /// Raw Cryptographic Algorithm enumeration for Password Salt Algorithm.
    pub password_salt_algorithm: Option<u32>,
    /// Precomputed Salted Password bytes.
    pub salted_password: Option<SecretBytes>,
    /// Optional iteration count. KMIP 3.0 defaults an absent value to 100.
    pub iteration_count: Option<i32>,
}

/// Typed, lossless KMIP 3.0 Password Credential system object.
pub struct PasswordCredentialObject {
    item: Item,
}

impl PasswordCredentialObject {
    /// Constructs the Password Credential Structure in Table 5 field order.
    ///
    /// Password Link is an Attribute and is not represented by these fields.
    /// The choice between that attribute and an in-structure password
    /// representation requires the enclosing Attribute Set and is validated by
    /// the client request validator.
    ///
    /// # Errors
    /// Returns a payload-free error for an invalid salted-password combination
    /// or a local TTLV model limit.
    pub fn try_new(fields: PasswordCredentialObjectFields) -> Result<Self, ManagedObjectError> {
        let mut children = Vec::new();
        if let Some(value) = fields.password {
            children.push(make_item(PASSWORD, value.into_ttlv_value())?);
        }
        if let Some(value) = fields.password_salt {
            children.push(make_item(PASSWORD_SALT, value.into_ttlv_value())?);
        }
        if let Some(value) = fields.password_salt_algorithm {
            children.push(make_item(
                PASSWORD_SALT_ALGORITHM,
                Value::enumeration(value),
            )?);
        }
        if let Some(value) = fields.salted_password {
            children.push(make_item(SALTED_PASSWORD, value.into_ttlv_value())?);
        }
        if let Some(value) = fields.iteration_count {
            children.push(make_item(ITERATION_COUNT, Value::integer(value))?);
        }
        Self::try_from_item(make_structure_item(PASSWORD_CREDENTIAL, children)?)
    }

    /// Validates and takes ownership of a Password Credential Structure.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        validate_structure(&item, PASSWORD_CREDENTIAL, PASSWORD_FIELDS)?;
        let has_password = has_field(&item, PASSWORD);
        let has_salted_password = has_field(&item, SALTED_PASSWORD);
        if has_salted_password && (has_password || !has_field(&item, PASSWORD_SALT_ALGORITHM)) {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        Ok(Self { item })
    }

    /// Lends the tagged Structure Item for callback-scoped generic TTLV access.
    pub fn with_item<R>(&self, callback: impl for<'a> FnOnce(&'a Item) -> R) -> R {
        callback(&self.item)
    }

    /// Lends the password text, if present, without making a caller-owned copy.
    pub fn with_password<R>(&self, callback: impl for<'a> FnOnce(Option<&'a str>) -> R) -> R {
        with_text_field(&self.item, PASSWORD, callback)
    }

    /// Lends the Password Salt bytes, if present.
    pub fn with_password_salt<R>(&self, callback: impl for<'a> FnOnce(Option<&'a [u8]>) -> R) -> R {
        with_bytes_field(&self.item, PASSWORD_SALT, callback)
    }

    /// Returns the raw Password Salt Algorithm enumeration, if present.
    #[must_use]
    pub fn password_salt_algorithm_raw(&self) -> Option<u32> {
        enum_field(&self.item, PASSWORD_SALT_ALGORITHM)
    }

    /// Lends the Salted Password bytes, if present.
    pub fn with_salted_password<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a [u8]>) -> R,
    ) -> R {
        with_bytes_field(&self.item, SALTED_PASSWORD, callback)
    }

    /// Returns the explicitly encoded Iteration Count, if present.
    #[must_use]
    pub fn iteration_count(&self) -> Option<i32> {
        integer_field(&self.item, ITERATION_COUNT)
    }

    /// Returns the source-defined effective count without inserting a field.
    #[must_use]
    pub fn effective_iteration_count(&self) -> i32 {
        self.iteration_count().unwrap_or(100)
    }

    /// Consumes this model into its object-specific Structure Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }
}

/// Input fields for a KMIP 3.0 Device Credential system object.
#[derive(Debug, Default)]
pub struct DeviceCredentialObjectFields {
    /// Optional device serial number.
    pub device_serial_number: Option<SecretText>,
    /// Optional device identifier.
    pub device_identifier: Option<SecretText>,
    /// Optional network identifier.
    pub network_identifier: Option<SecretText>,
    /// Optional machine identifier.
    pub machine_identifier: Option<SecretText>,
    /// Optional media identifier.
    pub media_identifier: Option<SecretText>,
}

/// Typed, lossless KMIP 3.0 Device Credential system object.
pub struct DeviceCredentialObject {
    item: Item,
}

impl DeviceCredentialObject {
    /// Constructs the Device Credential Structure in Table 7 field order.
    ///
    /// # Errors
    /// Returns a payload-free error if no identifier is supplied or a local
    /// TTLV model limit is exceeded.
    pub fn try_new(fields: DeviceCredentialObjectFields) -> Result<Self, ManagedObjectError> {
        let mut children = Vec::new();
        push_text(
            &mut children,
            DEVICE_SERIAL_NUMBER,
            fields.device_serial_number,
        )?;
        push_text(&mut children, DEVICE_IDENTIFIER, fields.device_identifier)?;
        push_text(&mut children, NETWORK_IDENTIFIER, fields.network_identifier)?;
        push_text(&mut children, MACHINE_IDENTIFIER, fields.machine_identifier)?;
        push_text(&mut children, MEDIA_IDENTIFIER, fields.media_identifier)?;
        Self::try_from_item(make_structure_item(DEVICE_CREDENTIAL, children)?)
    }

    /// Validates and takes ownership of a Device Credential Structure.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        validate_structure(&item, DEVICE_CREDENTIAL, DEVICE_FIELDS)?;
        if !DEVICE_FIELDS
            .iter()
            .any(|(tag, _, _)| has_field(&item, *tag))
        {
            return Err(ManagedObjectError::MissingRequiredField);
        }
        Ok(Self { item })
    }

    /// Lends the tagged Structure Item for callback-scoped generic TTLV access.
    pub fn with_item<R>(&self, callback: impl for<'a> FnOnce(&'a Item) -> R) -> R {
        callback(&self.item)
    }

    /// Lends the optional device serial number.
    pub fn with_device_serial_number<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a str>) -> R,
    ) -> R {
        with_text_field(&self.item, DEVICE_SERIAL_NUMBER, callback)
    }

    /// Lends the optional device identifier.
    pub fn with_device_identifier<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a str>) -> R,
    ) -> R {
        with_text_field(&self.item, DEVICE_IDENTIFIER, callback)
    }

    /// Lends the optional network identifier.
    pub fn with_network_identifier<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a str>) -> R,
    ) -> R {
        with_text_field(&self.item, NETWORK_IDENTIFIER, callback)
    }

    /// Lends the optional machine identifier.
    pub fn with_machine_identifier<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a str>) -> R,
    ) -> R {
        with_text_field(&self.item, MACHINE_IDENTIFIER, callback)
    }

    /// Lends the optional media identifier.
    pub fn with_media_identifier<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a str>) -> R,
    ) -> R {
        with_text_field(&self.item, MEDIA_IDENTIFIER, callback)
    }

    /// Consumes this model into its object-specific Structure Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }
}

/// Input fields for a KMIP 3.0 One Time Password Credential system object.
#[derive(Debug, Default)]
pub struct OneTimePasswordCredentialObjectFields {
    /// Required raw OTP Algorithm enumeration.
    pub otp_algorithm: Option<u32>,
    /// Optional raw Cryptographic Algorithm enumeration.
    pub otp_digest: Option<u32>,
    /// Optional OTP serial text.
    pub otp_serial: Option<SecretText>,
    /// Optional OTP seed bytes.
    pub otp_seed: Option<SecretBytes>,
    /// Optional OTP interval.
    pub otp_interval: Option<u32>,
    /// Optional number of OTP digits.
    pub otp_digits: Option<i32>,
}

/// Typed, lossless KMIP 3.0 One Time Password Credential system object.
pub struct OneTimePasswordCredentialObject {
    item: Item,
}

impl OneTimePasswordCredentialObject {
    /// Constructs the OTP Credential Structure in Table 9 field order.
    pub fn try_new(
        fields: OneTimePasswordCredentialObjectFields,
    ) -> Result<Self, ManagedObjectError> {
        let mut children = Vec::new();
        if let Some(value) = fields.otp_algorithm {
            children.push(make_item(OTP_ALGORITHM, Value::enumeration(value))?);
        }
        if let Some(value) = fields.otp_digest {
            children.push(make_item(OTP_DIGEST, Value::enumeration(value))?);
        }
        push_text(&mut children, OTP_SERIAL, fields.otp_serial)?;
        if let Some(value) = fields.otp_seed {
            children.push(make_item(OTP_SEED, value.into_ttlv_value())?);
        }
        if let Some(value) = fields.otp_interval {
            children.push(make_item(OTP_INTERVAL, Value::interval(value))?);
        }
        if let Some(value) = fields.otp_digits {
            children.push(make_item(OTP_DIGITS, Value::integer(value))?);
        }
        Self::try_from_item(make_structure_item(OTP_CREDENTIAL, children)?)
    }

    /// Validates and takes ownership of an OTP Credential Structure.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        validate_structure(&item, OTP_CREDENTIAL, OTP_FIELDS)?;
        Ok(Self { item })
    }

    /// Lends the tagged Structure Item for callback-scoped generic TTLV access.
    pub fn with_item<R>(&self, callback: impl for<'a> FnOnce(&'a Item) -> R) -> R {
        callback(&self.item)
    }

    /// Returns the required raw OTP Algorithm enumeration.
    #[must_use]
    pub fn otp_algorithm_raw(&self) -> Option<u32> {
        enum_field(&self.item, OTP_ALGORITHM)
    }

    /// Returns the optional raw OTP Digest enumeration.
    #[must_use]
    pub fn otp_digest_raw(&self) -> Option<u32> {
        enum_field(&self.item, OTP_DIGEST)
    }

    /// Lends the optional OTP serial string.
    pub fn with_otp_serial<R>(&self, callback: impl for<'a> FnOnce(Option<&'a str>) -> R) -> R {
        with_text_field(&self.item, OTP_SERIAL, callback)
    }

    /// Lends the optional OTP seed bytes.
    pub fn with_otp_seed<R>(&self, callback: impl for<'a> FnOnce(Option<&'a [u8]>) -> R) -> R {
        with_bytes_field(&self.item, OTP_SEED, callback)
    }

    /// Returns the optional OTP interval.
    #[must_use]
    pub fn otp_interval(&self) -> Option<u32> {
        interval_field(&self.item, OTP_INTERVAL)
    }

    /// Returns the optional number of OTP digits.
    #[must_use]
    pub fn otp_digits(&self) -> Option<i32> {
        integer_field(&self.item, OTP_DIGITS)
    }

    /// Consumes this model into its object-specific Structure Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }
}

/// Input fields for a KMIP 3.0 Hashed Password Credential system object.
#[derive(Debug, Default)]
pub struct HashedPasswordCredentialObjectFields {
    /// Optional raw Hashing Algorithm enumeration; the source-defined default
    /// is retained as absence rather than materialized locally.
    pub hashing_algorithm: Option<u32>,
    /// Required Hash Username Password bytes.
    pub hashed_username_password: Option<SecretBytes>,
    /// Required Hash Password Username bytes.
    pub hashed_password_username: Option<SecretBytes>,
}

/// Typed, lossless KMIP 3.0 Hashed Password Credential system object.
pub struct HashedPasswordCredentialObject {
    item: Item,
}

impl HashedPasswordCredentialObject {
    /// Constructs the Hashed Password Structure in Table 11 field order.
    pub fn try_new(
        fields: HashedPasswordCredentialObjectFields,
    ) -> Result<Self, ManagedObjectError> {
        let mut children = Vec::new();
        if let Some(value) = fields.hashing_algorithm {
            children.push(make_item(HASHING_ALGORITHM, Value::enumeration(value))?);
        }
        if let Some(value) = fields.hashed_username_password {
            children.push(make_item(
                HASHED_USERNAME_PASSWORD,
                value.into_ttlv_value(),
            )?);
        }
        if let Some(value) = fields.hashed_password_username {
            children.push(make_item(
                HASHED_PASSWORD_USERNAME,
                value.into_ttlv_value(),
            )?);
        }
        Self::try_from_item(make_structure_item(HASHED_PASSWORD_CREDENTIAL, children)?)
    }

    /// Validates and takes ownership of a Hashed Password Credential Structure.
    pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
        validate_structure(&item, HASHED_PASSWORD_CREDENTIAL, HASHED_PASSWORD_FIELDS)?;
        Ok(Self { item })
    }

    /// Lends the tagged Structure Item for callback-scoped generic TTLV access.
    pub fn with_item<R>(&self, callback: impl for<'a> FnOnce(&'a Item) -> R) -> R {
        callback(&self.item)
    }

    /// Returns the optional raw Hashing Algorithm Enumeration.
    #[must_use]
    pub fn hashing_algorithm_raw(&self) -> Option<u32> {
        enum_field(&self.item, HASHING_ALGORITHM)
    }

    /// Returns the effective algorithm without materializing an absent field.
    #[must_use]
    pub fn effective_hashing_algorithm_raw(&self) -> u32 {
        self.hashing_algorithm_raw()
            .unwrap_or(DEFAULT_HASHING_ALGORITHM_SHA256)
    }

    /// Lends the required Hash Username Password bytes.
    pub fn with_hashed_username_password<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a [u8]>) -> R,
    ) -> R {
        with_bytes_field(&self.item, HASHED_USERNAME_PASSWORD, callback)
    }

    /// Lends the required Hash Password Username bytes.
    pub fn with_hashed_password_username<R>(
        &self,
        callback: impl for<'a> FnOnce(Option<&'a [u8]>) -> R,
    ) -> R {
        with_bytes_field(&self.item, HASHED_PASSWORD_USERNAME, callback)
    }

    /// Consumes this model into its object-specific Structure Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }
}

fn push_text(
    children: &mut Vec<Item>,
    tag: u32,
    value: Option<SecretText>,
) -> Result<(), ManagedObjectError> {
    if let Some(value) = value {
        children.push(make_item(tag, value.into_ttlv_value())?);
    }
    Ok(())
}

fn has_field(item: &Item, tag: u32) -> bool {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => structure
            .children()
            .iter()
            .any(|field| field.tag().raw() == tag),
        _ => false,
    })
}

fn with_field<R>(
    item: &Item,
    tag: u32,
    callback: impl for<'a> FnOnce(Option<ValueView<'a>>) -> R,
) -> R {
    item.with_value(|value| match value {
        ValueView::Structure(structure) => match structure
            .children()
            .iter()
            .find(|field| field.tag().raw() == tag)
        {
            Some(field) => field.with_value(|value| callback(Some(value))),
            None => callback(None),
        },
        _ => callback(None),
    })
}

fn with_text_field<R>(
    item: &Item,
    tag: u32,
    callback: impl for<'a> FnOnce(Option<&'a str>) -> R,
) -> R {
    with_field(item, tag, |value| {
        callback(value.and_then(|value| match value {
            ValueView::TextString(text) => Some(text),
            _ => None,
        }))
    })
}

fn with_bytes_field<R>(
    item: &Item,
    tag: u32,
    callback: impl for<'a> FnOnce(Option<&'a [u8]>) -> R,
) -> R {
    with_field(item, tag, |value| {
        callback(value.and_then(|value| match value {
            ValueView::ByteString(bytes) => Some(bytes),
            _ => None,
        }))
    })
}

fn enum_field(item: &Item, tag: u32) -> Option<u32> {
    with_field(item, tag, |value| {
        value.and_then(|value| match value {
            ValueView::Enumeration(raw) => Some(*raw),
            _ => None,
        })
    })
}

fn integer_field(item: &Item, tag: u32) -> Option<i32> {
    with_field(item, tag, |value| {
        value.and_then(|value| match value {
            ValueView::Integer(raw) => Some(*raw),
            _ => None,
        })
    })
}

fn interval_field(item: &Item, tag: u32) -> Option<u32> {
    with_field(item, tag, |value| {
        value.and_then(|value| match value {
            ValueView::Interval(raw) => Some(*raw),
            _ => None,
        })
    })
}

impl fmt::Debug for PasswordCredentialObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_object(formatter, "PasswordCredentialObject", &self.item)
    }
}

impl fmt::Debug for DeviceCredentialObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_object(formatter, "DeviceCredentialObject", &self.item)
    }
}

impl fmt::Debug for OneTimePasswordCredentialObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_object(formatter, "OneTimePasswordCredentialObject", &self.item)
    }
}

impl fmt::Debug for HashedPasswordCredentialObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_object(formatter, "HashedPasswordCredentialObject", &self.item)
    }
}

fn debug_object(formatter: &mut fmt::Formatter<'_>, name: &str, item: &Item) -> fmt::Result {
    let child_count = item.with_value(|value| match value {
        ValueView::Structure(structure) => structure.children().len(),
        _ => 0,
    });
    formatter
        .debug_struct(name)
        .field("child_count", &child_count)
        .finish_non_exhaustive()
}
