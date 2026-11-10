/* Generated KMIP 2.1 named-value lookup helpers. */
#ifndef KMIPKIT_NAMED_VALUE_HELPERS_GENERATED_H
#define KMIPKIT_NAMED_VALUE_HELPERS_GENERATED_H

#include <stdint.h>

/* Opaque namespace marker for the KMIP named-value header surface. */
typedef struct kmipkit_named_values_namespace_t kmipkit_named_values_namespace_t;
typedef uint32_t kmipkit_enumeration_adjustment_type_t;
typedef uint32_t kmipkit_enumeration_alternative_name_type_t;
typedef uint32_t kmipkit_enumeration_asynchronous_indicator_t;
typedef uint32_t kmipkit_enumeration_attestation_type_t;
typedef uint32_t kmipkit_enumeration_batch_error_continuation_option_t;
typedef uint32_t kmipkit_enumeration_block_cipher_mode_t;
typedef uint32_t kmipkit_enumeration_cancellation_result_t;
typedef uint32_t kmipkit_enumeration_certificate_request_type_t;
typedef uint32_t kmipkit_enumeration_certificate_type_t;
typedef uint32_t kmipkit_enumeration_client_registration_method_t;
typedef uint32_t kmipkit_enumeration_credential_type_t;
typedef uint32_t kmipkit_enumeration_cryptographic_algorithm_t;
typedef uint32_t kmipkit_enumeration_data_t;
typedef uint32_t kmipkit_enumeration_derivation_method_t;
typedef uint32_t kmipkit_enumeration_destroy_action_t;
typedef uint32_t kmipkit_enumeration_digital_signature_algorithm_t;
typedef uint32_t kmipkit_enumeration_drbg_algorithm_t;
typedef uint32_t kmipkit_enumeration_encoding_option_t;
typedef uint32_t kmipkit_enumeration_endpoint_role_t;
typedef uint32_t kmipkit_enumeration_fips186_variation_t;
typedef uint32_t kmipkit_enumeration_hashing_algorithm_t;
typedef uint32_t kmipkit_enumeration_interop_function_t;
typedef uint32_t kmipkit_enumeration_item_type_t;
typedef uint32_t kmipkit_enumeration_key_compression_type_t;
typedef uint32_t kmipkit_enumeration_key_format_type_t;
typedef uint32_t kmipkit_enumeration_key_role_type_t;
typedef uint32_t kmipkit_enumeration_key_value_location_type_t;
typedef uint32_t kmipkit_enumeration_key_wrap_type_t;
typedef uint32_t kmipkit_enumeration_link_type_t;
typedef uint32_t kmipkit_enumeration_mask_generator_t;
typedef uint32_t kmipkit_enumeration_name_type_t;
typedef uint32_t kmipkit_enumeration_nist_key_type_t;
typedef uint32_t kmipkit_enumeration_object_group_member_t;
typedef uint32_t kmipkit_enumeration_object_type_t;
typedef uint32_t kmipkit_enumeration_opaque_data_type_t;
typedef uint32_t kmipkit_enumeration_operation_t;
typedef uint32_t kmipkit_enumeration_padding_method_t;
typedef uint32_t kmipkit_enumeration_pkcs_11_function_t;
typedef uint32_t kmipkit_enumeration_pkcs_11_return_code_t;
typedef uint32_t kmipkit_enumeration_processing_stage_t;
typedef uint32_t kmipkit_enumeration_profile_name_t;
typedef uint32_t kmipkit_enumeration_protection_level_t;
typedef uint32_t kmipkit_enumeration_put_function_t;
typedef uint32_t kmipkit_enumeration_query_function_t;
typedef uint32_t kmipkit_enumeration_recommended_curve_t;
typedef uint32_t kmipkit_enumeration_result_reason_t;
typedef uint32_t kmipkit_enumeration_result_status_t;
typedef uint32_t kmipkit_enumeration_revocation_reason_code_t;
typedef uint32_t kmipkit_enumeration_rng_algorithm_t;
typedef uint32_t kmipkit_enumeration_rng_mode_t;
typedef uint32_t kmipkit_enumeration_rotate_name_type_t;
typedef uint32_t kmipkit_enumeration_secret_data_type_t;
typedef uint32_t kmipkit_enumeration_shredding_algorithm_t;
typedef uint32_t kmipkit_enumeration_split_key_method_t;
typedef uint32_t kmipkit_enumeration_state_t;
typedef uint32_t kmipkit_enumeration_tag_t;
typedef uint32_t kmipkit_enumeration_ticket_type_t;
typedef uint32_t kmipkit_enumeration_unique_identifier_t;
typedef uint32_t kmipkit_enumeration_unwrap_mode_t;
typedef uint32_t kmipkit_enumeration_usage_limits_unit_t;
typedef uint32_t kmipkit_enumeration_validation_authority_type_t;
typedef uint32_t kmipkit_enumeration_validation_type_t;
typedef uint32_t kmipkit_enumeration_validity_indicator_t;
typedef uint32_t kmipkit_enumeration_wrapping_method_t;
typedef uint32_t kmipkit_bitmask_cryptographic_usage_mask_t;
typedef uint32_t kmipkit_bitmask_protection_storage_mask_t;
typedef uint32_t kmipkit_bitmask_storage_status_mask_t;

static inline const char *KMIPKIT_ENUMERATION_ADJUSTMENT_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ADJUSTMENT_TYPE_INCREMENT: return "Increment";
    case KMIPKIT_ENUMERATION_ADJUSTMENT_TYPE_DECREMENT: return "Decrement";
    case KMIPKIT_ENUMERATION_ADJUSTMENT_TYPE_NEGATE: return "Negate";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_UNINTERPRETED_TEXT_STRING: return "Uninterpreted Text String";
    case KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_URI: return "URI";
    case KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_OBJECT_SERIAL_NUMBER: return "Object Serial Number";
    case KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_EMAIL_ADDRESS: return "Email Address";
    case KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_DNS_NAME: return "DNS Name";
    case KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_X_500_DISTINGUISHED_NAME: return "X.500 Distinguished Name";
    case KMIPKIT_ENUMERATION_ALTERNATIVE_NAME_TYPE_IP_ADDRESS: return "IP Address";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_MANDATORY: return "Mandatory";
    case KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_OPTIONAL: return "Optional";
    case KMIPKIT_ENUMERATION_ASYNCHRONOUS_INDICATOR_PROHIBITED: return "Prohibited";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_ATTESTATION_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ATTESTATION_TYPE_TPM_QUOTE: return "TPM Quote";
    case KMIPKIT_ENUMERATION_ATTESTATION_TYPE_TCG_INTEGRITY_REPORT: return "TCG Integrity Report";
    case KMIPKIT_ENUMERATION_ATTESTATION_TYPE_SAML_ASSERTION: return "SAML Assertion";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_CONTINUE: return "Continue";
    case KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_STOP: return "Stop";
    case KMIPKIT_ENUMERATION_BATCH_ERROR_CONTINUATION_OPTION_UNDO: return "Undo";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_CBC: return "CBC";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_ECB: return "ECB";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_PCBC: return "PCBC";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_CFB: return "CFB";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_OFB: return "OFB";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_CTR: return "CTR";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_CMAC: return "CMAC";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_CCM: return "CCM";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_GCM: return "GCM";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_CBC_MAC: return "CBC-MAC";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_XTS: return "XTS";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_AESKEYWRAPPADDING: return "AESKeyWrapPadding";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_NISTKEYWRAP: return "NISTKeyWrap";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_X9_102_AESKW: return "X9.102 AESKW";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_X9_102_TDKW: return "X9.102 TDKW";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_X9_102_AKW1: return "X9.102 AKW1";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_X9_102_AKW2: return "X9.102 AKW2";
    case KMIPKIT_ENUMERATION_BLOCK_CIPHER_MODE_AEAD: return "AEAD";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_CANCELLATION_RESULT_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_CANCELLATION_RESULT_CANCELED: return "Canceled";
    case KMIPKIT_ENUMERATION_CANCELLATION_RESULT_UNABLE_TO_CANCEL: return "Unable to Cancel";
    case KMIPKIT_ENUMERATION_CANCELLATION_RESULT_COMPLETED: return "Completed";
    case KMIPKIT_ENUMERATION_CANCELLATION_RESULT_FAILED: return "Failed";
    case KMIPKIT_ENUMERATION_CANCELLATION_RESULT_UNAVAILABLE: return "Unavailable";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_CERTIFICATE_REQUEST_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_CERTIFICATE_REQUEST_TYPE_CRMF: return "CRMF";
    case KMIPKIT_ENUMERATION_CERTIFICATE_REQUEST_TYPE_PKCS_10: return "PKCS#10";
    case KMIPKIT_ENUMERATION_CERTIFICATE_REQUEST_TYPE_PEM: return "PEM";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_CERTIFICATE_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_CERTIFICATE_TYPE_X_509: return "X.509";
    case KMIPKIT_ENUMERATION_CERTIFICATE_TYPE_PGP: return "PGP";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_SERVER_PRE_GENERATED: return "Server Pre-Generated";
    case KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_SERVER_ON_DEMAND: return "Server On-Demand";
    case KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_CLIENT_GENERATED: return "Client Generated";
    case KMIPKIT_ENUMERATION_CLIENT_REGISTRATION_METHOD_CLIENT_REGISTERED: return "Client Registered";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_USERNAME_AND_PASSWORD: return "Username and Password";
    case KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_DEVICE: return "Device";
    case KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_ATTESTATION: return "Attestation";
    case KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_ONE_TIME_PASSWORD: return "One Time Password";
    case KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_HASHED_PASSWORD: return "Hashed Password";
    case KMIPKIT_ENUMERATION_CREDENTIAL_TYPE_TICKET: return "Ticket";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_DES: return "DES";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_VALUE_3DES: return "3DES";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_AES: return "AES";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_RSA: return "RSA";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_DSA: return "DSA";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ECDSA: return "ECDSA";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA1: return "HMAC-SHA1";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA224: return "HMAC-SHA224";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA256: return "HMAC-SHA256";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA384: return "HMAC-SHA384";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA512: return "HMAC-SHA512";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_MD5: return "HMAC-MD5";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_DH: return "DH";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ECDH: return "ECDH";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ECMQV: return "ECMQV";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_BLOWFISH: return "Blowfish";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_CAMELLIA: return "Camellia";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_CAST5: return "CAST5";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_IDEA: return "IDEA";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_MARS: return "MARS";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_RC2: return "RC2";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_RC4: return "RC4";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_RC5: return "RC5";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SKIPJACK: return "SKIPJACK";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_TWOFISH: return "Twofish";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_EC: return "EC";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ONE_TIME_PAD: return "One Time Pad";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_CHACHA20: return "ChaCha20";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_POLY1305: return "Poly1305";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_CHACHA20POLY1305: return "ChaCha20Poly1305";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SHA3_224: return "SHA3-224";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SHA3_256: return "SHA3-256";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SHA3_384: return "SHA3-384";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SHA3_512: return "SHA3-512";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA3_224: return "HMAC-SHA3-224";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA3_256: return "HMAC-SHA3-256";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA3_384: return "HMAC-SHA3-384";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_HMAC_SHA3_512: return "HMAC-SHA3-512";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SHAKE_128: return "SHAKE-128";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SHAKE_256: return "SHAKE-256";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ARIA: return "ARIA";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SEED: return "SEED";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SM2: return "SM2";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SM3: return "SM3";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SM4: return "SM4";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_GOST_R_34_10_2012: return "GOST R 34.10-2012";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_GOST_R_34_11_2012: return "GOST R 34.11-2012";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_GOST_R_34_13_2015: return "GOST R 34.13-2015";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_GOST_28147_89: return "GOST 28147-89";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_XMSS: return "XMSS";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_SPHINCS_256: return "SPHINCS-256";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_MCELIECE: return "McEliece";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_MCELIECE_6960119: return "McEliece-6960119";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_MCELIECE_8192128: return "McEliece-8192128";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ED25519: return "Ed25519";
    case KMIPKIT_ENUMERATION_CRYPTOGRAPHIC_ALGORITHM_ED448: return "Ed448";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_DATA_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_DATA_DECRYPT: return "Decrypt";
    case KMIPKIT_ENUMERATION_DATA_ENCRYPT: return "Encrypt";
    case KMIPKIT_ENUMERATION_DATA_HASH: return "Hash";
    case KMIPKIT_ENUMERATION_DATA_MAC_MAC_DATA: return "MAC MAC Data";
    case KMIPKIT_ENUMERATION_DATA_RNG_RETRIEVE: return "RNG Retrieve";
    case KMIPKIT_ENUMERATION_DATA_SIGN_SIGNATURE_DATA: return "Sign Signature Data";
    case KMIPKIT_ENUMERATION_DATA_SIGNATURE_VERIFY: return "Signature Verify";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_DERIVATION_METHOD_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_PBKDF2: return "PBKDF2";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_HASH: return "HASH";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_HMAC: return "HMAC";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_ENCRYPT: return "ENCRYPT";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_NIST800_108_C: return "NIST800-108-C";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_NIST800_108_F: return "NIST800-108-F";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_NIST800_108_DPI: return "NIST800-108-DPI";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_ASYMMETRIC_KEY: return "Asymmetric Key";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_AWS_SIGNATURE_VERSION_4: return "AWS Signature Version 4";
    case KMIPKIT_ENUMERATION_DERIVATION_METHOD_HKDF: return "HKDF";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_DESTROY_ACTION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_DESTROY_ACTION_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_DESTROY_ACTION_KEY_MATERIAL_DELETED: return "Key Material Deleted";
    case KMIPKIT_ENUMERATION_DESTROY_ACTION_KEY_MATERIAL_SHREDDED: return "Key Material Shredded";
    case KMIPKIT_ENUMERATION_DESTROY_ACTION_META_DATA_DELETED: return "Meta Data Deleted";
    case KMIPKIT_ENUMERATION_DESTROY_ACTION_META_DATA_SHREDDED: return "Meta Data Shredded";
    case KMIPKIT_ENUMERATION_DESTROY_ACTION_DELETED: return "Deleted";
    case KMIPKIT_ENUMERATION_DESTROY_ACTION_SHREDDED: return "Shredded";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_MD2_WITH_RSA_ENCRYPTION: return "MD2 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_MD5_WITH_RSA_ENCRYPTION: return "MD5 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA_1_WITH_RSA_ENCRYPTION: return "SHA-1 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA_224_WITH_RSA_ENCRYPTION: return "SHA-224 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA_256_WITH_RSA_ENCRYPTION: return "SHA-256 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA_384_WITH_RSA_ENCRYPTION: return "SHA-384 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA_512_WITH_RSA_ENCRYPTION: return "SHA-512 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_RSASSA_PSS: return "RSASSA-PSS";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_DSA_WITH_SHA_1: return "DSA with SHA-1";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_DSA_WITH_SHA224: return "DSA with SHA224";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_DSA_WITH_SHA256: return "DSA with SHA256";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_ECDSA_WITH_SHA_1: return "ECDSA with SHA-1";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_ECDSA_WITH_SHA224: return "ECDSA with SHA224";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_ECDSA_WITH_SHA256: return "ECDSA with SHA256";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_ECDSA_WITH_SHA384: return "ECDSA with SHA384";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_ECDSA_WITH_SHA512: return "ECDSA with SHA512";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA3_256_WITH_RSA_ENCRYPTION: return "SHA3-256 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA3_384_WITH_RSA_ENCRYPTION: return "SHA3-384 with RSA Encryption";
    case KMIPKIT_ENUMERATION_DIGITAL_SIGNATURE_ALGORITHM_SHA3_512_WITH_RSA_ENCRYPTION: return "SHA3-512 with RSA Encryption";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_DRBG_ALGORITHM_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_DRBG_ALGORITHM_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_DRBG_ALGORITHM_DUAL_EC: return "Dual-EC";
    case KMIPKIT_ENUMERATION_DRBG_ALGORITHM_HASH: return "Hash";
    case KMIPKIT_ENUMERATION_DRBG_ALGORITHM_HMAC: return "HMAC";
    case KMIPKIT_ENUMERATION_DRBG_ALGORITHM_CTR: return "CTR";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_ENCODING_OPTION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ENCODING_OPTION_NO_ENCODING: return "No Encoding";
    case KMIPKIT_ENUMERATION_ENCODING_OPTION_TTLV_ENCODING: return "TTLV Encoding";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_ENDPOINT_ROLE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ENDPOINT_ROLE_CLIENT: return "Client";
    case KMIPKIT_ENUMERATION_ENDPOINT_ROLE_SERVER: return "Server";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_FIPS186_VARIATION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_FIPS186_VARIATION_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_FIPS186_VARIATION_GP_X_ORIGINAL: return "GP x-Original";
    case KMIPKIT_ENUMERATION_FIPS186_VARIATION_GP_X_CHANGE_NOTICE: return "GP x-Change Notice";
    case KMIPKIT_ENUMERATION_FIPS186_VARIATION_X_ORIGINAL: return "x-Original";
    case KMIPKIT_ENUMERATION_FIPS186_VARIATION_X_CHANGE_NOTICE: return "x-Change Notice";
    case KMIPKIT_ENUMERATION_FIPS186_VARIATION_K_ORIGINAL: return "k-Original";
    case KMIPKIT_ENUMERATION_FIPS186_VARIATION_K_CHANGE_NOTICE: return "k-Change Notice";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_HASHING_ALGORITHM_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_MD2: return "MD2";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_MD4: return "MD4";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_MD5: return "MD5";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_1: return "SHA-1";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_224: return "SHA-224";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_256: return "SHA-256";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_384: return "SHA-384";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_512: return "SHA-512";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_RIPEMD_160: return "RIPEMD-160";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_TIGER: return "Tiger";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_WHIRLPOOL: return "Whirlpool";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_512_224: return "SHA-512/224";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA_512_256: return "SHA-512/256";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA3_224: return "SHA3-224";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA3_256: return "SHA3-256";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA3_384: return "SHA3-384";
    case KMIPKIT_ENUMERATION_HASHING_ALGORITHM_SHA3_512: return "SHA3-512";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_INTEROP_FUNCTION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_INTEROP_FUNCTION_BEGIN: return "Begin";
    case KMIPKIT_ENUMERATION_INTEROP_FUNCTION_END: return "End";
    case KMIPKIT_ENUMERATION_INTEROP_FUNCTION_RESET: return "Reset";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_ITEM_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ITEM_TYPE_STRUCTURE: return "Structure";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_INTEGER: return "Integer";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_LONG_INTEGER: return "Long Integer";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_BIG_INTEGER: return "Big Integer";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_ENUMERATION: return "Enumeration";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_BOOLEAN: return "Boolean";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_TEXT_STRING: return "Text String";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_BYTE_STRING: return "Byte String";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_DATE_TIME: return "Date Time";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_INTERVAL: return "Interval";
    case KMIPKIT_ENUMERATION_ITEM_TYPE_DATE_TIME_EXTENDED: return "Date Time Extended";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_KEY_COMPRESSION_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_KEY_COMPRESSION_TYPE_EC_PUBLIC_KEY_TYPE_UNCOMPRESSED: return "EC Public Key Type Uncompressed";
    case KMIPKIT_ENUMERATION_KEY_COMPRESSION_TYPE_EC_PUBLIC_KEY_TYPE_X9_62_COMPRESSED_PRIME: return "EC Public Key Type X9.62 Compressed Prime";
    case KMIPKIT_ENUMERATION_KEY_COMPRESSION_TYPE_EC_PUBLIC_KEY_TYPE_X9_62_COMPRESSED_CHAR2: return "EC Public Key Type X9.62 Compressed Char2";
    case KMIPKIT_ENUMERATION_KEY_COMPRESSION_TYPE_EC_PUBLIC_KEY_TYPE_X9_62_HYBRID: return "EC Public Key Type X9.62 Hybrid";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_RAW: return "Raw";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_OPAQUE: return "Opaque";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_PKCS_1: return "PKCS#1";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_PKCS_8: return "PKCS#8";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_X_509: return "X.509";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_ECPRIVATEKEY: return "ECPrivateKey";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_SYMMETRIC_KEY: return "Transparent Symmetric Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_DSA_PRIVATE_KEY: return "Transparent DSA Private Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_DSA_PUBLIC_KEY: return "Transparent DSA Public Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_RSA_PRIVATE_KEY: return "Transparent RSA Private Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_RSA_PUBLIC_KEY: return "Transparent RSA Public Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_DH_PRIVATE_KEY: return "Transparent DH Private Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_DH_PUBLIC_KEY: return "Transparent DH Public Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_EC_PRIVATE_KEY: return "Transparent EC Private Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_TRANSPARENT_EC_PUBLIC_KEY: return "Transparent EC Public Key";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_PKCS_12: return "PKCS#12";
    case KMIPKIT_ENUMERATION_KEY_FORMAT_TYPE_PKCS_10: return "PKCS#10";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_BDK: return "BDK";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_CVK: return "CVK";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_DEK: return "DEK";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MKAC: return "MKAC";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MKSMC: return "MKSMC";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MKSMI: return "MKSMI";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MKDAC: return "MKDAC";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MKDN: return "MKDN";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MKCP: return "MKCP";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MKOTH: return "MKOTH";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_KEK: return "KEK";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MAC16609: return "MAC16609";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MAC97971: return "MAC97971";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MAC97972: return "MAC97972";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MAC97973: return "MAC97973";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MAC97974: return "MAC97974";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_MAC97975: return "MAC97975";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_ZPK: return "ZPK";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_PVKIBM: return "PVKIBM";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_PVKPVV: return "PVKPVV";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_PVKOTH: return "PVKOTH";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_DUKPT: return "DUKPT";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_IV: return "IV";
    case KMIPKIT_ENUMERATION_KEY_ROLE_TYPE_TRKBK: return "TRKBK";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_KEY_VALUE_LOCATION_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_KEY_VALUE_LOCATION_TYPE_UNINTERPRETED_TEXT_STRING: return "Uninterpreted Text String";
    case KMIPKIT_ENUMERATION_KEY_VALUE_LOCATION_TYPE_URI: return "URI";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_KEY_WRAP_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_KEY_WRAP_TYPE_NOT_WRAPPED: return "Not Wrapped";
    case KMIPKIT_ENUMERATION_KEY_WRAP_TYPE_AS_REGISTERED: return "As Registered";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_LINK_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_LINK_TYPE_CERTIFICATE_LINK: return "Certificate Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_PUBLIC_KEY_LINK: return "Public Key Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_PRIVATE_KEY_LINK: return "Private Key Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_DERIVATION_BASE_OBJECT_LINK: return "Derivation Base Object Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_DERIVED_KEY_LINK: return "Derived Key Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_REPLACEMENT_OBJECT_LINK: return "Replacement Object Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_REPLACED_OBJECT_LINK: return "Replaced Object Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_PARENT_LINK: return "Parent Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_CHILD_LINK: return "Child Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_PREVIOUS_LINK: return "Previous Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_NEXT_LINK: return "Next Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_PKCS_12_CERTIFICATE_LINK: return "PKCS#12 Certificate Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_PKCS_12_PASSWORD_LINK: return "PKCS#12 Password Link";
    case KMIPKIT_ENUMERATION_LINK_TYPE_WRAPPING_KEY_LINK: return "Wrapping Key Link";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_MASK_GENERATOR_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_MASK_GENERATOR_MFG1: return "MFG1";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_NAME_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_NAME_TYPE_UNINTERPRETED_TEXT_STRING: return "Uninterpreted Text String";
    case KMIPKIT_ENUMERATION_NAME_TYPE_URI: return "URI";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_NIST_KEY_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PRIVATE_SIGNATURE_KEY: return "Private signature key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PUBLIC_SIGNATURE_VERIFICATION_KEY: return "Public signature verification key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_SYMMETRIC_AUTHENTICATION_KEY: return "Symmetric authentication key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PRIVATE_AUTHENTICATION_KEY: return "Private authentication key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PUBLIC_AUTHENTICATION_KEY: return "Public authentication key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_SYMMETRIC_DATA_ENCRYPTION_KEY: return "Symmetric data encryption key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_SYMMETRIC_KEY_WRAPPING_KEY: return "Symmetric key wrapping key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_SYMMETRIC_RANDOM_NUMBER_GENERATION_KEY: return "Symmetric random number generation key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_SYMMETRIC_MASTER_KEY: return "Symmetric master key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PRIVATE_KEY_TRANSPORT_KEY: return "Private key transport key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PUBLIC_KEY_TRANSPORT_KEY: return "Public key transport key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_SYMMETRIC_KEY_AGREEMENT_KEY: return "Symmetric key agreement key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PRIVATE_STATIC_KEY_AGREEMENT_KEY: return "Private static key agreement key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PUBLIC_STATIC_KEY_AGREEMENT_KEY: return "Public static key agreement key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PRIVATE_EPHEMERAL_KEY_AGREEMENT_KEY: return "Private ephemeral key agreement key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PUBLIC_EPHEMERAL_KEY_AGREEMENT_KEY: return "Public ephemeral key agreement key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_SYMMETRIC_AUTHORIZATION_KEY: return "Symmetric authorization key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PRIVATE_AUTHORIZATION_KEY: return "Private authorization key";
    case KMIPKIT_ENUMERATION_NIST_KEY_TYPE_PUBLIC_AUTHORIZATION_KEY: return "Public authorization key";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_OBJECT_GROUP_MEMBER_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_OBJECT_GROUP_MEMBER_GROUP_MEMBER_FRESH: return "Group Member Fresh";
    case KMIPKIT_ENUMERATION_OBJECT_GROUP_MEMBER_GROUP_MEMBER_DEFAULT: return "Group Member Default";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_OBJECT_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_CERTIFICATE: return "Certificate";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_SYMMETRIC_KEY: return "Symmetric Key";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_PUBLIC_KEY: return "Public Key";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_PRIVATE_KEY: return "Private Key";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_SPLIT_KEY: return "Split Key";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_SECRET_DATA: return "Secret Data";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_OPAQUE_OBJECT: return "Opaque Object";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_PGP_KEY: return "PGP Key";
    case KMIPKIT_ENUMERATION_OBJECT_TYPE_CERTIFICATE_REQUEST: return "Certificate Request";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_OPAQUE_DATA_TYPE_name(uint32_t raw) {
    switch (raw) {
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_OPERATION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_OPERATION_CREATE: return "Create";
    case KMIPKIT_ENUMERATION_OPERATION_CREATE_KEY_PAIR: return "Create Key Pair";
    case KMIPKIT_ENUMERATION_OPERATION_REGISTER: return "Register";
    case KMIPKIT_ENUMERATION_OPERATION_RE_KEY: return "Re-key";
    case KMIPKIT_ENUMERATION_OPERATION_DERIVE_KEY: return "Derive Key";
    case KMIPKIT_ENUMERATION_OPERATION_CERTIFY: return "Certify";
    case KMIPKIT_ENUMERATION_OPERATION_RE_CERTIFY: return "Re-certify";
    case KMIPKIT_ENUMERATION_OPERATION_LOCATE: return "Locate";
    case KMIPKIT_ENUMERATION_OPERATION_CHECK: return "Check";
    case KMIPKIT_ENUMERATION_OPERATION_GET: return "Get";
    case KMIPKIT_ENUMERATION_OPERATION_GET_ATTRIBUTES: return "Get Attributes";
    case KMIPKIT_ENUMERATION_OPERATION_GET_ATTRIBUTE_LIST: return "Get Attribute List";
    case KMIPKIT_ENUMERATION_OPERATION_ADD_ATTRIBUTE: return "Add Attribute";
    case KMIPKIT_ENUMERATION_OPERATION_MODIFY_ATTRIBUTE: return "Modify Attribute";
    case KMIPKIT_ENUMERATION_OPERATION_DELETE_ATTRIBUTE: return "Delete Attribute";
    case KMIPKIT_ENUMERATION_OPERATION_OBTAIN_LEASE: return "Obtain Lease";
    case KMIPKIT_ENUMERATION_OPERATION_GET_USAGE_ALLOCATION: return "Get Usage Allocation";
    case KMIPKIT_ENUMERATION_OPERATION_ACTIVATE: return "Activate";
    case KMIPKIT_ENUMERATION_OPERATION_REVOKE: return "Revoke";
    case KMIPKIT_ENUMERATION_OPERATION_DESTROY: return "Destroy";
    case KMIPKIT_ENUMERATION_OPERATION_ARCHIVE: return "Archive";
    case KMIPKIT_ENUMERATION_OPERATION_RECOVER: return "Recover";
    case KMIPKIT_ENUMERATION_OPERATION_VALIDATE: return "Validate";
    case KMIPKIT_ENUMERATION_OPERATION_QUERY: return "Query";
    case KMIPKIT_ENUMERATION_OPERATION_CANCEL: return "Cancel";
    case KMIPKIT_ENUMERATION_OPERATION_POLL: return "Poll";
    case KMIPKIT_ENUMERATION_OPERATION_NOTIFY: return "Notify";
    case KMIPKIT_ENUMERATION_OPERATION_PUT: return "Put";
    case KMIPKIT_ENUMERATION_OPERATION_RE_KEY_KEY_PAIR: return "Re-key Key Pair";
    case KMIPKIT_ENUMERATION_OPERATION_DISCOVER_VERSIONS: return "Discover Versions";
    case KMIPKIT_ENUMERATION_OPERATION_ENCRYPT: return "Encrypt";
    case KMIPKIT_ENUMERATION_OPERATION_DECRYPT: return "Decrypt";
    case KMIPKIT_ENUMERATION_OPERATION_SIGN: return "Sign";
    case KMIPKIT_ENUMERATION_OPERATION_SIGNATURE_VERIFY: return "Signature Verify";
    case KMIPKIT_ENUMERATION_OPERATION_MAC: return "MAC";
    case KMIPKIT_ENUMERATION_OPERATION_MAC_VERIFY: return "MAC Verify";
    case KMIPKIT_ENUMERATION_OPERATION_RNG_RETRIEVE: return "RNG Retrieve";
    case KMIPKIT_ENUMERATION_OPERATION_RNG_SEED: return "RNG Seed";
    case KMIPKIT_ENUMERATION_OPERATION_HASH: return "Hash";
    case KMIPKIT_ENUMERATION_OPERATION_CREATE_SPLIT_KEY: return "Create Split Key";
    case KMIPKIT_ENUMERATION_OPERATION_JOIN_SPLIT_KEY: return "Join Split Key";
    case KMIPKIT_ENUMERATION_OPERATION_IMPORT: return "Import";
    case KMIPKIT_ENUMERATION_OPERATION_EXPORT: return "Export";
    case KMIPKIT_ENUMERATION_OPERATION_LOG: return "Log";
    case KMIPKIT_ENUMERATION_OPERATION_LOGIN: return "Login";
    case KMIPKIT_ENUMERATION_OPERATION_LOGOUT: return "Logout";
    case KMIPKIT_ENUMERATION_OPERATION_DELEGATED_LOGIN: return "Delegated Login";
    case KMIPKIT_ENUMERATION_OPERATION_ADJUST_ATTRIBUTE: return "Adjust Attribute";
    case KMIPKIT_ENUMERATION_OPERATION_SET_ATTRIBUTE: return "Set Attribute";
    case KMIPKIT_ENUMERATION_OPERATION_SET_ENDPOINT_ROLE: return "Set Endpoint Role";
    case KMIPKIT_ENUMERATION_OPERATION_PKCS_11: return "PKCS#11";
    case KMIPKIT_ENUMERATION_OPERATION_INTEROP: return "Interop";
    case KMIPKIT_ENUMERATION_OPERATION_RE_PROVISION: return "Re-Provision";
    case KMIPKIT_ENUMERATION_OPERATION_SET_DEFAULTS: return "Set Defaults";
    case KMIPKIT_ENUMERATION_OPERATION_SET_CONSTRAINTS: return "Set Constraints";
    case KMIPKIT_ENUMERATION_OPERATION_GET_CONSTRAINTS: return "Get Constraints";
    case KMIPKIT_ENUMERATION_OPERATION_QUERY_ASYNCHRONOUS_REQUESTS: return "Query Asynchronous Requests";
    case KMIPKIT_ENUMERATION_OPERATION_PROCESS: return "Process";
    case KMIPKIT_ENUMERATION_OPERATION_PING: return "Ping";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_PADDING_METHOD_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_PADDING_METHOD_NONE: return "None";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_OAEP: return "OAEP";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_PKCS5: return "PKCS5";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_SSL3: return "SSL3";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_ZEROS: return "Zeros";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_ANSI_X9_23: return "ANSI X9.23";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_ISO_10126: return "ISO 10126";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_PKCS1_V1_5: return "PKCS1 v1.5";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_X9_31: return "X9.31";
    case KMIPKIT_ENUMERATION_PADDING_METHOD_PSS: return "PSS";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_PKCS_11_FUNCTION_name(uint32_t raw) {
    switch (raw) {
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_PKCS_11_RETURN_CODE_name(uint32_t raw) {
    switch (raw) {
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_PROCESSING_STAGE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_PROCESSING_STAGE_SUBMITTED: return "Submitted";
    case KMIPKIT_ENUMERATION_PROCESSING_STAGE_IN_PROCESS: return "In Process";
    case KMIPKIT_ENUMERATION_PROCESSING_STAGE_COMPLETED: return "Completed";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_PROFILE_NAME_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_PROFILE_NAME_COMPLETE_SERVER_BASIC: return "Complete Server Basic";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_COMPLETE_SERVER_TLS_V1_2: return "Complete Server TLS v1.2";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_TAPE_LIBRARY_CLIENT: return "Tape Library Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_TAPE_LIBRARY_SERVER: return "Tape Library Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_SYMMETRIC_KEY_LIFECYCLE_CLIENT: return "Symmetric Key Lifecycle Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_SYMMETRIC_KEY_LIFECYCLE_SERVER: return "Symmetric Key Lifecycle Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_ASYMMETRIC_KEY_LIFECYCLE_CLIENT: return "Asymmetric Key Lifecycle Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_ASYMMETRIC_KEY_LIFECYCLE_SERVER: return "Asymmetric Key Lifecycle Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_BASIC_CRYPTOGRAPHIC_CLIENT: return "Basic Cryptographic Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_BASIC_CRYPTOGRAPHIC_SERVER: return "Basic Cryptographic Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_ADVANCED_CRYPTOGRAPHIC_CLIENT: return "Advanced Cryptographic Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_ADVANCED_CRYPTOGRAPHIC_SERVER: return "Advanced Cryptographic Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_RNG_CRYPTOGRAPHIC_CLIENT: return "RNG Cryptographic Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_RNG_CRYPTOGRAPHIC_SERVER: return "RNG Cryptographic Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_BASIC_SYMMETRIC_KEY_FOUNDRY_CLIENT: return "Basic Symmetric Key Foundry Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_INTERMEDIATE_SYMMETRIC_KEY_FOUNDRY_CLIENT: return "Intermediate Symmetric Key Foundry Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_ADVANCED_SYMMETRIC_KEY_FOUNDRY_CLIENT: return "Advanced Symmetric Key Foundry Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_SYMMETRIC_KEY_FOUNDRY_SERVER: return "Symmetric Key Foundry Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_OPAQUE_MANAGED_OBJECT_STORE_CLIENT: return "Opaque Managed Object Store Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_OPAQUE_MANAGED_OBJECT_STORE_SERVER: return "Opaque Managed Object Store Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_STORAGE_ARRAY_WITH_SELF_ENCRYPTING_DRIVE_CLIENT: return "Storage Array with Self Encrypting Drive Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_STORAGE_ARRAY_WITH_SELF_ENCRYPTING_DRIVE_SERVER: return "Storage Array with Self Encrypting Drive Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_HTTPS_CLIENT: return "HTTPS Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_HTTPS_SERVER: return "HTTPS Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_JSON_CLIENT: return "JSON Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_JSON_SERVER: return "JSON Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_XML_CLIENT: return "XML Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_XML_SERVER: return "XML Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_AES_XTS_CLIENT: return "AES XTS Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_AES_XTS_SERVER: return "AES XTS Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_QUANTUM_SAFE_CLIENT: return "Quantum Safe Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_QUANTUM_SAFE_SERVER: return "Quantum Safe Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_PKCS_11_CLIENT: return "PKCS#11 Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_PKCS_11_SERVER: return "PKCS#11 Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_BASELINE_CLIENT: return "Baseline Client";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_BASELINE_SERVER: return "Baseline Server";
    case KMIPKIT_ENUMERATION_PROFILE_NAME_COMPLETE_SERVER: return "Complete Server";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_PROTECTION_LEVEL_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_PROTECTION_LEVEL_HIGH: return "High";
    case KMIPKIT_ENUMERATION_PROTECTION_LEVEL_LOW: return "Low";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_PUT_FUNCTION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_PUT_FUNCTION_NEW: return "New";
    case KMIPKIT_ENUMERATION_PUT_FUNCTION_REPLACE: return "Replace";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_QUERY_FUNCTION_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_OPERATIONS: return "Query Operations";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_OBJECTS: return "Query Objects";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_SERVER_INFORMATION: return "Query Server Information";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_APPLICATION_NAMESPACES: return "Query Application Namespaces";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_EXTENSION_LIST: return "Query Extension List";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_EXTENSION_MAP: return "Query Extension Map";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_ATTESTATION_TYPES: return "Query Attestation Types";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_RNGS: return "Query RNGs";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_VALIDATIONS: return "Query Validations";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_PROFILES: return "Query Profiles";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_CAPABILITIES: return "Query Capabilities";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_CLIENT_REGISTRATION_METHODS: return "Query Client Registration Methods";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_DEFAULTS_INFORMATION: return "Query Defaults Information";
    case KMIPKIT_ENUMERATION_QUERY_FUNCTION_QUERY_STORAGE_PROTECTION_MASKS: return "Query Storage Protection Masks";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_P_192: return "P-192";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_K_163: return "K-163";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_B_163: return "B-163";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_P_224: return "P-224";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_K_233: return "K-233";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_B_233: return "B-233";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_P_256: return "P-256";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_K_283: return "K-283";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_B_283: return "B-283";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_P_384: return "P-384";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_K_409: return "K-409";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_B_409: return "B-409";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_P_521: return "P-521";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_K_571: return "K-571";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_B_571: return "B-571";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP112R1: return "SECP112R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP112R2: return "SECP112R2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP128R1: return "SECP128R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP128R2: return "SECP128R2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP160K1: return "SECP160K1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP160R1: return "SECP160R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP160R2: return "SECP160R2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP192K1: return "SECP192K1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP224K1: return "SECP224K1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECP256K1: return "SECP256K1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT113R1: return "SECT113R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT113R2: return "SECT113R2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT131R1: return "SECT131R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT131R2: return "SECT131R2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT163R1: return "SECT163R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT193R1: return "SECT193R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT193R2: return "SECT193R2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_SECT239K1: return "SECT239K1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9P192V2: return "ANSIX9P192V2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9P192V3: return "ANSIX9P192V3";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9P239V1: return "ANSIX9P239V1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9P239V2: return "ANSIX9P239V2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9P239V3: return "ANSIX9P239V3";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB163V1: return "ANSIX9C2PNB163V1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB163V2: return "ANSIX9C2PNB163V2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB163V3: return "ANSIX9C2PNB163V3";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB176V1: return "ANSIX9C2PNB176V1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB191V1: return "ANSIX9C2TNB191V1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB191V2: return "ANSIX9C2TNB191V2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB191V3: return "ANSIX9C2TNB191V3";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB208W1: return "ANSIX9C2PNB208W1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB239V1: return "ANSIX9C2TNB239V1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB239V2: return "ANSIX9C2TNB239V2";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB239V3: return "ANSIX9C2TNB239V3";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB272W1: return "ANSIX9C2PNB272W1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB304W1: return "ANSIX9C2PNB304W1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB359V1: return "ANSIX9C2TNB359V1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2PNB368W1: return "ANSIX9C2PNB368W1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_ANSIX9C2TNB431R1: return "ANSIX9C2TNB431R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP160R1: return "BRAINPOOLP160R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP160T1: return "BRAINPOOLP160T1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP192R1: return "BRAINPOOLP192R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP192T1: return "BRAINPOOLP192T1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP224R1: return "BRAINPOOLP224R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP224T1: return "BRAINPOOLP224T1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP256R1: return "BRAINPOOLP256R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP256T1: return "BRAINPOOLP256T1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP320R1: return "BRAINPOOLP320R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP320T1: return "BRAINPOOLP320T1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP384R1: return "BRAINPOOLP384R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP384T1: return "BRAINPOOLP384T1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP512R1: return "BRAINPOOLP512R1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_BRAINPOOLP512T1: return "BRAINPOOLP512T1";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_CURVE25519: return "CURVE25519";
    case KMIPKIT_ENUMERATION_RECOMMENDED_CURVE_CURVE448: return "CURVE448";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_RESULT_REASON_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_RESULT_REASON_ITEM_NOT_FOUND: return "Item Not Found";
    case KMIPKIT_ENUMERATION_RESULT_REASON_RESPONSE_TOO_LARGE: return "Response Too Large";
    case KMIPKIT_ENUMERATION_RESULT_REASON_AUTHENTICATION_NOT_SUCCESSFUL: return "Authentication Not Successful";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_MESSAGE: return "Invalid Message";
    case KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_NOT_SUPPORTED: return "Operation Not Supported";
    case KMIPKIT_ENUMERATION_RESULT_REASON_MISSING_DATA: return "Missing Data";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_FIELD: return "Invalid Field";
    case KMIPKIT_ENUMERATION_RESULT_REASON_FEATURE_NOT_SUPPORTED: return "Feature Not Supported";
    case KMIPKIT_ENUMERATION_RESULT_REASON_OPERATION_CANCELED_BY_REQUESTER: return "Operation Canceled By Requester";
    case KMIPKIT_ENUMERATION_RESULT_REASON_CRYPTOGRAPHIC_FAILURE: return "Cryptographic Failure";
    case KMIPKIT_ENUMERATION_RESULT_REASON_PERMISSION_DENIED: return "Permission Denied";
    case KMIPKIT_ENUMERATION_RESULT_REASON_OBJECT_ARCHIVED: return "Object Archived";
    case KMIPKIT_ENUMERATION_RESULT_REASON_APPLICATION_NAMESPACE_NOT_SUPPORTED: return "Application Namespace Not Supported";
    case KMIPKIT_ENUMERATION_RESULT_REASON_KEY_FORMAT_TYPE_NOT_SUPPORTED: return "Key Format Type Not Supported";
    case KMIPKIT_ENUMERATION_RESULT_REASON_KEY_COMPRESSION_TYPE_NOT_SUPPORTED: return "Key Compression Type Not Supported";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ENCODING_OPTION_ERROR: return "Encoding Option Error";
    case KMIPKIT_ENUMERATION_RESULT_REASON_KEY_VALUE_NOT_PRESENT: return "Key Value Not Present";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ATTESTATION_REQUIRED: return "Attestation Required";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ATTESTATION_FAILED: return "Attestation Failed";
    case KMIPKIT_ENUMERATION_RESULT_REASON_SENSITIVE: return "Sensitive";
    case KMIPKIT_ENUMERATION_RESULT_REASON_NOT_EXTRACTABLE: return "Not Extractable";
    case KMIPKIT_ENUMERATION_RESULT_REASON_OBJECT_ALREADY_EXISTS: return "Object Already Exists";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_TICKET: return "Invalid Ticket";
    case KMIPKIT_ENUMERATION_RESULT_REASON_USAGE_LIMIT_EXCEEDED: return "Usage Limit Exceeded";
    case KMIPKIT_ENUMERATION_RESULT_REASON_NUMERIC_RANGE: return "Numeric Range";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_DATA_TYPE: return "Invalid Data Type";
    case KMIPKIT_ENUMERATION_RESULT_REASON_READ_ONLY_ATTRIBUTE: return "Read Only Attribute";
    case KMIPKIT_ENUMERATION_RESULT_REASON_MULTI_VALUED_ATTRIBUTE: return "Multi Valued Attribute";
    case KMIPKIT_ENUMERATION_RESULT_REASON_UNSUPPORTED_ATTRIBUTE: return "Unsupported Attribute";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ATTRIBUTE_INSTANCE_NOT_FOUND: return "Attribute Instance Not Found";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ATTRIBUTE_NOT_FOUND: return "Attribute Not Found";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ATTRIBUTE_READ_ONLY: return "Attribute Read Only";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ATTRIBUTE_SINGLE_VALUED: return "Attribute Single Valued";
    case KMIPKIT_ENUMERATION_RESULT_REASON_BAD_CRYPTOGRAPHIC_PARAMETERS: return "Bad Cryptographic Parameters";
    case KMIPKIT_ENUMERATION_RESULT_REASON_BAD_PASSWORD: return "Bad Password";
    case KMIPKIT_ENUMERATION_RESULT_REASON_CODEC_ERROR: return "Codec Error";
    case KMIPKIT_ENUMERATION_RESULT_REASON_ILLEGAL_OBJECT_TYPE: return "Illegal Object Type";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INCOMPATIBLE_CRYPTOGRAPHIC_USAGE_MASK: return "Incompatible Cryptographic Usage Mask";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INTERNAL_SERVER_ERROR: return "Internal Server Error";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_ASYNCHRONOUS_CORRELATION_VALUE: return "Invalid Asynchronous Correlation Value";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_ATTRIBUTE: return "Invalid Attribute";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_ATTRIBUTE_VALUE: return "Invalid Attribute Value";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_CORRELATION_VALUE: return "Invalid Correlation Value";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_CSR: return "Invalid CSR";
    case KMIPKIT_ENUMERATION_RESULT_REASON_INVALID_OBJECT_TYPE: return "Invalid Object Type";
    case KMIPKIT_ENUMERATION_RESULT_REASON_KEY_WRAP_TYPE_NOT_SUPPORTED: return "Key Wrap Type Not Supported";
    case KMIPKIT_ENUMERATION_RESULT_REASON_MISSING_INITIALIZATION_VECTOR: return "Missing Initialization Vector";
    case KMIPKIT_ENUMERATION_RESULT_REASON_NON_UNIQUE_NAME_ATTRIBUTE: return "Non Unique Name Attribute";
    case KMIPKIT_ENUMERATION_RESULT_REASON_OBJECT_DESTROYED: return "Object Destroyed";
    case KMIPKIT_ENUMERATION_RESULT_REASON_OBJECT_NOT_FOUND: return "Object Not Found";
    case KMIPKIT_ENUMERATION_RESULT_REASON_NOT_AUTHORISED: return "Not Authorised";
    case KMIPKIT_ENUMERATION_RESULT_REASON_SERVER_LIMIT_EXCEEDED: return "Server Limit Exceeded";
    case KMIPKIT_ENUMERATION_RESULT_REASON_UNKNOWN_ENUMERATION: return "Unknown Enumeration";
    case KMIPKIT_ENUMERATION_RESULT_REASON_UNKNOWN_MESSAGE_EXTENSION: return "Unknown Message Extension";
    case KMIPKIT_ENUMERATION_RESULT_REASON_UNKNOWN_TAG: return "Unknown Tag";
    case KMIPKIT_ENUMERATION_RESULT_REASON_UNSUPPORTED_CRYPTOGRAPHIC_PARAMETERS: return "Unsupported Cryptographic Parameters";
    case KMIPKIT_ENUMERATION_RESULT_REASON_UNSUPPORTED_PROTOCOL_VERSION: return "Unsupported Protocol Version";
    case KMIPKIT_ENUMERATION_RESULT_REASON_WRAPPING_OBJECT_ARCHIVED: return "Wrapping Object Archived";
    case KMIPKIT_ENUMERATION_RESULT_REASON_WRAPPING_OBJECT_DESTROYED: return "Wrapping Object Destroyed";
    case KMIPKIT_ENUMERATION_RESULT_REASON_WRAPPING_OBJECT_NOT_FOUND: return "Wrapping Object Not Found";
    case KMIPKIT_ENUMERATION_RESULT_REASON_WRONG_KEY_LIFECYCLE_STATE: return "Wrong Key Lifecycle State";
    case KMIPKIT_ENUMERATION_RESULT_REASON_PROTECTION_STORAGE_UNAVAILABLE: return "Protection Storage Unavailable";
    case KMIPKIT_ENUMERATION_RESULT_REASON_PKCS_11_CODEC_ERROR: return "PKCS#11 Codec Error";
    case KMIPKIT_ENUMERATION_RESULT_REASON_PKCS_11_INVALID_FUNCTION: return "PKCS#11 Invalid Function";
    case KMIPKIT_ENUMERATION_RESULT_REASON_PKCS_11_INVALID_INTERFACE: return "PKCS#11 Invalid Interface";
    case KMIPKIT_ENUMERATION_RESULT_REASON_PRIVATE_PROTECTION_STORAGE_UNAVAILABLE: return "Private Protection Storage Unavailable";
    case KMIPKIT_ENUMERATION_RESULT_REASON_PUBLIC_PROTECTION_STORAGE_UNAVAILABLE: return "Public Protection Storage Unavailable";
    case KMIPKIT_ENUMERATION_RESULT_REASON_UNKNOWN_OBJECT_GROUP: return "Unknown Object Group";
    case KMIPKIT_ENUMERATION_RESULT_REASON_CONSTRAINT_VIOLATION: return "Constraint Violation";
    case KMIPKIT_ENUMERATION_RESULT_REASON_DUPLICATE_PROCESS_REQUEST: return "Duplicate Process Request";
    case KMIPKIT_ENUMERATION_RESULT_REASON_GENERAL_FAILURE: return "General Failure";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_RESULT_STATUS_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_RESULT_STATUS_SUCCESS: return "Success";
    case KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_FAILED: return "Operation Failed";
    case KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_PENDING: return "Operation Pending";
    case KMIPKIT_ENUMERATION_RESULT_STATUS_OPERATION_UNDONE: return "Operation Undone";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_KEY_COMPROMISE: return "Key Compromise";
    case KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_CA_COMPROMISE: return "CA Compromise";
    case KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_AFFILIATION_CHANGED: return "Affiliation Changed";
    case KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_SUPERSEDED: return "Superseded";
    case KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_CESSATION_OF_OPERATION: return "Cessation of Operation";
    case KMIPKIT_ENUMERATION_REVOCATION_REASON_CODE_PRIVILEGE_WITHDRAWN: return "Privilege Withdrawn";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_RNG_ALGORITHM_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_RNG_ALGORITHM_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_RNG_ALGORITHM_FIPS_186_2: return "FIPS 186-2";
    case KMIPKIT_ENUMERATION_RNG_ALGORITHM_DRBG: return "DRBG";
    case KMIPKIT_ENUMERATION_RNG_ALGORITHM_NRBG: return "NRBG";
    case KMIPKIT_ENUMERATION_RNG_ALGORITHM_ANSI_X9_31: return "ANSI X9.31";
    case KMIPKIT_ENUMERATION_RNG_ALGORITHM_ANSI_X9_62: return "ANSI X9.62";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_RNG_MODE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_RNG_MODE_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_RNG_MODE_SHARED_INSTANTIATION: return "Shared Instantiation";
    case KMIPKIT_ENUMERATION_RNG_MODE_NON_SHARED_INSTANTIATION: return "Non-Shared Instantiation";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_ROTATE_NAME_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_ROTATE_NAME_TYPE_UNINTERPRETED_TEXT_STRING: return "Uninterpreted Text String";
    case KMIPKIT_ENUMERATION_ROTATE_NAME_TYPE_URI: return "URI";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_SECRET_DATA_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_SECRET_DATA_TYPE_PASSWORD: return "Password";
    case KMIPKIT_ENUMERATION_SECRET_DATA_TYPE_SEED: return "Seed";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_SHREDDING_ALGORITHM_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_SHREDDING_ALGORITHM_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_SHREDDING_ALGORITHM_CRYPTOGRAPHIC: return "Cryptographic";
    case KMIPKIT_ENUMERATION_SHREDDING_ALGORITHM_UNSUPPORTED: return "Unsupported";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_XOR: return "XOR";
    case KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_POLYNOMIAL_SHARING_GF_216: return "Polynomial Sharing GF (2\302\271\342\201\266)";
    case KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_POLYNOMIAL_SHARING_PRIME_FIELD: return "Polynomial Sharing Prime Field";
    case KMIPKIT_ENUMERATION_SPLIT_KEY_METHOD_POLYNOMIAL_SHARING_GF_28: return "Polynomial Sharing GF (2\342\201\270)";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_STATE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_STATE_PRE_ACTIVE: return "Pre-Active";
    case KMIPKIT_ENUMERATION_STATE_ACTIVE: return "Active";
    case KMIPKIT_ENUMERATION_STATE_DEACTIVATED: return "Deactivated";
    case KMIPKIT_ENUMERATION_STATE_COMPROMISED: return "Compromised";
    case KMIPKIT_ENUMERATION_STATE_DESTROYED: return "Destroyed";
    case KMIPKIT_ENUMERATION_STATE_DESTROYED_COMPROMISED: return "Destroyed Compromised";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_TAG_name(uint32_t raw) {
    switch (raw) {
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_TICKET_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_TICKET_TYPE_LOGIN: return "Login";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_ID_PLACEHOLDER: return "ID Placeholder";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_CERTIFY: return "Certify";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_CREATE: return "Create";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_CREATE_KEY_PAIR: return "Create Key Pair";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_CREATE_KEY_PAIR_PRIVATE_KEY: return "Create Key Pair Private Key";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_CREATE_KEY_PAIR_PUBLIC_KEY: return "Create Key Pair Public Key";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_CREATE_SPLIT_KEY: return "Create Split Key";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_DERIVE_KEY: return "Derive Key";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_IMPORT: return "Import";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_JOIN_SPLIT_KEY: return "Join Split Key";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_LOCATE: return "Locate";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_REGISTER: return "Register";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_RE_KEY: return "Re-key";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_RE_CERTIFY: return "Re-certify";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_RE_KEY_KEY_PAIR: return "Re-key Key Pair";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_RE_KEY_KEY_PAIR_PRIVATE_KEY: return "Re-key Key Pair Private Key";
    case KMIPKIT_ENUMERATION_UNIQUE_IDENTIFIER_RE_KEY_KEY_PAIR_PUBLIC_KEY: return "Re-key Key Pair Public Key";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_UNWRAP_MODE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_UNWRAP_MODE_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_UNWRAP_MODE_PROCESSED: return "Processed";
    case KMIPKIT_ENUMERATION_UNWRAP_MODE_NOT_PROCESSED: return "Not Processed";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_USAGE_LIMITS_UNIT_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_USAGE_LIMITS_UNIT_BYTE: return "Byte";
    case KMIPKIT_ENUMERATION_USAGE_LIMITS_UNIT_OBJECT: return "Object";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_VALIDATION_AUTHORITY_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_VALIDATION_AUTHORITY_TYPE_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_VALIDATION_AUTHORITY_TYPE_NIST_CMVP: return "NIST CMVP";
    case KMIPKIT_ENUMERATION_VALIDATION_AUTHORITY_TYPE_COMMON_CRITERIA: return "Common Criteria";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_VALIDATION_TYPE_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_VALIDATION_TYPE_UNSPECIFIED: return "Unspecified";
    case KMIPKIT_ENUMERATION_VALIDATION_TYPE_HARDWARE: return "Hardware";
    case KMIPKIT_ENUMERATION_VALIDATION_TYPE_SOFTWARE: return "Software";
    case KMIPKIT_ENUMERATION_VALIDATION_TYPE_FIRMWARE: return "Firmware";
    case KMIPKIT_ENUMERATION_VALIDATION_TYPE_HYBRID: return "Hybrid";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_VALIDITY_INDICATOR_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_VALIDITY_INDICATOR_VALID: return "Valid";
    case KMIPKIT_ENUMERATION_VALIDITY_INDICATOR_INVALID: return "Invalid";
    case KMIPKIT_ENUMERATION_VALIDITY_INDICATOR_UNKNOWN: return "Unknown";
    default: return 0;
    }
}

static inline const char *KMIPKIT_ENUMERATION_WRAPPING_METHOD_name(uint32_t raw) {
    switch (raw) {
    case KMIPKIT_ENUMERATION_WRAPPING_METHOD_ENCRYPT: return "Encrypt";
    case KMIPKIT_ENUMERATION_WRAPPING_METHOD_MAC_SIGN: return "MAC/sign";
    case KMIPKIT_ENUMERATION_WRAPPING_METHOD_ENCRYPT_THEN_MAC_SIGN: return "Encrypt then MAC/sign";
    case KMIPKIT_ENUMERATION_WRAPPING_METHOD_MAC_SIGN_THEN_ENCRYPT: return "MAC/sign then encrypt";
    case KMIPKIT_ENUMERATION_WRAPPING_METHOD_TR_31: return "TR-31";
    default: return 0;
    }
}

static inline uint32_t KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_known_bits(uint32_t raw) { return raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_KNOWN_MASK; }
static inline uint32_t KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_unknown_bits(uint32_t raw) { return raw & ~KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_KNOWN_MASK; }
static inline const char *KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_name(uint32_t bit) {
    switch (bit) {
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_SIGN: return "Sign";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_VERIFY: return "Verify";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT: return "Encrypt";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT: return "Decrypt";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_WRAP_KEY: return "Wrap Key";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_UNWRAP_KEY: return "Unwrap Key";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_MAC_GENERATE: return "MAC Generate";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_MAC_VERIFY: return "MAC Verify";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DERIVE_KEY: return "Derive Key";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_KEY_AGREEMENT: return "Key Agreement";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_CERTIFICATE_SIGN: return "Certificate Sign";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_CRL_SIGN: return "CRL Sign";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_AUTHENTICATE: return "Authenticate";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_UNRESTRICTED: return "Unrestricted";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_FPE_ENCRYPT: return "FPE Encrypt";
    case KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_FPE_DECRYPT: return "FPE Decrypt";
    default: return 0;
    }
}
static inline uint32_t KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_names(uint32_t raw, const char **out_names, uint32_t capacity) {
    uint32_t count = 0;
    if (out_names == 0 && capacity != 0) return 0;
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_SIGN) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Sign";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_VERIFY) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Verify";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_ENCRYPT) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Encrypt";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DECRYPT) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Decrypt";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_WRAP_KEY) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Wrap Key";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_UNWRAP_KEY) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Unwrap Key";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_MAC_GENERATE) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "MAC Generate";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_MAC_VERIFY) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "MAC Verify";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_DERIVE_KEY) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Derive Key";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_KEY_AGREEMENT) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Key Agreement";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_CERTIFICATE_SIGN) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Certificate Sign";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_CRL_SIGN) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "CRL Sign";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_AUTHENTICATE) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Authenticate";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_UNRESTRICTED) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Unrestricted";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_FPE_ENCRYPT) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "FPE Encrypt";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_CRYPTOGRAPHIC_USAGE_MASK_FPE_DECRYPT) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "FPE Decrypt";
        ++count;
    }
    return count;
}

static inline uint32_t KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_known_bits(uint32_t raw) { return raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_KNOWN_MASK; }
static inline uint32_t KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_unknown_bits(uint32_t raw) { return raw & ~KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_KNOWN_MASK; }
static inline const char *KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_name(uint32_t bit) {
    switch (bit) {
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_SOFTWARE: return "Software";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_HARDWARE: return "Hardware";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_ON_PROCESSOR: return "On Processor";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_ON_SYSTEM: return "On System";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OFF_SYSTEM: return "Off System";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_HYPERVISOR: return "Hypervisor";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OPERATING_SYSTEM: return "Operating System";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_CONTAINER: return "Container";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_ON_PREMISES: return "On Premises";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OFF_PREMISES: return "Off Premises";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_SELF_MANAGED: return "Self Managed";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OUTSOURCED: return "Outsourced";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_VALIDATED: return "Validated";
    case KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_SAME_JURISDICTION: return "Same Jurisdiction";
    default: return 0;
    }
}
static inline uint32_t KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_names(uint32_t raw, const char **out_names, uint32_t capacity) {
    uint32_t count = 0;
    if (out_names == 0 && capacity != 0) return 0;
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_SOFTWARE) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Software";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_HARDWARE) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Hardware";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_ON_PROCESSOR) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "On Processor";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_ON_SYSTEM) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "On System";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OFF_SYSTEM) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Off System";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_HYPERVISOR) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Hypervisor";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OPERATING_SYSTEM) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Operating System";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_CONTAINER) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Container";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_ON_PREMISES) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "On Premises";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OFF_PREMISES) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Off Premises";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_SELF_MANAGED) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Self Managed";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_OUTSOURCED) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Outsourced";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_VALIDATED) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Validated";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_PROTECTION_STORAGE_MASK_SAME_JURISDICTION) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Same Jurisdiction";
        ++count;
    }
    return count;
}

static inline uint32_t KMIPKIT_BITMASK_STORAGE_STATUS_MASK_known_bits(uint32_t raw) { return raw & KMIPKIT_BITMASK_STORAGE_STATUS_MASK_KNOWN_MASK; }
static inline uint32_t KMIPKIT_BITMASK_STORAGE_STATUS_MASK_unknown_bits(uint32_t raw) { return raw & ~KMIPKIT_BITMASK_STORAGE_STATUS_MASK_KNOWN_MASK; }
static inline const char *KMIPKIT_BITMASK_STORAGE_STATUS_MASK_name(uint32_t bit) {
    switch (bit) {
    case KMIPKIT_BITMASK_STORAGE_STATUS_MASK_ON_LINE_STORAGE: return "On-line storage";
    case KMIPKIT_BITMASK_STORAGE_STATUS_MASK_ARCHIVAL_STORAGE: return "Archival storage";
    case KMIPKIT_BITMASK_STORAGE_STATUS_MASK_DESTROYED_STORAGE: return "Destroyed storage";
    default: return 0;
    }
}
static inline uint32_t KMIPKIT_BITMASK_STORAGE_STATUS_MASK_names(uint32_t raw, const char **out_names, uint32_t capacity) {
    uint32_t count = 0;
    if (out_names == 0 && capacity != 0) return 0;
    if ((raw & KMIPKIT_BITMASK_STORAGE_STATUS_MASK_ON_LINE_STORAGE) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "On-line storage";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_STORAGE_STATUS_MASK_ARCHIVAL_STORAGE) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Archival storage";
        ++count;
    }
    if ((raw & KMIPKIT_BITMASK_STORAGE_STATUS_MASK_DESTROYED_STORAGE) != 0) {
        if (out_names != 0 && count < capacity) out_names[count] = "Destroyed storage";
        ++count;
    }
    return count;
}

/* Exact assigned values added by later KMIP editions. */
static inline const char *KMIPKIT_V2_1_OPERATION_name(uint32_t raw) {
    switch (raw) {
    case UINT32_C(0x00000036): return "Set Defaults";
    case UINT32_C(0x00000037): return "Set Constraints";
    case UINT32_C(0x00000038): return "Get Constraints";
    case UINT32_C(0x00000039): return "Query Asynchronous Requests";
    case UINT32_C(0x0000003A): return "Process";
    case UINT32_C(0x0000003B): return "Ping";
    default: return 0;
    }
}
static inline const char *KMIPKIT_V3_0_OPERATION_name(uint32_t raw) {
    switch (raw) {
    case UINT32_C(0x0000003C): return "Create Group";
    case UINT32_C(0x0000003D): return "Obliterate";
    case UINT32_C(0x0000003E): return "Create User";
    case UINT32_C(0x0000003F): return "Create Credential";
    case UINT32_C(0x00000040): return "Deactivate";
    case UINT32_C(0x00000041): return "Encapsulate";
    case UINT32_C(0x00000042): return "Decapsulate";
    default: return 0;
    }
}
static inline const char *KMIPKIT_V3_0_OBJECT_CLASS_name(uint32_t raw) {
    switch (raw) {
    case UINT32_C(0x00000001): return "User";
    case UINT32_C(0x00000002): return "System";
    default: return 0;
    }
}
static inline const char *KMIPKIT_V3_0_OBJECT_TYPE_name(uint32_t raw) {
    switch (raw) {
    case UINT32_C(0x0000000B): return "User";
    case UINT32_C(0x0000000C): return "Group";
    case UINT32_C(0x0000000D): return "Password Credential";
    case UINT32_C(0x0000000E): return "Device Credential";
    case UINT32_C(0x0000000F): return "One Time Password Credential";
    case UINT32_C(0x00000010): return "Hashed Password Credential";
    default: return 0;
    }
}
#endif
