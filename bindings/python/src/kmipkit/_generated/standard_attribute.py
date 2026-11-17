"""Generated catalog of OASIS KMIP 2.0–3.0 direct object attributes."""

from enum import Enum


class StandardAttribute(Enum):
    """Assigned tag, TTLV value kind, and source multiplicity for an attribute."""
    ACTIVATION_DATE = (0x420001, "DateTime", "No", "2.0", None)
    APPLICATION_SPECIFIC_INFORMATION = (0x420004, "Structure", "Yes", "2.0", None)
    ARCHIVE_DATE = (0x420005, "DateTime", "No", "2.0", None)
    VENDOR_ATTRIBUTE = (0x420008, "Structure", None, "2.0", None)
    CERTIFICATE_TYPE = (0x42001D, "Enumeration", "No", "2.0", None)
    COMPROMISE_DATE = (0x420020, "DateTime", "No", "2.0", None)
    COMPROMISE_OCCURRENCE_DATE = (0x420021, "DateTime", "No", "2.0", None)
    CONTACT_INFORMATION = (0x420022, "TextString", "No", "2.0", None)
    CREDENTIAL_TYPE = (0x420024, "Enumeration", "No", "3.0", None)
    CRYPTOGRAPHIC_ALGORITHM = (0x420028, "Enumeration", "No", "2.0", None)
    CRYPTOGRAPHIC_DOMAIN_PARAMETERS = (0x420029, "Structure", "No", "2.0", None)
    CRYPTOGRAPHIC_LENGTH = (0x42002A, "Integer", "No", "2.0", None)
    CRYPTOGRAPHIC_PARAMETERS = (0x42002B, "Structure", "Yes", "2.0", None)
    CRYPTOGRAPHIC_USAGE_MASK = (0x42002C, "Integer", "No", "2.0", None)
    DEACTIVATION_DATE = (0x42002F, "DateTime", "No", "2.0", None)
    DESTROY_DATE = (0x420033, "DateTime", "No", "2.0", None)
    DIGEST = (0x420034, "Structure", "Yes", "2.0", None)
    INITIAL_DATE = (0x420039, "DateTime", "No", "2.0", None)
    KEY_FORMAT_TYPE = (0x420042, "Enumeration", "No", "2.0", None)
    KEY_PART_IDENTIFIER = (0x420044, "Integer", "No", "3.0", None)
    LAST_CHANGE_DATE = (0x420048, "DateTime", "No", "2.0", None)
    LEASE_TIME = (0x420049, "Interval", "No", "2.0", None)
    LINK = (0x42004A, "Structure", "Yes", "2.0", "2.1")
    NAME = (0x420053, "Structure", "Yes", "2.0", None)
    OBJECT_GROUP = (0x420056, "TextString", "Yes", "2.0", None)
    OBJECT_TYPE = (0x420057, "Enumeration", "No", "2.0", None)
    OPAQUE_DATA_TYPE = (0x420059, "Enumeration", "No", "2.0", None)
    PROCESS_START_DATE = (0x420067, "DateTime", "No", "2.0", None)
    PROTECT_STOP_DATE = (0x420068, "DateTime", "No", "2.0", None)
    REVOCATION_REASON = (0x420081, "Structure", "No", "2.0", None)
    SPLIT_KEY_METHOD = (0x42008A, "Enumeration", "No", "3.0", None)
    SPLIT_KEY_PARTS = (0x42008B, "Integer", "No", "3.0", None)
    SPLIT_KEY_THRESHOLD = (0x42008C, "Integer", "No", "3.0", None)
    STATE = (0x42008D, "Enumeration", "No", "2.0", None)
    UNIQUE_IDENTIFIER = (0x420094, "UniqueIdentifier", "No", "2.0", None)
    USAGE_LIMITS = (0x420095, "Structure", "No", "2.0", None)
    FRESH = (0x4200A8, "Boolean", "No", "2.0", None)
    CERTIFICATE_LENGTH = (0x4200AD, "Integer", "No", "2.0", None)
    DIGITAL_SIGNATURE_ALGORITHM = (0x4200AE, "Enumeration", "Yes for PGP keys. No for X.509 certificates.", "2.0", None)
    X509_CERTIFICATE_IDENTIFIER = (0x4200B5, "Structure", "No", "2.0", "2.1")
    X509_CERTIFICATE_ISSUER = (0x4200B6, "Structure", "No", "2.0", "2.1")
    X509_CERTIFICATE_SUBJECT = (0x4200B7, "Structure", "No", "2.0", "2.1")
    KEY_VALUE_LOCATION = (0x4200B8, "Structure", "Yes", "2.0", None)
    KEY_VALUE_PRESENT = (0x4200BB, "Boolean", "No", "2.0", None)
    ORIGINAL_CREATION_DATE = (0x4200BC, "DateTime", "No", "2.0", None)
    ALTERNATIVE_NAME = (0x4200BF, "Structure", "Yes", "2.0", None)
    RANDOM_NUMBER_GENERATOR = (0x4200DE, "Structure", "No", "2.0", None)
    PKCS12_FRIENDLY_NAME = (0x4200FB, "TextString", "No", "2.0", None)
    DESCRIPTION = (0x4200FC, "TextString", "No", "2.0", None)
    COMMENT = (0x4200FD, "TextString", "No", "2.0", None)
    SENSITIVE = (0x420120, "Boolean", "No", "2.0", None)
    ALWAYS_SENSITIVE = (0x420121, "Boolean", "No", "2.0", None)
    EXTRACTABLE = (0x420122, "Boolean", "No", "2.0", None)
    NEVER_EXTRACTABLE = (0x420123, "Boolean", "No", "2.0", None)
    SHORT_UNIQUE_IDENTIFIER = (0x420136, "ByteString", "No", "2.0", None)
    NIST_KEY_TYPE = (0x42013A, "Enumeration", "Yes", "2.0", None)
    PROTECTION_LEVEL = (0x420145, "Enumeration", "No", "2.0", None)
    PROTECTION_PERIOD = (0x420146, "Interval", "No", "2.0", None)
    QUANTUM_SAFE = (0x420147, "Boolean", "No", "2.0", None)
    PROTECTION_STORAGE_MASK = (0x42015E, "Integer", "No", "2.0", None)
    ROTATE_INTERVAL = (0x42016A, "Interval", "No", "2.1", None)
    ROTATE_AUTOMATIC = (0x42016B, "Boolean", "No", "2.1", None)
    ROTATE_OFFSET = (0x42016C, "Interval", "No", "2.1", None)
    ROTATE_DATE = (0x42016D, "DateTime", "No", "2.1", None)
    ROTATE_GENERATION = (0x42016E, "Integer", "No", "2.1", None)
    ROTATE_NAME = (0x42016F, "Structure", "No", "2.1", None)
    ROTATE_LATEST = (0x420172, "Boolean", "No", "2.1", None)
    CERTIFICATE_LINK = (0x420190, "UniqueIdentifier", "Yes", "3.0", None)
    CHILD_LINK = (0x420191, "UniqueIdentifier", "No", "3.0", None)
    DERIVATION_BASE_OBJECT_LINK = (0x420192, "UniqueIdentifier", "Yes", "3.0", None)
    DERIVED_OBJECT_LINK = (0x420193, "UniqueIdentifier", "Yes", "3.0", None)
    NEXT_LINK = (0x420194, "UniqueIdentifier", "No", "3.0", None)
    PARENT_LINK = (0x420195, "UniqueIdentifier", "Yes", "3.0", None)
    PKCS12_CERTIFICATE_LINK = (0x420196, "UniqueIdentifier", "No", "3.0", None)
    PKCS12_PASSWORD_LINK = (0x420197, "UniqueIdentifier", "No", "3.0", None)
    PREVIOUS_LINK = (0x420198, "UniqueIdentifier", "No", "3.0", None)
    PRIVATE_KEY_LINK = (0x420199, "UniqueIdentifier", "No", "3.0", None)
    PUBLIC_KEY_LINK = (0x42019A, "UniqueIdentifier", "No", "3.0", None)
    REPLACED_OBJECT_LINK = (0x42019B, "UniqueIdentifier", "No", "3.0", None)
    REPLACEMENT_OBJECT_LINK = (0x42019C, "UniqueIdentifier", "No", "3.0", None)
    WRAPPING_KEY_LINK = (0x42019D, "UniqueIdentifier", "No", "3.0", None)
    OBJECT_CLASS = (0x42019E, "Enumeration", "No", "3.0", None)
    CREDENTIAL_LINK = (0x4201A0, "UniqueIdentifier", "Yes", "3.0", None)
    PASSWORD_LINK = (0x4201A5, "UniqueIdentifier", "Yes", "3.0", None)
    OTP_COUNTER = (0x4201AE, "ByteString", "No", "3.0", None)
    GROUP_LINK = (0x4201B3, "UniqueIdentifier", "Yes", "3.0", None)
    SPLIT_KEY_BASE_LINK = (0x4201B4, "UniqueIdentifier", "Yes", "3.0", None)
    JOINED_SPLIT_KEY_PARTS_LINK = (0x4201B5, "UniqueIdentifier", "Yes", "3.0", None)
    SPLIT_KEY_POLYNOMIAL = (0x4201B6, "Enumeration", "No", "3.0", None)
    DEACTIVATION_REASON = (0x4201B8, "Structure", "No", "3.0", None)
    CERTIFICATE_SUBJECT_DN = (0x4201BA, "TextString", "No", "3.0", None)
    CERTIFICATE_ISSUER_DN = (0x4201BB, "TextString", "No", "3.0", None)
    CERTIFICATE_REQUEST_LINK = (0x4201BC, "UniqueIdentifier", "Yes", "3.0", None)
    CERTIFY_COUNTER = (0x4201BD, "LongInteger", "No", "3.0", None)
    DECRYPT_COUNTER = (0x4201BE, "LongInteger", "No", "3.0", None)
    ENCRYPT_COUNTER = (0x4201BF, "LongInteger", "No", "3.0", None)
    SIGN_COUNTER = (0x4201C0, "LongInteger", "No", "3.0", None)
    SIGNATURE_VERIFY_COUNTER = (0x4201C1, "LongInteger", "No", "3.0", None)
    NIST_SECURITY_CATEGORY = (0x4201C2, "Integer", "No", "3.0", None)

    @property
    def tag(self) -> int:
        return self.value[0]

    @property
    def value_kind(self) -> str:
        return self.value[1]

    @property
    def source_multiplicity(self) -> str | None:
        return self.value[2]

    @property
    def min_version(self) -> str:
        return self.value[3]

    @property
    def max_version(self) -> str | None:
        return self.value[4]
