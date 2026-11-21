package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.kmipkit.protocol.CryptographicParameters;

final class JniBoundaryCoverageTest {
    @Test
    void legacyCryptographicParametersReadFieldsMatchesTypedModel() {
        byte[] pSource = new byte[] {0x01, 0x23, (byte) 0xFE};
        try (CryptographicParameters parameters = CryptographicParameters.builder()
                .blockCipherMode(2)
                .paddingMethod(3)
                .hashingAlgorithm(4)
                .keyRoleType(5)
                .digitalSignatureAlgorithm(6)
                .cryptographicAlgorithm(7)
                .randomIv(true)
                .ivLength(12)
                .tagLength(16)
                .fixedFieldLength(8)
                .invocationFieldLength(4)
                .counterLength(8)
                .initialCounterValue(1)
                .saltLength(16)
                .maskGenerator(2)
                .maskGeneratorHashingAlgorithm(4)
                .pSource(pSource)
                .trailerField(1)
                .build()) {
            boolean[] present = new boolean[18];
            long[] values = new long[18];
            byte[] actualPSource = parameters.nativeHandleOwner().withValue(raw ->
                    NativeExtensionRegistry.cryptographicParametersReadFields(
                            raw, present, values));

            assertEquals(18, present.length);
            for (boolean fieldPresent : present) assertTrue(fieldPresent);
            assertArrayEquals(pSource, actualPSource);
            assertArrayEquals(new long[] {
                    2, 3, 4, 5, 6, 7, 1, 12, 16, 8, 4, 8, 1, 16, 2, 4,
                    pSource.length, 1
            }, values);
        }
    }

    @Test
    void nativeBridgeRejectsInvalidTypedInputsAndHandles() {
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.cryptographicParametersFromFields(null, null, null));

        boolean[] present = new boolean[18];
        long[] values = new long[18];
        values[6] = 2;
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.cryptographicParametersFromFields(
                        present, values, null));
        values[6] = 0;
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.cryptographicParametersFromFields(
                        present, values, new byte[0]));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.cryptographicParametersReadFields(
                        0, present, values));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.clientBatchSetAuthentication(0, 0));

        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseQueryFieldCount(0));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseQueryFieldAt(0, -1));

        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.managedObjectFromFields(1, null));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.keyBlockFromFields(null));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.keyValueFromFields(1, null));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.keyMaterialFromFields(1, null));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.keyWrappingDataFromFields(null));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.managedValueFieldRead(
                        0, 0, 0, 0x0042_001d, 1, null, null));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.managedValueFieldItemRead(
                        0, 0, 0, 0x0042_001d));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.attributeSetTypedAt(0, null, 0));

        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.validationStructure(0, null));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.validationRequest(0));
        assertFalse(NativeExtensionRegistry.validationReportIsValid(0));
        assertEquals(0, NativeExtensionRegistry.validationReportIssueCount(0));
        assertThrows(RuntimeException.class,
                () -> NativeExtensionRegistry.validationReportIssueAt(0, 0));
    }
}
