//! Generated catalog-backed KMIP 2.1 named values.

/// KMIP protocol Enumeration and Bit Mask values.
pub mod values {
    /// Assigned KMIP 2.1 Enumeration values.
    pub mod enumeration {
        /// Values for the KMIP Adjustment Type Enumeration.
        pub mod adjustment_type {
            /// `Increment`; KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-INCREMENT-00000001; KMIPKIT-SRC-spec §11.1.
            pub const INCREMENT: u32 = 0x0000_0001;
            /// `Decrement`; KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-DECREMENT-00000002; KMIPKIT-SRC-spec §11.1.
            pub const DECREMENT: u32 = 0x0000_0002;
            /// `Negate`; KMIPKIT-ELEM-ENUM-VALUE-ADJUSTMENT-TYPE-NEGATE-00000003; KMIPKIT-SRC-spec §11.1.
            pub const NEGATE: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Increment"),
                    0x0000_0002 => Some("Decrement"),
                    0x0000_0003 => Some("Negate"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Alternative Name Type Enumeration.
        pub mod alternative_name_type {
            /// `Uninterpreted Text String`; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.2.
            pub const UNINTERPRETED_TEXT_STRING: u32 = 0x0000_0001;
            /// `URI`; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.2.
            pub const URI: u32 = 0x0000_0002;
            /// `Object Serial Number`; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-OBJECT-SERIAL-NUMBER-00000003; KMIPKIT-SRC-spec §11.2.
            pub const OBJECT_SERIAL_NUMBER: u32 = 0x0000_0003;
            /// `Email Address`; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-EMAIL-ADDRESS-00000004; KMIPKIT-SRC-spec §11.2.
            pub const EMAIL_ADDRESS: u32 = 0x0000_0004;
            /// `DNS Name`; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-DNS-NAME-00000005; KMIPKIT-SRC-spec §11.2.
            pub const DNS_NAME: u32 = 0x0000_0005;
            /// `X.500 Distinguished Name`; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-X-500-DISTINGUISHED-NAME-00000006; KMIPKIT-SRC-spec §11.2.
            pub const X_500_DISTINGUISHED_NAME: u32 = 0x0000_0006;
            /// `IP Address`; KMIPKIT-ELEM-ENUM-VALUE-ALTERNATIVE-NAME-TYPE-IP-ADDRESS-00000007; KMIPKIT-SRC-spec §11.2.
            pub const IP_ADDRESS: u32 = 0x0000_0007;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Uninterpreted Text String"),
                    0x0000_0002 => Some("URI"),
                    0x0000_0003 => Some("Object Serial Number"),
                    0x0000_0004 => Some("Email Address"),
                    0x0000_0005 => Some("DNS Name"),
                    0x0000_0006 => Some("X.500 Distinguished Name"),
                    0x0000_0007 => Some("IP Address"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Asynchronous Indicator Enumeration.
        pub mod asynchronous_indicator {
            /// `Mandatory`; KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-MANDATORY-00000001; KMIPKIT-SRC-spec §11.3.
            pub const MANDATORY: u32 = 0x0000_0001;
            /// `Optional`; KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-OPTIONAL-00000002; KMIPKIT-SRC-spec §11.3.
            pub const OPTIONAL: u32 = 0x0000_0002;
            /// `Prohibited`; KMIPKIT-ELEM-ENUM-VALUE-ASYNCHRONOUS-INDICATOR-PROHIBITED-00000003; KMIPKIT-SRC-spec §11.3.
            pub const PROHIBITED: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Mandatory"),
                    0x0000_0002 => Some("Optional"),
                    0x0000_0003 => Some("Prohibited"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Attestation Type Enumeration.
        pub mod attestation_type {
            /// `TPM Quote`; KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TPM-QUOTE-00000001; KMIPKIT-SRC-spec §11.4.
            pub const TPM_QUOTE: u32 = 0x0000_0001;
            /// `TCG Integrity Report`; KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-TCG-INTEGRITY-REPORT-00000002; KMIPKIT-SRC-spec §11.4.
            pub const TCG_INTEGRITY_REPORT: u32 = 0x0000_0002;
            /// `SAML Assertion`; KMIPKIT-ELEM-ENUM-VALUE-ATTESTATION-TYPE-SAML-ASSERTION-00000003; KMIPKIT-SRC-spec §11.4.
            pub const SAML_ASSERTION: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("TPM Quote"),
                    0x0000_0002 => Some("TCG Integrity Report"),
                    0x0000_0003 => Some("SAML Assertion"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Batch Error Continuation Option Enumeration.
        pub mod batch_error_continuation_option {
            /// `Continue`; KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-CONTINUE-00000001; KMIPKIT-SRC-spec §11.5.
            pub const CONTINUE: u32 = 0x0000_0001;
            /// `Stop`; KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-STOP-00000002; KMIPKIT-SRC-spec §11.5.
            pub const STOP: u32 = 0x0000_0002;
            /// `Undo`; KMIPKIT-ELEM-ENUM-VALUE-BATCH-ERROR-CONTINUATION-OPTION-UNDO-00000003; KMIPKIT-SRC-spec §11.5.
            pub const UNDO: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Continue"),
                    0x0000_0002 => Some("Stop"),
                    0x0000_0003 => Some("Undo"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Block Cipher Mode Enumeration.
        pub mod block_cipher_mode {
            /// `CBC`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-00000001; KMIPKIT-SRC-spec §11.6.
            pub const CBC: u32 = 0x0000_0001;
            /// `ECB`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-ECB-00000002; KMIPKIT-SRC-spec §11.6.
            pub const ECB: u32 = 0x0000_0002;
            /// `PCBC`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-PCBC-00000003; KMIPKIT-SRC-spec §11.6.
            pub const PCBC: u32 = 0x0000_0003;
            /// `CFB`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CFB-00000004; KMIPKIT-SRC-spec §11.6.
            pub const CFB: u32 = 0x0000_0004;
            /// `OFB`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-OFB-00000005; KMIPKIT-SRC-spec §11.6.
            pub const OFB: u32 = 0x0000_0005;
            /// `CTR`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CTR-00000006; KMIPKIT-SRC-spec §11.6.
            pub const CTR: u32 = 0x0000_0006;
            /// `CMAC`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CMAC-00000007; KMIPKIT-SRC-spec §11.6.
            pub const CMAC: u32 = 0x0000_0007;
            /// `CCM`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CCM-00000008; KMIPKIT-SRC-spec §11.6.
            pub const CCM: u32 = 0x0000_0008;
            /// `GCM`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-GCM-00000009; KMIPKIT-SRC-spec §11.6.
            pub const GCM: u32 = 0x0000_0009;
            /// `CBC-MAC`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-CBC-MAC-0000000A; KMIPKIT-SRC-spec §11.6.
            pub const CBC_MAC: u32 = 0x0000_000A;
            /// `XTS`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-XTS-0000000B; KMIPKIT-SRC-spec §11.6.
            pub const XTS: u32 = 0x0000_000B;
            /// `AESKeyWrapPadding`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AESKEYWRAPPADDING-0000000C; KMIPKIT-SRC-spec §11.6.
            pub const AESKEYWRAPPADDING: u32 = 0x0000_000C;
            /// `NISTKeyWrap`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-NISTKEYWRAP-0000000D; KMIPKIT-SRC-spec §11.6.
            pub const NISTKEYWRAP: u32 = 0x0000_000D;
            /// `X9.102 AESKW`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AESKW-0000000E; KMIPKIT-SRC-spec §11.6.
            pub const X9_102_AESKW: u32 = 0x0000_000E;
            /// `X9.102 TDKW`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-TDKW-0000000F; KMIPKIT-SRC-spec §11.6.
            pub const X9_102_TDKW: u32 = 0x0000_000F;
            /// `X9.102 AKW1`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW1-00000010; KMIPKIT-SRC-spec §11.6.
            pub const X9_102_AKW1: u32 = 0x0000_0010;
            /// `X9.102 AKW2`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-X9-102-AKW2-00000011; KMIPKIT-SRC-spec §11.6.
            pub const X9_102_AKW2: u32 = 0x0000_0011;
            /// `AEAD`; KMIPKIT-ELEM-ENUM-VALUE-BLOCK-CIPHER-MODE-AEAD-00000012; KMIPKIT-SRC-spec §11.6.
            pub const AEAD: u32 = 0x0000_0012;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("CBC"),
                    0x0000_0002 => Some("ECB"),
                    0x0000_0003 => Some("PCBC"),
                    0x0000_0004 => Some("CFB"),
                    0x0000_0005 => Some("OFB"),
                    0x0000_0006 => Some("CTR"),
                    0x0000_0007 => Some("CMAC"),
                    0x0000_0008 => Some("CCM"),
                    0x0000_0009 => Some("GCM"),
                    0x0000_000A => Some("CBC-MAC"),
                    0x0000_000B => Some("XTS"),
                    0x0000_000C => Some("AESKeyWrapPadding"),
                    0x0000_000D => Some("NISTKeyWrap"),
                    0x0000_000E => Some("X9.102 AESKW"),
                    0x0000_000F => Some("X9.102 TDKW"),
                    0x0000_0010 => Some("X9.102 AKW1"),
                    0x0000_0011 => Some("X9.102 AKW2"),
                    0x0000_0012 => Some("AEAD"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Cancellation Result Enumeration.
        pub mod cancellation_result {
            /// `Canceled`; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-CANCELED-00000001; KMIPKIT-SRC-spec §11.7.
            pub const CANCELED: u32 = 0x0000_0001;
            /// `Unable to Cancel`; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNABLE-TO-CANCEL-00000002; KMIPKIT-SRC-spec §11.7.
            pub const UNABLE_TO_CANCEL: u32 = 0x0000_0002;
            /// `Completed`; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-COMPLETED-00000003; KMIPKIT-SRC-spec §11.7.
            pub const COMPLETED: u32 = 0x0000_0003;
            /// `Failed`; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-FAILED-00000004; KMIPKIT-SRC-spec §11.7.
            pub const FAILED: u32 = 0x0000_0004;
            /// `Unavailable`; KMIPKIT-ELEM-ENUM-VALUE-CANCELLATION-RESULT-UNAVAILABLE-00000005; KMIPKIT-SRC-spec §11.7.
            pub const UNAVAILABLE: u32 = 0x0000_0005;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Canceled"),
                    0x0000_0002 => Some("Unable to Cancel"),
                    0x0000_0003 => Some("Completed"),
                    0x0000_0004 => Some("Failed"),
                    0x0000_0005 => Some("Unavailable"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Certificate Request Type Enumeration.
        pub mod certificate_request_type {
            /// `CRMF`; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-CRMF-00000001; KMIPKIT-SRC-spec §11.8.
            pub const CRMF: u32 = 0x0000_0001;
            /// `PKCS#10`; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PKCS-10-00000002; KMIPKIT-SRC-spec §11.8.
            pub const PKCS_10: u32 = 0x0000_0002;
            /// `PEM`; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-REQUEST-TYPE-PEM-00000003; KMIPKIT-SRC-spec §11.8.
            pub const PEM: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("CRMF"),
                    0x0000_0002 => Some("PKCS#10"),
                    0x0000_0003 => Some("PEM"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Certificate Type Enumeration.
        pub mod certificate_type {
            /// `X.509`; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-X-509-00000001; KMIPKIT-SRC-spec §11.9.
            pub const X_509: u32 = 0x0000_0001;
            /// `PGP`; KMIPKIT-ELEM-ENUM-VALUE-CERTIFICATE-TYPE-PGP-00000002; KMIPKIT-SRC-spec §11.9.
            pub const PGP: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("X.509"),
                    0x0000_0002 => Some("PGP"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Client Registration Method Enumeration.
        pub mod client_registration_method {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.10.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Server Pre-Generated`; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-PRE-GENERATED-00000002; KMIPKIT-SRC-spec §11.10.
            pub const SERVER_PRE_GENERATED: u32 = 0x0000_0002;
            /// `Server On-Demand`; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-SERVER-ON-DEMAND-00000003; KMIPKIT-SRC-spec §11.10.
            pub const SERVER_ON_DEMAND: u32 = 0x0000_0003;
            /// `Client Generated`; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-GENERATED-00000004; KMIPKIT-SRC-spec §11.10.
            pub const CLIENT_GENERATED: u32 = 0x0000_0004;
            /// `Client Registered`; KMIPKIT-ELEM-ENUM-VALUE-CLIENT-REGISTRATION-METHOD-CLIENT-REGISTERED-00000005; KMIPKIT-SRC-spec §11.10.
            pub const CLIENT_REGISTERED: u32 = 0x0000_0005;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Server Pre-Generated"),
                    0x0000_0003 => Some("Server On-Demand"),
                    0x0000_0004 => Some("Client Generated"),
                    0x0000_0005 => Some("Client Registered"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Credential Type Enumeration.
        pub mod credential_type {
            /// `Username and Password`; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-USERNAME-AND-PASSWORD-00000001; KMIPKIT-SRC-spec §11.11.
            pub const USERNAME_AND_PASSWORD: u32 = 0x0000_0001;
            /// `Device`; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-DEVICE-00000002; KMIPKIT-SRC-spec §11.11.
            pub const DEVICE: u32 = 0x0000_0002;
            /// `Attestation`; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ATTESTATION-00000003; KMIPKIT-SRC-spec §11.11.
            pub const ATTESTATION: u32 = 0x0000_0003;
            /// `One Time Password`; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-ONE-TIME-PASSWORD-00000004; KMIPKIT-SRC-spec §11.11.
            pub const ONE_TIME_PASSWORD: u32 = 0x0000_0004;
            /// `Hashed Password`; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-HASHED-PASSWORD-00000005; KMIPKIT-SRC-spec §11.11.
            pub const HASHED_PASSWORD: u32 = 0x0000_0005;
            /// `Ticket`; KMIPKIT-ELEM-ENUM-VALUE-CREDENTIAL-TYPE-TICKET-00000006; KMIPKIT-SRC-spec §11.11.
            pub const TICKET: u32 = 0x0000_0006;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Username and Password"),
                    0x0000_0002 => Some("Device"),
                    0x0000_0003 => Some("Attestation"),
                    0x0000_0004 => Some("One Time Password"),
                    0x0000_0005 => Some("Hashed Password"),
                    0x0000_0006 => Some("Ticket"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Cryptographic Algorithm Enumeration.
        pub mod cryptographic_algorithm {
            /// `DES`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DES-00000001; KMIPKIT-SRC-spec §11.12.
            pub const DES: u32 = 0x0000_0001;
            /// `3DES`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-3DES-00000002; KMIPKIT-SRC-spec §11.12.
            pub const VALUE_3DES: u32 = 0x0000_0002;
            /// `AES`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-AES-00000003; KMIPKIT-SRC-spec §11.12.
            pub const AES: u32 = 0x0000_0003;
            /// `RSA`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RSA-00000004; KMIPKIT-SRC-spec §11.12.
            pub const RSA: u32 = 0x0000_0004;
            /// `DSA`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DSA-00000005; KMIPKIT-SRC-spec §11.12.
            pub const DSA: u32 = 0x0000_0005;
            /// `ECDSA`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDSA-00000006; KMIPKIT-SRC-spec §11.12.
            pub const ECDSA: u32 = 0x0000_0006;
            /// `HMAC-SHA1`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA1-00000007; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA1: u32 = 0x0000_0007;
            /// `HMAC-SHA224`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA224-00000008; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA224: u32 = 0x0000_0008;
            /// `HMAC-SHA256`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA256-00000009; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA256: u32 = 0x0000_0009;
            /// `HMAC-SHA384`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA384-0000000A; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA384: u32 = 0x0000_000A;
            /// `HMAC-SHA512`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA512-0000000B; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA512: u32 = 0x0000_000B;
            /// `HMAC-MD5`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-MD5-0000000C; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_MD5: u32 = 0x0000_000C;
            /// `DH`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-DH-0000000D; KMIPKIT-SRC-spec §11.12.
            pub const DH: u32 = 0x0000_000D;
            /// `ECDH`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECDH-0000000E; KMIPKIT-SRC-spec §11.12.
            pub const ECDH: u32 = 0x0000_000E;
            /// `ECMQV`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ECMQV-0000000F; KMIPKIT-SRC-spec §11.12.
            pub const ECMQV: u32 = 0x0000_000F;
            /// `Blowfish`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-BLOWFISH-00000010; KMIPKIT-SRC-spec §11.12.
            pub const BLOWFISH: u32 = 0x0000_0010;
            /// `Camellia`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAMELLIA-00000011; KMIPKIT-SRC-spec §11.12.
            pub const CAMELLIA: u32 = 0x0000_0011;
            /// `CAST5`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CAST5-00000012; KMIPKIT-SRC-spec §11.12.
            pub const CAST5: u32 = 0x0000_0012;
            /// `IDEA`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-IDEA-00000013; KMIPKIT-SRC-spec §11.12.
            pub const IDEA: u32 = 0x0000_0013;
            /// `MARS`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MARS-00000014; KMIPKIT-SRC-spec §11.12.
            pub const MARS: u32 = 0x0000_0014;
            /// `RC2`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC2-00000015; KMIPKIT-SRC-spec §11.12.
            pub const RC2: u32 = 0x0000_0015;
            /// `RC4`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC4-00000016; KMIPKIT-SRC-spec §11.12.
            pub const RC4: u32 = 0x0000_0016;
            /// `RC5`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-RC5-00000017; KMIPKIT-SRC-spec §11.12.
            pub const RC5: u32 = 0x0000_0017;
            /// `SKIPJACK`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SKIPJACK-00000018; KMIPKIT-SRC-spec §11.12.
            pub const SKIPJACK: u32 = 0x0000_0018;
            /// `Twofish`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-TWOFISH-00000019; KMIPKIT-SRC-spec §11.12.
            pub const TWOFISH: u32 = 0x0000_0019;
            /// `EC`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-EC-0000001A; KMIPKIT-SRC-spec §11.12.
            pub const EC: u32 = 0x0000_001A;
            /// `One Time Pad`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ONE-TIME-PAD-0000001B; KMIPKIT-SRC-spec §11.12.
            pub const ONE_TIME_PAD: u32 = 0x0000_001B;
            /// `ChaCha20`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20-0000001C; KMIPKIT-SRC-spec §11.12.
            pub const CHACHA20: u32 = 0x0000_001C;
            /// `Poly1305`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-POLY1305-0000001D; KMIPKIT-SRC-spec §11.12.
            pub const POLY1305: u32 = 0x0000_001D;
            /// `ChaCha20Poly1305`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-CHACHA20POLY1305-0000001E; KMIPKIT-SRC-spec §11.12.
            pub const CHACHA20POLY1305: u32 = 0x0000_001E;
            /// `SHA3-224`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-224-0000001F; KMIPKIT-SRC-spec §11.12.
            pub const SHA3_224: u32 = 0x0000_001F;
            /// `SHA3-256`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-256-00000020; KMIPKIT-SRC-spec §11.12.
            pub const SHA3_256: u32 = 0x0000_0020;
            /// `SHA3-384`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-384-00000021; KMIPKIT-SRC-spec §11.12.
            pub const SHA3_384: u32 = 0x0000_0021;
            /// `SHA3-512`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHA3-512-00000022; KMIPKIT-SRC-spec §11.12.
            pub const SHA3_512: u32 = 0x0000_0022;
            /// `HMAC-SHA3-224`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-224-00000023; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA3_224: u32 = 0x0000_0023;
            /// `HMAC-SHA3-256`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-256-00000024; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA3_256: u32 = 0x0000_0024;
            /// `HMAC-SHA3-384`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-384-00000025; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA3_384: u32 = 0x0000_0025;
            /// `HMAC-SHA3-512`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-HMAC-SHA3-512-00000026; KMIPKIT-SRC-spec §11.12.
            pub const HMAC_SHA3_512: u32 = 0x0000_0026;
            /// `SHAKE-128`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-128-00000027; KMIPKIT-SRC-spec §11.12.
            pub const SHAKE_128: u32 = 0x0000_0027;
            /// `SHAKE-256`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SHAKE-256-00000028; KMIPKIT-SRC-spec §11.12.
            pub const SHAKE_256: u32 = 0x0000_0028;
            /// `ARIA`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ARIA-00000029; KMIPKIT-SRC-spec §11.12.
            pub const ARIA: u32 = 0x0000_0029;
            /// `SEED`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SEED-0000002A; KMIPKIT-SRC-spec §11.12.
            pub const SEED: u32 = 0x0000_002A;
            /// `SM2`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM2-0000002B; KMIPKIT-SRC-spec §11.12.
            pub const SM2: u32 = 0x0000_002B;
            /// `SM3`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM3-0000002C; KMIPKIT-SRC-spec §11.12.
            pub const SM3: u32 = 0x0000_002C;
            /// `SM4`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SM4-0000002D; KMIPKIT-SRC-spec §11.12.
            pub const SM4: u32 = 0x0000_002D;
            /// `GOST R 34.10-2012`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-10-2012-0000002E; KMIPKIT-SRC-spec §11.12.
            pub const GOST_R_34_10_2012: u32 = 0x0000_002E;
            /// `GOST R 34.11-2012`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-11-2012-0000002F; KMIPKIT-SRC-spec §11.12.
            pub const GOST_R_34_11_2012: u32 = 0x0000_002F;
            /// `GOST R 34.13-2015`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-R-34-13-2015-00000030; KMIPKIT-SRC-spec §11.12.
            pub const GOST_R_34_13_2015: u32 = 0x0000_0030;
            /// `GOST 28147-89`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-GOST-28147-89-00000031; KMIPKIT-SRC-spec §11.12.
            pub const GOST_28147_89: u32 = 0x0000_0031;
            /// `XMSS`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-XMSS-00000032; KMIPKIT-SRC-spec §11.12.
            pub const XMSS: u32 = 0x0000_0032;
            /// `SPHINCS-256`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-SPHINCS-256-00000033; KMIPKIT-SRC-spec §11.12.
            pub const SPHINCS_256: u32 = 0x0000_0033;
            /// `McEliece`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-00000034; KMIPKIT-SRC-spec §11.12.
            pub const MCELIECE: u32 = 0x0000_0034;
            /// `McEliece-6960119`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-6960119-00000035; KMIPKIT-SRC-spec §11.12.
            pub const MCELIECE_6960119: u32 = 0x0000_0035;
            /// `McEliece-8192128`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-MCELIECE-8192128-00000036; KMIPKIT-SRC-spec §11.12.
            pub const MCELIECE_8192128: u32 = 0x0000_0036;
            /// `Ed25519`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED25519-00000037; KMIPKIT-SRC-spec §11.12.
            pub const ED25519: u32 = 0x0000_0037;
            /// `Ed448`; KMIPKIT-ELEM-ENUM-VALUE-CRYPTOGRAPHIC-ALGORITHM-ED448-00000038; KMIPKIT-SRC-spec §11.12.
            pub const ED448: u32 = 0x0000_0038;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("DES"),
                    0x0000_0002 => Some("3DES"),
                    0x0000_0003 => Some("AES"),
                    0x0000_0004 => Some("RSA"),
                    0x0000_0005 => Some("DSA"),
                    0x0000_0006 => Some("ECDSA"),
                    0x0000_0007 => Some("HMAC-SHA1"),
                    0x0000_0008 => Some("HMAC-SHA224"),
                    0x0000_0009 => Some("HMAC-SHA256"),
                    0x0000_000A => Some("HMAC-SHA384"),
                    0x0000_000B => Some("HMAC-SHA512"),
                    0x0000_000C => Some("HMAC-MD5"),
                    0x0000_000D => Some("DH"),
                    0x0000_000E => Some("ECDH"),
                    0x0000_000F => Some("ECMQV"),
                    0x0000_0010 => Some("Blowfish"),
                    0x0000_0011 => Some("Camellia"),
                    0x0000_0012 => Some("CAST5"),
                    0x0000_0013 => Some("IDEA"),
                    0x0000_0014 => Some("MARS"),
                    0x0000_0015 => Some("RC2"),
                    0x0000_0016 => Some("RC4"),
                    0x0000_0017 => Some("RC5"),
                    0x0000_0018 => Some("SKIPJACK"),
                    0x0000_0019 => Some("Twofish"),
                    0x0000_001A => Some("EC"),
                    0x0000_001B => Some("One Time Pad"),
                    0x0000_001C => Some("ChaCha20"),
                    0x0000_001D => Some("Poly1305"),
                    0x0000_001E => Some("ChaCha20Poly1305"),
                    0x0000_001F => Some("SHA3-224"),
                    0x0000_0020 => Some("SHA3-256"),
                    0x0000_0021 => Some("SHA3-384"),
                    0x0000_0022 => Some("SHA3-512"),
                    0x0000_0023 => Some("HMAC-SHA3-224"),
                    0x0000_0024 => Some("HMAC-SHA3-256"),
                    0x0000_0025 => Some("HMAC-SHA3-384"),
                    0x0000_0026 => Some("HMAC-SHA3-512"),
                    0x0000_0027 => Some("SHAKE-128"),
                    0x0000_0028 => Some("SHAKE-256"),
                    0x0000_0029 => Some("ARIA"),
                    0x0000_002A => Some("SEED"),
                    0x0000_002B => Some("SM2"),
                    0x0000_002C => Some("SM3"),
                    0x0000_002D => Some("SM4"),
                    0x0000_002E => Some("GOST R 34.10-2012"),
                    0x0000_002F => Some("GOST R 34.11-2012"),
                    0x0000_0030 => Some("GOST R 34.13-2015"),
                    0x0000_0031 => Some("GOST 28147-89"),
                    0x0000_0032 => Some("XMSS"),
                    0x0000_0033 => Some("SPHINCS-256"),
                    0x0000_0034 => Some("McEliece"),
                    0x0000_0035 => Some("McEliece-6960119"),
                    0x0000_0036 => Some("McEliece-8192128"),
                    0x0000_0037 => Some("Ed25519"),
                    0x0000_0038 => Some("Ed448"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Data Enumeration.
        pub mod data {
            /// `Decrypt`; KMIPKIT-ELEM-ENUM-VALUE-DATA-DECRYPT-00000001; KMIPKIT-SRC-spec §11.13.
            pub const DECRYPT: u32 = 0x0000_0001;
            /// `Encrypt`; KMIPKIT-ELEM-ENUM-VALUE-DATA-ENCRYPT-00000002; KMIPKIT-SRC-spec §11.13.
            pub const ENCRYPT: u32 = 0x0000_0002;
            /// `Hash`; KMIPKIT-ELEM-ENUM-VALUE-DATA-HASH-00000003; KMIPKIT-SRC-spec §11.13.
            pub const HASH: u32 = 0x0000_0003;
            /// `MAC MAC Data`; KMIPKIT-ELEM-ENUM-VALUE-DATA-MAC-MAC-DATA-00000004; KMIPKIT-SRC-spec §11.13.
            pub const MAC_MAC_DATA: u32 = 0x0000_0004;
            /// `RNG Retrieve`; KMIPKIT-ELEM-ENUM-VALUE-DATA-RNG-RETRIEVE-00000005; KMIPKIT-SRC-spec §11.13.
            pub const RNG_RETRIEVE: u32 = 0x0000_0005;
            /// `Sign Signature Data`; KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGN-SIGNATURE-DATA-00000006; KMIPKIT-SRC-spec §11.13.
            pub const SIGN_SIGNATURE_DATA: u32 = 0x0000_0006;
            /// `Signature Verify`; KMIPKIT-ELEM-ENUM-VALUE-DATA-SIGNATURE-VERIFY-00000007; KMIPKIT-SRC-spec §11.13.
            pub const SIGNATURE_VERIFY: u32 = 0x0000_0007;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Decrypt"),
                    0x0000_0002 => Some("Encrypt"),
                    0x0000_0003 => Some("Hash"),
                    0x0000_0004 => Some("MAC MAC Data"),
                    0x0000_0005 => Some("RNG Retrieve"),
                    0x0000_0006 => Some("Sign Signature Data"),
                    0x0000_0007 => Some("Signature Verify"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Derivation Method Enumeration.
        pub mod derivation_method {
            /// `PBKDF2`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-PBKDF2-00000001; KMIPKIT-SRC-spec §11.14.
            pub const PBKDF2: u32 = 0x0000_0001;
            /// `HASH`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HASH-00000002; KMIPKIT-SRC-spec §11.14.
            pub const HASH: u32 = 0x0000_0002;
            /// `HMAC`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HMAC-00000003; KMIPKIT-SRC-spec §11.14.
            pub const HMAC: u32 = 0x0000_0003;
            /// `ENCRYPT`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ENCRYPT-00000004; KMIPKIT-SRC-spec §11.14.
            pub const ENCRYPT: u32 = 0x0000_0004;
            /// `NIST800-108-C`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-C-00000005; KMIPKIT-SRC-spec §11.14.
            pub const NIST800_108_C: u32 = 0x0000_0005;
            /// `NIST800-108-F`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-F-00000006; KMIPKIT-SRC-spec §11.14.
            pub const NIST800_108_F: u32 = 0x0000_0006;
            /// `NIST800-108-DPI`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-NIST800-108-DPI-00000007; KMIPKIT-SRC-spec §11.14.
            pub const NIST800_108_DPI: u32 = 0x0000_0007;
            /// `Asymmetric Key`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-ASYMMETRIC-KEY-00000008; KMIPKIT-SRC-spec §11.14.
            pub const ASYMMETRIC_KEY: u32 = 0x0000_0008;
            /// `AWS Signature Version 4`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-AWS-SIGNATURE-VERSION-4-00000009; KMIPKIT-SRC-spec §11.14.
            pub const AWS_SIGNATURE_VERSION_4: u32 = 0x0000_0009;
            /// `HKDF`; KMIPKIT-ELEM-ENUM-VALUE-DERIVATION-METHOD-HKDF-0000000A; KMIPKIT-SRC-spec §11.14.
            pub const HKDF: u32 = 0x0000_000A;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("PBKDF2"),
                    0x0000_0002 => Some("HASH"),
                    0x0000_0003 => Some("HMAC"),
                    0x0000_0004 => Some("ENCRYPT"),
                    0x0000_0005 => Some("NIST800-108-C"),
                    0x0000_0006 => Some("NIST800-108-F"),
                    0x0000_0007 => Some("NIST800-108-DPI"),
                    0x0000_0008 => Some("Asymmetric Key"),
                    0x0000_0009 => Some("AWS Signature Version 4"),
                    0x0000_000A => Some("HKDF"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Destroy Action Enumeration.
        pub mod destroy_action {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.15.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Key Material Deleted`; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-DELETED-00000002; KMIPKIT-SRC-spec §11.15.
            pub const KEY_MATERIAL_DELETED: u32 = 0x0000_0002;
            /// `Key Material Shredded`; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-KEY-MATERIAL-SHREDDED-00000003; KMIPKIT-SRC-spec §11.15.
            pub const KEY_MATERIAL_SHREDDED: u32 = 0x0000_0003;
            /// `Meta Data Deleted`; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-DELETED-00000004; KMIPKIT-SRC-spec §11.15.
            pub const META_DATA_DELETED: u32 = 0x0000_0004;
            /// `Meta Data Shredded`; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-META-DATA-SHREDDED-00000005; KMIPKIT-SRC-spec §11.15.
            pub const META_DATA_SHREDDED: u32 = 0x0000_0005;
            /// `Deleted`; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-DELETED-00000006; KMIPKIT-SRC-spec §11.15.
            pub const DELETED: u32 = 0x0000_0006;
            /// `Shredded`; KMIPKIT-ELEM-ENUM-VALUE-DESTROY-ACTION-SHREDDED-00000007; KMIPKIT-SRC-spec §11.15.
            pub const SHREDDED: u32 = 0x0000_0007;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Key Material Deleted"),
                    0x0000_0003 => Some("Key Material Shredded"),
                    0x0000_0004 => Some("Meta Data Deleted"),
                    0x0000_0005 => Some("Meta Data Shredded"),
                    0x0000_0006 => Some("Deleted"),
                    0x0000_0007 => Some("Shredded"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Digital Signature Algorithm Enumeration.
        pub mod digital_signature_algorithm {
            /// `MD2 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD2-WITH-RSA-ENCRYPTION-00000001; KMIPKIT-SRC-spec §11.16.
            pub const MD2_WITH_RSA_ENCRYPTION: u32 = 0x0000_0001;
            /// `MD5 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-MD5-WITH-RSA-ENCRYPTION-00000002; KMIPKIT-SRC-spec §11.16.
            pub const MD5_WITH_RSA_ENCRYPTION: u32 = 0x0000_0002;
            /// `SHA-1 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-1-WITH-RSA-ENCRYPTION-00000003; KMIPKIT-SRC-spec §11.16.
            pub const SHA_1_WITH_RSA_ENCRYPTION: u32 = 0x0000_0003;
            /// `SHA-224 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-224-WITH-RSA-ENCRYPTION-00000004; KMIPKIT-SRC-spec §11.16.
            pub const SHA_224_WITH_RSA_ENCRYPTION: u32 = 0x0000_0004;
            /// `SHA-256 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-256-WITH-RSA-ENCRYPTION-00000005; KMIPKIT-SRC-spec §11.16.
            pub const SHA_256_WITH_RSA_ENCRYPTION: u32 = 0x0000_0005;
            /// `SHA-384 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-384-WITH-RSA-ENCRYPTION-00000006; KMIPKIT-SRC-spec §11.16.
            pub const SHA_384_WITH_RSA_ENCRYPTION: u32 = 0x0000_0006;
            /// `SHA-512 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA-512-WITH-RSA-ENCRYPTION-00000007; KMIPKIT-SRC-spec §11.16.
            pub const SHA_512_WITH_RSA_ENCRYPTION: u32 = 0x0000_0007;
            /// `RSASSA-PSS`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-RSASSA-PSS-00000008; KMIPKIT-SRC-spec §11.16.
            pub const RSASSA_PSS: u32 = 0x0000_0008;
            /// `DSA with SHA-1`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA-1-00000009; KMIPKIT-SRC-spec §11.16.
            pub const DSA_WITH_SHA_1: u32 = 0x0000_0009;
            /// `DSA with SHA224`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA224-0000000A; KMIPKIT-SRC-spec §11.16.
            pub const DSA_WITH_SHA224: u32 = 0x0000_000A;
            /// `DSA with SHA256`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-DSA-WITH-SHA256-0000000B; KMIPKIT-SRC-spec §11.16.
            pub const DSA_WITH_SHA256: u32 = 0x0000_000B;
            /// `ECDSA with SHA-1`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA-1-0000000C; KMIPKIT-SRC-spec §11.16.
            pub const ECDSA_WITH_SHA_1: u32 = 0x0000_000C;
            /// `ECDSA with SHA224`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA224-0000000D; KMIPKIT-SRC-spec §11.16.
            pub const ECDSA_WITH_SHA224: u32 = 0x0000_000D;
            /// `ECDSA with SHA256`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA256-0000000E; KMIPKIT-SRC-spec §11.16.
            pub const ECDSA_WITH_SHA256: u32 = 0x0000_000E;
            /// `ECDSA with SHA384`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA384-0000000F; KMIPKIT-SRC-spec §11.16.
            pub const ECDSA_WITH_SHA384: u32 = 0x0000_000F;
            /// `ECDSA with SHA512`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-ECDSA-WITH-SHA512-00000010; KMIPKIT-SRC-spec §11.16.
            pub const ECDSA_WITH_SHA512: u32 = 0x0000_0010;
            /// `SHA3-256 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-256-WITH-RSA-ENCRYPTION-00000011; KMIPKIT-SRC-spec §11.16.
            pub const SHA3_256_WITH_RSA_ENCRYPTION: u32 = 0x0000_0011;
            /// `SHA3-384 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-384-WITH-RSA-ENCRYPTION-00000012; KMIPKIT-SRC-spec §11.16.
            pub const SHA3_384_WITH_RSA_ENCRYPTION: u32 = 0x0000_0012;
            /// `SHA3-512 with RSA Encryption`; KMIPKIT-ELEM-ENUM-VALUE-DIGITAL-SIGNATURE-ALGORITHM-SHA3-512-WITH-RSA-ENCRYPTION-00000013; KMIPKIT-SRC-spec §11.16.
            pub const SHA3_512_WITH_RSA_ENCRYPTION: u32 = 0x0000_0013;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("MD2 with RSA Encryption"),
                    0x0000_0002 => Some("MD5 with RSA Encryption"),
                    0x0000_0003 => Some("SHA-1 with RSA Encryption"),
                    0x0000_0004 => Some("SHA-224 with RSA Encryption"),
                    0x0000_0005 => Some("SHA-256 with RSA Encryption"),
                    0x0000_0006 => Some("SHA-384 with RSA Encryption"),
                    0x0000_0007 => Some("SHA-512 with RSA Encryption"),
                    0x0000_0008 => Some("RSASSA-PSS"),
                    0x0000_0009 => Some("DSA with SHA-1"),
                    0x0000_000A => Some("DSA with SHA224"),
                    0x0000_000B => Some("DSA with SHA256"),
                    0x0000_000C => Some("ECDSA with SHA-1"),
                    0x0000_000D => Some("ECDSA with SHA224"),
                    0x0000_000E => Some("ECDSA with SHA256"),
                    0x0000_000F => Some("ECDSA with SHA384"),
                    0x0000_0010 => Some("ECDSA with SHA512"),
                    0x0000_0011 => Some("SHA3-256 with RSA Encryption"),
                    0x0000_0012 => Some("SHA3-384 with RSA Encryption"),
                    0x0000_0013 => Some("SHA3-512 with RSA Encryption"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP DRBG Algorithm Enumeration.
        pub mod drbg_algorithm {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.17.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Dual-EC`; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-DUAL-EC-00000002; KMIPKIT-SRC-spec §11.17.
            pub const DUAL_EC: u32 = 0x0000_0002;
            /// `Hash`; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HASH-00000003; KMIPKIT-SRC-spec §11.17.
            pub const HASH: u32 = 0x0000_0003;
            /// `HMAC`; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-HMAC-00000004; KMIPKIT-SRC-spec §11.17.
            pub const HMAC: u32 = 0x0000_0004;
            /// `CTR`; KMIPKIT-ELEM-ENUM-VALUE-DRBG-ALGORITHM-CTR-00000005; KMIPKIT-SRC-spec §11.17.
            pub const CTR: u32 = 0x0000_0005;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Dual-EC"),
                    0x0000_0003 => Some("Hash"),
                    0x0000_0004 => Some("HMAC"),
                    0x0000_0005 => Some("CTR"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Encoding Option Enumeration.
        pub mod encoding_option {
            /// `No Encoding`; KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-NO-ENCODING-00000001; KMIPKIT-SRC-spec §11.18.
            pub const NO_ENCODING: u32 = 0x0000_0001;
            /// `TTLV Encoding`; KMIPKIT-ELEM-ENUM-VALUE-ENCODING-OPTION-TTLV-ENCODING-00000002; KMIPKIT-SRC-spec §11.18.
            pub const TTLV_ENCODING: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("No Encoding"),
                    0x0000_0002 => Some("TTLV Encoding"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Endpoint Role Enumeration.
        pub mod endpoint_role {
            /// `Client`; KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-CLIENT-00000001; KMIPKIT-SRC-spec §11.19.
            pub const CLIENT: u32 = 0x0000_0001;
            /// `Server`; KMIPKIT-ELEM-ENUM-VALUE-ENDPOINT-ROLE-SERVER-00000002; KMIPKIT-SRC-spec §11.19.
            pub const SERVER: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Client"),
                    0x0000_0002 => Some("Server"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP FIPS186 Variation Enumeration.
        pub mod fips186_variation {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.20.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `GP x-Original`; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-ORIGINAL-00000002; KMIPKIT-SRC-spec §11.20.
            pub const GP_X_ORIGINAL: u32 = 0x0000_0002;
            /// `GP x-Change Notice`; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-GP-X-CHANGE-NOTICE-00000003; KMIPKIT-SRC-spec §11.20.
            pub const GP_X_CHANGE_NOTICE: u32 = 0x0000_0003;
            /// `x-Original`; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-ORIGINAL-00000004; KMIPKIT-SRC-spec §11.20.
            pub const X_ORIGINAL: u32 = 0x0000_0004;
            /// `x-Change Notice`; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-X-CHANGE-NOTICE-00000005; KMIPKIT-SRC-spec §11.20.
            pub const X_CHANGE_NOTICE: u32 = 0x0000_0005;
            /// `k-Original`; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-ORIGINAL-00000006; KMIPKIT-SRC-spec §11.20.
            pub const K_ORIGINAL: u32 = 0x0000_0006;
            /// `k-Change Notice`; KMIPKIT-ELEM-ENUM-VALUE-FIPS186-VARIATION-K-CHANGE-NOTICE-00000007; KMIPKIT-SRC-spec §11.20.
            pub const K_CHANGE_NOTICE: u32 = 0x0000_0007;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("GP x-Original"),
                    0x0000_0003 => Some("GP x-Change Notice"),
                    0x0000_0004 => Some("x-Original"),
                    0x0000_0005 => Some("x-Change Notice"),
                    0x0000_0006 => Some("k-Original"),
                    0x0000_0007 => Some("k-Change Notice"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Hashing Algorithm Enumeration.
        pub mod hashing_algorithm {
            /// `MD2`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD2-00000001; KMIPKIT-SRC-spec §11.21.
            pub const MD2: u32 = 0x0000_0001;
            /// `MD4`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD4-00000002; KMIPKIT-SRC-spec §11.21.
            pub const MD4: u32 = 0x0000_0002;
            /// `MD5`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-MD5-00000003; KMIPKIT-SRC-spec §11.21.
            pub const MD5: u32 = 0x0000_0003;
            /// `SHA-1`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-1-00000004; KMIPKIT-SRC-spec §11.21.
            pub const SHA_1: u32 = 0x0000_0004;
            /// `SHA-224`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-224-00000005; KMIPKIT-SRC-spec §11.21.
            pub const SHA_224: u32 = 0x0000_0005;
            /// `SHA-256`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-256-00000006; KMIPKIT-SRC-spec §11.21.
            pub const SHA_256: u32 = 0x0000_0006;
            /// `SHA-384`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-384-00000007; KMIPKIT-SRC-spec §11.21.
            pub const SHA_384: u32 = 0x0000_0007;
            /// `SHA-512`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-00000008; KMIPKIT-SRC-spec §11.21.
            pub const SHA_512: u32 = 0x0000_0008;
            /// `RIPEMD-160`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-RIPEMD-160-00000009; KMIPKIT-SRC-spec §11.21.
            pub const RIPEMD_160: u32 = 0x0000_0009;
            /// `Tiger`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-TIGER-0000000A; KMIPKIT-SRC-spec §11.21.
            pub const TIGER: u32 = 0x0000_000A;
            /// `Whirlpool`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-WHIRLPOOL-0000000B; KMIPKIT-SRC-spec §11.21.
            pub const WHIRLPOOL: u32 = 0x0000_000B;
            /// `SHA-512/224`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-224-0000000C; KMIPKIT-SRC-spec §11.21.
            pub const SHA_512_224: u32 = 0x0000_000C;
            /// `SHA-512/256`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA-512-256-0000000D; KMIPKIT-SRC-spec §11.21.
            pub const SHA_512_256: u32 = 0x0000_000D;
            /// `SHA3-224`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-224-0000000E; KMIPKIT-SRC-spec §11.21.
            pub const SHA3_224: u32 = 0x0000_000E;
            /// `SHA3-256`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-256-0000000F; KMIPKIT-SRC-spec §11.21.
            pub const SHA3_256: u32 = 0x0000_000F;
            /// `SHA3-384`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-384-00000010; KMIPKIT-SRC-spec §11.21.
            pub const SHA3_384: u32 = 0x0000_0010;
            /// `SHA3-512`; KMIPKIT-ELEM-ENUM-VALUE-HASHING-ALGORITHM-SHA3-512-00000011; KMIPKIT-SRC-spec §11.21.
            pub const SHA3_512: u32 = 0x0000_0011;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("MD2"),
                    0x0000_0002 => Some("MD4"),
                    0x0000_0003 => Some("MD5"),
                    0x0000_0004 => Some("SHA-1"),
                    0x0000_0005 => Some("SHA-224"),
                    0x0000_0006 => Some("SHA-256"),
                    0x0000_0007 => Some("SHA-384"),
                    0x0000_0008 => Some("SHA-512"),
                    0x0000_0009 => Some("RIPEMD-160"),
                    0x0000_000A => Some("Tiger"),
                    0x0000_000B => Some("Whirlpool"),
                    0x0000_000C => Some("SHA-512/224"),
                    0x0000_000D => Some("SHA-512/256"),
                    0x0000_000E => Some("SHA3-224"),
                    0x0000_000F => Some("SHA3-256"),
                    0x0000_0010 => Some("SHA3-384"),
                    0x0000_0011 => Some("SHA3-512"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Interop Function Enumeration.
        pub mod interop_function {
            /// `Begin`; KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-BEGIN-00000001; KMIPKIT-SRC-spec §11.22.
            pub const BEGIN: u32 = 0x0000_0001;
            /// `End`; KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-END-00000002; KMIPKIT-SRC-spec §11.22.
            pub const END: u32 = 0x0000_0002;
            /// `Reset`; KMIPKIT-ELEM-ENUM-VALUE-INTEROP-FUNCTION-RESET-00000003; KMIPKIT-SRC-spec §11.22.
            pub const RESET: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Begin"),
                    0x0000_0002 => Some("End"),
                    0x0000_0003 => Some("Reset"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Item Type Enumeration.
        pub mod item_type {
            /// `Structure`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-STRUCTURE-00000001; KMIPKIT-SRC-spec §11.23.
            pub const STRUCTURE: u32 = 0x0000_0001;
            /// `Integer`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTEGER-00000002; KMIPKIT-SRC-spec §11.23.
            pub const INTEGER: u32 = 0x0000_0002;
            /// `Long Integer`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-LONG-INTEGER-00000003; KMIPKIT-SRC-spec §11.23.
            pub const LONG_INTEGER: u32 = 0x0000_0003;
            /// `Big Integer`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BIG-INTEGER-00000004; KMIPKIT-SRC-spec §11.23.
            pub const BIG_INTEGER: u32 = 0x0000_0004;
            /// `Enumeration`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-ENUMERATION-00000005; KMIPKIT-SRC-spec §11.23.
            pub const ENUMERATION: u32 = 0x0000_0005;
            /// `Boolean`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BOOLEAN-00000006; KMIPKIT-SRC-spec §11.23.
            pub const BOOLEAN: u32 = 0x0000_0006;
            /// `Text String`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-TEXT-STRING-00000007; KMIPKIT-SRC-spec §11.23.
            pub const TEXT_STRING: u32 = 0x0000_0007;
            /// `Byte String`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-BYTE-STRING-00000008; KMIPKIT-SRC-spec §11.23.
            pub const BYTE_STRING: u32 = 0x0000_0008;
            /// `Date Time`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-00000009; KMIPKIT-SRC-spec §11.23.
            pub const DATE_TIME: u32 = 0x0000_0009;
            /// `Interval`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-INTERVAL-0000000A; KMIPKIT-SRC-spec §11.23.
            pub const INTERVAL: u32 = 0x0000_000A;
            /// `Date Time Extended`; KMIPKIT-ELEM-ENUM-VALUE-ITEM-TYPE-DATE-TIME-EXTENDED-0000000B; KMIPKIT-SRC-spec §11.23.
            pub const DATE_TIME_EXTENDED: u32 = 0x0000_000B;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Structure"),
                    0x0000_0002 => Some("Integer"),
                    0x0000_0003 => Some("Long Integer"),
                    0x0000_0004 => Some("Big Integer"),
                    0x0000_0005 => Some("Enumeration"),
                    0x0000_0006 => Some("Boolean"),
                    0x0000_0007 => Some("Text String"),
                    0x0000_0008 => Some("Byte String"),
                    0x0000_0009 => Some("Date Time"),
                    0x0000_000A => Some("Interval"),
                    0x0000_000B => Some("Date Time Extended"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Key Compression Type Enumeration.
        pub mod key_compression_type {
            /// `EC Public Key Type Uncompressed`; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-UNCOMPRESSED-00000001; KMIPKIT-SRC-spec §11.24.
            pub const EC_PUBLIC_KEY_TYPE_UNCOMPRESSED: u32 = 0x0000_0001;
            /// `EC Public Key Type X9.62 Compressed Prime`; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-PRIME-00000002; KMIPKIT-SRC-spec §11.24.
            pub const EC_PUBLIC_KEY_TYPE_X9_62_COMPRESSED_PRIME: u32 = 0x0000_0002;
            /// `EC Public Key Type X9.62 Compressed Char2`; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-COMPRESSED-CHAR2-00000003; KMIPKIT-SRC-spec §11.24.
            pub const EC_PUBLIC_KEY_TYPE_X9_62_COMPRESSED_CHAR2: u32 = 0x0000_0003;
            /// `EC Public Key Type X9.62 Hybrid`; KMIPKIT-ELEM-ENUM-VALUE-KEY-COMPRESSION-TYPE-EC-PUBLIC-KEY-TYPE-X9-62-HYBRID-00000004; KMIPKIT-SRC-spec §11.24.
            pub const EC_PUBLIC_KEY_TYPE_X9_62_HYBRID: u32 = 0x0000_0004;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("EC Public Key Type Uncompressed"),
                    0x0000_0002 => Some("EC Public Key Type X9.62 Compressed Prime"),
                    0x0000_0003 => Some("EC Public Key Type X9.62 Compressed Char2"),
                    0x0000_0004 => Some("EC Public Key Type X9.62 Hybrid"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Key Format Type Enumeration.
        pub mod key_format_type {
            /// `Raw`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-RAW-00000001; KMIPKIT-SRC-spec §11.25.
            pub const RAW: u32 = 0x0000_0001;
            /// `Opaque`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-OPAQUE-00000002; KMIPKIT-SRC-spec §11.25.
            pub const OPAQUE: u32 = 0x0000_0002;
            /// `PKCS#1`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-1-00000003; KMIPKIT-SRC-spec §11.25.
            pub const PKCS_1: u32 = 0x0000_0003;
            /// `PKCS#8`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-8-00000004; KMIPKIT-SRC-spec §11.25.
            pub const PKCS_8: u32 = 0x0000_0004;
            /// `X.509`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-X-509-00000005; KMIPKIT-SRC-spec §11.25.
            pub const X_509: u32 = 0x0000_0005;
            /// `ECPrivateKey`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-ECPRIVATEKEY-00000006; KMIPKIT-SRC-spec §11.25.
            pub const ECPRIVATEKEY: u32 = 0x0000_0006;
            /// `Transparent Symmetric Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-SYMMETRIC-KEY-00000007; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_SYMMETRIC_KEY: u32 = 0x0000_0007;
            /// `Transparent DSA Private Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PRIVATE-KEY-00000008; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_DSA_PRIVATE_KEY: u32 = 0x0000_0008;
            /// `Transparent DSA Public Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DSA-PUBLIC-KEY-00000009; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_DSA_PUBLIC_KEY: u32 = 0x0000_0009;
            /// `Transparent RSA Private Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PRIVATE-KEY-0000000A; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_RSA_PRIVATE_KEY: u32 = 0x0000_000A;
            /// `Transparent RSA Public Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-RSA-PUBLIC-KEY-0000000B; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_RSA_PUBLIC_KEY: u32 = 0x0000_000B;
            /// `Transparent DH Private Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PRIVATE-KEY-0000000C; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_DH_PRIVATE_KEY: u32 = 0x0000_000C;
            /// `Transparent DH Public Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-DH-PUBLIC-KEY-0000000D; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_DH_PUBLIC_KEY: u32 = 0x0000_000D;
            /// `Transparent EC Private Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PRIVATE-KEY-00000014; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_EC_PRIVATE_KEY: u32 = 0x0000_0014;
            /// `Transparent EC Public Key`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-TRANSPARENT-EC-PUBLIC-KEY-00000015; KMIPKIT-SRC-spec §11.25.
            pub const TRANSPARENT_EC_PUBLIC_KEY: u32 = 0x0000_0015;
            /// `PKCS#12`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-12-00000016; KMIPKIT-SRC-spec §11.25.
            pub const PKCS_12: u32 = 0x0000_0016;
            /// `PKCS#10`; KMIPKIT-ELEM-ENUM-VALUE-KEY-FORMAT-TYPE-PKCS-10-00000017; KMIPKIT-SRC-spec §11.25.
            pub const PKCS_10: u32 = 0x0000_0017;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Raw"),
                    0x0000_0002 => Some("Opaque"),
                    0x0000_0003 => Some("PKCS#1"),
                    0x0000_0004 => Some("PKCS#8"),
                    0x0000_0005 => Some("X.509"),
                    0x0000_0006 => Some("ECPrivateKey"),
                    0x0000_0007 => Some("Transparent Symmetric Key"),
                    0x0000_0008 => Some("Transparent DSA Private Key"),
                    0x0000_0009 => Some("Transparent DSA Public Key"),
                    0x0000_000A => Some("Transparent RSA Private Key"),
                    0x0000_000B => Some("Transparent RSA Public Key"),
                    0x0000_000C => Some("Transparent DH Private Key"),
                    0x0000_000D => Some("Transparent DH Public Key"),
                    0x0000_0014 => Some("Transparent EC Private Key"),
                    0x0000_0015 => Some("Transparent EC Public Key"),
                    0x0000_0016 => Some("PKCS#12"),
                    0x0000_0017 => Some("PKCS#10"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Key Role Type Enumeration.
        pub mod key_role_type {
            /// `BDK`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-BDK-00000001; KMIPKIT-SRC-spec §11.26.
            pub const BDK: u32 = 0x0000_0001;
            /// `CVK`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-CVK-00000002; KMIPKIT-SRC-spec §11.26.
            pub const CVK: u32 = 0x0000_0002;
            /// `DEK`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DEK-00000003; KMIPKIT-SRC-spec §11.26.
            pub const DEK: u32 = 0x0000_0003;
            /// `MKAC`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKAC-00000004; KMIPKIT-SRC-spec §11.26.
            pub const MKAC: u32 = 0x0000_0004;
            /// `MKSMC`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMC-00000005; KMIPKIT-SRC-spec §11.26.
            pub const MKSMC: u32 = 0x0000_0005;
            /// `MKSMI`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKSMI-00000006; KMIPKIT-SRC-spec §11.26.
            pub const MKSMI: u32 = 0x0000_0006;
            /// `MKDAC`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDAC-00000007; KMIPKIT-SRC-spec §11.26.
            pub const MKDAC: u32 = 0x0000_0007;
            /// `MKDN`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKDN-00000008; KMIPKIT-SRC-spec §11.26.
            pub const MKDN: u32 = 0x0000_0008;
            /// `MKCP`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKCP-00000009; KMIPKIT-SRC-spec §11.26.
            pub const MKCP: u32 = 0x0000_0009;
            /// `MKOTH`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MKOTH-0000000A; KMIPKIT-SRC-spec §11.26.
            pub const MKOTH: u32 = 0x0000_000A;
            /// `KEK`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-KEK-0000000B; KMIPKIT-SRC-spec §11.26.
            pub const KEK: u32 = 0x0000_000B;
            /// `MAC16609`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC16609-0000000C; KMIPKIT-SRC-spec §11.26.
            pub const MAC16609: u32 = 0x0000_000C;
            /// `MAC97971`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97971-0000000D; KMIPKIT-SRC-spec §11.26.
            pub const MAC97971: u32 = 0x0000_000D;
            /// `MAC97972`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97972-0000000E; KMIPKIT-SRC-spec §11.26.
            pub const MAC97972: u32 = 0x0000_000E;
            /// `MAC97973`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97973-0000000F; KMIPKIT-SRC-spec §11.26.
            pub const MAC97973: u32 = 0x0000_000F;
            /// `MAC97974`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97974-00000010; KMIPKIT-SRC-spec §11.26.
            pub const MAC97974: u32 = 0x0000_0010;
            /// `MAC97975`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-MAC97975-00000011; KMIPKIT-SRC-spec §11.26.
            pub const MAC97975: u32 = 0x0000_0011;
            /// `ZPK`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-ZPK-00000012; KMIPKIT-SRC-spec §11.26.
            pub const ZPK: u32 = 0x0000_0012;
            /// `PVKIBM`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKIBM-00000013; KMIPKIT-SRC-spec §11.26.
            pub const PVKIBM: u32 = 0x0000_0013;
            /// `PVKPVV`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKPVV-00000014; KMIPKIT-SRC-spec §11.26.
            pub const PVKPVV: u32 = 0x0000_0014;
            /// `PVKOTH`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-PVKOTH-00000015; KMIPKIT-SRC-spec §11.26.
            pub const PVKOTH: u32 = 0x0000_0015;
            /// `DUKPT`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-DUKPT-00000016; KMIPKIT-SRC-spec §11.26.
            pub const DUKPT: u32 = 0x0000_0016;
            /// `IV`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-IV-00000017; KMIPKIT-SRC-spec §11.26.
            pub const IV: u32 = 0x0000_0017;
            /// `TRKBK`; KMIPKIT-ELEM-ENUM-VALUE-KEY-ROLE-TYPE-TRKBK-00000018; KMIPKIT-SRC-spec §11.26.
            pub const TRKBK: u32 = 0x0000_0018;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("BDK"),
                    0x0000_0002 => Some("CVK"),
                    0x0000_0003 => Some("DEK"),
                    0x0000_0004 => Some("MKAC"),
                    0x0000_0005 => Some("MKSMC"),
                    0x0000_0006 => Some("MKSMI"),
                    0x0000_0007 => Some("MKDAC"),
                    0x0000_0008 => Some("MKDN"),
                    0x0000_0009 => Some("MKCP"),
                    0x0000_000A => Some("MKOTH"),
                    0x0000_000B => Some("KEK"),
                    0x0000_000C => Some("MAC16609"),
                    0x0000_000D => Some("MAC97971"),
                    0x0000_000E => Some("MAC97972"),
                    0x0000_000F => Some("MAC97973"),
                    0x0000_0010 => Some("MAC97974"),
                    0x0000_0011 => Some("MAC97975"),
                    0x0000_0012 => Some("ZPK"),
                    0x0000_0013 => Some("PVKIBM"),
                    0x0000_0014 => Some("PVKPVV"),
                    0x0000_0015 => Some("PVKOTH"),
                    0x0000_0016 => Some("DUKPT"),
                    0x0000_0017 => Some("IV"),
                    0x0000_0018 => Some("TRKBK"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Key Value Location Type Enumeration.
        pub mod key_value_location_type {
            /// `Uninterpreted Text String`; KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.27.
            pub const UNINTERPRETED_TEXT_STRING: u32 = 0x0000_0001;
            /// `URI`; KMIPKIT-ELEM-ENUM-VALUE-KEY-VALUE-LOCATION-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.27.
            pub const URI: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Uninterpreted Text String"),
                    0x0000_0002 => Some("URI"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Key Wrap Type Enumeration.
        pub mod key_wrap_type {
            /// `Not Wrapped`; KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-NOT-WRAPPED-00000001; KMIPKIT-SRC-spec §11.29.
            pub const NOT_WRAPPED: u32 = 0x0000_0001;
            /// `As Registered`; KMIPKIT-ELEM-ENUM-VALUE-KEY-WRAP-TYPE-AS-REGISTERED-00000002; KMIPKIT-SRC-spec §11.29.
            pub const AS_REGISTERED: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Not Wrapped"),
                    0x0000_0002 => Some("As Registered"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Link Type Enumeration.
        pub mod link_type {
            /// `Certificate Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CERTIFICATE-LINK-00000101; KMIPKIT-SRC-spec §11.28.
            pub const CERTIFICATE_LINK: u32 = 0x0000_0101;
            /// `Public Key Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PUBLIC-KEY-LINK-00000102; KMIPKIT-SRC-spec §11.28.
            pub const PUBLIC_KEY_LINK: u32 = 0x0000_0102;
            /// `Private Key Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PRIVATE-KEY-LINK-00000103; KMIPKIT-SRC-spec §11.28.
            pub const PRIVATE_KEY_LINK: u32 = 0x0000_0103;
            /// `Derivation Base Object Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVATION-BASE-OBJECT-LINK-00000104; KMIPKIT-SRC-spec §11.28.
            pub const DERIVATION_BASE_OBJECT_LINK: u32 = 0x0000_0104;
            /// `Derived Key Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-DERIVED-KEY-LINK-00000105; KMIPKIT-SRC-spec §11.28.
            pub const DERIVED_KEY_LINK: u32 = 0x0000_0105;
            /// `Replacement Object Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACEMENT-OBJECT-LINK-00000106; KMIPKIT-SRC-spec §11.28.
            pub const REPLACEMENT_OBJECT_LINK: u32 = 0x0000_0106;
            /// `Replaced Object Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-REPLACED-OBJECT-LINK-00000107; KMIPKIT-SRC-spec §11.28.
            pub const REPLACED_OBJECT_LINK: u32 = 0x0000_0107;
            /// `Parent Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PARENT-LINK-00000108; KMIPKIT-SRC-spec §11.28.
            pub const PARENT_LINK: u32 = 0x0000_0108;
            /// `Child Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-CHILD-LINK-00000109; KMIPKIT-SRC-spec §11.28.
            pub const CHILD_LINK: u32 = 0x0000_0109;
            /// `Previous Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PREVIOUS-LINK-0000010A; KMIPKIT-SRC-spec §11.28.
            pub const PREVIOUS_LINK: u32 = 0x0000_010A;
            /// `Next Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-NEXT-LINK-0000010B; KMIPKIT-SRC-spec §11.28.
            pub const NEXT_LINK: u32 = 0x0000_010B;
            /// `PKCS#12 Certificate Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-CERTIFICATE-LINK-0000010C; KMIPKIT-SRC-spec §11.28.
            pub const PKCS_12_CERTIFICATE_LINK: u32 = 0x0000_010C;
            /// `PKCS#12 Password Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-PKCS-12-PASSWORD-LINK-0000010D; KMIPKIT-SRC-spec §11.28.
            pub const PKCS_12_PASSWORD_LINK: u32 = 0x0000_010D;
            /// `Wrapping Key Link`; KMIPKIT-ELEM-ENUM-VALUE-LINK-TYPE-WRAPPING-KEY-LINK-0000010E; KMIPKIT-SRC-spec §11.28.
            pub const WRAPPING_KEY_LINK: u32 = 0x0000_010E;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0101 => Some("Certificate Link"),
                    0x0000_0102 => Some("Public Key Link"),
                    0x0000_0103 => Some("Private Key Link"),
                    0x0000_0104 => Some("Derivation Base Object Link"),
                    0x0000_0105 => Some("Derived Key Link"),
                    0x0000_0106 => Some("Replacement Object Link"),
                    0x0000_0107 => Some("Replaced Object Link"),
                    0x0000_0108 => Some("Parent Link"),
                    0x0000_0109 => Some("Child Link"),
                    0x0000_010A => Some("Previous Link"),
                    0x0000_010B => Some("Next Link"),
                    0x0000_010C => Some("PKCS#12 Certificate Link"),
                    0x0000_010D => Some("PKCS#12 Password Link"),
                    0x0000_010E => Some("Wrapping Key Link"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Mask Generator Enumeration.
        pub mod mask_generator {
            /// `MFG1`; KMIPKIT-ELEM-ENUM-VALUE-MASK-GENERATOR-MFG1-00000001; KMIPKIT-SRC-spec §11.30.
            pub const MFG1: u32 = 0x0000_0001;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("MFG1"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Name Type Enumeration.
        pub mod name_type {
            /// `Uninterpreted Text String`; KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.31.
            pub const UNINTERPRETED_TEXT_STRING: u32 = 0x0000_0001;
            /// `URI`; KMIPKIT-ELEM-ENUM-VALUE-NAME-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.31.
            pub const URI: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Uninterpreted Text String"),
                    0x0000_0002 => Some("URI"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP NIST Key Type Enumeration.
        pub mod nist_key_type {
            /// `Private signature key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-SIGNATURE-KEY-00000001; KMIPKIT-SRC-spec §11.32.
            pub const PRIVATE_SIGNATURE_KEY: u32 = 0x0000_0001;
            /// `Public signature verification key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-SIGNATURE-VERIFICATION-KEY-00000002; KMIPKIT-SRC-spec §11.32.
            pub const PUBLIC_SIGNATURE_VERIFICATION_KEY: u32 = 0x0000_0002;
            /// `Symmetric authentication key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHENTICATION-KEY-00000003; KMIPKIT-SRC-spec §11.32.
            pub const SYMMETRIC_AUTHENTICATION_KEY: u32 = 0x0000_0003;
            /// `Private authentication key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHENTICATION-KEY-00000004; KMIPKIT-SRC-spec §11.32.
            pub const PRIVATE_AUTHENTICATION_KEY: u32 = 0x0000_0004;
            /// `Public authentication key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHENTICATION-KEY-00000005; KMIPKIT-SRC-spec §11.32.
            pub const PUBLIC_AUTHENTICATION_KEY: u32 = 0x0000_0005;
            /// `Symmetric data encryption key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-DATA-ENCRYPTION-KEY-00000006; KMIPKIT-SRC-spec §11.32.
            pub const SYMMETRIC_DATA_ENCRYPTION_KEY: u32 = 0x0000_0006;
            /// `Symmetric key wrapping key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-WRAPPING-KEY-00000007; KMIPKIT-SRC-spec §11.32.
            pub const SYMMETRIC_KEY_WRAPPING_KEY: u32 = 0x0000_0007;
            /// `Symmetric random number generation key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-RANDOM-NUMBER-GENERATION-KEY-00000008; KMIPKIT-SRC-spec §11.32.
            pub const SYMMETRIC_RANDOM_NUMBER_GENERATION_KEY: u32 = 0x0000_0008;
            /// `Symmetric master key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-MASTER-KEY-00000009; KMIPKIT-SRC-spec §11.32.
            pub const SYMMETRIC_MASTER_KEY: u32 = 0x0000_0009;
            /// `Private key transport key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-KEY-TRANSPORT-KEY-0000000A; KMIPKIT-SRC-spec §11.32.
            pub const PRIVATE_KEY_TRANSPORT_KEY: u32 = 0x0000_000A;
            /// `Public key transport key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-KEY-TRANSPORT-KEY-0000000B; KMIPKIT-SRC-spec §11.32.
            pub const PUBLIC_KEY_TRANSPORT_KEY: u32 = 0x0000_000B;
            /// `Symmetric key agreement key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-KEY-AGREEMENT-KEY-0000000C; KMIPKIT-SRC-spec §11.32.
            pub const SYMMETRIC_KEY_AGREEMENT_KEY: u32 = 0x0000_000C;
            /// `Private static key agreement key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-STATIC-KEY-AGREEMENT-KEY-0000000D; KMIPKIT-SRC-spec §11.32.
            pub const PRIVATE_STATIC_KEY_AGREEMENT_KEY: u32 = 0x0000_000D;
            /// `Public static key agreement key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-STATIC-KEY-AGREEMENT-KEY-0000000E; KMIPKIT-SRC-spec §11.32.
            pub const PUBLIC_STATIC_KEY_AGREEMENT_KEY: u32 = 0x0000_000E;
            /// `Private ephemeral key agreement key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-EPHEMERAL-KEY-AGREEMENT-KEY-0000000F; KMIPKIT-SRC-spec §11.32.
            pub const PRIVATE_EPHEMERAL_KEY_AGREEMENT_KEY: u32 = 0x0000_000F;
            /// `Public ephemeral key agreement key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-EPHEMERAL-KEY-AGREEMENT-KEY-00000010; KMIPKIT-SRC-spec §11.32.
            pub const PUBLIC_EPHEMERAL_KEY_AGREEMENT_KEY: u32 = 0x0000_0010;
            /// `Symmetric authorization key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-SYMMETRIC-AUTHORIZATION-KEY-00000011; KMIPKIT-SRC-spec §11.32.
            pub const SYMMETRIC_AUTHORIZATION_KEY: u32 = 0x0000_0011;
            /// `Private authorization key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PRIVATE-AUTHORIZATION-KEY-00000012; KMIPKIT-SRC-spec §11.32.
            pub const PRIVATE_AUTHORIZATION_KEY: u32 = 0x0000_0012;
            /// `Public authorization key`; KMIPKIT-ELEM-ENUM-VALUE-NIST-KEY-TYPE-PUBLIC-AUTHORIZATION-KEY-00000013; KMIPKIT-SRC-spec §11.32.
            pub const PUBLIC_AUTHORIZATION_KEY: u32 = 0x0000_0013;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Private signature key"),
                    0x0000_0002 => Some("Public signature verification key"),
                    0x0000_0003 => Some("Symmetric authentication key"),
                    0x0000_0004 => Some("Private authentication key"),
                    0x0000_0005 => Some("Public authentication key"),
                    0x0000_0006 => Some("Symmetric data encryption key"),
                    0x0000_0007 => Some("Symmetric key wrapping key"),
                    0x0000_0008 => Some("Symmetric random number generation key"),
                    0x0000_0009 => Some("Symmetric master key"),
                    0x0000_000A => Some("Private key transport key"),
                    0x0000_000B => Some("Public key transport key"),
                    0x0000_000C => Some("Symmetric key agreement key"),
                    0x0000_000D => Some("Private static key agreement key"),
                    0x0000_000E => Some("Public static key agreement key"),
                    0x0000_000F => Some("Private ephemeral key agreement key"),
                    0x0000_0010 => Some("Public ephemeral key agreement key"),
                    0x0000_0011 => Some("Symmetric authorization key"),
                    0x0000_0012 => Some("Private authorization key"),
                    0x0000_0013 => Some("Public authorization key"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Object Group Member Enumeration.
        pub mod object_group_member {
            /// `Group Member Fresh`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-FRESH-00000001; KMIPKIT-SRC-spec §11.33.
            pub const GROUP_MEMBER_FRESH: u32 = 0x0000_0001;
            /// `Group Member Default`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-GROUP-MEMBER-GROUP-MEMBER-DEFAULT-00000002; KMIPKIT-SRC-spec §11.33.
            pub const GROUP_MEMBER_DEFAULT: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Group Member Fresh"),
                    0x0000_0002 => Some("Group Member Default"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Object Type Enumeration.
        pub mod object_type {
            /// `Certificate`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-00000001; KMIPKIT-SRC-spec §11.34.
            pub const CERTIFICATE: u32 = 0x0000_0001;
            /// `Symmetric Key`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SYMMETRIC-KEY-00000002; KMIPKIT-SRC-spec §11.34.
            pub const SYMMETRIC_KEY: u32 = 0x0000_0002;
            /// `Public Key`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PUBLIC-KEY-00000003; KMIPKIT-SRC-spec §11.34.
            pub const PUBLIC_KEY: u32 = 0x0000_0003;
            /// `Private Key`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PRIVATE-KEY-00000004; KMIPKIT-SRC-spec §11.34.
            pub const PRIVATE_KEY: u32 = 0x0000_0004;
            /// `Split Key`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SPLIT-KEY-00000005; KMIPKIT-SRC-spec §11.34.
            pub const SPLIT_KEY: u32 = 0x0000_0005;
            /// `Secret Data`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-SECRET-DATA-00000007; KMIPKIT-SRC-spec §11.34.
            pub const SECRET_DATA: u32 = 0x0000_0007;
            /// `Opaque Object`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-OPAQUE-OBJECT-00000008; KMIPKIT-SRC-spec §11.34.
            pub const OPAQUE_OBJECT: u32 = 0x0000_0008;
            /// `PGP Key`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-PGP-KEY-00000009; KMIPKIT-SRC-spec §11.34.
            pub const PGP_KEY: u32 = 0x0000_0009;
            /// `Certificate Request`; KMIPKIT-ELEM-ENUM-VALUE-OBJECT-TYPE-CERTIFICATE-REQUEST-0000000A; KMIPKIT-SRC-spec §11.34.
            pub const CERTIFICATE_REQUEST: u32 = 0x0000_000A;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Certificate"),
                    0x0000_0002 => Some("Symmetric Key"),
                    0x0000_0003 => Some("Public Key"),
                    0x0000_0004 => Some("Private Key"),
                    0x0000_0005 => Some("Split Key"),
                    0x0000_0007 => Some("Secret Data"),
                    0x0000_0008 => Some("Opaque Object"),
                    0x0000_0009 => Some("PGP Key"),
                    0x0000_000A => Some("Certificate Request"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Opaque Data Type Enumeration.
        pub mod opaque_data_type {
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(_raw: u32) -> Option<&'static str> {
                None
            }
        }

        /// Values for the KMIP Operation Enumeration.
        pub mod operation {
            /// `Create`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-00000001; KMIPKIT-SRC-spec §11.36.
            pub const CREATE: u32 = 0x0000_0001;
            /// `Create Key Pair`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-KEY-PAIR-00000002; KMIPKIT-SRC-spec §11.36.
            pub const CREATE_KEY_PAIR: u32 = 0x0000_0002;
            /// `Register`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REGISTER-00000003; KMIPKIT-SRC-spec §11.36.
            pub const REGISTER: u32 = 0x0000_0003;
            /// `Re-key`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-00000004; KMIPKIT-SRC-spec §11.36.
            pub const RE_KEY: u32 = 0x0000_0004;
            /// `Derive Key`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DERIVE-KEY-00000005; KMIPKIT-SRC-spec §11.36.
            pub const DERIVE_KEY: u32 = 0x0000_0005;
            /// `Certify`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CERTIFY-00000006; KMIPKIT-SRC-spec §11.36.
            pub const CERTIFY: u32 = 0x0000_0006;
            /// `Re-certify`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-CERTIFY-00000007; KMIPKIT-SRC-spec §11.36.
            pub const RE_CERTIFY: u32 = 0x0000_0007;
            /// `Locate`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOCATE-00000008; KMIPKIT-SRC-spec §11.36.
            pub const LOCATE: u32 = 0x0000_0008;
            /// `Check`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CHECK-00000009; KMIPKIT-SRC-spec §11.36.
            pub const CHECK: u32 = 0x0000_0009;
            /// `Get`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-0000000A; KMIPKIT-SRC-spec §11.36.
            pub const GET: u32 = 0x0000_000A;
            /// `Get Attributes`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTES-0000000B; KMIPKIT-SRC-spec §11.36.
            pub const GET_ATTRIBUTES: u32 = 0x0000_000B;
            /// `Get Attribute List`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-ATTRIBUTE-LIST-0000000C; KMIPKIT-SRC-spec §11.36.
            pub const GET_ATTRIBUTE_LIST: u32 = 0x0000_000C;
            /// `Add Attribute`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADD-ATTRIBUTE-0000000D; KMIPKIT-SRC-spec §11.36.
            pub const ADD_ATTRIBUTE: u32 = 0x0000_000D;
            /// `Modify Attribute`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MODIFY-ATTRIBUTE-0000000E; KMIPKIT-SRC-spec §11.36.
            pub const MODIFY_ATTRIBUTE: u32 = 0x0000_000E;
            /// `Delete Attribute`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELETE-ATTRIBUTE-0000000F; KMIPKIT-SRC-spec §11.36.
            pub const DELETE_ATTRIBUTE: u32 = 0x0000_000F;
            /// `Obtain Lease`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-OBTAIN-LEASE-00000010; KMIPKIT-SRC-spec §11.36.
            pub const OBTAIN_LEASE: u32 = 0x0000_0010;
            /// `Get Usage Allocation`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-USAGE-ALLOCATION-00000011; KMIPKIT-SRC-spec §11.36.
            pub const GET_USAGE_ALLOCATION: u32 = 0x0000_0011;
            /// `Activate`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ACTIVATE-00000012; KMIPKIT-SRC-spec §11.36.
            pub const ACTIVATE: u32 = 0x0000_0012;
            /// `Revoke`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-REVOKE-00000013; KMIPKIT-SRC-spec §11.36.
            pub const REVOKE: u32 = 0x0000_0013;
            /// `Destroy`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DESTROY-00000014; KMIPKIT-SRC-spec §11.36.
            pub const DESTROY: u32 = 0x0000_0014;
            /// `Archive`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ARCHIVE-00000015; KMIPKIT-SRC-spec §11.36.
            pub const ARCHIVE: u32 = 0x0000_0015;
            /// `Recover`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RECOVER-00000016; KMIPKIT-SRC-spec §11.36.
            pub const RECOVER: u32 = 0x0000_0016;
            /// `Validate`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-VALIDATE-00000017; KMIPKIT-SRC-spec §11.36.
            pub const VALIDATE: u32 = 0x0000_0017;
            /// `Query`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-00000018; KMIPKIT-SRC-spec §11.36.
            pub const QUERY: u32 = 0x0000_0018;
            /// `Cancel`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CANCEL-00000019; KMIPKIT-SRC-spec §11.36.
            pub const CANCEL: u32 = 0x0000_0019;
            /// `Poll`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-POLL-0000001A; KMIPKIT-SRC-spec §11.36.
            pub const POLL: u32 = 0x0000_001A;
            /// `Notify`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-NOTIFY-0000001B; KMIPKIT-SRC-spec §11.36.
            pub const NOTIFY: u32 = 0x0000_001B;
            /// `Put`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PUT-0000001C; KMIPKIT-SRC-spec §11.36.
            pub const PUT: u32 = 0x0000_001C;
            /// `Re-key Key Pair`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-KEY-KEY-PAIR-0000001D; KMIPKIT-SRC-spec §11.36.
            pub const RE_KEY_KEY_PAIR: u32 = 0x0000_001D;
            /// `Discover Versions`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DISCOVER-VERSIONS-0000001E; KMIPKIT-SRC-spec §11.36.
            pub const DISCOVER_VERSIONS: u32 = 0x0000_001E;
            /// `Encrypt`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ENCRYPT-0000001F; KMIPKIT-SRC-spec §11.36.
            pub const ENCRYPT: u32 = 0x0000_001F;
            /// `Decrypt`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DECRYPT-00000020; KMIPKIT-SRC-spec §11.36.
            pub const DECRYPT: u32 = 0x0000_0020;
            /// `Sign`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGN-00000021; KMIPKIT-SRC-spec §11.36.
            pub const SIGN: u32 = 0x0000_0021;
            /// `Signature Verify`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SIGNATURE-VERIFY-00000022; KMIPKIT-SRC-spec §11.36.
            pub const SIGNATURE_VERIFY: u32 = 0x0000_0022;
            /// `MAC`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-00000023; KMIPKIT-SRC-spec §11.36.
            pub const MAC: u32 = 0x0000_0023;
            /// `MAC Verify`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-MAC-VERIFY-00000024; KMIPKIT-SRC-spec §11.36.
            pub const MAC_VERIFY: u32 = 0x0000_0024;
            /// `RNG Retrieve`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-RETRIEVE-00000025; KMIPKIT-SRC-spec §11.36.
            pub const RNG_RETRIEVE: u32 = 0x0000_0025;
            /// `RNG Seed`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RNG-SEED-00000026; KMIPKIT-SRC-spec §11.36.
            pub const RNG_SEED: u32 = 0x0000_0026;
            /// `Hash`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-HASH-00000027; KMIPKIT-SRC-spec §11.36.
            pub const HASH: u32 = 0x0000_0027;
            /// `Create Split Key`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-CREATE-SPLIT-KEY-00000028; KMIPKIT-SRC-spec §11.36.
            pub const CREATE_SPLIT_KEY: u32 = 0x0000_0028;
            /// `Join Split Key`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-JOIN-SPLIT-KEY-00000029; KMIPKIT-SRC-spec §11.36.
            pub const JOIN_SPLIT_KEY: u32 = 0x0000_0029;
            /// `Import`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-IMPORT-0000002A; KMIPKIT-SRC-spec §11.36.
            pub const IMPORT: u32 = 0x0000_002A;
            /// `Export`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-EXPORT-0000002B; KMIPKIT-SRC-spec §11.36.
            pub const EXPORT: u32 = 0x0000_002B;
            /// `Log`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOG-0000002C; KMIPKIT-SRC-spec §11.36.
            pub const LOG: u32 = 0x0000_002C;
            /// `Login`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGIN-0000002D; KMIPKIT-SRC-spec §11.36.
            pub const LOGIN: u32 = 0x0000_002D;
            /// `Logout`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-LOGOUT-0000002E; KMIPKIT-SRC-spec §11.36.
            pub const LOGOUT: u32 = 0x0000_002E;
            /// `Delegated Login`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-DELEGATED-LOGIN-0000002F; KMIPKIT-SRC-spec §11.36.
            pub const DELEGATED_LOGIN: u32 = 0x0000_002F;
            /// `Adjust Attribute`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-ADJUST-ATTRIBUTE-00000030; KMIPKIT-SRC-spec §11.36.
            pub const ADJUST_ATTRIBUTE: u32 = 0x0000_0030;
            /// `Set Attribute`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ATTRIBUTE-00000031; KMIPKIT-SRC-spec §11.36.
            pub const SET_ATTRIBUTE: u32 = 0x0000_0031;
            /// `Set Endpoint Role`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-ENDPOINT-ROLE-00000032; KMIPKIT-SRC-spec §11.36.
            pub const SET_ENDPOINT_ROLE: u32 = 0x0000_0032;
            /// `PKCS#11`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PKCS-11-00000033; KMIPKIT-SRC-spec §11.36.
            pub const PKCS_11: u32 = 0x0000_0033;
            /// `Interop`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-INTEROP-00000034; KMIPKIT-SRC-spec §11.36.
            pub const INTEROP: u32 = 0x0000_0034;
            /// `Re-Provision`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-RE-PROVISION-00000035; KMIPKIT-SRC-spec §11.36.
            pub const RE_PROVISION: u32 = 0x0000_0035;
            /// `Set Defaults`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-DEFAULTS-00000036; KMIPKIT-SRC-spec §11.36.
            pub const SET_DEFAULTS: u32 = 0x0000_0036;
            /// `Set Constraints`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-SET-CONSTRAINTS-00000037; KMIPKIT-SRC-spec §11.36.
            pub const SET_CONSTRAINTS: u32 = 0x0000_0037;
            /// `Get Constraints`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-GET-CONSTRAINTS-00000038; KMIPKIT-SRC-spec §11.36.
            pub const GET_CONSTRAINTS: u32 = 0x0000_0038;
            /// `Query Asynchronous Requests`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-QUERY-ASYNCHRONOUS-REQUESTS-00000039; KMIPKIT-SRC-spec §11.36.
            pub const QUERY_ASYNCHRONOUS_REQUESTS: u32 = 0x0000_0039;
            /// `Process`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PROCESS-0000003A; KMIPKIT-SRC-spec §11.36.
            pub const PROCESS: u32 = 0x0000_003A;
            /// `Ping`; KMIPKIT-ELEM-ENUM-VALUE-OPERATION-PING-0000003B; KMIPKIT-SRC-spec §11.36.
            pub const PING: u32 = 0x0000_003B;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Create"),
                    0x0000_0002 => Some("Create Key Pair"),
                    0x0000_0003 => Some("Register"),
                    0x0000_0004 => Some("Re-key"),
                    0x0000_0005 => Some("Derive Key"),
                    0x0000_0006 => Some("Certify"),
                    0x0000_0007 => Some("Re-certify"),
                    0x0000_0008 => Some("Locate"),
                    0x0000_0009 => Some("Check"),
                    0x0000_000A => Some("Get"),
                    0x0000_000B => Some("Get Attributes"),
                    0x0000_000C => Some("Get Attribute List"),
                    0x0000_000D => Some("Add Attribute"),
                    0x0000_000E => Some("Modify Attribute"),
                    0x0000_000F => Some("Delete Attribute"),
                    0x0000_0010 => Some("Obtain Lease"),
                    0x0000_0011 => Some("Get Usage Allocation"),
                    0x0000_0012 => Some("Activate"),
                    0x0000_0013 => Some("Revoke"),
                    0x0000_0014 => Some("Destroy"),
                    0x0000_0015 => Some("Archive"),
                    0x0000_0016 => Some("Recover"),
                    0x0000_0017 => Some("Validate"),
                    0x0000_0018 => Some("Query"),
                    0x0000_0019 => Some("Cancel"),
                    0x0000_001A => Some("Poll"),
                    0x0000_001B => Some("Notify"),
                    0x0000_001C => Some("Put"),
                    0x0000_001D => Some("Re-key Key Pair"),
                    0x0000_001E => Some("Discover Versions"),
                    0x0000_001F => Some("Encrypt"),
                    0x0000_0020 => Some("Decrypt"),
                    0x0000_0021 => Some("Sign"),
                    0x0000_0022 => Some("Signature Verify"),
                    0x0000_0023 => Some("MAC"),
                    0x0000_0024 => Some("MAC Verify"),
                    0x0000_0025 => Some("RNG Retrieve"),
                    0x0000_0026 => Some("RNG Seed"),
                    0x0000_0027 => Some("Hash"),
                    0x0000_0028 => Some("Create Split Key"),
                    0x0000_0029 => Some("Join Split Key"),
                    0x0000_002A => Some("Import"),
                    0x0000_002B => Some("Export"),
                    0x0000_002C => Some("Log"),
                    0x0000_002D => Some("Login"),
                    0x0000_002E => Some("Logout"),
                    0x0000_002F => Some("Delegated Login"),
                    0x0000_0030 => Some("Adjust Attribute"),
                    0x0000_0031 => Some("Set Attribute"),
                    0x0000_0032 => Some("Set Endpoint Role"),
                    0x0000_0033 => Some("PKCS#11"),
                    0x0000_0034 => Some("Interop"),
                    0x0000_0035 => Some("Re-Provision"),
                    0x0000_0036 => Some("Set Defaults"),
                    0x0000_0037 => Some("Set Constraints"),
                    0x0000_0038 => Some("Get Constraints"),
                    0x0000_0039 => Some("Query Asynchronous Requests"),
                    0x0000_003A => Some("Process"),
                    0x0000_003B => Some("Ping"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Padding Method Enumeration.
        pub mod padding_method {
            /// `None`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-NONE-00000001; KMIPKIT-SRC-spec §11.37.
            pub const NONE: u32 = 0x0000_0001;
            /// `OAEP`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-OAEP-00000002; KMIPKIT-SRC-spec §11.37.
            pub const OAEP: u32 = 0x0000_0002;
            /// `PKCS5`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS5-00000003; KMIPKIT-SRC-spec §11.37.
            pub const PKCS5: u32 = 0x0000_0003;
            /// `SSL3`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-SSL3-00000004; KMIPKIT-SRC-spec §11.37.
            pub const SSL3: u32 = 0x0000_0004;
            /// `Zeros`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ZEROS-00000005; KMIPKIT-SRC-spec §11.37.
            pub const ZEROS: u32 = 0x0000_0005;
            /// `ANSI X9.23`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ANSI-X9-23-00000006; KMIPKIT-SRC-spec §11.37.
            pub const ANSI_X9_23: u32 = 0x0000_0006;
            /// `ISO 10126`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-ISO-10126-00000007; KMIPKIT-SRC-spec §11.37.
            pub const ISO_10126: u32 = 0x0000_0007;
            /// `PKCS1 v1.5`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PKCS1-V1-5-00000008; KMIPKIT-SRC-spec §11.37.
            pub const PKCS1_V1_5: u32 = 0x0000_0008;
            /// `X9.31`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-X9-31-00000009; KMIPKIT-SRC-spec §11.37.
            pub const X9_31: u32 = 0x0000_0009;
            /// `PSS`; KMIPKIT-ELEM-ENUM-VALUE-PADDING-METHOD-PSS-0000000A; KMIPKIT-SRC-spec §11.37.
            pub const PSS: u32 = 0x0000_000A;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("None"),
                    0x0000_0002 => Some("OAEP"),
                    0x0000_0003 => Some("PKCS5"),
                    0x0000_0004 => Some("SSL3"),
                    0x0000_0005 => Some("Zeros"),
                    0x0000_0006 => Some("ANSI X9.23"),
                    0x0000_0007 => Some("ISO 10126"),
                    0x0000_0008 => Some("PKCS1 v1.5"),
                    0x0000_0009 => Some("X9.31"),
                    0x0000_000A => Some("PSS"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP PKCS#11 Function Enumeration.
        pub mod pkcs_11_function {
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(_raw: u32) -> Option<&'static str> {
                None
            }
        }

        /// Values for the KMIP PKCS#11 Return Code Enumeration.
        pub mod pkcs_11_return_code {
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(_raw: u32) -> Option<&'static str> {
                None
            }
        }

        /// Values for the KMIP Processing Stage Enumeration.
        pub mod processing_stage {
            /// `Submitted`; KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-SUBMITTED-00000001; KMIPKIT-SRC-spec §11.40.
            pub const SUBMITTED: u32 = 0x0000_0001;
            /// `In Process`; KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-IN-PROCESS-00000002; KMIPKIT-SRC-spec §11.40.
            pub const IN_PROCESS: u32 = 0x0000_0002;
            /// `Completed`; KMIPKIT-ELEM-ENUM-VALUE-PROCESSING-STAGE-COMPLETED-00000003; KMIPKIT-SRC-spec §11.40.
            pub const COMPLETED: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Submitted"),
                    0x0000_0002 => Some("In Process"),
                    0x0000_0003 => Some("Completed"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Profile Name Enumeration.
        pub mod profile_name {
            /// `Complete Server Basic`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-BASIC-00000104; KMIPKIT-SRC-spec §11.41.
            pub const COMPLETE_SERVER_BASIC: u32 = 0x0000_0104;
            /// `Complete Server TLS v1.2`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-TLS-V1-2-00000105; KMIPKIT-SRC-spec §11.41.
            pub const COMPLETE_SERVER_TLS_V1_2: u32 = 0x0000_0105;
            /// `Tape Library Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-CLIENT-00000106; KMIPKIT-SRC-spec §11.41.
            pub const TAPE_LIBRARY_CLIENT: u32 = 0x0000_0106;
            /// `Tape Library Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-TAPE-LIBRARY-SERVER-00000107; KMIPKIT-SRC-spec §11.41.
            pub const TAPE_LIBRARY_SERVER: u32 = 0x0000_0107;
            /// `Symmetric Key Lifecycle Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-CLIENT-00000108; KMIPKIT-SRC-spec §11.41.
            pub const SYMMETRIC_KEY_LIFECYCLE_CLIENT: u32 = 0x0000_0108;
            /// `Symmetric Key Lifecycle Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-LIFECYCLE-SERVER-00000109; KMIPKIT-SRC-spec §11.41.
            pub const SYMMETRIC_KEY_LIFECYCLE_SERVER: u32 = 0x0000_0109;
            /// `Asymmetric Key Lifecycle Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-CLIENT-0000010A; KMIPKIT-SRC-spec §11.41.
            pub const ASYMMETRIC_KEY_LIFECYCLE_CLIENT: u32 = 0x0000_010A;
            /// `Asymmetric Key Lifecycle Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ASYMMETRIC-KEY-LIFECYCLE-SERVER-0000010B; KMIPKIT-SRC-spec §11.41.
            pub const ASYMMETRIC_KEY_LIFECYCLE_SERVER: u32 = 0x0000_010B;
            /// `Basic Cryptographic Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-CLIENT-0000010C; KMIPKIT-SRC-spec §11.41.
            pub const BASIC_CRYPTOGRAPHIC_CLIENT: u32 = 0x0000_010C;
            /// `Basic Cryptographic Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-CRYPTOGRAPHIC-SERVER-0000010D; KMIPKIT-SRC-spec §11.41.
            pub const BASIC_CRYPTOGRAPHIC_SERVER: u32 = 0x0000_010D;
            /// `Advanced Cryptographic Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-CLIENT-0000010E; KMIPKIT-SRC-spec §11.41.
            pub const ADVANCED_CRYPTOGRAPHIC_CLIENT: u32 = 0x0000_010E;
            /// `Advanced Cryptographic Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-CRYPTOGRAPHIC-SERVER-0000010F; KMIPKIT-SRC-spec §11.41.
            pub const ADVANCED_CRYPTOGRAPHIC_SERVER: u32 = 0x0000_010F;
            /// `RNG Cryptographic Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-CLIENT-00000110; KMIPKIT-SRC-spec §11.41.
            pub const RNG_CRYPTOGRAPHIC_CLIENT: u32 = 0x0000_0110;
            /// `RNG Cryptographic Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-RNG-CRYPTOGRAPHIC-SERVER-00000111; KMIPKIT-SRC-spec §11.41.
            pub const RNG_CRYPTOGRAPHIC_SERVER: u32 = 0x0000_0111;
            /// `Basic Symmetric Key Foundry Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASIC-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000112; KMIPKIT-SRC-spec §11.41.
            pub const BASIC_SYMMETRIC_KEY_FOUNDRY_CLIENT: u32 = 0x0000_0112;
            /// `Intermediate Symmetric Key Foundry Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-INTERMEDIATE-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000113; KMIPKIT-SRC-spec §11.41.
            pub const INTERMEDIATE_SYMMETRIC_KEY_FOUNDRY_CLIENT: u32 = 0x0000_0113;
            /// `Advanced Symmetric Key Foundry Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-ADVANCED-SYMMETRIC-KEY-FOUNDRY-CLIENT-00000114; KMIPKIT-SRC-spec §11.41.
            pub const ADVANCED_SYMMETRIC_KEY_FOUNDRY_CLIENT: u32 = 0x0000_0114;
            /// `Symmetric Key Foundry Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-SYMMETRIC-KEY-FOUNDRY-SERVER-00000115; KMIPKIT-SRC-spec §11.41.
            pub const SYMMETRIC_KEY_FOUNDRY_SERVER: u32 = 0x0000_0115;
            /// `Opaque Managed Object Store Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-CLIENT-00000116; KMIPKIT-SRC-spec §11.41.
            pub const OPAQUE_MANAGED_OBJECT_STORE_CLIENT: u32 = 0x0000_0116;
            /// `Opaque Managed Object Store Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-OPAQUE-MANAGED-OBJECT-STORE-SERVER-00000117; KMIPKIT-SRC-spec §11.41.
            pub const OPAQUE_MANAGED_OBJECT_STORE_SERVER: u32 = 0x0000_0117;
            /// `Storage Array with Self Encrypting Drive Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-CLIENT-0000011C; KMIPKIT-SRC-spec §11.41.
            pub const STORAGE_ARRAY_WITH_SELF_ENCRYPTING_DRIVE_CLIENT: u32 = 0x0000_011C;
            /// `Storage Array with Self Encrypting Drive Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-STORAGE-ARRAY-WITH-SELF-ENCRYPTING-DRIVE-SERVER-0000011D; KMIPKIT-SRC-spec §11.41.
            pub const STORAGE_ARRAY_WITH_SELF_ENCRYPTING_DRIVE_SERVER: u32 = 0x0000_011D;
            /// `HTTPS Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-CLIENT-0000011E; KMIPKIT-SRC-spec §11.41.
            pub const HTTPS_CLIENT: u32 = 0x0000_011E;
            /// `HTTPS Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-HTTPS-SERVER-0000011F; KMIPKIT-SRC-spec §11.41.
            pub const HTTPS_SERVER: u32 = 0x0000_011F;
            /// `JSON Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-CLIENT-00000120; KMIPKIT-SRC-spec §11.41.
            pub const JSON_CLIENT: u32 = 0x0000_0120;
            /// `JSON Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-JSON-SERVER-00000121; KMIPKIT-SRC-spec §11.41.
            pub const JSON_SERVER: u32 = 0x0000_0121;
            /// `XML Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-CLIENT-00000122; KMIPKIT-SRC-spec §11.41.
            pub const XML_CLIENT: u32 = 0x0000_0122;
            /// `XML Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-XML-SERVER-00000123; KMIPKIT-SRC-spec §11.41.
            pub const XML_SERVER: u32 = 0x0000_0123;
            /// `AES XTS Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-CLIENT-00000124; KMIPKIT-SRC-spec §11.41.
            pub const AES_XTS_CLIENT: u32 = 0x0000_0124;
            /// `AES XTS Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-AES-XTS-SERVER-00000125; KMIPKIT-SRC-spec §11.41.
            pub const AES_XTS_SERVER: u32 = 0x0000_0125;
            /// `Quantum Safe Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-CLIENT-00000126; KMIPKIT-SRC-spec §11.41.
            pub const QUANTUM_SAFE_CLIENT: u32 = 0x0000_0126;
            /// `Quantum Safe Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-QUANTUM-SAFE-SERVER-00000127; KMIPKIT-SRC-spec §11.41.
            pub const QUANTUM_SAFE_SERVER: u32 = 0x0000_0127;
            /// `PKCS#11 Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-CLIENT-00000128; KMIPKIT-SRC-spec §11.41.
            pub const PKCS_11_CLIENT: u32 = 0x0000_0128;
            /// `PKCS#11 Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-PKCS-11-SERVER-00000129; KMIPKIT-SRC-spec §11.41.
            pub const PKCS_11_SERVER: u32 = 0x0000_0129;
            /// `Baseline Client`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-CLIENT-0000012A; KMIPKIT-SRC-spec §11.41.
            pub const BASELINE_CLIENT: u32 = 0x0000_012A;
            /// `Baseline Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-BASELINE-SERVER-0000012B; KMIPKIT-SRC-spec §11.41.
            pub const BASELINE_SERVER: u32 = 0x0000_012B;
            /// `Complete Server`; KMIPKIT-ELEM-ENUM-VALUE-PROFILE-NAME-COMPLETE-SERVER-0000012C; KMIPKIT-SRC-spec §11.41.
            pub const COMPLETE_SERVER: u32 = 0x0000_012C;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0104 => Some("Complete Server Basic"),
                    0x0000_0105 => Some("Complete Server TLS v1.2"),
                    0x0000_0106 => Some("Tape Library Client"),
                    0x0000_0107 => Some("Tape Library Server"),
                    0x0000_0108 => Some("Symmetric Key Lifecycle Client"),
                    0x0000_0109 => Some("Symmetric Key Lifecycle Server"),
                    0x0000_010A => Some("Asymmetric Key Lifecycle Client"),
                    0x0000_010B => Some("Asymmetric Key Lifecycle Server"),
                    0x0000_010C => Some("Basic Cryptographic Client"),
                    0x0000_010D => Some("Basic Cryptographic Server"),
                    0x0000_010E => Some("Advanced Cryptographic Client"),
                    0x0000_010F => Some("Advanced Cryptographic Server"),
                    0x0000_0110 => Some("RNG Cryptographic Client"),
                    0x0000_0111 => Some("RNG Cryptographic Server"),
                    0x0000_0112 => Some("Basic Symmetric Key Foundry Client"),
                    0x0000_0113 => Some("Intermediate Symmetric Key Foundry Client"),
                    0x0000_0114 => Some("Advanced Symmetric Key Foundry Client"),
                    0x0000_0115 => Some("Symmetric Key Foundry Server"),
                    0x0000_0116 => Some("Opaque Managed Object Store Client"),
                    0x0000_0117 => Some("Opaque Managed Object Store Server"),
                    0x0000_011C => Some("Storage Array with Self Encrypting Drive Client"),
                    0x0000_011D => Some("Storage Array with Self Encrypting Drive Server"),
                    0x0000_011E => Some("HTTPS Client"),
                    0x0000_011F => Some("HTTPS Server"),
                    0x0000_0120 => Some("JSON Client"),
                    0x0000_0121 => Some("JSON Server"),
                    0x0000_0122 => Some("XML Client"),
                    0x0000_0123 => Some("XML Server"),
                    0x0000_0124 => Some("AES XTS Client"),
                    0x0000_0125 => Some("AES XTS Server"),
                    0x0000_0126 => Some("Quantum Safe Client"),
                    0x0000_0127 => Some("Quantum Safe Server"),
                    0x0000_0128 => Some("PKCS#11 Client"),
                    0x0000_0129 => Some("PKCS#11 Server"),
                    0x0000_012A => Some("Baseline Client"),
                    0x0000_012B => Some("Baseline Server"),
                    0x0000_012C => Some("Complete Server"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Protection Level Enumeration.
        pub mod protection_level {
            /// `High`; KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-HIGH-00000001; KMIPKIT-SRC-spec §11.42.
            pub const HIGH: u32 = 0x0000_0001;
            /// `Low`; KMIPKIT-ELEM-ENUM-VALUE-PROTECTION-LEVEL-LOW-00000002; KMIPKIT-SRC-spec §11.42.
            pub const LOW: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("High"),
                    0x0000_0002 => Some("Low"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Put Function Enumeration.
        pub mod put_function {
            /// `New`; KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-NEW-00000001; KMIPKIT-SRC-spec §11.43.
            pub const NEW: u32 = 0x0000_0001;
            /// `Replace`; KMIPKIT-ELEM-ENUM-VALUE-PUT-FUNCTION-REPLACE-00000002; KMIPKIT-SRC-spec §11.43.
            pub const REPLACE: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("New"),
                    0x0000_0002 => Some("Replace"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Query Function Enumeration.
        pub mod query_function {
            /// `Query Operations`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OPERATIONS-00000001; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_OPERATIONS: u32 = 0x0000_0001;
            /// `Query Objects`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-OBJECTS-00000002; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_OBJECTS: u32 = 0x0000_0002;
            /// `Query Server Information`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-SERVER-INFORMATION-00000003; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_SERVER_INFORMATION: u32 = 0x0000_0003;
            /// `Query Application Namespaces`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-APPLICATION-NAMESPACES-00000004; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_APPLICATION_NAMESPACES: u32 = 0x0000_0004;
            /// `Query Extension List`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-LIST-00000005; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_EXTENSION_LIST: u32 = 0x0000_0005;
            /// `Query Extension Map`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-EXTENSION-MAP-00000006; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_EXTENSION_MAP: u32 = 0x0000_0006;
            /// `Query Attestation Types`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-ATTESTATION-TYPES-00000007; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_ATTESTATION_TYPES: u32 = 0x0000_0007;
            /// `Query RNGs`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-RNGS-00000008; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_RNGS: u32 = 0x0000_0008;
            /// `Query Validations`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-VALIDATIONS-00000009; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_VALIDATIONS: u32 = 0x0000_0009;
            /// `Query Profiles`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-PROFILES-0000000A; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_PROFILES: u32 = 0x0000_000A;
            /// `Query Capabilities`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CAPABILITIES-0000000B; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_CAPABILITIES: u32 = 0x0000_000B;
            /// `Query Client Registration Methods`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-CLIENT-REGISTRATION-METHODS-0000000C; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_CLIENT_REGISTRATION_METHODS: u32 = 0x0000_000C;
            /// `Query Defaults Information`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-DEFAULTS-INFORMATION-0000000D; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_DEFAULTS_INFORMATION: u32 = 0x0000_000D;
            /// `Query Storage Protection Masks`; KMIPKIT-ELEM-ENUM-VALUE-QUERY-FUNCTION-QUERY-STORAGE-PROTECTION-MASKS-0000000E; KMIPKIT-SRC-spec §11.44.
            pub const QUERY_STORAGE_PROTECTION_MASKS: u32 = 0x0000_000E;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Query Operations"),
                    0x0000_0002 => Some("Query Objects"),
                    0x0000_0003 => Some("Query Server Information"),
                    0x0000_0004 => Some("Query Application Namespaces"),
                    0x0000_0005 => Some("Query Extension List"),
                    0x0000_0006 => Some("Query Extension Map"),
                    0x0000_0007 => Some("Query Attestation Types"),
                    0x0000_0008 => Some("Query RNGs"),
                    0x0000_0009 => Some("Query Validations"),
                    0x0000_000A => Some("Query Profiles"),
                    0x0000_000B => Some("Query Capabilities"),
                    0x0000_000C => Some("Query Client Registration Methods"),
                    0x0000_000D => Some("Query Defaults Information"),
                    0x0000_000E => Some("Query Storage Protection Masks"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Recommended Curve Enumeration.
        pub mod recommended_curve {
            /// `P-192`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-192-00000001; KMIPKIT-SRC-spec §11.45.
            pub const P_192: u32 = 0x0000_0001;
            /// `K-163`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-163-00000002; KMIPKIT-SRC-spec §11.45.
            pub const K_163: u32 = 0x0000_0002;
            /// `B-163`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-163-00000003; KMIPKIT-SRC-spec §11.45.
            pub const B_163: u32 = 0x0000_0003;
            /// `P-224`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-224-00000004; KMIPKIT-SRC-spec §11.45.
            pub const P_224: u32 = 0x0000_0004;
            /// `K-233`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-233-00000005; KMIPKIT-SRC-spec §11.45.
            pub const K_233: u32 = 0x0000_0005;
            /// `B-233`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-233-00000006; KMIPKIT-SRC-spec §11.45.
            pub const B_233: u32 = 0x0000_0006;
            /// `P-256`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-256-00000007; KMIPKIT-SRC-spec §11.45.
            pub const P_256: u32 = 0x0000_0007;
            /// `K-283`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-283-00000008; KMIPKIT-SRC-spec §11.45.
            pub const K_283: u32 = 0x0000_0008;
            /// `B-283`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-283-00000009; KMIPKIT-SRC-spec §11.45.
            pub const B_283: u32 = 0x0000_0009;
            /// `P-384`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-384-0000000A; KMIPKIT-SRC-spec §11.45.
            pub const P_384: u32 = 0x0000_000A;
            /// `K-409`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-409-0000000B; KMIPKIT-SRC-spec §11.45.
            pub const K_409: u32 = 0x0000_000B;
            /// `B-409`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-409-0000000C; KMIPKIT-SRC-spec §11.45.
            pub const B_409: u32 = 0x0000_000C;
            /// `P-521`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-P-521-0000000D; KMIPKIT-SRC-spec §11.45.
            pub const P_521: u32 = 0x0000_000D;
            /// `K-571`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-K-571-0000000E; KMIPKIT-SRC-spec §11.45.
            pub const K_571: u32 = 0x0000_000E;
            /// `B-571`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-B-571-0000000F; KMIPKIT-SRC-spec §11.45.
            pub const B_571: u32 = 0x0000_000F;
            /// `SECP112R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R1-00000010; KMIPKIT-SRC-spec §11.45.
            pub const SECP112R1: u32 = 0x0000_0010;
            /// `SECP112R2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP112R2-00000011; KMIPKIT-SRC-spec §11.45.
            pub const SECP112R2: u32 = 0x0000_0011;
            /// `SECP128R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R1-00000012; KMIPKIT-SRC-spec §11.45.
            pub const SECP128R1: u32 = 0x0000_0012;
            /// `SECP128R2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP128R2-00000013; KMIPKIT-SRC-spec §11.45.
            pub const SECP128R2: u32 = 0x0000_0013;
            /// `SECP160K1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160K1-00000014; KMIPKIT-SRC-spec §11.45.
            pub const SECP160K1: u32 = 0x0000_0014;
            /// `SECP160R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R1-00000015; KMIPKIT-SRC-spec §11.45.
            pub const SECP160R1: u32 = 0x0000_0015;
            /// `SECP160R2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP160R2-00000016; KMIPKIT-SRC-spec §11.45.
            pub const SECP160R2: u32 = 0x0000_0016;
            /// `SECP192K1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP192K1-00000017; KMIPKIT-SRC-spec §11.45.
            pub const SECP192K1: u32 = 0x0000_0017;
            /// `SECP224K1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP224K1-00000018; KMIPKIT-SRC-spec §11.45.
            pub const SECP224K1: u32 = 0x0000_0018;
            /// `SECP256K1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECP256K1-00000019; KMIPKIT-SRC-spec §11.45.
            pub const SECP256K1: u32 = 0x0000_0019;
            /// `SECT113R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R1-0000001A; KMIPKIT-SRC-spec §11.45.
            pub const SECT113R1: u32 = 0x0000_001A;
            /// `SECT113R2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT113R2-0000001B; KMIPKIT-SRC-spec §11.45.
            pub const SECT113R2: u32 = 0x0000_001B;
            /// `SECT131R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R1-0000001C; KMIPKIT-SRC-spec §11.45.
            pub const SECT131R1: u32 = 0x0000_001C;
            /// `SECT131R2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT131R2-0000001D; KMIPKIT-SRC-spec §11.45.
            pub const SECT131R2: u32 = 0x0000_001D;
            /// `SECT163R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT163R1-0000001E; KMIPKIT-SRC-spec §11.45.
            pub const SECT163R1: u32 = 0x0000_001E;
            /// `SECT193R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R1-0000001F; KMIPKIT-SRC-spec §11.45.
            pub const SECT193R1: u32 = 0x0000_001F;
            /// `SECT193R2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT193R2-00000020; KMIPKIT-SRC-spec §11.45.
            pub const SECT193R2: u32 = 0x0000_0020;
            /// `SECT239K1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-SECT239K1-00000021; KMIPKIT-SRC-spec §11.45.
            pub const SECT239K1: u32 = 0x0000_0021;
            /// `ANSIX9P192V2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V2-00000022; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9P192V2: u32 = 0x0000_0022;
            /// `ANSIX9P192V3`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P192V3-00000023; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9P192V3: u32 = 0x0000_0023;
            /// `ANSIX9P239V1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V1-00000024; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9P239V1: u32 = 0x0000_0024;
            /// `ANSIX9P239V2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V2-00000025; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9P239V2: u32 = 0x0000_0025;
            /// `ANSIX9P239V3`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9P239V3-00000026; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9P239V3: u32 = 0x0000_0026;
            /// `ANSIX9C2PNB163V1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V1-00000027; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB163V1: u32 = 0x0000_0027;
            /// `ANSIX9C2PNB163V2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V2-00000028; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB163V2: u32 = 0x0000_0028;
            /// `ANSIX9C2PNB163V3`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB163V3-00000029; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB163V3: u32 = 0x0000_0029;
            /// `ANSIX9C2PNB176V1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB176V1-0000002A; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB176V1: u32 = 0x0000_002A;
            /// `ANSIX9C2TNB191V1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V1-0000002B; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB191V1: u32 = 0x0000_002B;
            /// `ANSIX9C2TNB191V2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V2-0000002C; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB191V2: u32 = 0x0000_002C;
            /// `ANSIX9C2TNB191V3`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB191V3-0000002D; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB191V3: u32 = 0x0000_002D;
            /// `ANSIX9C2PNB208W1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB208W1-0000002E; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB208W1: u32 = 0x0000_002E;
            /// `ANSIX9C2TNB239V1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V1-0000002F; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB239V1: u32 = 0x0000_002F;
            /// `ANSIX9C2TNB239V2`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V2-00000030; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB239V2: u32 = 0x0000_0030;
            /// `ANSIX9C2TNB239V3`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB239V3-00000031; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB239V3: u32 = 0x0000_0031;
            /// `ANSIX9C2PNB272W1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB272W1-00000032; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB272W1: u32 = 0x0000_0032;
            /// `ANSIX9C2PNB304W1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB304W1-00000033; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB304W1: u32 = 0x0000_0033;
            /// `ANSIX9C2TNB359V1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB359V1-00000034; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB359V1: u32 = 0x0000_0034;
            /// `ANSIX9C2PNB368W1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2PNB368W1-00000035; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2PNB368W1: u32 = 0x0000_0035;
            /// `ANSIX9C2TNB431R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-ANSIX9C2TNB431R1-00000036; KMIPKIT-SRC-spec §11.45.
            pub const ANSIX9C2TNB431R1: u32 = 0x0000_0036;
            /// `BRAINPOOLP160R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160R1-00000037; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP160R1: u32 = 0x0000_0037;
            /// `BRAINPOOLP160T1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP160T1-00000038; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP160T1: u32 = 0x0000_0038;
            /// `BRAINPOOLP192R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192R1-00000039; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP192R1: u32 = 0x0000_0039;
            /// `BRAINPOOLP192T1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP192T1-0000003A; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP192T1: u32 = 0x0000_003A;
            /// `BRAINPOOLP224R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224R1-0000003B; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP224R1: u32 = 0x0000_003B;
            /// `BRAINPOOLP224T1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP224T1-0000003C; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP224T1: u32 = 0x0000_003C;
            /// `BRAINPOOLP256R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256R1-0000003D; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP256R1: u32 = 0x0000_003D;
            /// `BRAINPOOLP256T1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP256T1-0000003E; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP256T1: u32 = 0x0000_003E;
            /// `BRAINPOOLP320R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320R1-0000003F; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP320R1: u32 = 0x0000_003F;
            /// `BRAINPOOLP320T1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP320T1-00000040; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP320T1: u32 = 0x0000_0040;
            /// `BRAINPOOLP384R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384R1-00000041; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP384R1: u32 = 0x0000_0041;
            /// `BRAINPOOLP384T1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP384T1-00000042; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP384T1: u32 = 0x0000_0042;
            /// `BRAINPOOLP512R1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512R1-00000043; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP512R1: u32 = 0x0000_0043;
            /// `BRAINPOOLP512T1`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-BRAINPOOLP512T1-00000044; KMIPKIT-SRC-spec §11.45.
            pub const BRAINPOOLP512T1: u32 = 0x0000_0044;
            /// `CURVE25519`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE25519-00000045; KMIPKIT-SRC-spec §11.45.
            pub const CURVE25519: u32 = 0x0000_0045;
            /// `CURVE448`; KMIPKIT-ELEM-ENUM-VALUE-RECOMMENDED-CURVE-CURVE448-00000046; KMIPKIT-SRC-spec §11.45.
            pub const CURVE448: u32 = 0x0000_0046;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("P-192"),
                    0x0000_0002 => Some("K-163"),
                    0x0000_0003 => Some("B-163"),
                    0x0000_0004 => Some("P-224"),
                    0x0000_0005 => Some("K-233"),
                    0x0000_0006 => Some("B-233"),
                    0x0000_0007 => Some("P-256"),
                    0x0000_0008 => Some("K-283"),
                    0x0000_0009 => Some("B-283"),
                    0x0000_000A => Some("P-384"),
                    0x0000_000B => Some("K-409"),
                    0x0000_000C => Some("B-409"),
                    0x0000_000D => Some("P-521"),
                    0x0000_000E => Some("K-571"),
                    0x0000_000F => Some("B-571"),
                    0x0000_0010 => Some("SECP112R1"),
                    0x0000_0011 => Some("SECP112R2"),
                    0x0000_0012 => Some("SECP128R1"),
                    0x0000_0013 => Some("SECP128R2"),
                    0x0000_0014 => Some("SECP160K1"),
                    0x0000_0015 => Some("SECP160R1"),
                    0x0000_0016 => Some("SECP160R2"),
                    0x0000_0017 => Some("SECP192K1"),
                    0x0000_0018 => Some("SECP224K1"),
                    0x0000_0019 => Some("SECP256K1"),
                    0x0000_001A => Some("SECT113R1"),
                    0x0000_001B => Some("SECT113R2"),
                    0x0000_001C => Some("SECT131R1"),
                    0x0000_001D => Some("SECT131R2"),
                    0x0000_001E => Some("SECT163R1"),
                    0x0000_001F => Some("SECT193R1"),
                    0x0000_0020 => Some("SECT193R2"),
                    0x0000_0021 => Some("SECT239K1"),
                    0x0000_0022 => Some("ANSIX9P192V2"),
                    0x0000_0023 => Some("ANSIX9P192V3"),
                    0x0000_0024 => Some("ANSIX9P239V1"),
                    0x0000_0025 => Some("ANSIX9P239V2"),
                    0x0000_0026 => Some("ANSIX9P239V3"),
                    0x0000_0027 => Some("ANSIX9C2PNB163V1"),
                    0x0000_0028 => Some("ANSIX9C2PNB163V2"),
                    0x0000_0029 => Some("ANSIX9C2PNB163V3"),
                    0x0000_002A => Some("ANSIX9C2PNB176V1"),
                    0x0000_002B => Some("ANSIX9C2TNB191V1"),
                    0x0000_002C => Some("ANSIX9C2TNB191V2"),
                    0x0000_002D => Some("ANSIX9C2TNB191V3"),
                    0x0000_002E => Some("ANSIX9C2PNB208W1"),
                    0x0000_002F => Some("ANSIX9C2TNB239V1"),
                    0x0000_0030 => Some("ANSIX9C2TNB239V2"),
                    0x0000_0031 => Some("ANSIX9C2TNB239V3"),
                    0x0000_0032 => Some("ANSIX9C2PNB272W1"),
                    0x0000_0033 => Some("ANSIX9C2PNB304W1"),
                    0x0000_0034 => Some("ANSIX9C2TNB359V1"),
                    0x0000_0035 => Some("ANSIX9C2PNB368W1"),
                    0x0000_0036 => Some("ANSIX9C2TNB431R1"),
                    0x0000_0037 => Some("BRAINPOOLP160R1"),
                    0x0000_0038 => Some("BRAINPOOLP160T1"),
                    0x0000_0039 => Some("BRAINPOOLP192R1"),
                    0x0000_003A => Some("BRAINPOOLP192T1"),
                    0x0000_003B => Some("BRAINPOOLP224R1"),
                    0x0000_003C => Some("BRAINPOOLP224T1"),
                    0x0000_003D => Some("BRAINPOOLP256R1"),
                    0x0000_003E => Some("BRAINPOOLP256T1"),
                    0x0000_003F => Some("BRAINPOOLP320R1"),
                    0x0000_0040 => Some("BRAINPOOLP320T1"),
                    0x0000_0041 => Some("BRAINPOOLP384R1"),
                    0x0000_0042 => Some("BRAINPOOLP384T1"),
                    0x0000_0043 => Some("BRAINPOOLP512R1"),
                    0x0000_0044 => Some("BRAINPOOLP512T1"),
                    0x0000_0045 => Some("CURVE25519"),
                    0x0000_0046 => Some("CURVE448"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Result Reason Enumeration.
        pub mod result_reason {
            /// `Item Not Found`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ITEM-NOT-FOUND-00000001; KMIPKIT-SRC-spec §11.46.
            pub const ITEM_NOT_FOUND: u32 = 0x0000_0001;
            /// `Response Too Large`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-RESPONSE-TOO-LARGE-00000002; KMIPKIT-SRC-spec §11.46.
            pub const RESPONSE_TOO_LARGE: u32 = 0x0000_0002;
            /// `Authentication Not Successful`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-AUTHENTICATION-NOT-SUCCESSFUL-00000003; KMIPKIT-SRC-spec §11.46.
            pub const AUTHENTICATION_NOT_SUCCESSFUL: u32 = 0x0000_0003;
            /// `Invalid Message`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-MESSAGE-00000004; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_MESSAGE: u32 = 0x0000_0004;
            /// `Operation Not Supported`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OPERATION-NOT-SUPPORTED-00000005; KMIPKIT-SRC-spec §11.46.
            pub const OPERATION_NOT_SUPPORTED: u32 = 0x0000_0005;
            /// `Missing Data`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MISSING-DATA-00000006; KMIPKIT-SRC-spec §11.46.
            pub const MISSING_DATA: u32 = 0x0000_0006;
            /// `Invalid Field`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-FIELD-00000007; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_FIELD: u32 = 0x0000_0007;
            /// `Feature Not Supported`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-FEATURE-NOT-SUPPORTED-00000008; KMIPKIT-SRC-spec §11.46.
            pub const FEATURE_NOT_SUPPORTED: u32 = 0x0000_0008;
            /// `Operation Canceled By Requester`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OPERATION-CANCELED-BY-REQUESTER-00000009; KMIPKIT-SRC-spec §11.46.
            pub const OPERATION_CANCELED_BY_REQUESTER: u32 = 0x0000_0009;
            /// `Cryptographic Failure`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CRYPTOGRAPHIC-FAILURE-0000000A; KMIPKIT-SRC-spec §11.46.
            pub const CRYPTOGRAPHIC_FAILURE: u32 = 0x0000_000A;
            /// `Permission Denied`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PERMISSION-DENIED-0000000C; KMIPKIT-SRC-spec §11.46.
            pub const PERMISSION_DENIED: u32 = 0x0000_000C;
            /// `Object Archived`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-ARCHIVED-0000000D; KMIPKIT-SRC-spec §11.46.
            pub const OBJECT_ARCHIVED: u32 = 0x0000_000D;
            /// `Application Namespace Not Supported`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-APPLICATION-NAMESPACE-NOT-SUPPORTED-0000000F; KMIPKIT-SRC-spec §11.46.
            pub const APPLICATION_NAMESPACE_NOT_SUPPORTED: u32 = 0x0000_000F;
            /// `Key Format Type Not Supported`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-FORMAT-TYPE-NOT-SUPPORTED-00000010; KMIPKIT-SRC-spec §11.46.
            pub const KEY_FORMAT_TYPE_NOT_SUPPORTED: u32 = 0x0000_0010;
            /// `Key Compression Type Not Supported`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-COMPRESSION-TYPE-NOT-SUPPORTED-00000011; KMIPKIT-SRC-spec §11.46.
            pub const KEY_COMPRESSION_TYPE_NOT_SUPPORTED: u32 = 0x0000_0011;
            /// `Encoding Option Error`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ENCODING-OPTION-ERROR-00000012; KMIPKIT-SRC-spec §11.46.
            pub const ENCODING_OPTION_ERROR: u32 = 0x0000_0012;
            /// `Key Value Not Present`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-VALUE-NOT-PRESENT-00000013; KMIPKIT-SRC-spec §11.46.
            pub const KEY_VALUE_NOT_PRESENT: u32 = 0x0000_0013;
            /// `Attestation Required`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTESTATION-REQUIRED-00000014; KMIPKIT-SRC-spec §11.46.
            pub const ATTESTATION_REQUIRED: u32 = 0x0000_0014;
            /// `Attestation Failed`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTESTATION-FAILED-00000015; KMIPKIT-SRC-spec §11.46.
            pub const ATTESTATION_FAILED: u32 = 0x0000_0015;
            /// `Sensitive`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-SENSITIVE-00000016; KMIPKIT-SRC-spec §11.46.
            pub const SENSITIVE: u32 = 0x0000_0016;
            /// `Not Extractable`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NOT-EXTRACTABLE-00000017; KMIPKIT-SRC-spec §11.46.
            pub const NOT_EXTRACTABLE: u32 = 0x0000_0017;
            /// `Object Already Exists`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-ALREADY-EXISTS-00000018; KMIPKIT-SRC-spec §11.46.
            pub const OBJECT_ALREADY_EXISTS: u32 = 0x0000_0018;
            /// `Invalid Ticket`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-TICKET-00000019; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_TICKET: u32 = 0x0000_0019;
            /// `Usage Limit Exceeded`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-USAGE-LIMIT-EXCEEDED-0000001A; KMIPKIT-SRC-spec §11.46.
            pub const USAGE_LIMIT_EXCEEDED: u32 = 0x0000_001A;
            /// `Numeric Range`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NUMERIC-RANGE-0000001B; KMIPKIT-SRC-spec §11.46.
            pub const NUMERIC_RANGE: u32 = 0x0000_001B;
            /// `Invalid Data Type`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-DATA-TYPE-0000001C; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_DATA_TYPE: u32 = 0x0000_001C;
            /// `Read Only Attribute`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-READ-ONLY-ATTRIBUTE-0000001D; KMIPKIT-SRC-spec §11.46.
            pub const READ_ONLY_ATTRIBUTE: u32 = 0x0000_001D;
            /// `Multi Valued Attribute`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MULTI-VALUED-ATTRIBUTE-0000001E; KMIPKIT-SRC-spec §11.46.
            pub const MULTI_VALUED_ATTRIBUTE: u32 = 0x0000_001E;
            /// `Unsupported Attribute`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-ATTRIBUTE-0000001F; KMIPKIT-SRC-spec §11.46.
            pub const UNSUPPORTED_ATTRIBUTE: u32 = 0x0000_001F;
            /// `Attribute Instance Not Found`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-INSTANCE-NOT-FOUND-00000020; KMIPKIT-SRC-spec §11.46.
            pub const ATTRIBUTE_INSTANCE_NOT_FOUND: u32 = 0x0000_0020;
            /// `Attribute Not Found`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-NOT-FOUND-00000021; KMIPKIT-SRC-spec §11.46.
            pub const ATTRIBUTE_NOT_FOUND: u32 = 0x0000_0021;
            /// `Attribute Read Only`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-READ-ONLY-00000022; KMIPKIT-SRC-spec §11.46.
            pub const ATTRIBUTE_READ_ONLY: u32 = 0x0000_0022;
            /// `Attribute Single Valued`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ATTRIBUTE-SINGLE-VALUED-00000023; KMIPKIT-SRC-spec §11.46.
            pub const ATTRIBUTE_SINGLE_VALUED: u32 = 0x0000_0023;
            /// `Bad Cryptographic Parameters`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-BAD-CRYPTOGRAPHIC-PARAMETERS-00000024; KMIPKIT-SRC-spec §11.46.
            pub const BAD_CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0000_0024;
            /// `Bad Password`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-BAD-PASSWORD-00000025; KMIPKIT-SRC-spec §11.46.
            pub const BAD_PASSWORD: u32 = 0x0000_0025;
            /// `Codec Error`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CODEC-ERROR-00000026; KMIPKIT-SRC-spec §11.46.
            pub const CODEC_ERROR: u32 = 0x0000_0026;
            /// `Illegal Object Type`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-ILLEGAL-OBJECT-TYPE-00000028; KMIPKIT-SRC-spec §11.46.
            pub const ILLEGAL_OBJECT_TYPE: u32 = 0x0000_0028;
            /// `Incompatible Cryptographic Usage Mask`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INCOMPATIBLE-CRYPTOGRAPHIC-USAGE-MASK-00000029; KMIPKIT-SRC-spec §11.46.
            pub const INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK: u32 = 0x0000_0029;
            /// `Internal Server Error`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INTERNAL-SERVER-ERROR-0000002A; KMIPKIT-SRC-spec §11.46.
            pub const INTERNAL_SERVER_ERROR: u32 = 0x0000_002A;
            /// `Invalid Asynchronous Correlation Value`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ASYNCHRONOUS-CORRELATION-VALUE-0000002B; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_ASYNCHRONOUS_CORRELATION_VALUE: u32 = 0x0000_002B;
            /// `Invalid Attribute`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ATTRIBUTE-0000002C; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_ATTRIBUTE: u32 = 0x0000_002C;
            /// `Invalid Attribute Value`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-ATTRIBUTE-VALUE-0000002D; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_ATTRIBUTE_VALUE: u32 = 0x0000_002D;
            /// `Invalid Correlation Value`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-CORRELATION-VALUE-0000002E; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_CORRELATION_VALUE: u32 = 0x0000_002E;
            /// `Invalid CSR`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-CSR-0000002F; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_CSR: u32 = 0x0000_002F;
            /// `Invalid Object Type`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-INVALID-OBJECT-TYPE-00000030; KMIPKIT-SRC-spec §11.46.
            pub const INVALID_OBJECT_TYPE: u32 = 0x0000_0030;
            /// `Key Wrap Type Not Supported`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-KEY-WRAP-TYPE-NOT-SUPPORTED-00000032; KMIPKIT-SRC-spec §11.46.
            pub const KEY_WRAP_TYPE_NOT_SUPPORTED: u32 = 0x0000_0032;
            /// `Missing Initialization Vector`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-MISSING-INITIALIZATION-VECTOR-00000034; KMIPKIT-SRC-spec §11.46.
            pub const MISSING_INITIALIZATION_VECTOR: u32 = 0x0000_0034;
            /// `Non Unique Name Attribute`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NON-UNIQUE-NAME-ATTRIBUTE-00000035; KMIPKIT-SRC-spec §11.46.
            pub const NON_UNIQUE_NAME_ATTRIBUTE: u32 = 0x0000_0035;
            /// `Object Destroyed`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-DESTROYED-00000036; KMIPKIT-SRC-spec §11.46.
            pub const OBJECT_DESTROYED: u32 = 0x0000_0036;
            /// `Object Not Found`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-OBJECT-NOT-FOUND-00000037; KMIPKIT-SRC-spec §11.46.
            pub const OBJECT_NOT_FOUND: u32 = 0x0000_0037;
            /// `Not Authorised`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-NOT-AUTHORISED-00000039; KMIPKIT-SRC-spec §11.46.
            pub const NOT_AUTHORISED: u32 = 0x0000_0039;
            /// `Server Limit Exceeded`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-SERVER-LIMIT-EXCEEDED-0000003A; KMIPKIT-SRC-spec §11.46.
            pub const SERVER_LIMIT_EXCEEDED: u32 = 0x0000_003A;
            /// `Unknown Enumeration`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-ENUMERATION-0000003B; KMIPKIT-SRC-spec §11.46.
            pub const UNKNOWN_ENUMERATION: u32 = 0x0000_003B;
            /// `Unknown Message Extension`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-MESSAGE-EXTENSION-0000003C; KMIPKIT-SRC-spec §11.46.
            pub const UNKNOWN_MESSAGE_EXTENSION: u32 = 0x0000_003C;
            /// `Unknown Tag`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-TAG-0000003D; KMIPKIT-SRC-spec §11.46.
            pub const UNKNOWN_TAG: u32 = 0x0000_003D;
            /// `Unsupported Cryptographic Parameters`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-CRYPTOGRAPHIC-PARAMETERS-0000003E; KMIPKIT-SRC-spec §11.46.
            pub const UNSUPPORTED_CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0000_003E;
            /// `Unsupported Protocol Version`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNSUPPORTED-PROTOCOL-VERSION-0000003F; KMIPKIT-SRC-spec §11.46.
            pub const UNSUPPORTED_PROTOCOL_VERSION: u32 = 0x0000_003F;
            /// `Wrapping Object Archived`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-ARCHIVED-00000040; KMIPKIT-SRC-spec §11.46.
            pub const WRAPPING_OBJECT_ARCHIVED: u32 = 0x0000_0040;
            /// `Wrapping Object Destroyed`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-DESTROYED-00000041; KMIPKIT-SRC-spec §11.46.
            pub const WRAPPING_OBJECT_DESTROYED: u32 = 0x0000_0041;
            /// `Wrapping Object Not Found`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRAPPING-OBJECT-NOT-FOUND-00000042; KMIPKIT-SRC-spec §11.46.
            pub const WRAPPING_OBJECT_NOT_FOUND: u32 = 0x0000_0042;
            /// `Wrong Key Lifecycle State`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-WRONG-KEY-LIFECYCLE-STATE-00000043; KMIPKIT-SRC-spec §11.46.
            pub const WRONG_KEY_LIFECYCLE_STATE: u32 = 0x0000_0043;
            /// `Protection Storage Unavailable`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PROTECTION-STORAGE-UNAVAILABLE-00000044; KMIPKIT-SRC-spec §11.46.
            pub const PROTECTION_STORAGE_UNAVAILABLE: u32 = 0x0000_0044;
            /// `PKCS#11 Codec Error`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-CODEC-ERROR-00000045; KMIPKIT-SRC-spec §11.46.
            pub const PKCS_11_CODEC_ERROR: u32 = 0x0000_0045;
            /// `PKCS#11 Invalid Function`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-INVALID-FUNCTION-00000046; KMIPKIT-SRC-spec §11.46.
            pub const PKCS_11_INVALID_FUNCTION: u32 = 0x0000_0046;
            /// `PKCS#11 Invalid Interface`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PKCS-11-INVALID-INTERFACE-00000047; KMIPKIT-SRC-spec §11.46.
            pub const PKCS_11_INVALID_INTERFACE: u32 = 0x0000_0047;
            /// `Private Protection Storage Unavailable`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PRIVATE-PROTECTION-STORAGE-UNAVAILABLE-00000048; KMIPKIT-SRC-spec §11.46.
            pub const PRIVATE_PROTECTION_STORAGE_UNAVAILABLE: u32 = 0x0000_0048;
            /// `Public Protection Storage Unavailable`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-PUBLIC-PROTECTION-STORAGE-UNAVAILABLE-00000049; KMIPKIT-SRC-spec §11.46.
            pub const PUBLIC_PROTECTION_STORAGE_UNAVAILABLE: u32 = 0x0000_0049;
            /// `Unknown Object Group`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-UNKNOWN-OBJECT-GROUP-0000004A; KMIPKIT-SRC-spec §11.46.
            pub const UNKNOWN_OBJECT_GROUP: u32 = 0x0000_004A;
            /// `Constraint Violation`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-CONSTRAINT-VIOLATION-0000004B; KMIPKIT-SRC-spec §11.46.
            pub const CONSTRAINT_VIOLATION: u32 = 0x0000_004B;
            /// `Duplicate Process Request`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-DUPLICATE-PROCESS-REQUEST-0000004C; KMIPKIT-SRC-spec §11.46.
            pub const DUPLICATE_PROCESS_REQUEST: u32 = 0x0000_004C;
            /// `General Failure`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-REASON-GENERAL-FAILURE-00000100; KMIPKIT-SRC-spec §11.46.
            pub const GENERAL_FAILURE: u32 = 0x0000_0100;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Item Not Found"),
                    0x0000_0002 => Some("Response Too Large"),
                    0x0000_0003 => Some("Authentication Not Successful"),
                    0x0000_0004 => Some("Invalid Message"),
                    0x0000_0005 => Some("Operation Not Supported"),
                    0x0000_0006 => Some("Missing Data"),
                    0x0000_0007 => Some("Invalid Field"),
                    0x0000_0008 => Some("Feature Not Supported"),
                    0x0000_0009 => Some("Operation Canceled By Requester"),
                    0x0000_000A => Some("Cryptographic Failure"),
                    0x0000_000C => Some("Permission Denied"),
                    0x0000_000D => Some("Object Archived"),
                    0x0000_000F => Some("Application Namespace Not Supported"),
                    0x0000_0010 => Some("Key Format Type Not Supported"),
                    0x0000_0011 => Some("Key Compression Type Not Supported"),
                    0x0000_0012 => Some("Encoding Option Error"),
                    0x0000_0013 => Some("Key Value Not Present"),
                    0x0000_0014 => Some("Attestation Required"),
                    0x0000_0015 => Some("Attestation Failed"),
                    0x0000_0016 => Some("Sensitive"),
                    0x0000_0017 => Some("Not Extractable"),
                    0x0000_0018 => Some("Object Already Exists"),
                    0x0000_0019 => Some("Invalid Ticket"),
                    0x0000_001A => Some("Usage Limit Exceeded"),
                    0x0000_001B => Some("Numeric Range"),
                    0x0000_001C => Some("Invalid Data Type"),
                    0x0000_001D => Some("Read Only Attribute"),
                    0x0000_001E => Some("Multi Valued Attribute"),
                    0x0000_001F => Some("Unsupported Attribute"),
                    0x0000_0020 => Some("Attribute Instance Not Found"),
                    0x0000_0021 => Some("Attribute Not Found"),
                    0x0000_0022 => Some("Attribute Read Only"),
                    0x0000_0023 => Some("Attribute Single Valued"),
                    0x0000_0024 => Some("Bad Cryptographic Parameters"),
                    0x0000_0025 => Some("Bad Password"),
                    0x0000_0026 => Some("Codec Error"),
                    0x0000_0028 => Some("Illegal Object Type"),
                    0x0000_0029 => Some("Incompatible Cryptographic Usage Mask"),
                    0x0000_002A => Some("Internal Server Error"),
                    0x0000_002B => Some("Invalid Asynchronous Correlation Value"),
                    0x0000_002C => Some("Invalid Attribute"),
                    0x0000_002D => Some("Invalid Attribute Value"),
                    0x0000_002E => Some("Invalid Correlation Value"),
                    0x0000_002F => Some("Invalid CSR"),
                    0x0000_0030 => Some("Invalid Object Type"),
                    0x0000_0032 => Some("Key Wrap Type Not Supported"),
                    0x0000_0034 => Some("Missing Initialization Vector"),
                    0x0000_0035 => Some("Non Unique Name Attribute"),
                    0x0000_0036 => Some("Object Destroyed"),
                    0x0000_0037 => Some("Object Not Found"),
                    0x0000_0039 => Some("Not Authorised"),
                    0x0000_003A => Some("Server Limit Exceeded"),
                    0x0000_003B => Some("Unknown Enumeration"),
                    0x0000_003C => Some("Unknown Message Extension"),
                    0x0000_003D => Some("Unknown Tag"),
                    0x0000_003E => Some("Unsupported Cryptographic Parameters"),
                    0x0000_003F => Some("Unsupported Protocol Version"),
                    0x0000_0040 => Some("Wrapping Object Archived"),
                    0x0000_0041 => Some("Wrapping Object Destroyed"),
                    0x0000_0042 => Some("Wrapping Object Not Found"),
                    0x0000_0043 => Some("Wrong Key Lifecycle State"),
                    0x0000_0044 => Some("Protection Storage Unavailable"),
                    0x0000_0045 => Some("PKCS#11 Codec Error"),
                    0x0000_0046 => Some("PKCS#11 Invalid Function"),
                    0x0000_0047 => Some("PKCS#11 Invalid Interface"),
                    0x0000_0048 => Some("Private Protection Storage Unavailable"),
                    0x0000_0049 => Some("Public Protection Storage Unavailable"),
                    0x0000_004A => Some("Unknown Object Group"),
                    0x0000_004B => Some("Constraint Violation"),
                    0x0000_004C => Some("Duplicate Process Request"),
                    0x0000_0100 => Some("General Failure"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Result Status Enumeration.
        pub mod result_status {
            /// `Success`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-SUCCESS-00000000; KMIPKIT-SRC-spec §11.47.
            pub const SUCCESS: u32 = 0x0000_0000;
            /// `Operation Failed`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-FAILED-00000001; KMIPKIT-SRC-spec §11.47.
            pub const OPERATION_FAILED: u32 = 0x0000_0001;
            /// `Operation Pending`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-PENDING-00000002; KMIPKIT-SRC-spec §11.47.
            pub const OPERATION_PENDING: u32 = 0x0000_0002;
            /// `Operation Undone`; KMIPKIT-ELEM-ENUM-VALUE-RESULT-STATUS-OPERATION-UNDONE-00000003; KMIPKIT-SRC-spec §11.47.
            pub const OPERATION_UNDONE: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0000 => Some("Success"),
                    0x0000_0001 => Some("Operation Failed"),
                    0x0000_0002 => Some("Operation Pending"),
                    0x0000_0003 => Some("Operation Undone"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Revocation Reason Code Enumeration.
        pub mod revocation_reason_code {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.48.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Key Compromise`; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-KEY-COMPROMISE-00000002; KMIPKIT-SRC-spec §11.48.
            pub const KEY_COMPROMISE: u32 = 0x0000_0002;
            /// `CA Compromise`; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CA-COMPROMISE-00000003; KMIPKIT-SRC-spec §11.48.
            pub const CA_COMPROMISE: u32 = 0x0000_0003;
            /// `Affiliation Changed`; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-AFFILIATION-CHANGED-00000004; KMIPKIT-SRC-spec §11.48.
            pub const AFFILIATION_CHANGED: u32 = 0x0000_0004;
            /// `Superseded`; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-SUPERSEDED-00000005; KMIPKIT-SRC-spec §11.48.
            pub const SUPERSEDED: u32 = 0x0000_0005;
            /// `Cessation of Operation`; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-CESSATION-OF-OPERATION-00000006; KMIPKIT-SRC-spec §11.48.
            pub const CESSATION_OF_OPERATION: u32 = 0x0000_0006;
            /// `Privilege Withdrawn`; KMIPKIT-ELEM-ENUM-VALUE-REVOCATION-REASON-CODE-PRIVILEGE-WITHDRAWN-00000007; KMIPKIT-SRC-spec §11.48.
            pub const PRIVILEGE_WITHDRAWN: u32 = 0x0000_0007;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Key Compromise"),
                    0x0000_0003 => Some("CA Compromise"),
                    0x0000_0004 => Some("Affiliation Changed"),
                    0x0000_0005 => Some("Superseded"),
                    0x0000_0006 => Some("Cessation of Operation"),
                    0x0000_0007 => Some("Privilege Withdrawn"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP RNG Algorithm Enumeration.
        pub mod rng_algorithm {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.49.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `FIPS 186-2`; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-FIPS-186-2-00000002; KMIPKIT-SRC-spec §11.49.
            pub const FIPS_186_2: u32 = 0x0000_0002;
            /// `DRBG`; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-DRBG-00000003; KMIPKIT-SRC-spec §11.49.
            pub const DRBG: u32 = 0x0000_0003;
            /// `NRBG`; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-NRBG-00000004; KMIPKIT-SRC-spec §11.49.
            pub const NRBG: u32 = 0x0000_0004;
            /// `ANSI X9.31`; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-31-00000005; KMIPKIT-SRC-spec §11.49.
            pub const ANSI_X9_31: u32 = 0x0000_0005;
            /// `ANSI X9.62`; KMIPKIT-ELEM-ENUM-VALUE-RNG-ALGORITHM-ANSI-X9-62-00000006; KMIPKIT-SRC-spec §11.49.
            pub const ANSI_X9_62: u32 = 0x0000_0006;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("FIPS 186-2"),
                    0x0000_0003 => Some("DRBG"),
                    0x0000_0004 => Some("NRBG"),
                    0x0000_0005 => Some("ANSI X9.31"),
                    0x0000_0006 => Some("ANSI X9.62"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP RNG Mode Enumeration.
        pub mod rng_mode {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.50.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Shared Instantiation`; KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-SHARED-INSTANTIATION-00000002; KMIPKIT-SRC-spec §11.50.
            pub const SHARED_INSTANTIATION: u32 = 0x0000_0002;
            /// `Non-Shared Instantiation`; KMIPKIT-ELEM-ENUM-VALUE-RNG-MODE-NON-SHARED-INSTANTIATION-00000003; KMIPKIT-SRC-spec §11.50.
            pub const NON_SHARED_INSTANTIATION: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Shared Instantiation"),
                    0x0000_0003 => Some("Non-Shared Instantiation"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Rotate Name Type Enumeration.
        pub mod rotate_name_type {
            /// `Uninterpreted Text String`; KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-UNINTERPRETED-TEXT-STRING-00000001; KMIPKIT-SRC-spec §11.51.
            pub const UNINTERPRETED_TEXT_STRING: u32 = 0x0000_0001;
            /// `URI`; KMIPKIT-ELEM-ENUM-VALUE-ROTATE-NAME-TYPE-URI-00000002; KMIPKIT-SRC-spec §11.51.
            pub const URI: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Uninterpreted Text String"),
                    0x0000_0002 => Some("URI"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Secret Data Type Enumeration.
        pub mod secret_data_type {
            /// `Password`; KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-PASSWORD-00000001; KMIPKIT-SRC-spec §11.52.
            pub const PASSWORD: u32 = 0x0000_0001;
            /// `Seed`; KMIPKIT-ELEM-ENUM-VALUE-SECRET-DATA-TYPE-SEED-00000002; KMIPKIT-SRC-spec §11.52.
            pub const SEED: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Password"),
                    0x0000_0002 => Some("Seed"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Shredding Algorithm Enumeration.
        pub mod shredding_algorithm {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.53.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Cryptographic`; KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-CRYPTOGRAPHIC-00000002; KMIPKIT-SRC-spec §11.53.
            pub const CRYPTOGRAPHIC: u32 = 0x0000_0002;
            /// `Unsupported`; KMIPKIT-ELEM-ENUM-VALUE-SHREDDING-ALGORITHM-UNSUPPORTED-00000003; KMIPKIT-SRC-spec §11.53.
            pub const UNSUPPORTED: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Cryptographic"),
                    0x0000_0003 => Some("Unsupported"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Split Key Method Enumeration.
        pub mod split_key_method {
            /// `XOR`; KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-XOR-00000001; KMIPKIT-SRC-spec §11.54.
            pub const XOR: u32 = 0x0000_0001;
            /// `Polynomial Sharing GF (2¹⁶)`; KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000002; KMIPKIT-SRC-spec §11.54.
            pub const POLYNOMIAL_SHARING_GF_216: u32 = 0x0000_0002;
            /// `Polynomial Sharing Prime Field`; KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-PRIME-FIELD-00000003; KMIPKIT-SRC-spec §11.54.
            pub const POLYNOMIAL_SHARING_PRIME_FIELD: u32 = 0x0000_0003;
            /// `Polynomial Sharing GF (2⁸)`; KMIPKIT-ELEM-ENUM-VALUE-SPLIT-KEY-METHOD-POLYNOMIAL-SHARING-GF-2-00000004; KMIPKIT-SRC-spec §11.54.
            pub const POLYNOMIAL_SHARING_GF_28: u32 = 0x0000_0004;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("XOR"),
                    0x0000_0002 => Some("Polynomial Sharing GF (2¹⁶)"),
                    0x0000_0003 => Some("Polynomial Sharing Prime Field"),
                    0x0000_0004 => Some("Polynomial Sharing GF (2⁸)"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP State Enumeration.
        pub mod state {
            /// `Pre-Active`; KMIPKIT-ELEM-ENUM-VALUE-STATE-PRE-ACTIVE-00000001; KMIPKIT-SRC-spec §11.55.
            pub const PRE_ACTIVE: u32 = 0x0000_0001;
            /// `Active`; KMIPKIT-ELEM-ENUM-VALUE-STATE-ACTIVE-00000002; KMIPKIT-SRC-spec §11.55.
            pub const ACTIVE: u32 = 0x0000_0002;
            /// `Deactivated`; KMIPKIT-ELEM-ENUM-VALUE-STATE-DEACTIVATED-00000003; KMIPKIT-SRC-spec §11.55.
            pub const DEACTIVATED: u32 = 0x0000_0003;
            /// `Compromised`; KMIPKIT-ELEM-ENUM-VALUE-STATE-COMPROMISED-00000004; KMIPKIT-SRC-spec §11.55.
            pub const COMPROMISED: u32 = 0x0000_0004;
            /// `Destroyed`; KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-00000005; KMIPKIT-SRC-spec §11.55.
            pub const DESTROYED: u32 = 0x0000_0005;
            /// `Destroyed Compromised`; KMIPKIT-ELEM-ENUM-VALUE-STATE-DESTROYED-COMPROMISED-00000006; KMIPKIT-SRC-spec §11.55.
            pub const DESTROYED_COMPROMISED: u32 = 0x0000_0006;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Pre-Active"),
                    0x0000_0002 => Some("Active"),
                    0x0000_0003 => Some("Deactivated"),
                    0x0000_0004 => Some("Compromised"),
                    0x0000_0005 => Some("Destroyed"),
                    0x0000_0006 => Some("Destroyed Compromised"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Tag Enumeration.
        pub mod tag {
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(_raw: u32) -> Option<&'static str> {
                None
            }
        }

        /// Values for the KMIP Ticket Type Enumeration.
        pub mod ticket_type {
            /// `Login`; KMIPKIT-ELEM-ENUM-VALUE-TICKET-TYPE-LOGIN-00000001; KMIPKIT-SRC-spec §11.57.
            pub const LOGIN: u32 = 0x0000_0001;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Login"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Unique Identifier Enumeration.
        pub mod unique_identifier {
            /// `ID Placeholder`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-ID-PLACEHOLDER-00000001; KMIPKIT-SRC-spec §11.58.
            pub const ID_PLACEHOLDER: u32 = 0x0000_0001;
            /// `Certify`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CERTIFY-00000002; KMIPKIT-SRC-spec §11.58.
            pub const CERTIFY: u32 = 0x0000_0002;
            /// `Create`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-00000003; KMIPKIT-SRC-spec §11.58.
            pub const CREATE: u32 = 0x0000_0003;
            /// `Create Key Pair`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-00000004; KMIPKIT-SRC-spec §11.58.
            pub const CREATE_KEY_PAIR: u32 = 0x0000_0004;
            /// `Create Key Pair Private Key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PRIVATE-KEY-00000005; KMIPKIT-SRC-spec §11.58.
            pub const CREATE_KEY_PAIR_PRIVATE_KEY: u32 = 0x0000_0005;
            /// `Create Key Pair Public Key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-KEY-PAIR-PUBLIC-KEY-00000006; KMIPKIT-SRC-spec §11.58.
            pub const CREATE_KEY_PAIR_PUBLIC_KEY: u32 = 0x0000_0006;
            /// `Create Split Key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-CREATE-SPLIT-KEY-00000007; KMIPKIT-SRC-spec §11.58.
            pub const CREATE_SPLIT_KEY: u32 = 0x0000_0007;
            /// `Derive Key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-DERIVE-KEY-00000008; KMIPKIT-SRC-spec §11.58.
            pub const DERIVE_KEY: u32 = 0x0000_0008;
            /// `Import`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-IMPORT-00000009; KMIPKIT-SRC-spec §11.58.
            pub const IMPORT: u32 = 0x0000_0009;
            /// `Join Split Key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-JOIN-SPLIT-KEY-0000000A; KMIPKIT-SRC-spec §11.58.
            pub const JOIN_SPLIT_KEY: u32 = 0x0000_000A;
            /// `Locate`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-LOCATE-0000000B; KMIPKIT-SRC-spec §11.58.
            pub const LOCATE: u32 = 0x0000_000B;
            /// `Register`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-REGISTER-0000000C; KMIPKIT-SRC-spec §11.58.
            pub const REGISTER: u32 = 0x0000_000C;
            /// `Re-key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-0000000D; KMIPKIT-SRC-spec §11.58.
            pub const RE_KEY: u32 = 0x0000_000D;
            /// `Re-certify`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-CERTIFY-0000000E; KMIPKIT-SRC-spec §11.58.
            pub const RE_CERTIFY: u32 = 0x0000_000E;
            /// `Re-key Key Pair`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-0000000F; KMIPKIT-SRC-spec §11.58.
            pub const RE_KEY_KEY_PAIR: u32 = 0x0000_000F;
            /// `Re-key Key Pair Private Key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PRIVATE-KEY-00000010; KMIPKIT-SRC-spec §11.58.
            pub const RE_KEY_KEY_PAIR_PRIVATE_KEY: u32 = 0x0000_0010;
            /// `Re-key Key Pair Public Key`; KMIPKIT-ELEM-ENUM-VALUE-UNIQUE-IDENTIFIER-RE-KEY-KEY-PAIR-PUBLIC-KEY-00000011; KMIPKIT-SRC-spec §11.58.
            pub const RE_KEY_KEY_PAIR_PUBLIC_KEY: u32 = 0x0000_0011;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("ID Placeholder"),
                    0x0000_0002 => Some("Certify"),
                    0x0000_0003 => Some("Create"),
                    0x0000_0004 => Some("Create Key Pair"),
                    0x0000_0005 => Some("Create Key Pair Private Key"),
                    0x0000_0006 => Some("Create Key Pair Public Key"),
                    0x0000_0007 => Some("Create Split Key"),
                    0x0000_0008 => Some("Derive Key"),
                    0x0000_0009 => Some("Import"),
                    0x0000_000A => Some("Join Split Key"),
                    0x0000_000B => Some("Locate"),
                    0x0000_000C => Some("Register"),
                    0x0000_000D => Some("Re-key"),
                    0x0000_000E => Some("Re-certify"),
                    0x0000_000F => Some("Re-key Key Pair"),
                    0x0000_0010 => Some("Re-key Key Pair Private Key"),
                    0x0000_0011 => Some("Re-key Key Pair Public Key"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Unwrap Mode Enumeration.
        pub mod unwrap_mode {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.59.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Processed`; KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-PROCESSED-00000002; KMIPKIT-SRC-spec §11.59.
            pub const PROCESSED: u32 = 0x0000_0002;
            /// `Not Processed`; KMIPKIT-ELEM-ENUM-VALUE-UNWRAP-MODE-NOT-PROCESSED-00000003; KMIPKIT-SRC-spec §11.59.
            pub const NOT_PROCESSED: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Processed"),
                    0x0000_0003 => Some("Not Processed"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Usage Limits Unit Enumeration.
        pub mod usage_limits_unit {
            /// `Byte`; KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-BYTE-00000001; KMIPKIT-SRC-spec §11.60.
            pub const BYTE: u32 = 0x0000_0001;
            /// `Object`; KMIPKIT-ELEM-ENUM-VALUE-USAGE-LIMITS-UNIT-OBJECT-00000002; KMIPKIT-SRC-spec §11.60.
            pub const OBJECT: u32 = 0x0000_0002;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Byte"),
                    0x0000_0002 => Some("Object"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Validation Authority Type Enumeration.
        pub mod validation_authority_type {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.63.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `NIST CMVP`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-NIST-CMVP-00000002; KMIPKIT-SRC-spec §11.63.
            pub const NIST_CMVP: u32 = 0x0000_0002;
            /// `Common Criteria`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-AUTHORITY-TYPE-COMMON-CRITERIA-00000003; KMIPKIT-SRC-spec §11.63.
            pub const COMMON_CRITERIA: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("NIST CMVP"),
                    0x0000_0003 => Some("Common Criteria"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Validation Type Enumeration.
        pub mod validation_type {
            /// `Unspecified`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-UNSPECIFIED-00000001; KMIPKIT-SRC-spec §11.64.
            pub const UNSPECIFIED: u32 = 0x0000_0001;
            /// `Hardware`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HARDWARE-00000002; KMIPKIT-SRC-spec §11.64.
            pub const HARDWARE: u32 = 0x0000_0002;
            /// `Software`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-SOFTWARE-00000003; KMIPKIT-SRC-spec §11.64.
            pub const SOFTWARE: u32 = 0x0000_0003;
            /// `Firmware`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-FIRMWARE-00000004; KMIPKIT-SRC-spec §11.64.
            pub const FIRMWARE: u32 = 0x0000_0004;
            /// `Hybrid`; KMIPKIT-ELEM-ENUM-VALUE-VALIDATION-TYPE-HYBRID-00000005; KMIPKIT-SRC-spec §11.64.
            pub const HYBRID: u32 = 0x0000_0005;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Unspecified"),
                    0x0000_0002 => Some("Hardware"),
                    0x0000_0003 => Some("Software"),
                    0x0000_0004 => Some("Firmware"),
                    0x0000_0005 => Some("Hybrid"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Validity Indicator Enumeration.
        pub mod validity_indicator {
            /// `Valid`; KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-VALID-00000001; KMIPKIT-SRC-spec §11.61.
            pub const VALID: u32 = 0x0000_0001;
            /// `Invalid`; KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-INVALID-00000002; KMIPKIT-SRC-spec §11.61.
            pub const INVALID: u32 = 0x0000_0002;
            /// `Unknown`; KMIPKIT-ELEM-ENUM-VALUE-VALIDITY-INDICATOR-UNKNOWN-00000003; KMIPKIT-SRC-spec §11.61.
            pub const UNKNOWN: u32 = 0x0000_0003;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Valid"),
                    0x0000_0002 => Some("Invalid"),
                    0x0000_0003 => Some("Unknown"),
                    _ => None,
                }
            }
        }

        /// Values for the KMIP Wrapping Method Enumeration.
        pub mod wrapping_method {
            /// `Encrypt`; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-00000001; KMIPKIT-SRC-spec §11.62.
            pub const ENCRYPT: u32 = 0x0000_0001;
            /// `MAC/sign`; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-00000002; KMIPKIT-SRC-spec §11.62.
            pub const MAC_SIGN: u32 = 0x0000_0002;
            /// `Encrypt then MAC/sign`; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-ENCRYPT-THEN-MAC-SIGN-00000003; KMIPKIT-SRC-spec §11.62.
            pub const ENCRYPT_THEN_MAC_SIGN: u32 = 0x0000_0003;
            /// `MAC/sign then encrypt`; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-MAC-SIGN-THEN-ENCRYPT-00000004; KMIPKIT-SRC-spec §11.62.
            pub const MAC_SIGN_THEN_ENCRYPT: u32 = 0x0000_0004;
            /// `TR-31`; KMIPKIT-ELEM-ENUM-VALUE-WRAPPING-METHOD-TR-31-00000005; KMIPKIT-SRC-spec §11.62.
            pub const TR_31: u32 = 0x0000_0005;
            /// Returns the assigned catalog name for a known raw value.
            #[must_use]
            pub const fn name(raw: u32) -> Option<&'static str> {
                match raw {
                    0x0000_0001 => Some("Encrypt"),
                    0x0000_0002 => Some("MAC/sign"),
                    0x0000_0003 => Some("Encrypt then MAC/sign"),
                    0x0000_0004 => Some("MAC/sign then encrypt"),
                    0x0000_0005 => Some("TR-31"),
                    _ => None,
                }
            }
        }
    }

    /// Assigned KMIP 2.1 Bitmask values.
    pub mod bitmask {
        /// Values for the KMIP Cryptographic Usage Mask Bitmask.
        pub mod cryptographic_usage_mask {
            /// `Sign`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-SIGN-00000001; KMIPKIT-SRC-spec §12.1.
            pub const SIGN: u32 = 0x0000_0001;
            /// `Verify`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-VERIFY-00000002; KMIPKIT-SRC-spec §12.1.
            pub const VERIFY: u32 = 0x0000_0002;
            /// `Encrypt`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-ENCRYPT-00000004; KMIPKIT-SRC-spec §12.1.
            pub const ENCRYPT: u32 = 0x0000_0004;
            /// `Decrypt`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DECRYPT-00000008; KMIPKIT-SRC-spec §12.1.
            pub const DECRYPT: u32 = 0x0000_0008;
            /// `Wrap Key`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-WRAP-KEY-00000010; KMIPKIT-SRC-spec §12.1.
            pub const WRAP_KEY: u32 = 0x0000_0010;
            /// `Unwrap Key`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNWRAP-KEY-00000020; KMIPKIT-SRC-spec §12.1.
            pub const UNWRAP_KEY: u32 = 0x0000_0020;
            /// `MAC Generate`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-GENERATE-00000080; KMIPKIT-SRC-spec §12.1.
            pub const MAC_GENERATE: u32 = 0x0000_0080;
            /// `MAC Verify`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-MAC-VERIFY-00000100; KMIPKIT-SRC-spec §12.1.
            pub const MAC_VERIFY: u32 = 0x0000_0100;
            /// `Derive Key`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-DERIVE-KEY-00000200; KMIPKIT-SRC-spec §12.1.
            pub const DERIVE_KEY: u32 = 0x0000_0200;
            /// `Key Agreement`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-KEY-AGREEMENT-00000800; KMIPKIT-SRC-spec §12.1.
            pub const KEY_AGREEMENT: u32 = 0x0000_0800;
            /// `Certificate Sign`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CERTIFICATE-SIGN-00001000; KMIPKIT-SRC-spec §12.1.
            pub const CERTIFICATE_SIGN: u32 = 0x0000_1000;
            /// `CRL Sign`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-CRL-SIGN-00002000; KMIPKIT-SRC-spec §12.1.
            pub const CRL_SIGN: u32 = 0x0000_2000;
            /// `Authenticate`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-AUTHENTICATE-00100000; KMIPKIT-SRC-spec §12.1.
            pub const AUTHENTICATE: u32 = 0x0010_0000;
            /// `Unrestricted`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-UNRESTRICTED-00200000; KMIPKIT-SRC-spec §12.1.
            pub const UNRESTRICTED: u32 = 0x0020_0000;
            /// `FPE Encrypt`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-ENCRYPT-00400000; KMIPKIT-SRC-spec §12.1.
            pub const FPE_ENCRYPT: u32 = 0x0040_0000;
            /// `FPE Decrypt`; KMIPKIT-ELEM-BITMASK-VALUE-CRYPTOGRAPHIC-USAGE-MASK-FPE-DECRYPT-00800000; KMIPKIT-SRC-spec §12.1.
            pub const FPE_DECRYPT: u32 = 0x0080_0000;
            /// Mask containing every assigned bit in this group.
            pub const KNOWN_MASK: u32 = 0x00F0_3BBF;
            /// Returns only the bits assigned by KMIP 2.1.
            #[must_use]
            pub const fn known_bits(raw: u32) -> u32 {
                raw & KNOWN_MASK
            }
            /// Returns all raw bits not assigned by KMIP 2.1.
            #[must_use]
            pub const fn unknown_bits(raw: u32) -> u32 {
                raw & !KNOWN_MASK
            }
            /// Returns assigned bit names in ascending numeric order.
            pub fn names(raw: u32) -> impl Iterator<Item = &'static str> {
                [
                    (0x0000_0001, "Sign"),
                    (0x0000_0002, "Verify"),
                    (0x0000_0004, "Encrypt"),
                    (0x0000_0008, "Decrypt"),
                    (0x0000_0010, "Wrap Key"),
                    (0x0000_0020, "Unwrap Key"),
                    (0x0000_0080, "MAC Generate"),
                    (0x0000_0100, "MAC Verify"),
                    (0x0000_0200, "Derive Key"),
                    (0x0000_0800, "Key Agreement"),
                    (0x0000_1000, "Certificate Sign"),
                    (0x0000_2000, "CRL Sign"),
                    (0x0010_0000, "Authenticate"),
                    (0x0020_0000, "Unrestricted"),
                    (0x0040_0000, "FPE Encrypt"),
                    (0x0080_0000, "FPE Decrypt"),
                ]
                .into_iter()
                .filter_map(move |(bit, name)| (raw & bit != 0).then_some(name))
            }
        }

        /// Values for the KMIP Protection Storage Mask Bitmask.
        pub mod protection_storage_mask {
            /// `Software`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SOFTWARE-00000001; KMIPKIT-SRC-spec §12.2.
            pub const SOFTWARE: u32 = 0x0000_0001;
            /// `Hardware`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HARDWARE-00000002; KMIPKIT-SRC-spec §12.2.
            pub const HARDWARE: u32 = 0x0000_0002;
            /// `On Processor`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PROCESSOR-00000004; KMIPKIT-SRC-spec §12.2.
            pub const ON_PROCESSOR: u32 = 0x0000_0004;
            /// `On System`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-SYSTEM-00000008; KMIPKIT-SRC-spec §12.2.
            pub const ON_SYSTEM: u32 = 0x0000_0008;
            /// `Off System`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-SYSTEM-00000010; KMIPKIT-SRC-spec §12.2.
            pub const OFF_SYSTEM: u32 = 0x0000_0010;
            /// `Hypervisor`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-HYPERVISOR-00000020; KMIPKIT-SRC-spec §12.2.
            pub const HYPERVISOR: u32 = 0x0000_0020;
            /// `Operating System`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OPERATING-SYSTEM-00000040; KMIPKIT-SRC-spec §12.2.
            pub const OPERATING_SYSTEM: u32 = 0x0000_0040;
            /// `Container`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-CONTAINER-00000080; KMIPKIT-SRC-spec §12.2.
            pub const CONTAINER: u32 = 0x0000_0080;
            /// `On Premises`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-ON-PREMISES-00000100; KMIPKIT-SRC-spec §12.2.
            pub const ON_PREMISES: u32 = 0x0000_0100;
            /// `Off Premises`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OFF-PREMISES-00000200; KMIPKIT-SRC-spec §12.2.
            pub const OFF_PREMISES: u32 = 0x0000_0200;
            /// `Self Managed`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SELF-MANAGED-00000400; KMIPKIT-SRC-spec §12.2.
            pub const SELF_MANAGED: u32 = 0x0000_0400;
            /// `Outsourced`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-OUTSOURCED-00000800; KMIPKIT-SRC-spec §12.2.
            pub const OUTSOURCED: u32 = 0x0000_0800;
            /// `Validated`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-VALIDATED-00001000; KMIPKIT-SRC-spec §12.2.
            pub const VALIDATED: u32 = 0x0000_1000;
            /// `Same Jurisdiction`; KMIPKIT-ELEM-BITMASK-VALUE-PROTECTION-STORAGE-MASK-SAME-JURISDICTION-00002000; KMIPKIT-SRC-spec §12.2.
            pub const SAME_JURISDICTION: u32 = 0x0000_2000;
            /// Mask containing every assigned bit in this group.
            pub const KNOWN_MASK: u32 = 0x0000_3FFF;
            /// Returns only the bits assigned by KMIP 2.1.
            #[must_use]
            pub const fn known_bits(raw: u32) -> u32 {
                raw & KNOWN_MASK
            }
            /// Returns all raw bits not assigned by KMIP 2.1.
            #[must_use]
            pub const fn unknown_bits(raw: u32) -> u32 {
                raw & !KNOWN_MASK
            }
            /// Returns assigned bit names in ascending numeric order.
            pub fn names(raw: u32) -> impl Iterator<Item = &'static str> {
                [
                    (0x0000_0001, "Software"),
                    (0x0000_0002, "Hardware"),
                    (0x0000_0004, "On Processor"),
                    (0x0000_0008, "On System"),
                    (0x0000_0010, "Off System"),
                    (0x0000_0020, "Hypervisor"),
                    (0x0000_0040, "Operating System"),
                    (0x0000_0080, "Container"),
                    (0x0000_0100, "On Premises"),
                    (0x0000_0200, "Off Premises"),
                    (0x0000_0400, "Self Managed"),
                    (0x0000_0800, "Outsourced"),
                    (0x0000_1000, "Validated"),
                    (0x0000_2000, "Same Jurisdiction"),
                ]
                .into_iter()
                .filter_map(move |(bit, name)| (raw & bit != 0).then_some(name))
            }
        }

        /// Values for the KMIP Storage Status Mask Bitmask.
        pub mod storage_status_mask {
            /// `On-line storage`; KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ON-LINE-STORAGE-00000001; KMIPKIT-SRC-spec §12.3.
            pub const ON_LINE_STORAGE: u32 = 0x0000_0001;
            /// `Archival storage`; KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-ARCHIVAL-STORAGE-00000002; KMIPKIT-SRC-spec §12.3.
            pub const ARCHIVAL_STORAGE: u32 = 0x0000_0002;
            /// `Destroyed storage`; KMIPKIT-ELEM-BITMASK-VALUE-STORAGE-STATUS-MASK-DESTROYED-STORAGE-00000004; KMIPKIT-SRC-spec §12.3.
            pub const DESTROYED_STORAGE: u32 = 0x0000_0004;
            /// Mask containing every assigned bit in this group.
            pub const KNOWN_MASK: u32 = 0x0000_0007;
            /// Returns only the bits assigned by KMIP 2.1.
            #[must_use]
            pub const fn known_bits(raw: u32) -> u32 {
                raw & KNOWN_MASK
            }
            /// Returns all raw bits not assigned by KMIP 2.1.
            #[must_use]
            pub const fn unknown_bits(raw: u32) -> u32 {
                raw & !KNOWN_MASK
            }
            /// Returns assigned bit names in ascending numeric order.
            pub fn names(raw: u32) -> impl Iterator<Item = &'static str> {
                [
                    (0x0000_0001, "On-line storage"),
                    (0x0000_0002, "Archival storage"),
                    (0x0000_0004, "Destroyed storage"),
                ]
                .into_iter()
                .filter_map(move |(bit, name)| (raw & bit != 0).then_some(name))
            }
        }
    }
    /// Exact assigned values added by later KMIP editions.
    pub mod versions {
        /// Values assigned in KMIP v2.1.
        pub mod v2_1 {
            pub mod enumeration {
                /// KMIP Operation values from §11.36, Table 470.
                pub mod operation {
                    /// `Set Defaults`; kmip-2.1 §11.36, Table 470.
                    pub const SET_DEFAULTS: u32 = 0x0000_0036;
                    /// `Set Constraints`; kmip-2.1 §11.36, Table 470.
                    pub const SET_CONSTRAINTS: u32 = 0x0000_0037;
                    /// `Get Constraints`; kmip-2.1 §11.36, Table 470.
                    pub const GET_CONSTRAINTS: u32 = 0x0000_0038;
                    /// `Query Asynchronous Requests`; kmip-2.1 §11.36, Table 470.
                    pub const QUERY_ASYNCHRONOUS_REQUESTS: u32 = 0x0000_0039;
                    /// `Process`; kmip-2.1 §11.36, Table 470.
                    pub const PROCESS: u32 = 0x0000_003A;
                    /// `Ping`; kmip-2.1 §11.36, Table 470.
                    pub const PING: u32 = 0x0000_003B;
                    /// Returns the assigned name for a known value.
                    #[must_use]
                    pub const fn name(raw: u32) -> Option<&'static str> {
                        match raw {
                            0x0000_0036 => Some("Set Defaults"),
                            0x0000_0037 => Some("Set Constraints"),
                            0x0000_0038 => Some("Get Constraints"),
                            0x0000_0039 => Some("Query Asynchronous Requests"),
                            0x0000_003A => Some("Process"),
                            0x0000_003B => Some("Ping"),
                            _ => None,
                        }
                    }
                }
            }
        }
        /// Values assigned in KMIP v3.0.
        pub mod v3_0 {
            pub mod enumeration {
                /// KMIP Operation values from §11.37, Table 583.
                pub mod operation {
                    /// `Create Group`; kmip-3.0 §11.37, Table 583.
                    pub const CREATE_GROUP: u32 = 0x0000_003C;
                    /// `Obliterate`; kmip-3.0 §11.37, Table 583.
                    pub const OBLITERATE: u32 = 0x0000_003D;
                    /// `Create User`; kmip-3.0 §11.37, Table 583.
                    pub const CREATE_USER: u32 = 0x0000_003E;
                    /// `Create Credential`; kmip-3.0 §11.37, Table 583.
                    pub const CREATE_CREDENTIAL: u32 = 0x0000_003F;
                    /// `Deactivate`; kmip-3.0 §11.37, Table 583.
                    pub const DEACTIVATE: u32 = 0x0000_0040;
                    /// `Encapsulate`; kmip-3.0 §11.37, Table 583.
                    pub const ENCAPSULATE: u32 = 0x0000_0041;
                    /// `Decapsulate`; kmip-3.0 §11.37, Table 583.
                    pub const DECAPSULATE: u32 = 0x0000_0042;
                    /// Returns the assigned name for a known value.
                    #[must_use]
                    pub const fn name(raw: u32) -> Option<&'static str> {
                        match raw {
                            0x0000_003C => Some("Create Group"),
                            0x0000_003D => Some("Obliterate"),
                            0x0000_003E => Some("Create User"),
                            0x0000_003F => Some("Create Credential"),
                            0x0000_0040 => Some("Deactivate"),
                            0x0000_0041 => Some("Encapsulate"),
                            0x0000_0042 => Some("Decapsulate"),
                            _ => None,
                        }
                    }
                }
                /// KMIP Object Class values from §11.34, Table 581.
                pub mod object_class {
                    /// `User`; kmip-3.0 §11.34, Table 581.
                    pub const USER: u32 = 0x0000_0001;
                    /// `System`; kmip-3.0 §11.34, Table 581.
                    pub const SYSTEM: u32 = 0x0000_0002;
                    /// Returns the assigned name for a known value.
                    #[must_use]
                    pub const fn name(raw: u32) -> Option<&'static str> {
                        match raw {
                            0x0000_0001 => Some("User"),
                            0x0000_0002 => Some("System"),
                            _ => None,
                        }
                    }
                }
                /// KMIP Object Type values from §11.35, Table 582.
                pub mod object_type {
                    /// `User`; kmip-3.0 §11.35, Table 582.
                    pub const USER: u32 = 0x0000_000B;
                    /// `Group`; kmip-3.0 §11.35, Table 582.
                    pub const GROUP: u32 = 0x0000_000C;
                    /// `Password Credential`; kmip-3.0 §11.35, Table 582.
                    pub const PASSWORD_CREDENTIAL: u32 = 0x0000_000D;
                    /// `Device Credential`; kmip-3.0 §11.35, Table 582.
                    pub const DEVICE_CREDENTIAL: u32 = 0x0000_000E;
                    /// `One Time Password Credential`; kmip-3.0 §11.35, Table 582.
                    pub const ONE_TIME_PASSWORD_CREDENTIAL: u32 = 0x0000_000F;
                    /// `Hashed Password Credential`; kmip-3.0 §11.35, Table 582.
                    pub const HASHED_PASSWORD_CREDENTIAL: u32 = 0x0000_0010;
                    /// Returns the assigned name for a known value.
                    #[must_use]
                    pub const fn name(raw: u32) -> Option<&'static str> {
                        match raw {
                            0x0000_000B => Some("User"),
                            0x0000_000C => Some("Group"),
                            0x0000_000D => Some("Password Credential"),
                            0x0000_000E => Some("Device Credential"),
                            0x0000_000F => Some("One Time Password Credential"),
                            0x0000_0010 => Some("Hashed Password Credential"),
                            _ => None,
                        }
                    }
                }
            }
        }
    }
}
