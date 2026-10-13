package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.junit.jupiter.api.Test;

/** Checks that the shared parity contract can drive the Java consumer tests. */
final class SharedCapabilityParityFixtureTest {
    private static final Pattern LANGUAGES = Pattern.compile(
            "\"languages\"\\s*:\\s*\\[([^]]*)]");
    private static final Pattern CONSUMERS = Pattern.compile(
            "\"consumerTests\"\\s*:\\s*\\{([^}]*)}", Pattern.DOTALL);

    @Test
    void sharedCapabilitiesDeclareJavaConsumersBeforeJavaParityCanRun() throws IOException {
        String fixture = Files.readString(sharedFixturePath());

        Matcher languages = LANGUAGES.matcher(fixture);
        assertTrue(languages.find(), "shared capability fixture must declare its language set");
        assertTrue(languages.group(1).contains("\"java\""),
                "shared capability fixture currently has no Java language mapping");

        Matcher consumers = CONSUMERS.matcher(fixture);
        int consumerCount = 0;
        while (consumers.find()) {
            consumerCount++;
            assertTrue(consumers.group(1).contains("\"java\""),
                    "every shared capability case needs a Java consumer test");
        }
        assertTrue(consumerCount > 0, "shared capability fixture must include consumer tests");
    }

    private static Path sharedFixturePath() {
        Path workingDirectory = Path.of("").toAbsolutePath();
        Path direct = workingDirectory.resolve("tests/fixtures/api/shared-capability-parity.json");
        if (Files.isRegularFile(direct)) {
            return direct;
        }
        return workingDirectory.resolve("../../tests/fixtures/api/shared-capability-parity.json")
                .normalize();
    }
}
