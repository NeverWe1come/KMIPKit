package org.kmipkit.validator;

import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class ConnectionConfigTest {
    private static final Path EXAMPLE = Path.of("../shared/profiles/connection.example.json");

    @Test
    void acceptsTheSharedDedicatedEnvironmentContract() throws Exception {
        var connection = ConnectionConfig.load(EXAMPLE);
        assertDoesNotThrow(() -> connection.requireLiveExecution(false));
        assertEquals("Unspecified KMIP server", connection.serverProduct());
        assertEquals("not-recorded", connection.serverVersion());
    }

    @Test
    void rejectsUnconfirmedEnvironmentAndMalformedFingerprint() throws Exception {
        String example = Files.readString(EXAMPLE);
        Path unsafe = Path.of("build/unsafe-connection.json");
        Files.createDirectories(unsafe.getParent());
        Files.writeString(unsafe, example.replace("\"dedicated_test_environment_confirmed\": true",
                "\"dedicated_test_environment_confirmed\": false"));
        assertThrows(IllegalArgumentException.class, () -> ConnectionConfig.load(unsafe));

        Path badFingerprint = Path.of("build/bad-fingerprint-connection.json");
        Files.writeString(badFingerprint, example.replace(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", "not-a-digest"));
        assertThrows(IllegalArgumentException.class, () -> ConnectionConfig.load(badFingerprint));
    }

    @Test
    void mutationsRequireConfigurationAndCommandLineOptIn() throws Exception {
        var connection = ConnectionConfig.load(EXAMPLE);
        assertThrows(IllegalArgumentException.class, () -> connection.requireLiveExecution(true));
        String example = Files.readString(EXAMPLE).replace("\"allow_mutations\": false",
                "\"allow_mutations\": true");
        Path enabled = Path.of("build/mutations-enabled-connection.json");
        Files.writeString(enabled, example);
        var mutationConnection = ConnectionConfig.load(enabled);
        assertThrows(IllegalArgumentException.class, () -> mutationConnection.requireLiveExecution(true));
        assertThrows(IllegalArgumentException.class, () -> mutationConnection.requireLiveExecution(true, false));
        assertDoesNotThrow(() -> mutationConnection.requireLiveExecution(true, true));
    }
}
