package org.kmipkit.fluent;

import org.kmipkit.protocol.CryptographicParameters;

/** Curated encryption and decryption cipher parameter presets. */
public enum CipherSpec {
    AES_CBC_PKCS5,
    AES_CBC_NO_PADDING,
    AES_GCM,
    AES_CTR,
    CHACHA20_POLY1305,
    RSA_OAEP_SHA256,
    RSA_OAEP_SHA384,
    RSA_OAEP_SHA512,
    RSA_PKCS1_V1_5;

    /** Builds validated KMIP {@link CryptographicParameters} for this preset. */
    public CryptographicParameters toCryptographicParameters() {
        try (CryptographicParameters.Builder b = CryptographicParameters.builder()) {
            switch (this) {
                case AES_CBC_PKCS5:
                    b.cryptographicAlgorithm(3L).blockCipherMode(1L).paddingMethod(3L);
                    break;
                case AES_CBC_NO_PADDING:
                    b.cryptographicAlgorithm(3L).blockCipherMode(1L);
                    break;
                case AES_GCM:
                    b.cryptographicAlgorithm(3L).blockCipherMode(9L)
                            .randomIv(true).ivLength(96).tagLength(128);
                    break;
                case AES_CTR:
                    b.cryptographicAlgorithm(3L).blockCipherMode(6L)
                            .randomIv(true).ivLength(128);
                    break;
                case CHACHA20_POLY1305:
                    b.cryptographicAlgorithm(0x1EL);
                    break;
                case RSA_OAEP_SHA256:
                    oaep(b, 6L);
                    break;
                case RSA_OAEP_SHA384:
                    oaep(b, 7L);
                    break;
                case RSA_OAEP_SHA512:
                    oaep(b, 8L);
                    break;
                case RSA_PKCS1_V1_5:
                    b.cryptographicAlgorithm(4L).paddingMethod(8L);
                    break;
            }
            return b.build();
        }
    }

    private static void oaep(CryptographicParameters.Builder b, long hash) {
        b.cryptographicAlgorithm(4L).paddingMethod(2L).hashingAlgorithm(hash)
                .maskGenerator(1L).maskGeneratorHashingAlgorithm(hash);
    }
}
