package org.kmipkit.protocol;

/** Generated KMIP 2.1 named values. */
public final class KmipValues {
    private KmipValues() {}

    /** Named KMIP 2.1 values for Adjustment Type. */
    public static final class AdjustmentType {
        private AdjustmentType() {}
        /** Increment; KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-INCREMENT-00000001; KMIPKIT-SRC-spec §11.1. */
        public static final long INCREMENT = 0x00000001L;
        /** Decrement; KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-DECREMENT-00000002; KMIPKIT-SRC-spec §11.1. */
        public static final long DECREMENT = 0x00000002L;
        /** Negate; KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-NEGATE-00000003; KMIPKIT-SRC-spec §11.1. */
        public static final long NEGATE = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Increment";
                case 0x00000002 -> "Decrement";
                case 0x00000003 -> "Negate";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Alternative Name Type. */
    public static final class AlternativeNameType {
        private AlternativeNameType() {}
        /** Uninterpreted Text String; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.2. */
        public static final long UNINTERPRETED_TEXT_STRING = 0x00000001L;
        /** URI; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.2. */
        public static final long URI = 0x00000002L;
        /** Object Serial Number; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-OBJECT-SERIAL-NUMBER-00000003; KMIPKIT-SRC-spec §11.2. */
        public static final long OBJECT_SERIAL_NUMBER = 0x00000003L;
        /** Email Address; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-EMAIL-ADDRESS-00000004; KMIPKIT-SRC-spec §11.2. */
        public static final long EMAIL_ADDRESS = 0x00000004L;
        /** DNS Name; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-DNS-NAME-00000005; KMIPKIT-SRC-spec §11.2. */
        public static final long DNS_NAME = 0x00000005L;
        /** X.500 Distinguished Name; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-X-500-DISTINGUISHED-NAME-00000006; KMIPKIT-SRC-spec §11.2. */
        public static final long X_500_DISTINGUISHED_NAME = 0x00000006L;
        /** IP Address; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-IP-ADDRESS-00000007; KMIPKIT-SRC-spec §11.2. */
        public static final long IP_ADDRESS = 0x00000007L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Uninterpreted Text String";
                case 0x00000002 -> "URI";
                case 0x00000003 -> "Object Serial Number";
                case 0x00000004 -> "Email Address";
                case 0x00000005 -> "DNS Name";
                case 0x00000006 -> "X.500 Distinguished Name";
                case 0x00000007 -> "IP Address";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Asynchronous Indicator. */
    public static final class AsynchronousIndicator {
        private AsynchronousIndicator() {}
        /** Mandatory; KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-MANDATORY-00000001; KMIPKIT-SRC-spec §11.3. */
        public static final long MANDATORY = 0x00000001L;
        /** Optional; KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-OPTIONAL-00000002; KMIPKIT-SRC-spec §11.3. */
        public static final long OPTIONAL = 0x00000002L;
        /** Prohibited; KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-PROHIBITED-00000003; KMIPKIT-SRC-spec §11.3. */
        public static final long PROHIBITED = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Mandatory";
                case 0x00000002 -> "Optional";
                case 0x00000003 -> "Prohibited";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Attestation Type. */
    public static final class AttestationType {
        private AttestationType() {}
        /** TPM Quote; KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TPM-QUOTE-00000001; KMIPKIT-SRC-spec §11.4. */
        public static final long TPM_QUOTE = 0x00000001L;
        /** TCG Integrity Report; KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TCG-INTEGRITY-REPORT-00000002; KMIPKIT-SRC-spec §11.4. */
        public static final long TCG_INTEGRITY_REPORT = 0x00000002L;
        /** SAML Assertion; KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-SAML-ASSERTION-00000003; KMIPKIT-SRC-spec §11.4. */
        public static final long SAML_ASSERTION = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "TPM Quote";
                case 0x00000002 -> "TCG Integrity Report";
                case 0x00000003 -> "SAML Assertion";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Batch Error Continuation Option. */
    public static final class BatchErrorContinuationOption {
        private BatchErrorContinuationOption() {}
        /** Continue; KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-CONTINUE-00000001; KMIPKIT-SRC-spec §11.5. */
        public static final long CONTINUE = 0x00000001L;
        /** Stop; KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-STOP-00000002; KMIPKIT-SRC-spec §11.5. */
        public static final long STOP = 0x00000002L;
        /** Undo; KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-UNDO-00000003; KMIPKIT-SRC-spec §11.5. */
        public static final long UNDO = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Continue";
                case 0x00000002 -> "Stop";
                case 0x00000003 -> "Undo";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Block Cipher Mode. */
    public static final class BlockCipherMode {
        private BlockCipherMode() {}
        /** CBC; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-00000001; KMIPKIT-SRC-spec §11.6. */
        public static final long CBC = 0x00000001L;
        /** ECB; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-ECB-00000002; KMIPKIT-SRC-spec §11.6. */
        public static final long ECB = 0x00000002L;
        /** PCBC; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-PCBC-00000003; KMIPKIT-SRC-spec §11.6. */
        public static final long PCBC = 0x00000003L;
        /** CFB; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CFB-00000004; KMIPKIT-SRC-spec §11.6. */
        public static final long CFB = 0x00000004L;
        /** OFB; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-OFB-00000005; KMIPKIT-SRC-spec §11.6. */
        public static final long OFB = 0x00000005L;
        /** CTR; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CTR-00000006; KMIPKIT-SRC-spec §11.6. */
        public static final long CTR = 0x00000006L;
        /** CMAC; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CMAC-00000007; KMIPKIT-SRC-spec §11.6. */
        public static final long CMAC = 0x00000007L;
        /** CCM; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CCM-00000008; KMIPKIT-SRC-spec §11.6. */
        public static final long CCM = 0x00000008L;
        /** GCM; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-GCM-00000009; KMIPKIT-SRC-spec §11.6. */
        public static final long GCM = 0x00000009L;
        /** CBC-MAC; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-MAC-0000000A; KMIPKIT-SRC-spec §11.6. */
        public static final long CBC_MAC = 0x0000000AL;
        /** XTS; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-XTS-0000000B; KMIPKIT-SRC-spec §11.6. */
        public static final long XTS = 0x0000000BL;
        /** AESKeyWrapPadding; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AESKEYWRAPPADDING-0000000C; KMIPKIT-SRC-spec §11.6. */
        public static final long AESKEYWRAPPADDING = 0x0000000CL;
        /** NISTKeyWrap; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-NISTKEYWRAP-0000000D; KMIPKIT-SRC-spec §11.6. */
        public static final long NISTKEYWRAP = 0x0000000DL;
        /** X9.102 AESKW; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AESKW-0000000E; KMIPKIT-SRC-spec §11.6. */
        public static final long X9_102_AESKW = 0x0000000EL;
        /** X9.102 TDKW; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-TDKW-0000000F; KMIPKIT-SRC-spec §11.6. */
        public static final long X9_102_TDKW = 0x0000000FL;
        /** X9.102 AKW1; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW1-00000010; KMIPKIT-SRC-spec §11.6. */
        public static final long X9_102_AKW1 = 0x00000010L;
        /** X9.102 AKW2; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW2-00000011; KMIPKIT-SRC-spec §11.6. */
        public static final long X9_102_AKW2 = 0x00000011L;
        /** AEAD; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AEAD-00000012; KMIPKIT-SRC-spec §11.6. */
        public static final long AEAD = 0x00000012L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "CBC";
                case 0x00000002 -> "ECB";
                case 0x00000003 -> "PCBC";
                case 0x00000004 -> "CFB";
                case 0x00000005 -> "OFB";
                case 0x00000006 -> "CTR";
                case 0x00000007 -> "CMAC";
                case 0x00000008 -> "CCM";
                case 0x00000009 -> "GCM";
                case 0x0000000A -> "CBC-MAC";
                case 0x0000000B -> "XTS";
                case 0x0000000C -> "AESKeyWrapPadding";
                case 0x0000000D -> "NISTKeyWrap";
                case 0x0000000E -> "X9.102 AESKW";
                case 0x0000000F -> "X9.102 TDKW";
                case 0x00000010 -> "X9.102 AKW1";
                case 0x00000011 -> "X9.102 AKW2";
                case 0x00000012 -> "AEAD";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Cancellation Result. */
    public static final class CancellationResult {
        private CancellationResult() {}
        /** Canceled; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-CANCELED-00000001; KMIPKIT-SRC-spec §11.7. */
        public static final long CANCELED = 0x00000001L;
        /** Unable to Cancel; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNABLE-TO-CANCEL-00000002; KMIPKIT-SRC-spec §11.7. */
        public static final long UNABLE_TO_CANCEL = 0x00000002L;
        /** Completed; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-COMPLETED-00000003; KMIPKIT-SRC-spec §11.7. */
        public static final long COMPLETED = 0x00000003L;
        /** Failed; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-FAILED-00000004; KMIPKIT-SRC-spec §11.7. */
        public static final long FAILED = 0x00000004L;
        /** Unavailable; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNAVAILABLE-00000005; KMIPKIT-SRC-spec §11.7. */
        public static final long UNAVAILABLE = 0x00000005L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Canceled";
                case 0x00000002 -> "Unable to Cancel";
                case 0x00000003 -> "Completed";
                case 0x00000004 -> "Failed";
                case 0x00000005 -> "Unavailable";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Certificate Request Type. */
    public static final class CertificateRequestType {
        private CertificateRequestType() {}
        /** CRMF; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-CRMF-00000001; KMIPKIT-SRC-spec §11.8. */
        public static final long CRMF = 0x00000001L;
        /** PKCS#10; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PKCS-10-00000002; KMIPKIT-SRC-spec §11.8. */
        public static final long PKCS_10 = 0x00000002L;
        /** PEM; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PEM-00000003; KMIPKIT-SRC-spec §11.8. */
        public static final long PEM = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "CRMF";
                case 0x00000002 -> "PKCS#10";
                case 0x00000003 -> "PEM";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Certificate Type. */
    public static final class CertificateType {
        private CertificateType() {}
        /** X.509; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-X-509-00000001; KMIPKIT-SRC-spec §11.9. */
        public static final long X_509 = 0x00000001L;
        /** PGP; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-PGP-00000002; KMIPKIT-SRC-spec §11.9. */
        public static final long PGP = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "X.509";
                case 0x00000002 -> "PGP";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Client Registration Method. */
    public static final class ClientRegistrationMethod {
        private ClientRegistrationMethod() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.10. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Server Pre-Generated; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-PRE-GENERATED-00000002; KMIPKIT-SRC-spec §11.10. */
        public static final long SERVER_PRE_GENERATED = 0x00000002L;
        /** Server On-Demand; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-ON-DEMAND-00000003; KMIPKIT-SRC-spec §11.10. */
        public static final long SERVER_ON_DEMAND = 0x00000003L;
        /** Client Generated; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-GENERATED-00000004; KMIPKIT-SRC-spec §11.10. */
        public static final long CLIENT_GENERATED = 0x00000004L;
        /** Client Registered; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-REGISTERED-00000005; KMIPKIT-SRC-spec §11.10. */
        public static final long CLIENT_REGISTERED = 0x00000005L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Server Pre-Generated";
                case 0x00000003 -> "Server On-Demand";
                case 0x00000004 -> "Client Generated";
                case 0x00000005 -> "Client Registered";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Credential Type. */
    public static final class CredentialType {
        private CredentialType() {}
        /** Username and Password; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001; KMIPKIT-SRC-spec §11.11. */
        public static final long USERNAME_AND_PASSWORD = 0x00000001L;
        /** Device; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002; KMIPKIT-SRC-spec §11.11. */
        public static final long DEVICE = 0x00000002L;
        /** Attestation; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003; KMIPKIT-SRC-spec §11.11. */
        public static final long ATTESTATION = 0x00000003L;
        /** One Time Password; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ONE-TIME-PASSWORD-00000004; KMIPKIT-SRC-spec §11.11. */
        public static final long ONE_TIME_PASSWORD = 0x00000004L;
        /** Hashed Password; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005; KMIPKIT-SRC-spec §11.11. */
        public static final long HASHED_PASSWORD = 0x00000005L;
        /** Ticket; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-TICKET-00000006; KMIPKIT-SRC-spec §11.11. */
        public static final long TICKET = 0x00000006L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Username and Password";
                case 0x00000002 -> "Device";
                case 0x00000003 -> "Attestation";
                case 0x00000004 -> "One Time Password";
                case 0x00000005 -> "Hashed Password";
                case 0x00000006 -> "Ticket";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Cryptographic Algorithm. */
    public static final class CryptographicAlgorithm {
        private CryptographicAlgorithm() {}
        /** DES; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DES-00000001; KMIPKIT-SRC-spec §11.12. */
        public static final long DES = 0x00000001L;
        /** 3DES; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-3DES-00000002; KMIPKIT-SRC-spec §11.12. */
        public static final long VALUE_3DES = 0x00000002L;
        /** AES; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003; KMIPKIT-SRC-spec §11.12. */
        public static final long AES = 0x00000003L;
        /** RSA; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RSA-00000004; KMIPKIT-SRC-spec §11.12. */
        public static final long RSA = 0x00000004L;
        /** DSA; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DSA-00000005; KMIPKIT-SRC-spec §11.12. */
        public static final long DSA = 0x00000005L;
        /** ECDSA; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDSA-00000006; KMIPKIT-SRC-spec §11.12. */
        public static final long ECDSA = 0x00000006L;
        /** HMAC-SHA1; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA1-00000007; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA1 = 0x00000007L;
        /** HMAC-SHA224; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA224-00000008; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA224 = 0x00000008L;
        /** HMAC-SHA256; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA256-00000009; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA256 = 0x00000009L;
        /** HMAC-SHA384; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA384-0000000A; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA384 = 0x0000000AL;
        /** HMAC-SHA512; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA512-0000000B; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA512 = 0x0000000BL;
        /** HMAC-MD5; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-MD5-0000000C; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_MD5 = 0x0000000CL;
        /** DH; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DH-0000000D; KMIPKIT-SRC-spec §11.12. */
        public static final long DH = 0x0000000DL;
        /** ECDH; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDH-0000000E; KMIPKIT-SRC-spec §11.12. */
        public static final long ECDH = 0x0000000EL;
        /** ECMQV; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECMQV-0000000F; KMIPKIT-SRC-spec §11.12. */
        public static final long ECMQV = 0x0000000FL;
        /** Blowfish; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-BLOWFISH-00000010; KMIPKIT-SRC-spec §11.12. */
        public static final long BLOWFISH = 0x00000010L;
        /** Camellia; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAMELLIA-00000011; KMIPKIT-SRC-spec §11.12. */
        public static final long CAMELLIA = 0x00000011L;
        /** CAST5; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAST5-00000012; KMIPKIT-SRC-spec §11.12. */
        public static final long CAST5 = 0x00000012L;
        /** IDEA; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-IDEA-00000013; KMIPKIT-SRC-spec §11.12. */
        public static final long IDEA = 0x00000013L;
        /** MARS; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MARS-00000014; KMIPKIT-SRC-spec §11.12. */
        public static final long MARS = 0x00000014L;
        /** RC2; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC2-00000015; KMIPKIT-SRC-spec §11.12. */
        public static final long RC2 = 0x00000015L;
        /** RC4; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC4-00000016; KMIPKIT-SRC-spec §11.12. */
        public static final long RC4 = 0x00000016L;
        /** RC5; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC5-00000017; KMIPKIT-SRC-spec §11.12. */
        public static final long RC5 = 0x00000017L;
        /** SKIPJACK; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SKIPJACK-00000018; KMIPKIT-SRC-spec §11.12. */
        public static final long SKIPJACK = 0x00000018L;
        /** Twofish; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-TWOFISH-00000019; KMIPKIT-SRC-spec §11.12. */
        public static final long TWOFISH = 0x00000019L;
        /** EC; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EC-0000001A; KMIPKIT-SRC-spec §11.12. */
        public static final long EC = 0x0000001AL;
        /** One Time Pad; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ONE-TIME-PAD-0000001B; KMIPKIT-SRC-spec §11.12. */
        public static final long ONE_TIME_PAD = 0x0000001BL;
        /** ChaCha20; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20-0000001C; KMIPKIT-SRC-spec §11.12. */
        public static final long CHACHA20 = 0x0000001CL;
        /** Poly1305; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-POLY1305-0000001D; KMIPKIT-SRC-spec §11.12. */
        public static final long POLY1305 = 0x0000001DL;
        /** ChaCha20Poly1305; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20POLY1305-0000001E; KMIPKIT-SRC-spec §11.12. */
        public static final long CHACHA20POLY1305 = 0x0000001EL;
        /** SHA3-224; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-224-0000001F; KMIPKIT-SRC-spec §11.12. */
        public static final long SHA3_224 = 0x0000001FL;
        /** SHA3-256; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-256-00000020; KMIPKIT-SRC-spec §11.12. */
        public static final long SHA3_256 = 0x00000020L;
        /** SHA3-384; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-384-00000021; KMIPKIT-SRC-spec §11.12. */
        public static final long SHA3_384 = 0x00000021L;
        /** SHA3-512; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-512-00000022; KMIPKIT-SRC-spec §11.12. */
        public static final long SHA3_512 = 0x00000022L;
        /** HMAC-SHA3-224; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-224-00000023; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA3_224 = 0x00000023L;
        /** HMAC-SHA3-256; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-256-00000024; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA3_256 = 0x00000024L;
        /** HMAC-SHA3-384; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-384-00000025; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA3_384 = 0x00000025L;
        /** HMAC-SHA3-512; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-512-00000026; KMIPKIT-SRC-spec §11.12. */
        public static final long HMAC_SHA3_512 = 0x00000026L;
        /** SHAKE-128; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-128-00000027; KMIPKIT-SRC-spec §11.12. */
        public static final long SHAKE_128 = 0x00000027L;
        /** SHAKE-256; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-256-00000028; KMIPKIT-SRC-spec §11.12. */
        public static final long SHAKE_256 = 0x00000028L;
        /** ARIA; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ARIA-00000029; KMIPKIT-SRC-spec §11.12. */
        public static final long ARIA = 0x00000029L;
        /** SEED; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SEED-0000002A; KMIPKIT-SRC-spec §11.12. */
        public static final long SEED = 0x0000002AL;
        /** SM2; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM2-0000002B; KMIPKIT-SRC-spec §11.12. */
        public static final long SM2 = 0x0000002BL;
        /** SM3; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM3-0000002C; KMIPKIT-SRC-spec §11.12. */
        public static final long SM3 = 0x0000002CL;
        /** SM4; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM4-0000002D; KMIPKIT-SRC-spec §11.12. */
        public static final long SM4 = 0x0000002DL;
        /** GOST R 34.10-2012; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-10-2012-0000002E; KMIPKIT-SRC-spec §11.12. */
        public static final long GOST_R_34_10_2012 = 0x0000002EL;
        /** GOST R 34.11-2012; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-11-2012-0000002F; KMIPKIT-SRC-spec §11.12. */
        public static final long GOST_R_34_11_2012 = 0x0000002FL;
        /** GOST R 34.13-2015; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-13-2015-00000030; KMIPKIT-SRC-spec §11.12. */
        public static final long GOST_R_34_13_2015 = 0x00000030L;
        /** GOST 28147-89; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-28147-89-00000031; KMIPKIT-SRC-spec §11.12. */
        public static final long GOST_28147_89 = 0x00000031L;
        /** XMSS; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-XMSS-00000032; KMIPKIT-SRC-spec §11.12. */
        public static final long XMSS = 0x00000032L;
        /** SPHINCS-256; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SPHINCS-256-00000033; KMIPKIT-SRC-spec §11.12. */
        public static final long SPHINCS_256 = 0x00000033L;
        /** McEliece; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-00000034; KMIPKIT-SRC-spec §11.12. */
        public static final long MCELIECE = 0x00000034L;
        /** McEliece-6960119; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-6960119-00000035; KMIPKIT-SRC-spec §11.12. */
        public static final long MCELIECE_6960119 = 0x00000035L;
        /** McEliece-8192128; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-8192128-00000036; KMIPKIT-SRC-spec §11.12. */
        public static final long MCELIECE_8192128 = 0x00000036L;
        /** Ed25519; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED25519-00000037; KMIPKIT-SRC-spec §11.12. */
        public static final long ED25519 = 0x00000037L;
        /** Ed448; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED448-00000038; KMIPKIT-SRC-spec §11.12. */
        public static final long ED448 = 0x00000038L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "DES";
                case 0x00000002 -> "3DES";
                case 0x00000003 -> "AES";
                case 0x00000004 -> "RSA";
                case 0x00000005 -> "DSA";
                case 0x00000006 -> "ECDSA";
                case 0x00000007 -> "HMAC-SHA1";
                case 0x00000008 -> "HMAC-SHA224";
                case 0x00000009 -> "HMAC-SHA256";
                case 0x0000000A -> "HMAC-SHA384";
                case 0x0000000B -> "HMAC-SHA512";
                case 0x0000000C -> "HMAC-MD5";
                case 0x0000000D -> "DH";
                case 0x0000000E -> "ECDH";
                case 0x0000000F -> "ECMQV";
                case 0x00000010 -> "Blowfish";
                case 0x00000011 -> "Camellia";
                case 0x00000012 -> "CAST5";
                case 0x00000013 -> "IDEA";
                case 0x00000014 -> "MARS";
                case 0x00000015 -> "RC2";
                case 0x00000016 -> "RC4";
                case 0x00000017 -> "RC5";
                case 0x00000018 -> "SKIPJACK";
                case 0x00000019 -> "Twofish";
                case 0x0000001A -> "EC";
                case 0x0000001B -> "One Time Pad";
                case 0x0000001C -> "ChaCha20";
                case 0x0000001D -> "Poly1305";
                case 0x0000001E -> "ChaCha20Poly1305";
                case 0x0000001F -> "SHA3-224";
                case 0x00000020 -> "SHA3-256";
                case 0x00000021 -> "SHA3-384";
                case 0x00000022 -> "SHA3-512";
                case 0x00000023 -> "HMAC-SHA3-224";
                case 0x00000024 -> "HMAC-SHA3-256";
                case 0x00000025 -> "HMAC-SHA3-384";
                case 0x00000026 -> "HMAC-SHA3-512";
                case 0x00000027 -> "SHAKE-128";
                case 0x00000028 -> "SHAKE-256";
                case 0x00000029 -> "ARIA";
                case 0x0000002A -> "SEED";
                case 0x0000002B -> "SM2";
                case 0x0000002C -> "SM3";
                case 0x0000002D -> "SM4";
                case 0x0000002E -> "GOST R 34.10-2012";
                case 0x0000002F -> "GOST R 34.11-2012";
                case 0x00000030 -> "GOST R 34.13-2015";
                case 0x00000031 -> "GOST 28147-89";
                case 0x00000032 -> "XMSS";
                case 0x00000033 -> "SPHINCS-256";
                case 0x00000034 -> "McEliece";
                case 0x00000035 -> "McEliece-6960119";
                case 0x00000036 -> "McEliece-8192128";
                case 0x00000037 -> "Ed25519";
                case 0x00000038 -> "Ed448";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Data. */
    public static final class Data {
        private Data() {}
        /** Decrypt; KMIPKIT-ELEM-ENUM-VALUE-DATA-DECRYPT-00000001; KMIPKIT-SRC-spec §11.13. */
        public static final long DECRYPT = 0x00000001L;
        /** Encrypt; KMIPKIT-ELEM-ENUM-VALUE-DATA-ENCRYPT-00000002; KMIPKIT-SRC-spec §11.13. */
        public static final long ENCRYPT = 0x00000002L;
        /** Hash; KMIPKIT-ELEM-ENUM-VALUE-DATA-HASH-00000003; KMIPKIT-SRC-spec §11.13. */
        public static final long HASH = 0x00000003L;
        /** MAC MAC Data; KMIPKIT-ELEM-ENUM-VALUE-DATA-MAC-MAC-DATA-00000004; KMIPKIT-SRC-spec §11.13. */
        public static final long MAC_MAC_DATA = 0x00000004L;
        /** RNG Retrieve; KMIPKIT-ELEM-ENUM-VALUE-DATA-RNG-RETRIEVE-00000005; KMIPKIT-SRC-spec §11.13. */
        public static final long RNG_RETRIEVE = 0x00000005L;
        /** Sign Signature Data; KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGN-SIGNATURE-DATA-00000006; KMIPKIT-SRC-spec §11.13. */
        public static final long SIGN_SIGNATURE_DATA = 0x00000006L;
        /** Signature Verify; KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGNATURE-VERIFY-00000007; KMIPKIT-SRC-spec §11.13. */
        public static final long SIGNATURE_VERIFY = 0x00000007L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Decrypt";
                case 0x00000002 -> "Encrypt";
                case 0x00000003 -> "Hash";
                case 0x00000004 -> "MAC MAC Data";
                case 0x00000005 -> "RNG Retrieve";
                case 0x00000006 -> "Sign Signature Data";
                case 0x00000007 -> "Signature Verify";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Derivation Method. */
    public static final class DerivationMethod {
        private DerivationMethod() {}
        /** PBKDF2; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-PBKDF2-00000001; KMIPKIT-SRC-spec §11.14. */
        public static final long PBKDF2 = 0x00000001L;
        /** HASH; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HASH-00000002; KMIPKIT-SRC-spec §11.14. */
        public static final long HASH = 0x00000002L;
        /** HMAC; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HMAC-00000003; KMIPKIT-SRC-spec §11.14. */
        public static final long HMAC = 0x00000003L;
        /** ENCRYPT; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ENCRYPT-00000004; KMIPKIT-SRC-spec §11.14. */
        public static final long ENCRYPT = 0x00000004L;
        /** NIST800-108-C; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-C-00000005; KMIPKIT-SRC-spec §11.14. */
        public static final long NIST800_108_C = 0x00000005L;
        /** NIST800-108-F; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-F-00000006; KMIPKIT-SRC-spec §11.14. */
        public static final long NIST800_108_F = 0x00000006L;
        /** NIST800-108-DPI; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-DPI-00000007; KMIPKIT-SRC-spec §11.14. */
        public static final long NIST800_108_DPI = 0x00000007L;
        /** Asymmetric Key; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ASYMMETRIC-KEY-00000008; KMIPKIT-SRC-spec §11.14. */
        public static final long ASYMMETRIC_KEY = 0x00000008L;
        /** AWS Signature Version 4; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-AWS-SIGNATURE-VERSION-4-00000009; KMIPKIT-SRC-spec §11.14. */
        public static final long AWS_SIGNATURE_VERSION_4 = 0x00000009L;
        /** HKDF; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HKDF-0000000A; KMIPKIT-SRC-spec §11.14. */
        public static final long HKDF = 0x0000000AL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "PBKDF2";
                case 0x00000002 -> "HASH";
                case 0x00000003 -> "HMAC";
                case 0x00000004 -> "ENCRYPT";
                case 0x00000005 -> "NIST800-108-C";
                case 0x00000006 -> "NIST800-108-F";
                case 0x00000007 -> "NIST800-108-DPI";
                case 0x00000008 -> "Asymmetric Key";
                case 0x00000009 -> "AWS Signature Version 4";
                case 0x0000000A -> "HKDF";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Destroy Action. */
    public static final class DestroyAction {
        private DestroyAction() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.15. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Key Material Deleted; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-DELETED-00000002; KMIPKIT-SRC-spec §11.15. */
        public static final long KEY_MATERIAL_DELETED = 0x00000002L;
        /** Key Material Shredded; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-SHREDDED-00000003; KMIPKIT-SRC-spec §11.15. */
        public static final long KEY_MATERIAL_SHREDDED = 0x00000003L;
        /** Meta Data Deleted; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-DELETED-00000004; KMIPKIT-SRC-spec §11.15. */
        public static final long META_DATA_DELETED = 0x00000004L;
        /** Meta Data Shredded; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-SHREDDED-00000005; KMIPKIT-SRC-spec §11.15. */
        public static final long META_DATA_SHREDDED = 0x00000005L;
        /** Deleted; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-DELETED-00000006; KMIPKIT-SRC-spec §11.15. */
        public static final long DELETED = 0x00000006L;
        /** Shredded; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-SHREDDED-00000007; KMIPKIT-SRC-spec §11.15. */
        public static final long SHREDDED = 0x00000007L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Key Material Deleted";
                case 0x00000003 -> "Key Material Shredded";
                case 0x00000004 -> "Meta Data Deleted";
                case 0x00000005 -> "Meta Data Shredded";
                case 0x00000006 -> "Deleted";
                case 0x00000007 -> "Shredded";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Digital Signature Algorithm. */
    public static final class DigitalSignatureAlgorithm {
        private DigitalSignatureAlgorithm() {}
        /** MD2 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD2-WITH-RSA-ENCRYPTION-00000001; KMIPKIT-SRC-spec §11.16. */
        public static final long MD2_WITH_RSA_ENCRYPTION = 0x00000001L;
        /** MD5 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD5-WITH-RSA-ENCRYPTION-00000002; KMIPKIT-SRC-spec §11.16. */
        public static final long MD5_WITH_RSA_ENCRYPTION = 0x00000002L;
        /** SHA-1 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-1-WITH-RSA-ENCRYPTION-00000003; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA_1_WITH_RSA_ENCRYPTION = 0x00000003L;
        /** SHA-224 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-224-WITH-RSA-ENCRYPTION-00000004; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA_224_WITH_RSA_ENCRYPTION = 0x00000004L;
        /** SHA-256 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-256-WITH-RSA-ENCRYPTION-00000005; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA_256_WITH_RSA_ENCRYPTION = 0x00000005L;
        /** SHA-384 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-384-WITH-RSA-ENCRYPTION-00000006; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA_384_WITH_RSA_ENCRYPTION = 0x00000006L;
        /** SHA-512 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-512-WITH-RSA-ENCRYPTION-00000007; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA_512_WITH_RSA_ENCRYPTION = 0x00000007L;
        /** RSASSA-PSS; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-RSASSA-PSS-00000008; KMIPKIT-SRC-spec §11.16. */
        public static final long RSASSA_PSS = 0x00000008L;
        /** DSA with SHA-1; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA-1-00000009; KMIPKIT-SRC-spec §11.16. */
        public static final long DSA_WITH_SHA_1 = 0x00000009L;
        /** DSA with SHA224; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA224-0000000A; KMIPKIT-SRC-spec §11.16. */
        public static final long DSA_WITH_SHA224 = 0x0000000AL;
        /** DSA with SHA256; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA256-0000000B; KMIPKIT-SRC-spec §11.16. */
        public static final long DSA_WITH_SHA256 = 0x0000000BL;
        /** ECDSA with SHA-1; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA-1-0000000C; KMIPKIT-SRC-spec §11.16. */
        public static final long ECDSA_WITH_SHA_1 = 0x0000000CL;
        /** ECDSA with SHA224; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA224-0000000D; KMIPKIT-SRC-spec §11.16. */
        public static final long ECDSA_WITH_SHA224 = 0x0000000DL;
        /** ECDSA with SHA256; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA256-0000000E; KMIPKIT-SRC-spec §11.16. */
        public static final long ECDSA_WITH_SHA256 = 0x0000000EL;
        /** ECDSA with SHA384; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA384-0000000F; KMIPKIT-SRC-spec §11.16. */
        public static final long ECDSA_WITH_SHA384 = 0x0000000FL;
        /** ECDSA with SHA512; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA512-00000010; KMIPKIT-SRC-spec §11.16. */
        public static final long ECDSA_WITH_SHA512 = 0x00000010L;
        /** SHA3-256 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-256-WITH-RSA-ENCRYPTION-00000011; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA3_256_WITH_RSA_ENCRYPTION = 0x00000011L;
        /** SHA3-384 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-384-WITH-RSA-ENCRYPTION-00000012; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA3_384_WITH_RSA_ENCRYPTION = 0x00000012L;
        /** SHA3-512 with RSA Encryption; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-512-WITH-RSA-ENCRYPTION-00000013; KMIPKIT-SRC-spec §11.16. */
        public static final long SHA3_512_WITH_RSA_ENCRYPTION = 0x00000013L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "MD2 with RSA Encryption";
                case 0x00000002 -> "MD5 with RSA Encryption";
                case 0x00000003 -> "SHA-1 with RSA Encryption";
                case 0x00000004 -> "SHA-224 with RSA Encryption";
                case 0x00000005 -> "SHA-256 with RSA Encryption";
                case 0x00000006 -> "SHA-384 with RSA Encryption";
                case 0x00000007 -> "SHA-512 with RSA Encryption";
                case 0x00000008 -> "RSASSA-PSS";
                case 0x00000009 -> "DSA with SHA-1";
                case 0x0000000A -> "DSA with SHA224";
                case 0x0000000B -> "DSA with SHA256";
                case 0x0000000C -> "ECDSA with SHA-1";
                case 0x0000000D -> "ECDSA with SHA224";
                case 0x0000000E -> "ECDSA with SHA256";
                case 0x0000000F -> "ECDSA with SHA384";
                case 0x00000010 -> "ECDSA with SHA512";
                case 0x00000011 -> "SHA3-256 with RSA Encryption";
                case 0x00000012 -> "SHA3-384 with RSA Encryption";
                case 0x00000013 -> "SHA3-512 with RSA Encryption";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for DRBG Algorithm. */
    public static final class DrbgAlgorithm {
        private DrbgAlgorithm() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.17. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Dual-EC; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-DUAL-EC-00000002; KMIPKIT-SRC-spec §11.17. */
        public static final long DUAL_EC = 0x00000002L;
        /** Hash; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HASH-00000003; KMIPKIT-SRC-spec §11.17. */
        public static final long HASH = 0x00000003L;
        /** HMAC; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HMAC-00000004; KMIPKIT-SRC-spec §11.17. */
        public static final long HMAC = 0x00000004L;
        /** CTR; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-CTR-00000005; KMIPKIT-SRC-spec §11.17. */
        public static final long CTR = 0x00000005L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Dual-EC";
                case 0x00000003 -> "Hash";
                case 0x00000004 -> "HMAC";
                case 0x00000005 -> "CTR";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Encoding Option. */
    public static final class EncodingOption {
        private EncodingOption() {}
        /** No Encoding; KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-NO-ENCODING-00000001; KMIPKIT-SRC-spec §11.18. */
        public static final long NO_ENCODING = 0x00000001L;
        /** TTLV Encoding; KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-TTLV-ENCODING-00000002; KMIPKIT-SRC-spec §11.18. */
        public static final long TTLV_ENCODING = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "No Encoding";
                case 0x00000002 -> "TTLV Encoding";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Endpoint Role. */
    public static final class EndpointRole {
        private EndpointRole() {}
        /** Client; KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001; KMIPKIT-SRC-spec §11.19. */
        public static final long CLIENT = 0x00000001L;
        /** Server; KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-SERVER-00000002; KMIPKIT-SRC-spec §11.19. */
        public static final long SERVER = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Client";
                case 0x00000002 -> "Server";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for FIPS186 Variation. */
    public static final class Fips186Variation {
        private Fips186Variation() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.20. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** GP x-Original; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-ORIGINAL-00000002; KMIPKIT-SRC-spec §11.20. */
        public static final long GP_X_ORIGINAL = 0x00000002L;
        /** GP x-Change Notice; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-CHANGE-NOTICE-00000003; KMIPKIT-SRC-spec §11.20. */
        public static final long GP_X_CHANGE_NOTICE = 0x00000003L;
        /** x-Original; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-ORIGINAL-00000004; KMIPKIT-SRC-spec §11.20. */
        public static final long X_ORIGINAL = 0x00000004L;
        /** x-Change Notice; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-CHANGE-NOTICE-00000005; KMIPKIT-SRC-spec §11.20. */
        public static final long X_CHANGE_NOTICE = 0x00000005L;
        /** k-Original; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-ORIGINAL-00000006; KMIPKIT-SRC-spec §11.20. */
        public static final long K_ORIGINAL = 0x00000006L;
        /** k-Change Notice; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-CHANGE-NOTICE-00000007; KMIPKIT-SRC-spec §11.20. */
        public static final long K_CHANGE_NOTICE = 0x00000007L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "GP x-Original";
                case 0x00000003 -> "GP x-Change Notice";
                case 0x00000004 -> "x-Original";
                case 0x00000005 -> "x-Change Notice";
                case 0x00000006 -> "k-Original";
                case 0x00000007 -> "k-Change Notice";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Hashing Algorithm. */
    public static final class HashingAlgorithm {
        private HashingAlgorithm() {}
        /** MD2; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD2-00000001; KMIPKIT-SRC-spec §11.21. */
        public static final long MD2 = 0x00000001L;
        /** MD4; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD4-00000002; KMIPKIT-SRC-spec §11.21. */
        public static final long MD4 = 0x00000002L;
        /** MD5; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD5-00000003; KMIPKIT-SRC-spec §11.21. */
        public static final long MD5 = 0x00000003L;
        /** SHA-1; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-1-00000004; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA_1 = 0x00000004L;
        /** SHA-224; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-224-00000005; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA_224 = 0x00000005L;
        /** SHA-256; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-256-00000006; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA_256 = 0x00000006L;
        /** SHA-384; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-384-00000007; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA_384 = 0x00000007L;
        /** SHA-512; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-00000008; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA_512 = 0x00000008L;
        /** RIPEMD-160; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-RIPEMD-160-00000009; KMIPKIT-SRC-spec §11.21. */
        public static final long RIPEMD_160 = 0x00000009L;
        /** Tiger; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-TIGER-0000000A; KMIPKIT-SRC-spec §11.21. */
        public static final long TIGER = 0x0000000AL;
        /** Whirlpool; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-WHIRLPOOL-0000000B; KMIPKIT-SRC-spec §11.21. */
        public static final long WHIRLPOOL = 0x0000000BL;
        /** SHA-512/224; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-224-0000000C; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA_512_224 = 0x0000000CL;
        /** SHA-512/256; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-256-0000000D; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA_512_256 = 0x0000000DL;
        /** SHA3-224; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-224-0000000E; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA3_224 = 0x0000000EL;
        /** SHA3-256; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-256-0000000F; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA3_256 = 0x0000000FL;
        /** SHA3-384; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-384-00000010; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA3_384 = 0x00000010L;
        /** SHA3-512; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-512-00000011; KMIPKIT-SRC-spec §11.21. */
        public static final long SHA3_512 = 0x00000011L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "MD2";
                case 0x00000002 -> "MD4";
                case 0x00000003 -> "MD5";
                case 0x00000004 -> "SHA-1";
                case 0x00000005 -> "SHA-224";
                case 0x00000006 -> "SHA-256";
                case 0x00000007 -> "SHA-384";
                case 0x00000008 -> "SHA-512";
                case 0x00000009 -> "RIPEMD-160";
                case 0x0000000A -> "Tiger";
                case 0x0000000B -> "Whirlpool";
                case 0x0000000C -> "SHA-512/224";
                case 0x0000000D -> "SHA-512/256";
                case 0x0000000E -> "SHA3-224";
                case 0x0000000F -> "SHA3-256";
                case 0x00000010 -> "SHA3-384";
                case 0x00000011 -> "SHA3-512";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Interop Function. */
    public static final class InteropFunction {
        private InteropFunction() {}
        /** Begin; KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-BEGIN-00000001; KMIPKIT-SRC-spec §11.22. */
        public static final long BEGIN = 0x00000001L;
        /** End; KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-END-00000002; KMIPKIT-SRC-spec §11.22. */
        public static final long END = 0x00000002L;
        /** Reset; KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-RESET-00000003; KMIPKIT-SRC-spec §11.22. */
        public static final long RESET = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Begin";
                case 0x00000002 -> "End";
                case 0x00000003 -> "Reset";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Item Type. */
    public static final class ItemType {
        private ItemType() {}
        /** Structure; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-STRUCTURE-00000001; KMIPKIT-SRC-spec §11.23. */
        public static final long STRUCTURE = 0x00000001L;
        /** Integer; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTEGER-00000002; KMIPKIT-SRC-spec §11.23. */
        public static final long INTEGER = 0x00000002L;
        /** Long Integer; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-LONG-INTEGER-00000003; KMIPKIT-SRC-spec §11.23. */
        public static final long LONG_INTEGER = 0x00000003L;
        /** Big Integer; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BIG-INTEGER-00000004; KMIPKIT-SRC-spec §11.23. */
        public static final long BIG_INTEGER = 0x00000004L;
        /** Enumeration; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-ENUMERATION-00000005; KMIPKIT-SRC-spec §11.23. */
        public static final long ENUMERATION = 0x00000005L;
        /** Boolean; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BOOLEAN-00000006; KMIPKIT-SRC-spec §11.23. */
        public static final long BOOLEAN = 0x00000006L;
        /** Text String; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-TEXT-STRING-00000007; KMIPKIT-SRC-spec §11.23. */
        public static final long TEXT_STRING = 0x00000007L;
        /** Byte String; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BYTE-STRING-00000008; KMIPKIT-SRC-spec §11.23. */
        public static final long BYTE_STRING = 0x00000008L;
        /** Date Time; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-00000009; KMIPKIT-SRC-spec §11.23. */
        public static final long DATE_TIME = 0x00000009L;
        /** Interval; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTERVAL-0000000A; KMIPKIT-SRC-spec §11.23. */
        public static final long INTERVAL = 0x0000000AL;
        /** Date Time Extended; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-EXTENDED-0000000B; KMIPKIT-SRC-spec §11.23. */
        public static final long DATE_TIME_EXTENDED = 0x0000000BL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Structure";
                case 0x00000002 -> "Integer";
                case 0x00000003 -> "Long Integer";
                case 0x00000004 -> "Big Integer";
                case 0x00000005 -> "Enumeration";
                case 0x00000006 -> "Boolean";
                case 0x00000007 -> "Text String";
                case 0x00000008 -> "Byte String";
                case 0x00000009 -> "Date Time";
                case 0x0000000A -> "Interval";
                case 0x0000000B -> "Date Time Extended";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Key Compression Type. */
    public static final class KeyCompressionType {
        private KeyCompressionType() {}
        /** EC Public Key Type Uncompressed; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-UNCOMPRESSED-00000001; KMIPKIT-SRC-spec §11.24. */
        public static final long EC_PUBLIC_KEY_TYPE_UNCOMPRESSED = 0x00000001L;
        /** EC Public Key Type X9.62 Compressed Prime; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-PRIME-00000002; KMIPKIT-SRC-spec §11.24. */
        public static final long EC_PUBLIC_KEY_TYPE_X9_62_COMPRESSED_PRIME = 0x00000002L;
        /** EC Public Key Type X9.62 Compressed Char2; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-CHAR2-00000003; KMIPKIT-SRC-spec §11.24. */
        public static final long EC_PUBLIC_KEY_TYPE_X9_62_COMPRESSED_CHAR2 = 0x00000003L;
        /** EC Public Key Type X9.62 Hybrid; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-HYBRID-00000004; KMIPKIT-SRC-spec §11.24. */
        public static final long EC_PUBLIC_KEY_TYPE_X9_62_HYBRID = 0x00000004L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "EC Public Key Type Uncompressed";
                case 0x00000002 -> "EC Public Key Type X9.62 Compressed Prime";
                case 0x00000003 -> "EC Public Key Type X9.62 Compressed Char2";
                case 0x00000004 -> "EC Public Key Type X9.62 Hybrid";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Key Format Type. */
    public static final class KeyFormatType {
        private KeyFormatType() {}
        /** Raw; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RAW-00000001; KMIPKIT-SRC-spec §11.25. */
        public static final long RAW = 0x00000001L;
        /** Opaque; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-OPAQUE-00000002; KMIPKIT-SRC-spec §11.25. */
        public static final long OPAQUE = 0x00000002L;
        /** PKCS#1; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-1-00000003; KMIPKIT-SRC-spec §11.25. */
        public static final long PKCS_1 = 0x00000003L;
        /** PKCS#8; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-8-00000004; KMIPKIT-SRC-spec §11.25. */
        public static final long PKCS_8 = 0x00000004L;
        /** X.509; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-X-509-00000005; KMIPKIT-SRC-spec §11.25. */
        public static final long X_509 = 0x00000005L;
        /** ECPrivateKey; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-ECPRIVATEKEY-00000006; KMIPKIT-SRC-spec §11.25. */
        public static final long ECPRIVATEKEY = 0x00000006L;
        /** Transparent Symmetric Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-SYMMETRIC-KEY-00000007; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_SYMMETRIC_KEY = 0x00000007L;
        /** Transparent DSA Private Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PRIVATE-KEY-00000008; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_DSA_PRIVATE_KEY = 0x00000008L;
        /** Transparent DSA Public Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PUBLIC-KEY-00000009; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_DSA_PUBLIC_KEY = 0x00000009L;
        /** Transparent RSA Private Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PRIVATE-KEY-0000000A; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_RSA_PRIVATE_KEY = 0x0000000AL;
        /** Transparent RSA Public Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PUBLIC-KEY-0000000B; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_RSA_PUBLIC_KEY = 0x0000000BL;
        /** Transparent DH Private Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PRIVATE-KEY-0000000C; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_DH_PRIVATE_KEY = 0x0000000CL;
        /** Transparent DH Public Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PUBLIC-KEY-0000000D; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_DH_PUBLIC_KEY = 0x0000000DL;
        /** Transparent EC Private Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PRIVATE-KEY-00000014; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_EC_PRIVATE_KEY = 0x00000014L;
        /** Transparent EC Public Key; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PUBLIC-KEY-00000015; KMIPKIT-SRC-spec §11.25. */
        public static final long TRANSPARENT_EC_PUBLIC_KEY = 0x00000015L;
        /** PKCS#12; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-12-00000016; KMIPKIT-SRC-spec §11.25. */
        public static final long PKCS_12 = 0x00000016L;
        /** PKCS#10; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-10-00000017; KMIPKIT-SRC-spec §11.25. */
        public static final long PKCS_10 = 0x00000017L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Raw";
                case 0x00000002 -> "Opaque";
                case 0x00000003 -> "PKCS#1";
                case 0x00000004 -> "PKCS#8";
                case 0x00000005 -> "X.509";
                case 0x00000006 -> "ECPrivateKey";
                case 0x00000007 -> "Transparent Symmetric Key";
                case 0x00000008 -> "Transparent DSA Private Key";
                case 0x00000009 -> "Transparent DSA Public Key";
                case 0x0000000A -> "Transparent RSA Private Key";
                case 0x0000000B -> "Transparent RSA Public Key";
                case 0x0000000C -> "Transparent DH Private Key";
                case 0x0000000D -> "Transparent DH Public Key";
                case 0x00000014 -> "Transparent EC Private Key";
                case 0x00000015 -> "Transparent EC Public Key";
                case 0x00000016 -> "PKCS#12";
                case 0x00000017 -> "PKCS#10";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Key Role Type. */
    public static final class KeyRoleType {
        private KeyRoleType() {}
        /** BDK; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-BDK-00000001; KMIPKIT-SRC-spec §11.26. */
        public static final long BDK = 0x00000001L;
        /** CVK; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-CVK-00000002; KMIPKIT-SRC-spec §11.26. */
        public static final long CVK = 0x00000002L;
        /** DEK; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DEK-00000003; KMIPKIT-SRC-spec §11.26. */
        public static final long DEK = 0x00000003L;
        /** MKAC; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKAC-00000004; KMIPKIT-SRC-spec §11.26. */
        public static final long MKAC = 0x00000004L;
        /** MKSMC; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMC-00000005; KMIPKIT-SRC-spec §11.26. */
        public static final long MKSMC = 0x00000005L;
        /** MKSMI; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMI-00000006; KMIPKIT-SRC-spec §11.26. */
        public static final long MKSMI = 0x00000006L;
        /** MKDAC; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDAC-00000007; KMIPKIT-SRC-spec §11.26. */
        public static final long MKDAC = 0x00000007L;
        /** MKDN; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDN-00000008; KMIPKIT-SRC-spec §11.26. */
        public static final long MKDN = 0x00000008L;
        /** MKCP; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKCP-00000009; KMIPKIT-SRC-spec §11.26. */
        public static final long MKCP = 0x00000009L;
        /** MKOTH; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKOTH-0000000A; KMIPKIT-SRC-spec §11.26. */
        public static final long MKOTH = 0x0000000AL;
        /** KEK; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-KEK-0000000B; KMIPKIT-SRC-spec §11.26. */
        public static final long KEK = 0x0000000BL;
        /** MAC16609; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC16609-0000000C; KMIPKIT-SRC-spec §11.26. */
        public static final long MAC16609 = 0x0000000CL;
        /** MAC97971; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97971-0000000D; KMIPKIT-SRC-spec §11.26. */
        public static final long MAC97971 = 0x0000000DL;
        /** MAC97972; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97972-0000000E; KMIPKIT-SRC-spec §11.26. */
        public static final long MAC97972 = 0x0000000EL;
        /** MAC97973; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97973-0000000F; KMIPKIT-SRC-spec §11.26. */
        public static final long MAC97973 = 0x0000000FL;
        /** MAC97974; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97974-00000010; KMIPKIT-SRC-spec §11.26. */
        public static final long MAC97974 = 0x00000010L;
        /** MAC97975; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97975-00000011; KMIPKIT-SRC-spec §11.26. */
        public static final long MAC97975 = 0x00000011L;
        /** ZPK; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-ZPK-00000012; KMIPKIT-SRC-spec §11.26. */
        public static final long ZPK = 0x00000012L;
        /** PVKIBM; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKIBM-00000013; KMIPKIT-SRC-spec §11.26. */
        public static final long PVKIBM = 0x00000013L;
        /** PVKPVV; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKPVV-00000014; KMIPKIT-SRC-spec §11.26. */
        public static final long PVKPVV = 0x00000014L;
        /** PVKOTH; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKOTH-00000015; KMIPKIT-SRC-spec §11.26. */
        public static final long PVKOTH = 0x00000015L;
        /** DUKPT; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DUKPT-00000016; KMIPKIT-SRC-spec §11.26. */
        public static final long DUKPT = 0x00000016L;
        /** IV; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-IV-00000017; KMIPKIT-SRC-spec §11.26. */
        public static final long IV = 0x00000017L;
        /** TRKBK; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-TRKBK-00000018; KMIPKIT-SRC-spec §11.26. */
        public static final long TRKBK = 0x00000018L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "BDK";
                case 0x00000002 -> "CVK";
                case 0x00000003 -> "DEK";
                case 0x00000004 -> "MKAC";
                case 0x00000005 -> "MKSMC";
                case 0x00000006 -> "MKSMI";
                case 0x00000007 -> "MKDAC";
                case 0x00000008 -> "MKDN";
                case 0x00000009 -> "MKCP";
                case 0x0000000A -> "MKOTH";
                case 0x0000000B -> "KEK";
                case 0x0000000C -> "MAC16609";
                case 0x0000000D -> "MAC97971";
                case 0x0000000E -> "MAC97972";
                case 0x0000000F -> "MAC97973";
                case 0x00000010 -> "MAC97974";
                case 0x00000011 -> "MAC97975";
                case 0x00000012 -> "ZPK";
                case 0x00000013 -> "PVKIBM";
                case 0x00000014 -> "PVKPVV";
                case 0x00000015 -> "PVKOTH";
                case 0x00000016 -> "DUKPT";
                case 0x00000017 -> "IV";
                case 0x00000018 -> "TRKBK";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Key Value Location Type. */
    public static final class KeyValueLocationType {
        private KeyValueLocationType() {}
        /** Uninterpreted Text String; KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.27. */
        public static final long UNINTERPRETED_TEXT_STRING = 0x00000001L;
        /** URI; KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.27. */
        public static final long URI = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Uninterpreted Text String";
                case 0x00000002 -> "URI";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Key Wrap Type. */
    public static final class KeyWrapType {
        private KeyWrapType() {}
        /** Not Wrapped; KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-NOT-WRAPPED-00000001; KMIPKIT-SRC-spec §11.29. */
        public static final long NOT_WRAPPED = 0x00000001L;
        /** As Registered; KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-AS-REGISTERED-00000002; KMIPKIT-SRC-spec §11.29. */
        public static final long AS_REGISTERED = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Not Wrapped";
                case 0x00000002 -> "As Registered";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Link Type. */
    public static final class LinkType {
        private LinkType() {}
        /** Certificate Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CERTIFICATE-LINK-00000101; KMIPKIT-SRC-spec §11.28. */
        public static final long CERTIFICATE_LINK = 0x00000101L;
        /** Public Key Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PUBLIC-KEY-LINK-00000102; KMIPKIT-SRC-spec §11.28. */
        public static final long PUBLIC_KEY_LINK = 0x00000102L;
        /** Private Key Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PRIVATE-KEY-LINK-00000103; KMIPKIT-SRC-spec §11.28. */
        public static final long PRIVATE_KEY_LINK = 0x00000103L;
        /** Derivation Base Object Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVATION-BASE-OBJECT-LINK-00000104; KMIPKIT-SRC-spec §11.28. */
        public static final long DERIVATION_BASE_OBJECT_LINK = 0x00000104L;
        /** Derived Key Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVED-KEY-LINK-00000105; KMIPKIT-SRC-spec §11.28. */
        public static final long DERIVED_KEY_LINK = 0x00000105L;
        /** Replacement Object Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACEMENT-OBJECT-LINK-00000106; KMIPKIT-SRC-spec §11.28. */
        public static final long REPLACEMENT_OBJECT_LINK = 0x00000106L;
        /** Replaced Object Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACED-OBJECT-LINK-00000107; KMIPKIT-SRC-spec §11.28. */
        public static final long REPLACED_OBJECT_LINK = 0x00000107L;
        /** Parent Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PARENT-LINK-00000108; KMIPKIT-SRC-spec §11.28. */
        public static final long PARENT_LINK = 0x00000108L;
        /** Child Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CHILD-LINK-00000109; KMIPKIT-SRC-spec §11.28. */
        public static final long CHILD_LINK = 0x00000109L;
        /** Previous Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PREVIOUS-LINK-0000010A; KMIPKIT-SRC-spec §11.28. */
        public static final long PREVIOUS_LINK = 0x0000010AL;
        /** Next Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-NEXT-LINK-0000010B; KMIPKIT-SRC-spec §11.28. */
        public static final long NEXT_LINK = 0x0000010BL;
        /** PKCS#12 Certificate Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-CERTIFICATE-LINK-0000010C; KMIPKIT-SRC-spec §11.28. */
        public static final long PKCS_12_CERTIFICATE_LINK = 0x0000010CL;
        /** PKCS#12 Password Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-PASSWORD-LINK-0000010D; KMIPKIT-SRC-spec §11.28. */
        public static final long PKCS_12_PASSWORD_LINK = 0x0000010DL;
        /** Wrapping Key Link; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-WRAPPING-KEY-LINK-0000010E; KMIPKIT-SRC-spec §11.28. */
        public static final long WRAPPING_KEY_LINK = 0x0000010EL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000101 -> "Certificate Link";
                case 0x00000102 -> "Public Key Link";
                case 0x00000103 -> "Private Key Link";
                case 0x00000104 -> "Derivation Base Object Link";
                case 0x00000105 -> "Derived Key Link";
                case 0x00000106 -> "Replacement Object Link";
                case 0x00000107 -> "Replaced Object Link";
                case 0x00000108 -> "Parent Link";
                case 0x00000109 -> "Child Link";
                case 0x0000010A -> "Previous Link";
                case 0x0000010B -> "Next Link";
                case 0x0000010C -> "PKCS#12 Certificate Link";
                case 0x0000010D -> "PKCS#12 Password Link";
                case 0x0000010E -> "Wrapping Key Link";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Mask Generator. */
    public static final class MaskGenerator {
        private MaskGenerator() {}
        /** MFG1; KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-MFG1-00000001; KMIPKIT-SRC-spec §11.30. */
        public static final long MFG1 = 0x00000001L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "MFG1";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Name Type. */
    public static final class NameType {
        private NameType() {}
        /** Uninterpreted Text String; KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.31. */
        public static final long UNINTERPRETED_TEXT_STRING = 0x00000001L;
        /** URI; KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.31. */
        public static final long URI = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Uninterpreted Text String";
                case 0x00000002 -> "URI";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for NIST Key Type. */
    public static final class NistKeyType {
        private NistKeyType() {}
        /** Private signature key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-SIGNATURE-KEY-00000001; KMIPKIT-SRC-spec §11.32. */
        public static final long PRIVATE_SIGNATURE_KEY = 0x00000001L;
        /** Public signature verification key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-SIGNATURE-VERIFICATION-KEY-00000002; KMIPKIT-SRC-spec §11.32. */
        public static final long PUBLIC_SIGNATURE_VERIFICATION_KEY = 0x00000002L;
        /** Symmetric authentication key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHENTICATION-KEY-00000003; KMIPKIT-SRC-spec §11.32. */
        public static final long SYMMETRIC_AUTHENTICATION_KEY = 0x00000003L;
        /** Private authentication key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHENTICATION-KEY-00000004; KMIPKIT-SRC-spec §11.32. */
        public static final long PRIVATE_AUTHENTICATION_KEY = 0x00000004L;
        /** Public authentication key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHENTICATION-KEY-00000005; KMIPKIT-SRC-spec §11.32. */
        public static final long PUBLIC_AUTHENTICATION_KEY = 0x00000005L;
        /** Symmetric data encryption key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-DATA-ENCRYPTION-KEY-00000006; KMIPKIT-SRC-spec §11.32. */
        public static final long SYMMETRIC_DATA_ENCRYPTION_KEY = 0x00000006L;
        /** Symmetric key wrapping key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-WRAPPING-KEY-00000007; KMIPKIT-SRC-spec §11.32. */
        public static final long SYMMETRIC_KEY_WRAPPING_KEY = 0x00000007L;
        /** Symmetric random number generation key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-RANDOM-NUMBER-GENERATION-KEY-00000008; KMIPKIT-SRC-spec §11.32. */
        public static final long SYMMETRIC_RANDOM_NUMBER_GENERATION_KEY = 0x00000008L;
        /** Symmetric master key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-MASTER-KEY-00000009; KMIPKIT-SRC-spec §11.32. */
        public static final long SYMMETRIC_MASTER_KEY = 0x00000009L;
        /** Private key transport key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-KEY-TRANSPORT-KEY-0000000A; KMIPKIT-SRC-spec §11.32. */
        public static final long PRIVATE_KEY_TRANSPORT_KEY = 0x0000000AL;
        /** Public key transport key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-KEY-TRANSPORT-KEY-0000000B; KMIPKIT-SRC-spec §11.32. */
        public static final long PUBLIC_KEY_TRANSPORT_KEY = 0x0000000BL;
        /** Symmetric key agreement key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-AGREEMENT-KEY-0000000C; KMIPKIT-SRC-spec §11.32. */
        public static final long SYMMETRIC_KEY_AGREEMENT_KEY = 0x0000000CL;
        /** Private static key agreement key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-STATIC-KEY-AGREEMENT-KEY-0000000D; KMIPKIT-SRC-spec §11.32. */
        public static final long PRIVATE_STATIC_KEY_AGREEMENT_KEY = 0x0000000DL;
        /** Public static key agreement key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-STATIC-KEY-AGREEMENT-KEY-0000000E; KMIPKIT-SRC-spec §11.32. */
        public static final long PUBLIC_STATIC_KEY_AGREEMENT_KEY = 0x0000000EL;
        /** Private ephemeral key agreement key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-EPHEMERAL-KEY-AGREEMENT-KEY-0000000F; KMIPKIT-SRC-spec §11.32. */
        public static final long PRIVATE_EPHEMERAL_KEY_AGREEMENT_KEY = 0x0000000FL;
        /** Public ephemeral key agreement key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-EPHEMERAL-KEY-AGREEMENT-KEY-00000010; KMIPKIT-SRC-spec §11.32. */
        public static final long PUBLIC_EPHEMERAL_KEY_AGREEMENT_KEY = 0x00000010L;
        /** Symmetric authorization key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHORIZATION-KEY-00000011; KMIPKIT-SRC-spec §11.32. */
        public static final long SYMMETRIC_AUTHORIZATION_KEY = 0x00000011L;
        /** Private authorization key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHORIZATION-KEY-00000012; KMIPKIT-SRC-spec §11.32. */
        public static final long PRIVATE_AUTHORIZATION_KEY = 0x00000012L;
        /** Public authorization key; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHORIZATION-KEY-00000013; KMIPKIT-SRC-spec §11.32. */
        public static final long PUBLIC_AUTHORIZATION_KEY = 0x00000013L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Private signature key";
                case 0x00000002 -> "Public signature verification key";
                case 0x00000003 -> "Symmetric authentication key";
                case 0x00000004 -> "Private authentication key";
                case 0x00000005 -> "Public authentication key";
                case 0x00000006 -> "Symmetric data encryption key";
                case 0x00000007 -> "Symmetric key wrapping key";
                case 0x00000008 -> "Symmetric random number generation key";
                case 0x00000009 -> "Symmetric master key";
                case 0x0000000A -> "Private key transport key";
                case 0x0000000B -> "Public key transport key";
                case 0x0000000C -> "Symmetric key agreement key";
                case 0x0000000D -> "Private static key agreement key";
                case 0x0000000E -> "Public static key agreement key";
                case 0x0000000F -> "Private ephemeral key agreement key";
                case 0x00000010 -> "Public ephemeral key agreement key";
                case 0x00000011 -> "Symmetric authorization key";
                case 0x00000012 -> "Private authorization key";
                case 0x00000013 -> "Public authorization key";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Object Group Member. */
    public static final class ObjectGroupMember {
        private ObjectGroupMember() {}
        /** Group Member Fresh; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-FRESH-00000001; KMIPKIT-SRC-spec §11.33. */
        public static final long GROUP_MEMBER_FRESH = 0x00000001L;
        /** Group Member Default; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-DEFAULT-00000002; KMIPKIT-SRC-spec §11.33. */
        public static final long GROUP_MEMBER_DEFAULT = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Group Member Fresh";
                case 0x00000002 -> "Group Member Default";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Object Type. */
    public static final class ObjectType {
        private ObjectType() {}
        /** Certificate; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-00000001; KMIPKIT-SRC-spec §11.34. */
        public static final long CERTIFICATE = 0x00000001L;
        /** Symmetric Key; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SYMMETRIC-KEY-00000002; KMIPKIT-SRC-spec §11.34. */
        public static final long SYMMETRIC_KEY = 0x00000002L;
        /** Public Key; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PUBLIC-KEY-00000003; KMIPKIT-SRC-spec §11.34. */
        public static final long PUBLIC_KEY = 0x00000003L;
        /** Private Key; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PRIVATE-KEY-00000004; KMIPKIT-SRC-spec §11.34. */
        public static final long PRIVATE_KEY = 0x00000004L;
        /** Split Key; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SPLIT-KEY-00000005; KMIPKIT-SRC-spec §11.34. */
        public static final long SPLIT_KEY = 0x00000005L;
        /** Secret Data; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SECRET-DATA-00000007; KMIPKIT-SRC-spec §11.34. */
        public static final long SECRET_DATA = 0x00000007L;
        /** Opaque Object; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-OPAQUE-OBJECT-00000008; KMIPKIT-SRC-spec §11.34. */
        public static final long OPAQUE_OBJECT = 0x00000008L;
        /** PGP Key; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PGP-KEY-00000009; KMIPKIT-SRC-spec §11.34. */
        public static final long PGP_KEY = 0x00000009L;
        /** Certificate Request; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-REQUEST-0000000A; KMIPKIT-SRC-spec §11.34. */
        public static final long CERTIFICATE_REQUEST = 0x0000000AL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Certificate";
                case 0x00000002 -> "Symmetric Key";
                case 0x00000003 -> "Public Key";
                case 0x00000004 -> "Private Key";
                case 0x00000005 -> "Split Key";
                case 0x00000007 -> "Secret Data";
                case 0x00000008 -> "Opaque Object";
                case 0x00000009 -> "PGP Key";
                case 0x0000000A -> "Certificate Request";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Opaque Data Type. */
    public static final class OpaqueDataType {
        private OpaqueDataType() {}
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Operation. */
    public static final class Operation {
        private Operation() {}
        /** Create; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-00000001; KMIPKIT-SRC-spec §11.36. */
        public static final long CREATE = 0x00000001L;
        /** Create Key Pair; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-KEY-PAIR-00000002; KMIPKIT-SRC-spec §11.36. */
        public static final long CREATE_KEY_PAIR = 0x00000002L;
        /** Register; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REGISTER-00000003; KMIPKIT-SRC-spec §11.36. */
        public static final long REGISTER = 0x00000003L;
        /** Re-key; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-00000004; KMIPKIT-SRC-spec §11.36. */
        public static final long RE_KEY = 0x00000004L;
        /** Derive Key; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DERIVE-KEY-00000005; KMIPKIT-SRC-spec §11.36. */
        public static final long DERIVE_KEY = 0x00000005L;
        /** Certify; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CERTIFY-00000006; KMIPKIT-SRC-spec §11.36. */
        public static final long CERTIFY = 0x00000006L;
        /** Re-certify; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-CERTIFY-00000007; KMIPKIT-SRC-spec §11.36. */
        public static final long RE_CERTIFY = 0x00000007L;
        /** Locate; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOCATE-00000008; KMIPKIT-SRC-spec §11.36. */
        public static final long LOCATE = 0x00000008L;
        /** Check; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CHECK-00000009; KMIPKIT-SRC-spec §11.36. */
        public static final long CHECK = 0x00000009L;
        /** Get; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-0000000A; KMIPKIT-SRC-spec §11.36. */
        public static final long GET = 0x0000000AL;
        /** Get Attributes; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTES-0000000B; KMIPKIT-SRC-spec §11.36. */
        public static final long GET_ATTRIBUTES = 0x0000000BL;
        /** Get Attribute List; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTE-LIST-0000000C; KMIPKIT-SRC-spec §11.36. */
        public static final long GET_ATTRIBUTE_LIST = 0x0000000CL;
        /** Add Attribute; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADD-ATTRIBUTE-0000000D; KMIPKIT-SRC-spec §11.36. */
        public static final long ADD_ATTRIBUTE = 0x0000000DL;
        /** Modify Attribute; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MODIFY-ATTRIBUTE-0000000E; KMIPKIT-SRC-spec §11.36. */
        public static final long MODIFY_ATTRIBUTE = 0x0000000EL;
        /** Delete Attribute; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELETE-ATTRIBUTE-0000000F; KMIPKIT-SRC-spec §11.36. */
        public static final long DELETE_ATTRIBUTE = 0x0000000FL;
        /** Obtain Lease; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-OBTAIN-LEASE-00000010; KMIPKIT-SRC-spec §11.36. */
        public static final long OBTAIN_LEASE = 0x00000010L;
        /** Get Usage Allocation; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-USAGE-ALLOCATION-00000011; KMIPKIT-SRC-spec §11.36. */
        public static final long GET_USAGE_ALLOCATION = 0x00000011L;
        /** Activate; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ACTIVATE-00000012; KMIPKIT-SRC-spec §11.36. */
        public static final long ACTIVATE = 0x00000012L;
        /** Revoke; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REVOKE-00000013; KMIPKIT-SRC-spec §11.36. */
        public static final long REVOKE = 0x00000013L;
        /** Destroy; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DESTROY-00000014; KMIPKIT-SRC-spec §11.36. */
        public static final long DESTROY = 0x00000014L;
        /** Archive; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ARCHIVE-00000015; KMIPKIT-SRC-spec §11.36. */
        public static final long ARCHIVE = 0x00000015L;
        /** Recover; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RECOVER-00000016; KMIPKIT-SRC-spec §11.36. */
        public static final long RECOVER = 0x00000016L;
        /** Validate; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-VALIDATE-00000017; KMIPKIT-SRC-spec §11.36. */
        public static final long VALIDATE = 0x00000017L;
        /** Query; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-00000018; KMIPKIT-SRC-spec §11.36. */
        public static final long QUERY = 0x00000018L;
        /** Cancel; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CANCEL-00000019; KMIPKIT-SRC-spec §11.36. */
        public static final long CANCEL = 0x00000019L;
        /** Poll; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-POLL-0000001A; KMIPKIT-SRC-spec §11.36. */
        public static final long POLL = 0x0000001AL;
        /** Notify; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-NOTIFY-0000001B; KMIPKIT-SRC-spec §11.36. */
        public static final long NOTIFY = 0x0000001BL;
        /** Put; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PUT-0000001C; KMIPKIT-SRC-spec §11.36. */
        public static final long PUT = 0x0000001CL;
        /** Re-key Key Pair; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-KEY-PAIR-0000001D; KMIPKIT-SRC-spec §11.36. */
        public static final long RE_KEY_KEY_PAIR = 0x0000001DL;
        /** Discover Versions; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DISCOVER-VERSIONS-0000001E; KMIPKIT-SRC-spec §11.36. */
        public static final long DISCOVER_VERSIONS = 0x0000001EL;
        /** Encrypt; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ENCRYPT-0000001F; KMIPKIT-SRC-spec §11.36. */
        public static final long ENCRYPT = 0x0000001FL;
        /** Decrypt; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DECRYPT-00000020; KMIPKIT-SRC-spec §11.36. */
        public static final long DECRYPT = 0x00000020L;
        /** Sign; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGN-00000021; KMIPKIT-SRC-spec §11.36. */
        public static final long SIGN = 0x00000021L;
        /** Signature Verify; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGNATURE-VERIFY-00000022; KMIPKIT-SRC-spec §11.36. */
        public static final long SIGNATURE_VERIFY = 0x00000022L;
        /** MAC; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-00000023; KMIPKIT-SRC-spec §11.36. */
        public static final long MAC = 0x00000023L;
        /** MAC Verify; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-VERIFY-00000024; KMIPKIT-SRC-spec §11.36. */
        public static final long MAC_VERIFY = 0x00000024L;
        /** RNG Retrieve; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-RETRIEVE-00000025; KMIPKIT-SRC-spec §11.36. */
        public static final long RNG_RETRIEVE = 0x00000025L;
        /** RNG Seed; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-SEED-00000026; KMIPKIT-SRC-spec §11.36. */
        public static final long RNG_SEED = 0x00000026L;
        /** Hash; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-HASH-00000027; KMIPKIT-SRC-spec §11.36. */
        public static final long HASH = 0x00000027L;
        /** Create Split Key; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-SPLIT-KEY-00000028; KMIPKIT-SRC-spec §11.36. */
        public static final long CREATE_SPLIT_KEY = 0x00000028L;
        /** Join Split Key; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-JOIN-SPLIT-KEY-00000029; KMIPKIT-SRC-spec §11.36. */
        public static final long JOIN_SPLIT_KEY = 0x00000029L;
        /** Import; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-IMPORT-0000002A; KMIPKIT-SRC-spec §11.36. */
        public static final long IMPORT = 0x0000002AL;
        /** Export; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-EXPORT-0000002B; KMIPKIT-SRC-spec §11.36. */
        public static final long EXPORT = 0x0000002BL;
        /** Log; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOG-0000002C; KMIPKIT-SRC-spec §11.36. */
        public static final long LOG = 0x0000002CL;
        /** Login; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGIN-0000002D; KMIPKIT-SRC-spec §11.36. */
        public static final long LOGIN = 0x0000002DL;
        /** Logout; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGOUT-0000002E; KMIPKIT-SRC-spec §11.36. */
        public static final long LOGOUT = 0x0000002EL;
        /** Delegated Login; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELEGATED-LOGIN-0000002F; KMIPKIT-SRC-spec §11.36. */
        public static final long DELEGATED_LOGIN = 0x0000002FL;
        /** Adjust Attribute; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADJUST-ATTRIBUTE-00000030; KMIPKIT-SRC-spec §11.36. */
        public static final long ADJUST_ATTRIBUTE = 0x00000030L;
        /** Set Attribute; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ATTRIBUTE-00000031; KMIPKIT-SRC-spec §11.36. */
        public static final long SET_ATTRIBUTE = 0x00000031L;
        /** Set Endpoint Role; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ENDPOINT-ROLE-00000032; KMIPKIT-SRC-spec §11.36. */
        public static final long SET_ENDPOINT_ROLE = 0x00000032L;
        /** PKCS#11; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PKCS-11-00000033; KMIPKIT-SRC-spec §11.36. */
        public static final long PKCS_11 = 0x00000033L;
        /** Interop; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-INTEROP-00000034; KMIPKIT-SRC-spec §11.36. */
        public static final long INTEROP = 0x00000034L;
        /** Re-Provision; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-PROVISION-00000035; KMIPKIT-SRC-spec §11.36. */
        public static final long RE_PROVISION = 0x00000035L;
        /** Set Defaults; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-DEFAULTS-00000036; KMIPKIT-SRC-spec §11.36. */
        public static final long SET_DEFAULTS = 0x00000036L;
        /** Set Constraints; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-CONSTRAINTS-00000037; KMIPKIT-SRC-spec §11.36. */
        public static final long SET_CONSTRAINTS = 0x00000037L;
        /** Get Constraints; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-CONSTRAINTS-00000038; KMIPKIT-SRC-spec §11.36. */
        public static final long GET_CONSTRAINTS = 0x00000038L;
        /** Query Asynchronous Requests; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-ASYNCHRONOUS-REQUESTS-00000039; KMIPKIT-SRC-spec §11.36. */
        public static final long QUERY_ASYNCHRONOUS_REQUESTS = 0x00000039L;
        /** Process; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PROCESS-0000003A; KMIPKIT-SRC-spec §11.36. */
        public static final long PROCESS = 0x0000003AL;
        /** Ping; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PING-0000003B; KMIPKIT-SRC-spec §11.36. */
        public static final long PING = 0x0000003BL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Create";
                case 0x00000002 -> "Create Key Pair";
                case 0x00000003 -> "Register";
                case 0x00000004 -> "Re-key";
                case 0x00000005 -> "Derive Key";
                case 0x00000006 -> "Certify";
                case 0x00000007 -> "Re-certify";
                case 0x00000008 -> "Locate";
                case 0x00000009 -> "Check";
                case 0x0000000A -> "Get";
                case 0x0000000B -> "Get Attributes";
                case 0x0000000C -> "Get Attribute List";
                case 0x0000000D -> "Add Attribute";
                case 0x0000000E -> "Modify Attribute";
                case 0x0000000F -> "Delete Attribute";
                case 0x00000010 -> "Obtain Lease";
                case 0x00000011 -> "Get Usage Allocation";
                case 0x00000012 -> "Activate";
                case 0x00000013 -> "Revoke";
                case 0x00000014 -> "Destroy";
                case 0x00000015 -> "Archive";
                case 0x00000016 -> "Recover";
                case 0x00000017 -> "Validate";
                case 0x00000018 -> "Query";
                case 0x00000019 -> "Cancel";
                case 0x0000001A -> "Poll";
                case 0x0000001B -> "Notify";
                case 0x0000001C -> "Put";
                case 0x0000001D -> "Re-key Key Pair";
                case 0x0000001E -> "Discover Versions";
                case 0x0000001F -> "Encrypt";
                case 0x00000020 -> "Decrypt";
                case 0x00000021 -> "Sign";
                case 0x00000022 -> "Signature Verify";
                case 0x00000023 -> "MAC";
                case 0x00000024 -> "MAC Verify";
                case 0x00000025 -> "RNG Retrieve";
                case 0x00000026 -> "RNG Seed";
                case 0x00000027 -> "Hash";
                case 0x00000028 -> "Create Split Key";
                case 0x00000029 -> "Join Split Key";
                case 0x0000002A -> "Import";
                case 0x0000002B -> "Export";
                case 0x0000002C -> "Log";
                case 0x0000002D -> "Login";
                case 0x0000002E -> "Logout";
                case 0x0000002F -> "Delegated Login";
                case 0x00000030 -> "Adjust Attribute";
                case 0x00000031 -> "Set Attribute";
                case 0x00000032 -> "Set Endpoint Role";
                case 0x00000033 -> "PKCS#11";
                case 0x00000034 -> "Interop";
                case 0x00000035 -> "Re-Provision";
                case 0x00000036 -> "Set Defaults";
                case 0x00000037 -> "Set Constraints";
                case 0x00000038 -> "Get Constraints";
                case 0x00000039 -> "Query Asynchronous Requests";
                case 0x0000003A -> "Process";
                case 0x0000003B -> "Ping";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Padding Method. */
    public static final class PaddingMethod {
        private PaddingMethod() {}
        /** None; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-NONE-00000001; KMIPKIT-SRC-spec §11.37. */
        public static final long NONE = 0x00000001L;
        /** OAEP; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-OAEP-00000002; KMIPKIT-SRC-spec §11.37. */
        public static final long OAEP = 0x00000002L;
        /** PKCS5; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS5-00000003; KMIPKIT-SRC-spec §11.37. */
        public static final long PKCS5 = 0x00000003L;
        /** SSL3; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-SSL3-00000004; KMIPKIT-SRC-spec §11.37. */
        public static final long SSL3 = 0x00000004L;
        /** Zeros; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ZEROS-00000005; KMIPKIT-SRC-spec §11.37. */
        public static final long ZEROS = 0x00000005L;
        /** ANSI X9.23; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ANSI-X9-23-00000006; KMIPKIT-SRC-spec §11.37. */
        public static final long ANSI_X9_23 = 0x00000006L;
        /** ISO 10126; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ISO-10126-00000007; KMIPKIT-SRC-spec §11.37. */
        public static final long ISO_10126 = 0x00000007L;
        /** PKCS1 v1.5; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS1-V1-5-00000008; KMIPKIT-SRC-spec §11.37. */
        public static final long PKCS1_V1_5 = 0x00000008L;
        /** X9.31; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-X9-31-00000009; KMIPKIT-SRC-spec §11.37. */
        public static final long X9_31 = 0x00000009L;
        /** PSS; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PSS-0000000A; KMIPKIT-SRC-spec §11.37. */
        public static final long PSS = 0x0000000AL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "None";
                case 0x00000002 -> "OAEP";
                case 0x00000003 -> "PKCS5";
                case 0x00000004 -> "SSL3";
                case 0x00000005 -> "Zeros";
                case 0x00000006 -> "ANSI X9.23";
                case 0x00000007 -> "ISO 10126";
                case 0x00000008 -> "PKCS1 v1.5";
                case 0x00000009 -> "X9.31";
                case 0x0000000A -> "PSS";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for PKCS#11 Function. */
    public static final class Pkcs11Function {
        private Pkcs11Function() {}
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for PKCS#11 Return Code. */
    public static final class Pkcs11ReturnCode {
        private Pkcs11ReturnCode() {}
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Processing Stage. */
    public static final class ProcessingStage {
        private ProcessingStage() {}
        /** Submitted; KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-SUBMITTED-00000001; KMIPKIT-SRC-spec §11.40. */
        public static final long SUBMITTED = 0x00000001L;
        /** In Process; KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-IN-PROCESS-00000002; KMIPKIT-SRC-spec §11.40. */
        public static final long IN_PROCESS = 0x00000002L;
        /** Completed; KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-COMPLETED-00000003; KMIPKIT-SRC-spec §11.40. */
        public static final long COMPLETED = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Submitted";
                case 0x00000002 -> "In Process";
                case 0x00000003 -> "Completed";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Profile Name. */
    public static final class ProfileName {
        private ProfileName() {}
        /** Complete Server Basic; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-BASIC-00000104; KMIPKIT-SRC-spec §11.41. */
        public static final long COMPLETE_SERVER_BASIC = 0x00000104L;
        /** Complete Server TLS v1.2; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-TLS-V1-2-00000105; KMIPKIT-SRC-spec §11.41. */
        public static final long COMPLETE_SERVER_TLS_V1_2 = 0x00000105L;
        /** Tape Library Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-CLIENT-00000106; KMIPKIT-SRC-spec §11.41. */
        public static final long TAPE_LIBRARY_CLIENT = 0x00000106L;
        /** Tape Library Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-SERVER-00000107; KMIPKIT-SRC-spec §11.41. */
        public static final long TAPE_LIBRARY_SERVER = 0x00000107L;
        /** Symmetric Key Lifecycle Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-CLIENT-00000108; KMIPKIT-SRC-spec §11.41. */
        public static final long SYMMETRIC_KEY_LIFECYCLE_CLIENT = 0x00000108L;
        /** Symmetric Key Lifecycle Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-SERVER-00000109; KMIPKIT-SRC-spec §11.41. */
        public static final long SYMMETRIC_KEY_LIFECYCLE_SERVER = 0x00000109L;
        /** Asymmetric Key Lifecycle Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-CLIENT-0000010A; KMIPKIT-SRC-spec §11.41. */
        public static final long ASYMMETRIC_KEY_LIFECYCLE_CLIENT = 0x0000010AL;
        /** Asymmetric Key Lifecycle Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-SERVER-0000010B; KMIPKIT-SRC-spec §11.41. */
        public static final long ASYMMETRIC_KEY_LIFECYCLE_SERVER = 0x0000010BL;
        /** Basic Cryptographic Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-CLIENT-0000010C; KMIPKIT-SRC-spec §11.41. */
        public static final long BASIC_CRYPTOGRAPHIC_CLIENT = 0x0000010CL;
        /** Basic Cryptographic Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-SERVER-0000010D; KMIPKIT-SRC-spec §11.41. */
        public static final long BASIC_CRYPTOGRAPHIC_SERVER = 0x0000010DL;
        /** Advanced Cryptographic Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-CLIENT-0000010E; KMIPKIT-SRC-spec §11.41. */
        public static final long ADVANCED_CRYPTOGRAPHIC_CLIENT = 0x0000010EL;
        /** Advanced Cryptographic Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-SERVER-0000010F; KMIPKIT-SRC-spec §11.41. */
        public static final long ADVANCED_CRYPTOGRAPHIC_SERVER = 0x0000010FL;
        /** RNG Cryptographic Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-CLIENT-00000110; KMIPKIT-SRC-spec §11.41. */
        public static final long RNG_CRYPTOGRAPHIC_CLIENT = 0x00000110L;
        /** RNG Cryptographic Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-SERVER-00000111; KMIPKIT-SRC-spec §11.41. */
        public static final long RNG_CRYPTOGRAPHIC_SERVER = 0x00000111L;
        /** Basic Symmetric Key Foundry Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000112; KMIPKIT-SRC-spec §11.41. */
        public static final long BASIC_SYMMETRIC_KEY_FOUNDRY_CLIENT = 0x00000112L;
        /** Intermediate Symmetric Key Foundry Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-INTERMEDIATE-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000113; KMIPKIT-SRC-spec §11.41. */
        public static final long INTERMEDIATE_SYMMETRIC_KEY_FOUNDRY_CLIENT = 0x00000113L;
        /** Advanced Symmetric Key Foundry Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000114; KMIPKIT-SRC-spec §11.41. */
        public static final long ADVANCED_SYMMETRIC_KEY_FOUNDRY_CLIENT = 0x00000114L;
        /** Symmetric Key Foundry Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-FOUNDRY-SERVER-00000115; KMIPKIT-SRC-spec §11.41. */
        public static final long SYMMETRIC_KEY_FOUNDRY_SERVER = 0x00000115L;
        /** Opaque Managed Object Store Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-CLIENT-00000116; KMIPKIT-SRC-spec §11.41. */
        public static final long OPAQUE_MANAGED_OBJECT_STORE_CLIENT = 0x00000116L;
        /** Opaque Managed Object Store Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-SERVER-00000117; KMIPKIT-SRC-spec §11.41. */
        public static final long OPAQUE_MANAGED_OBJECT_STORE_SERVER = 0x00000117L;
        /** Storage Array with Self Encrypting Drive Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-CLIENT-0000011C; KMIPKIT-SRC-spec §11.41. */
        public static final long STORAGE_ARRAY_WITH_SELF_ENCRYPTING_DRIVE_CLIENT = 0x0000011CL;
        /** Storage Array with Self Encrypting Drive Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-SERVER-0000011D; KMIPKIT-SRC-spec §11.41. */
        public static final long STORAGE_ARRAY_WITH_SELF_ENCRYPTING_DRIVE_SERVER = 0x0000011DL;
        /** HTTPS Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-CLIENT-0000011E; KMIPKIT-SRC-spec §11.41. */
        public static final long HTTPS_CLIENT = 0x0000011EL;
        /** HTTPS Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-SERVER-0000011F; KMIPKIT-SRC-spec §11.41. */
        public static final long HTTPS_SERVER = 0x0000011FL;
        /** JSON Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-CLIENT-00000120; KMIPKIT-SRC-spec §11.41. */
        public static final long JSON_CLIENT = 0x00000120L;
        /** JSON Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-SERVER-00000121; KMIPKIT-SRC-spec §11.41. */
        public static final long JSON_SERVER = 0x00000121L;
        /** XML Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-CLIENT-00000122; KMIPKIT-SRC-spec §11.41. */
        public static final long XML_CLIENT = 0x00000122L;
        /** XML Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-SERVER-00000123; KMIPKIT-SRC-spec §11.41. */
        public static final long XML_SERVER = 0x00000123L;
        /** AES XTS Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-CLIENT-00000124; KMIPKIT-SRC-spec §11.41. */
        public static final long AES_XTS_CLIENT = 0x00000124L;
        /** AES XTS Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-SERVER-00000125; KMIPKIT-SRC-spec §11.41. */
        public static final long AES_XTS_SERVER = 0x00000125L;
        /** Quantum Safe Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-CLIENT-00000126; KMIPKIT-SRC-spec §11.41. */
        public static final long QUANTUM_SAFE_CLIENT = 0x00000126L;
        /** Quantum Safe Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-SERVER-00000127; KMIPKIT-SRC-spec §11.41. */
        public static final long QUANTUM_SAFE_SERVER = 0x00000127L;
        /** PKCS#11 Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-CLIENT-00000128; KMIPKIT-SRC-spec §11.41. */
        public static final long PKCS_11_CLIENT = 0x00000128L;
        /** PKCS#11 Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-SERVER-00000129; KMIPKIT-SRC-spec §11.41. */
        public static final long PKCS_11_SERVER = 0x00000129L;
        /** Baseline Client; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-CLIENT-0000012A; KMIPKIT-SRC-spec §11.41. */
        public static final long BASELINE_CLIENT = 0x0000012AL;
        /** Baseline Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-SERVER-0000012B; KMIPKIT-SRC-spec §11.41. */
        public static final long BASELINE_SERVER = 0x0000012BL;
        /** Complete Server; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-0000012C; KMIPKIT-SRC-spec §11.41. */
        public static final long COMPLETE_SERVER = 0x0000012CL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000104 -> "Complete Server Basic";
                case 0x00000105 -> "Complete Server TLS v1.2";
                case 0x00000106 -> "Tape Library Client";
                case 0x00000107 -> "Tape Library Server";
                case 0x00000108 -> "Symmetric Key Lifecycle Client";
                case 0x00000109 -> "Symmetric Key Lifecycle Server";
                case 0x0000010A -> "Asymmetric Key Lifecycle Client";
                case 0x0000010B -> "Asymmetric Key Lifecycle Server";
                case 0x0000010C -> "Basic Cryptographic Client";
                case 0x0000010D -> "Basic Cryptographic Server";
                case 0x0000010E -> "Advanced Cryptographic Client";
                case 0x0000010F -> "Advanced Cryptographic Server";
                case 0x00000110 -> "RNG Cryptographic Client";
                case 0x00000111 -> "RNG Cryptographic Server";
                case 0x00000112 -> "Basic Symmetric Key Foundry Client";
                case 0x00000113 -> "Intermediate Symmetric Key Foundry Client";
                case 0x00000114 -> "Advanced Symmetric Key Foundry Client";
                case 0x00000115 -> "Symmetric Key Foundry Server";
                case 0x00000116 -> "Opaque Managed Object Store Client";
                case 0x00000117 -> "Opaque Managed Object Store Server";
                case 0x0000011C -> "Storage Array with Self Encrypting Drive Client";
                case 0x0000011D -> "Storage Array with Self Encrypting Drive Server";
                case 0x0000011E -> "HTTPS Client";
                case 0x0000011F -> "HTTPS Server";
                case 0x00000120 -> "JSON Client";
                case 0x00000121 -> "JSON Server";
                case 0x00000122 -> "XML Client";
                case 0x00000123 -> "XML Server";
                case 0x00000124 -> "AES XTS Client";
                case 0x00000125 -> "AES XTS Server";
                case 0x00000126 -> "Quantum Safe Client";
                case 0x00000127 -> "Quantum Safe Server";
                case 0x00000128 -> "PKCS#11 Client";
                case 0x00000129 -> "PKCS#11 Server";
                case 0x0000012A -> "Baseline Client";
                case 0x0000012B -> "Baseline Server";
                case 0x0000012C -> "Complete Server";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Protection Level. */
    public static final class ProtectionLevel {
        private ProtectionLevel() {}
        /** High; KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-HIGH-00000001; KMIPKIT-SRC-spec §11.42. */
        public static final long HIGH = 0x00000001L;
        /** Low; KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-LOW-00000002; KMIPKIT-SRC-spec §11.42. */
        public static final long LOW = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "High";
                case 0x00000002 -> "Low";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Put Function. */
    public static final class PutFunction {
        private PutFunction() {}
        /** New; KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-NEW-00000001; KMIPKIT-SRC-spec §11.43. */
        public static final long NEW = 0x00000001L;
        /** Replace; KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-REPLACE-00000002; KMIPKIT-SRC-spec §11.43. */
        public static final long REPLACE = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "New";
                case 0x00000002 -> "Replace";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Query Function. */
    public static final class QueryFunction {
        private QueryFunction() {}
        /** Query Operations; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OPERATIONS-00000001; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_OPERATIONS = 0x00000001L;
        /** Query Objects; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OBJECTS-00000002; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_OBJECTS = 0x00000002L;
        /** Query Server Information; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-SERVER-INFORMATION-00000003; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_SERVER_INFORMATION = 0x00000003L;
        /** Query Application Namespaces; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-APPLICATION-NAMESPACES-00000004; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_APPLICATION_NAMESPACES = 0x00000004L;
        /** Query Extension List; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-LIST-00000005; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_EXTENSION_LIST = 0x00000005L;
        /** Query Extension Map; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-MAP-00000006; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_EXTENSION_MAP = 0x00000006L;
        /** Query Attestation Types; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-ATTESTATION-TYPES-00000007; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_ATTESTATION_TYPES = 0x00000007L;
        /** Query RNGs; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-RNGS-00000008; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_RNGS = 0x00000008L;
        /** Query Validations; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-VALIDATIONS-00000009; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_VALIDATIONS = 0x00000009L;
        /** Query Profiles; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-PROFILES-0000000A; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_PROFILES = 0x0000000AL;
        /** Query Capabilities; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CAPABILITIES-0000000B; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_CAPABILITIES = 0x0000000BL;
        /** Query Client Registration Methods; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CLIENT-REGISTRATION-METHODS-0000000C; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_CLIENT_REGISTRATION_METHODS = 0x0000000CL;
        /** Query Defaults Information; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-DEFAULTS-INFORMATION-0000000D; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_DEFAULTS_INFORMATION = 0x0000000DL;
        /** Query Storage Protection Masks; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-STORAGE-PROTECTION-MASKS-0000000E; KMIPKIT-SRC-spec §11.44. */
        public static final long QUERY_STORAGE_PROTECTION_MASKS = 0x0000000EL;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Query Operations";
                case 0x00000002 -> "Query Objects";
                case 0x00000003 -> "Query Server Information";
                case 0x00000004 -> "Query Application Namespaces";
                case 0x00000005 -> "Query Extension List";
                case 0x00000006 -> "Query Extension Map";
                case 0x00000007 -> "Query Attestation Types";
                case 0x00000008 -> "Query RNGs";
                case 0x00000009 -> "Query Validations";
                case 0x0000000A -> "Query Profiles";
                case 0x0000000B -> "Query Capabilities";
                case 0x0000000C -> "Query Client Registration Methods";
                case 0x0000000D -> "Query Defaults Information";
                case 0x0000000E -> "Query Storage Protection Masks";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Recommended Curve. */
    public static final class RecommendedCurve {
        private RecommendedCurve() {}
        /** P-192; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-192-00000001; KMIPKIT-SRC-spec §11.45. */
        public static final long P_192 = 0x00000001L;
        /** K-163; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-163-00000002; KMIPKIT-SRC-spec §11.45. */
        public static final long K_163 = 0x00000002L;
        /** B-163; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-163-00000003; KMIPKIT-SRC-spec §11.45. */
        public static final long B_163 = 0x00000003L;
        /** P-224; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-224-00000004; KMIPKIT-SRC-spec §11.45. */
        public static final long P_224 = 0x00000004L;
        /** K-233; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-233-00000005; KMIPKIT-SRC-spec §11.45. */
        public static final long K_233 = 0x00000005L;
        /** B-233; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-233-00000006; KMIPKIT-SRC-spec §11.45. */
        public static final long B_233 = 0x00000006L;
        /** P-256; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-256-00000007; KMIPKIT-SRC-spec §11.45. */
        public static final long P_256 = 0x00000007L;
        /** K-283; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-283-00000008; KMIPKIT-SRC-spec §11.45. */
        public static final long K_283 = 0x00000008L;
        /** B-283; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-283-00000009; KMIPKIT-SRC-spec §11.45. */
        public static final long B_283 = 0x00000009L;
        /** P-384; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-384-0000000A; KMIPKIT-SRC-spec §11.45. */
        public static final long P_384 = 0x0000000AL;
        /** K-409; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-409-0000000B; KMIPKIT-SRC-spec §11.45. */
        public static final long K_409 = 0x0000000BL;
        /** B-409; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-409-0000000C; KMIPKIT-SRC-spec §11.45. */
        public static final long B_409 = 0x0000000CL;
        /** P-521; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-521-0000000D; KMIPKIT-SRC-spec §11.45. */
        public static final long P_521 = 0x0000000DL;
        /** K-571; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-571-0000000E; KMIPKIT-SRC-spec §11.45. */
        public static final long K_571 = 0x0000000EL;
        /** B-571; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-571-0000000F; KMIPKIT-SRC-spec §11.45. */
        public static final long B_571 = 0x0000000FL;
        /** SECP112R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R1-00000010; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP112R1 = 0x00000010L;
        /** SECP112R2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R2-00000011; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP112R2 = 0x00000011L;
        /** SECP128R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R1-00000012; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP128R1 = 0x00000012L;
        /** SECP128R2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R2-00000013; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP128R2 = 0x00000013L;
        /** SECP160K1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160K1-00000014; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP160K1 = 0x00000014L;
        /** SECP160R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R1-00000015; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP160R1 = 0x00000015L;
        /** SECP160R2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R2-00000016; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP160R2 = 0x00000016L;
        /** SECP192K1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP192K1-00000017; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP192K1 = 0x00000017L;
        /** SECP224K1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP224K1-00000018; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP224K1 = 0x00000018L;
        /** SECP256K1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP256K1-00000019; KMIPKIT-SRC-spec §11.45. */
        public static final long SECP256K1 = 0x00000019L;
        /** SECT113R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R1-0000001A; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT113R1 = 0x0000001AL;
        /** SECT113R2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R2-0000001B; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT113R2 = 0x0000001BL;
        /** SECT131R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R1-0000001C; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT131R1 = 0x0000001CL;
        /** SECT131R2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R2-0000001D; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT131R2 = 0x0000001DL;
        /** SECT163R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT163R1-0000001E; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT163R1 = 0x0000001EL;
        /** SECT193R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R1-0000001F; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT193R1 = 0x0000001FL;
        /** SECT193R2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R2-00000020; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT193R2 = 0x00000020L;
        /** SECT239K1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT239K1-00000021; KMIPKIT-SRC-spec §11.45. */
        public static final long SECT239K1 = 0x00000021L;
        /** ANSIX9P192V2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V2-00000022; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9P192V2 = 0x00000022L;
        /** ANSIX9P192V3; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V3-00000023; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9P192V3 = 0x00000023L;
        /** ANSIX9P239V1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V1-00000024; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9P239V1 = 0x00000024L;
        /** ANSIX9P239V2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V2-00000025; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9P239V2 = 0x00000025L;
        /** ANSIX9P239V3; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V3-00000026; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9P239V3 = 0x00000026L;
        /** ANSIX9C2PNB163V1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V1-00000027; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB163V1 = 0x00000027L;
        /** ANSIX9C2PNB163V2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V2-00000028; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB163V2 = 0x00000028L;
        /** ANSIX9C2PNB163V3; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V3-00000029; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB163V3 = 0x00000029L;
        /** ANSIX9C2PNB176V1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB176V1-0000002A; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB176V1 = 0x0000002AL;
        /** ANSIX9C2TNB191V1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V1-0000002B; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB191V1 = 0x0000002BL;
        /** ANSIX9C2TNB191V2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V2-0000002C; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB191V2 = 0x0000002CL;
        /** ANSIX9C2TNB191V3; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V3-0000002D; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB191V3 = 0x0000002DL;
        /** ANSIX9C2PNB208W1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB208W1-0000002E; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB208W1 = 0x0000002EL;
        /** ANSIX9C2TNB239V1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V1-0000002F; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB239V1 = 0x0000002FL;
        /** ANSIX9C2TNB239V2; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V2-00000030; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB239V2 = 0x00000030L;
        /** ANSIX9C2TNB239V3; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V3-00000031; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB239V3 = 0x00000031L;
        /** ANSIX9C2PNB272W1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB272W1-00000032; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB272W1 = 0x00000032L;
        /** ANSIX9C2PNB304W1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB304W1-00000033; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB304W1 = 0x00000033L;
        /** ANSIX9C2TNB359V1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB359V1-00000034; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB359V1 = 0x00000034L;
        /** ANSIX9C2PNB368W1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB368W1-00000035; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2PNB368W1 = 0x00000035L;
        /** ANSIX9C2TNB431R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB431R1-00000036; KMIPKIT-SRC-spec §11.45. */
        public static final long ANSIX9C2TNB431R1 = 0x00000036L;
        /** BRAINPOOLP160R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160R1-00000037; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP160R1 = 0x00000037L;
        /** BRAINPOOLP160T1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160T1-00000038; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP160T1 = 0x00000038L;
        /** BRAINPOOLP192R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192R1-00000039; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP192R1 = 0x00000039L;
        /** BRAINPOOLP192T1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192T1-0000003A; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP192T1 = 0x0000003AL;
        /** BRAINPOOLP224R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224R1-0000003B; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP224R1 = 0x0000003BL;
        /** BRAINPOOLP224T1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224T1-0000003C; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP224T1 = 0x0000003CL;
        /** BRAINPOOLP256R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256R1-0000003D; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP256R1 = 0x0000003DL;
        /** BRAINPOOLP256T1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256T1-0000003E; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP256T1 = 0x0000003EL;
        /** BRAINPOOLP320R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320R1-0000003F; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP320R1 = 0x0000003FL;
        /** BRAINPOOLP320T1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320T1-00000040; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP320T1 = 0x00000040L;
        /** BRAINPOOLP384R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384R1-00000041; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP384R1 = 0x00000041L;
        /** BRAINPOOLP384T1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384T1-00000042; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP384T1 = 0x00000042L;
        /** BRAINPOOLP512R1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512R1-00000043; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP512R1 = 0x00000043L;
        /** BRAINPOOLP512T1; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512T1-00000044; KMIPKIT-SRC-spec §11.45. */
        public static final long BRAINPOOLP512T1 = 0x00000044L;
        /** CURVE25519; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE25519-00000045; KMIPKIT-SRC-spec §11.45. */
        public static final long CURVE25519 = 0x00000045L;
        /** CURVE448; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE448-00000046; KMIPKIT-SRC-spec §11.45. */
        public static final long CURVE448 = 0x00000046L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "P-192";
                case 0x00000002 -> "K-163";
                case 0x00000003 -> "B-163";
                case 0x00000004 -> "P-224";
                case 0x00000005 -> "K-233";
                case 0x00000006 -> "B-233";
                case 0x00000007 -> "P-256";
                case 0x00000008 -> "K-283";
                case 0x00000009 -> "B-283";
                case 0x0000000A -> "P-384";
                case 0x0000000B -> "K-409";
                case 0x0000000C -> "B-409";
                case 0x0000000D -> "P-521";
                case 0x0000000E -> "K-571";
                case 0x0000000F -> "B-571";
                case 0x00000010 -> "SECP112R1";
                case 0x00000011 -> "SECP112R2";
                case 0x00000012 -> "SECP128R1";
                case 0x00000013 -> "SECP128R2";
                case 0x00000014 -> "SECP160K1";
                case 0x00000015 -> "SECP160R1";
                case 0x00000016 -> "SECP160R2";
                case 0x00000017 -> "SECP192K1";
                case 0x00000018 -> "SECP224K1";
                case 0x00000019 -> "SECP256K1";
                case 0x0000001A -> "SECT113R1";
                case 0x0000001B -> "SECT113R2";
                case 0x0000001C -> "SECT131R1";
                case 0x0000001D -> "SECT131R2";
                case 0x0000001E -> "SECT163R1";
                case 0x0000001F -> "SECT193R1";
                case 0x00000020 -> "SECT193R2";
                case 0x00000021 -> "SECT239K1";
                case 0x00000022 -> "ANSIX9P192V2";
                case 0x00000023 -> "ANSIX9P192V3";
                case 0x00000024 -> "ANSIX9P239V1";
                case 0x00000025 -> "ANSIX9P239V2";
                case 0x00000026 -> "ANSIX9P239V3";
                case 0x00000027 -> "ANSIX9C2PNB163V1";
                case 0x00000028 -> "ANSIX9C2PNB163V2";
                case 0x00000029 -> "ANSIX9C2PNB163V3";
                case 0x0000002A -> "ANSIX9C2PNB176V1";
                case 0x0000002B -> "ANSIX9C2TNB191V1";
                case 0x0000002C -> "ANSIX9C2TNB191V2";
                case 0x0000002D -> "ANSIX9C2TNB191V3";
                case 0x0000002E -> "ANSIX9C2PNB208W1";
                case 0x0000002F -> "ANSIX9C2TNB239V1";
                case 0x00000030 -> "ANSIX9C2TNB239V2";
                case 0x00000031 -> "ANSIX9C2TNB239V3";
                case 0x00000032 -> "ANSIX9C2PNB272W1";
                case 0x00000033 -> "ANSIX9C2PNB304W1";
                case 0x00000034 -> "ANSIX9C2TNB359V1";
                case 0x00000035 -> "ANSIX9C2PNB368W1";
                case 0x00000036 -> "ANSIX9C2TNB431R1";
                case 0x00000037 -> "BRAINPOOLP160R1";
                case 0x00000038 -> "BRAINPOOLP160T1";
                case 0x00000039 -> "BRAINPOOLP192R1";
                case 0x0000003A -> "BRAINPOOLP192T1";
                case 0x0000003B -> "BRAINPOOLP224R1";
                case 0x0000003C -> "BRAINPOOLP224T1";
                case 0x0000003D -> "BRAINPOOLP256R1";
                case 0x0000003E -> "BRAINPOOLP256T1";
                case 0x0000003F -> "BRAINPOOLP320R1";
                case 0x00000040 -> "BRAINPOOLP320T1";
                case 0x00000041 -> "BRAINPOOLP384R1";
                case 0x00000042 -> "BRAINPOOLP384T1";
                case 0x00000043 -> "BRAINPOOLP512R1";
                case 0x00000044 -> "BRAINPOOLP512T1";
                case 0x00000045 -> "CURVE25519";
                case 0x00000046 -> "CURVE448";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Result Reason. */
    public static final class ResultReason {
        private ResultReason() {}
        /** Item Not Found; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ITEM-NOT-FOUND-00000001; KMIPKIT-SRC-spec §11.46. */
        public static final long ITEM_NOT_FOUND = 0x00000001L;
        /** Response Too Large; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESPONSE-TOO-LARGE-00000002; KMIPKIT-SRC-spec §11.46. */
        public static final long RESPONSE_TOO_LARGE = 0x00000002L;
        /** Authentication Not Successful; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-AUTHENTICATION-NOT-SUCCESSFUL-00000003; KMIPKIT-SRC-spec §11.46. */
        public static final long AUTHENTICATION_NOT_SUCCESSFUL = 0x00000003L;
        /** Invalid Message; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-MESSAGE-00000004; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_MESSAGE = 0x00000004L;
        /** Operation Not Supported; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OPERATION-NOT-SUPPORTED-00000005; KMIPKIT-SRC-spec §11.46. */
        public static final long OPERATION_NOT_SUPPORTED = 0x00000005L;
        /** Missing Data; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MISSING-DATA-00000006; KMIPKIT-SRC-spec §11.46. */
        public static final long MISSING_DATA = 0x00000006L;
        /** Invalid Field; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-FIELD-00000007; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_FIELD = 0x00000007L;
        /** Feature Not Supported; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-FEATURE-NOT-SUPPORTED-00000008; KMIPKIT-SRC-spec §11.46. */
        public static final long FEATURE_NOT_SUPPORTED = 0x00000008L;
        /** Operation Canceled By Requester; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OPERATION-CANCELED-BY-REQUESTER-00000009; KMIPKIT-SRC-spec §11.46. */
        public static final long OPERATION_CANCELED_BY_REQUESTER = 0x00000009L;
        /** Cryptographic Failure; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CRYPTOGRAPHIC-FAILURE-0000000A; KMIPKIT-SRC-spec §11.46. */
        public static final long CRYPTOGRAPHIC_FAILURE = 0x0000000AL;
        /** Permission Denied; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PERMISSION-DENIED-0000000C; KMIPKIT-SRC-spec §11.46. */
        public static final long PERMISSION_DENIED = 0x0000000CL;
        /** Object Archived; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-ARCHIVED-0000000D; KMIPKIT-SRC-spec §11.46. */
        public static final long OBJECT_ARCHIVED = 0x0000000DL;
        /** Application Namespace Not Supported; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-APPLICATION-NAMESPACE-NOT-SUPPORTED-0000000F; KMIPKIT-SRC-spec §11.46. */
        public static final long APPLICATION_NAMESPACE_NOT_SUPPORTED = 0x0000000FL;
        /** Key Format Type Not Supported; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-FORMAT-TYPE-NOT-SUPPORTED-00000010; KMIPKIT-SRC-spec §11.46. */
        public static final long KEY_FORMAT_TYPE_NOT_SUPPORTED = 0x00000010L;
        /** Key Compression Type Not Supported; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-COMPRESSION-TYPE-NOT-SUPPORTED-00000011; KMIPKIT-SRC-spec §11.46. */
        public static final long KEY_COMPRESSION_TYPE_NOT_SUPPORTED = 0x00000011L;
        /** Encoding Option Error; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ENCODING-OPTION-ERROR-00000012; KMIPKIT-SRC-spec §11.46. */
        public static final long ENCODING_OPTION_ERROR = 0x00000012L;
        /** Key Value Not Present; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-VALUE-NOT-PRESENT-00000013; KMIPKIT-SRC-spec §11.46. */
        public static final long KEY_VALUE_NOT_PRESENT = 0x00000013L;
        /** Attestation Required; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTESTATION-REQUIRED-00000014; KMIPKIT-SRC-spec §11.46. */
        public static final long ATTESTATION_REQUIRED = 0x00000014L;
        /** Attestation Failed; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTESTATION-FAILED-00000015; KMIPKIT-SRC-spec §11.46. */
        public static final long ATTESTATION_FAILED = 0x00000015L;
        /** Sensitive; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-SENSITIVE-00000016; KMIPKIT-SRC-spec §11.46. */
        public static final long SENSITIVE = 0x00000016L;
        /** Not Extractable; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NOT-EXTRACTABLE-00000017; KMIPKIT-SRC-spec §11.46. */
        public static final long NOT_EXTRACTABLE = 0x00000017L;
        /** Object Already Exists; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-ALREADY-EXISTS-00000018; KMIPKIT-SRC-spec §11.46. */
        public static final long OBJECT_ALREADY_EXISTS = 0x00000018L;
        /** Invalid Ticket; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-TICKET-00000019; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_TICKET = 0x00000019L;
        /** Usage Limit Exceeded; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-USAGE-LIMIT-EXCEEDED-0000001A; KMIPKIT-SRC-spec §11.46. */
        public static final long USAGE_LIMIT_EXCEEDED = 0x0000001AL;
        /** Numeric Range; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NUMERIC-RANGE-0000001B; KMIPKIT-SRC-spec §11.46. */
        public static final long NUMERIC_RANGE = 0x0000001BL;
        /** Invalid Data Type; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-DATA-TYPE-0000001C; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_DATA_TYPE = 0x0000001CL;
        /** Read Only Attribute; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-READ-ONLY-ATTRIBUTE-0000001D; KMIPKIT-SRC-spec §11.46. */
        public static final long READ_ONLY_ATTRIBUTE = 0x0000001DL;
        /** Multi Valued Attribute; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MULTI-VALUED-ATTRIBUTE-0000001E; KMIPKIT-SRC-spec §11.46. */
        public static final long MULTI_VALUED_ATTRIBUTE = 0x0000001EL;
        /** Unsupported Attribute; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-ATTRIBUTE-0000001F; KMIPKIT-SRC-spec §11.46. */
        public static final long UNSUPPORTED_ATTRIBUTE = 0x0000001FL;
        /** Attribute Instance Not Found; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-INSTANCE-NOT-FOUND-00000020; KMIPKIT-SRC-spec §11.46. */
        public static final long ATTRIBUTE_INSTANCE_NOT_FOUND = 0x00000020L;
        /** Attribute Not Found; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-NOT-FOUND-00000021; KMIPKIT-SRC-spec §11.46. */
        public static final long ATTRIBUTE_NOT_FOUND = 0x00000021L;
        /** Attribute Read Only; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-READ-ONLY-00000022; KMIPKIT-SRC-spec §11.46. */
        public static final long ATTRIBUTE_READ_ONLY = 0x00000022L;
        /** Attribute Single Valued; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-SINGLE-VALUED-00000023; KMIPKIT-SRC-spec §11.46. */
        public static final long ATTRIBUTE_SINGLE_VALUED = 0x00000023L;
        /** Bad Cryptographic Parameters; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-BAD-CRYPTOGRAPHIC-PARAMETERS-00000024; KMIPKIT-SRC-spec §11.46. */
        public static final long BAD_CRYPTOGRAPHIC_PARAMETERS = 0x00000024L;
        /** Bad Password; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-BAD-PASSWORD-00000025; KMIPKIT-SRC-spec §11.46. */
        public static final long BAD_PASSWORD = 0x00000025L;
        /** Codec Error; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CODEC-ERROR-00000026; KMIPKIT-SRC-spec §11.46. */
        public static final long CODEC_ERROR = 0x00000026L;
        /** Illegal Object Type; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ILLEGAL-OBJECT-TYPE-00000028; KMIPKIT-SRC-spec §11.46. */
        public static final long ILLEGAL_OBJECT_TYPE = 0x00000028L;
        /** Incompatible Cryptographic Usage Mask; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INCOMPATIBLE-CRYPTOGRAPHIC-USAGE-MASK-00000029; KMIPKIT-SRC-spec §11.46. */
        public static final long INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK = 0x00000029L;
        /** Internal Server Error; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INTERNAL-SERVER-ERROR-0000002A; KMIPKIT-SRC-spec §11.46. */
        public static final long INTERNAL_SERVER_ERROR = 0x0000002AL;
        /** Invalid Asynchronous Correlation Value; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ASYNCHRONOUS-CORRELATION-VALUE-0000002B; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_ASYNCHRONOUS_CORRELATION_VALUE = 0x0000002BL;
        /** Invalid Attribute; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ATTRIBUTE-0000002C; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_ATTRIBUTE = 0x0000002CL;
        /** Invalid Attribute Value; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ATTRIBUTE-VALUE-0000002D; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_ATTRIBUTE_VALUE = 0x0000002DL;
        /** Invalid Correlation Value; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-CORRELATION-VALUE-0000002E; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_CORRELATION_VALUE = 0x0000002EL;
        /** Invalid CSR; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-CSR-0000002F; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_CSR = 0x0000002FL;
        /** Invalid Object Type; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-OBJECT-TYPE-00000030; KMIPKIT-SRC-spec §11.46. */
        public static final long INVALID_OBJECT_TYPE = 0x00000030L;
        /** Key Wrap Type Not Supported; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-WRAP-TYPE-NOT-SUPPORTED-00000032; KMIPKIT-SRC-spec §11.46. */
        public static final long KEY_WRAP_TYPE_NOT_SUPPORTED = 0x00000032L;
        /** Missing Initialization Vector; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MISSING-INITIALIZATION-VECTOR-00000034; KMIPKIT-SRC-spec §11.46. */
        public static final long MISSING_INITIALIZATION_VECTOR = 0x00000034L;
        /** Non Unique Name Attribute; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NON-UNIQUE-NAME-ATTRIBUTE-00000035; KMIPKIT-SRC-spec §11.46. */
        public static final long NON_UNIQUE_NAME_ATTRIBUTE = 0x00000035L;
        /** Object Destroyed; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-DESTROYED-00000036; KMIPKIT-SRC-spec §11.46. */
        public static final long OBJECT_DESTROYED = 0x00000036L;
        /** Object Not Found; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-NOT-FOUND-00000037; KMIPKIT-SRC-spec §11.46. */
        public static final long OBJECT_NOT_FOUND = 0x00000037L;
        /** Not Authorised; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NOT-AUTHORISED-00000039; KMIPKIT-SRC-spec §11.46. */
        public static final long NOT_AUTHORISED = 0x00000039L;
        /** Server Limit Exceeded; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-SERVER-LIMIT-EXCEEDED-0000003A; KMIPKIT-SRC-spec §11.46. */
        public static final long SERVER_LIMIT_EXCEEDED = 0x0000003AL;
        /** Unknown Enumeration; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-ENUMERATION-0000003B; KMIPKIT-SRC-spec §11.46. */
        public static final long UNKNOWN_ENUMERATION = 0x0000003BL;
        /** Unknown Message Extension; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-MESSAGE-EXTENSION-0000003C; KMIPKIT-SRC-spec §11.46. */
        public static final long UNKNOWN_MESSAGE_EXTENSION = 0x0000003CL;
        /** Unknown Tag; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-TAG-0000003D; KMIPKIT-SRC-spec §11.46. */
        public static final long UNKNOWN_TAG = 0x0000003DL;
        /** Unsupported Cryptographic Parameters; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-CRYPTOGRAPHIC-PARAMETERS-0000003E; KMIPKIT-SRC-spec §11.46. */
        public static final long UNSUPPORTED_CRYPTOGRAPHIC_PARAMETERS = 0x0000003EL;
        /** Unsupported Protocol Version; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-PROTOCOL-VERSION-0000003F; KMIPKIT-SRC-spec §11.46. */
        public static final long UNSUPPORTED_PROTOCOL_VERSION = 0x0000003FL;
        /** Wrapping Object Archived; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-ARCHIVED-00000040; KMIPKIT-SRC-spec §11.46. */
        public static final long WRAPPING_OBJECT_ARCHIVED = 0x00000040L;
        /** Wrapping Object Destroyed; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-DESTROYED-00000041; KMIPKIT-SRC-spec §11.46. */
        public static final long WRAPPING_OBJECT_DESTROYED = 0x00000041L;
        /** Wrapping Object Not Found; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-NOT-FOUND-00000042; KMIPKIT-SRC-spec §11.46. */
        public static final long WRAPPING_OBJECT_NOT_FOUND = 0x00000042L;
        /** Wrong Key Lifecycle State; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRONG-KEY-LIFECYCLE-STATE-00000043; KMIPKIT-SRC-spec §11.46. */
        public static final long WRONG_KEY_LIFECYCLE_STATE = 0x00000043L;
        /** Protection Storage Unavailable; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PROTECTION-STORAGE-UNAVAILABLE-00000044; KMIPKIT-SRC-spec §11.46. */
        public static final long PROTECTION_STORAGE_UNAVAILABLE = 0x00000044L;
        /** PKCS#11 Codec Error; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-CODEC-ERROR-00000045; KMIPKIT-SRC-spec §11.46. */
        public static final long PKCS_11_CODEC_ERROR = 0x00000045L;
        /** PKCS#11 Invalid Function; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-INVALID-FUNCTION-00000046; KMIPKIT-SRC-spec §11.46. */
        public static final long PKCS_11_INVALID_FUNCTION = 0x00000046L;
        /** PKCS#11 Invalid Interface; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-INVALID-INTERFACE-00000047; KMIPKIT-SRC-spec §11.46. */
        public static final long PKCS_11_INVALID_INTERFACE = 0x00000047L;
        /** Private Protection Storage Unavailable; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PRIVATE-PROTECTION-STORAGE-UNAVAILABLE-00000048; KMIPKIT-SRC-spec §11.46. */
        public static final long PRIVATE_PROTECTION_STORAGE_UNAVAILABLE = 0x00000048L;
        /** Public Protection Storage Unavailable; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PUBLIC-PROTECTION-STORAGE-UNAVAILABLE-00000049; KMIPKIT-SRC-spec §11.46. */
        public static final long PUBLIC_PROTECTION_STORAGE_UNAVAILABLE = 0x00000049L;
        /** Unknown Object Group; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-OBJECT-GROUP-0000004A; KMIPKIT-SRC-spec §11.46. */
        public static final long UNKNOWN_OBJECT_GROUP = 0x0000004AL;
        /** Constraint Violation; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CONSTRAINT-VIOLATION-0000004B; KMIPKIT-SRC-spec §11.46. */
        public static final long CONSTRAINT_VIOLATION = 0x0000004BL;
        /** Duplicate Process Request; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-DUPLICATE-PROCESS-REQUEST-0000004C; KMIPKIT-SRC-spec §11.46. */
        public static final long DUPLICATE_PROCESS_REQUEST = 0x0000004CL;
        /** General Failure; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-GENERAL-FAILURE-00000100; KMIPKIT-SRC-spec §11.46. */
        public static final long GENERAL_FAILURE = 0x00000100L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Item Not Found";
                case 0x00000002 -> "Response Too Large";
                case 0x00000003 -> "Authentication Not Successful";
                case 0x00000004 -> "Invalid Message";
                case 0x00000005 -> "Operation Not Supported";
                case 0x00000006 -> "Missing Data";
                case 0x00000007 -> "Invalid Field";
                case 0x00000008 -> "Feature Not Supported";
                case 0x00000009 -> "Operation Canceled By Requester";
                case 0x0000000A -> "Cryptographic Failure";
                case 0x0000000C -> "Permission Denied";
                case 0x0000000D -> "Object Archived";
                case 0x0000000F -> "Application Namespace Not Supported";
                case 0x00000010 -> "Key Format Type Not Supported";
                case 0x00000011 -> "Key Compression Type Not Supported";
                case 0x00000012 -> "Encoding Option Error";
                case 0x00000013 -> "Key Value Not Present";
                case 0x00000014 -> "Attestation Required";
                case 0x00000015 -> "Attestation Failed";
                case 0x00000016 -> "Sensitive";
                case 0x00000017 -> "Not Extractable";
                case 0x00000018 -> "Object Already Exists";
                case 0x00000019 -> "Invalid Ticket";
                case 0x0000001A -> "Usage Limit Exceeded";
                case 0x0000001B -> "Numeric Range";
                case 0x0000001C -> "Invalid Data Type";
                case 0x0000001D -> "Read Only Attribute";
                case 0x0000001E -> "Multi Valued Attribute";
                case 0x0000001F -> "Unsupported Attribute";
                case 0x00000020 -> "Attribute Instance Not Found";
                case 0x00000021 -> "Attribute Not Found";
                case 0x00000022 -> "Attribute Read Only";
                case 0x00000023 -> "Attribute Single Valued";
                case 0x00000024 -> "Bad Cryptographic Parameters";
                case 0x00000025 -> "Bad Password";
                case 0x00000026 -> "Codec Error";
                case 0x00000028 -> "Illegal Object Type";
                case 0x00000029 -> "Incompatible Cryptographic Usage Mask";
                case 0x0000002A -> "Internal Server Error";
                case 0x0000002B -> "Invalid Asynchronous Correlation Value";
                case 0x0000002C -> "Invalid Attribute";
                case 0x0000002D -> "Invalid Attribute Value";
                case 0x0000002E -> "Invalid Correlation Value";
                case 0x0000002F -> "Invalid CSR";
                case 0x00000030 -> "Invalid Object Type";
                case 0x00000032 -> "Key Wrap Type Not Supported";
                case 0x00000034 -> "Missing Initialization Vector";
                case 0x00000035 -> "Non Unique Name Attribute";
                case 0x00000036 -> "Object Destroyed";
                case 0x00000037 -> "Object Not Found";
                case 0x00000039 -> "Not Authorised";
                case 0x0000003A -> "Server Limit Exceeded";
                case 0x0000003B -> "Unknown Enumeration";
                case 0x0000003C -> "Unknown Message Extension";
                case 0x0000003D -> "Unknown Tag";
                case 0x0000003E -> "Unsupported Cryptographic Parameters";
                case 0x0000003F -> "Unsupported Protocol Version";
                case 0x00000040 -> "Wrapping Object Archived";
                case 0x00000041 -> "Wrapping Object Destroyed";
                case 0x00000042 -> "Wrapping Object Not Found";
                case 0x00000043 -> "Wrong Key Lifecycle State";
                case 0x00000044 -> "Protection Storage Unavailable";
                case 0x00000045 -> "PKCS#11 Codec Error";
                case 0x00000046 -> "PKCS#11 Invalid Function";
                case 0x00000047 -> "PKCS#11 Invalid Interface";
                case 0x00000048 -> "Private Protection Storage Unavailable";
                case 0x00000049 -> "Public Protection Storage Unavailable";
                case 0x0000004A -> "Unknown Object Group";
                case 0x0000004B -> "Constraint Violation";
                case 0x0000004C -> "Duplicate Process Request";
                case 0x00000100 -> "General Failure";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Result Status. */
    public static final class ResultStatus {
        private ResultStatus() {}
        /** Success; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-SUCCESS-00000000; KMIPKIT-SRC-spec §11.47. */
        public static final long SUCCESS = 0x00000000L;
        /** Operation Failed; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-FAILED-00000001; KMIPKIT-SRC-spec §11.47. */
        public static final long OPERATION_FAILED = 0x00000001L;
        /** Operation Pending; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-PENDING-00000002; KMIPKIT-SRC-spec §11.47. */
        public static final long OPERATION_PENDING = 0x00000002L;
        /** Operation Undone; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-UNDONE-00000003; KMIPKIT-SRC-spec §11.47. */
        public static final long OPERATION_UNDONE = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000000 -> "Success";
                case 0x00000001 -> "Operation Failed";
                case 0x00000002 -> "Operation Pending";
                case 0x00000003 -> "Operation Undone";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Revocation Reason Code. */
    public static final class RevocationReasonCode {
        private RevocationReasonCode() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.48. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Key Compromise; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-KEY-COMPROMISE-00000002; KMIPKIT-SRC-spec §11.48. */
        public static final long KEY_COMPROMISE = 0x00000002L;
        /** CA Compromise; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CA-COMPROMISE-00000003; KMIPKIT-SRC-spec §11.48. */
        public static final long CA_COMPROMISE = 0x00000003L;
        /** Affiliation Changed; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-AFFILIATION-CHANGED-00000004; KMIPKIT-SRC-spec §11.48. */
        public static final long AFFILIATION_CHANGED = 0x00000004L;
        /** Superseded; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-SUPERSEDED-00000005; KMIPKIT-SRC-spec §11.48. */
        public static final long SUPERSEDED = 0x00000005L;
        /** Cessation of Operation; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CESSATION-OF-OPERATION-00000006; KMIPKIT-SRC-spec §11.48. */
        public static final long CESSATION_OF_OPERATION = 0x00000006L;
        /** Privilege Withdrawn; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-PRIVILEGE-WITHDRAWN-00000007; KMIPKIT-SRC-spec §11.48. */
        public static final long PRIVILEGE_WITHDRAWN = 0x00000007L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Key Compromise";
                case 0x00000003 -> "CA Compromise";
                case 0x00000004 -> "Affiliation Changed";
                case 0x00000005 -> "Superseded";
                case 0x00000006 -> "Cessation of Operation";
                case 0x00000007 -> "Privilege Withdrawn";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for RNG Algorithm. */
    public static final class RngAlgorithm {
        private RngAlgorithm() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.49. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** FIPS 186-2; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-FIPS-186-2-00000002; KMIPKIT-SRC-spec §11.49. */
        public static final long FIPS_186_2 = 0x00000002L;
        /** DRBG; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-DRBG-00000003; KMIPKIT-SRC-spec §11.49. */
        public static final long DRBG = 0x00000003L;
        /** NRBG; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-NRBG-00000004; KMIPKIT-SRC-spec §11.49. */
        public static final long NRBG = 0x00000004L;
        /** ANSI X9.31; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-31-00000005; KMIPKIT-SRC-spec §11.49. */
        public static final long ANSI_X9_31 = 0x00000005L;
        /** ANSI X9.62; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-62-00000006; KMIPKIT-SRC-spec §11.49. */
        public static final long ANSI_X9_62 = 0x00000006L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "FIPS 186-2";
                case 0x00000003 -> "DRBG";
                case 0x00000004 -> "NRBG";
                case 0x00000005 -> "ANSI X9.31";
                case 0x00000006 -> "ANSI X9.62";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for RNG Mode. */
    public static final class RngMode {
        private RngMode() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.50. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Shared Instantiation; KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-SHARED-INSTANTIATION-00000002; KMIPKIT-SRC-spec §11.50. */
        public static final long SHARED_INSTANTIATION = 0x00000002L;
        /** Non-Shared Instantiation; KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-NON-SHARED-INSTANTIATION-00000003; KMIPKIT-SRC-spec §11.50. */
        public static final long NON_SHARED_INSTANTIATION = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Shared Instantiation";
                case 0x00000003 -> "Non-Shared Instantiation";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Rotate Name Type. */
    public static final class RotateNameType {
        private RotateNameType() {}
        /** Uninterpreted Text String; KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.51. */
        public static final long UNINTERPRETED_TEXT_STRING = 0x00000001L;
        /** URI; KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.51. */
        public static final long URI = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Uninterpreted Text String";
                case 0x00000002 -> "URI";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Secret Data Type. */
    public static final class SecretDataType {
        private SecretDataType() {}
        /** Password; KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-PASSWORD-00000001; KMIPKIT-SRC-spec §11.52. */
        public static final long PASSWORD = 0x00000001L;
        /** Seed; KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-SEED-00000002; KMIPKIT-SRC-spec §11.52. */
        public static final long SEED = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Password";
                case 0x00000002 -> "Seed";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Shredding Algorithm. */
    public static final class ShreddingAlgorithm {
        private ShreddingAlgorithm() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.53. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Cryptographic; KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-CRYPTOGRAPHIC-00000002; KMIPKIT-SRC-spec §11.53. */
        public static final long CRYPTOGRAPHIC = 0x00000002L;
        /** Unsupported; KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSUPPORTED-00000003; KMIPKIT-SRC-spec §11.53. */
        public static final long UNSUPPORTED = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Cryptographic";
                case 0x00000003 -> "Unsupported";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Split Key Method. */
    public static final class SplitKeyMethod {
        private SplitKeyMethod() {}
        /** XOR; KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-XOR-00000001; KMIPKIT-SRC-spec §11.54. */
        public static final long XOR = 0x00000001L;
        /** Polynomial Sharing GF (2¹⁶); KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000002; KMIPKIT-SRC-spec §11.54. */
        public static final long POLYNOMIAL_SHARING_GF_216 = 0x00000002L;
        /** Polynomial Sharing Prime Field; KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-PRIME-FIELD-00000003; KMIPKIT-SRC-spec §11.54. */
        public static final long POLYNOMIAL_SHARING_PRIME_FIELD = 0x00000003L;
        /** Polynomial Sharing GF (2⁸); KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000004; KMIPKIT-SRC-spec §11.54. */
        public static final long POLYNOMIAL_SHARING_GF_28 = 0x00000004L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "XOR";
                case 0x00000002 -> "Polynomial Sharing GF (2¹⁶)";
                case 0x00000003 -> "Polynomial Sharing Prime Field";
                case 0x00000004 -> "Polynomial Sharing GF (2⁸)";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for State. */
    public static final class State {
        private State() {}
        /** Pre-Active; KMIPKIT-ELEM-ENUM-VALUE-STATE-PRE-ACTIVE-00000001; KMIPKIT-SRC-spec §11.55. */
        public static final long PRE_ACTIVE = 0x00000001L;
        /** Active; KMIPKIT-ELEM-ENUM-VALUE-STATE-ACTIVE-00000002; KMIPKIT-SRC-spec §11.55. */
        public static final long ACTIVE = 0x00000002L;
        /** Deactivated; KMIPKIT-ELEM-ENUM-VALUE-STATE-DEACTIVATED-00000003; KMIPKIT-SRC-spec §11.55. */
        public static final long DEACTIVATED = 0x00000003L;
        /** Compromised; KMIPKIT-ELEM-ENUM-VALUE-STATE-COMPROMISED-00000004; KMIPKIT-SRC-spec §11.55. */
        public static final long COMPROMISED = 0x00000004L;
        /** Destroyed; KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-00000005; KMIPKIT-SRC-spec §11.55. */
        public static final long DESTROYED = 0x00000005L;
        /** Destroyed Compromised; KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-COMPROMISED-00000006; KMIPKIT-SRC-spec §11.55. */
        public static final long DESTROYED_COMPROMISED = 0x00000006L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Pre-Active";
                case 0x00000002 -> "Active";
                case 0x00000003 -> "Deactivated";
                case 0x00000004 -> "Compromised";
                case 0x00000005 -> "Destroyed";
                case 0x00000006 -> "Destroyed Compromised";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Tag. */
    public static final class Tag {
        private Tag() {}
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Ticket Type. */
    public static final class TicketType {
        private TicketType() {}
        /** Login; KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-LOGIN-00000001; KMIPKIT-SRC-spec §11.57. */
        public static final long LOGIN = 0x00000001L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Login";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Unique Identifier. */
    public static final class UniqueIdentifier {
        private UniqueIdentifier() {}
        /** ID Placeholder; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-ID-PLACEHOLDER-00000001; KMIPKIT-SRC-spec §11.58. */
        public static final long ID_PLACEHOLDER = 0x00000001L;
        /** Certify; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CERTIFY-00000002; KMIPKIT-SRC-spec §11.58. */
        public static final long CERTIFY = 0x00000002L;
        /** Create; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-00000003; KMIPKIT-SRC-spec §11.58. */
        public static final long CREATE = 0x00000003L;
        /** Create Key Pair; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-00000004; KMIPKIT-SRC-spec §11.58. */
        public static final long CREATE_KEY_PAIR = 0x00000004L;
        /** Create Key Pair Private Key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PRIVATE-KEY-00000005; KMIPKIT-SRC-spec §11.58. */
        public static final long CREATE_KEY_PAIR_PRIVATE_KEY = 0x00000005L;
        /** Create Key Pair Public Key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PUBLIC-KEY-00000006; KMIPKIT-SRC-spec §11.58. */
        public static final long CREATE_KEY_PAIR_PUBLIC_KEY = 0x00000006L;
        /** Create Split Key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-SPLIT-KEY-00000007; KMIPKIT-SRC-spec §11.58. */
        public static final long CREATE_SPLIT_KEY = 0x00000007L;
        /** Derive Key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-DERIVE-KEY-00000008; KMIPKIT-SRC-spec §11.58. */
        public static final long DERIVE_KEY = 0x00000008L;
        /** Import; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-IMPORT-00000009; KMIPKIT-SRC-spec §11.58. */
        public static final long IMPORT = 0x00000009L;
        /** Join Split Key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-JOIN-SPLIT-KEY-0000000A; KMIPKIT-SRC-spec §11.58. */
        public static final long JOIN_SPLIT_KEY = 0x0000000AL;
        /** Locate; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-LOCATE-0000000B; KMIPKIT-SRC-spec §11.58. */
        public static final long LOCATE = 0x0000000BL;
        /** Register; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-REGISTER-0000000C; KMIPKIT-SRC-spec §11.58. */
        public static final long REGISTER = 0x0000000CL;
        /** Re-key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-0000000D; KMIPKIT-SRC-spec §11.58. */
        public static final long RE_KEY = 0x0000000DL;
        /** Re-certify; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-CERTIFY-0000000E; KMIPKIT-SRC-spec §11.58. */
        public static final long RE_CERTIFY = 0x0000000EL;
        /** Re-key Key Pair; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-0000000F; KMIPKIT-SRC-spec §11.58. */
        public static final long RE_KEY_KEY_PAIR = 0x0000000FL;
        /** Re-key Key Pair Private Key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PRIVATE-KEY-00000010; KMIPKIT-SRC-spec §11.58. */
        public static final long RE_KEY_KEY_PAIR_PRIVATE_KEY = 0x00000010L;
        /** Re-key Key Pair Public Key; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PUBLIC-KEY-00000011; KMIPKIT-SRC-spec §11.58. */
        public static final long RE_KEY_KEY_PAIR_PUBLIC_KEY = 0x00000011L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "ID Placeholder";
                case 0x00000002 -> "Certify";
                case 0x00000003 -> "Create";
                case 0x00000004 -> "Create Key Pair";
                case 0x00000005 -> "Create Key Pair Private Key";
                case 0x00000006 -> "Create Key Pair Public Key";
                case 0x00000007 -> "Create Split Key";
                case 0x00000008 -> "Derive Key";
                case 0x00000009 -> "Import";
                case 0x0000000A -> "Join Split Key";
                case 0x0000000B -> "Locate";
                case 0x0000000C -> "Register";
                case 0x0000000D -> "Re-key";
                case 0x0000000E -> "Re-certify";
                case 0x0000000F -> "Re-key Key Pair";
                case 0x00000010 -> "Re-key Key Pair Private Key";
                case 0x00000011 -> "Re-key Key Pair Public Key";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Unwrap Mode. */
    public static final class UnwrapMode {
        private UnwrapMode() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.59. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Processed; KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-PROCESSED-00000002; KMIPKIT-SRC-spec §11.59. */
        public static final long PROCESSED = 0x00000002L;
        /** Not Processed; KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-NOT-PROCESSED-00000003; KMIPKIT-SRC-spec §11.59. */
        public static final long NOT_PROCESSED = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Processed";
                case 0x00000003 -> "Not Processed";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Usage Limits Unit. */
    public static final class UsageLimitsUnit {
        private UsageLimitsUnit() {}
        /** Byte; KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-BYTE-00000001; KMIPKIT-SRC-spec §11.60. */
        public static final long BYTE = 0x00000001L;
        /** Object; KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-OBJECT-00000002; KMIPKIT-SRC-spec §11.60. */
        public static final long OBJECT = 0x00000002L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Byte";
                case 0x00000002 -> "Object";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Validation Authority Type. */
    public static final class ValidationAuthorityType {
        private ValidationAuthorityType() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.63. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** NIST CMVP; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-NIST-CMVP-00000002; KMIPKIT-SRC-spec §11.63. */
        public static final long NIST_CMVP = 0x00000002L;
        /** Common Criteria; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-COMMON-CRITERIA-00000003; KMIPKIT-SRC-spec §11.63. */
        public static final long COMMON_CRITERIA = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "NIST CMVP";
                case 0x00000003 -> "Common Criteria";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Validation Type. */
    public static final class ValidationType {
        private ValidationType() {}
        /** Unspecified; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.64. */
        public static final long UNSPECIFIED = 0x00000001L;
        /** Hardware; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HARDWARE-00000002; KMIPKIT-SRC-spec §11.64. */
        public static final long HARDWARE = 0x00000002L;
        /** Software; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-SOFTWARE-00000003; KMIPKIT-SRC-spec §11.64. */
        public static final long SOFTWARE = 0x00000003L;
        /** Firmware; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-FIRMWARE-00000004; KMIPKIT-SRC-spec §11.64. */
        public static final long FIRMWARE = 0x00000004L;
        /** Hybrid; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HYBRID-00000005; KMIPKIT-SRC-spec §11.64. */
        public static final long HYBRID = 0x00000005L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Unspecified";
                case 0x00000002 -> "Hardware";
                case 0x00000003 -> "Software";
                case 0x00000004 -> "Firmware";
                case 0x00000005 -> "Hybrid";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Validity Indicator. */
    public static final class ValidityIndicator {
        private ValidityIndicator() {}
        /** Valid; KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-VALID-00000001; KMIPKIT-SRC-spec §11.61. */
        public static final long VALID = 0x00000001L;
        /** Invalid; KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-INVALID-00000002; KMIPKIT-SRC-spec §11.61. */
        public static final long INVALID = 0x00000002L;
        /** Unknown; KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-UNKNOWN-00000003; KMIPKIT-SRC-spec §11.61. */
        public static final long UNKNOWN = 0x00000003L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Valid";
                case 0x00000002 -> "Invalid";
                case 0x00000003 -> "Unknown";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Wrapping Method. */
    public static final class WrappingMethod {
        private WrappingMethod() {}
        /** Encrypt; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-00000001; KMIPKIT-SRC-spec §11.62. */
        public static final long ENCRYPT = 0x00000001L;
        /** MAC/sign; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-00000002; KMIPKIT-SRC-spec §11.62. */
        public static final long MAC_SIGN = 0x00000002L;
        /** Encrypt then MAC/sign; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-THEN-MAC-SIGN-00000003; KMIPKIT-SRC-spec §11.62. */
        public static final long ENCRYPT_THEN_MAC_SIGN = 0x00000003L;
        /** MAC/sign then encrypt; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-THEN-ENCRYPT-00000004; KMIPKIT-SRC-spec §11.62. */
        public static final long MAC_SIGN_THEN_ENCRYPT = 0x00000004L;
        /** TR-31; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-TR-31-00000005; KMIPKIT-SRC-spec §11.62. */
        public static final long TR_31 = 0x00000005L;
        /** Returns the assigned catalog name, or null for an unknown raw value. */
        public static String name(long raw) {
            if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
            return switch ((int) raw) {
                case 0x00000001 -> "Encrypt";
                case 0x00000002 -> "MAC/sign";
                case 0x00000003 -> "Encrypt then MAC/sign";
                case 0x00000004 -> "MAC/sign then encrypt";
                case 0x00000005 -> "TR-31";
                default -> null;
            };
        }
    }

    /** Named KMIP 2.1 values for Cryptographic Usage Mask. */
    public static final class CryptographicUsageMask {
        private CryptographicUsageMask() {}
        /** Sign; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-SIGN-00000001; KMIPKIT-SRC-spec §12.1. */
        public static final long SIGN = 0x00000001L;
        /** Verify; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-VERIFY-00000002; KMIPKIT-SRC-spec §12.1. */
        public static final long VERIFY = 0x00000002L;
        /** Encrypt; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-ENCRYPT-00000004; KMIPKIT-SRC-spec §12.1. */
        public static final long ENCRYPT = 0x00000004L;
        /** Decrypt; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DECRYPT-00000008; KMIPKIT-SRC-spec §12.1. */
        public static final long DECRYPT = 0x00000008L;
        /** Wrap Key; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-WRAP-KEY-00000010; KMIPKIT-SRC-spec §12.1. */
        public static final long WRAP_KEY = 0x00000010L;
        /** Unwrap Key; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNWRAP-KEY-00000020; KMIPKIT-SRC-spec §12.1. */
        public static final long UNWRAP_KEY = 0x00000020L;
        /** MAC Generate; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-GENERATE-00000080; KMIPKIT-SRC-spec §12.1. */
        public static final long MAC_GENERATE = 0x00000080L;
        /** MAC Verify; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-VERIFY-00000100; KMIPKIT-SRC-spec §12.1. */
        public static final long MAC_VERIFY = 0x00000100L;
        /** Derive Key; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DERIVE-KEY-00000200; KMIPKIT-SRC-spec §12.1. */
        public static final long DERIVE_KEY = 0x00000200L;
        /** Key Agreement; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-KEY-AGREEMENT-00000800; KMIPKIT-SRC-spec §12.1. */
        public static final long KEY_AGREEMENT = 0x00000800L;
        /** Certificate Sign; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CERTIFICATE-SIGN-00001000; KMIPKIT-SRC-spec §12.1. */
        public static final long CERTIFICATE_SIGN = 0x00001000L;
        /** CRL Sign; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CRL-SIGN-00002000; KMIPKIT-SRC-spec §12.1. */
        public static final long CRL_SIGN = 0x00002000L;
        /** Authenticate; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-AUTHENTICATE-00100000; KMIPKIT-SRC-spec §12.1. */
        public static final long AUTHENTICATE = 0x00100000L;
        /** Unrestricted; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNRESTRICTED-00200000; KMIPKIT-SRC-spec §12.1. */
        public static final long UNRESTRICTED = 0x00200000L;
        /** FPE Encrypt; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-ENCRYPT-00400000; KMIPKIT-SRC-spec §12.1. */
        public static final long FPE_ENCRYPT = 0x00400000L;
        /** FPE Decrypt; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-DECRYPT-00800000; KMIPKIT-SRC-spec §12.1. */
        public static final long FPE_DECRYPT = 0x00800000L;
        public static final long KNOWN_MASK = 0x00F03BBFL;
        /** Returns the assigned bits without changing the original raw value. */
        public static long knownBits(long raw) { return raw & KNOWN_MASK; }
        /** Returns all raw bits not assigned by KMIP 2.1. */
        public static long unknownBits(long raw) { return raw & ~KNOWN_MASK & 0xFFFF_FFFFL; }
        /** Returns names for set assigned bits in ascending numeric order. */
        public static java.util.List<String> names(long raw) {
            var names = new java.util.ArrayList<String>();
            if ((raw & SIGN) != 0) names.add("Sign");
            if ((raw & VERIFY) != 0) names.add("Verify");
            if ((raw & ENCRYPT) != 0) names.add("Encrypt");
            if ((raw & DECRYPT) != 0) names.add("Decrypt");
            if ((raw & WRAP_KEY) != 0) names.add("Wrap Key");
            if ((raw & UNWRAP_KEY) != 0) names.add("Unwrap Key");
            if ((raw & MAC_GENERATE) != 0) names.add("MAC Generate");
            if ((raw & MAC_VERIFY) != 0) names.add("MAC Verify");
            if ((raw & DERIVE_KEY) != 0) names.add("Derive Key");
            if ((raw & KEY_AGREEMENT) != 0) names.add("Key Agreement");
            if ((raw & CERTIFICATE_SIGN) != 0) names.add("Certificate Sign");
            if ((raw & CRL_SIGN) != 0) names.add("CRL Sign");
            if ((raw & AUTHENTICATE) != 0) names.add("Authenticate");
            if ((raw & UNRESTRICTED) != 0) names.add("Unrestricted");
            if ((raw & FPE_ENCRYPT) != 0) names.add("FPE Encrypt");
            if ((raw & FPE_DECRYPT) != 0) names.add("FPE Decrypt");
            return java.util.List.copyOf(names);
        }
    }

    /** Named KMIP 2.1 values for Protection Storage Mask. */
    public static final class ProtectionStorageMask {
        private ProtectionStorageMask() {}
        /** Software; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SOFTWARE-00000001; KMIPKIT-SRC-spec §12.2. */
        public static final long SOFTWARE = 0x00000001L;
        /** Hardware; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HARDWARE-00000002; KMIPKIT-SRC-spec §12.2. */
        public static final long HARDWARE = 0x00000002L;
        /** On Processor; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PROCESSOR-00000004; KMIPKIT-SRC-spec §12.2. */
        public static final long ON_PROCESSOR = 0x00000004L;
        /** On System; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-SYSTEM-00000008; KMIPKIT-SRC-spec §12.2. */
        public static final long ON_SYSTEM = 0x00000008L;
        /** Off System; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-SYSTEM-00000010; KMIPKIT-SRC-spec §12.2. */
        public static final long OFF_SYSTEM = 0x00000010L;
        /** Hypervisor; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HYPERVISOR-00000020; KMIPKIT-SRC-spec §12.2. */
        public static final long HYPERVISOR = 0x00000020L;
        /** Operating System; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OPERATING-SYSTEM-00000040; KMIPKIT-SRC-spec §12.2. */
        public static final long OPERATING_SYSTEM = 0x00000040L;
        /** Container; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-CONTAINER-00000080; KMIPKIT-SRC-spec §12.2. */
        public static final long CONTAINER = 0x00000080L;
        /** On Premises; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PREMISES-00000100; KMIPKIT-SRC-spec §12.2. */
        public static final long ON_PREMISES = 0x00000100L;
        /** Off Premises; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-PREMISES-00000200; KMIPKIT-SRC-spec §12.2. */
        public static final long OFF_PREMISES = 0x00000200L;
        /** Self Managed; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SELF-MANAGED-00000400; KMIPKIT-SRC-spec §12.2. */
        public static final long SELF_MANAGED = 0x00000400L;
        /** Outsourced; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OUTSOURCED-00000800; KMIPKIT-SRC-spec §12.2. */
        public static final long OUTSOURCED = 0x00000800L;
        /** Validated; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-VALIDATED-00001000; KMIPKIT-SRC-spec §12.2. */
        public static final long VALIDATED = 0x00001000L;
        /** Same Jurisdiction; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SAME-JURISDICTION-00002000; KMIPKIT-SRC-spec §12.2. */
        public static final long SAME_JURISDICTION = 0x00002000L;
        public static final long KNOWN_MASK = 0x00003FFFL;
        /** Returns the assigned bits without changing the original raw value. */
        public static long knownBits(long raw) { return raw & KNOWN_MASK; }
        /** Returns all raw bits not assigned by KMIP 2.1. */
        public static long unknownBits(long raw) { return raw & ~KNOWN_MASK & 0xFFFF_FFFFL; }
        /** Returns names for set assigned bits in ascending numeric order. */
        public static java.util.List<String> names(long raw) {
            var names = new java.util.ArrayList<String>();
            if ((raw & SOFTWARE) != 0) names.add("Software");
            if ((raw & HARDWARE) != 0) names.add("Hardware");
            if ((raw & ON_PROCESSOR) != 0) names.add("On Processor");
            if ((raw & ON_SYSTEM) != 0) names.add("On System");
            if ((raw & OFF_SYSTEM) != 0) names.add("Off System");
            if ((raw & HYPERVISOR) != 0) names.add("Hypervisor");
            if ((raw & OPERATING_SYSTEM) != 0) names.add("Operating System");
            if ((raw & CONTAINER) != 0) names.add("Container");
            if ((raw & ON_PREMISES) != 0) names.add("On Premises");
            if ((raw & OFF_PREMISES) != 0) names.add("Off Premises");
            if ((raw & SELF_MANAGED) != 0) names.add("Self Managed");
            if ((raw & OUTSOURCED) != 0) names.add("Outsourced");
            if ((raw & VALIDATED) != 0) names.add("Validated");
            if ((raw & SAME_JURISDICTION) != 0) names.add("Same Jurisdiction");
            return java.util.List.copyOf(names);
        }
    }

    /** Named KMIP 2.1 values for Storage Status Mask. */
    public static final class StorageStatusMask {
        private StorageStatusMask() {}
        /** On-line storage; KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ON-LINE-STORAGE-00000001; KMIPKIT-SRC-spec §12.3. */
        public static final long ON_LINE_STORAGE = 0x00000001L;
        /** Archival storage; KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ARCHIVAL-STORAGE-00000002; KMIPKIT-SRC-spec §12.3. */
        public static final long ARCHIVAL_STORAGE = 0x00000002L;
        /** Destroyed storage; KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-DESTROYED-STORAGE-00000004; KMIPKIT-SRC-spec §12.3. */
        public static final long DESTROYED_STORAGE = 0x00000004L;
        public static final long KNOWN_MASK = 0x00000007L;
        /** Returns the assigned bits without changing the original raw value. */
        public static long knownBits(long raw) { return raw & KNOWN_MASK; }
        /** Returns all raw bits not assigned by KMIP 2.1. */
        public static long unknownBits(long raw) { return raw & ~KNOWN_MASK & 0xFFFF_FFFFL; }
        /** Returns names for set assigned bits in ascending numeric order. */
        public static java.util.List<String> names(long raw) {
            var names = new java.util.ArrayList<String>();
            if ((raw & ON_LINE_STORAGE) != 0) names.add("On-line storage");
            if ((raw & ARCHIVAL_STORAGE) != 0) names.add("Archival storage");
            if ((raw & DESTROYED_STORAGE) != 0) names.add("Destroyed storage");
            return java.util.List.copyOf(names);
        }
    }

    // Exact assigned values added by later KMIP editions.
    /** Set Defaults; kmip-2.1 §11.36, Table 470. */
    public static final long V2_1_OPERATION_SET_DEFAULTS = 0x00000036L;
    /** Set Constraints; kmip-2.1 §11.36, Table 470. */
    public static final long V2_1_OPERATION_SET_CONSTRAINTS = 0x00000037L;
    /** Get Constraints; kmip-2.1 §11.36, Table 470. */
    public static final long V2_1_OPERATION_GET_CONSTRAINTS = 0x00000038L;
    /** Query Asynchronous Requests; kmip-2.1 §11.36, Table 470. */
    public static final long V2_1_OPERATION_QUERY_ASYNCHRONOUS_REQUESTS = 0x00000039L;
    /** Process; kmip-2.1 §11.36, Table 470. */
    public static final long V2_1_OPERATION_PROCESS = 0x0000003AL;
    /** Ping; kmip-2.1 §11.36, Table 470. */
    public static final long V2_1_OPERATION_PING = 0x0000003BL;
    /** Returns the KMIP v2.1 Operation name, or null for an unassigned value. */
    public static String v2_1OperationName(long raw) {
        if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
        return switch ((int) raw) {
            case 0x00000036 -> "Set Defaults";
            case 0x00000037 -> "Set Constraints";
            case 0x00000038 -> "Get Constraints";
            case 0x00000039 -> "Query Asynchronous Requests";
            case 0x0000003A -> "Process";
            case 0x0000003B -> "Ping";
            default -> null;
        };
    }
    /** Create Group; kmip-3.0 §11.37, Table 584. */
    public static final long V3_0_OPERATION_CREATE_GROUP = 0x0000003CL;
    /** Obliterate; kmip-3.0 §11.37, Table 584. */
    public static final long V3_0_OPERATION_OBLITERATE = 0x0000003DL;
    /** Create User; kmip-3.0 §11.37, Table 584. */
    public static final long V3_0_OPERATION_CREATE_USER = 0x0000003EL;
    /** Create Credential; kmip-3.0 §11.37, Table 584. */
    public static final long V3_0_OPERATION_CREATE_CREDENTIAL = 0x0000003FL;
    /** Deactivate; kmip-3.0 §11.37, Table 584. */
    public static final long V3_0_OPERATION_DEACTIVATE = 0x00000040L;
    /** Encapsulate; kmip-3.0 §11.37, Table 584. */
    public static final long V3_0_OPERATION_ENCAPSULATE = 0x00000041L;
    /** Decapsulate; kmip-3.0 §11.37, Table 584. */
    public static final long V3_0_OPERATION_DECAPSULATE = 0x00000042L;
    /** Returns the KMIP v3.0 Operation name, or null for an unassigned value. */
    public static String v3_0OperationName(long raw) {
        if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
        return switch ((int) raw) {
            case 0x0000003C -> "Create Group";
            case 0x0000003D -> "Obliterate";
            case 0x0000003E -> "Create User";
            case 0x0000003F -> "Create Credential";
            case 0x00000040 -> "Deactivate";
            case 0x00000041 -> "Encapsulate";
            case 0x00000042 -> "Decapsulate";
            default -> null;
        };
    }
    /** User; kmip-3.0 §11.34, Table 581. */
    public static final long V3_0_OBJECT_CLASS_USER = 0x00000001L;
    /** System; kmip-3.0 §11.34, Table 581. */
    public static final long V3_0_OBJECT_CLASS_SYSTEM = 0x00000002L;
    /** Returns the KMIP v3.0 Object Class name, or null for an unassigned value. */
    public static String v3_0ObjectClassName(long raw) {
        if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
        return switch ((int) raw) {
            case 0x00000001 -> "User";
            case 0x00000002 -> "System";
            default -> null;
        };
    }
    /** User; kmip-3.0 §11.35, Table 582. */
    public static final long V3_0_OBJECT_TYPE_USER = 0x0000000BL;
    /** Group; kmip-3.0 §11.35, Table 582. */
    public static final long V3_0_OBJECT_TYPE_GROUP = 0x0000000CL;
    /** Password Credential; kmip-3.0 §11.35, Table 582. */
    public static final long V3_0_OBJECT_TYPE_PASSWORD_CREDENTIAL = 0x0000000DL;
    /** Device Credential; kmip-3.0 §11.35, Table 582. */
    public static final long V3_0_OBJECT_TYPE_DEVICE_CREDENTIAL = 0x0000000EL;
    /** One Time Password Credential; kmip-3.0 §11.35, Table 582. */
    public static final long V3_0_OBJECT_TYPE_ONE_TIME_PASSWORD_CREDENTIAL = 0x0000000FL;
    /** Hashed Password Credential; kmip-3.0 §11.35, Table 582. */
    public static final long V3_0_OBJECT_TYPE_HASHED_PASSWORD_CREDENTIAL = 0x00000010L;
    /** Returns the KMIP v3.0 Object Type name, or null for an unassigned value. */
    public static String v3_0ObjectTypeName(long raw) {
        if (raw < 0 || raw > 0xFFFF_FFFFL) return null;
        return switch ((int) raw) {
            case 0x0000000B -> "User";
            case 0x0000000C -> "Group";
            case 0x0000000D -> "Password Credential";
            case 0x0000000E -> "Device Credential";
            case 0x0000000F -> "One Time Password Credential";
            case 0x00000010 -> "Hashed Password Credential";
            default -> null;
        };
    }
}
