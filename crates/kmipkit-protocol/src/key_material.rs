//! Typed KMIP 2.1 Key Material and transparent key structures (§§3.2–3.12).
#![allow(clippy::missing_errors_doc)] // Fallible constructors share the sanitized schema error documented on ManagedObjectError.

use std::fmt;

use kmipkit_ttlv::{Item, ItemType, Structure, Value};
use zeroize::Zeroizing;

use crate::ManagedObjectError;
use crate::managed_object::{make_item, make_structure_item, validate_structure};

const KEY_MATERIAL: u32 = 0x0042_0043;
const KEY: u32 = 0x0042_003F;
const P: u32 = 0x0042_005E;
const Q: u32 = 0x0042_0071;
const G: u32 = 0x0042_0037;
const X: u32 = 0x0042_009F;
const Y: u32 = 0x0042_00A0;
const J: u32 = 0x0042_003E;
const MODULUS: u32 = 0x0042_0052;
const PUBLIC_EXPONENT: u32 = 0x0042_006C;
const PRIVATE_EXPONENT: u32 = 0x0042_0063;
const PRIME_EXPONENT_P: u32 = 0x0042_0060;
const PRIME_EXPONENT_Q: u32 = 0x0042_0061;
const CRT_COEFFICIENT: u32 = 0x0042_0027;
const RECOMMENDED_CURVE: u32 = 0x0042_0075;
const D: u32 = 0x0042_002E;
const Q_STRING: u32 = 0x0042_0072;

/// A lossless Key Material Item with format-aware constructors.
pub struct KeyMaterial {
    item: Item,
    format_raw: u32,
}

impl fmt::Debug for KeyMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KeyMaterial")
            .field("format_raw", &self.format_raw)
            .field("item", &self.item)
            .finish()
    }
}

impl KeyMaterial {
    /// Creates octet material for the Raw, Opaque, PKCS, X.509, or EC private formats.
    /// Unknown formats remain open and accept either TTLV form.
    pub fn byte_string(format_raw: u32, bytes: Vec<u8>) -> Result<Self, ManagedObjectError> {
        let mut bytes = Zeroizing::new(bytes);
        if !format_accepts_type(format_raw, ItemType::ByteString) {
            return Err(ManagedObjectError::IncorrectFieldType);
        }
        Ok(Self {
            item: make_item(
                KEY_MATERIAL,
                Value::byte_string(std::mem::take(&mut *bytes)),
            )?,
            format_raw,
        })
    }

    /// Creates structure material for a transparent key format.
    /// Unknown formats remain open and accept either TTLV form.
    pub fn structure(format_raw: u32, structure: Structure) -> Result<Self, ManagedObjectError> {
        if !format_accepts_type(format_raw, ItemType::Structure) {
            return Err(ManagedObjectError::IncorrectFieldType);
        }
        let item = make_item(KEY_MATERIAL, Value::structure(structure))?;
        validate_key_material_item(&item, format_raw)?;
        Ok(Self { item, format_raw })
    }

    /// Consumes the Key Material into its TTLV Item.
    #[must_use]
    pub fn into_item(self) -> Item {
        self.item
    }

    /// Reads Key Material while checking its format's required TTLV representation.
    pub fn try_from_item(
        item: Item,
        format: crate::KeyFormatType,
    ) -> Result<Self, ManagedObjectError> {
        validate_key_material_item(&item, format.raw())?;
        Ok(Self {
            item,
            format_raw: format.raw(),
        })
    }

    /// Returns the exact source-preserving Key Format Type.
    #[must_use]
    pub const fn format_raw(&self) -> u32 {
        self.format_raw
    }

    /// Returns the Key Material TTLV Item Type.
    #[must_use]
    pub fn item_type(&self) -> ItemType {
        self.item.item_type()
    }

    /// Lends the Key Material value for callback-scoped inspection.
    pub fn with_raw<R>(
        &self,
        callback: impl for<'a> FnOnce(kmipkit_ttlv::ValueView<'a>) -> R,
    ) -> R {
        self.item.with_value(callback)
    }
}

fn format_accepts_type(format: u32, item_type: ItemType) -> bool {
    expected_key_material_item_type(format).map_or(
        matches!(item_type, ItemType::ByteString | ItemType::Structure),
        |expected| expected == item_type,
    )
}

/// Returns the TTLV type required by an assigned KMIP 2.1 Key Format Type.
/// Reserved and vendor-defined values remain open and return `None`.
#[must_use]
pub const fn expected_key_material_item_type(format: u32) -> Option<ItemType> {
    match format {
        0x0000_0001..=0x0000_0006 | 0x0000_0016..=0x0000_0017 => Some(ItemType::ByteString),
        0x0000_0007..=0x0000_000D | 0x0000_0014..=0x0000_0015 => Some(ItemType::Structure),
        _ => None,
    }
}

pub(crate) fn validate_key_material_item(
    item: &Item,
    format: u32,
) -> Result<(), ManagedObjectError> {
    if item.tag().raw() != KEY_MATERIAL || !format_accepts_type(format, item.item_type()) {
        return Err(ManagedObjectError::IncorrectFieldType);
    }
    match format {
        7 => TransparentSymmetricKey::validate_item(item),
        8 => TransparentDsaPrivateKey::validate_item(item),
        9 => TransparentDsaPublicKey::validate_item(item),
        10 => TransparentRsaPrivateKey::validate_item(item),
        11 => TransparentRsaPublicKey::validate_item(item),
        12 => TransparentDhPrivateKey::validate_item(item),
        13 => TransparentDhPublicKey::validate_item(item),
        14 => TransparentEcPrivateKey::validate_item(item),
        15 => TransparentEcPublicKey::validate_item(item),
        _ => Ok(()),
    }
}

fn big(raw: u32, value: Vec<u8>) -> Result<Item, ManagedObjectError> {
    make_item(raw, Value::big_integer(value))
}

fn enum_item(raw: u32, value: u32) -> Result<Item, ManagedObjectError> {
    make_item(raw, Value::enumeration(value))
}

fn key_material_structure(
    fields: impl IntoIterator<Item = Item>,
) -> Result<Item, ManagedObjectError> {
    make_structure_item(KEY_MATERIAL, fields)
}

macro_rules! transparent_key {
    ($name:ident, $schema:expr) => {
        transparent_key!($name, $schema, |_item: &Item| Ok(()));
    };
    ($name:ident, $schema:expr, $additional_validation:expr) => {
        /// A typed transparent KMIP Key Material structure.
        pub struct $name {
            item: Item,
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("item", &self.item)
                    .finish()
            }
        }

        impl $name {
            fn validate_item(item: &Item) -> Result<(), ManagedObjectError> {
                validate_structure(item, KEY_MATERIAL, $schema)?;
                $additional_validation(item)
            }

            /// Reads a transparent Key Material Item and retains vendor fields.
            pub fn try_from_item(item: Item) -> Result<Self, ManagedObjectError> {
                Self::validate_item(&item)?;
                Ok(Self { item })
            }

            /// Consumes the typed structure into its Key Material Item.
            #[must_use]
            pub fn into_item(self) -> Item {
                self.item
            }
        }
    };
}

transparent_key!(
    TransparentSymmetricKey,
    &[(KEY, ItemType::ByteString, true)]
);
transparent_key!(
    TransparentDsaPrivateKey,
    &[
        (P, ItemType::BigInteger, true),
        (Q, ItemType::BigInteger, true),
        (G, ItemType::BigInteger, true),
        (X, ItemType::BigInteger, true)
    ]
);
transparent_key!(
    TransparentDsaPublicKey,
    &[
        (P, ItemType::BigInteger, true),
        (Q, ItemType::BigInteger, true),
        (G, ItemType::BigInteger, true),
        (Y, ItemType::BigInteger, true)
    ]
);
transparent_key!(
    TransparentRsaPrivateKey,
    &[
        (MODULUS, ItemType::BigInteger, true),
        (PUBLIC_EXPONENT, ItemType::BigInteger, false),
        (PRIVATE_EXPONENT, ItemType::BigInteger, false),
        (P, ItemType::BigInteger, false),
        (Q, ItemType::BigInteger, false),
        (PRIME_EXPONENT_P, ItemType::BigInteger, false),
        (PRIME_EXPONENT_Q, ItemType::BigInteger, false),
        (CRT_COEFFICIENT, ItemType::BigInteger, false)
    ],
    validate_rsa_private_alternatives
);
transparent_key!(
    TransparentRsaPublicKey,
    &[
        (MODULUS, ItemType::BigInteger, true),
        (PUBLIC_EXPONENT, ItemType::BigInteger, true)
    ]
);
transparent_key!(
    TransparentDhPrivateKey,
    &[
        (P, ItemType::BigInteger, true),
        (Q, ItemType::BigInteger, false),
        (G, ItemType::BigInteger, true),
        (J, ItemType::BigInteger, false),
        (X, ItemType::BigInteger, true)
    ]
);
transparent_key!(
    TransparentDhPublicKey,
    &[
        (P, ItemType::BigInteger, true),
        (Q, ItemType::BigInteger, false),
        (G, ItemType::BigInteger, true),
        (J, ItemType::BigInteger, false),
        (Y, ItemType::BigInteger, true)
    ]
);
transparent_key!(
    TransparentEcPrivateKey,
    &[
        (RECOMMENDED_CURVE, ItemType::Enumeration, true),
        (D, ItemType::BigInteger, true)
    ]
);
transparent_key!(
    TransparentEcPublicKey,
    &[
        (RECOMMENDED_CURVE, ItemType::Enumeration, true),
        (Q_STRING, ItemType::ByteString, true)
    ]
);

impl TransparentSymmetricKey {
    /// Constructs the required symmetric Key bytes.
    pub fn try_new(key: Vec<u8>) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(key_material_structure([make_item(
            KEY,
            Value::byte_string(key),
        )?])?)
    }
}

impl TransparentDsaPrivateKey {
    /// Constructs DSA private material in Table 19 order.
    pub fn try_new(
        p: Vec<u8>,
        q: Vec<u8>,
        g: Vec<u8>,
        x: Vec<u8>,
    ) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(key_material_structure([
            big(P, p)?,
            big(Q, q)?,
            big(G, g)?,
            big(X, x)?,
        ])?)
    }
}

impl TransparentDsaPublicKey {
    /// Constructs DSA public material in Table 20 order.
    pub fn try_new(
        p: Vec<u8>,
        q: Vec<u8>,
        g: Vec<u8>,
        y: Vec<u8>,
    ) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(key_material_structure([
            big(P, p)?,
            big(Q, q)?,
            big(G, g)?,
            big(Y, y)?,
        ])?)
    }
}

impl TransparentRsaPrivateKey {
    /// Constructs RSA private material from one of the standard alternatives.
    #[allow(clippy::too_many_arguments)]
    pub fn try_from_components(
        modulus: Vec<u8>,
        private_exponent: Option<Vec<u8>>,
        public_exponent: Option<Vec<u8>>,
        p: Option<Vec<u8>>,
        q: Option<Vec<u8>>,
        prime_exponent_p: Option<Vec<u8>>,
        prime_exponent_q: Option<Vec<u8>>,
        crt_coefficient: Option<Vec<u8>>,
    ) -> Result<Self, ManagedObjectError> {
        let mut modulus = Zeroizing::new(modulus);
        let mut public_exponent = public_exponent.map(Zeroizing::new);
        let mut private_exponent = private_exponent.map(Zeroizing::new);
        let mut p = p.map(Zeroizing::new);
        let mut q = q.map(Zeroizing::new);
        let mut prime_exponent_p = prime_exponent_p.map(Zeroizing::new);
        let mut prime_exponent_q = prime_exponent_q.map(Zeroizing::new);
        let mut crt_coefficient = crt_coefficient.map(Zeroizing::new);
        let exponent = private_exponent.is_some();
        let factors = p.is_some() && q.is_some();
        let crt = prime_exponent_p.is_some() && prime_exponent_q.is_some();
        if !(exponent || factors || crt)
            || p.is_some() != q.is_some()
            || prime_exponent_p.is_some() != prime_exponent_q.is_some()
        {
            return Err(ManagedObjectError::InvalidFieldCombination);
        }
        let mut fields = vec![big(MODULUS, std::mem::take(&mut *modulus))?];
        if let Some(mut value) = public_exponent.take() {
            fields.push(big(PUBLIC_EXPONENT, std::mem::take(&mut *value))?);
        }
        if let Some(mut value) = private_exponent.take() {
            fields.push(big(PRIVATE_EXPONENT, std::mem::take(&mut *value))?);
        }
        if let Some(mut value) = p.take() {
            fields.push(big(P, std::mem::take(&mut *value))?);
        }
        if let Some(mut value) = q.take() {
            fields.push(big(Q, std::mem::take(&mut *value))?);
        }
        if let Some(mut value) = prime_exponent_p.take() {
            fields.push(big(PRIME_EXPONENT_P, std::mem::take(&mut *value))?);
        }
        if let Some(mut value) = prime_exponent_q.take() {
            fields.push(big(PRIME_EXPONENT_Q, std::mem::take(&mut *value))?);
        }
        if let Some(mut value) = crt_coefficient.take() {
            fields.push(big(CRT_COEFFICIENT, std::mem::take(&mut *value))?);
        }
        Self::try_from_item(key_material_structure(fields)?)
    }
}

fn validate_rsa_private_alternatives(item: &Item) -> Result<(), ManagedObjectError> {
    let has_private_exponent =
        crate::managed_object::with_field(item, PRIVATE_EXPONENT, |_| ()).is_some();
    let has_prime_factors = crate::managed_object::with_field(item, P, |_| ()).is_some()
        && crate::managed_object::with_field(item, Q, |_| ()).is_some();
    let has_prime_exponents = crate::managed_object::with_field(item, PRIME_EXPONENT_P, |_| ())
        .is_some()
        && crate::managed_object::with_field(item, PRIME_EXPONENT_Q, |_| ()).is_some();
    if has_private_exponent || has_prime_factors || has_prime_exponents {
        Ok(())
    } else {
        Err(ManagedObjectError::InvalidFieldCombination)
    }
}

impl TransparentRsaPublicKey {
    /// Constructs RSA public material.
    pub fn try_new(modulus: Vec<u8>, public_exponent: Vec<u8>) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(key_material_structure([
            big(MODULUS, modulus)?,
            big(PUBLIC_EXPONENT, public_exponent)?,
        ])?)
    }
}

macro_rules! dh_key {
    ($name:ident, $last:ident, $last_tag:expr) => {
        impl $name {
            /// Constructs DH material with optional Q and J values.
            pub fn try_new(
                p: Vec<u8>,
                g: Vec<u8>,
                last: Vec<u8>,
                q: Option<Vec<u8>>,
                j: Option<Vec<u8>>,
            ) -> Result<Self, ManagedObjectError> {
                let mut fields = vec![big(P, p)?];
                if let Some(value) = q {
                    fields.push(big(Q, value)?);
                }
                fields.push(big(G, g)?);
                if let Some(value) = j {
                    fields.push(big(J, value)?);
                }
                fields.push(big($last_tag, last)?);
                Self::try_from_item(key_material_structure(fields)?)
            }
        }
    };
}
dh_key!(TransparentDhPrivateKey, X, X);
dh_key!(TransparentDhPublicKey, Y, Y);

impl TransparentEcPrivateKey {
    /// Constructs EC private material and preserves future curve values.
    pub fn try_new(curve: u32, d: Vec<u8>) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(key_material_structure([
            enum_item(RECOMMENDED_CURVE, curve)?,
            big(D, d)?,
        ])?)
    }
}

impl TransparentEcPublicKey {
    /// Constructs EC public material and preserves future curve values.
    pub fn try_new(curve: u32, q_string: Vec<u8>) -> Result<Self, ManagedObjectError> {
        Self::try_from_item(key_material_structure([
            enum_item(RECOMMENDED_CURVE, curve)?,
            make_item(Q_STRING, Value::byte_string(q_string))?,
        ])?)
    }
}
