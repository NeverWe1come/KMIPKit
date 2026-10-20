package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.TtlvStructure;

final class Pkcs11MappingTest {
    @Test
    void pkcs11FactoryAndTypedResponseArePublicJavaDeclarations() throws Exception {
        assertTrue(ClientBatchItem.class.getDeclaredMethod(
                "pkcs11",
                TtlvStructure.class,
                TtlvStructure.class,
                byte[].class,
                TtlvStructure.class) != null);
        assertTrue(ClientBatchItemResult.class.getDeclaredMethod("pkcs11Response") != null);
        assertTrue(
                java.util.Arrays.stream(ClientBatchItemResult.class.getClasses())
                        .anyMatch(type -> type.getSimpleName().equals("Pkcs11Response")));
    }

    @Test
    void publicFactoryAcceptsOptionalStructuresAndCorrelationValue() {
        try (TtlvStructure function = TtlvStructure.create();
                TtlvStructure interfaceStructure = TtlvStructure.create();
                TtlvStructure inputParameters = TtlvStructure.create()) {
            try (ClientBatchItem allFields = ClientBatchItem.pkcs11(
                            function, interfaceStructure,
                            new byte[] {0x41, 0x00, (byte) 0xff}, inputParameters);
                    ClientBatchItem omitted = ClientBatchItem.pkcs11(function, null, null, null);
                    ClientBatchItem emptyCorrelation = ClientBatchItem.pkcs11(
                            function, null, new byte[0], null)) {
                assertNotNull(allFields);
                assertNotNull(omitted);
                assertNotNull(emptyCorrelation);
            }
        }
    }

    @Test
    void publicFactoryRejectsAMissingRequiredFunction() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.pkcs11(null, null, null, null));
    }

    @Test
    void pkcs11ResponseParserRejectsAnInconsistentPresentHandle() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parsePkcs11Response(
                        new Object[] {new long[] {1}, new long[] {0}}));
    }
}
