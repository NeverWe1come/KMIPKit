package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;

final class JniBoundaryCoverageTest {
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
