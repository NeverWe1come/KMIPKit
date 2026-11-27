package org.kmipkit.fluent;

import org.kmipkit.protocol.CryptographicParameters;

/** Curated KMIP hashing algorithm presets. */
public enum HashAlgorithm {
    SHA256(6L),
    SHA384(7L),
    SHA512(8L),
    SHA3_256(0x0FL),
    SHA3_384(0x10L),
    SHA3_512(0x11L);

    private final long raw;

    HashAlgorithm(long raw) {
        this.raw = raw;
    }

    public long raw() { return raw; }

    /** Builds validated KMIP {@link CryptographicParameters} for Hash operations. */
    public CryptographicParameters toCryptographicParameters() {
        try (CryptographicParameters.Builder b = CryptographicParameters.builder()) {
            return b.hashingAlgorithm(raw).build();
        }
    }
}
