package org.kmipkit.fluent;

import org.kmipkit.protocol.CryptographicParameters;

/** Curated Key Derivation Function (KDF) presets. */
public enum KdfSpec {
    HKDF_SHA256(0x0AL, HashAlgorithm.SHA256),
    HKDF_SHA384(0x0AL, HashAlgorithm.SHA384),
    HKDF_SHA512(0x0AL, HashAlgorithm.SHA512),
    PBKDF2_SHA256(1L, HashAlgorithm.SHA256),
    NIST800_108_COUNTER_HMAC_SHA256(5L, HashAlgorithm.SHA256),
    HMAC_SHA256(3L, HashAlgorithm.SHA256);

    private final long derivationMethod;
    private final HashAlgorithm hashAlgorithm;

    KdfSpec(long derivationMethod, HashAlgorithm hashAlgorithm) {
        this.derivationMethod = derivationMethod;
        this.hashAlgorithm = hashAlgorithm;
    }

    public long derivationMethod() { return derivationMethod; }
    public HashAlgorithm hashAlgorithm() { return hashAlgorithm; }

    /** Builds validated {@link CryptographicParameters} for the KDF hash function. */
    public CryptographicParameters toCryptographicParameters() {
        return hashAlgorithm.toCryptographicParameters();
    }
}
