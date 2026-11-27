package org.kmipkit.fluent;

import java.util.OptionalLong;

/** Curated symmetric, classical asymmetric, and KMIP 3.0 post-quantum key specifications. */
public enum KeySpec {
    AES_128(3L, 128, null, true, 2),
    AES_192(3L, 192, null, true, 2),
    AES_256(3L, 256, null, true, 2),
    HMAC_SHA256(9L, 256, null, true, 2),
    HMAC_SHA384(0x0AL, 384, null, true, 2),
    HMAC_SHA512(0x0BL, 512, null, true, 2),
    CHACHA20(0x1CL, 256, null, true, 2),
    CHACHA20_POLY1305(0x1EL, 256, null, true, 2),
    RSA_2048(4L, 2048, null, false, 2),
    RSA_3072(4L, 3072, null, false, 2),
    RSA_4096(4L, 4096, null, false, 2),
    ECDSA_P256(6L, 256, 7L, false, 2),
    ECDSA_P384(6L, 384, 0x0AL, false, 2),
    ECDSA_P521(6L, 521, 0x0DL, false, 2),
    ECDSA_SECP256K1(6L, 256, 0x19L, false, 2),
    ED25519(0x37L, 256, 0x45L, false, 2),
    ED448(0x38L, 448, 0x46L, false, 2),
    X25519(0x5AL, 256, 0x45L, false, 3),
    X448(0x5BL, 448, 0x46L, false, 3),
    ML_KEM_512(0x39L, 512, null, false, 3),
    ML_KEM_768(0x3AL, 768, null, false, 3),
    ML_KEM_1024(0x3BL, 1024, null, false, 3),
    ML_DSA_44(0x3CL, 1312 * 8, null, false, 3),
    ML_DSA_65(0x3DL, 1952 * 8, null, false, 3),
    ML_DSA_87(0x3EL, 2592 * 8, null, false, 3),
    SLH_DSA_SHA2_128F(0x40L, 128, null, false, 3),
    X25519_MLKEM768(0x5CL, 768, null, false, 3),
    SECP256R1_MLKEM768(0x5DL, 768, 7L, false, 3);

    private final long algorithm;
    private final int length;
    private final Long recommendedCurve;
    private final boolean symmetric;
    private final int minKmipMajor;

    KeySpec(long algorithm, int length, Long recommendedCurve, boolean symmetric, int minKmipMajor) {
        this.algorithm = algorithm;
        this.length = length;
        this.recommendedCurve = recommendedCurve;
        this.symmetric = symmetric;
        this.minKmipMajor = minKmipMajor;
    }

    public long algorithm() { return algorithm; }
    public int length() { return length; }
    public OptionalLong recommendedCurve() {
        return recommendedCurve == null ? OptionalLong.empty() : OptionalLong.of(recommendedCurve);
    }
    public boolean isSymmetric() { return symmetric; }
    public boolean isAsymmetric() { return !symmetric; }
    public int minKmipMajor() { return minKmipMajor; }
    public boolean requiresKmip30() { return minKmipMajor == 3; }
}
