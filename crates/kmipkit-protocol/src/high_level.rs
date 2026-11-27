//! High-level presets, PEM/DER constructors, and version-aware attribute builders.
//!
//! These types allow application developers to configure KMIP 2.0, 2.1, and 3.0
//! requests without memorizing numeric enumerations, TTLV tags, or manual
//! Key Block / Key Material / Credential nesting.

use std::error::Error;
use std::fmt;

use kmipkit_ttlv::{Item, RawTag, Structure, Tag, Value};

use crate::named_values_generated::values::{
    enumeration::{
        block_cipher_mode, certificate_type, cryptographic_algorithm, derivation_method,
        digital_signature_algorithm, hashing_algorithm, key_format_type, padding_method,
        recommended_curve, secret_data_type,
    },
    versions::v3_0::enumeration::{
        cryptographic_algorithm as v3_cryptographic_algorithm, kem_algorithm as v3_kem_algorithm,
    },
};
use crate::typed_attributes::typed;
use crate::{
    Attribute, AttributeDefinition, AttributeSet, AttributeValueError, Authentication, Certificate,
    Credential, CredentialType, CredentialValidationError, CryptographicAlgorithm,
    CryptographicParameters, CryptographicParametersBuilder, DerivationMethod,
    DerivationParameters, HashingAlgorithm, KeyBlock, KeyBlockValue, KeyFormatType, KeyMaterial,
    KeyValue, LinkNormalizer, LinkType, ManagedObject, ManagedObjectError, OpaqueObject,
    PrivateKey, ProtocolError, ProtocolVersion, PublicKey, SecretBytes, SecretData, SymmetricKey,
};

const CREDENTIAL_TYPE_TAG: u32 = 0x0042_0024;
const CREDENTIAL_VALUE_TAG: u32 = 0x0042_0025;
const HASHING_ALGORITHM_TAG: u32 = 0x0042_0038;
const LINK_TYPE_TAG: u32 = 0x0042_004B;
const LINKED_OBJECT_IDENTIFIER_TAG: u32 = 0x0042_004C;
const NAME_TYPE_TAG: u32 = 0x0042_0054;
const NAME_VALUE_TAG: u32 = 0x0042_0055;
const RECOMMENDED_CURVE_TAG: u32 = 0x0042_0075;
const TIME_STAMP_TAG: u32 = 0x0042_0092;
const USERNAME_TAG: u32 = 0x0042_0099;
const PASSWORD_TAG: u32 = 0x0042_00A1;
const DEVICE_IDENTIFIER_TAG: u32 = 0x0042_00A2;
const MACHINE_IDENTIFIER_TAG: u32 = 0x0042_00A9;
const MEDIA_IDENTIFIER_TAG: u32 = 0x0042_00AA;
const NETWORK_IDENTIFIER_TAG: u32 = 0x0042_00AB;
const DEVICE_SERIAL_NUMBER_TAG: u32 = 0x0042_00B0;
const ALTERNATIVE_NAME_VALUE_TAG: u32 = 0x0042_00C0;
const ALTERNATIVE_NAME_TYPE_TAG: u32 = 0x0042_00C1;
const TICKET_TAG: u32 = 0x0042_0149;
const TICKET_TYPE_TAG: u32 = 0x0042_014A;
const TICKET_VALUE_TAG: u32 = 0x0042_014B;
const HASHED_PASSWORD_TAG: u32 = 0x0042_0157;

const NAME_TYPE_UNINTERPRETED_TEXT_STRING: u32 = 1;
const MASK_GENERATOR_MGF1: u32 = 1;

/// Payload-free error returned by high-level preset, PEM, object, and attribute helpers.
#[non_exhaustive]
#[derive(Debug)]
pub enum PresetError {
    /// Required input bytes or text were empty.
    EmptyInput,
    /// PEM framing header/footer validation failed.
    InvalidPem(&'static str),
    /// Base64 body could not be decoded according to RFC 7468.
    InvalidBase64,
    /// Key preset category does not match the constructor (symmetric vs asymmetric).
    IncompatibleKeySpec(&'static str),
    /// Raw symmetric key byte length did not match the selected key specification.
    InvalidKeyLength {
        /// Expected byte length derived from the preset.
        expected: usize,
        /// Actual byte length supplied by the caller.
        actual: usize,
    },
    /// TTLV construction exceeded model limits or used an invalid value.
    InvalidTtlv,
    /// Protocol parameter validation failed.
    Protocol(ProtocolError),
    /// Managed object or key block validation failed.
    ManagedObject(ManagedObjectError),
    /// Credential validation failed.
    Credential(CredentialValidationError),
    /// Typed attribute construction failed.
    Attribute(AttributeValueError),
    /// Version-specific attribute normalization failed.
    Normalization(String),
}

impl fmt::Display for PresetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => formatter.write_str("high-level input must not be empty"),
            Self::InvalidPem(reason) => write!(formatter, "invalid PEM encoding: {reason}"),
            Self::InvalidBase64 => formatter.write_str("invalid base64 payload in PEM block"),
            Self::IncompatibleKeySpec(reason) => {
                write!(formatter, "incompatible KeySpec for constructor: {reason}")
            }
            Self::InvalidKeyLength { expected, actual } => write!(
                formatter,
                "key byte length mismatch: expected {expected} bytes, got {actual}"
            ),
            Self::InvalidTtlv => formatter.write_str("failed to construct valid TTLV structure"),
            Self::Protocol(error) => write!(formatter, "protocol parameter error: {error}"),
            Self::ManagedObject(error) => write!(formatter, "managed object error: {error}"),
            Self::Credential(error) => write!(formatter, "credential validation error: {error}"),
            Self::Attribute(error) => write!(formatter, "attribute error: {error}"),
            Self::Normalization(message) => {
                write!(formatter, "attribute normalization error: {message}")
            }
        }
    }
}

impl Error for PresetError {}

impl From<ProtocolError> for PresetError {
    fn from(value: ProtocolError) -> Self {
        Self::Protocol(value)
    }
}

impl From<ManagedObjectError> for PresetError {
    fn from(value: ManagedObjectError) -> Self {
        Self::ManagedObject(value)
    }
}

impl From<CredentialValidationError> for PresetError {
    fn from(value: CredentialValidationError) -> Self {
        Self::Credential(value)
    }
}

impl From<AttributeValueError> for PresetError {
    fn from(value: AttributeValueError) -> Self {
        Self::Attribute(value)
    }
}

#[derive(Clone, Copy)]
struct KeySpecMetadata {
    algorithm: u32,
    length: i32,
    curve: Option<u32>,
    symmetric: bool,
    min_version: ProtocolVersion,
}

/// Curated symmetric, classical asymmetric, and KMIP 3.0 post-quantum key specifications.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KeySpec {
    /// AES symmetric key with 128-bit length.
    Aes128,
    /// AES symmetric key with 192-bit length.
    Aes192,
    /// AES symmetric key with 256-bit length.
    Aes256,
    /// HMAC-SHA256 key with 256-bit length.
    HmacSha256,
    /// HMAC-SHA384 key with 384-bit length.
    HmacSha384,
    /// HMAC-SHA512 key with 512-bit length.
    HmacSha512,
    /// `ChaCha20` symmetric stream cipher key with 256-bit length.
    ChaCha20,
    /// ChaCha20-Poly1305 AEAD symmetric key with 256-bit length.
    ChaCha20Poly1305,
    /// RSA 2048-bit key pair.
    Rsa2048,
    /// RSA 3072-bit key pair.
    Rsa3072,
    /// RSA 4096-bit key pair.
    Rsa4096,
    /// NIST P-256 (secp256r1) ECDSA key pair.
    EcdsaP256,
    /// NIST P-384 (secp384r1) ECDSA key pair.
    EcdsaP384,
    /// NIST P-521 (secp521r1) ECDSA key pair.
    EcdsaP521,
    /// secp256k1 ECDSA key pair.
    EcdsaSecp256k1,
    /// Ed25519 signature key pair.
    Ed25519,
    /// Ed448 signature key pair.
    Ed448,
    /// X25519 Diffie-Hellman key agreement key pair (KMIP 3.0).
    X25519,
    /// X448 Diffie-Hellman key agreement key pair (KMIP 3.0).
    X448,
    /// ML-KEM-512 post-quantum KEM key pair (KMIP 3.0).
    MlKem512,
    /// ML-KEM-768 post-quantum KEM key pair (KMIP 3.0).
    MlKem768,
    /// ML-KEM-1024 post-quantum KEM key pair (KMIP 3.0).
    MlKem1024,
    /// ML-DSA-44 post-quantum signature key pair (KMIP 3.0).
    MlDsa44,
    /// ML-DSA-65 post-quantum signature key pair (KMIP 3.0).
    MlDsa65,
    /// ML-DSA-87 post-quantum signature key pair (KMIP 3.0).
    MlDsa87,
    /// SLH-DSA-SHA2-128s post-quantum signature key pair (KMIP 3.0).
    SlhDsaSha2_128s,
    /// SLH-DSA-SHA2-128f post-quantum signature key pair (KMIP 3.0).
    SlhDsaSha2_128f,
    /// SLH-DSA-SHA2-192s post-quantum signature key pair (KMIP 3.0).
    SlhDsaSha2_192s,
    /// SLH-DSA-SHA2-192f post-quantum signature key pair (KMIP 3.0).
    SlhDsaSha2_192f,
    /// SLH-DSA-SHA2-256s post-quantum signature key pair (KMIP 3.0).
    SlhDsaSha2_256s,
    /// SLH-DSA-SHA2-256f post-quantum signature key pair (KMIP 3.0).
    SlhDsaSha2_256f,
    /// SLH-DSA-SHAKE-128s post-quantum signature key pair (KMIP 3.0).
    SlhDsaShake128s,
    /// SLH-DSA-SHAKE-128f post-quantum signature key pair (KMIP 3.0).
    SlhDsaShake128f,
    /// SLH-DSA-SHAKE-192s post-quantum signature key pair (KMIP 3.0).
    SlhDsaShake192s,
    /// SLH-DSA-SHAKE-192f post-quantum signature key pair (KMIP 3.0).
    SlhDsaShake192f,
    /// SLH-DSA-SHAKE-256s post-quantum signature key pair (KMIP 3.0).
    SlhDsaShake256s,
    /// SLH-DSA-SHAKE-256f post-quantum signature key pair (KMIP 3.0).
    SlhDsaShake256f,
    /// Hybrid X25519 + ML-KEM-768 KEM key pair (KMIP 3.0).
    X25519MlKem768,
    /// Hybrid secp256r1 + ML-KEM-768 KEM key pair (KMIP 3.0).
    Secp256r1MlKem768,
}

impl KeySpec {
    const fn meta(self) -> KeySpecMetadata {
        const V2_0: ProtocolVersion = ProtocolVersion::V2_0;
        const V3_0: ProtocolVersion = ProtocolVersion::V3_0;
        match self {
            Self::Aes128 => sym(cryptographic_algorithm::AES, 128, V2_0),
            Self::Aes192 => sym(cryptographic_algorithm::AES, 192, V2_0),
            Self::Aes256 => sym(cryptographic_algorithm::AES, 256, V2_0),
            Self::HmacSha256 => sym(cryptographic_algorithm::HMAC_SHA256, 256, V2_0),
            Self::HmacSha384 => sym(cryptographic_algorithm::HMAC_SHA384, 384, V2_0),
            Self::HmacSha512 => sym(cryptographic_algorithm::HMAC_SHA512, 512, V2_0),
            Self::ChaCha20 => sym(cryptographic_algorithm::CHACHA20, 256, V2_0),
            Self::ChaCha20Poly1305 => sym(cryptographic_algorithm::CHACHA20POLY1305, 256, V2_0),
            Self::Rsa2048 => asym(cryptographic_algorithm::RSA, 2048, None, V2_0),
            Self::Rsa3072 => asym(cryptographic_algorithm::RSA, 3072, None, V2_0),
            Self::Rsa4096 => asym(cryptographic_algorithm::RSA, 4096, None, V2_0),
            Self::EcdsaP256 => asym(
                cryptographic_algorithm::ECDSA,
                256,
                Some(recommended_curve::P_256),
                V2_0,
            ),
            Self::EcdsaP384 => asym(
                cryptographic_algorithm::ECDSA,
                384,
                Some(recommended_curve::P_384),
                V2_0,
            ),
            Self::EcdsaP521 => asym(
                cryptographic_algorithm::ECDSA,
                521,
                Some(recommended_curve::P_521),
                V2_0,
            ),
            Self::EcdsaSecp256k1 => asym(
                cryptographic_algorithm::ECDSA,
                256,
                Some(recommended_curve::SECP256K1),
                V2_0,
            ),
            Self::Ed25519 => asym(
                cryptographic_algorithm::ED25519,
                256,
                Some(recommended_curve::CURVE25519),
                V2_0,
            ),
            Self::Ed448 => asym(
                cryptographic_algorithm::ED448,
                448,
                Some(recommended_curve::CURVE448),
                V2_0,
            ),
            Self::X25519 => asym(
                v3_cryptographic_algorithm::X25519,
                256,
                Some(recommended_curve::CURVE25519),
                V3_0,
            ),
            Self::X448 => asym(
                v3_cryptographic_algorithm::X448,
                448,
                Some(recommended_curve::CURVE448),
                V3_0,
            ),
            other => other.pqc_meta(),
        }
    }

    const fn pqc_meta(self) -> KeySpecMetadata {
        const V3_0: ProtocolVersion = ProtocolVersion::V3_0;
        match self {
            Self::MlKem512 => asym(v3_cryptographic_algorithm::ML_KEM_512, 512, None, V3_0),
            Self::MlKem768 => asym(v3_cryptographic_algorithm::ML_KEM_768, 768, None, V3_0),
            Self::MlKem1024 => asym(v3_cryptographic_algorithm::ML_KEM_1024, 1024, None, V3_0),
            Self::MlDsa44 => asym(v3_cryptographic_algorithm::ML_DSA_44, 1312 * 8, None, V3_0),
            Self::MlDsa65 => asym(v3_cryptographic_algorithm::ML_DSA_65, 1952 * 8, None, V3_0),
            Self::MlDsa87 => asym(v3_cryptographic_algorithm::ML_DSA_87, 2592 * 8, None, V3_0),
            Self::SlhDsaSha2_128s => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHA2_128S,
                128,
                None,
                V3_0,
            ),
            Self::SlhDsaSha2_128f => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHA2_128F,
                128,
                None,
                V3_0,
            ),
            Self::SlhDsaSha2_192s => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHA2_192S,
                192,
                None,
                V3_0,
            ),
            Self::SlhDsaSha2_192f => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHA2_192F,
                192,
                None,
                V3_0,
            ),
            Self::SlhDsaSha2_256s => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHA2_256S,
                256,
                None,
                V3_0,
            ),
            Self::SlhDsaSha2_256f => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHA2_256F,
                256,
                None,
                V3_0,
            ),
            Self::SlhDsaShake128s => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHAKE_128S,
                128,
                None,
                V3_0,
            ),
            Self::SlhDsaShake128f => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHAKE_128F,
                128,
                None,
                V3_0,
            ),
            Self::SlhDsaShake192s => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHAKE_192S,
                192,
                None,
                V3_0,
            ),
            Self::SlhDsaShake192f => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHAKE_192F,
                192,
                None,
                V3_0,
            ),
            Self::SlhDsaShake256s => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHAKE_256S,
                256,
                None,
                V3_0,
            ),
            Self::SlhDsaShake256f => asym(
                v3_cryptographic_algorithm::SLH_DSA_SHAKE_256F,
                256,
                None,
                V3_0,
            ),
            Self::X25519MlKem768 => {
                asym(v3_cryptographic_algorithm::X25519MLKEM768, 768, None, V3_0)
            }
            Self::Secp256r1MlKem768 => asym(
                v3_cryptographic_algorithm::SECP256R1MLKEM768,
                768,
                Some(recommended_curve::P_256),
                V3_0,
            ),
            _ => unreachable!(),
        }
    }

    /// Returns true if this preset describes a symmetric key.
    #[must_use]
    pub const fn is_symmetric(self) -> bool {
        self.meta().symmetric
    }

    /// Returns true if this preset describes an asymmetric key pair.
    #[must_use]
    pub const fn is_asymmetric(self) -> bool {
        !self.meta().symmetric
    }

    /// Returns true if this preset requires KMIP 3.0.
    #[must_use]
    pub const fn requires_kmip_30(self) -> bool {
        self.meta().min_version.major() == 3
    }

    /// Returns the KMIP Cryptographic Algorithm value for this preset.
    #[must_use]
    pub const fn algorithm(self) -> CryptographicAlgorithm {
        CryptographicAlgorithm::from_raw(self.meta().algorithm)
    }

    /// Returns the KMIP Cryptographic Length in bits for this preset.
    #[must_use]
    pub const fn length(self) -> i32 {
        self.meta().length
    }

    /// Returns the raw KMIP Recommended Curve Enumeration value when applicable.
    #[must_use]
    pub const fn recommended_curve_raw(self) -> Option<u32> {
        self.meta().curve
    }

    /// Returns the minimum KMIP protocol version that defines this preset's algorithm.
    #[must_use]
    pub const fn min_kmip_version(self) -> ProtocolVersion {
        self.meta().min_version
    }
}

const fn sym(algorithm: u32, length: i32, min_version: ProtocolVersion) -> KeySpecMetadata {
    KeySpecMetadata {
        algorithm,
        length,
        curve: None,
        symmetric: true,
        min_version,
    }
}

const fn asym(
    algorithm: u32,
    length: i32,
    curve: Option<u32>,
    min_version: ProtocolVersion,
) -> KeySpecMetadata {
    KeySpecMetadata {
        algorithm,
        length,
        curve,
        symmetric: false,
        min_version,
    }
}

/// Curated encryption/decryption cipher parameter presets.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CipherSpec {
    /// AES-CBC with PKCS#5 padding.
    AesCbcPkcs5,
    /// AES-CBC without padding (caller handles block alignment).
    AesCbcNoPadding,
    /// AES-GCM with 96-bit IV, 128-bit authentication tag, and random server IV.
    AesGcm,
    /// AES-CTR with 128-bit IV and random server IV.
    AesCtr,
    /// ChaCha20-Poly1305 AEAD cipher.
    ChaCha20Poly1305,
    /// RSA-OAEP with SHA-256 and MGF1-SHA256.
    RsaOaepSha256,
    /// RSA-OAEP with SHA-384 and MGF1-SHA384.
    RsaOaepSha384,
    /// RSA-OAEP with SHA-512 and MGF1-SHA512.
    RsaOaepSha512,
    /// RSA PKCS#1 v1.5 encryption padding.
    RsaPkcs1v15,
}

impl CipherSpec {
    /// Converts the preset into a validated KMIP `CryptographicParameters` structure.
    ///
    /// # Errors
    /// Returns [`ProtocolError`] if parameter validation rejects a member.
    pub fn to_cryptographic_parameters(self) -> Result<CryptographicParameters, ProtocolError> {
        let builder = match self {
            Self::AesCbcPkcs5 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::AES)
                .with_block_cipher_mode(block_cipher_mode::CBC)
                .with_padding_method(padding_method::PKCS5),
            Self::AesCbcNoPadding => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::AES)
                .with_block_cipher_mode(block_cipher_mode::CBC),
            Self::AesGcm => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::AES)
                .with_block_cipher_mode(block_cipher_mode::GCM)
                .with_random_iv(true)
                .with_iv_length(96)
                .with_tag_length(128),
            Self::AesCtr => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::AES)
                .with_block_cipher_mode(block_cipher_mode::CTR)
                .with_random_iv(true)
                .with_iv_length(128),
            Self::ChaCha20Poly1305 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::CHACHA20POLY1305),
            Self::RsaOaepSha256 => rsa_oaep(hashing_algorithm::SHA_256),
            Self::RsaOaepSha384 => rsa_oaep(hashing_algorithm::SHA_384),
            Self::RsaOaepSha512 => rsa_oaep(hashing_algorithm::SHA_512),
            Self::RsaPkcs1v15 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::RSA)
                .with_padding_method(padding_method::PKCS1_V1_5),
        };
        builder.build()
    }

    /// Returns the minimum KMIP protocol version required for this cipher preset.
    #[must_use]
    pub const fn min_kmip_version(self) -> ProtocolVersion {
        ProtocolVersion::V2_0
    }
}

const fn rsa_oaep(hash: u32) -> CryptographicParametersBuilder {
    CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(cryptographic_algorithm::RSA)
        .with_padding_method(padding_method::OAEP)
        .with_hashing_algorithm(hash)
        .with_mask_generator(MASK_GENERATOR_MGF1)
        .with_mask_generator_hashing_algorithm(hash)
}

/// Curated digital signature and PQC signature presets.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SignatureSpec {
    /// ECDSA with SHA-256.
    EcdsaSha256,
    /// ECDSA with SHA-384.
    EcdsaSha384,
    /// ECDSA with SHA-512.
    EcdsaSha512,
    /// RSA-PSS with SHA-256.
    RsaPssSha256,
    /// RSA-PSS with SHA-384.
    RsaPssSha384,
    /// RSA-PSS with SHA-512.
    RsaPssSha512,
    /// RSA PKCS#1 v1.5 with SHA-256.
    RsaPkcs1Sha256,
    /// RSA PKCS#1 v1.5 with SHA-384.
    RsaPkcs1Sha384,
    /// RSA PKCS#1 v1.5 with SHA-512.
    RsaPkcs1Sha512,
    /// Ed25519 signature.
    Ed25519,
    /// Ed448 signature.
    Ed448,
    /// ML-DSA-44 signature (KMIP 3.0).
    MlDsa44,
    /// ML-DSA-65 signature (KMIP 3.0).
    MlDsa65,
    /// ML-DSA-87 signature (KMIP 3.0).
    MlDsa87,
    /// Deterministic ML-DSA-65 signature (KMIP 3.0).
    MlDsa65Deterministic,
    /// Stateless hash-based SLH-DSA signature (KMIP 3.0).
    SlhDsaSha2_128f,
}

impl SignatureSpec {
    /// Converts the preset into a validated KMIP `CryptographicParameters` structure.
    ///
    /// # Errors
    /// Returns [`ProtocolError`] if parameter construction fails.
    pub fn to_cryptographic_parameters(self) -> Result<CryptographicParameters, ProtocolError> {
        let builder = match self {
            Self::EcdsaSha256 => ecdsa_sig(
                digital_signature_algorithm::ECDSA_WITH_SHA256,
                hashing_algorithm::SHA_256,
            ),
            Self::EcdsaSha384 => ecdsa_sig(
                digital_signature_algorithm::ECDSA_WITH_SHA384,
                hashing_algorithm::SHA_384,
            ),
            Self::EcdsaSha512 => ecdsa_sig(
                digital_signature_algorithm::ECDSA_WITH_SHA512,
                hashing_algorithm::SHA_512,
            ),
            Self::RsaPssSha256 => rsa_pss(hashing_algorithm::SHA_256, 32),
            Self::RsaPssSha384 => rsa_pss(hashing_algorithm::SHA_384, 48),
            Self::RsaPssSha512 => rsa_pss(hashing_algorithm::SHA_512, 64),
            Self::RsaPkcs1Sha256 => rsa_pkcs1_sig(
                digital_signature_algorithm::SHA_256_WITH_RSA_ENCRYPTION,
                hashing_algorithm::SHA_256,
            ),
            Self::RsaPkcs1Sha384 => rsa_pkcs1_sig(
                digital_signature_algorithm::SHA_384_WITH_RSA_ENCRYPTION,
                hashing_algorithm::SHA_384,
            ),
            Self::RsaPkcs1Sha512 => rsa_pkcs1_sig(
                digital_signature_algorithm::SHA_512_WITH_RSA_ENCRYPTION,
                hashing_algorithm::SHA_512,
            ),
            Self::Ed25519 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::ED25519),
            Self::Ed448 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::ED448),
            Self::MlDsa44 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(v3_cryptographic_algorithm::ML_DSA_44),
            Self::MlDsa65 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(v3_cryptographic_algorithm::ML_DSA_65),
            Self::MlDsa87 => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(v3_cryptographic_algorithm::ML_DSA_87),
            Self::MlDsa65Deterministic => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(v3_cryptographic_algorithm::ML_DSA_65)
                .with_deterministic(true),
            Self::SlhDsaSha2_128f => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(v3_cryptographic_algorithm::SLH_DSA_SHA2_128F),
        };
        builder.build()
    }

    /// Returns the minimum KMIP protocol version for this signature preset.
    #[must_use]
    pub const fn min_kmip_version(self) -> ProtocolVersion {
        match self {
            Self::MlDsa44
            | Self::MlDsa65
            | Self::MlDsa87
            | Self::MlDsa65Deterministic
            | Self::SlhDsaSha2_128f => ProtocolVersion::V3_0,
            _ => ProtocolVersion::V2_0,
        }
    }
}

const fn ecdsa_sig(dsa: u32, hash: u32) -> CryptographicParametersBuilder {
    CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(cryptographic_algorithm::ECDSA)
        .with_digital_signature_algorithm(dsa)
        .with_hashing_algorithm(hash)
}

const fn rsa_pss(hash: u32, salt_length: i32) -> CryptographicParametersBuilder {
    CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(cryptographic_algorithm::RSA)
        .with_padding_method(padding_method::PSS)
        .with_hashing_algorithm(hash)
        .with_mask_generator(MASK_GENERATOR_MGF1)
        .with_mask_generator_hashing_algorithm(hash)
        .with_salt_length(salt_length)
}

const fn rsa_pkcs1_sig(dsa: u32, hash: u32) -> CryptographicParametersBuilder {
    CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(cryptographic_algorithm::RSA)
        .with_digital_signature_algorithm(dsa)
        .with_padding_method(padding_method::PKCS1_V1_5)
        .with_hashing_algorithm(hash)
}

/// Curated Message Authentication Code (MAC) presets.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MacSpec {
    /// HMAC with SHA-256.
    HmacSha256,
    /// HMAC with SHA-384.
    HmacSha384,
    /// HMAC with SHA-512.
    HmacSha512,
    /// HMAC with SHA3-256.
    HmacSha3_256,
    /// HMAC with SHA3-512.
    HmacSha3_512,
    /// AES-CMAC block cipher MAC.
    AesCmac,
}

impl MacSpec {
    /// Converts the preset into a validated KMIP `CryptographicParameters` structure.
    ///
    /// # Errors
    /// Returns [`ProtocolError`] if parameter construction fails.
    pub fn to_cryptographic_parameters(self) -> Result<CryptographicParameters, ProtocolError> {
        let builder = match self {
            Self::HmacSha256 => hmac_params(
                cryptographic_algorithm::HMAC_SHA256,
                hashing_algorithm::SHA_256,
            ),
            Self::HmacSha384 => hmac_params(
                cryptographic_algorithm::HMAC_SHA384,
                hashing_algorithm::SHA_384,
            ),
            Self::HmacSha512 => hmac_params(
                cryptographic_algorithm::HMAC_SHA512,
                hashing_algorithm::SHA_512,
            ),
            Self::HmacSha3_256 => hmac_params(
                cryptographic_algorithm::HMAC_SHA3_256,
                hashing_algorithm::SHA3_256,
            ),
            Self::HmacSha3_512 => hmac_params(
                cryptographic_algorithm::HMAC_SHA3_512,
                hashing_algorithm::SHA3_512,
            ),
            Self::AesCmac => CryptographicParametersBuilder::new()
                .with_cryptographic_algorithm(cryptographic_algorithm::AES)
                .with_block_cipher_mode(block_cipher_mode::CMAC),
        };
        builder.build()
    }
}

const fn hmac_params(alg: u32, hash: u32) -> CryptographicParametersBuilder {
    CryptographicParametersBuilder::new()
        .with_cryptographic_algorithm(alg)
        .with_hashing_algorithm(hash)
}

/// Curated KMIP hashing algorithm presets.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HashAlgorithm {
    /// SHA-256.
    Sha256,
    /// SHA-384.
    Sha384,
    /// SHA-512.
    Sha512,
    /// SHA3-256.
    Sha3_256,
    /// SHA3-384.
    Sha3_384,
    /// SHA3-512.
    Sha3_512,
}

impl HashAlgorithm {
    /// Returns the raw KMIP Hashing Algorithm Enumeration code.
    #[must_use]
    pub const fn raw(self) -> u32 {
        match self {
            Self::Sha256 => hashing_algorithm::SHA_256,
            Self::Sha384 => hashing_algorithm::SHA_384,
            Self::Sha512 => hashing_algorithm::SHA_512,
            Self::Sha3_256 => hashing_algorithm::SHA3_256,
            Self::Sha3_384 => hashing_algorithm::SHA3_384,
            Self::Sha3_512 => hashing_algorithm::SHA3_512,
        }
    }

    /// Returns the typed `HashingAlgorithm` wrapper.
    #[must_use]
    pub const fn to_hashing_algorithm(self) -> HashingAlgorithm {
        HashingAlgorithm::from_raw(self.raw())
    }

    /// Converts the algorithm into `CryptographicParameters` suitable for `Hash`.
    ///
    /// # Errors
    /// Returns [`ProtocolError`] if parameter validation fails.
    pub fn to_cryptographic_parameters(self) -> Result<CryptographicParameters, ProtocolError> {
        CryptographicParametersBuilder::new()
            .with_hashing_algorithm(self.raw())
            .build()
    }
}

/// Curated KMIP 3.0 Key Encapsulation Mechanism (KEM) presets.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KemSpec {
    /// ML-KEM-512 post-quantum KEM.
    MlKem512,
    /// ML-KEM-768 post-quantum KEM.
    MlKem768,
    /// ML-KEM-1024 post-quantum KEM.
    MlKem1024,
    /// Diffie-Hellman KEM (DHKEM).
    DhKem,
    /// RSA-KEM (RSASVE).
    RsaSve,
    /// Hybrid X25519 + ML-KEM-768 KEM.
    X25519MlKem768,
    /// Hybrid secp256r1 + ML-KEM-768 KEM.
    Secp256r1MlKem768,
}

impl KemSpec {
    /// Returns the KMIP 3.0 KEM Algorithm Enumeration code.
    #[must_use]
    pub const fn kem_algorithm_raw(self) -> u32 {
        match self {
            Self::MlKem512
            | Self::MlKem768
            | Self::MlKem1024
            | Self::X25519MlKem768
            | Self::Secp256r1MlKem768 => v3_kem_algorithm::MLKEM,
            Self::DhKem => v3_kem_algorithm::DHKEM,
            Self::RsaSve => v3_kem_algorithm::RSASVE,
        }
    }

    /// Returns the companion KMIP 3.0 Cryptographic Algorithm value when fixed by the preset.
    #[must_use]
    pub const fn cryptographic_algorithm(self) -> Option<CryptographicAlgorithm> {
        match self {
            Self::MlKem512 => Some(CryptographicAlgorithm::from_raw(
                v3_cryptographic_algorithm::ML_KEM_512,
            )),
            Self::MlKem768 => Some(CryptographicAlgorithm::from_raw(
                v3_cryptographic_algorithm::ML_KEM_768,
            )),
            Self::MlKem1024 => Some(CryptographicAlgorithm::from_raw(
                v3_cryptographic_algorithm::ML_KEM_1024,
            )),
            Self::X25519MlKem768 => Some(CryptographicAlgorithm::from_raw(
                v3_cryptographic_algorithm::X25519MLKEM768,
            )),
            Self::Secp256r1MlKem768 => Some(CryptographicAlgorithm::from_raw(
                v3_cryptographic_algorithm::SECP256R1MLKEM768,
            )),
            Self::DhKem | Self::RsaSve => None,
        }
    }

    /// Converts the preset into `CryptographicParameters` for KMIP 3.0 `Encapsulate`/`Decapsulate`.
    ///
    /// # Errors
    /// Returns [`ProtocolError`] if parameter validation fails.
    pub fn to_cryptographic_parameters(self) -> Result<CryptographicParameters, ProtocolError> {
        let mut builder =
            CryptographicParametersBuilder::new().with_kem_algorithm(self.kem_algorithm_raw());
        if let Some(alg) = self.cryptographic_algorithm() {
            builder = builder.with_cryptographic_algorithm(alg.raw());
        }
        builder.build()
    }

    /// Returns the minimum KMIP protocol version (3.0) required for KEM operations.
    #[must_use]
    pub const fn min_kmip_version(self) -> ProtocolVersion {
        ProtocolVersion::V3_0
    }
}

/// Curated Key Derivation Function (KDF) presets.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KdfSpec {
    /// HKDF with SHA-256.
    HkdfSha256,
    /// HKDF with SHA-384.
    HkdfSha384,
    /// HKDF with SHA-512.
    HkdfSha512,
    /// PBKDF2 with HMAC-SHA256.
    Pbkdf2Sha256,
    /// NIST SP 800-108 Counter Mode with HMAC-SHA256.
    Nist800_108CounterHmacSha256,
    /// KMIP HMAC-SHA256 derivation method.
    HmacSha256,
}

impl KdfSpec {
    /// Returns the KMIP Derivation Method Enumeration value.
    #[must_use]
    pub const fn derivation_method(self) -> DerivationMethod {
        let raw = match self {
            Self::HkdfSha256 | Self::HkdfSha384 | Self::HkdfSha512 => derivation_method::HKDF,
            Self::Pbkdf2Sha256 => derivation_method::PBKDF2,
            Self::Nist800_108CounterHmacSha256 => derivation_method::NIST800_108_C,
            Self::HmacSha256 => derivation_method::HMAC,
        };
        DerivationMethod::from_raw(raw)
    }

    /// Returns the KMIP Hashing Algorithm Enumeration associated with this KDF.
    #[must_use]
    pub const fn hash_algorithm(self) -> HashAlgorithm {
        match self {
            Self::HkdfSha384 => HashAlgorithm::Sha384,
            Self::HkdfSha512 => HashAlgorithm::Sha512,
            Self::HkdfSha256
            | Self::Pbkdf2Sha256
            | Self::Nist800_108CounterHmacSha256
            | Self::HmacSha256 => HashAlgorithm::Sha256,
        }
    }

    /// Builds typed `DerivationParameters` combining preset crypto parameters with optional salt/data/iterations.
    ///
    /// # Errors
    /// Returns [`ProtocolError`] if parameter construction fails.
    pub fn to_derivation_parameters(
        self,
        salt: Option<Vec<u8>>,
        derivation_data: Option<Vec<u8>>,
        iteration_count: Option<i32>,
    ) -> Result<DerivationParameters, ProtocolError> {
        let hash = self.hash_algorithm().raw();
        let crypto = CryptographicParametersBuilder::new()
            .with_hashing_algorithm(hash)
            .build()?;
        let mut params = DerivationParameters::new().with_typed_cryptographic_parameters(crypto);
        if let Some(salt) = salt {
            params = params.with_salt(SecretBytes::new(salt));
        }
        if let Some(data) = derivation_data {
            params = params.with_derivation_data(SecretBytes::new(data));
        }
        if let Some(count) = iteration_count {
            params = params.with_iteration_count(count);
        }
        Ok(params)
    }
}

/// Decoded RFC 7468 PEM block holding its boundary label and raw DER bytes.
#[derive(Clone, Eq, PartialEq)]
pub struct PemBlock {
    /// Boundary label between `-----BEGIN <label>-----` and `-----END <label>-----`.
    pub label: String,
    /// Decoded DER bytes.
    pub der: Vec<u8>,
}

impl fmt::Debug for PemBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PemBlock")
            .field("label", &self.label)
            .field("der_len", &self.der.len())
            .finish()
    }
}

/// Decodes the first PEM block found in `pem` according to RFC 7468 without external crates.
///
/// # Errors
/// Returns [`PresetError::EmptyInput`], [`PresetError::InvalidPem`], or [`PresetError::InvalidBase64`].
pub fn decode_pem(pem: &str) -> Result<PemBlock, PresetError> {
    if pem.trim().is_empty() {
        return Err(PresetError::EmptyInput);
    }
    let mut lines = pem.lines();
    let (label, body_lines) = extract_pem_bounds(&mut lines)?;
    let der = decode_base64_lines(&body_lines)?;
    if der.is_empty() {
        return Err(PresetError::InvalidPem("PEM block has no DER payload"));
    }
    Ok(PemBlock { label, der })
}

/// Decodes a PEM block and verifies its label matches one of `expected_labels`.
///
/// # Errors
/// Returns [`PresetError::InvalidPem`] if the label does not match, or any error from [`decode_pem`].
pub fn decode_pem_der(pem: &str, expected_labels: &[&str]) -> Result<Vec<u8>, PresetError> {
    let block = decode_pem(pem)?;
    if !expected_labels.is_empty()
        && !expected_labels
            .iter()
            .any(|allowed| *allowed == block.label)
    {
        return Err(PresetError::InvalidPem("unexpected PEM header label"));
    }
    Ok(block.der)
}

fn extract_pem_bounds<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
) -> Result<(String, Vec<&'a str>), PresetError> {
    let mut start_label: Option<&str> = None;
    for raw_line in &mut *lines {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix("-----BEGIN ")
            && let Some(label) = rest.strip_suffix("-----")
            && !label.is_empty()
        {
            start_label = Some(label);
            break;
        }
    }
    let Some(label) = start_label else {
        return Err(PresetError::InvalidPem(
            "missing -----BEGIN <LABEL>----- line",
        ));
    };
    let expected_end = format!("-----END {label}-----");
    let mut body = Vec::new();
    let mut found_end = false;
    for raw_line in lines {
        let line = raw_line.trim();
        if line == expected_end {
            found_end = true;
            break;
        }
        if line.starts_with("-----END ") {
            return Err(PresetError::InvalidPem("mismatched END label"));
        }
        if !line.is_empty() {
            body.push(line);
        }
    }
    if !found_end {
        return Err(PresetError::InvalidPem(
            "missing matching -----END <LABEL>----- line",
        ));
    }
    Ok((label.to_owned(), body))
}

fn decode_base64_lines(lines: &[&str]) -> Result<Vec<u8>, PresetError> {
    let mut clean = Vec::new();
    for line in lines {
        for byte in line.bytes() {
            if !byte.is_ascii_whitespace() {
                clean.push(byte);
            }
        }
    }
    if clean.is_empty() || !clean.len().is_multiple_of(4) {
        return Err(PresetError::InvalidBase64);
    }
    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    for (chunk_idx, chunk) in clean.chunks_exact(4).enumerate() {
        let is_last = chunk_idx + 1 == clean.len() / 4;
        decode_base64_quad(chunk, is_last, &mut out)?;
    }
    Ok(out)
}

fn decode_base64_quad(chunk: &[u8], is_last: bool, out: &mut Vec<u8>) -> Result<(), PresetError> {
    let a = b64_val(chunk[0]).ok_or(PresetError::InvalidBase64)?;
    let b = b64_val(chunk[1]).ok_or(PresetError::InvalidBase64)?;
    let c_pad = chunk[2] == b'=';
    let d_pad = chunk[3] == b'=';
    if (c_pad || d_pad) && !is_last {
        return Err(PresetError::InvalidBase64);
    }
    if c_pad && !d_pad {
        return Err(PresetError::InvalidBase64);
    }
    out.push((a << 2) | (b >> 4));
    match (c_pad, d_pad) {
        (true, true) => {
            if (b & 0x0F) != 0 {
                return Err(PresetError::InvalidBase64);
            }
        }
        (false, true) => {
            let c = b64_val(chunk[2]).ok_or(PresetError::InvalidBase64)?;
            if (c & 0x03) != 0 {
                return Err(PresetError::InvalidBase64);
            }
            out.push((b << 4) | (c >> 2));
        }
        (false, false) => {
            let c = b64_val(chunk[2]).ok_or(PresetError::InvalidBase64)?;
            let d = b64_val(chunk[3]).ok_or(PresetError::InvalidBase64)?;
            out.push((b << 4) | (c >> 2));
            out.push((c << 6) | d);
        }
        (true, false) => unreachable!(),
    }
    Ok(())
}

const fn b64_val(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn build_key_block(
    format_raw: u32,
    bytes: Vec<u8>,
    spec: Option<KeySpec>,
) -> Result<KeyBlock, PresetError> {
    let material = KeyMaterial::byte_string(format_raw, bytes)?;
    let key_value = KeyValue::try_new(material, None)?;
    let mut block = KeyBlock::try_new(KeyFormatType::from_raw(format_raw))?
        .with_key_value(KeyBlockValue::Plaintext(key_value));
    if let Some(spec) = spec {
        block =
            block.with_cryptographic_properties(Some(spec.algorithm().raw()), Some(spec.length()));
    }
    Ok(block)
}

impl ManagedObject {
    /// Constructs a raw `SymmetricKey` from a preset and exact plaintext key bytes.
    ///
    /// # Errors
    /// Returns [`PresetError::IncompatibleKeySpec`] if `spec` is asymmetric, or
    /// [`PresetError::InvalidKeyLength`] if `bytes.len()` does not match `spec.length() / 8`.
    pub fn symmetric_key_bytes(spec: KeySpec, bytes: Vec<u8>) -> Result<Self, PresetError> {
        if !spec.is_symmetric() {
            return Err(PresetError::IncompatibleKeySpec(
                "expected symmetric KeySpec",
            ));
        }
        let expected = usize::try_from(spec.length() / 8).unwrap_or(0);
        if bytes.len() != expected {
            return Err(PresetError::InvalidKeyLength {
                expected,
                actual: bytes.len(),
            });
        }
        let block = build_key_block(key_format_type::RAW, bytes, Some(spec))?;
        Ok(Self::SymmetricKey(SymmetricKey::try_new(block)?))
    }

    /// Constructs a PKCS#8 DER `PrivateKey` tagged with `spec` algorithm and bit length.
    ///
    /// # Errors
    /// Returns [`PresetError`] if `spec` is symmetric or `der` is empty.
    pub fn private_key_pkcs8_der(spec: KeySpec, der: Vec<u8>) -> Result<Self, PresetError> {
        if !spec.is_asymmetric() {
            return Err(PresetError::IncompatibleKeySpec(
                "expected asymmetric KeySpec",
            ));
        }
        if der.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        let block = build_key_block(key_format_type::PKCS_8, der, Some(spec))?;
        Ok(Self::PrivateKey(PrivateKey::try_new(block)?))
    }

    /// Decodes a `-----BEGIN PRIVATE KEY-----` PEM string into a PKCS#8 `PrivateKey`.
    ///
    /// # Errors
    /// Returns [`PresetError`] if PEM decoding fails or `spec` is not asymmetric.
    pub fn private_key_pkcs8_pem(spec: KeySpec, pem: &str) -> Result<Self, PresetError> {
        let der = decode_pem_der(pem, &["PRIVATE KEY"])?;
        Self::private_key_pkcs8_der(spec, der)
    }

    /// Constructs an X.509 `SubjectPublicKeyInfo` (SPKI) DER `PublicKey`.
    ///
    /// # Errors
    /// Returns [`PresetError`] if `spec` is symmetric or `der` is empty.
    pub fn public_key_spki_der(spec: KeySpec, der: Vec<u8>) -> Result<Self, PresetError> {
        if !spec.is_asymmetric() {
            return Err(PresetError::IncompatibleKeySpec(
                "expected asymmetric KeySpec",
            ));
        }
        if der.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        let block = build_key_block(key_format_type::X_509, der, Some(spec))?;
        Ok(Self::PublicKey(PublicKey::try_new(block)?))
    }

    /// Decodes a `-----BEGIN PUBLIC KEY-----` PEM string into an SPKI `PublicKey`.
    ///
    /// # Errors
    /// Returns [`PresetError`] if PEM decoding fails or `spec` is not asymmetric.
    pub fn public_key_spki_pem(spec: KeySpec, pem: &str) -> Result<Self, PresetError> {
        let der = decode_pem_der(pem, &["PUBLIC KEY"])?;
        Self::public_key_spki_der(spec, der)
    }

    /// Constructs an X.509 DER `Certificate`.
    ///
    /// # Errors
    /// Returns [`PresetError::EmptyInput`] if `der` is empty or [`PresetError::ManagedObject`] on error.
    pub fn certificate_x509_der(der: Vec<u8>) -> Result<Self, PresetError> {
        if der.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        Ok(Self::Certificate(Certificate::try_new(
            certificate_type::X_509,
            der,
        )?))
    }

    /// Decodes a `-----BEGIN CERTIFICATE-----` PEM string into an X.509 `Certificate`.
    ///
    /// # Errors
    /// Returns [`PresetError`] if PEM decoding fails.
    pub fn certificate_x509_pem(pem: &str) -> Result<Self, PresetError> {
        let der = decode_pem_der(pem, &["CERTIFICATE"])?;
        Self::certificate_x509_der(der)
    }

    /// Constructs a password `SecretData` managed object (omitting Key Block crypto properties per §3.2).
    ///
    /// # Errors
    /// Returns [`PresetError`] if `bytes` is empty or TTLV construction fails.
    pub fn secret_data_password(bytes: Vec<u8>) -> Result<Self, PresetError> {
        build_secret_data(secret_data_type::PASSWORD, bytes)
    }

    /// Constructs a seed `SecretData` managed object (omitting Key Block crypto properties per §3.2).
    ///
    /// # Errors
    /// Returns [`PresetError`] if `bytes` is empty or TTLV construction fails.
    pub fn secret_data_seed(bytes: Vec<u8>) -> Result<Self, PresetError> {
        build_secret_data(secret_data_type::SEED, bytes)
    }

    /// Constructs an `OpaqueObject` with the given KMIP Opaque Data Type and bytes.
    ///
    /// # Errors
    /// Returns [`PresetError::EmptyInput`] if `data` is empty or [`PresetError::ManagedObject`] on error.
    pub fn opaque(opaque_data_type: u32, data: Vec<u8>) -> Result<Self, PresetError> {
        if data.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        Ok(Self::OpaqueObject(OpaqueObject::try_new(
            opaque_data_type,
            data,
        )?))
    }
}

fn build_secret_data(data_type: u32, bytes: Vec<u8>) -> Result<ManagedObject, PresetError> {
    if bytes.is_empty() {
        return Err(PresetError::EmptyInput);
    }
    let block = build_key_block(key_format_type::RAW, bytes, None)?;
    Ok(ManagedObject::SecretData(SecretData::try_new(
        data_type, block,
    )?))
}

fn ttlv_item(raw_tag: u32, value: Value) -> Result<Item, PresetError> {
    let tag: Tag = RawTag::new(raw_tag)
        .and_then(|raw| raw.try_checked())
        .map_err(|_| PresetError::InvalidTtlv)?;
    Item::new(tag, value).map_err(|_| PresetError::InvalidTtlv)
}

fn build_credential(
    credential_type: CredentialType,
    value_fields: Vec<Item>,
) -> Result<Credential, PresetError> {
    let mut val_struct = Structure::new();
    for field in value_fields {
        val_struct
            .try_push(field)
            .map_err(|_| PresetError::InvalidTtlv)?;
    }
    let mut cred_struct = Structure::new();
    cred_struct
        .try_push(ttlv_item(
            CREDENTIAL_TYPE_TAG,
            Value::enumeration(credential_type.raw()),
        )?)
        .map_err(|_| PresetError::InvalidTtlv)?;
    cred_struct
        .try_push(ttlv_item(
            CREDENTIAL_VALUE_TAG,
            Value::structure(val_struct),
        )?)
        .map_err(|_| PresetError::InvalidTtlv)?;
    Ok(Credential::try_from_ttlv(cred_struct)?)
}

impl Authentication {
    /// Creates an [`Authentication`] header carrying a Username (and optional Password) credential.
    ///
    /// # Errors
    /// Returns [`PresetError`] if `username` is empty or TTLV/credential validation fails.
    pub fn username_password(username: &str, password: Option<&str>) -> Result<Self, PresetError> {
        if username.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        let mut fields = vec![ttlv_item(
            USERNAME_TAG,
            Value::text_string(username.to_owned()),
        )?];
        if let Some(password) = password {
            fields.push(ttlv_item(
                PASSWORD_TAG,
                Value::text_string(password.to_owned()),
            )?);
        }
        let cred = build_credential(CredentialType::UsernameAndPassword, fields)?;
        Ok(Self::new(vec![cred])?)
    }

    /// Creates an [`Authentication`] header carrying a KMIP Device credential.
    ///
    /// # Errors
    /// Returns [`PresetError`] if no identifier field is provided or validation fails.
    pub fn device(
        device_serial_number: Option<&str>,
        password: Option<&str>,
        device_identifier: Option<&str>,
        network_identifier: Option<&str>,
        machine_identifier: Option<&str>,
        media_identifier: Option<&str>,
    ) -> Result<Self, PresetError> {
        let mut fields = Vec::new();
        for (tag, opt) in [
            (DEVICE_SERIAL_NUMBER_TAG, device_serial_number),
            (PASSWORD_TAG, password),
            (DEVICE_IDENTIFIER_TAG, device_identifier),
            (NETWORK_IDENTIFIER_TAG, network_identifier),
            (MACHINE_IDENTIFIER_TAG, machine_identifier),
            (MEDIA_IDENTIFIER_TAG, media_identifier),
        ] {
            if let Some(text) = opt {
                fields.push(ttlv_item(tag, Value::text_string(text.to_owned()))?);
            }
        }
        if fields.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        let cred = build_credential(CredentialType::Device, fields)?;
        Ok(Self::new(vec![cred])?)
    }

    /// Creates an [`Authentication`] header carrying a Ticket credential.
    ///
    /// # Errors
    /// Returns [`PresetError`] if `ticket_value` is empty or validation fails.
    pub fn ticket(ticket_type: u32, ticket_value: Vec<u8>) -> Result<Self, PresetError> {
        if ticket_value.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        let mut ticket_fields = Structure::new();
        ticket_fields
            .try_push(ttlv_item(TICKET_TYPE_TAG, Value::enumeration(ticket_type))?)
            .map_err(|_| PresetError::InvalidTtlv)?;
        ticket_fields
            .try_push(ttlv_item(
                TICKET_VALUE_TAG,
                Value::byte_string(ticket_value),
            )?)
            .map_err(|_| PresetError::InvalidTtlv)?;
        let cred = build_credential(
            CredentialType::Ticket,
            vec![ttlv_item(TICKET_TAG, Value::structure(ticket_fields))?],
        )?;
        Ok(Self::new(vec![cred])?)
    }

    /// Creates an [`Authentication`] header carrying a pre-hashed password credential (Table 415).
    ///
    /// # Errors
    /// Returns [`PresetError`] if `username` or `hashed_password` is empty or validation fails.
    pub fn hashed_password(
        username: &str,
        timestamp_unix_seconds: i64,
        hashing_algorithm: HashAlgorithm,
        hashed_password: Vec<u8>,
    ) -> Result<Self, PresetError> {
        if username.is_empty() || hashed_password.is_empty() {
            return Err(PresetError::EmptyInput);
        }
        let cred = build_credential(
            CredentialType::HashedPassword,
            vec![
                ttlv_item(USERNAME_TAG, Value::text_string(username.to_owned()))?,
                ttlv_item(
                    TIME_STAMP_TAG,
                    Value::date_time_extended(timestamp_unix_seconds),
                )?,
                ttlv_item(
                    HASHING_ALGORITHM_TAG,
                    Value::enumeration(hashing_algorithm.raw()),
                )?,
                ttlv_item(HASHED_PASSWORD_TAG, Value::byte_string(hashed_password))?,
            ],
        )?;
        Ok(Self::new(vec![cred])?)
    }
}

/// High-level, version-aware builder for KMIP `AttributeSet` instances.
///
/// Handles structured attributes (`Name`, `AlternativeName`, `Link`), typed
/// flags/dates/intervals, and automatic KMIP 2.x/3.0 link normalization.
#[derive(Default)]
pub struct AttributesBuilder {
    items: Vec<Item>,
    pending_links: Vec<(LinkType, String)>,
    error: Option<PresetError>,
}

impl fmt::Debug for AttributesBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AttributesBuilder")
            .field("item_count", &self.items.len())
            .field("link_count", &self.pending_links.len())
            .field("has_error", &self.error.is_some())
            .finish()
    }
}

impl AttributesBuilder {
    /// Starts an empty attribute builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            items: Vec::new(),
            pending_links: Vec::new(),
            error: None,
        }
    }

    fn push_typed<D: AttributeDefinition>(mut self, value: D::Value) -> Self {
        if self.error.is_some() {
            return self;
        }
        match Attribute::<D>::new(value) {
            Ok(attr) => self.items.push(attr.into_item()),
            Err(err) => self.error = Some(PresetError::Attribute(err)),
        }
        self
    }

    /// Populates `CryptographicAlgorithm`, `CryptographicLength`, and optional `RecommendedCurve` from `spec`.
    #[must_use]
    pub fn key_spec(mut self, spec: KeySpec) -> Self {
        self = self
            .cryptographic_algorithm(spec.algorithm().raw())
            .cryptographic_length(spec.length());
        if let Some(curve) = spec.recommended_curve_raw() {
            self = self.recommended_curve(curve);
        }
        self
    }

    /// Sets the `CryptographicAlgorithm` attribute.
    #[must_use]
    pub fn cryptographic_algorithm(self, algorithm_raw: u32) -> Self {
        self.push_typed::<typed::CryptographicAlgorithm>(algorithm_raw)
    }

    /// Sets the `CryptographicLength` attribute in bits.
    #[must_use]
    pub fn cryptographic_length(self, bits: i32) -> Self {
        self.push_typed::<typed::CryptographicLength>(bits)
    }

    /// Sets the `CryptographicDomainParameters` attribute containing `RecommendedCurve`.
    #[must_use]
    pub fn recommended_curve(mut self, curve_raw: u32) -> Self {
        if self.error.is_some() {
            return self;
        }
        let mut s = Structure::new();
        let built = (|| -> Result<(), PresetError> {
            s.try_push(ttlv_item(
                RECOMMENDED_CURVE_TAG,
                Value::enumeration(curve_raw),
            )?)
            .map_err(|_| PresetError::InvalidTtlv)?;
            Ok(())
        })();
        match built {
            Ok(()) => self.push_typed::<typed::CryptographicDomainParameters>(s),
            Err(err) => {
                self.error = Some(err);
                self
            }
        }
    }

    /// Sets the `CryptographicUsageMask` attribute.
    #[must_use]
    pub fn usage_mask(self, mask: i32) -> Self {
        self.push_typed::<typed::CryptographicUsageMask>(mask)
    }

    /// Appends a KMIP `Name` attribute using Uninterpreted Text String (type 1).
    #[must_use]
    pub fn name(self, value: impl Into<String>) -> Self {
        self.name_with_type(value, NAME_TYPE_UNINTERPRETED_TEXT_STRING)
    }

    /// Appends a KMIP `Name` attribute with an explicit KMIP Name Type enumeration.
    #[must_use]
    pub fn name_with_type(mut self, value: impl Into<String>, name_type: u32) -> Self {
        if self.error.is_some() {
            return self;
        }
        let mut s = Structure::new();
        let built = (|| -> Result<(), PresetError> {
            s.try_push(ttlv_item(NAME_VALUE_TAG, Value::text_string(value.into()))?)
                .map_err(|_| PresetError::InvalidTtlv)?;
            s.try_push(ttlv_item(NAME_TYPE_TAG, Value::enumeration(name_type))?)
                .map_err(|_| PresetError::InvalidTtlv)?;
            Ok(())
        })();
        match built {
            Ok(()) => self.push_typed::<typed::Name>(s),
            Err(err) => {
                self.error = Some(err);
                self
            }
        }
    }

    /// Appends a KMIP `AlternativeName` attribute.
    #[must_use]
    pub fn alternative_name(mut self, value: impl Into<String>, alt_name_type: u32) -> Self {
        if self.error.is_some() {
            return self;
        }
        let mut s = Structure::new();
        let built = (|| -> Result<(), PresetError> {
            s.try_push(ttlv_item(
                ALTERNATIVE_NAME_VALUE_TAG,
                Value::text_string(value.into()),
            )?)
            .map_err(|_| PresetError::InvalidTtlv)?;
            s.try_push(ttlv_item(
                ALTERNATIVE_NAME_TYPE_TAG,
                Value::enumeration(alt_name_type),
            )?)
            .map_err(|_| PresetError::InvalidTtlv)?;
            Ok(())
        })();
        match built {
            Ok(()) => self.push_typed::<typed::AlternativeName>(s),
            Err(err) => {
                self.error = Some(err);
                self
            }
        }
    }

    /// Appends a `Description` attribute.
    #[must_use]
    pub fn description(self, text: impl Into<String>) -> Self {
        self.push_typed::<typed::Description>(text.into())
    }

    /// Appends a `Comment` attribute.
    #[must_use]
    pub fn comment(self, text: impl Into<String>) -> Self {
        self.push_typed::<typed::Comment>(text.into())
    }

    /// Appends an `ObjectGroup` attribute.
    #[must_use]
    pub fn object_group(self, group: impl Into<String>) -> Self {
        self.push_typed::<typed::ObjectGroup>(group.into())
    }

    /// Sets the `ActivationDate` attribute (Unix timestamp seconds).
    #[must_use]
    pub fn activation_date(self, unix_seconds: i64) -> Self {
        self.push_typed::<typed::ActivationDate>(unix_seconds)
    }

    /// Sets the `DeactivationDate` attribute (Unix timestamp seconds).
    #[must_use]
    pub fn deactivation_date(self, unix_seconds: i64) -> Self {
        self.push_typed::<typed::DeactivationDate>(unix_seconds)
    }

    /// Sets the `ProcessStartDate` attribute (Unix timestamp seconds).
    #[must_use]
    pub fn process_start_date(self, unix_seconds: i64) -> Self {
        self.push_typed::<typed::ProcessStartDate>(unix_seconds)
    }

    /// Sets the `ProtectStopDate` attribute (Unix timestamp seconds).
    #[must_use]
    pub fn protect_stop_date(self, unix_seconds: i64) -> Self {
        self.push_typed::<typed::ProtectStopDate>(unix_seconds)
    }

    /// Sets the `LeaseTime` interval in seconds.
    #[must_use]
    pub fn lease_time(self, seconds: u32) -> Self {
        self.push_typed::<typed::LeaseTime>(seconds)
    }

    /// Sets the `Sensitive` boolean attribute.
    #[must_use]
    pub fn sensitive(self, value: bool) -> Self {
        self.push_typed::<typed::Sensitive>(value)
    }

    /// Sets the `Extractable` boolean attribute.
    #[must_use]
    pub fn extractable(self, value: bool) -> Self {
        self.push_typed::<typed::Extractable>(value)
    }

    /// Sets the `AlwaysSensitive` boolean attribute.
    #[must_use]
    pub fn always_sensitive(self, value: bool) -> Self {
        self.push_typed::<typed::AlwaysSensitive>(value)
    }

    /// Sets the `NeverExtractable` boolean attribute.
    #[must_use]
    pub fn never_extractable(self, value: bool) -> Self {
        self.push_typed::<typed::NeverExtractable>(value)
    }

    /// Sets the `QuantumSafe` boolean attribute.
    #[must_use]
    pub fn quantum_safe(self, value: bool) -> Self {
        self.push_typed::<typed::QuantumSafe>(value)
    }

    /// Sets the `ProtectionLevel` enumeration attribute.
    #[must_use]
    pub fn protection_level(self, level: u32) -> Self {
        self.push_typed::<typed::ProtectionLevel>(level)
    }

    /// Sets the `ProtectionPeriod` interval attribute in seconds.
    #[must_use]
    pub fn protection_period(self, seconds: u32) -> Self {
        self.push_typed::<typed::ProtectionPeriod>(seconds)
    }

    /// Sets the `ProtectionStorageMask` integer mask attribute.
    #[must_use]
    pub fn protection_storage_mask(self, mask: i32) -> Self {
        self.push_typed::<typed::ProtectionStorageMask>(mask)
    }

    /// Queues a relationship link normalized at build time for the target KMIP version.
    #[must_use]
    pub fn link(mut self, link_type: LinkType, target_id: impl Into<String>) -> Self {
        self.pending_links.push((link_type, target_id.into()));
        self
    }

    /// Appends any pre-constructed typed `Attribute<D>`.
    #[must_use]
    pub fn typed_attribute<D: AttributeDefinition>(mut self, attribute: Attribute<D>) -> Self {
        self.items.push(attribute.into_item());
        self
    }

    /// Appends a raw TTLV `Item` attribute.
    #[must_use]
    pub fn raw_item(mut self, item: Item) -> Self {
        self.items.push(item);
        self
    }

    /// Finalizes the [`AttributeSet`] using KMIP 2.1 link encoding.
    ///
    /// # Errors
    /// Returns [`PresetError`] if any queued attribute or link failed validation.
    pub fn build(self) -> Result<AttributeSet, PresetError> {
        self.build_for_version(ProtocolVersion::V2_1)
    }

    /// Finalizes the [`AttributeSet`] while adapting links to `version` (KMIP 2.x `Link` vs KMIP 3.0 typed link attributes).
    ///
    /// # Errors
    /// Returns [`PresetError`] if any queued attribute or link is invalid for `version`.
    pub fn build_for_version(self, version: ProtocolVersion) -> Result<AttributeSet, PresetError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        let mut items = self.items;
        for (link_type, target_id) in self.pending_links {
            items.push(encode_link_for_version(link_type, &target_id, version)?);
        }
        let mut set = AttributeSet::new();
        for item in items {
            set.try_push(item).map_err(|_| PresetError::InvalidTtlv)?;
        }
        Ok(set)
    }
}

fn encode_link_for_version(
    link_type: LinkType,
    target_id: &str,
    version: ProtocolVersion,
) -> Result<Item, PresetError> {
    if version.major() >= 3 {
        if link_type.try_v2_enumeration().is_some() {
            let mut s = Structure::new();
            let enum_val = link_type.try_v2_enumeration().unwrap_or(0x0000_0101);
            s.try_push(ttlv_item(LINK_TYPE_TAG, Value::enumeration(enum_val))?)
                .map_err(|_| PresetError::InvalidTtlv)?;
            s.try_push(ttlv_item(
                LINKED_OBJECT_IDENTIFIER_TAG,
                Value::identifier(target_id.to_owned()),
            )?)
            .map_err(|_| PresetError::InvalidTtlv)?;
            let v2_link = Attribute::<typed::Link>::new(s)?.into_item();
            return LinkNormalizer::normalize_to_v3(&v2_link).map_err(PresetError::Normalization);
        }
        return ttlv_item(link_type.v3_tag(), Value::identifier(target_id.to_owned()));
    }
    let enum_val = link_type.try_v2_enumeration().ok_or_else(|| {
        PresetError::Normalization("selected LinkType is only defined in KMIP 3.0+".to_owned())
    })?;
    let mut s = Structure::new();
    s.try_push(ttlv_item(LINK_TYPE_TAG, Value::enumeration(enum_val))?)
        .map_err(|_| PresetError::InvalidTtlv)?;
    s.try_push(ttlv_item(
        LINKED_OBJECT_IDENTIFIER_TAG,
        Value::identifier(target_id.to_owned()),
    )?)
    .map_err(|_| PresetError::InvalidTtlv)?;
    Ok(Attribute::<typed::Link>::new(s)?.into_item())
}
