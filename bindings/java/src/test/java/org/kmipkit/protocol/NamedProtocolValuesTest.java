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
    // OASIS KMIP Specification v2.1 §§11.12 and 12.1; named protocol values.
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

    // KMIP 2.1 §§11.36 and 11.56 Tables 470 and 487; KMIP 3.0 §§11.37 and 11.58 Tables 584 and 605.
    @Test
    void versionedNamedValuesMatchExactEditionTables() {
        assertEquals(0x36L, KmipValues.V2_1_OPERATION_SET_DEFAULTS);
        assertEquals("Ping", KmipValues.v2_1OperationName(KmipValues.V2_1_OPERATION_PING));
        assertEquals(0x3DL, KmipValues.V3_0_OPERATION_OBLITERATE);
        assertEquals("Create Credential",
                KmipValues.v3_0OperationName(KmipValues.V3_0_OPERATION_CREATE_CREDENTIAL));
        assertEquals("User", KmipValues.v3_0ObjectClassName(1));
        assertNull(KmipValues.v3_0ObjectClassName(3));
        assertEquals("Password Credential", KmipValues.v3_0ObjectTypeName(0x0D));
        assertNull(KmipValues.v3_0ObjectTypeName(6));
        assertEquals(0x00420190L, KmipValues.V3_0_TAG_CERTIFICATE_LINK);
        assertEquals("Certificate Link", KmipValues.v3_0TagName(0x00420190L));
        assertNull(KmipValues.v3_0TagName(0x0042018F));
        assertNull(KmipValues.v3_0OperationName(0x43));
    }

    // OASIS KMIP Specification v2.1 §§11–12; named protocol values and named tag values.
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
        assertEquals(998, valueCount);
        assertEquals(
                Set.of("OpaqueDataType", "Pkcs11Function", "Pkcs11ReturnCode"),
                emptyEnumerations);
        assertEquals(0x00420001L, KmipValues.Tag.ACTIVATION_DATE);
        assertEquals("Activation Date", KmipValues.Tag.name(0x00420001L));
        assertNull(KmipValues.Tag.name(0x0042018F));
    }
}
