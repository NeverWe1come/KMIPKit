package org.kmipkit.fluent;

import org.kmipkit.protocol.CryptographicParameters;

/** Curated classical and KMIP 3.0 post-quantum digital signature presets. */
public enum SignatureSpec {
    ECDSA_SHA256(2),
    ECDSA_SHA384(2),
    ECDSA_SHA512(2),
    RSA_PSS_SHA256(2),
    RSA_PSS_SHA384(2),
    RSA_PSS_SHA512(2),
    RSA_PKCS1_SHA256(2),
    RSA_PKCS1_SHA384(2),
    RSA_PKCS1_SHA512(2),
    ED25519(2),
    ED448(2),
    ML_DSA_44(3),
    ML_DSA_65(3),
    ML_DSA_87(3),
    ML_DSA_65_DETERMINISTIC(3),
    SLH_DSA_SHA2_128F(3);

    private final int minKmipMajor;

    SignatureSpec(int minKmipMajor) {
        this.minKmipMajor = minKmipMajor;
    }

    public int minKmipMajor() { return minKmipMajor; }

    /** Builds validated KMIP {@link CryptographicParameters} for this preset. */
    public CryptographicParameters toCryptographicParameters() {
        try (CryptographicParameters.Builder b = CryptographicParameters.builder()) {
            switch (this) {
                case ECDSA_SHA256: ecdsa(b, 0x0EL, 6L); break;
                case ECDSA_SHA384: ecdsa(b, 0x0FL, 7L); break;
                case ECDSA_SHA512: ecdsa(b, 0x10L, 8L); break;
                case RSA_PSS_SHA256: pss(b, 6L, 32); break;
                case RSA_PSS_SHA384: pss(b, 7L, 48); break;
                case RSA_PSS_SHA512: pss(b, 8L, 64); break;
                case RSA_PKCS1_SHA256: pkcs1(b, 5L, 6L); break;
                case RSA_PKCS1_SHA384: pkcs1(b, 6L, 7L); break;
                case RSA_PKCS1_SHA512: pkcs1(b, 7L, 8L); break;
                case ED25519: b.cryptographicAlgorithm(0x37L); break;
                case ED448: b.cryptographicAlgorithm(0x38L); break;
                case ML_DSA_44: b.cryptographicAlgorithm(0x3CL); break;
                case ML_DSA_65: b.cryptographicAlgorithm(0x3DL); break;
                case ML_DSA_87: b.cryptographicAlgorithm(0x3EL); break;
                case ML_DSA_65_DETERMINISTIC:
                    b.cryptographicAlgorithm(0x3DL).deterministic(true);
                    break;
                case SLH_DSA_SHA2_128F: b.cryptographicAlgorithm(0x40L); break;
            }
            return b.build();
        }
    }

    private static void ecdsa(CryptographicParameters.Builder b, long dsa, long hash) {
        b.cryptographicAlgorithm(6L).digitalSignatureAlgorithm(dsa).hashingAlgorithm(hash);
    }

    private static void pss(CryptographicParameters.Builder b, long hash, int saltLen) {
        b.cryptographicAlgorithm(4L).paddingMethod(0x0AL).hashingAlgorithm(hash)
                .maskGenerator(1L).maskGeneratorHashingAlgorithm(hash).saltLength(saltLen);
    }

    private static void pkcs1(CryptographicParameters.Builder b, long dsa, long hash) {
        b.cryptographicAlgorithm(4L).digitalSignatureAlgorithm(dsa)
                .paddingMethod(8L).hashingAlgorithm(hash);
    }
}
