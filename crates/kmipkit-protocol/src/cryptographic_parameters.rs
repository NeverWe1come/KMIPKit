//! Typed KMIP Cryptographic Parameters and shared local validation.

use std::fmt;

use kmipkit_ttlv::{Item, ModelError, RawTag, Structure, StructureView, Tag, Value, ValueView};

use crate::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind, SecretBytes};

const BLOCK_CIPHER_MODE: u32 = 0x0042_0011;
const CRYPTOGRAPHIC_ALGORITHM: u32 = 0x0042_0028;
const HASHING_ALGORITHM: u32 = 0x0042_0038;
const PADDING_METHOD: u32 = 0x0042_005F;
const KEY_ROLE_TYPE: u32 = 0x0042_0083;
const DIGITAL_SIGNATURE_ALGORITHM: u32 = 0x0042_00AE;
const RANDOM_IV: u32 = 0x0042_00C5;
const IV_LENGTH: u32 = 0x0042_00CD;
const TAG_LENGTH: u32 = 0x0042_00CE;
const FIXED_FIELD_LENGTH: u32 = 0x0042_00CF;
const COUNTER_LENGTH: u32 = 0x0042_00D0;
const INITIAL_COUNTER_VALUE: u32 = 0x0042_00D1;
const INVOCATION_FIELD_LENGTH: u32 = 0x0042_00D2;
const SALT_LENGTH: u32 = 0x0042_0100;
const MASK_GENERATOR: u32 = 0x0042_0101;
const MASK_GENERATOR_HASHING_ALGORITHM: u32 = 0x0042_0102;
const P_SOURCE: u32 = 0x0042_0103;
const TRAILER_FIELD: u32 = 0x0042_0104;

// OASIS KMIP v2.1 §11.6, Table 436. NIST SP 800-38C Appendix A.1 supports
// classifying CCM as variable-IV through its permitted example nonce lengths.
const CTR_MODE: u32 = 6;
const CCM_MODE: u32 = 8;
const GCM_MODE: u32 = 9;

/// Typed, lossless view of the KMIP v2.1 §4.16 Cryptographic Parameters
/// Structure.
///
/// Known members are validated against Table 59. Unknown members remain in
/// the owned TTLV tree with their original positions and values. Enumeration
/// accessors return raw values so future and vendor values remain representable.
pub struct CryptographicParameters {
    structure: Structure,
}

impl CryptographicParameters {
    /// Parses and validates a Cryptographic Parameters Structure while
    /// preserving unknown members and their positions.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error for a known member with the wrong TTLV
    /// type, a repeated known singleton, known members outside Table 59 order,
    /// or locally checkable §4.16 parameter requirements.
    pub fn try_from_ttlv(structure: Structure) -> Result<Self, ProtocolError> {
        validate_known_cryptographic_parameters_view(&structure.view())?;
        validate_cryptographic_parameters_view(&structure.view())?;
        Ok(Self { structure })
    }

    /// Lends the complete original ordered TTLV Structure for inspection.
    pub fn with_ttlv<R>(&self, callback: impl for<'a> FnOnce(StructureView<'a>) -> R) -> R {
        callback(self.structure.view())
    }

    /// Consumes this typed value into its original ordered TTLV Structure.
    #[must_use]
    pub fn into_ttlv(self) -> Structure {
        self.structure
    }

    /// Returns the raw Block Cipher Mode Enumeration, if present.
    #[must_use]
    pub fn block_cipher_mode(&self) -> Option<u32> {
        enumeration(&self.structure, BLOCK_CIPHER_MODE)
    }

    /// Returns the raw Padding Method Enumeration, if present.
    #[must_use]
    pub fn padding_method(&self) -> Option<u32> {
        enumeration(&self.structure, PADDING_METHOD)
    }

    /// Returns the raw Hashing Algorithm Enumeration, if present.
    #[must_use]
    pub fn hashing_algorithm(&self) -> Option<u32> {
        enumeration(&self.structure, HASHING_ALGORITHM)
    }

    /// Returns the raw Key Role Type Enumeration, if present.
    #[must_use]
    pub fn key_role_type(&self) -> Option<u32> {
        enumeration(&self.structure, KEY_ROLE_TYPE)
    }

    /// Returns the raw Digital Signature Algorithm Enumeration, if present.
    #[must_use]
    pub fn digital_signature_algorithm(&self) -> Option<u32> {
        enumeration(&self.structure, DIGITAL_SIGNATURE_ALGORITHM)
    }

    /// Returns the raw Cryptographic Algorithm Enumeration, if present.
    #[must_use]
    pub fn cryptographic_algorithm(&self) -> Option<u32> {
        enumeration(&self.structure, CRYPTOGRAPHIC_ALGORITHM)
    }

    /// Returns Random IV, if present.
    #[must_use]
    pub fn random_iv(&self) -> Option<bool> {
        boolean(&self.structure, RANDOM_IV)
    }

    /// Returns IV Length, if present.
    #[must_use]
    pub fn iv_length(&self) -> Option<i32> {
        integer(&self.structure, IV_LENGTH)
    }

    /// Returns Tag Length, if present.
    #[must_use]
    pub fn tag_length(&self) -> Option<i32> {
        integer(&self.structure, TAG_LENGTH)
    }

    /// Returns Fixed Field Length, if present.
    #[must_use]
    pub fn fixed_field_length(&self) -> Option<i32> {
        integer(&self.structure, FIXED_FIELD_LENGTH)
    }

    /// Returns Invocation Field Length, if present.
    #[must_use]
    pub fn invocation_field_length(&self) -> Option<i32> {
        integer(&self.structure, INVOCATION_FIELD_LENGTH)
    }

    /// Returns Counter Length, if present.
    #[must_use]
    pub fn counter_length(&self) -> Option<i32> {
        integer(&self.structure, COUNTER_LENGTH)
    }

    /// Returns Initial Counter Value, if present.
    #[must_use]
    pub fn initial_counter_value(&self) -> Option<i32> {
        integer(&self.structure, INITIAL_COUNTER_VALUE)
    }

    /// Returns Salt Length, if present.
    #[must_use]
    pub fn salt_length(&self) -> Option<i32> {
        integer(&self.structure, SALT_LENGTH)
    }

    /// Returns the raw Mask Generator Enumeration, if present.
    #[must_use]
    pub fn mask_generator(&self) -> Option<u32> {
        enumeration(&self.structure, MASK_GENERATOR)
    }

    /// Returns the raw Mask Generator Hashing Algorithm Enumeration, if present.
    #[must_use]
    pub fn mask_generator_hashing_algorithm(&self) -> Option<u32> {
        enumeration(&self.structure, MASK_GENERATOR_HASHING_ALGORITHM)
    }

    /// Lends P Source bytes only for the duration of `callback`.
    ///
    /// Any copy made by the callback is outside `KMIPKit`'s zeroization
    /// guarantee.
    pub fn with_p_source<R>(&self, callback: impl for<'a> FnOnce(&'a [u8]) -> R) -> Option<R> {
        self.structure
            .view()
            .children()
            .iter()
            .find(|field| field.tag().raw() == P_SOURCE)
            .and_then(|field| {
                field.with_value(|value| match value {
                    ValueView::ByteString(bytes) => Some(callback(bytes)),
                    _ => None,
                })
            })
    }

    /// Returns Trailer Field, if present.
    #[must_use]
    pub fn trailer_field(&self) -> Option<i32> {
        integer(&self.structure, TRAILER_FIELD)
    }
}

impl fmt::Debug for CryptographicParameters {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CryptographicParameters")
            .field("member_count", &self.structure.view().children().len())
            .finish()
    }
}

/// Builder for a KMIP v2.1 §4.16 Cryptographic Parameters Structure.
///
/// Setter call order does not affect encoded Table 59 order. P Source is held
/// in a zeroizing secret owner until it moves into the TTLV tree.
pub struct CryptographicParametersBuilder {
    block_cipher_mode: Option<u32>,
    padding_method: Option<u32>,
    hashing_algorithm: Option<u32>,
    key_role_type: Option<u32>,
    digital_signature_algorithm: Option<u32>,
    cryptographic_algorithm: Option<u32>,
    random_iv: Option<bool>,
    iv_length: Option<i32>,
    tag_length: Option<i32>,
    fixed_field_length: Option<i32>,
    invocation_field_length: Option<i32>,
    counter_length: Option<i32>,
    initial_counter_value: Option<i32>,
    salt_length: Option<i32>,
    mask_generator: Option<u32>,
    mask_generator_hashing_algorithm: Option<u32>,
    p_source: Option<SecretBytes>,
    trailer_field: Option<i32>,
    extensions: Vec<Item>,
}

impl CryptographicParametersBuilder {
    /// Creates an empty builder whose members may be explicitly supplied.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            block_cipher_mode: None,
            padding_method: None,
            hashing_algorithm: None,
            key_role_type: None,
            digital_signature_algorithm: None,
            cryptographic_algorithm: None,
            random_iv: None,
            iv_length: None,
            tag_length: None,
            fixed_field_length: None,
            invocation_field_length: None,
            counter_length: None,
            initial_counter_value: None,
            salt_length: None,
            mask_generator: None,
            mask_generator_hashing_algorithm: None,
            p_source: None,
            trailer_field: None,
            extensions: Vec::new(),
        }
    }

    /// Sets the Block Cipher Mode raw Enumeration.
    #[must_use]
    pub const fn with_block_cipher_mode(mut self, value: u32) -> Self {
        self.block_cipher_mode = Some(value);
        self
    }
    /// Sets the Padding Method raw Enumeration.
    #[must_use]
    pub const fn with_padding_method(mut self, value: u32) -> Self {
        self.padding_method = Some(value);
        self
    }
    /// Sets the Hashing Algorithm raw Enumeration.
    #[must_use]
    pub const fn with_hashing_algorithm(mut self, value: u32) -> Self {
        self.hashing_algorithm = Some(value);
        self
    }
    /// Sets the Key Role Type raw Enumeration.
    #[must_use]
    pub const fn with_key_role_type(mut self, value: u32) -> Self {
        self.key_role_type = Some(value);
        self
    }
    /// Sets the Digital Signature Algorithm raw Enumeration.
    #[must_use]
    pub const fn with_digital_signature_algorithm(mut self, value: u32) -> Self {
        self.digital_signature_algorithm = Some(value);
        self
    }
    /// Sets the Cryptographic Algorithm raw Enumeration.
    #[must_use]
    pub const fn with_cryptographic_algorithm(mut self, value: u32) -> Self {
        self.cryptographic_algorithm = Some(value);
        self
    }
    /// Sets Random IV.
    #[must_use]
    pub const fn with_random_iv(mut self, value: bool) -> Self {
        self.random_iv = Some(value);
        self
    }
    /// Sets IV Length.
    #[must_use]
    pub const fn with_iv_length(mut self, value: i32) -> Self {
        self.iv_length = Some(value);
        self
    }
    /// Sets Tag Length.
    #[must_use]
    pub const fn with_tag_length(mut self, value: i32) -> Self {
        self.tag_length = Some(value);
        self
    }
    /// Sets Fixed Field Length.
    #[must_use]
    pub const fn with_fixed_field_length(mut self, value: i32) -> Self {
        self.fixed_field_length = Some(value);
        self
    }
    /// Sets Invocation Field Length.
    #[must_use]
    pub const fn with_invocation_field_length(mut self, value: i32) -> Self {
        self.invocation_field_length = Some(value);
        self
    }
    /// Sets Counter Length.
    #[must_use]
    pub const fn with_counter_length(mut self, value: i32) -> Self {
        self.counter_length = Some(value);
        self
    }
    /// Sets Initial Counter Value.
    #[must_use]
    pub const fn with_initial_counter_value(mut self, value: i32) -> Self {
        self.initial_counter_value = Some(value);
        self
    }
    /// Sets Salt Length.
    #[must_use]
    pub const fn with_salt_length(mut self, value: i32) -> Self {
        self.salt_length = Some(value);
        self
    }
    /// Sets the Mask Generator raw Enumeration.
    #[must_use]
    pub const fn with_mask_generator(mut self, value: u32) -> Self {
        self.mask_generator = Some(value);
        self
    }
    /// Sets the Mask Generator Hashing Algorithm raw Enumeration.
    #[must_use]
    pub const fn with_mask_generator_hashing_algorithm(mut self, value: u32) -> Self {
        self.mask_generator_hashing_algorithm = Some(value);
        self
    }
    /// Takes ownership of P Source bytes.
    #[must_use]
    pub fn with_p_source(mut self, value: Vec<u8>) -> Self {
        self.p_source = Some(SecretBytes::new(value));
        self
    }
    /// Sets Trailer Field.
    #[must_use]
    pub const fn with_trailer_field(mut self, value: i32) -> Self {
        self.trailer_field = Some(value);
        self
    }

    /// Takes ownership of an unknown or vendor extension member.
    #[must_use]
    pub fn with_extension(mut self, item: Item) -> Self {
        self.extensions.push(item);
        self
    }

    /// Builds and validates the Table 59 ordered Structure.
    ///
    /// # Errors
    ///
    /// Returns a payload-free error if construction exceeds the TTLV model
    /// limits or a locally checkable §4.16 parameter requirement is unmet.
    pub fn build(self) -> Result<CryptographicParameters, ProtocolError> {
        let mut structure = Structure::new();
        push_enum(&mut structure, BLOCK_CIPHER_MODE, self.block_cipher_mode)?;
        push_enum(&mut structure, PADDING_METHOD, self.padding_method)?;
        push_enum(&mut structure, HASHING_ALGORITHM, self.hashing_algorithm)?;
        push_enum(&mut structure, KEY_ROLE_TYPE, self.key_role_type)?;
        push_enum(
            &mut structure,
            DIGITAL_SIGNATURE_ALGORITHM,
            self.digital_signature_algorithm,
        )?;
        push_enum(
            &mut structure,
            CRYPTOGRAPHIC_ALGORITHM,
            self.cryptographic_algorithm,
        )?;
        push_boolean(&mut structure, RANDOM_IV, self.random_iv)?;
        push_integer(&mut structure, IV_LENGTH, self.iv_length)?;
        push_integer(&mut structure, TAG_LENGTH, self.tag_length)?;
        push_integer(&mut structure, FIXED_FIELD_LENGTH, self.fixed_field_length)?;
        push_integer(
            &mut structure,
            INVOCATION_FIELD_LENGTH,
            self.invocation_field_length,
        )?;
        push_integer(&mut structure, COUNTER_LENGTH, self.counter_length)?;
        push_integer(
            &mut structure,
            INITIAL_COUNTER_VALUE,
            self.initial_counter_value,
        )?;
        push_integer(&mut structure, SALT_LENGTH, self.salt_length)?;
        push_enum(&mut structure, MASK_GENERATOR, self.mask_generator)?;
        push_enum(
            &mut structure,
            MASK_GENERATOR_HASHING_ALGORITHM,
            self.mask_generator_hashing_algorithm,
        )?;
        if let Some(value) = self.p_source {
            push(&mut structure, P_SOURCE, value.into_ttlv_value())?;
        }
        push_integer(&mut structure, TRAILER_FIELD, self.trailer_field)?;
        for extension in self.extensions {
            structure.try_push(extension).map_err(model_error)?;
        }
        Self::validate_built(structure)
    }

    fn validate_built(structure: Structure) -> Result<CryptographicParameters, ProtocolError> {
        CryptographicParameters::try_from_ttlv(structure)
    }
}

impl Default for CryptographicParametersBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CryptographicParametersBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let member_count = [
            self.block_cipher_mode.is_some(),
            self.padding_method.is_some(),
            self.hashing_algorithm.is_some(),
            self.key_role_type.is_some(),
            self.digital_signature_algorithm.is_some(),
            self.cryptographic_algorithm.is_some(),
            self.random_iv.is_some(),
            self.iv_length.is_some(),
            self.tag_length.is_some(),
            self.fixed_field_length.is_some(),
            self.invocation_field_length.is_some(),
            self.counter_length.is_some(),
            self.initial_counter_value.is_some(),
            self.salt_length.is_some(),
            self.mask_generator.is_some(),
            self.mask_generator_hashing_algorithm.is_some(),
            self.p_source.is_some(),
            self.trailer_field.is_some(),
        ]
        .into_iter()
        .map(usize::from)
        .sum::<usize>();
        formatter
            .debug_struct("CryptographicParametersBuilder")
            .field("member_count", &member_count)
            .field("extension_count", &self.extensions.len())
            .finish()
    }
}

fn enumeration(structure: &Structure, tag: u32) -> Option<u32> {
    find_field(structure, tag, |value| match value {
        ValueView::Enumeration(value) => Some(*value),
        _ => None,
    })
}

fn integer(structure: &Structure, tag: u32) -> Option<i32> {
    find_field(structure, tag, |value| match value {
        ValueView::Integer(value) => Some(*value),
        _ => None,
    })
}

fn boolean(structure: &Structure, tag: u32) -> Option<bool> {
    find_field(structure, tag, |value| match value {
        ValueView::Boolean(value) => Some(*value),
        _ => None,
    })
}

fn find_field<R>(
    structure: &Structure,
    tag: u32,
    read: impl FnOnce(ValueView<'_>) -> Option<R>,
) -> Option<R> {
    structure
        .view()
        .children()
        .iter()
        .find(|field| field.tag().raw() == tag)
        .and_then(|field| field.with_value(read))
}

fn model_error(error: ModelError) -> ProtocolError {
    ProtocolError::new(
        ProtocolErrorKind::InvalidValue,
        ProtocolCauseCategory::InvalidValue,
        error,
    )
}

fn tag(raw: u32) -> Result<Tag, ProtocolError> {
    RawTag::new(raw)
        .and_then(|raw| raw.try_checked())
        .map_err(model_error)
}

fn push(structure: &mut Structure, raw_tag: u32, value: Value) -> Result<(), ProtocolError> {
    structure
        .try_push(Item::new(tag(raw_tag)?, value).map_err(model_error)?)
        .map_err(model_error)
}

fn push_enum(
    structure: &mut Structure,
    raw_tag: u32,
    value: Option<u32>,
) -> Result<(), ProtocolError> {
    if let Some(value) = value {
        push(structure, raw_tag, Value::enumeration(value))?;
    }
    Ok(())
}

fn push_integer(
    structure: &mut Structure,
    raw_tag: u32,
    value: Option<i32>,
) -> Result<(), ProtocolError> {
    if let Some(value) = value {
        push(structure, raw_tag, Value::integer(value))?;
    }
    Ok(())
}

fn push_boolean(
    structure: &mut Structure,
    raw_tag: u32,
    value: Option<bool>,
) -> Result<(), ProtocolError> {
    if let Some(value) = value {
        push(structure, raw_tag, Value::boolean(value))?;
    }
    Ok(())
}

/// Validates only §4.16 requirements that can be established from the supplied
/// Cryptographic Parameters Structure and returns the original Structure.
///
/// The supplied tree is never rebuilt or reordered. Unknown members, unknown
/// mode values, and the original encoded values remain intact. This validator
/// recognizes CTR (6), CCM (8), and GCM (9) as variable-IV modes requiring IV
/// Length. GCM also requires Tag Length; CCM does not. Other mode values are
/// preserved without inferred requirements. An omitted operation-level value
/// remains omitted, because the server may obtain parameters from object
/// attributes that the client cannot inspect.
///
/// This helper checks required-member presence only. It does not validate the
/// TTLV Item Types or multiplicity of known parameter members; typed request
/// conversion is responsible for those structural checks.
///
/// # Errors
///
/// Returns a sanitized `InvalidValue` error when supplied parameters identify
/// CTR, CCM, or GCM without IV Length, or GCM without Tag Length. The error
/// contains no member values or raw parameter data.
pub(crate) fn validate_cryptographic_parameters(
    parameters: Option<Structure>,
) -> Result<Option<Structure>, ProtocolError> {
    let Some(ref supplied) = parameters else {
        return Ok(None);
    };

    validate_cryptographic_parameters_view(&supplied.view())?;
    Ok(parameters)
}

/// Validates required-member presence while preserving the caller's borrowed
/// ordered Structure. Typed request boundaries use this when they validate a
/// generic payload without taking ownership of its nested values.
pub(crate) fn validate_cryptographic_parameters_view(
    supplied: &StructureView<'_>,
) -> Result<(), ProtocolError> {
    let mut requires_iv_length = false;
    let mut requires_tag_length = false;
    let mut has_iv_length = false;
    let mut has_tag_length = false;

    for item in supplied.children() {
        match item.tag().raw() {
            BLOCK_CIPHER_MODE => {
                let mode = item.with_value(|value| match value {
                    ValueView::Enumeration(mode) => Some(*mode),
                    _ => None,
                });
                match mode {
                    Some(CTR_MODE | CCM_MODE) => requires_iv_length = true,
                    Some(GCM_MODE) => {
                        requires_iv_length = true;
                        requires_tag_length = true;
                    }
                    Some(_) | None => {}
                }
            }
            IV_LENGTH => has_iv_length = true,
            TAG_LENGTH => has_tag_length = true,
            _ => {}
        }
    }

    if (requires_iv_length && !has_iv_length) || (requires_tag_length && !has_tag_length) {
        return Err(ProtocolError::categorized(
            ProtocolErrorKind::InvalidValue,
            ProtocolCauseCategory::InvalidValue,
        ));
    }

    Ok(())
}

/// Validates Table 59 member types, singleton cardinality, and known-member
/// order while retaining the original Structure and every unknown member.
pub(super) fn validate_known_cryptographic_parameters_view(
    parameters: &StructureView<'_>,
) -> Result<(), ProtocolError> {
    let mut seen = [false; 18];
    let mut last_position = None;
    for field in parameters.children() {
        let (index, valid_type) = match field.tag().raw() {
            BLOCK_CIPHER_MODE => (
                Some(0),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            PADDING_METHOD => (
                Some(1),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            HASHING_ALGORITHM => (
                Some(2),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            KEY_ROLE_TYPE => (
                Some(3),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            DIGITAL_SIGNATURE_ALGORITHM => (
                Some(4),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            CRYPTOGRAPHIC_ALGORITHM => (
                Some(5),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            RANDOM_IV => (
                Some(6),
                field.with_value(|value| matches!(value, ValueView::Boolean(_))),
            ),
            IV_LENGTH => (
                Some(7),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            TAG_LENGTH => (
                Some(8),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            FIXED_FIELD_LENGTH => (
                Some(9),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            INVOCATION_FIELD_LENGTH => (
                Some(10),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            COUNTER_LENGTH => (
                Some(11),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            INITIAL_COUNTER_VALUE => (
                Some(12),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            SALT_LENGTH => (
                Some(13),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            MASK_GENERATOR => (
                Some(14),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            MASK_GENERATOR_HASHING_ALGORITHM => (
                Some(15),
                field.with_value(|value| matches!(value, ValueView::Enumeration(_))),
            ),
            P_SOURCE => (
                Some(16),
                field.with_value(|value| matches!(value, ValueView::ByteString(_))),
            ),
            TRAILER_FIELD => (
                Some(17),
                field.with_value(|value| matches!(value, ValueView::Integer(_))),
            ),
            _ => (None, true),
        };
        if !valid_type {
            return Err(ProtocolError::categorized(
                ProtocolErrorKind::InvalidValue,
                ProtocolCauseCategory::InvalidValue,
            ));
        }
        if let Some(index) = index {
            if seen[index] || last_position.is_some_and(|last| index < last) {
                return Err(ProtocolError::categorized(
                    ProtocolErrorKind::InvalidValue,
                    ProtocolCauseCategory::InvalidValue,
                ));
            }
            seen[index] = true;
            last_position = Some(index);
        }
    }

    Ok(())
}
