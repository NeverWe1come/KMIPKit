package org.kmipkit.fluent;

import org.kmipkit.protocol.CryptographicParameters;

/** Curated KMIP 3.0 Key Encapsulation Mechanism (KEM) presets. */
public enum KemSpec {
    ML_KEM_512(3L, 0x39L),
    ML_KEM_768(3L, 0x3AL),
    ML_KEM_1024(3L, 0x3BL),
    DHKEM(2L, null),
    RSASVE(1L, null),
    X25519_MLKEM768(3L, 0x5CL),
    SECP256R1_MLKEM768(3L, 0x5DL);

    private final long kemAlgorithm;
    private final Long cryptographicAlgorithm;

    KemSpec(long kemAlgorithm, Long cryptographicAlgorithm) {
        this.kemAlgorithm = kemAlgorithm;
        this.cryptographicAlgorithm = cryptographicAlgorithm;
    }

    public long kemAlgorithm() { return kemAlgorithm; }
    public int minKmipMajor() { return 3; }

    /** Builds validated KMIP {@link CryptographicParameters} for Encapsulate/Decapsulate. */
    public CryptographicParameters toCryptographicParameters() {
        try (CryptographicParameters.Builder b = CryptographicParameters.builder()) {
            b.kemAlgorithm(kemAlgorithm);
            if (cryptographicAlgorithm != null) {
                b.cryptographicAlgorithm(cryptographicAlgorithm);
            }
            return b.build();
        }
    }
}
