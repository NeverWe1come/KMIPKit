package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.lang.reflect.Method;
import org.kmipkit.InvalidInputException;
import org.junit.jupiter.api.Test;

final class CredentialTypeParityTest {
    @Test
    void fromRawPreservesKnownExtensionAndUnknownValues() throws ReflectiveOperationException {
        Class<?> facade = assertDoesNotThrow(
                () -> Class.forName("org.kmipkit.credentials.CredentialType"));
        Method fromRaw = facade.getMethod("fromRaw", long.class);
        Method raw = facade.getMethod("raw");
        Method kind = facade.getMethod("kind");

        assertEquals(1L, facade.getField("USERNAME_AND_PASSWORD").getLong(null));
        assertEquals(2L, facade.getField("DEVICE").getLong(null));
        assertEquals(3L, facade.getField("ATTESTATION").getLong(null));
        assertEquals(4L, facade.getField("ONE_TIME_PASSWORD").getLong(null));
        assertEquals(5L, facade.getField("HASHED_PASSWORD").getLong(null));
        assertEquals(6L, facade.getField("TICKET").getLong(null));

        Object[][] cases = {
            {1L, "UsernameAndPassword"},
            {2L, "Device"},
            {3L, "Attestation"},
            {4L, "OneTimePassword"},
            {5L, "HashedPassword"},
            {6L, "Ticket"},
            {0x8000_0009L, "Extensions"},
            {7L, "Unknown"},
            {0xFFFF_FFFFL, "Unknown"}
        };
        for (Object[] testCase : cases) {
            Object value = fromRaw.invoke(null, testCase[0]);
            assertNotNull(value);
            assertEquals(testCase[0], raw.invoke(value));
            assertEquals(testCase[1], kind.invoke(value));
        }
    }

    @Test
    void fromRawRejectsValuesOutsideUnsigned32BitRange() {
        assertThrows(InvalidInputException.class, () -> CredentialType.fromRaw(-1));
        assertThrows(InvalidInputException.class, () -> CredentialType.fromRaw(0x1_0000_0000L));
    }
}
