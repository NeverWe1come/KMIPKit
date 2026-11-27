/* kmipkit_presets.h - Curated classical & KMIP 3.0 PQC presets for C11 callers. */
#ifndef KMIPKIT_PRESETS_H
#define KMIPKIT_PRESETS_H

#include <stdint.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct kmipkit_key_spec_t {
    uint32_t cryptographic_algorithm;
    int32_t cryptographic_length;
    uint8_t has_recommended_curve;
    uint8_t is_symmetric;
    uint8_t min_kmip_major;
    uint8_t reserved;
    uint32_t recommended_curve;
} kmipkit_key_spec_t;

typedef struct kmipkit_cipher_spec_t {
    uint32_t cryptographic_algorithm;
    uint8_t has_block_cipher_mode;
    uint8_t has_padding_method;
    uint8_t has_hashing_algorithm;
    uint8_t random_iv;
    uint32_t block_cipher_mode;
    uint32_t padding_method;
    uint32_t hashing_algorithm;
    int32_t iv_length;
    int32_t tag_length;
} kmipkit_cipher_spec_t;

/* Symmetric KeySpec presets */
#define KMIPKIT_KEY_SPEC_AES_128 ((kmipkit_key_spec_t){ 0x03u, 128, 0u, 1u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_AES_192 ((kmipkit_key_spec_t){ 0x03u, 192, 0u, 1u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_AES_256 ((kmipkit_key_spec_t){ 0x03u, 256, 0u, 1u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_HMAC_SHA256 ((kmipkit_key_spec_t){ 0x09u, 256, 0u, 1u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_HMAC_SHA384 ((kmipkit_key_spec_t){ 0x0Au, 384, 0u, 1u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_HMAC_SHA512 ((kmipkit_key_spec_t){ 0x0Bu, 512, 0u, 1u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_CHACHA20 ((kmipkit_key_spec_t){ 0x1Cu, 256, 0u, 1u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_CHACHA20_POLY1305 ((kmipkit_key_spec_t){ 0x1Eu, 256, 0u, 1u, 2u, 0u, 0u })

/* Classical Asymmetric KeySpec presets */
#define KMIPKIT_KEY_SPEC_RSA_2048 ((kmipkit_key_spec_t){ 0x04u, 2048, 0u, 0u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_RSA_3072 ((kmipkit_key_spec_t){ 0x04u, 3072, 0u, 0u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_RSA_4096 ((kmipkit_key_spec_t){ 0x04u, 4096, 0u, 0u, 2u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_ECDSA_P256 ((kmipkit_key_spec_t){ 0x06u, 256, 1u, 0u, 2u, 0u, 0x07u })
#define KMIPKIT_KEY_SPEC_ECDSA_P384 ((kmipkit_key_spec_t){ 0x06u, 384, 1u, 0u, 2u, 0u, 0x0Au })
#define KMIPKIT_KEY_SPEC_ECDSA_P521 ((kmipkit_key_spec_t){ 0x06u, 521, 1u, 0u, 2u, 0u, 0x0Du })
#define KMIPKIT_KEY_SPEC_ED25519 ((kmipkit_key_spec_t){ 0x37u, 256, 1u, 0u, 2u, 0u, 0x45u })
#define KMIPKIT_KEY_SPEC_ED448 ((kmipkit_key_spec_t){ 0x38u, 448, 1u, 0u, 2u, 0u, 0x46u })

/* KMIP 3.0 Post-Quantum & Hybrid KeySpec presets */
#define KMIPKIT_KEY_SPEC_ML_KEM_512 ((kmipkit_key_spec_t){ 0x39u, 512, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_ML_KEM_768 ((kmipkit_key_spec_t){ 0x3Au, 768, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_ML_KEM_1024 ((kmipkit_key_spec_t){ 0x3Bu, 1024, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_ML_DSA_44 ((kmipkit_key_spec_t){ 0x3Cu, 1312 * 8, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_ML_DSA_65 ((kmipkit_key_spec_t){ 0x3Du, 1952 * 8, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_ML_DSA_87 ((kmipkit_key_spec_t){ 0x3Eu, 2592 * 8, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_SLH_DSA_SHA2_128F ((kmipkit_key_spec_t){ 0x40u, 128, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_X25519_MLKEM768 ((kmipkit_key_spec_t){ 0x5Cu, 768, 0u, 0u, 3u, 0u, 0u })
#define KMIPKIT_KEY_SPEC_SECP256R1_MLKEM768 ((kmipkit_key_spec_t){ 0x5Du, 768, 1u, 0u, 3u, 0u, 0x07u })

/* CipherSpec presets */
#define KMIPKIT_CIPHER_SPEC_AES_CBC_PKCS5 ((kmipkit_cipher_spec_t){ 0x03u, 1u, 1u, 0u, 0u, 0x01u, 0x03u, 0u, 0, 0 })
#define KMIPKIT_CIPHER_SPEC_AES_GCM ((kmipkit_cipher_spec_t){ 0x03u, 1u, 0u, 0u, 1u, 0x09u, 0u, 0u, 96, 128 })
#define KMIPKIT_CIPHER_SPEC_AES_CTR ((kmipkit_cipher_spec_t){ 0x03u, 1u, 0u, 0u, 1u, 0x06u, 0u, 0u, 128, 0 })
#define KMIPKIT_CIPHER_SPEC_CHACHA20_POLY1305 ((kmipkit_cipher_spec_t){ 0x1Eu, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0, 0 })
#define KMIPKIT_CIPHER_SPEC_RSA_OAEP_SHA256 ((kmipkit_cipher_spec_t){ 0x04u, 0u, 1u, 1u, 0u, 0u, 0x02u, 0x06u, 0, 0 })

/* HashAlgorithm & KEM constants */
#define KMIPKIT_HASH_SHA256 0x06u
#define KMIPKIT_HASH_SHA384 0x07u
#define KMIPKIT_HASH_SHA512 0x08u
#define KMIPKIT_HASH_SHA3_256 0x0Fu
#define KMIPKIT_KEM_RSASVE 0x01u
#define KMIPKIT_KEM_DHKEM 0x02u
#define KMIPKIT_KEM_MLKEM 0x03u

#if defined(KMIPKIT_H)
static inline kmipkit_symmetric_key_request_v1_t kmipkit_fluent_symmetric_key_from_spec(
    kmipkit_key_spec_t spec,
    int32_t usage_mask,
    const char *name_utf8
) {
    kmipkit_symmetric_key_request_v1_t r;
    memset(&r, 0, sizeof(r));
    r.struct_size = (uint32_t)sizeof(r);
    r.abi_version = 1u;
    r.cryptographic_algorithm = spec.cryptographic_algorithm;
    r.cryptographic_length = spec.cryptographic_length;
    r.cryptographic_usage_mask = usage_mask;
    if (name_utf8 != NULL) {
        r.name_present = 1u;
        r.name.data = (const uint8_t *)name_utf8;
        r.name.length = (uint64_t)strlen(name_utf8);
    }
    return r;
}

static inline kmipkit_create_key_pair_request_v1_t kmipkit_fluent_key_pair_from_spec(
    kmipkit_key_spec_t spec,
    int32_t private_usage_mask,
    int32_t public_usage_mask
) {
    kmipkit_create_key_pair_request_v1_t r;
    memset(&r, 0, sizeof(r));
    r.struct_size = (uint32_t)sizeof(r);
    r.abi_version = 1u;
    r.cryptographic_algorithm = spec.cryptographic_algorithm;
    r.cryptographic_length = spec.cryptographic_length;
    r.private_usage_mask_present = 1u;
    r.private_usage_mask = private_usage_mask;
    r.public_usage_mask_present = 1u;
    r.public_usage_mask = public_usage_mask;
    return r;
}
#endif

#ifdef __cplusplus
}
#endif

#endif /* KMIPKIT_PRESETS_H */
