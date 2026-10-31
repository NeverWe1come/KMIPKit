package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.lang.reflect.Method;
import org.junit.jupiter.api.Test;

final class CredentialTypeParityTest {
    @Test
    void fromRawPreservesKnownExtensionAndUnknownValues() throws ReflectiveOperationException {
        Class<?> facade = assertDoesNotThrow(
                () -> Class.forName("org.kmipkit.credentials.CredentialType"));
        Method fromRaw = facade.getMethod("fromRaw", long.class);
        Method raw = facade.getMethod("raw");
        Method kind = facade.getMethod("kind");

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
}
