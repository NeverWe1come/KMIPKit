package org.kmipkit.protocol;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;

final class NamedProtocolValuesTest {
    // OASIS KMIP Specification v2.1 §§11.12 and 12.1; KMIPKIT-0047-FR-001–004.
    @Test
    void namedValuesRetainUnknownRawValuesAndBits() {
        assertEquals(3L, KmipValues.CryptographicAlgorithm.AES);
        assertEquals("AES", KmipValues.CryptographicAlgorithm.name(3L));
        assertNull(KmipValues.CryptographicAlgorithm.name(0xffff_ffffL));
        assertNull(KmipValues.CryptographicAlgorithm.name(0x1_0000_0003L));

        long raw = KmipValues.CryptographicUsageMask.ENCRYPT | 0x8000_0000L;
        assertEquals(KmipValues.CryptographicUsageMask.ENCRYPT,
                KmipValues.CryptographicUsageMask.knownBits(raw));
        assertEquals(0x8000_0000L, KmipValues.CryptographicUsageMask.unknownBits(raw));
        assertEquals(0x8000_0000L,
                KmipValues.CryptographicUsageMask.unknownBits(raw | 0x1_0000_0000L));
        assertEquals(List.of("Encrypt"), KmipValues.CryptographicUsageMask.names(raw));
    }

    // OASIS KMIP Specification v2.1 §§11–12; KMIPKIT-0047-FR-001–004.
    @Test
    void everyGeneratedEnumerationValueHasANameAndPreservesUnknownValues() throws Exception {
        int enumerationCount = 0;
        int valueCount = 0;
        Set<String> emptyEnumerations = new HashSet<>();

        for (Class<?> valueClass : KmipValues.class.getDeclaredClasses()) {
            Method name;
            try {
                name = valueClass.getMethod("name", long.class);
            } catch (NoSuchMethodException exception) {
                continue;
            }

            enumerationCount++;
            assertTrue(Modifier.isStatic(name.getModifiers()));
            assertEquals(String.class, name.getReturnType());

            int classValueCount = 0;
            for (Field field : valueClass.getDeclaredFields()) {
                if (field.getType() != long.class
                        || !Modifier.isStatic(field.getModifiers())
                        || !Modifier.isPublic(field.getModifiers())) {
                    continue;
                }

                long raw = field.getLong(null);
                assertNotNull(name.invoke(null, raw), valueClass.getSimpleName() + "." + field.getName());
                classValueCount++;
            }

            if (classValueCount == 0) {
                emptyEnumerations.add(valueClass.getSimpleName());
            }
            assertNull(name.invoke(null, -1L), valueClass.getSimpleName());
            assertNull(name.invoke(null, 0x1_0000_0000L), valueClass.getSimpleName());
            valueCount += classValueCount;
        }

        assertEquals(64, enumerationCount);
        assertEquals(644, valueCount);
        assertEquals(
                Set.of("OpaqueDataType", "Pkcs11Function", "Pkcs11ReturnCode", "Tag"),
                emptyEnumerations);
    }
}
