package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.function.Function;

/** Local KMIP 3.0 draft constructor evidence from the installed Java/JNI package. */
final class DraftCampaign {
    private static final ObjectMapper JSON = new ObjectMapper();
    private static final String PREFIX = "org.kmipkit.extensions.ClientBatchItem.";
    private static final Set<String> OPTIONS = Set.of(
            "--plan", "--matrix-sha256", "--scope", "--evidence", "--profile");

    private record Constructor(Function<TtlvStructure, ClientBatchItem> make, int code) {
        boolean check() {
            try (var payload = TtlvStructure.create(); var item = make.apply(payload)) {
                return item.operation() == code;
            }
        }
    }

    private static final Map<String, Constructor> ROUTES = Map.of(
            PREFIX + "createGroup", new Constructor(ClientBatchItem::createGroup, 0x3c),
            PREFIX + "obliterate", new Constructor(ClientBatchItem::obliterate, 0x3d),
            PREFIX + "createUser", new Constructor(ClientBatchItem::createUser, 0x3e),
            PREFIX + "createCredential", new Constructor(ClientBatchItem::createCredential, 0x3f),
            PREFIX + "deactivate", new Constructor(ClientBatchItem::deactivate, 0x40),
            PREFIX + "encapsulate", new Constructor(ClientBatchItem::encapsulate, 0x41),
            PREFIX + "decapsulate", new Constructor(ClientBatchItem::decapsulate, 0x42));

    private DraftCampaign() { }

    static int run(String[] args, Path manifest) throws IOException {
        Map<String, String> options = parseOptions(args);
        var artifact = ArtifactProvenance.loadForCheckout(manifest);
        if (!artifact.artifactSha256().equals(ArtifactProvenance.sha256(loadedJar()))) {
            throw new IllegalArgumentException("The loaded KMIPKit JAR differs from the verified artifact");
        }
        var events = JSON.createArrayNode();
        boolean allPassed = true;
        try (var reader = Files.newBufferedReader(Path.of(options.get("--plan")))) {
            String line;
            while ((line = reader.readLine()) != null) {
                var row = JSON.readTree(line);
                if (!"java".equals(row.path("language").asText())
                        || !"3.0".equals(row.path("protocol_version").asText())
                        || !"public_route".equals(row.path("kind").asText())
                        || !"construct".equals(row.path("action").asText())
                        || !"required".equals(row.path("execution_status").asText())) {
                    continue;
                }
                String symbol = row.path("public_api_symbol").asText();
                Constructor route = ROUTES.get(symbol);
                if (route == null) continue;
                String id = row.path("obligation_id").asText();
                if (!id.matches("[0-9a-f]{64}")) throw new IllegalArgumentException("Invalid obligation ID");
                boolean passed = route.check();
                allPassed &= passed;
                events.addObject().put("obligation_id", id).put("public_api_symbol", symbol)
                        .put("status", passed ? "passed" : "failed").put("assertion", "draft_operation_code");
            }
        }
        if (events.isEmpty()) throw new IllegalArgumentException("The plan has no supported draft routes");
        var result = JSON.createObjectNode().put("schema_version", "1.0").put("scope", "local")
                .put("language", "java").put("run_id", UUID.randomUUID().toString())
                .put("library_commit", artifact.libraryCommit()).put("artifact_sha256", artifact.artifactSha256())
                .put("matrix_sha256", options.get("--matrix-sha256"));
        result.set("events", events);
        try (var writer = Files.newBufferedWriter(Path.of(options.get("--evidence")))) {
            JSON.writeValue(writer, result);
        }
        return allPassed ? 0 : 1;
    }

    private static Path loadedJar() throws IOException {
        try {
            return Path.of(ClientBatchItem.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
        } catch (java.net.URISyntaxException exception) {
            throw new IOException("The loaded KMIPKit JAR location is invalid", exception);
        }
    }

    private static Map<String, String> parseOptions(String[] args) {
        var options = new HashMap<String, String>();
        for (int index = 0; index < args.length; index += 2) {
            String key = args[index];
            if (!OPTIONS.contains(key) || index + 1 >= args.length || options.putIfAbsent(key, args[index + 1]) != null) {
                throw new IllegalArgumentException("Unknown, duplicated or incomplete campaign option");
            }
        }
        if (!"local".equals(options.get("--scope")) || !options.containsKey("--plan")
                || !options.containsKey("--evidence") || !options.getOrDefault("--matrix-sha256", "").matches("[0-9a-f]{64}")) {
            throw new IllegalArgumentException("A local campaign requires a plan, fingerprint and evidence path");
        }
        return options;
    }
}
