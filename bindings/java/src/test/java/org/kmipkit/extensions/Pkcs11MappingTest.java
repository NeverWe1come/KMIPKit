package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.TtlvStructure;

final class Pkcs11MappingTest {
    @Test
    void pkcs11FactoryAcceptsOptionalGenericStructuresAndCorrelationValue() {
        try (TtlvStructure function = TtlvStructure.create();
                TtlvStructure interfaceStructure = TtlvStructure.create();
                TtlvStructure inputParameters = TtlvStructure.create();
                ClientBatchItem populated = ClientBatchItem.pkcs11(
                        function, interfaceStructure, new byte[] {0x41, 0x00, (byte) 0xff},
                        inputParameters);
                ClientBatchItem omitted = ClientBatchItem.pkcs11(function, null, null, null);
                ClientBatchItem presentEmptyCorrelation = ClientBatchItem.pkcs11(
                        function, null, new byte[0], null)) {
            assertEquals(0, populated.extensionCount());
            assertEquals(0, omitted.extensionCount());
            assertEquals(0, presentEmptyCorrelation.extensionCount());
        }
    }

    @Test
    void pkcs11FactoryRequiresFunctionAndBoundsCorrelationBytes() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.pkcs11(null, null, null, null));
        try (TtlvStructure function = TtlvStructure.create()) {
            assertThrows(ResourceLimitException.class,
                    () -> ClientBatchItem.pkcs11(function, null,
                            new byte[16 * 1024 * 1024 + 1], null));
        }
    }

    @Test
    void pkcs11ResponseParserPreservesAbsentAndRejectsInvalidOwnedHandle() {
        assertFalse(ClientBatchItemResult.parsePkcs11Response(0).isPresent());
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parsePkcs11Response(-1));
    }

    @Test
    void pkcs11ResponseAccessorRejectsAnInvalidNativeResponseHandle() {
        assertThrows(InvalidInputException.class, () -> Pkcs11NativeBridge.responseTtlv(0));
    }
}
