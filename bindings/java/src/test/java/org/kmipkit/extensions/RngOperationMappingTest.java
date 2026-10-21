package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;

/** Four-language binding checks derived from KMIP 2.1 §§6.1.49–6.1.50. */
final class RngOperationMappingTest {
    private static final byte[] SEED = {0, (byte) 0x81, (byte) 0xff};
    private static final String SECRET_SENTINEL = "KMIP_RNG_RETRIEVE_SECRET_SENTINEL";

    @Test
    void requestBuildersPreserveSignedLengthAndSecretSeedBytes() {
        try (ClientBatchItem retrieve = ClientBatchItem.rngRetrieve(-17);
                ClientBatchItem seed = ClientBatchItem.rngSeed(SEED);
                ClientBatchItem emptySeed = ClientBatchItem.rngSeed(new byte[0])) {
            assertTrue(retrieve != null);
            assertTrue(seed != null);
            assertTrue(emptySeed != null);
        }
    }

    @Test
    void seedBuilderRejectsNullRatherThanOmittingRequiredData() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.rngSeed(null));
    }

    @Test
    void responseMappingRetainsZeroSeedLengthAndRedactsRetrievedData() {
        NativeExtensionRegistry.ensureLoaded();
        long retrieveTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        long seedTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        byte[] secret = SECRET_SENTINEL.getBytes(StandardCharsets.UTF_8);

        try (var retrieve = ClientBatchItemResult.parseRngRetrieveResponse(new Object[] {
                    new long[] {1, 1}, secret, new long[] {retrieveTtlv}
                }).orElseThrow();
                var seed = ClientBatchItemResult.parseRngSeedResponse(new Object[] {
                    new long[] {1, 1, 0}, new long[] {seedTtlv}
                }).orElseThrow()) {
            assertArrayEquals(SECRET_SENTINEL.getBytes(StandardCharsets.UTF_8), retrieve.data().orElseThrow());
            assertFalse(retrieve.toString().contains(SECRET_SENTINEL));
            assertEquals(0, seed.dataLength().orElseThrow());
            assertTrue(retrieve.responseTtlv().isPresent());
            assertTrue(seed.responseTtlv().isPresent());
        }
        assertArrayEquals(new byte[secret.length], secret);
    }

    @Test
    void malformedNativeResponseTuplesAreRejectedAndOwnedOutputsAreReleased() {
        NativeExtensionRegistry.ensureLoaded();
        assertTrue(ClientBatchItemResult.parseRngRetrieveResponse(
                new Object[] {new long[] {0, 0}, null, new long[] {0}}).isEmpty());
        assertTrue(ClientBatchItemResult.parseRngSeedResponse(
                new Object[] {new long[] {0, 0, 0}, new long[] {0}}).isEmpty());

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseRngRetrieveResponse(new Object[] {
                    new long[] {2, 0}, null, new long[] {0}}
                ));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseRngSeedResponse(new Object[] {
                    new long[] {1, 2, 0}, new long[] {0}}
                ));

        byte[] unexpectedSecret = SECRET_SENTINEL.getBytes(StandardCharsets.UTF_8);
        long retrieveTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseRngRetrieveResponse(new Object[] {
                    new long[] {0, 0}, unexpectedSecret, new long[] {retrieveTtlv}}
                ));
        assertArrayEquals(new byte[unexpectedSecret.length], unexpectedSecret);
        ResponseTtlvTestSupport.assertReleased(retrieveTtlv);

        long seedTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult
                .parseRngSeedResponse(new Object[] {
                    new long[] {0, 0, 1}, new long[] {seedTtlv}}
                ));
        ResponseTtlvTestSupport.assertReleased(seedTtlv);
    }
}
