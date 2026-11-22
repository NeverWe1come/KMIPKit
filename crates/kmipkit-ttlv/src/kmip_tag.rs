//! Strongly typed KMIP Tag enumeration covering all official OASIS standard tags.
//! Generated from the normative OASIS specification catalog.

use crate::error::ModelError;
use crate::tag::{RawTag, Tag};

/// Canonical 24-bit KMIP Tag enumeration covering all official OASIS specifications
/// across KMIP 2.0, 2.1, and 3.0.
///
/// This is the single source of truth for official KMIP tag hexadecimal values.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u32)]
#[non_exhaustive]
pub enum KmipTag {
    /// `Activation Date` (`0x0042_0001`)
    ActivationDate = 0x0042_0001,
    /// `Application Data` (`0x0042_0002`)
    ApplicationData = 0x0042_0002,
    /// `Application Namespace` (`0x0042_0003`)
    ApplicationNamespace = 0x0042_0003,
    /// `Application Specific Information` (`0x0042_0004`)
    ApplicationSpecificInformation = 0x0042_0004,
    /// `Archive Date` (`0x0042_0005`)
    ArchiveDate = 0x0042_0005,
    /// `Asynchronous Correlation Value` (`0x0042_0006`)
    AsynchronousCorrelationValue = 0x0042_0006,
    /// `Asynchronous Indicator` (`0x0042_0007`)
    AsynchronousIndicator = 0x0042_0007,
    /// `Attribute` (`0x0042_0008`)
    Attribute = 0x0042_0008,
    /// `Attribute Name` (`0x0042_000A`)
    AttributeName = 0x0042_000A,
    /// `Attribute Value` (`0x0042_000B`)
    AttributeValue = 0x0042_000B,
    /// `Authentication` (`0x0042_000C`)
    Authentication = 0x0042_000C,
    /// `Batch Count` (`0x0042_000D`)
    BatchCount = 0x0042_000D,
    /// `Batch Error Continuation Option` (`0x0042_000E`)
    BatchErrorContinuationOption = 0x0042_000E,
    /// `Batch Item` (`0x0042_000F`)
    BatchItem = 0x0042_000F,
    /// `Batch Order Option` (`0x0042_0010`)
    BatchOrderOption = 0x0042_0010,
    /// `Block Cipher Mode` (`0x0042_0011`)
    BlockCipherMode = 0x0042_0011,
    /// `Cancellation Result` (`0x0042_0012`)
    CancellationResult = 0x0042_0012,
    /// `Certificate` (`0x0042_0013`)
    Certificate = 0x0042_0013,
    /// `Certificate Request` (`0x0042_0018`)
    CertificateRequest = 0x0042_0018,
    /// `Certificate Request Type` (`0x0042_0019`)
    CertificateRequestType = 0x0042_0019,
    /// `Certificate Type` (`0x0042_001D`)
    CertificateType = 0x0042_001D,
    /// `Certificate Value` (`0x0042_001E`)
    CertificateValue = 0x0042_001E,
    /// `Compromise Date` (`0x0042_0020`)
    CompromiseDate = 0x0042_0020,
    /// `Compromise Occurrence Date` (`0x0042_0021`)
    CompromiseOccurrenceDate = 0x0042_0021,
    /// `Contact Information` (`0x0042_0022`)
    ContactInformation = 0x0042_0022,
    /// `Credential` (`0x0042_0023`)
    Credential = 0x0042_0023,
    /// `Credential Type` (`0x0042_0024`)
    CredentialType = 0x0042_0024,
    /// `Credential Value` (`0x0042_0025`)
    CredentialValue = 0x0042_0025,
    /// `Criticality Indicator` (`0x0042_0026`)
    CriticalityIndicator = 0x0042_0026,
    /// `CRT Coefficient` (`0x0042_0027`)
    CRTCoefficient = 0x0042_0027,
    /// `Cryptographic Algorithm` (`0x0042_0028`)
    CryptographicAlgorithm = 0x0042_0028,
    /// `Cryptographic Domain Parameters` (`0x0042_0029`)
    CryptographicDomainParameters = 0x0042_0029,
    /// `Cryptographic Length` (`0x0042_002A`)
    CryptographicLength = 0x0042_002A,
    /// `Cryptographic Parameters` (`0x0042_002B`)
    CryptographicParameters = 0x0042_002B,
    /// `Cryptographic Usage Mask` (`0x0042_002C`)
    CryptographicUsageMask = 0x0042_002C,
    /// `D` (`0x0042_002E`)
    D = 0x0042_002E,
    /// `Deactivation Date` (`0x0042_002F`)
    DeactivationDate = 0x0042_002F,
    /// `Derivation Data` (`0x0042_0030`)
    DerivationData = 0x0042_0030,
    /// `Derivation Method` (`0x0042_0031`)
    DerivationMethod = 0x0042_0031,
    /// `Derivation Parameters` (`0x0042_0032`)
    DerivationParameters = 0x0042_0032,
    /// `Destroy Date` (`0x0042_0033`)
    DestroyDate = 0x0042_0033,
    /// `Digest` (`0x0042_0034`)
    Digest = 0x0042_0034,
    /// `Digest Value` (`0x0042_0035`)
    DigestValue = 0x0042_0035,
    /// `Encryption Key Information` (`0x0042_0036`)
    EncryptionKeyInformation = 0x0042_0036,
    /// `G` (`0x0042_0037`)
    G = 0x0042_0037,
    /// `Hashing Algorithm` (`0x0042_0038`)
    HashingAlgorithm = 0x0042_0038,
    /// `Initial Date` (`0x0042_0039`)
    InitialDate = 0x0042_0039,
    /// `Initialization Vector` (`0x0042_003A`)
    InitializationVector = 0x0042_003A,
    /// `Iteration Count` (`0x0042_003C`)
    IterationCount = 0x0042_003C,
    /// `IV/Counter/Nonce` (`0x0042_003D`)
    IVCounterNonce = 0x0042_003D,
    /// `J` (`0x0042_003E`)
    J = 0x0042_003E,
    /// `Key` (`0x0042_003F`)
    Key = 0x0042_003F,
    /// `Key Block` (`0x0042_0040`)
    KeyBlock = 0x0042_0040,
    /// `Key Compression Type` (`0x0042_0041`)
    KeyCompressionType = 0x0042_0041,
    /// `Key Format Type` (`0x0042_0042`)
    KeyFormatType = 0x0042_0042,
    /// `Key Material` (`0x0042_0043`)
    KeyMaterial = 0x0042_0043,
    /// `Key Part Identifier` (`0x0042_0044`)
    KeyPartIdentifier = 0x0042_0044,
    /// `Key Value` (`0x0042_0045`)
    KeyValue = 0x0042_0045,
    /// `Key Wrapping Data` (`0x0042_0046`)
    KeyWrappingData = 0x0042_0046,
    /// `Key Wrapping Specification` (`0x0042_0047`)
    KeyWrappingSpecification = 0x0042_0047,
    /// `Last Change Date` (`0x0042_0048`)
    LastChangeDate = 0x0042_0048,
    /// `Lease Time` (`0x0042_0049`)
    LeaseTime = 0x0042_0049,
    /// `Link` (`0x0042_004A`)
    Link = 0x0042_004A,
    /// `Link Type` (`0x0042_004B`)
    LinkType = 0x0042_004B,
    /// `Linked Object Identifier` (`0x0042_004C`)
    LinkedObjectIdentifier = 0x0042_004C,
    /// `MAC/Signature` (`0x0042_004D`)
    MACSignature = 0x0042_004D,
    /// `MAC/Signature Key Information` (`0x0042_004E`)
    MACSignatureKeyInformation = 0x0042_004E,
    /// `Maximum Items` (`0x0042_004F`)
    MaximumItems = 0x0042_004F,
    /// `Maximum Response Size` (`0x0042_0050`)
    MaximumResponseSize = 0x0042_0050,
    /// `Message Extension` (`0x0042_0051`)
    MessageExtension = 0x0042_0051,
    /// `Modulus` (`0x0042_0052`)
    Modulus = 0x0042_0052,
    /// `Name` (`0x0042_0053`)
    Name = 0x0042_0053,
    /// `Name Type` (`0x0042_0054`)
    NameType = 0x0042_0054,
    /// `Name Value` (`0x0042_0055`)
    NameValue = 0x0042_0055,
    /// `Object Group` (`0x0042_0056`)
    ObjectGroup = 0x0042_0056,
    /// `Object Type` (`0x0042_0057`)
    ObjectType = 0x0042_0057,
    /// `Offset` (`0x0042_0058`)
    Offset = 0x0042_0058,
    /// `Opaque Data Type` (`0x0042_0059`)
    OpaqueDataType = 0x0042_0059,
    /// `Opaque Data Value` (`0x0042_005A`)
    OpaqueDataValue = 0x0042_005A,
    /// `Opaque Object` (`0x0042_005B`)
    OpaqueObject = 0x0042_005B,
    /// `Operation` (`0x0042_005C`)
    Operation = 0x0042_005C,
    /// `P` (`0x0042_005E`)
    P = 0x0042_005E,
    /// `Padding Method` (`0x0042_005F`)
    PaddingMethod = 0x0042_005F,
    /// `Prime Exponent P` (`0x0042_0060`)
    PrimeExponentP = 0x0042_0060,
    /// `Prime Exponent Q` (`0x0042_0061`)
    PrimeExponentQ = 0x0042_0061,
    /// `Prime Field Size` (`0x0042_0062`)
    PrimeFieldSize = 0x0042_0062,
    /// `Private Exponent` (`0x0042_0063`)
    PrivateExponent = 0x0042_0063,
    /// `Private Key` (`0x0042_0064`)
    PrivateKey = 0x0042_0064,
    /// `Private Key Unique Identifier` (`0x0042_0066`)
    PrivateKeyUniqueIdentifier = 0x0042_0066,
    /// `Process Start Date` (`0x0042_0067`)
    ProcessStartDate = 0x0042_0067,
    /// `Protect Stop Date` (`0x0042_0068`)
    ProtectStopDate = 0x0042_0068,
    /// `Protocol Version` (`0x0042_0069`)
    ProtocolVersion = 0x0042_0069,
    /// `Protocol Version Major` (`0x0042_006A`)
    ProtocolVersionMajor = 0x0042_006A,
    /// `Protocol Version Minor` (`0x0042_006B`)
    ProtocolVersionMinor = 0x0042_006B,
    /// `Public Exponent` (`0x0042_006C`)
    PublicExponent = 0x0042_006C,
    /// `Public Key` (`0x0042_006D`)
    PublicKey = 0x0042_006D,
    /// `Public Key Unique Identifier` (`0x0042_006F`)
    PublicKeyUniqueIdentifier = 0x0042_006F,
    /// `Put Function` (`0x0042_0070`)
    PutFunction = 0x0042_0070,
    /// `Q` (`0x0042_0071`)
    Q = 0x0042_0071,
    /// `Q String` (`0x0042_0072`)
    QString = 0x0042_0072,
    /// `Qlength` (`0x0042_0073`)
    Qlength = 0x0042_0073,
    /// `Query Function` (`0x0042_0074`)
    QueryFunction = 0x0042_0074,
    /// `Recommended Curve` (`0x0042_0075`)
    RecommendedCurve = 0x0042_0075,
    /// `Replaced Unique Identifier` (`0x0042_0076`)
    ReplacedUniqueIdentifier = 0x0042_0076,
    /// `Request Header` (`0x0042_0077`)
    RequestHeader = 0x0042_0077,
    /// `Request Message` (`0x0042_0078`)
    RequestMessage = 0x0042_0078,
    /// `Request Payload` (`0x0042_0079`)
    RequestPayload = 0x0042_0079,
    /// `Response Header` (`0x0042_007A`)
    ResponseHeader = 0x0042_007A,
    /// `Response Message` (`0x0042_007B`)
    ResponseMessage = 0x0042_007B,
    /// `Response Payload` (`0x0042_007C`)
    ResponsePayload = 0x0042_007C,
    /// `Result Message` (`0x0042_007D`)
    ResultMessage = 0x0042_007D,
    /// `Result Reason` (`0x0042_007E`)
    ResultReason = 0x0042_007E,
    /// `Result Status` (`0x0042_007F`)
    ResultStatus = 0x0042_007F,
    /// `Revocation Message` (`0x0042_0080`)
    RevocationMessage = 0x0042_0080,
    /// `Revocation Reason` (`0x0042_0081`)
    RevocationReason = 0x0042_0081,
    /// `Revocation Reason Code` (`0x0042_0082`)
    RevocationReasonCode = 0x0042_0082,
    /// `Key Role Type` (`0x0042_0083`)
    KeyRoleType = 0x0042_0083,
    /// `Salt` (`0x0042_0084`)
    Salt = 0x0042_0084,
    /// `Secret Data` (`0x0042_0085`)
    SecretData = 0x0042_0085,
    /// `Secret Data Type` (`0x0042_0086`)
    SecretDataType = 0x0042_0086,
    /// `Server Information` (`0x0042_0088`)
    ServerInformation = 0x0042_0088,
    /// `Split Key` (`0x0042_0089`)
    SplitKey = 0x0042_0089,
    /// `Split Key Method` (`0x0042_008A`)
    SplitKeyMethod = 0x0042_008A,
    /// `Split Key Parts` (`0x0042_008B`)
    SplitKeyParts = 0x0042_008B,
    /// `Split Key Threshold` (`0x0042_008C`)
    SplitKeyThreshold = 0x0042_008C,
    /// `State` (`0x0042_008D`)
    State = 0x0042_008D,
    /// `Storage Status Mask` (`0x0042_008E`)
    StorageStatusMask = 0x0042_008E,
    /// `Symmetric Key` (`0x0042_008F`)
    SymmetricKey = 0x0042_008F,
    /// `Time Stamp` (`0x0042_0092`)
    TimeStamp = 0x0042_0092,
    /// `Unique Batch Item ID` (`0x0042_0093`)
    UniqueBatchItemID = 0x0042_0093,
    /// `Unique Identifier` (`0x0042_0094`)
    UniqueIdentifier = 0x0042_0094,
    /// `Usage Limits` (`0x0042_0095`)
    UsageLimits = 0x0042_0095,
    /// `Usage Limits Count` (`0x0042_0096`)
    UsageLimitsCount = 0x0042_0096,
    /// `Usage Limits Total` (`0x0042_0097`)
    UsageLimitsTotal = 0x0042_0097,
    /// `Usage Limits Unit` (`0x0042_0098`)
    UsageLimitsUnit = 0x0042_0098,
    /// `Username` (`0x0042_0099`)
    Username = 0x0042_0099,
    /// `Validity Date` (`0x0042_009A`)
    ValidityDate = 0x0042_009A,
    /// `Validity Indicator` (`0x0042_009B`)
    ValidityIndicator = 0x0042_009B,
    /// `Vendor Extension` (`0x0042_009C`)
    VendorExtension = 0x0042_009C,
    /// `Vendor Identification` (`0x0042_009D`)
    VendorIdentification = 0x0042_009D,
    /// `Wrapping Method` (`0x0042_009E`)
    WrappingMethod = 0x0042_009E,
    /// `X` (`0x0042_009F`)
    X = 0x0042_009F,
    /// `Y` (`0x0042_00A0`)
    Y = 0x0042_00A0,
    /// `Password` (`0x0042_00A1`)
    Password = 0x0042_00A1,
    /// `Device Identifier` (`0x0042_00A2`)
    DeviceIdentifier = 0x0042_00A2,
    /// `Encoding Option` (`0x0042_00A3`)
    EncodingOption = 0x0042_00A3,
    /// `Extension Information` (`0x0042_00A4`)
    ExtensionInformation = 0x0042_00A4,
    /// `Extension Name` (`0x0042_00A5`)
    ExtensionName = 0x0042_00A5,
    /// `Extension Tag` (`0x0042_00A6`)
    ExtensionTag = 0x0042_00A6,
    /// `Extension Type` (`0x0042_00A7`)
    ExtensionType = 0x0042_00A7,
    /// `Fresh` (`0x0042_00A8`)
    Fresh = 0x0042_00A8,
    /// `Machine Identifier` (`0x0042_00A9`)
    MachineIdentifier = 0x0042_00A9,
    /// `Media Identifier` (`0x0042_00AA`)
    MediaIdentifier = 0x0042_00AA,
    /// `Network Identifier` (`0x0042_00AB`)
    NetworkIdentifier = 0x0042_00AB,
    /// `Object Group Member` (`0x0042_00AC`)
    ObjectGroupMember = 0x0042_00AC,
    /// `Certificate Length` (`0x0042_00AD`)
    CertificateLength = 0x0042_00AD,
    /// `Digital Signature Algorithm` (`0x0042_00AE`)
    DigitalSignatureAlgorithm = 0x0042_00AE,
    /// `Certificate Serial Number` (`0x0042_00AF`)
    CertificateSerialNumber = 0x0042_00AF,
    /// `Device Serial Number` (`0x0042_00B0`)
    DeviceSerialNumber = 0x0042_00B0,
    /// `Issuer Alternative Name` (`0x0042_00B1`)
    IssuerAlternativeName = 0x0042_00B1,
    /// `Issuer Distinguished Name` (`0x0042_00B2`)
    IssuerDistinguishedName = 0x0042_00B2,
    /// `Subject Alternative Name` (`0x0042_00B3`)
    SubjectAlternativeName = 0x0042_00B3,
    /// `Subject Distinguished Name` (`0x0042_00B4`)
    SubjectDistinguishedName = 0x0042_00B4,
    /// `X.509 Certificate Identifier` (`0x0042_00B5`)
    X509CertificateIdentifier = 0x0042_00B5,
    /// `X.509 Certificate Issuer` (`0x0042_00B6`)
    X509CertificateIssuer = 0x0042_00B6,
    /// `X.509 Certificate Subject` (`0x0042_00B7`)
    X509CertificateSubject = 0x0042_00B7,
    /// `Key Value Location` (`0x0042_00B8`)
    KeyValueLocation = 0x0042_00B8,
    /// `Key Value Location Value` (`0x0042_00B9`)
    KeyValueLocationValue = 0x0042_00B9,
    /// `Key Value Location Type` (`0x0042_00BA`)
    KeyValueLocationType = 0x0042_00BA,
    /// `Key Value Present` (`0x0042_00BB`)
    KeyValuePresent = 0x0042_00BB,
    /// `Original Creation Date` (`0x0042_00BC`)
    OriginalCreationDate = 0x0042_00BC,
    /// `PGP Key` (`0x0042_00BD`)
    PGPKey = 0x0042_00BD,
    /// `PGP Key Version` (`0x0042_00BE`)
    PGPKeyVersion = 0x0042_00BE,
    /// `Alternative Name` (`0x0042_00BF`)
    AlternativeName = 0x0042_00BF,
    /// `Alternative Name Value` (`0x0042_00C0`)
    AlternativeNameValue = 0x0042_00C0,
    /// `Alternative Name Type` (`0x0042_00C1`)
    AlternativeNameType = 0x0042_00C1,
    /// `Data` (`0x0042_00C2`)
    Data = 0x0042_00C2,
    /// `Signature Data` (`0x0042_00C3`)
    SignatureData = 0x0042_00C3,
    /// `Data Length` (`0x0042_00C4`)
    DataLength = 0x0042_00C4,
    /// `Random IV` (`0x0042_00C5`)
    RandomIV = 0x0042_00C5,
    /// `MAC Data` (`0x0042_00C6`)
    MACData = 0x0042_00C6,
    /// `Attestation Type` (`0x0042_00C7`)
    AttestationType = 0x0042_00C7,
    /// `Nonce` (`0x0042_00C8`)
    Nonce = 0x0042_00C8,
    /// `Nonce ID` (`0x0042_00C9`)
    NonceID = 0x0042_00C9,
    /// `Nonce Value` (`0x0042_00CA`)
    NonceValue = 0x0042_00CA,
    /// `Attestation Measurement` (`0x0042_00CB`)
    AttestationMeasurement = 0x0042_00CB,
    /// `Attestation Assertion` (`0x0042_00CC`)
    AttestationAssertion = 0x0042_00CC,
    /// `IV Length` (`0x0042_00CD`)
    IVLength = 0x0042_00CD,
    /// `Tag Length` (`0x0042_00CE`)
    TagLength = 0x0042_00CE,
    /// `Fixed Field Length` (`0x0042_00CF`)
    FixedFieldLength = 0x0042_00CF,
    /// `Counter Length` (`0x0042_00D0`)
    CounterLength = 0x0042_00D0,
    /// `Initial Counter Value` (`0x0042_00D1`)
    InitialCounterValue = 0x0042_00D1,
    /// `Invocation Field Length` (`0x0042_00D2`)
    InvocationFieldLength = 0x0042_00D2,
    /// `Attestation Capable Indicator` (`0x0042_00D3`)
    AttestationCapableIndicator = 0x0042_00D3,
    /// `Offset Items` (`0x0042_00D4`)
    OffsetItems = 0x0042_00D4,
    /// `Located Items` (`0x0042_00D5`)
    LocatedItems = 0x0042_00D5,
    /// `Correlation Value` (`0x0042_00D6`)
    CorrelationValue = 0x0042_00D6,
    /// `Init Indicator` (`0x0042_00D7`)
    InitIndicator = 0x0042_00D7,
    /// `Final Indicator` (`0x0042_00D8`)
    FinalIndicator = 0x0042_00D8,
    /// `RNG Parameters` (`0x0042_00D9`)
    RNGParameters = 0x0042_00D9,
    /// `RNG Algorithm` (`0x0042_00DA`)
    RNGAlgorithm = 0x0042_00DA,
    /// `DRBG Algorithm` (`0x0042_00DB`)
    DRBGAlgorithm = 0x0042_00DB,
    /// `FIPS186 Variation` (`0x0042_00DC`)
    FIPS186Variation = 0x0042_00DC,
    /// `Prediction Resistance` (`0x0042_00DD`)
    PredictionResistance = 0x0042_00DD,
    /// `Random Number Generator` (`0x0042_00DE`)
    RandomNumberGenerator = 0x0042_00DE,
    /// `Validation Information` (`0x0042_00DF`)
    ValidationInformation = 0x0042_00DF,
    /// `Validation Authority Type` (`0x0042_00E0`)
    ValidationAuthorityType = 0x0042_00E0,
    /// `Validation Authority Country` (`0x0042_00E1`)
    ValidationAuthorityCountry = 0x0042_00E1,
    /// `Validation Authority URI` (`0x0042_00E2`)
    ValidationAuthorityURI = 0x0042_00E2,
    /// `Validation Version Major` (`0x0042_00E3`)
    ValidationVersionMajor = 0x0042_00E3,
    /// `Validation Version Minor` (`0x0042_00E4`)
    ValidationVersionMinor = 0x0042_00E4,
    /// `Validation Type` (`0x0042_00E5`)
    ValidationType = 0x0042_00E5,
    /// `Validation Level` (`0x0042_00E6`)
    ValidationLevel = 0x0042_00E6,
    /// `Validation Certificate Identifier` (`0x0042_00E7`)
    ValidationCertificateIdentifier = 0x0042_00E7,
    /// `Validation Certificate URI` (`0x0042_00E8`)
    ValidationCertificateURI = 0x0042_00E8,
    /// `Validation Vendor URI` (`0x0042_00E9`)
    ValidationVendorURI = 0x0042_00E9,
    /// `Validation Profile` (`0x0042_00EA`)
    ValidationProfile = 0x0042_00EA,
    /// `Profile Information` (`0x0042_00EB`)
    ProfileInformation = 0x0042_00EB,
    /// `Profile Name` (`0x0042_00EC`)
    ProfileName = 0x0042_00EC,
    /// `Server URI` (`0x0042_00ED`)
    ServerURI = 0x0042_00ED,
    /// `Server Port` (`0x0042_00EE`)
    ServerPort = 0x0042_00EE,
    /// `Streaming Capability` (`0x0042_00EF`)
    StreamingCapability = 0x0042_00EF,
    /// `Asynchronous Capability` (`0x0042_00F0`)
    AsynchronousCapability = 0x0042_00F0,
    /// `Attestation Capability` (`0x0042_00F1`)
    AttestationCapability = 0x0042_00F1,
    /// `Unwrap Mode` (`0x0042_00F2`)
    UnwrapMode = 0x0042_00F2,
    /// `Destroy Action` (`0x0042_00F3`)
    DestroyAction = 0x0042_00F3,
    /// `Shredding Algorithm` (`0x0042_00F4`)
    ShreddingAlgorithm = 0x0042_00F4,
    /// `RNG Mode` (`0x0042_00F5`)
    RNGMode = 0x0042_00F5,
    /// `Client Registration Method` (`0x0042_00F6`)
    ClientRegistrationMethod = 0x0042_00F6,
    /// `Capability Information` (`0x0042_00F7`)
    CapabilityInformation = 0x0042_00F7,
    /// `Key Wrap Type` (`0x0042_00F8`)
    KeyWrapType = 0x0042_00F8,
    /// `Batch Undo Capability` (`0x0042_00F9`)
    BatchUndoCapability = 0x0042_00F9,
    /// `Batch Continue Capability` (`0x0042_00FA`)
    BatchContinueCapability = 0x0042_00FA,
    /// `PKCS#12 Friendly Name` (`0x0042_00FB`)
    PKCS12FriendlyName = 0x0042_00FB,
    /// `Description` (`0x0042_00FC`)
    Description = 0x0042_00FC,
    /// `Comment` (`0x0042_00FD`)
    Comment = 0x0042_00FD,
    /// `Authenticated Encryption Additional Data` (`0x0042_00FE`)
    AuthenticatedEncryptionAdditionalData = 0x0042_00FE,
    /// `Authenticated Encryption Tag` (`0x0042_00FF`)
    AuthenticatedEncryptionTag = 0x0042_00FF,
    /// `Salt Length` (`0x0042_0100`)
    SaltLength = 0x0042_0100,
    /// `Mask Generator` (`0x0042_0101`)
    MaskGenerator = 0x0042_0101,
    /// `Mask Generator Hashing Algorithm` (`0x0042_0102`)
    MaskGeneratorHashingAlgorithm = 0x0042_0102,
    /// `P Source` (`0x0042_0103`)
    PSource = 0x0042_0103,
    /// `Trailer Field` (`0x0042_0104`)
    TrailerField = 0x0042_0104,
    /// `Client Correlation Value` (`0x0042_0105`)
    ClientCorrelationValue = 0x0042_0105,
    /// `Server Correlation Value` (`0x0042_0106`)
    ServerCorrelationValue = 0x0042_0106,
    /// `Digested Data` (`0x0042_0107`)
    DigestedData = 0x0042_0107,
    /// `Certificate Subject CN` (`0x0042_0108`)
    CertificateSubjectCN = 0x0042_0108,
    /// `Certificate Subject O` (`0x0042_0109`)
    CertificateSubjectO = 0x0042_0109,
    /// `Certificate Subject OU` (`0x0042_010A`)
    CertificateSubjectOU = 0x0042_010A,
    /// `Certificate Subject Email` (`0x0042_010B`)
    CertificateSubjectEmail = 0x0042_010B,
    /// `Certificate Subject C` (`0x0042_010C`)
    CertificateSubjectC = 0x0042_010C,
    /// `Certificate Subject ST` (`0x0042_010D`)
    CertificateSubjectST = 0x0042_010D,
    /// `Certificate Subject L` (`0x0042_010E`)
    CertificateSubjectL = 0x0042_010E,
    /// `Certificate Subject UID` (`0x0042_010F`)
    CertificateSubjectUID = 0x0042_010F,
    /// `Certificate Subject Serial Number` (`0x0042_0110`)
    CertificateSubjectSerialNumber = 0x0042_0110,
    /// `Certificate Subject Title` (`0x0042_0111`)
    CertificateSubjectTitle = 0x0042_0111,
    /// `Certificate Subject DC` (`0x0042_0112`)
    CertificateSubjectDC = 0x0042_0112,
    /// `Certificate Subject DN Qualifier` (`0x0042_0113`)
    CertificateSubjectDNQualifier = 0x0042_0113,
    /// `Certificate Issuer CN` (`0x0042_0114`)
    CertificateIssuerCN = 0x0042_0114,
    /// `Certificate Issuer O` (`0x0042_0115`)
    CertificateIssuerO = 0x0042_0115,
    /// `Certificate Issuer OU` (`0x0042_0116`)
    CertificateIssuerOU = 0x0042_0116,
    /// `Certificate Issuer Email` (`0x0042_0117`)
    CertificateIssuerEmail = 0x0042_0117,
    /// `Certificate Issuer C` (`0x0042_0118`)
    CertificateIssuerC = 0x0042_0118,
    /// `Certificate Issuer ST` (`0x0042_0119`)
    CertificateIssuerST = 0x0042_0119,
    /// `Certificate Issuer L` (`0x0042_011A`)
    CertificateIssuerL = 0x0042_011A,
    /// `Certificate Issuer UID` (`0x0042_011B`)
    CertificateIssuerUID = 0x0042_011B,
    /// `Certificate Issuer Serial Number` (`0x0042_011C`)
    CertificateIssuerSerialNumber = 0x0042_011C,
    /// `Certificate Issuer Title` (`0x0042_011D`)
    CertificateIssuerTitle = 0x0042_011D,
    /// `Certificate Issuer DC` (`0x0042_011E`)
    CertificateIssuerDC = 0x0042_011E,
    /// `Certificate Issuer DN Qualifier` (`0x0042_011F`)
    CertificateIssuerDNQualifier = 0x0042_011F,
    /// `Sensitive` (`0x0042_0120`)
    Sensitive = 0x0042_0120,
    /// `Always Sensitive` (`0x0042_0121`)
    AlwaysSensitive = 0x0042_0121,
    /// `Extractable` (`0x0042_0122`)
    Extractable = 0x0042_0122,
    /// `Never Extractable` (`0x0042_0123`)
    NeverExtractable = 0x0042_0123,
    /// `Replace Existing` (`0x0042_0124`)
    ReplaceExisting = 0x0042_0124,
    /// `Attributes` (`0x0042_0125`)
    Attributes = 0x0042_0125,
    /// `Common Attributes` (`0x0042_0126`)
    CommonAttributes = 0x0042_0126,
    /// `Private Key Attributes` (`0x0042_0127`)
    PrivateKeyAttributes = 0x0042_0127,
    /// `Public Key Attributes` (`0x0042_0128`)
    PublicKeyAttributes = 0x0042_0128,
    /// `Extension Enumeration` (`0x0042_0129`)
    ExtensionEnumeration = 0x0042_0129,
    /// `Extension Attribute` (`0x0042_012A`)
    ExtensionAttribute = 0x0042_012A,
    /// `Extension Parent Structure Tag` (`0x0042_012B`)
    ExtensionParentStructureTag = 0x0042_012B,
    /// `Extension Description` (`0x0042_012C`)
    ExtensionDescription = 0x0042_012C,
    /// `Server Name` (`0x0042_012D`)
    ServerName = 0x0042_012D,
    /// `Server Serial Number` (`0x0042_012E`)
    ServerSerialNumber = 0x0042_012E,
    /// `Server Version` (`0x0042_012F`)
    ServerVersion = 0x0042_012F,
    /// `Server Load` (`0x0042_0130`)
    ServerLoad = 0x0042_0130,
    /// `Product Name` (`0x0042_0131`)
    ProductName = 0x0042_0131,
    /// `Build Level` (`0x0042_0132`)
    BuildLevel = 0x0042_0132,
    /// `Build Date` (`0x0042_0133`)
    BuildDate = 0x0042_0133,
    /// `Cluster Info` (`0x0042_0134`)
    ClusterInfo = 0x0042_0134,
    /// `Alternate Failover Endpoints` (`0x0042_0135`)
    AlternateFailoverEndpoints = 0x0042_0135,
    /// `Short Unique Identifier` (`0x0042_0136`)
    ShortUniqueIdentifier = 0x0042_0136,
    /// `Tag` (`0x0042_0138`)
    Tag = 0x0042_0138,
    /// `Certificate Request Unique Identifier` (`0x0042_0139`)
    CertificateRequestUniqueIdentifier = 0x0042_0139,
    /// `NIST Key Type` (`0x0042_013A`)
    NISTKeyType = 0x0042_013A,
    /// `Attribute Reference` (`0x0042_013B`)
    AttributeReference = 0x0042_013B,
    /// `Current Attribute` (`0x0042_013C`)
    CurrentAttribute = 0x0042_013C,
    /// `New Attribute` (`0x0042_013D`)
    NewAttribute = 0x0042_013D,
    /// `Certificate Request Value` (`0x0042_0140`)
    CertificateRequestValue = 0x0042_0140,
    /// `Log Message` (`0x0042_0141`)
    LogMessage = 0x0042_0141,
    /// `Profile Version` (`0x0042_0142`)
    ProfileVersion = 0x0042_0142,
    /// `Profile Version Major` (`0x0042_0143`)
    ProfileVersionMajor = 0x0042_0143,
    /// `Profile Version Minor` (`0x0042_0144`)
    ProfileVersionMinor = 0x0042_0144,
    /// `Protection Level` (`0x0042_0145`)
    ProtectionLevel = 0x0042_0145,
    /// `Protection Period` (`0x0042_0146`)
    ProtectionPeriod = 0x0042_0146,
    /// `Quantum Safe` (`0x0042_0147`)
    QuantumSafe = 0x0042_0147,
    /// `Quantum Safe Capability` (`0x0042_0148`)
    QuantumSafeCapability = 0x0042_0148,
    /// `Ticket` (`0x0042_0149`)
    Ticket = 0x0042_0149,
    /// `Ticket Type` (`0x0042_014A`)
    TicketType = 0x0042_014A,
    /// `Ticket Value` (`0x0042_014B`)
    TicketValue = 0x0042_014B,
    /// `Request Count` (`0x0042_014C`)
    RequestCount = 0x0042_014C,
    /// `Rights` (`0x0042_014D`)
    Rights = 0x0042_014D,
    /// `Objects` (`0x0042_014E`)
    Objects = 0x0042_014E,
    /// `Operations` (`0x0042_014F`)
    Operations = 0x0042_014F,
    /// `Right` (`0x0042_0150`)
    Right = 0x0042_0150,
    /// `Endpoint Role` (`0x0042_0151`)
    EndpointRole = 0x0042_0151,
    /// `Defaults Information` (`0x0042_0152`)
    DefaultsInformation = 0x0042_0152,
    /// `Object Defaults` (`0x0042_0153`)
    ObjectDefaults = 0x0042_0153,
    /// `Ephemeral` (`0x0042_0154`)
    Ephemeral = 0x0042_0154,
    /// `Server Hashed Password` (`0x0042_0155`)
    ServerHashedPassword = 0x0042_0155,
    /// `One Time Password` (`0x0042_0156`)
    OneTimePassword = 0x0042_0156,
    /// `Hashed Password` (`0x0042_0157`)
    HashedPassword = 0x0042_0157,
    /// `Adjustment Type` (`0x0042_0158`)
    AdjustmentType = 0x0042_0158,
    /// `PKCS#11 Interface` (`0x0042_0159`)
    PKCS11Interface = 0x0042_0159,
    /// `PKCS#11 Function` (`0x0042_015A`)
    PKCS11Function = 0x0042_015A,
    /// `PKCS#11 Input Parameters` (`0x0042_015B`)
    PKCS11InputParameters = 0x0042_015B,
    /// `PKCS#11 Output Parameters` (`0x0042_015C`)
    PKCS11OutputParameters = 0x0042_015C,
    /// `PKCS#11 Return Code` (`0x0042_015D`)
    PKCS11ReturnCode = 0x0042_015D,
    /// `Protection Storage Mask` (`0x0042_015E`)
    ProtectionStorageMask = 0x0042_015E,
    /// `Protection Storage Masks` (`0x0042_015F`)
    ProtectionStorageMasks = 0x0042_015F,
    /// `Interop Function` (`0x0042_0160`)
    InteropFunction = 0x0042_0160,
    /// `Interop Identifier` (`0x0042_0161`)
    InteropIdentifier = 0x0042_0161,
    /// `Adjustment Value` (`0x0042_0162`)
    AdjustmentValue = 0x0042_0162,
    /// `Common Protection Storage Masks` (`0x0042_0163`)
    CommonProtectionStorageMasks = 0x0042_0163,
    /// `Private Protection Storage Masks` (`0x0042_0164`)
    PrivateProtectionStorageMasks = 0x0042_0164,
    /// `Public Protection Storage Masks` (`0x0042_0165`)
    PublicProtectionStorageMasks = 0x0042_0165,
    /// `Object Groups` (`0x0042_0166`)
    ObjectGroups = 0x0042_0166,
    /// `Object Types` (`0x0042_0167`)
    ObjectTypes = 0x0042_0167,
    /// `Constraints` (`0x0042_0168`)
    Constraints = 0x0042_0168,
    /// `Constraint` (`0x0042_0169`)
    Constraint = 0x0042_0169,
    /// `Rotate Interval` (`0x0042_016A`)
    RotateInterval = 0x0042_016A,
    /// `Rotate Automatic` (`0x0042_016B`)
    RotateAutomatic = 0x0042_016B,
    /// `Rotate Offset` (`0x0042_016C`)
    RotateOffset = 0x0042_016C,
    /// `Rotate Date` (`0x0042_016D`)
    RotateDate = 0x0042_016D,
    /// `Rotate Generation` (`0x0042_016E`)
    RotateGeneration = 0x0042_016E,
    /// `Rotate Name` (`0x0042_016F`)
    RotateName = 0x0042_016F,
    /// `Rotate Name Value` (`0x0042_0170`)
    RotateNameValue = 0x0042_0170,
    /// `Rotate Name Type` (`0x0042_0171`)
    RotateNameType = 0x0042_0171,
    /// `Rotate Latest` (`0x0042_0172`)
    RotateLatest = 0x0042_0172,
    /// `Asynchronous Request` (`0x0042_0173`)
    AsynchronousRequest = 0x0042_0173,
    /// `Submission Date` (`0x0042_0174`)
    SubmissionDate = 0x0042_0174,
    /// `Processing Stage` (`0x0042_0175`)
    ProcessingStage = 0x0042_0175,
    /// `Asynchronous Correlation Values` (`0x0042_0176`)
    AsynchronousCorrelationValues = 0x0042_0176,
    /// `Certificate Link` (`0x0042_0190`)
    CertificateLink = 0x0042_0190,
    /// `Child Link` (`0x0042_0191`)
    ChildLink = 0x0042_0191,
    /// `Derivation Base Object Link` (`0x0042_0192`)
    DerivationBaseObjectLink = 0x0042_0192,
    /// `Derived Object Link` (`0x0042_0193`)
    DerivedObjectLink = 0x0042_0193,
    /// `Next Link` (`0x0042_0194`)
    NextLink = 0x0042_0194,
    /// `Parent Link` (`0x0042_0195`)
    ParentLink = 0x0042_0195,
    /// `PKCS#12 Certificate Link` (`0x0042_0196`)
    PKCS12CertificateLink = 0x0042_0196,
    /// `PKCS#12 Password Link` (`0x0042_0197`)
    PKCS12PasswordLink = 0x0042_0197,
    /// `Previous Link` (`0x0042_0198`)
    PreviousLink = 0x0042_0198,
    /// `Private Key Link` (`0x0042_0199`)
    PrivateKeyLink = 0x0042_0199,
    /// `Public Key Link` (`0x0042_019A`)
    PublicKeyLink = 0x0042_019A,
    /// `Replaced Object Link` (`0x0042_019B`)
    ReplacedObjectLink = 0x0042_019B,
    /// `Replacement Object Link` (`0x0042_019C`)
    ReplacementObjectLink = 0x0042_019C,
    /// `Wrapping Key Link` (`0x0042_019D`)
    WrappingKeyLink = 0x0042_019D,
    /// `Object Class` (`0x0042_019E`)
    ObjectClass = 0x0042_019E,
    /// `Object Class Mask` (`0x0042_019F`)
    ObjectClassMask = 0x0042_019F,
    /// `Credential Link` (`0x0042_01A0`)
    CredentialLink = 0x0042_01A0,
    /// `Password Credential` (`0x0042_01A1`)
    PasswordCredential = 0x0042_01A1,
    /// `Password Salt` (`0x0042_01A2`)
    PasswordSalt = 0x0042_01A2,
    /// `Password Salt Algorithm` (`0x0042_01A3`)
    PasswordSaltAlgorithm = 0x0042_01A3,
    /// `Salted Password` (`0x0042_01A4`)
    SaltedPassword = 0x0042_01A4,
    /// `Password Link` (`0x0042_01A5`)
    PasswordLink = 0x0042_01A5,
    /// `Device Credential` (`0x0042_01A6`)
    DeviceCredential = 0x0042_01A6,
    /// `OTP Credential` (`0x0042_01A7`)
    OTPCredential = 0x0042_01A7,
    /// `OTP Algorithm` (`0x0042_01A8`)
    OTPAlgorithm = 0x0042_01A8,
    /// `OTP Digest` (`0x0042_01A9`)
    OTPDigest = 0x0042_01A9,
    /// `OTP Serial` (`0x0042_01AA`)
    OTPSerial = 0x0042_01AA,
    /// `OTP Seed` (`0x0042_01AB`)
    OTPSeed = 0x0042_01AB,
    /// `OTP Interval` (`0x0042_01AC`)
    OTPInterval = 0x0042_01AC,
    /// `OTP Digits` (`0x0042_01AD`)
    OTPDigits = 0x0042_01AD,
    /// `OTP Counter` (`0x0042_01AE`)
    OTPCounter = 0x0042_01AE,
    /// `Hashed Password Credential` (`0x0042_01AF`)
    HashedPasswordCredential = 0x0042_01AF,
    /// `Hashed Username Password` (`0x0042_01B0`)
    HashedUsernamePassword = 0x0042_01B0,
    /// `Hashed Password Username` (`0x0042_01B1`)
    HashedPasswordUsername = 0x0042_01B1,
    /// `Credential Information` (`0x0042_01B2`)
    CredentialInformation = 0x0042_01B2,
    /// `Group Link` (`0x0042_01B3`)
    GroupLink = 0x0042_01B3,
    /// `Split Key Base Link` (`0x0042_01B4`)
    SplitKeyBaseLink = 0x0042_01B4,
    /// `Joined Split Key Parts Link` (`0x0042_01B5`)
    JoinedSplitKeyPartsLink = 0x0042_01B5,
    /// `Split Key Polynomial` (`0x0042_01B6`)
    SplitKeyPolynomial = 0x0042_01B6,
    /// `Deactivation Message` (`0x0042_01B7`)
    DeactivationMessage = 0x0042_01B7,
    /// `Deactivation Reason` (`0x0042_01B8`)
    DeactivationReason = 0x0042_01B8,
    /// `Deactivation Reason Code` (`0x0042_01B9`)
    DeactivationReasonCode = 0x0042_01B9,
    /// `Certificate Subject DN` (`0x0042_01BA`)
    CertificateSubjectDN = 0x0042_01BA,
    /// `Certificate Issuer DN` (`0x0042_01BB`)
    CertificateIssuerDN = 0x0042_01BB,
    /// `Certificate Request Link` (`0x0042_01BC`)
    CertificateRequestLink = 0x0042_01BC,
    /// `Certify Counter` (`0x0042_01BD`)
    CertifyCounter = 0x0042_01BD,
    /// `Decrypt Counter` (`0x0042_01BE`)
    DecryptCounter = 0x0042_01BE,
    /// `Encrypt Counter` (`0x0042_01BF`)
    EncryptCounter = 0x0042_01BF,
    /// `Sign Counter` (`0x0042_01C0`)
    SignCounter = 0x0042_01C0,
    /// `Signature Verify Counter` (`0x0042_01C1`)
    SignatureVerifyCounter = 0x0042_01C1,
    /// `NIST Security Category` (`0x0042_01C2`)
    NISTSecurityCategory = 0x0042_01C2,
    /// `KEM Algorithm` (`0x0042_01C3`)
    KEMAlgorithm = 0x0042_01C3,
    /// `Deterministic` (`0x0042_01C4`)
    Deterministic = 0x0042_01C4,
    /// `Context String` (`0x0042_01C5`)
    ContextString = 0x0042_01C5,
    /// `Seed` (`0x0042_01C6`)
    Seed = 0x0042_01C6,
    /// `Input Key Material` (`0x0042_01C7`)
    InputKeyMaterial = 0x0042_01C7,
    /// `Internal` (`0x0042_01C8`)
    Internal = 0x0042_01C8,
    /// `External Mu` (`0x0042_01C9`)
    ExternalMu = 0x0042_01C9,
    /// `Random` (`0x0042_01CA`)
    Random = 0x0042_01CA,
    /// `Performs CRL Checks` (`0x0042_01CB`)
    PerformsCRLChecks = 0x0042_01CB,
}

impl KmipTag {
    /// Returns the 24-bit integer representation of this tag.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Returns the 24-bit integer representation of this tag (synonym for `as_u32`).
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self as u32
    }

    /// Returns the official normative OASIS name for this tag.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ActivationDate => "Activation Date",
            Self::ApplicationData => "Application Data",
            Self::ApplicationNamespace => "Application Namespace",
            Self::ApplicationSpecificInformation => "Application Specific Information",
            Self::ArchiveDate => "Archive Date",
            Self::AsynchronousCorrelationValue => "Asynchronous Correlation Value",
            Self::AsynchronousIndicator => "Asynchronous Indicator",
            Self::Attribute => "Attribute",
            Self::AttributeName => "Attribute Name",
            Self::AttributeValue => "Attribute Value",
            Self::Authentication => "Authentication",
            Self::BatchCount => "Batch Count",
            Self::BatchErrorContinuationOption => "Batch Error Continuation Option",
            Self::BatchItem => "Batch Item",
            Self::BatchOrderOption => "Batch Order Option",
            Self::BlockCipherMode => "Block Cipher Mode",
            Self::CancellationResult => "Cancellation Result",
            Self::Certificate => "Certificate",
            Self::CertificateRequest => "Certificate Request",
            Self::CertificateRequestType => "Certificate Request Type",
            Self::CertificateType => "Certificate Type",
            Self::CertificateValue => "Certificate Value",
            Self::CompromiseDate => "Compromise Date",
            Self::CompromiseOccurrenceDate => "Compromise Occurrence Date",
            Self::ContactInformation => "Contact Information",
            Self::Credential => "Credential",
            Self::CredentialType => "Credential Type",
            Self::CredentialValue => "Credential Value",
            Self::CriticalityIndicator => "Criticality Indicator",
            Self::CRTCoefficient => "CRT Coefficient",
            Self::CryptographicAlgorithm => "Cryptographic Algorithm",
            Self::CryptographicDomainParameters => "Cryptographic Domain Parameters",
            Self::CryptographicLength => "Cryptographic Length",
            Self::CryptographicParameters => "Cryptographic Parameters",
            Self::CryptographicUsageMask => "Cryptographic Usage Mask",
            Self::D => "D",
            Self::DeactivationDate => "Deactivation Date",
            Self::DerivationData => "Derivation Data",
            Self::DerivationMethod => "Derivation Method",
            Self::DerivationParameters => "Derivation Parameters",
            Self::DestroyDate => "Destroy Date",
            Self::Digest => "Digest",
            Self::DigestValue => "Digest Value",
            Self::EncryptionKeyInformation => "Encryption Key Information",
            Self::G => "G",
            Self::HashingAlgorithm => "Hashing Algorithm",
            Self::InitialDate => "Initial Date",
            Self::InitializationVector => "Initialization Vector",
            Self::IterationCount => "Iteration Count",
            Self::IVCounterNonce => "IV/Counter/Nonce",
            Self::J => "J",
            Self::Key => "Key",
            Self::KeyBlock => "Key Block",
            Self::KeyCompressionType => "Key Compression Type",
            Self::KeyFormatType => "Key Format Type",
            Self::KeyMaterial => "Key Material",
            Self::KeyPartIdentifier => "Key Part Identifier",
            Self::KeyValue => "Key Value",
            Self::KeyWrappingData => "Key Wrapping Data",
            Self::KeyWrappingSpecification => "Key Wrapping Specification",
            Self::LastChangeDate => "Last Change Date",
            Self::LeaseTime => "Lease Time",
            Self::Link => "Link",
            Self::LinkType => "Link Type",
            Self::LinkedObjectIdentifier => "Linked Object Identifier",
            Self::MACSignature => "MAC/Signature",
            Self::MACSignatureKeyInformation => "MAC/Signature Key Information",
            Self::MaximumItems => "Maximum Items",
            Self::MaximumResponseSize => "Maximum Response Size",
            Self::MessageExtension => "Message Extension",
            Self::Modulus => "Modulus",
            Self::Name => "Name",
            Self::NameType => "Name Type",
            Self::NameValue => "Name Value",
            Self::ObjectGroup => "Object Group",
            Self::ObjectType => "Object Type",
            Self::Offset => "Offset",
            Self::OpaqueDataType => "Opaque Data Type",
            Self::OpaqueDataValue => "Opaque Data Value",
            Self::OpaqueObject => "Opaque Object",
            Self::Operation => "Operation",
            Self::P => "P",
            Self::PaddingMethod => "Padding Method",
            Self::PrimeExponentP => "Prime Exponent P",
            Self::PrimeExponentQ => "Prime Exponent Q",
            Self::PrimeFieldSize => "Prime Field Size",
            Self::PrivateExponent => "Private Exponent",
            Self::PrivateKey => "Private Key",
            Self::PrivateKeyUniqueIdentifier => "Private Key Unique Identifier",
            Self::ProcessStartDate => "Process Start Date",
            Self::ProtectStopDate => "Protect Stop Date",
            Self::ProtocolVersion => "Protocol Version",
            Self::ProtocolVersionMajor => "Protocol Version Major",
            Self::ProtocolVersionMinor => "Protocol Version Minor",
            Self::PublicExponent => "Public Exponent",
            Self::PublicKey => "Public Key",
            Self::PublicKeyUniqueIdentifier => "Public Key Unique Identifier",
            Self::PutFunction => "Put Function",
            Self::Q => "Q",
            Self::QString => "Q String",
            Self::Qlength => "Qlength",
            Self::QueryFunction => "Query Function",
            Self::RecommendedCurve => "Recommended Curve",
            Self::ReplacedUniqueIdentifier => "Replaced Unique Identifier",
            Self::RequestHeader => "Request Header",
            Self::RequestMessage => "Request Message",
            Self::RequestPayload => "Request Payload",
            Self::ResponseHeader => "Response Header",
            Self::ResponseMessage => "Response Message",
            Self::ResponsePayload => "Response Payload",
            Self::ResultMessage => "Result Message",
            Self::ResultReason => "Result Reason",
            Self::ResultStatus => "Result Status",
            Self::RevocationMessage => "Revocation Message",
            Self::RevocationReason => "Revocation Reason",
            Self::RevocationReasonCode => "Revocation Reason Code",
            Self::KeyRoleType => "Key Role Type",
            Self::Salt => "Salt",
            Self::SecretData => "Secret Data",
            Self::SecretDataType => "Secret Data Type",
            Self::ServerInformation => "Server Information",
            Self::SplitKey => "Split Key",
            Self::SplitKeyMethod => "Split Key Method",
            Self::SplitKeyParts => "Split Key Parts",
            Self::SplitKeyThreshold => "Split Key Threshold",
            Self::State => "State",
            Self::StorageStatusMask => "Storage Status Mask",
            Self::SymmetricKey => "Symmetric Key",
            Self::TimeStamp => "Time Stamp",
            Self::UniqueBatchItemID => "Unique Batch Item ID",
            Self::UniqueIdentifier => "Unique Identifier",
            Self::UsageLimits => "Usage Limits",
            Self::UsageLimitsCount => "Usage Limits Count",
            Self::UsageLimitsTotal => "Usage Limits Total",
            Self::UsageLimitsUnit => "Usage Limits Unit",
            Self::Username => "Username",
            Self::ValidityDate => "Validity Date",
            Self::ValidityIndicator => "Validity Indicator",
            Self::VendorExtension => "Vendor Extension",
            Self::VendorIdentification => "Vendor Identification",
            Self::WrappingMethod => "Wrapping Method",
            Self::X => "X",
            Self::Y => "Y",
            Self::Password => "Password",
            Self::DeviceIdentifier => "Device Identifier",
            Self::EncodingOption => "Encoding Option",
            Self::ExtensionInformation => "Extension Information",
            Self::ExtensionName => "Extension Name",
            Self::ExtensionTag => "Extension Tag",
            Self::ExtensionType => "Extension Type",
            Self::Fresh => "Fresh",
            Self::MachineIdentifier => "Machine Identifier",
            Self::MediaIdentifier => "Media Identifier",
            Self::NetworkIdentifier => "Network Identifier",
            Self::ObjectGroupMember => "Object Group Member",
            Self::CertificateLength => "Certificate Length",
            Self::DigitalSignatureAlgorithm => "Digital Signature Algorithm",
            Self::CertificateSerialNumber => "Certificate Serial Number",
            Self::DeviceSerialNumber => "Device Serial Number",
            Self::IssuerAlternativeName => "Issuer Alternative Name",
            Self::IssuerDistinguishedName => "Issuer Distinguished Name",
            Self::SubjectAlternativeName => "Subject Alternative Name",
            Self::SubjectDistinguishedName => "Subject Distinguished Name",
            Self::X509CertificateIdentifier => "X.509 Certificate Identifier",
            Self::X509CertificateIssuer => "X.509 Certificate Issuer",
            Self::X509CertificateSubject => "X.509 Certificate Subject",
            Self::KeyValueLocation => "Key Value Location",
            Self::KeyValueLocationValue => "Key Value Location Value",
            Self::KeyValueLocationType => "Key Value Location Type",
            Self::KeyValuePresent => "Key Value Present",
            Self::OriginalCreationDate => "Original Creation Date",
            Self::PGPKey => "PGP Key",
            Self::PGPKeyVersion => "PGP Key Version",
            Self::AlternativeName => "Alternative Name",
            Self::AlternativeNameValue => "Alternative Name Value",
            Self::AlternativeNameType => "Alternative Name Type",
            Self::Data => "Data",
            Self::SignatureData => "Signature Data",
            Self::DataLength => "Data Length",
            Self::RandomIV => "Random IV",
            Self::MACData => "MAC Data",
            Self::AttestationType => "Attestation Type",
            Self::Nonce => "Nonce",
            Self::NonceID => "Nonce ID",
            Self::NonceValue => "Nonce Value",
            Self::AttestationMeasurement => "Attestation Measurement",
            Self::AttestationAssertion => "Attestation Assertion",
            Self::IVLength => "IV Length",
            Self::TagLength => "Tag Length",
            Self::FixedFieldLength => "Fixed Field Length",
            Self::CounterLength => "Counter Length",
            Self::InitialCounterValue => "Initial Counter Value",
            Self::InvocationFieldLength => "Invocation Field Length",
            Self::AttestationCapableIndicator => "Attestation Capable Indicator",
            Self::OffsetItems => "Offset Items",
            Self::LocatedItems => "Located Items",
            Self::CorrelationValue => "Correlation Value",
            Self::InitIndicator => "Init Indicator",
            Self::FinalIndicator => "Final Indicator",
            Self::RNGParameters => "RNG Parameters",
            Self::RNGAlgorithm => "RNG Algorithm",
            Self::DRBGAlgorithm => "DRBG Algorithm",
            Self::FIPS186Variation => "FIPS186 Variation",
            Self::PredictionResistance => "Prediction Resistance",
            Self::RandomNumberGenerator => "Random Number Generator",
            Self::ValidationInformation => "Validation Information",
            Self::ValidationAuthorityType => "Validation Authority Type",
            Self::ValidationAuthorityCountry => "Validation Authority Country",
            Self::ValidationAuthorityURI => "Validation Authority URI",
            Self::ValidationVersionMajor => "Validation Version Major",
            Self::ValidationVersionMinor => "Validation Version Minor",
            Self::ValidationType => "Validation Type",
            Self::ValidationLevel => "Validation Level",
            Self::ValidationCertificateIdentifier => "Validation Certificate Identifier",
            Self::ValidationCertificateURI => "Validation Certificate URI",
            Self::ValidationVendorURI => "Validation Vendor URI",
            Self::ValidationProfile => "Validation Profile",
            Self::ProfileInformation => "Profile Information",
            Self::ProfileName => "Profile Name",
            Self::ServerURI => "Server URI",
            Self::ServerPort => "Server Port",
            Self::StreamingCapability => "Streaming Capability",
            Self::AsynchronousCapability => "Asynchronous Capability",
            Self::AttestationCapability => "Attestation Capability",
            Self::UnwrapMode => "Unwrap Mode",
            Self::DestroyAction => "Destroy Action",
            Self::ShreddingAlgorithm => "Shredding Algorithm",
            Self::RNGMode => "RNG Mode",
            Self::ClientRegistrationMethod => "Client Registration Method",
            Self::CapabilityInformation => "Capability Information",
            Self::KeyWrapType => "Key Wrap Type",
            Self::BatchUndoCapability => "Batch Undo Capability",
            Self::BatchContinueCapability => "Batch Continue Capability",
            Self::PKCS12FriendlyName => "PKCS#12 Friendly Name",
            Self::Description => "Description",
            Self::Comment => "Comment",
            Self::AuthenticatedEncryptionAdditionalData => {
                "Authenticated Encryption Additional Data"
            }
            Self::AuthenticatedEncryptionTag => "Authenticated Encryption Tag",
            Self::SaltLength => "Salt Length",
            Self::MaskGenerator => "Mask Generator",
            Self::MaskGeneratorHashingAlgorithm => "Mask Generator Hashing Algorithm",
            Self::PSource => "P Source",
            Self::TrailerField => "Trailer Field",
            Self::ClientCorrelationValue => "Client Correlation Value",
            Self::ServerCorrelationValue => "Server Correlation Value",
            Self::DigestedData => "Digested Data",
            Self::CertificateSubjectCN => "Certificate Subject CN",
            Self::CertificateSubjectO => "Certificate Subject O",
            Self::CertificateSubjectOU => "Certificate Subject OU",
            Self::CertificateSubjectEmail => "Certificate Subject Email",
            Self::CertificateSubjectC => "Certificate Subject C",
            Self::CertificateSubjectST => "Certificate Subject ST",
            Self::CertificateSubjectL => "Certificate Subject L",
            Self::CertificateSubjectUID => "Certificate Subject UID",
            Self::CertificateSubjectSerialNumber => "Certificate Subject Serial Number",
            Self::CertificateSubjectTitle => "Certificate Subject Title",
            Self::CertificateSubjectDC => "Certificate Subject DC",
            Self::CertificateSubjectDNQualifier => "Certificate Subject DN Qualifier",
            Self::CertificateIssuerCN => "Certificate Issuer CN",
            Self::CertificateIssuerO => "Certificate Issuer O",
            Self::CertificateIssuerOU => "Certificate Issuer OU",
            Self::CertificateIssuerEmail => "Certificate Issuer Email",
            Self::CertificateIssuerC => "Certificate Issuer C",
            Self::CertificateIssuerST => "Certificate Issuer ST",
            Self::CertificateIssuerL => "Certificate Issuer L",
            Self::CertificateIssuerUID => "Certificate Issuer UID",
            Self::CertificateIssuerSerialNumber => "Certificate Issuer Serial Number",
            Self::CertificateIssuerTitle => "Certificate Issuer Title",
            Self::CertificateIssuerDC => "Certificate Issuer DC",
            Self::CertificateIssuerDNQualifier => "Certificate Issuer DN Qualifier",
            Self::Sensitive => "Sensitive",
            Self::AlwaysSensitive => "Always Sensitive",
            Self::Extractable => "Extractable",
            Self::NeverExtractable => "Never Extractable",
            Self::ReplaceExisting => "Replace Existing",
            Self::Attributes => "Attributes",
            Self::CommonAttributes => "Common Attributes",
            Self::PrivateKeyAttributes => "Private Key Attributes",
            Self::PublicKeyAttributes => "Public Key Attributes",
            Self::ExtensionEnumeration => "Extension Enumeration",
            Self::ExtensionAttribute => "Extension Attribute",
            Self::ExtensionParentStructureTag => "Extension Parent Structure Tag",
            Self::ExtensionDescription => "Extension Description",
            Self::ServerName => "Server Name",
            Self::ServerSerialNumber => "Server Serial Number",
            Self::ServerVersion => "Server Version",
            Self::ServerLoad => "Server Load",
            Self::ProductName => "Product Name",
            Self::BuildLevel => "Build Level",
            Self::BuildDate => "Build Date",
            Self::ClusterInfo => "Cluster Info",
            Self::AlternateFailoverEndpoints => "Alternate Failover Endpoints",
            Self::ShortUniqueIdentifier => "Short Unique Identifier",
            Self::Tag => "Tag",
            Self::CertificateRequestUniqueIdentifier => "Certificate Request Unique Identifier",
            Self::NISTKeyType => "NIST Key Type",
            Self::AttributeReference => "Attribute Reference",
            Self::CurrentAttribute => "Current Attribute",
            Self::NewAttribute => "New Attribute",
            Self::CertificateRequestValue => "Certificate Request Value",
            Self::LogMessage => "Log Message",
            Self::ProfileVersion => "Profile Version",
            Self::ProfileVersionMajor => "Profile Version Major",
            Self::ProfileVersionMinor => "Profile Version Minor",
            Self::ProtectionLevel => "Protection Level",
            Self::ProtectionPeriod => "Protection Period",
            Self::QuantumSafe => "Quantum Safe",
            Self::QuantumSafeCapability => "Quantum Safe Capability",
            Self::Ticket => "Ticket",
            Self::TicketType => "Ticket Type",
            Self::TicketValue => "Ticket Value",
            Self::RequestCount => "Request Count",
            Self::Rights => "Rights",
            Self::Objects => "Objects",
            Self::Operations => "Operations",
            Self::Right => "Right",
            Self::EndpointRole => "Endpoint Role",
            Self::DefaultsInformation => "Defaults Information",
            Self::ObjectDefaults => "Object Defaults",
            Self::Ephemeral => "Ephemeral",
            Self::ServerHashedPassword => "Server Hashed Password",
            Self::OneTimePassword => "One Time Password",
            Self::HashedPassword => "Hashed Password",
            Self::AdjustmentType => "Adjustment Type",
            Self::PKCS11Interface => "PKCS#11 Interface",
            Self::PKCS11Function => "PKCS#11 Function",
            Self::PKCS11InputParameters => "PKCS#11 Input Parameters",
            Self::PKCS11OutputParameters => "PKCS#11 Output Parameters",
            Self::PKCS11ReturnCode => "PKCS#11 Return Code",
            Self::ProtectionStorageMask => "Protection Storage Mask",
            Self::ProtectionStorageMasks => "Protection Storage Masks",
            Self::InteropFunction => "Interop Function",
            Self::InteropIdentifier => "Interop Identifier",
            Self::AdjustmentValue => "Adjustment Value",
            Self::CommonProtectionStorageMasks => "Common Protection Storage Masks",
            Self::PrivateProtectionStorageMasks => "Private Protection Storage Masks",
            Self::PublicProtectionStorageMasks => "Public Protection Storage Masks",
            Self::ObjectGroups => "Object Groups",
            Self::ObjectTypes => "Object Types",
            Self::Constraints => "Constraints",
            Self::Constraint => "Constraint",
            Self::RotateInterval => "Rotate Interval",
            Self::RotateAutomatic => "Rotate Automatic",
            Self::RotateOffset => "Rotate Offset",
            Self::RotateDate => "Rotate Date",
            Self::RotateGeneration => "Rotate Generation",
            Self::RotateName => "Rotate Name",
            Self::RotateNameValue => "Rotate Name Value",
            Self::RotateNameType => "Rotate Name Type",
            Self::RotateLatest => "Rotate Latest",
            Self::AsynchronousRequest => "Asynchronous Request",
            Self::SubmissionDate => "Submission Date",
            Self::ProcessingStage => "Processing Stage",
            Self::AsynchronousCorrelationValues => "Asynchronous Correlation Values",
            Self::CertificateLink => "Certificate Link",
            Self::ChildLink => "Child Link",
            Self::DerivationBaseObjectLink => "Derivation Base Object Link",
            Self::DerivedObjectLink => "Derived Object Link",
            Self::NextLink => "Next Link",
            Self::ParentLink => "Parent Link",
            Self::PKCS12CertificateLink => "PKCS#12 Certificate Link",
            Self::PKCS12PasswordLink => "PKCS#12 Password Link",
            Self::PreviousLink => "Previous Link",
            Self::PrivateKeyLink => "Private Key Link",
            Self::PublicKeyLink => "Public Key Link",
            Self::ReplacedObjectLink => "Replaced Object Link",
            Self::ReplacementObjectLink => "Replacement Object Link",
            Self::WrappingKeyLink => "Wrapping Key Link",
            Self::ObjectClass => "Object Class",
            Self::ObjectClassMask => "Object Class Mask",
            Self::CredentialLink => "Credential Link",
            Self::PasswordCredential => "Password Credential",
            Self::PasswordSalt => "Password Salt",
            Self::PasswordSaltAlgorithm => "Password Salt Algorithm",
            Self::SaltedPassword => "Salted Password",
            Self::PasswordLink => "Password Link",
            Self::DeviceCredential => "Device Credential",
            Self::OTPCredential => "OTP Credential",
            Self::OTPAlgorithm => "OTP Algorithm",
            Self::OTPDigest => "OTP Digest",
            Self::OTPSerial => "OTP Serial",
            Self::OTPSeed => "OTP Seed",
            Self::OTPInterval => "OTP Interval",
            Self::OTPDigits => "OTP Digits",
            Self::OTPCounter => "OTP Counter",
            Self::HashedPasswordCredential => "Hashed Password Credential",
            Self::HashedUsernamePassword => "Hashed Username Password",
            Self::HashedPasswordUsername => "Hashed Password Username",
            Self::CredentialInformation => "Credential Information",
            Self::GroupLink => "Group Link",
            Self::SplitKeyBaseLink => "Split Key Base Link",
            Self::JoinedSplitKeyPartsLink => "Joined Split Key Parts Link",
            Self::SplitKeyPolynomial => "Split Key Polynomial",
            Self::DeactivationMessage => "Deactivation Message",
            Self::DeactivationReason => "Deactivation Reason",
            Self::DeactivationReasonCode => "Deactivation Reason Code",
            Self::CertificateSubjectDN => "Certificate Subject DN",
            Self::CertificateIssuerDN => "Certificate Issuer DN",
            Self::CertificateRequestLink => "Certificate Request Link",
            Self::CertifyCounter => "Certify Counter",
            Self::DecryptCounter => "Decrypt Counter",
            Self::EncryptCounter => "Encrypt Counter",
            Self::SignCounter => "Sign Counter",
            Self::SignatureVerifyCounter => "Signature Verify Counter",
            Self::NISTSecurityCategory => "NIST Security Category",
            Self::KEMAlgorithm => "KEM Algorithm",
            Self::Deterministic => "Deterministic",
            Self::ContextString => "Context String",
            Self::Seed => "Seed",
            Self::InputKeyMaterial => "Input Key Material",
            Self::Internal => "Internal",
            Self::ExternalMu => "External Mu",
            Self::Random => "Random",
            Self::PerformsCRLChecks => "Performs CRL Checks",
        }
    }

    /// Resolves a 24-bit raw integer tag to a known official `KmipTag` if catalogued.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub const fn from_u32(raw: u32) -> Option<Self> {
        match raw {
            0x0042_0001 => Some(Self::ActivationDate),
            0x0042_0002 => Some(Self::ApplicationData),
            0x0042_0003 => Some(Self::ApplicationNamespace),
            0x0042_0004 => Some(Self::ApplicationSpecificInformation),
            0x0042_0005 => Some(Self::ArchiveDate),
            0x0042_0006 => Some(Self::AsynchronousCorrelationValue),
            0x0042_0007 => Some(Self::AsynchronousIndicator),
            0x0042_0008 => Some(Self::Attribute),
            0x0042_000A => Some(Self::AttributeName),
            0x0042_000B => Some(Self::AttributeValue),
            0x0042_000C => Some(Self::Authentication),
            0x0042_000D => Some(Self::BatchCount),
            0x0042_000E => Some(Self::BatchErrorContinuationOption),
            0x0042_000F => Some(Self::BatchItem),
            0x0042_0010 => Some(Self::BatchOrderOption),
            0x0042_0011 => Some(Self::BlockCipherMode),
            0x0042_0012 => Some(Self::CancellationResult),
            0x0042_0013 => Some(Self::Certificate),
            0x0042_0018 => Some(Self::CertificateRequest),
            0x0042_0019 => Some(Self::CertificateRequestType),
            0x0042_001D => Some(Self::CertificateType),
            0x0042_001E => Some(Self::CertificateValue),
            0x0042_0020 => Some(Self::CompromiseDate),
            0x0042_0021 => Some(Self::CompromiseOccurrenceDate),
            0x0042_0022 => Some(Self::ContactInformation),
            0x0042_0023 => Some(Self::Credential),
            0x0042_0024 => Some(Self::CredentialType),
            0x0042_0025 => Some(Self::CredentialValue),
            0x0042_0026 => Some(Self::CriticalityIndicator),
            0x0042_0027 => Some(Self::CRTCoefficient),
            0x0042_0028 => Some(Self::CryptographicAlgorithm),
            0x0042_0029 => Some(Self::CryptographicDomainParameters),
            0x0042_002A => Some(Self::CryptographicLength),
            0x0042_002B => Some(Self::CryptographicParameters),
            0x0042_002C => Some(Self::CryptographicUsageMask),
            0x0042_002E => Some(Self::D),
            0x0042_002F => Some(Self::DeactivationDate),
            0x0042_0030 => Some(Self::DerivationData),
            0x0042_0031 => Some(Self::DerivationMethod),
            0x0042_0032 => Some(Self::DerivationParameters),
            0x0042_0033 => Some(Self::DestroyDate),
            0x0042_0034 => Some(Self::Digest),
            0x0042_0035 => Some(Self::DigestValue),
            0x0042_0036 => Some(Self::EncryptionKeyInformation),
            0x0042_0037 => Some(Self::G),
            0x0042_0038 => Some(Self::HashingAlgorithm),
            0x0042_0039 => Some(Self::InitialDate),
            0x0042_003A => Some(Self::InitializationVector),
            0x0042_003C => Some(Self::IterationCount),
            0x0042_003D => Some(Self::IVCounterNonce),
            0x0042_003E => Some(Self::J),
            0x0042_003F => Some(Self::Key),
            0x0042_0040 => Some(Self::KeyBlock),
            0x0042_0041 => Some(Self::KeyCompressionType),
            0x0042_0042 => Some(Self::KeyFormatType),
            0x0042_0043 => Some(Self::KeyMaterial),
            0x0042_0044 => Some(Self::KeyPartIdentifier),
            0x0042_0045 => Some(Self::KeyValue),
            0x0042_0046 => Some(Self::KeyWrappingData),
            0x0042_0047 => Some(Self::KeyWrappingSpecification),
            0x0042_0048 => Some(Self::LastChangeDate),
            0x0042_0049 => Some(Self::LeaseTime),
            0x0042_004A => Some(Self::Link),
            0x0042_004B => Some(Self::LinkType),
            0x0042_004C => Some(Self::LinkedObjectIdentifier),
            0x0042_004D => Some(Self::MACSignature),
            0x0042_004E => Some(Self::MACSignatureKeyInformation),
            0x0042_004F => Some(Self::MaximumItems),
            0x0042_0050 => Some(Self::MaximumResponseSize),
            0x0042_0051 => Some(Self::MessageExtension),
            0x0042_0052 => Some(Self::Modulus),
            0x0042_0053 => Some(Self::Name),
            0x0042_0054 => Some(Self::NameType),
            0x0042_0055 => Some(Self::NameValue),
            0x0042_0056 => Some(Self::ObjectGroup),
            0x0042_0057 => Some(Self::ObjectType),
            0x0042_0058 => Some(Self::Offset),
            0x0042_0059 => Some(Self::OpaqueDataType),
            0x0042_005A => Some(Self::OpaqueDataValue),
            0x0042_005B => Some(Self::OpaqueObject),
            0x0042_005C => Some(Self::Operation),
            0x0042_005E => Some(Self::P),
            0x0042_005F => Some(Self::PaddingMethod),
            0x0042_0060 => Some(Self::PrimeExponentP),
            0x0042_0061 => Some(Self::PrimeExponentQ),
            0x0042_0062 => Some(Self::PrimeFieldSize),
            0x0042_0063 => Some(Self::PrivateExponent),
            0x0042_0064 => Some(Self::PrivateKey),
            0x0042_0066 => Some(Self::PrivateKeyUniqueIdentifier),
            0x0042_0067 => Some(Self::ProcessStartDate),
            0x0042_0068 => Some(Self::ProtectStopDate),
            0x0042_0069 => Some(Self::ProtocolVersion),
            0x0042_006A => Some(Self::ProtocolVersionMajor),
            0x0042_006B => Some(Self::ProtocolVersionMinor),
            0x0042_006C => Some(Self::PublicExponent),
            0x0042_006D => Some(Self::PublicKey),
            0x0042_006F => Some(Self::PublicKeyUniqueIdentifier),
            0x0042_0070 => Some(Self::PutFunction),
            0x0042_0071 => Some(Self::Q),
            0x0042_0072 => Some(Self::QString),
            0x0042_0073 => Some(Self::Qlength),
            0x0042_0074 => Some(Self::QueryFunction),
            0x0042_0075 => Some(Self::RecommendedCurve),
            0x0042_0076 => Some(Self::ReplacedUniqueIdentifier),
            0x0042_0077 => Some(Self::RequestHeader),
            0x0042_0078 => Some(Self::RequestMessage),
            0x0042_0079 => Some(Self::RequestPayload),
            0x0042_007A => Some(Self::ResponseHeader),
            0x0042_007B => Some(Self::ResponseMessage),
            0x0042_007C => Some(Self::ResponsePayload),
            0x0042_007D => Some(Self::ResultMessage),
            0x0042_007E => Some(Self::ResultReason),
            0x0042_007F => Some(Self::ResultStatus),
            0x0042_0080 => Some(Self::RevocationMessage),
            0x0042_0081 => Some(Self::RevocationReason),
            0x0042_0082 => Some(Self::RevocationReasonCode),
            0x0042_0083 => Some(Self::KeyRoleType),
            0x0042_0084 => Some(Self::Salt),
            0x0042_0085 => Some(Self::SecretData),
            0x0042_0086 => Some(Self::SecretDataType),
            0x0042_0088 => Some(Self::ServerInformation),
            0x0042_0089 => Some(Self::SplitKey),
            0x0042_008A => Some(Self::SplitKeyMethod),
            0x0042_008B => Some(Self::SplitKeyParts),
            0x0042_008C => Some(Self::SplitKeyThreshold),
            0x0042_008D => Some(Self::State),
            0x0042_008E => Some(Self::StorageStatusMask),
            0x0042_008F => Some(Self::SymmetricKey),
            0x0042_0092 => Some(Self::TimeStamp),
            0x0042_0093 => Some(Self::UniqueBatchItemID),
            0x0042_0094 => Some(Self::UniqueIdentifier),
            0x0042_0095 => Some(Self::UsageLimits),
            0x0042_0096 => Some(Self::UsageLimitsCount),
            0x0042_0097 => Some(Self::UsageLimitsTotal),
            0x0042_0098 => Some(Self::UsageLimitsUnit),
            0x0042_0099 => Some(Self::Username),
            0x0042_009A => Some(Self::ValidityDate),
            0x0042_009B => Some(Self::ValidityIndicator),
            0x0042_009C => Some(Self::VendorExtension),
            0x0042_009D => Some(Self::VendorIdentification),
            0x0042_009E => Some(Self::WrappingMethod),
            0x0042_009F => Some(Self::X),
            0x0042_00A0 => Some(Self::Y),
            0x0042_00A1 => Some(Self::Password),
            0x0042_00A2 => Some(Self::DeviceIdentifier),
            0x0042_00A3 => Some(Self::EncodingOption),
            0x0042_00A4 => Some(Self::ExtensionInformation),
            0x0042_00A5 => Some(Self::ExtensionName),
            0x0042_00A6 => Some(Self::ExtensionTag),
            0x0042_00A7 => Some(Self::ExtensionType),
            0x0042_00A8 => Some(Self::Fresh),
            0x0042_00A9 => Some(Self::MachineIdentifier),
            0x0042_00AA => Some(Self::MediaIdentifier),
            0x0042_00AB => Some(Self::NetworkIdentifier),
            0x0042_00AC => Some(Self::ObjectGroupMember),
            0x0042_00AD => Some(Self::CertificateLength),
            0x0042_00AE => Some(Self::DigitalSignatureAlgorithm),
            0x0042_00AF => Some(Self::CertificateSerialNumber),
            0x0042_00B0 => Some(Self::DeviceSerialNumber),
            0x0042_00B1 => Some(Self::IssuerAlternativeName),
            0x0042_00B2 => Some(Self::IssuerDistinguishedName),
            0x0042_00B3 => Some(Self::SubjectAlternativeName),
            0x0042_00B4 => Some(Self::SubjectDistinguishedName),
            0x0042_00B5 => Some(Self::X509CertificateIdentifier),
            0x0042_00B6 => Some(Self::X509CertificateIssuer),
            0x0042_00B7 => Some(Self::X509CertificateSubject),
            0x0042_00B8 => Some(Self::KeyValueLocation),
            0x0042_00B9 => Some(Self::KeyValueLocationValue),
            0x0042_00BA => Some(Self::KeyValueLocationType),
            0x0042_00BB => Some(Self::KeyValuePresent),
            0x0042_00BC => Some(Self::OriginalCreationDate),
            0x0042_00BD => Some(Self::PGPKey),
            0x0042_00BE => Some(Self::PGPKeyVersion),
            0x0042_00BF => Some(Self::AlternativeName),
            0x0042_00C0 => Some(Self::AlternativeNameValue),
            0x0042_00C1 => Some(Self::AlternativeNameType),
            0x0042_00C2 => Some(Self::Data),
            0x0042_00C3 => Some(Self::SignatureData),
            0x0042_00C4 => Some(Self::DataLength),
            0x0042_00C5 => Some(Self::RandomIV),
            0x0042_00C6 => Some(Self::MACData),
            0x0042_00C7 => Some(Self::AttestationType),
            0x0042_00C8 => Some(Self::Nonce),
            0x0042_00C9 => Some(Self::NonceID),
            0x0042_00CA => Some(Self::NonceValue),
            0x0042_00CB => Some(Self::AttestationMeasurement),
            0x0042_00CC => Some(Self::AttestationAssertion),
            0x0042_00CD => Some(Self::IVLength),
            0x0042_00CE => Some(Self::TagLength),
            0x0042_00CF => Some(Self::FixedFieldLength),
            0x0042_00D0 => Some(Self::CounterLength),
            0x0042_00D1 => Some(Self::InitialCounterValue),
            0x0042_00D2 => Some(Self::InvocationFieldLength),
            0x0042_00D3 => Some(Self::AttestationCapableIndicator),
            0x0042_00D4 => Some(Self::OffsetItems),
            0x0042_00D5 => Some(Self::LocatedItems),
            0x0042_00D6 => Some(Self::CorrelationValue),
            0x0042_00D7 => Some(Self::InitIndicator),
            0x0042_00D8 => Some(Self::FinalIndicator),
            0x0042_00D9 => Some(Self::RNGParameters),
            0x0042_00DA => Some(Self::RNGAlgorithm),
            0x0042_00DB => Some(Self::DRBGAlgorithm),
            0x0042_00DC => Some(Self::FIPS186Variation),
            0x0042_00DD => Some(Self::PredictionResistance),
            0x0042_00DE => Some(Self::RandomNumberGenerator),
            0x0042_00DF => Some(Self::ValidationInformation),
            0x0042_00E0 => Some(Self::ValidationAuthorityType),
            0x0042_00E1 => Some(Self::ValidationAuthorityCountry),
            0x0042_00E2 => Some(Self::ValidationAuthorityURI),
            0x0042_00E3 => Some(Self::ValidationVersionMajor),
            0x0042_00E4 => Some(Self::ValidationVersionMinor),
            0x0042_00E5 => Some(Self::ValidationType),
            0x0042_00E6 => Some(Self::ValidationLevel),
            0x0042_00E7 => Some(Self::ValidationCertificateIdentifier),
            0x0042_00E8 => Some(Self::ValidationCertificateURI),
            0x0042_00E9 => Some(Self::ValidationVendorURI),
            0x0042_00EA => Some(Self::ValidationProfile),
            0x0042_00EB => Some(Self::ProfileInformation),
            0x0042_00EC => Some(Self::ProfileName),
            0x0042_00ED => Some(Self::ServerURI),
            0x0042_00EE => Some(Self::ServerPort),
            0x0042_00EF => Some(Self::StreamingCapability),
            0x0042_00F0 => Some(Self::AsynchronousCapability),
            0x0042_00F1 => Some(Self::AttestationCapability),
            0x0042_00F2 => Some(Self::UnwrapMode),
            0x0042_00F3 => Some(Self::DestroyAction),
            0x0042_00F4 => Some(Self::ShreddingAlgorithm),
            0x0042_00F5 => Some(Self::RNGMode),
            0x0042_00F6 => Some(Self::ClientRegistrationMethod),
            0x0042_00F7 => Some(Self::CapabilityInformation),
            0x0042_00F8 => Some(Self::KeyWrapType),
            0x0042_00F9 => Some(Self::BatchUndoCapability),
            0x0042_00FA => Some(Self::BatchContinueCapability),
            0x0042_00FB => Some(Self::PKCS12FriendlyName),
            0x0042_00FC => Some(Self::Description),
            0x0042_00FD => Some(Self::Comment),
            0x0042_00FE => Some(Self::AuthenticatedEncryptionAdditionalData),
            0x0042_00FF => Some(Self::AuthenticatedEncryptionTag),
            0x0042_0100 => Some(Self::SaltLength),
            0x0042_0101 => Some(Self::MaskGenerator),
            0x0042_0102 => Some(Self::MaskGeneratorHashingAlgorithm),
            0x0042_0103 => Some(Self::PSource),
            0x0042_0104 => Some(Self::TrailerField),
            0x0042_0105 => Some(Self::ClientCorrelationValue),
            0x0042_0106 => Some(Self::ServerCorrelationValue),
            0x0042_0107 => Some(Self::DigestedData),
            0x0042_0108 => Some(Self::CertificateSubjectCN),
            0x0042_0109 => Some(Self::CertificateSubjectO),
            0x0042_010A => Some(Self::CertificateSubjectOU),
            0x0042_010B => Some(Self::CertificateSubjectEmail),
            0x0042_010C => Some(Self::CertificateSubjectC),
            0x0042_010D => Some(Self::CertificateSubjectST),
            0x0042_010E => Some(Self::CertificateSubjectL),
            0x0042_010F => Some(Self::CertificateSubjectUID),
            0x0042_0110 => Some(Self::CertificateSubjectSerialNumber),
            0x0042_0111 => Some(Self::CertificateSubjectTitle),
            0x0042_0112 => Some(Self::CertificateSubjectDC),
            0x0042_0113 => Some(Self::CertificateSubjectDNQualifier),
            0x0042_0114 => Some(Self::CertificateIssuerCN),
            0x0042_0115 => Some(Self::CertificateIssuerO),
            0x0042_0116 => Some(Self::CertificateIssuerOU),
            0x0042_0117 => Some(Self::CertificateIssuerEmail),
            0x0042_0118 => Some(Self::CertificateIssuerC),
            0x0042_0119 => Some(Self::CertificateIssuerST),
            0x0042_011A => Some(Self::CertificateIssuerL),
            0x0042_011B => Some(Self::CertificateIssuerUID),
            0x0042_011C => Some(Self::CertificateIssuerSerialNumber),
            0x0042_011D => Some(Self::CertificateIssuerTitle),
            0x0042_011E => Some(Self::CertificateIssuerDC),
            0x0042_011F => Some(Self::CertificateIssuerDNQualifier),
            0x0042_0120 => Some(Self::Sensitive),
            0x0042_0121 => Some(Self::AlwaysSensitive),
            0x0042_0122 => Some(Self::Extractable),
            0x0042_0123 => Some(Self::NeverExtractable),
            0x0042_0124 => Some(Self::ReplaceExisting),
            0x0042_0125 => Some(Self::Attributes),
            0x0042_0126 => Some(Self::CommonAttributes),
            0x0042_0127 => Some(Self::PrivateKeyAttributes),
            0x0042_0128 => Some(Self::PublicKeyAttributes),
            0x0042_0129 => Some(Self::ExtensionEnumeration),
            0x0042_012A => Some(Self::ExtensionAttribute),
            0x0042_012B => Some(Self::ExtensionParentStructureTag),
            0x0042_012C => Some(Self::ExtensionDescription),
            0x0042_012D => Some(Self::ServerName),
            0x0042_012E => Some(Self::ServerSerialNumber),
            0x0042_012F => Some(Self::ServerVersion),
            0x0042_0130 => Some(Self::ServerLoad),
            0x0042_0131 => Some(Self::ProductName),
            0x0042_0132 => Some(Self::BuildLevel),
            0x0042_0133 => Some(Self::BuildDate),
            0x0042_0134 => Some(Self::ClusterInfo),
            0x0042_0135 => Some(Self::AlternateFailoverEndpoints),
            0x0042_0136 => Some(Self::ShortUniqueIdentifier),
            0x0042_0138 => Some(Self::Tag),
            0x0042_0139 => Some(Self::CertificateRequestUniqueIdentifier),
            0x0042_013A => Some(Self::NISTKeyType),
            0x0042_013B => Some(Self::AttributeReference),
            0x0042_013C => Some(Self::CurrentAttribute),
            0x0042_013D => Some(Self::NewAttribute),
            0x0042_0140 => Some(Self::CertificateRequestValue),
            0x0042_0141 => Some(Self::LogMessage),
            0x0042_0142 => Some(Self::ProfileVersion),
            0x0042_0143 => Some(Self::ProfileVersionMajor),
            0x0042_0144 => Some(Self::ProfileVersionMinor),
            0x0042_0145 => Some(Self::ProtectionLevel),
            0x0042_0146 => Some(Self::ProtectionPeriod),
            0x0042_0147 => Some(Self::QuantumSafe),
            0x0042_0148 => Some(Self::QuantumSafeCapability),
            0x0042_0149 => Some(Self::Ticket),
            0x0042_014A => Some(Self::TicketType),
            0x0042_014B => Some(Self::TicketValue),
            0x0042_014C => Some(Self::RequestCount),
            0x0042_014D => Some(Self::Rights),
            0x0042_014E => Some(Self::Objects),
            0x0042_014F => Some(Self::Operations),
            0x0042_0150 => Some(Self::Right),
            0x0042_0151 => Some(Self::EndpointRole),
            0x0042_0152 => Some(Self::DefaultsInformation),
            0x0042_0153 => Some(Self::ObjectDefaults),
            0x0042_0154 => Some(Self::Ephemeral),
            0x0042_0155 => Some(Self::ServerHashedPassword),
            0x0042_0156 => Some(Self::OneTimePassword),
            0x0042_0157 => Some(Self::HashedPassword),
            0x0042_0158 => Some(Self::AdjustmentType),
            0x0042_0159 => Some(Self::PKCS11Interface),
            0x0042_015A => Some(Self::PKCS11Function),
            0x0042_015B => Some(Self::PKCS11InputParameters),
            0x0042_015C => Some(Self::PKCS11OutputParameters),
            0x0042_015D => Some(Self::PKCS11ReturnCode),
            0x0042_015E => Some(Self::ProtectionStorageMask),
            0x0042_015F => Some(Self::ProtectionStorageMasks),
            0x0042_0160 => Some(Self::InteropFunction),
            0x0042_0161 => Some(Self::InteropIdentifier),
            0x0042_0162 => Some(Self::AdjustmentValue),
            0x0042_0163 => Some(Self::CommonProtectionStorageMasks),
            0x0042_0164 => Some(Self::PrivateProtectionStorageMasks),
            0x0042_0165 => Some(Self::PublicProtectionStorageMasks),
            0x0042_0166 => Some(Self::ObjectGroups),
            0x0042_0167 => Some(Self::ObjectTypes),
            0x0042_0168 => Some(Self::Constraints),
            0x0042_0169 => Some(Self::Constraint),
            0x0042_016A => Some(Self::RotateInterval),
            0x0042_016B => Some(Self::RotateAutomatic),
            0x0042_016C => Some(Self::RotateOffset),
            0x0042_016D => Some(Self::RotateDate),
            0x0042_016E => Some(Self::RotateGeneration),
            0x0042_016F => Some(Self::RotateName),
            0x0042_0170 => Some(Self::RotateNameValue),
            0x0042_0171 => Some(Self::RotateNameType),
            0x0042_0172 => Some(Self::RotateLatest),
            0x0042_0173 => Some(Self::AsynchronousRequest),
            0x0042_0174 => Some(Self::SubmissionDate),
            0x0042_0175 => Some(Self::ProcessingStage),
            0x0042_0176 => Some(Self::AsynchronousCorrelationValues),
            0x0042_0190 => Some(Self::CertificateLink),
            0x0042_0191 => Some(Self::ChildLink),
            0x0042_0192 => Some(Self::DerivationBaseObjectLink),
            0x0042_0193 => Some(Self::DerivedObjectLink),
            0x0042_0194 => Some(Self::NextLink),
            0x0042_0195 => Some(Self::ParentLink),
            0x0042_0196 => Some(Self::PKCS12CertificateLink),
            0x0042_0197 => Some(Self::PKCS12PasswordLink),
            0x0042_0198 => Some(Self::PreviousLink),
            0x0042_0199 => Some(Self::PrivateKeyLink),
            0x0042_019A => Some(Self::PublicKeyLink),
            0x0042_019B => Some(Self::ReplacedObjectLink),
            0x0042_019C => Some(Self::ReplacementObjectLink),
            0x0042_019D => Some(Self::WrappingKeyLink),
            0x0042_019E => Some(Self::ObjectClass),
            0x0042_019F => Some(Self::ObjectClassMask),
            0x0042_01A0 => Some(Self::CredentialLink),
            0x0042_01A1 => Some(Self::PasswordCredential),
            0x0042_01A2 => Some(Self::PasswordSalt),
            0x0042_01A3 => Some(Self::PasswordSaltAlgorithm),
            0x0042_01A4 => Some(Self::SaltedPassword),
            0x0042_01A5 => Some(Self::PasswordLink),
            0x0042_01A6 => Some(Self::DeviceCredential),
            0x0042_01A7 => Some(Self::OTPCredential),
            0x0042_01A8 => Some(Self::OTPAlgorithm),
            0x0042_01A9 => Some(Self::OTPDigest),
            0x0042_01AA => Some(Self::OTPSerial),
            0x0042_01AB => Some(Self::OTPSeed),
            0x0042_01AC => Some(Self::OTPInterval),
            0x0042_01AD => Some(Self::OTPDigits),
            0x0042_01AE => Some(Self::OTPCounter),
            0x0042_01AF => Some(Self::HashedPasswordCredential),
            0x0042_01B0 => Some(Self::HashedUsernamePassword),
            0x0042_01B1 => Some(Self::HashedPasswordUsername),
            0x0042_01B2 => Some(Self::CredentialInformation),
            0x0042_01B3 => Some(Self::GroupLink),
            0x0042_01B4 => Some(Self::SplitKeyBaseLink),
            0x0042_01B5 => Some(Self::JoinedSplitKeyPartsLink),
            0x0042_01B6 => Some(Self::SplitKeyPolynomial),
            0x0042_01B7 => Some(Self::DeactivationMessage),
            0x0042_01B8 => Some(Self::DeactivationReason),
            0x0042_01B9 => Some(Self::DeactivationReasonCode),
            0x0042_01BA => Some(Self::CertificateSubjectDN),
            0x0042_01BB => Some(Self::CertificateIssuerDN),
            0x0042_01BC => Some(Self::CertificateRequestLink),
            0x0042_01BD => Some(Self::CertifyCounter),
            0x0042_01BE => Some(Self::DecryptCounter),
            0x0042_01BF => Some(Self::EncryptCounter),
            0x0042_01C0 => Some(Self::SignCounter),
            0x0042_01C1 => Some(Self::SignatureVerifyCounter),
            0x0042_01C2 => Some(Self::NISTSecurityCategory),
            0x0042_01C3 => Some(Self::KEMAlgorithm),
            0x0042_01C4 => Some(Self::Deterministic),
            0x0042_01C5 => Some(Self::ContextString),
            0x0042_01C6 => Some(Self::Seed),
            0x0042_01C7 => Some(Self::InputKeyMaterial),
            0x0042_01C8 => Some(Self::Internal),
            0x0042_01C9 => Some(Self::ExternalMu),
            0x0042_01CA => Some(Self::Random),
            0x0042_01CB => Some(Self::PerformsCRLChecks),
            _ => None,
        }
    }

    /// Converts this known tag into a checked [`Tag`].
    ///
    /// Because every variant of `KmipTag` is an allocated OASIS standard tag,
    /// this conversion is guaranteed to succeed.
    ///
    /// # Panics
    ///
    /// Panics only if a `KmipTag` variant has a value outside the 24-bit tag
    /// range or is not assigned by OASIS. Every declared variant satisfies
    /// both conditions by construction.
    #[inline]
    #[must_use]
    pub fn into_tag(self) -> Tag {
        RawTag::new(self as u32)
            .expect("KmipTag discriminant is within 24 bits")
            .try_checked()
            .expect("KmipTag variant is an assigned OASIS tag")
    }

    /// Converts this known tag into a [`RawTag`].
    ///
    /// # Panics
    ///
    /// Panics only if a `KmipTag` variant has a value outside the 24-bit tag
    /// range. Every declared variant satisfies this condition by construction.
    #[inline]
    #[must_use]
    pub fn into_raw_tag(self) -> RawTag {
        RawTag::new(self as u32).expect("KmipTag discriminant is within 24 bits")
    }
}

impl std::fmt::Display for KmipTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (0x{:06X})", self.name(), self.as_u32())
    }
}

impl From<KmipTag> for Tag {
    #[inline]
    fn from(tag: KmipTag) -> Self {
        tag.into_tag()
    }
}

impl From<KmipTag> for RawTag {
    #[inline]
    fn from(tag: KmipTag) -> Self {
        tag.into_raw_tag()
    }
}

impl TryFrom<RawTag> for KmipTag {
    type Error = ModelError;
    #[inline]
    fn try_from(tag: RawTag) -> Result<Self, Self::Error> {
        Self::from_u32(tag.raw()).ok_or(ModelError::TagNotAllocated)
    }
}

impl TryFrom<Tag> for KmipTag {
    type Error = ModelError;
    #[inline]
    fn try_from(tag: Tag) -> Result<Self, Self::Error> {
        Self::from_u32(tag.raw()).ok_or(ModelError::TagNotAllocated)
    }
}
