package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;

final class DraftCampaignTest {
    @TempDir Path temp;

    @Test
    void recordsAllSevenOperationsFromTheInstalledArtifact() throws Exception {
        var mapper = new ObjectMapper();
        var plan = temp.resolve("plan.jsonl");
        var report = temp.resolve("evidence.json");
        List<String> names = List.of("createGroup", "obliterate", "createUser", "createCredential",
                "deactivate", "encapsulate", "decapsulate");
        try (var output = Files.newBufferedWriter(plan)) {
            for (int index = 0; index < names.size(); index++) {
                output.write(mapper.writeValueAsString(Map.of(
                        "language", "java", "protocol_version", "3.0", "kind", "public_route",
                        "action", "construct", "execution_status", "required",
                        "obligation_id", String.format("%064x", index),
                        "public_api_symbol", "org.kmipkit.extensions.ClientBatchItem." + names.get(index))));
                output.newLine();
            }
        }
        var manifest = Path.of(System.getProperty("kmipkit.artifactManifest"));
        int exit = DraftCampaign.run(new String[] {
                "--plan", plan.toString(), "--matrix-sha256", "a".repeat(64),
                "--scope", "local", "--evidence", report.toString()
        }, manifest);
        assertEquals(0, exit);
        var events = mapper.readTree(report.toFile()).path("events");
        assertEquals(names.size(), events.size());
        for (var event : events) assertEquals("passed", event.path("status").asText());
    }
}
