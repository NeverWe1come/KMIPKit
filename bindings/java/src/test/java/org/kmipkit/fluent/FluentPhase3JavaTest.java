package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.*;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.protocol.CryptographicParameters;

final class FluentPhase3JavaTest {
    @Test
    void keySpecMetadataCoversClassicalAndPqc() {
        assertTrue(KeySpec.AES_256.isSymmetric());
        assertEquals(256, KeySpec.AES_256.length());
        assertEquals(7L, KeySpec.ECDSA_P256.recommendedCurve().orElseThrow());
        assertTrue(KeySpec.ML_KEM_768.requiresKmip30());
        assertFalse(KeySpec.RSA_2048.requiresKmip30());
        assertTrue(KeySpec.RSA_2048.isAsymmetric());
    }

    @Test
    void allPresetEnumsProduceValidCryptographicParameters() {
        for (CipherSpec spec : CipherSpec.values()) {
            try (CryptographicParameters p = spec.toCryptographicParameters()) {
                assertNotNull(p);
            }
        }
        for (SignatureSpec spec : SignatureSpec.values()) {
            try (CryptographicParameters p = spec.toCryptographicParameters()) {
                assertNotNull(p);
            }
        }
        for (MacSpec spec : MacSpec.values()) {
            try (CryptographicParameters p = spec.toCryptographicParameters()) {
                assertNotNull(p);
            }
        }
        for (HashAlgorithm spec : HashAlgorithm.values()) {
            try (CryptographicParameters p = spec.toCryptographicParameters()) {
                assertNotNull(p);
            }
        }
        for (KemSpec spec : KemSpec.values()) {
            try (CryptographicParameters p = spec.toCryptographicParameters()) {
                assertNotNull(p);
            }
        }
        for (KdfSpec spec : KdfSpec.values()) {
            try (CryptographicParameters p = spec.toCryptographicParameters()) {
                assertNotNull(p);
            }
        }
    }

    @Test
    void buildersValidateKeySpecCategoryAndBuildPresets() {
        try (SymmetricKeyBuilder sym = new SymmetricKeyBuilder(null).withKeySpec(KeySpec.AES_256)) {
            assertNotNull(sym);
        }
        assertThrows(InvalidInputException.class,
                () -> new SymmetricKeyBuilder(null).withKeySpec(KeySpec.RSA_2048));
        assertThrows(InvalidInputException.class,
                () -> new CreateKeyPairBuilder(null).withKeySpec(KeySpec.AES_256));
        try (CreateKeyPairBuilder pair = new CreateKeyPairBuilder(null).withKeySpec(KeySpec.ML_KEM_768)) {
            assertNotNull(pair);
        }
    }
}
