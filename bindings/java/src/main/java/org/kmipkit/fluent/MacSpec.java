package org.kmipkit.fluent;

import org.kmipkit.protocol.CryptographicParameters;

/** Curated Message Authentication Code (MAC) presets. */
public enum MacSpec {
    HMAC_SHA256(9L, 6L),
    HMAC_SHA384(0x0AL, 7L),
    HMAC_SHA512(0x0BL, 8L),
    HMAC_SHA3_256(0x24L, 0x0FL),
    HMAC_SHA3_512(0x26L, 0x11L),
    AES_CMAC(3L, null);

    private final long algorithm;
    private final Long hashingAlgorithm;

    MacSpec(long algorithm, Long hashingAlgorithm) {
        this.algorithm = algorithm;
        this.hashingAlgorithm = hashingAlgorithm;
    }

    /** Builds validated KMIP {@link CryptographicParameters} for this preset. */
    public CryptographicParameters toCryptographicParameters() {
        try (CryptographicParameters.Builder b = CryptographicParameters.builder()) {
            b.cryptographicAlgorithm(algorithm);
            if (hashingAlgorithm == null) {
                b.blockCipherMode(7L);
            } else {
                b.hashingAlgorithm(hashingAlgorithm);
            }
            return b.build();
        }
    }
}
