package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.TtlvStructure;

final class Pkcs11MappingTest {
    @Test
    void ownerGatedFactoryAndResultAreNotPublicJavaDeclarations() {
        assertThrows(
                NoSuchMethodException.class,
                () -> ClientBatchItem.class.getDeclaredMethod(
                        "pkcs11",
                        TtlvStructure.class,
                        TtlvStructure.class,
                        byte[].class,
                        TtlvStructure.class));
        assertThrows(
                NoSuchMethodException.class,
                () -> ClientBatchItemResult.class.getDeclaredMethod("pkcs11Response"));
        assertFalse(
                java.util.Arrays.stream(ClientBatchItemResult.class.getClasses())
                        .anyMatch(type -> type.getSimpleName().equals("Pkcs11Response")));
    }

    @Test
    void candidateJniHarnessAcceptsOptionalStructuresAndCorrelationValue() {
        try (TtlvStructure function = TtlvStructure.create();
                TtlvStructure interfaceStructure = TtlvStructure.create();
                TtlvStructure inputParameters = TtlvStructure.create()) {
            java.util.List<Long> items = new java.util.ArrayList<>();
            try {
                items.add(Pkcs11NativeBridge.create(
                        function, interfaceStructure,
                        new byte[] {0x41, 0x00, (byte) 0xff}, inputParameters));
                items.add(Pkcs11NativeBridge.create(function, null, null, null));
                items.add(Pkcs11NativeBridge.create(function, null, new byte[0], null));
                assertTrue(items.stream().allMatch(item -> item > 0));
            } finally {
                items.forEach(Pkcs11NativeBridge::releaseBatchItem);
            }
        }
    }

    @Test
    void candidateJniHarnessRejectsAMissingRequiredFunction() {
        assertThrows(InvalidInputException.class,
                () -> Pkcs11NativeBridge.create(null, null, null, null));
    }

    @Test
    void pkcs11ResponseAccessorRejectsAnInvalidNativeResponseHandle() {
        assertThrows(InvalidInputException.class, () -> Pkcs11NativeBridge.responseTtlv(0));
    }
}
