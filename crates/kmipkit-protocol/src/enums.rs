//! Strongly typed multi-enum definitions for KMIP attributes, links, operations,
//! states, and algorithms. Discriminants are derived directly from [`KmipTag`].

use kmipkit_ttlv::{KmipTag, Tag};

/// Dedicated enum for KMIP object attributes catalogued across KMIP 2.0, 2.1, and 3.0.
///
/// Values are mapped directly from [`KmipTag`] ensuring a single source of truth.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
#[non_exhaustive]
pub enum AttributeTag {
    /// `ActivationDate` attribute tag
    ActivationDate = KmipTag::ActivationDate as u32,
    /// `ApplicationSpecificInformation` attribute tag
    ApplicationSpecificInformation = KmipTag::ApplicationSpecificInformation as u32,
    /// `ArchiveDate` attribute tag
    ArchiveDate = KmipTag::ArchiveDate as u32,
    /// `Attribute` attribute tag
    Attribute = KmipTag::Attribute as u32,
    /// `CertificateType` attribute tag
    CertificateType = KmipTag::CertificateType as u32,
    /// `CompromiseDate` attribute tag
    CompromiseDate = KmipTag::CompromiseDate as u32,
    /// `CompromiseOccurrenceDate` attribute tag
    CompromiseOccurrenceDate = KmipTag::CompromiseOccurrenceDate as u32,
    /// `ContactInformation` attribute tag
    ContactInformation = KmipTag::ContactInformation as u32,
    /// `CredentialType` attribute tag
    CredentialType = KmipTag::CredentialType as u32,
    /// `CryptographicAlgorithm` attribute tag
    CryptographicAlgorithm = KmipTag::CryptographicAlgorithm as u32,
    /// `CryptographicDomainParameters` attribute tag
    CryptographicDomainParameters = KmipTag::CryptographicDomainParameters as u32,
    /// `CryptographicLength` attribute tag
    CryptographicLength = KmipTag::CryptographicLength as u32,
    /// `CryptographicParameters` attribute tag
    CryptographicParameters = KmipTag::CryptographicParameters as u32,
    /// `CryptographicUsageMask` attribute tag
    CryptographicUsageMask = KmipTag::CryptographicUsageMask as u32,
    /// `DeactivationDate` attribute tag
    DeactivationDate = KmipTag::DeactivationDate as u32,
    /// `DestroyDate` attribute tag
    DestroyDate = KmipTag::DestroyDate as u32,
    /// `Digest` attribute tag
    Digest = KmipTag::Digest as u32,
    /// `InitialDate` attribute tag
    InitialDate = KmipTag::InitialDate as u32,
    /// `KeyFormatType` attribute tag
    KeyFormatType = KmipTag::KeyFormatType as u32,
    /// `KeyPartIdentifier` attribute tag
    KeyPartIdentifier = KmipTag::KeyPartIdentifier as u32,
    /// `LastChangeDate` attribute tag
    LastChangeDate = KmipTag::LastChangeDate as u32,
    /// `LeaseTime` attribute tag
    LeaseTime = KmipTag::LeaseTime as u32,
    /// `Link` attribute tag
    Link = KmipTag::Link as u32,
    /// `Name` attribute tag
    Name = KmipTag::Name as u32,
    /// `ObjectGroup` attribute tag
    ObjectGroup = KmipTag::ObjectGroup as u32,
    /// `ObjectType` attribute tag
    ObjectType = KmipTag::ObjectType as u32,
    /// `OpaqueDataType` attribute tag
    OpaqueDataType = KmipTag::OpaqueDataType as u32,
    /// `ProcessStartDate` attribute tag
    ProcessStartDate = KmipTag::ProcessStartDate as u32,
    /// `ProtectStopDate` attribute tag
    ProtectStopDate = KmipTag::ProtectStopDate as u32,
    /// `RevocationReason` attribute tag
    RevocationReason = KmipTag::RevocationReason as u32,
    /// `SplitKeyMethod` attribute tag
    SplitKeyMethod = KmipTag::SplitKeyMethod as u32,
    /// `SplitKeyParts` attribute tag
    SplitKeyParts = KmipTag::SplitKeyParts as u32,
    /// `SplitKeyThreshold` attribute tag
    SplitKeyThreshold = KmipTag::SplitKeyThreshold as u32,
    /// `State` attribute tag
    State = KmipTag::State as u32,
    /// `UniqueIdentifier` attribute tag
    UniqueIdentifier = KmipTag::UniqueIdentifier as u32,
    /// `UsageLimits` attribute tag
    UsageLimits = KmipTag::UsageLimits as u32,
    /// `Fresh` attribute tag
    Fresh = KmipTag::Fresh as u32,
    /// `CertificateLength` attribute tag
    CertificateLength = KmipTag::CertificateLength as u32,
    /// `DigitalSignatureAlgorithm` attribute tag
    DigitalSignatureAlgorithm = KmipTag::DigitalSignatureAlgorithm as u32,
    /// `X509CertificateIdentifier` attribute tag
    X509CertificateIdentifier = KmipTag::X509CertificateIdentifier as u32,
    /// `X509CertificateIssuer` attribute tag
    X509CertificateIssuer = KmipTag::X509CertificateIssuer as u32,
    /// `X509CertificateSubject` attribute tag
    X509CertificateSubject = KmipTag::X509CertificateSubject as u32,
    /// `KeyValueLocation` attribute tag
    KeyValueLocation = KmipTag::KeyValueLocation as u32,
    /// `KeyValuePresent` attribute tag
    KeyValuePresent = KmipTag::KeyValuePresent as u32,
    /// `OriginalCreationDate` attribute tag
    OriginalCreationDate = KmipTag::OriginalCreationDate as u32,
    /// `AlternativeName` attribute tag
    AlternativeName = KmipTag::AlternativeName as u32,
    /// `RandomNumberGenerator` attribute tag
    RandomNumberGenerator = KmipTag::RandomNumberGenerator as u32,
    /// `PKCS12FriendlyName` attribute tag
    PKCS12FriendlyName = KmipTag::PKCS12FriendlyName as u32,
    /// `Description` attribute tag
    Description = KmipTag::Description as u32,
    /// `Comment` attribute tag
    Comment = KmipTag::Comment as u32,
    /// `Sensitive` attribute tag
    Sensitive = KmipTag::Sensitive as u32,
    /// `AlwaysSensitive` attribute tag
    AlwaysSensitive = KmipTag::AlwaysSensitive as u32,
    /// `Extractable` attribute tag
    Extractable = KmipTag::Extractable as u32,
    /// `NeverExtractable` attribute tag
    NeverExtractable = KmipTag::NeverExtractable as u32,
    /// `ShortUniqueIdentifier` attribute tag
    ShortUniqueIdentifier = KmipTag::ShortUniqueIdentifier as u32,
    /// `NISTKeyType` attribute tag
    NISTKeyType = KmipTag::NISTKeyType as u32,
    /// `ProtectionLevel` attribute tag
    ProtectionLevel = KmipTag::ProtectionLevel as u32,
    /// `ProtectionPeriod` attribute tag
    ProtectionPeriod = KmipTag::ProtectionPeriod as u32,
    /// `QuantumSafe` attribute tag
    QuantumSafe = KmipTag::QuantumSafe as u32,
    /// `ProtectionStorageMask` attribute tag
    ProtectionStorageMask = KmipTag::ProtectionStorageMask as u32,
    /// `RotateInterval` attribute tag
    RotateInterval = KmipTag::RotateInterval as u32,
    /// `RotateAutomatic` attribute tag
    RotateAutomatic = KmipTag::RotateAutomatic as u32,
    /// `RotateOffset` attribute tag
    RotateOffset = KmipTag::RotateOffset as u32,
    /// `RotateDate` attribute tag
    RotateDate = KmipTag::RotateDate as u32,
    /// `RotateGeneration` attribute tag
    RotateGeneration = KmipTag::RotateGeneration as u32,
    /// `RotateName` attribute tag
    RotateName = KmipTag::RotateName as u32,
    /// `RotateLatest` attribute tag
    RotateLatest = KmipTag::RotateLatest as u32,
    /// `CertificateLink` attribute tag
    CertificateLink = KmipTag::CertificateLink as u32,
    /// `ChildLink` attribute tag
    ChildLink = KmipTag::ChildLink as u32,
    /// `DerivationBaseObjectLink` attribute tag
    DerivationBaseObjectLink = KmipTag::DerivationBaseObjectLink as u32,
    /// `DerivedObjectLink` attribute tag
    DerivedObjectLink = KmipTag::DerivedObjectLink as u32,
    /// `NextLink` attribute tag
    NextLink = KmipTag::NextLink as u32,
    /// `ParentLink` attribute tag
    ParentLink = KmipTag::ParentLink as u32,
    /// `PKCS12CertificateLink` attribute tag
    PKCS12CertificateLink = KmipTag::PKCS12CertificateLink as u32,
    /// `PKCS12PasswordLink` attribute tag
    PKCS12PasswordLink = KmipTag::PKCS12PasswordLink as u32,
    /// `PreviousLink` attribute tag
    PreviousLink = KmipTag::PreviousLink as u32,
    /// `PrivateKeyLink` attribute tag
    PrivateKeyLink = KmipTag::PrivateKeyLink as u32,
    /// `PublicKeyLink` attribute tag
    PublicKeyLink = KmipTag::PublicKeyLink as u32,
    /// `ReplacedObjectLink` attribute tag
    ReplacedObjectLink = KmipTag::ReplacedObjectLink as u32,
    /// `ReplacementObjectLink` attribute tag
    ReplacementObjectLink = KmipTag::ReplacementObjectLink as u32,
    /// `WrappingKeyLink` attribute tag
    WrappingKeyLink = KmipTag::WrappingKeyLink as u32,
    /// `ObjectClass` attribute tag
    ObjectClass = KmipTag::ObjectClass as u32,
    /// `CredentialLink` attribute tag
    CredentialLink = KmipTag::CredentialLink as u32,
    /// `PasswordLink` attribute tag
    PasswordLink = KmipTag::PasswordLink as u32,
    /// `OTPCounter` attribute tag
    OTPCounter = KmipTag::OTPCounter as u32,
    /// `GroupLink` attribute tag
    GroupLink = KmipTag::GroupLink as u32,
    /// `SplitKeyBaseLink` attribute tag
    SplitKeyBaseLink = KmipTag::SplitKeyBaseLink as u32,
    /// `JoinedSplitKeyPartsLink` attribute tag
    JoinedSplitKeyPartsLink = KmipTag::JoinedSplitKeyPartsLink as u32,
    /// `SplitKeyPolynomial` attribute tag
    SplitKeyPolynomial = KmipTag::SplitKeyPolynomial as u32,
    /// `DeactivationReason` attribute tag
    DeactivationReason = KmipTag::DeactivationReason as u32,
    /// `CertificateSubjectDN` attribute tag
    CertificateSubjectDN = KmipTag::CertificateSubjectDN as u32,
    /// `CertificateIssuerDN` attribute tag
    CertificateIssuerDN = KmipTag::CertificateIssuerDN as u32,
    /// `CertificateRequestLink` attribute tag
    CertificateRequestLink = KmipTag::CertificateRequestLink as u32,
    /// `CertifyCounter` attribute tag
    CertifyCounter = KmipTag::CertifyCounter as u32,
    /// `DecryptCounter` attribute tag
    DecryptCounter = KmipTag::DecryptCounter as u32,
    /// `EncryptCounter` attribute tag
    EncryptCounter = KmipTag::EncryptCounter as u32,
    /// `SignCounter` attribute tag
    SignCounter = KmipTag::SignCounter as u32,
    /// `SignatureVerifyCounter` attribute tag
    SignatureVerifyCounter = KmipTag::SignatureVerifyCounter as u32,
    /// `NISTSecurityCategory` attribute tag
    NISTSecurityCategory = KmipTag::NISTSecurityCategory as u32,
}

impl AttributeTag {
    /// Returns the 24-bit raw tag value.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Converts this attribute tag to the corresponding [`KmipTag`].
    #[inline]
    #[must_use]
    pub const fn as_kmip_tag(self) -> KmipTag {
        match KmipTag::from_u32(self as u32) {
            Some(tag) => tag,
            None => unreachable!(),
        }
    }

    /// Resolves a 24-bit raw tag integer to an `AttributeTag` if it is an attribute tag.
    #[must_use]
    // Keep the exhaustive catalog mapping in one auditable match.
    #[allow(clippy::too_many_lines)]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            x if x == KmipTag::ActivationDate as u32 => Some(Self::ActivationDate),
            x if x == KmipTag::ApplicationSpecificInformation as u32 => {
                Some(Self::ApplicationSpecificInformation)
            }
            x if x == KmipTag::ArchiveDate as u32 => Some(Self::ArchiveDate),
            x if x == KmipTag::Attribute as u32 => Some(Self::Attribute),
            x if x == KmipTag::CertificateType as u32 => Some(Self::CertificateType),
            x if x == KmipTag::CompromiseDate as u32 => Some(Self::CompromiseDate),
            x if x == KmipTag::CompromiseOccurrenceDate as u32 => {
                Some(Self::CompromiseOccurrenceDate)
            }
            x if x == KmipTag::ContactInformation as u32 => Some(Self::ContactInformation),
            x if x == KmipTag::CredentialType as u32 => Some(Self::CredentialType),
            x if x == KmipTag::CryptographicAlgorithm as u32 => Some(Self::CryptographicAlgorithm),
            x if x == KmipTag::CryptographicDomainParameters as u32 => {
                Some(Self::CryptographicDomainParameters)
            }
            x if x == KmipTag::CryptographicLength as u32 => Some(Self::CryptographicLength),
            x if x == KmipTag::CryptographicParameters as u32 => {
                Some(Self::CryptographicParameters)
            }
            x if x == KmipTag::CryptographicUsageMask as u32 => Some(Self::CryptographicUsageMask),
            x if x == KmipTag::DeactivationDate as u32 => Some(Self::DeactivationDate),
            x if x == KmipTag::DestroyDate as u32 => Some(Self::DestroyDate),
            x if x == KmipTag::Digest as u32 => Some(Self::Digest),
            x if x == KmipTag::InitialDate as u32 => Some(Self::InitialDate),
            x if x == KmipTag::KeyFormatType as u32 => Some(Self::KeyFormatType),
            x if x == KmipTag::KeyPartIdentifier as u32 => Some(Self::KeyPartIdentifier),
            x if x == KmipTag::LastChangeDate as u32 => Some(Self::LastChangeDate),
            x if x == KmipTag::LeaseTime as u32 => Some(Self::LeaseTime),
            x if x == KmipTag::Link as u32 => Some(Self::Link),
            x if x == KmipTag::Name as u32 => Some(Self::Name),
            x if x == KmipTag::ObjectGroup as u32 => Some(Self::ObjectGroup),
            x if x == KmipTag::ObjectType as u32 => Some(Self::ObjectType),
            x if x == KmipTag::OpaqueDataType as u32 => Some(Self::OpaqueDataType),
            x if x == KmipTag::ProcessStartDate as u32 => Some(Self::ProcessStartDate),
            x if x == KmipTag::ProtectStopDate as u32 => Some(Self::ProtectStopDate),
            x if x == KmipTag::RevocationReason as u32 => Some(Self::RevocationReason),
            x if x == KmipTag::SplitKeyMethod as u32 => Some(Self::SplitKeyMethod),
            x if x == KmipTag::SplitKeyParts as u32 => Some(Self::SplitKeyParts),
            x if x == KmipTag::SplitKeyThreshold as u32 => Some(Self::SplitKeyThreshold),
            x if x == KmipTag::State as u32 => Some(Self::State),
            x if x == KmipTag::UniqueIdentifier as u32 => Some(Self::UniqueIdentifier),
            x if x == KmipTag::UsageLimits as u32 => Some(Self::UsageLimits),
            x if x == KmipTag::Fresh as u32 => Some(Self::Fresh),
            x if x == KmipTag::CertificateLength as u32 => Some(Self::CertificateLength),
            x if x == KmipTag::DigitalSignatureAlgorithm as u32 => {
                Some(Self::DigitalSignatureAlgorithm)
            }
            x if x == KmipTag::X509CertificateIdentifier as u32 => {
                Some(Self::X509CertificateIdentifier)
            }
            x if x == KmipTag::X509CertificateIssuer as u32 => Some(Self::X509CertificateIssuer),
            x if x == KmipTag::X509CertificateSubject as u32 => Some(Self::X509CertificateSubject),
            x if x == KmipTag::KeyValueLocation as u32 => Some(Self::KeyValueLocation),
            x if x == KmipTag::KeyValuePresent as u32 => Some(Self::KeyValuePresent),
            x if x == KmipTag::OriginalCreationDate as u32 => Some(Self::OriginalCreationDate),
            x if x == KmipTag::AlternativeName as u32 => Some(Self::AlternativeName),
            x if x == KmipTag::RandomNumberGenerator as u32 => Some(Self::RandomNumberGenerator),
            x if x == KmipTag::PKCS12FriendlyName as u32 => Some(Self::PKCS12FriendlyName),
            x if x == KmipTag::Description as u32 => Some(Self::Description),
            x if x == KmipTag::Comment as u32 => Some(Self::Comment),
            x if x == KmipTag::Sensitive as u32 => Some(Self::Sensitive),
            x if x == KmipTag::AlwaysSensitive as u32 => Some(Self::AlwaysSensitive),
            x if x == KmipTag::Extractable as u32 => Some(Self::Extractable),
            x if x == KmipTag::NeverExtractable as u32 => Some(Self::NeverExtractable),
            x if x == KmipTag::ShortUniqueIdentifier as u32 => Some(Self::ShortUniqueIdentifier),
            x if x == KmipTag::NISTKeyType as u32 => Some(Self::NISTKeyType),
            x if x == KmipTag::ProtectionLevel as u32 => Some(Self::ProtectionLevel),
            x if x == KmipTag::ProtectionPeriod as u32 => Some(Self::ProtectionPeriod),
            x if x == KmipTag::QuantumSafe as u32 => Some(Self::QuantumSafe),
            x if x == KmipTag::ProtectionStorageMask as u32 => Some(Self::ProtectionStorageMask),
            x if x == KmipTag::RotateInterval as u32 => Some(Self::RotateInterval),
            x if x == KmipTag::RotateAutomatic as u32 => Some(Self::RotateAutomatic),
            x if x == KmipTag::RotateOffset as u32 => Some(Self::RotateOffset),
            x if x == KmipTag::RotateDate as u32 => Some(Self::RotateDate),
            x if x == KmipTag::RotateGeneration as u32 => Some(Self::RotateGeneration),
            x if x == KmipTag::RotateName as u32 => Some(Self::RotateName),
            x if x == KmipTag::RotateLatest as u32 => Some(Self::RotateLatest),
            x if x == KmipTag::CertificateLink as u32 => Some(Self::CertificateLink),
            x if x == KmipTag::ChildLink as u32 => Some(Self::ChildLink),
            x if x == KmipTag::DerivationBaseObjectLink as u32 => {
                Some(Self::DerivationBaseObjectLink)
            }
            x if x == KmipTag::DerivedObjectLink as u32 => Some(Self::DerivedObjectLink),
            x if x == KmipTag::NextLink as u32 => Some(Self::NextLink),
            x if x == KmipTag::ParentLink as u32 => Some(Self::ParentLink),
            x if x == KmipTag::PKCS12CertificateLink as u32 => Some(Self::PKCS12CertificateLink),
            x if x == KmipTag::PKCS12PasswordLink as u32 => Some(Self::PKCS12PasswordLink),
            x if x == KmipTag::PreviousLink as u32 => Some(Self::PreviousLink),
            x if x == KmipTag::PrivateKeyLink as u32 => Some(Self::PrivateKeyLink),
            x if x == KmipTag::PublicKeyLink as u32 => Some(Self::PublicKeyLink),
            x if x == KmipTag::ReplacedObjectLink as u32 => Some(Self::ReplacedObjectLink),
            x if x == KmipTag::ReplacementObjectLink as u32 => Some(Self::ReplacementObjectLink),
            x if x == KmipTag::WrappingKeyLink as u32 => Some(Self::WrappingKeyLink),
            x if x == KmipTag::ObjectClass as u32 => Some(Self::ObjectClass),
            x if x == KmipTag::CredentialLink as u32 => Some(Self::CredentialLink),
            x if x == KmipTag::PasswordLink as u32 => Some(Self::PasswordLink),
            x if x == KmipTag::OTPCounter as u32 => Some(Self::OTPCounter),
            x if x == KmipTag::GroupLink as u32 => Some(Self::GroupLink),
            x if x == KmipTag::SplitKeyBaseLink as u32 => Some(Self::SplitKeyBaseLink),
            x if x == KmipTag::JoinedSplitKeyPartsLink as u32 => {
                Some(Self::JoinedSplitKeyPartsLink)
            }
            x if x == KmipTag::SplitKeyPolynomial as u32 => Some(Self::SplitKeyPolynomial),
            x if x == KmipTag::DeactivationReason as u32 => Some(Self::DeactivationReason),
            x if x == KmipTag::CertificateSubjectDN as u32 => Some(Self::CertificateSubjectDN),
            x if x == KmipTag::CertificateIssuerDN as u32 => Some(Self::CertificateIssuerDN),
            x if x == KmipTag::CertificateRequestLink as u32 => Some(Self::CertificateRequestLink),
            x if x == KmipTag::CertifyCounter as u32 => Some(Self::CertifyCounter),
            x if x == KmipTag::DecryptCounter as u32 => Some(Self::DecryptCounter),
            x if x == KmipTag::EncryptCounter as u32 => Some(Self::EncryptCounter),
            x if x == KmipTag::SignCounter as u32 => Some(Self::SignCounter),
            x if x == KmipTag::SignatureVerifyCounter as u32 => Some(Self::SignatureVerifyCounter),
            x if x == KmipTag::NISTSecurityCategory as u32 => Some(Self::NISTSecurityCategory),
            _ => None,
        }
    }

    /// Converts to a checked [`Tag`].
    #[inline]
    #[must_use]
    pub fn into_tag(self) -> Tag {
        self.as_kmip_tag().into_tag()
    }
}

/// Dedicated enum for the 20 official KMIP 3.0 Link attributes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
#[non_exhaustive]
pub enum LinkAttributeTag {
    /// `CertificateLink` (KMIP 3.0)
    CertificateLink = KmipTag::CertificateLink as u32,
    /// `ChildLink` (KMIP 3.0)
    ChildLink = KmipTag::ChildLink as u32,
    /// `DerivationBaseObjectLink` (KMIP 3.0)
    DerivationBaseObjectLink = KmipTag::DerivationBaseObjectLink as u32,
    /// `DerivedObjectLink` (KMIP 3.0)
    DerivedObjectLink = KmipTag::DerivedObjectLink as u32,
    /// `NextLink` (KMIP 3.0)
    NextLink = KmipTag::NextLink as u32,
    /// `ParentLink` (KMIP 3.0)
    ParentLink = KmipTag::ParentLink as u32,
    /// `PKCS12CertificateLink` (KMIP 3.0)
    PKCS12CertificateLink = KmipTag::PKCS12CertificateLink as u32,
    /// `PKCS12PasswordLink` (KMIP 3.0)
    PKCS12PasswordLink = KmipTag::PKCS12PasswordLink as u32,
    /// `PreviousLink` (KMIP 3.0)
    PreviousLink = KmipTag::PreviousLink as u32,
    /// `PrivateKeyLink` (KMIP 3.0)
    PrivateKeyLink = KmipTag::PrivateKeyLink as u32,
    /// `PublicKeyLink` (KMIP 3.0)
    PublicKeyLink = KmipTag::PublicKeyLink as u32,
    /// `ReplacedObjectLink` (KMIP 3.0)
    ReplacedObjectLink = KmipTag::ReplacedObjectLink as u32,
    /// `ReplacementObjectLink` (KMIP 3.0)
    ReplacementObjectLink = KmipTag::ReplacementObjectLink as u32,
    /// `WrappingKeyLink` (KMIP 3.0)
    WrappingKeyLink = KmipTag::WrappingKeyLink as u32,
    /// `CredentialLink` (KMIP 3.0)
    CredentialLink = KmipTag::CredentialLink as u32,
    /// `PasswordLink` (KMIP 3.0)
    PasswordLink = KmipTag::PasswordLink as u32,
    /// `GroupLink` (KMIP 3.0)
    GroupLink = KmipTag::GroupLink as u32,
    /// `SplitKeyBaseLink` (KMIP 3.0)
    SplitKeyBaseLink = KmipTag::SplitKeyBaseLink as u32,
    /// `JoinedSplitKeyPartsLink` (KMIP 3.0)
    JoinedSplitKeyPartsLink = KmipTag::JoinedSplitKeyPartsLink as u32,
    /// `CertificateRequestLink` (KMIP 3.0)
    CertificateRequestLink = KmipTag::CertificateRequestLink as u32,
}

impl LinkAttributeTag {
    /// Returns the 24-bit raw tag value.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Converts this link attribute tag to the corresponding [`KmipTag`].
    #[inline]
    #[must_use]
    pub const fn as_kmip_tag(self) -> KmipTag {
        match KmipTag::from_u32(self as u32) {
            Some(tag) => tag,
            None => unreachable!(),
        }
    }

    /// Converts this link attribute tag to a checked [`Tag`].
    #[inline]
    #[must_use]
    pub fn into_tag(self) -> Tag {
        self.as_kmip_tag().into_tag()
    }

    /// Resolves a 24-bit raw tag to a `LinkAttributeTag`.
    #[must_use]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            x if x == KmipTag::CertificateLink as u32 => Some(Self::CertificateLink),
            x if x == KmipTag::ChildLink as u32 => Some(Self::ChildLink),
            x if x == KmipTag::DerivationBaseObjectLink as u32 => {
                Some(Self::DerivationBaseObjectLink)
            }
            x if x == KmipTag::DerivedObjectLink as u32 => Some(Self::DerivedObjectLink),
            x if x == KmipTag::NextLink as u32 => Some(Self::NextLink),
            x if x == KmipTag::ParentLink as u32 => Some(Self::ParentLink),
            x if x == KmipTag::PKCS12CertificateLink as u32 => Some(Self::PKCS12CertificateLink),
            x if x == KmipTag::PKCS12PasswordLink as u32 => Some(Self::PKCS12PasswordLink),
            x if x == KmipTag::PreviousLink as u32 => Some(Self::PreviousLink),
            x if x == KmipTag::PrivateKeyLink as u32 => Some(Self::PrivateKeyLink),
            x if x == KmipTag::PublicKeyLink as u32 => Some(Self::PublicKeyLink),
            x if x == KmipTag::ReplacedObjectLink as u32 => Some(Self::ReplacedObjectLink),
            x if x == KmipTag::ReplacementObjectLink as u32 => Some(Self::ReplacementObjectLink),
            x if x == KmipTag::WrappingKeyLink as u32 => Some(Self::WrappingKeyLink),
            x if x == KmipTag::CredentialLink as u32 => Some(Self::CredentialLink),
            x if x == KmipTag::PasswordLink as u32 => Some(Self::PasswordLink),
            x if x == KmipTag::GroupLink as u32 => Some(Self::GroupLink),
            x if x == KmipTag::SplitKeyBaseLink as u32 => Some(Self::SplitKeyBaseLink),
            x if x == KmipTag::JoinedSplitKeyPartsLink as u32 => {
                Some(Self::JoinedSplitKeyPartsLink)
            }
            x if x == KmipTag::CertificateRequestLink as u32 => Some(Self::CertificateRequestLink),
            _ => None,
        }
    }
}

/// Canonical KMIP Operation Enumeration codes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
#[non_exhaustive]
pub enum OperationCode {
    /// `CREATE` (`0x0000_0001`)
    Create = 0x0000_0001,
    /// `CREATE_KEY_PAIR` (`0x0000_0002`)
    CreateKeyPair = 0x0000_0002,
    /// `REGISTER` (`0x0000_0003`)
    Register = 0x0000_0003,
    /// `RE_KEY` (`0x0000_0004`)
    ReKey = 0x0000_0004,
    /// `DERIVE_KEY` (`0x0000_0005`)
    DeriveKey = 0x0000_0005,
    /// `CERTIFY` (`0x0000_0006`)
    Certify = 0x0000_0006,
    /// `RE_CERTIFY` (`0x0000_0007`)
    ReCertify = 0x0000_0007,
    /// `LOCATE` (`0x0000_0008`)
    Locate = 0x0000_0008,
    /// `CHECK` (`0x0000_0009`)
    Check = 0x0000_0009,
    /// `GET` (`0x0000_000A`)
    Get = 0x0000_000A,
    /// `GET_ATTRIBUTES` (`0x0000_000B`)
    GetAttributes = 0x0000_000B,
    /// `GET_ATTRIBUTE_LIST` (`0x0000_000C`)
    GetAttributeList = 0x0000_000C,
    /// `ADD_ATTRIBUTE` (`0x0000_000D`)
    AddAttribute = 0x0000_000D,
    /// `MODIFY_ATTRIBUTE` (`0x0000_000E`)
    ModifyAttribute = 0x0000_000E,
    /// `DELETE_ATTRIBUTE` (`0x0000_000F`)
    DeleteAttribute = 0x0000_000F,
    /// `OBTAIN_LEASE` (`0x0000_0010`)
    ObtainLease = 0x0000_0010,
    /// `GET_USAGE_ALLOCATION` (`0x0000_0011`)
    GetUsageAllocation = 0x0000_0011,
    /// `ACTIVATE` (`0x0000_0012`)
    Activate = 0x0000_0012,
    /// `REVOKE` (`0x0000_0013`)
    Revoke = 0x0000_0013,
    /// `DESTROY` (`0x0000_0014`)
    Destroy = 0x0000_0014,
    /// `ARCHIVE` (`0x0000_0015`)
    Archive = 0x0000_0015,
    /// `RECOVER` (`0x0000_0016`)
    Recover = 0x0000_0016,
    /// `VALIDATE` (`0x0000_0017`)
    Validate = 0x0000_0017,
    /// `QUERY` (`0x0000_0018`)
    Query = 0x0000_0018,
    /// `CANCEL` (`0x0000_0019`)
    Cancel = 0x0000_0019,
    /// `POLL` (`0x0000_001A`)
    Poll = 0x0000_001A,
    /// `NOTIFY` (`0x0000_001B`)
    Notify = 0x0000_001B,
    /// `PUT` (`0x0000_001C`)
    Put = 0x0000_001C,
    /// `RE_KEY_KEY_PAIR` (`0x0000_001D`)
    ReKeyKeyPair = 0x0000_001D,
    /// `DISCOVER_VERSIONS` (`0x0000_001E`)
    DiscoverVersions = 0x0000_001E,
    /// `ENCRYPT` (`0x0000_001F`)
    Encrypt = 0x0000_001F,
    /// `DECRYPT` (`0x0000_0020`)
    Decrypt = 0x0000_0020,
    /// `SIGN` (`0x0000_0021`)
    Sign = 0x0000_0021,
    /// `SIGNATURE_VERIFY` (`0x0000_0022`)
    SignatureVerify = 0x0000_0022,
    /// `MAC` (`0x0000_0023`)
    Mac = 0x0000_0023,
    /// `MAC_VERIFY` (`0x0000_0024`)
    MacVerify = 0x0000_0024,
    /// `RNG_RETRIEVE` (`0x0000_0025`)
    RngRetrieve = 0x0000_0025,
    /// `RNG_SEED` (`0x0000_0026`)
    RngSeed = 0x0000_0026,
    /// `HASH` (`0x0000_0027`)
    Hash = 0x0000_0027,
    /// `CREATE_SPLIT_KEY` (`0x0000_0028`)
    CreateSplitKey = 0x0000_0028,
    /// `JOIN_SPLIT_KEY` (`0x0000_0029`)
    JoinSplitKey = 0x0000_0029,
    /// `IMPORT` (`0x0000_002A`)
    Import = 0x0000_002A,
    /// `EXPORT` (`0x0000_002B`)
    Export = 0x0000_002B,
    /// `LOG` (`0x0000_002C`)
    Log = 0x0000_002C,
    /// `LOGIN` (`0x0000_002D`)
    Login = 0x0000_002D,
    /// `LOGOUT` (`0x0000_002E`)
    Logout = 0x0000_002E,
    /// `DELEGATED_LOGIN` (`0x0000_002F`)
    DelegatedLogin = 0x0000_002F,
    /// `ADJUST_ATTRIBUTE` (`0x0000_0030`)
    AdjustAttribute = 0x0000_0030,
    /// `SET_ATTRIBUTE` (`0x0000_0031`)
    SetAttribute = 0x0000_0031,
    /// `SET_ENDPOINT_ROLE` (`0x0000_0032`)
    SetEndpointRole = 0x0000_0032,
    /// `INTEROP` (`0x0000_0034`)
    Interop = 0x0000_0034,
    /// `RE_PROVISION` (`0x0000_0035`)
    ReProvision = 0x0000_0035,
    /// `SET_DEFAULTS` (`0x0000_0036`)
    SetDefaults = 0x0000_0036,
    /// `SET_CONSTRAINTS` (`0x0000_0037`)
    SetConstraints = 0x0000_0037,
    /// `GET_CONSTRAINTS` (`0x0000_0038`)
    GetConstraints = 0x0000_0038,
    /// `QUERY_ASYNCHRONOUS_REQUESTS` (`0x0000_0039`)
    QueryAsynchronousRequests = 0x0000_0039,
    /// `PROCESS` (`0x0000_003A`)
    Process = 0x0000_003A,
    /// `PING` (`0x0000_003B`)
    Ping = 0x0000_003B,
    /// `KEM_ENCRYPT` (`0x0000_003C`)
    KemEncrypt = 0x0000_003C,
    /// `KEM_DECRYPT` (`0x0000_003D`)
    KemDecrypt = 0x0000_003D,
    /// `ENCAPSULATE` (`0x0000_0041`, KMIP 3.0)
    Encapsulate = 0x0000_0041,
    /// `DECAPSULATE` (`0x0000_0042`, KMIP 3.0)
    Decapsulate = 0x0000_0042,
}

impl OperationCode {
    /// Returns the 32-bit enumeration value.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Resolves an integer to an `OperationCode`.
    #[must_use]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            0x0000_0041 => Some(Self::Encapsulate),
            0x0000_0042 => Some(Self::Decapsulate),
            0x0000_0001 => Some(Self::Create),
            0x0000_0002 => Some(Self::CreateKeyPair),
            0x0000_0003 => Some(Self::Register),
            0x0000_0004 => Some(Self::ReKey),
            0x0000_0005 => Some(Self::DeriveKey),
            0x0000_0006 => Some(Self::Certify),
            0x0000_0007 => Some(Self::ReCertify),
            0x0000_0008 => Some(Self::Locate),
            0x0000_0009 => Some(Self::Check),
            0x0000_000A => Some(Self::Get),
            0x0000_000B => Some(Self::GetAttributes),
            0x0000_000C => Some(Self::GetAttributeList),
            0x0000_000D => Some(Self::AddAttribute),
            0x0000_000E => Some(Self::ModifyAttribute),
            0x0000_000F => Some(Self::DeleteAttribute),
            0x0000_0010 => Some(Self::ObtainLease),
            0x0000_0011 => Some(Self::GetUsageAllocation),
            0x0000_0012 => Some(Self::Activate),
            0x0000_0013 => Some(Self::Revoke),
            0x0000_0014 => Some(Self::Destroy),
            0x0000_0015 => Some(Self::Archive),
            0x0000_0016 => Some(Self::Recover),
            0x0000_0017 => Some(Self::Validate),
            0x0000_0018 => Some(Self::Query),
            0x0000_0019 => Some(Self::Cancel),
            0x0000_001A => Some(Self::Poll),
            0x0000_001B => Some(Self::Notify),
            0x0000_001C => Some(Self::Put),
            0x0000_001D => Some(Self::ReKeyKeyPair),
            0x0000_001E => Some(Self::DiscoverVersions),
            0x0000_001F => Some(Self::Encrypt),
            0x0000_0020 => Some(Self::Decrypt),
            0x0000_0021 => Some(Self::Sign),
            0x0000_0022 => Some(Self::SignatureVerify),
            0x0000_0023 => Some(Self::Mac),
            0x0000_0024 => Some(Self::MacVerify),
            0x0000_0025 => Some(Self::RngRetrieve),
            0x0000_0026 => Some(Self::RngSeed),
            0x0000_0027 => Some(Self::Hash),
            0x0000_0028 => Some(Self::CreateSplitKey),
            0x0000_0029 => Some(Self::JoinSplitKey),
            0x0000_002A => Some(Self::Import),
            0x0000_002B => Some(Self::Export),
            0x0000_002C => Some(Self::Log),
            0x0000_002D => Some(Self::Login),
            0x0000_002E => Some(Self::Logout),
            0x0000_002F => Some(Self::DelegatedLogin),
            0x0000_0030 => Some(Self::AdjustAttribute),
            0x0000_0031 => Some(Self::SetAttribute),
            0x0000_0032 => Some(Self::SetEndpointRole),
            0x0000_0034 => Some(Self::Interop),
            0x0000_0035 => Some(Self::ReProvision),
            0x0000_0036 => Some(Self::SetDefaults),
            0x0000_0037 => Some(Self::SetConstraints),
            0x0000_0038 => Some(Self::GetConstraints),
            0x0000_0039 => Some(Self::QueryAsynchronousRequests),
            0x0000_003A => Some(Self::Process),
            0x0000_003B => Some(Self::Ping),
            0x0000_003C => Some(Self::KemEncrypt),
            0x0000_003D => Some(Self::KemDecrypt),
            _ => None,
        }
    }
}

/// Canonical KMIP Object Type Enumeration codes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
#[non_exhaustive]
pub enum ObjectTypeCode {
    /// `Certificate` (`0x0000_0001`)
    Certificate = 0x0000_0001,
    /// `Symmetric Key` (`0x0000_0002`)
    SymmetricKey = 0x0000_0002,
    /// `Public Key` (`0x0000_0003`)
    PublicKey = 0x0000_0003,
    /// `Private Key` (`0x0000_0004`)
    PrivateKey = 0x0000_0004,
    /// `Split Key` (`0x0000_0005`)
    SplitKey = 0x0000_0005,
    /// `Template` (`0x0000_0006`)
    Template = 0x0000_0006,
    /// `Secret Data` (`0x0000_0007`)
    SecretData = 0x0000_0007,
    /// `Opaque Object` (`0x0000_0008`)
    OpaqueObject = 0x0000_0008,
    /// `PGP Key` (`0x0000_0009`)
    PgpKey = 0x0000_0009,
    /// `Certificate Request` (`0x0000_000A`)
    CertificateRequest = 0x0000_000A,
    /// `User Object (KMIP 3.0+)` (`0x0000_000B`)
    UserObject = 0x0000_000B,
    /// `Group Object (KMIP 3.0+)` (`0x0000_000C`)
    GroupObject = 0x0000_000C,
}

impl ObjectTypeCode {
    /// Returns the 32-bit enumeration value.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Resolves an integer to an `ObjectTypeCode`.
    #[must_use]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            0x0000_0001 => Some(Self::Certificate),
            0x0000_0002 => Some(Self::SymmetricKey),
            0x0000_0003 => Some(Self::PublicKey),
            0x0000_0004 => Some(Self::PrivateKey),
            0x0000_0005 => Some(Self::SplitKey),
            0x0000_0006 => Some(Self::Template),
            0x0000_0007 => Some(Self::SecretData),
            0x0000_0008 => Some(Self::OpaqueObject),
            0x0000_0009 => Some(Self::PgpKey),
            0x0000_000A => Some(Self::CertificateRequest),
            0x0000_000B => Some(Self::UserObject),
            0x0000_000C => Some(Self::GroupObject),
            _ => None,
        }
    }
}

/// NIST SP 800-57 Key Lifecycle State Enumeration codes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum KeyLifecycleState {
    /// `Pre-Active` (`0x0000_0001`)
    PreActive = 0x0000_0001,
    /// `Active` (`0x0000_0002`)
    Active = 0x0000_0002,
    /// `Deactivated` (`0x0000_0003`)
    Deactivated = 0x0000_0003,
    /// `Compromised` (`0x0000_0004`)
    Compromised = 0x0000_0004,
    /// `Destroyed` (`0x0000_0005`)
    Destroyed = 0x0000_0005,
    /// `Destroyed Compromised` (`0x0000_0006`)
    DestroyedCompromised = 0x0000_0006,
}

impl KeyLifecycleState {
    /// Returns the 32-bit enumeration value.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Resolves an integer to a `KeyLifecycleState`.
    #[must_use]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            0x0000_0001 => Some(Self::PreActive),
            0x0000_0002 => Some(Self::Active),
            0x0000_0003 => Some(Self::Deactivated),
            0x0000_0004 => Some(Self::Compromised),
            0x0000_0005 => Some(Self::Destroyed),
            0x0000_0006 => Some(Self::DestroyedCompromised),
            _ => None,
        }
    }
}

/// Post-Quantum Cryptography (PQC) KEM Algorithm Enumeration codes (OASIS Table 573 / Tag 0x4201C3).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum KemAlgorithmCode {
    /// `RSASVE` (`0x0000_0001`)
    Rsasve = 0x0000_0001,
    /// `DHKEM` (`0x0000_0002`)
    Dhkem = 0x0000_0002,
    /// `ML-KEM` (`0x0000_0003`)
    Mlkem = 0x0000_0003,
    /// `ML-KEM-512 (FIPS 203)` (`0x0000_0004`)
    MlKem512 = 0x0000_0004,
    /// `ML-KEM-768 (FIPS 203)` (`0x0000_0005`)
    MlKem768 = 0x0000_0005,
    /// `ML-KEM-1024 (FIPS 203)` (`0x0000_0006`)
    MlKem1024 = 0x0000_0006,
}

impl KemAlgorithmCode {
    /// Returns the 32-bit enumeration value.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Resolves an integer to a `KemAlgorithmCode`.
    #[must_use]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            0x0000_0001 => Some(Self::Rsasve),
            0x0000_0002 => Some(Self::Dhkem),
            0x0000_0003 => Some(Self::Mlkem),
            0x0000_0004 => Some(Self::MlKem512),
            0x0000_0005 => Some(Self::MlKem768),
            0x0000_0006 => Some(Self::MlKem1024),
            _ => None,
        }
    }
}

/// KMIP 3.0 Ephemeral Mode Enumeration codes (Tag `0x4201BF`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum EphemeralMode {
    /// `No ephemeral handling` (`0x0000_0001`)
    No = 0x0000_0001,
    /// `Destroy immediately after request completes` (`0x0000_0002`)
    Immediate = 0x0000_0002,
    /// `Destroy on success; retain on failure for diagnostics` (`0x0000_0003`)
    RetainOnFailure = 0x0000_0003,
}

impl EphemeralMode {
    /// Returns the 32-bit enumeration value.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Resolves an integer to an `EphemeralMode`.
    #[must_use]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            0x0000_0001 => Some(Self::No),
            0x0000_0002 => Some(Self::Immediate),
            0x0000_0003 => Some(Self::RetainOnFailure),
            _ => None,
        }
    }
}
