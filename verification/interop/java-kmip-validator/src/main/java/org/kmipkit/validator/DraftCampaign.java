package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.InvalidExtensionSchemaException;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import com.fasterxml.jackson.databind.node.ArrayNode;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.function.Function;

/** Local draft and generic TTLV evidence from the installed Java/JNI package. */
final class DraftCampaign {
    private static final ObjectMapper JSON = new ObjectMapper();
    private static final String PREFIX = "org.kmipkit.extensions.ClientBatchItem.";
    private static final Set<String> OPTIONS = Set.of(
            "--plan", "--matrix-sha256", "--scope", "--evidence", "--profile", "--connection");

    private record Constructor(Function<TtlvStructure, ClientBatchItem> make, int code) {
        boolean check() {
            try (var payload = TtlvStructure.create(); var item = make.apply(payload)) {
                return item.operation() == code;
            }
        }
    }

    private record Outcome(boolean passed, String assertion) { }

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
        var artifact = verifyLoadedArtifact(manifest);
        String scope = options.get("--scope");
        var result = JSON.createObjectNode().put("schema_version", "1.0").put("scope", scope)
                .put("language", "java").put("run_id", UUID.randomUUID().toString())
                .put("library_commit", artifact.libraryCommit()).put("artifact_sha256", artifact.artifactSha256())
                .put("matrix_sha256", options.get("--matrix-sha256"));
        int exit = "live".equals(scope) ? QueryCampaign.run(options, result) : local(options, result);
        try (var writer = Files.newBufferedWriter(Path.of(options.get("--evidence")))) {
            JSON.writeValue(writer, result);
        }
        return exit;
    }

    static int inspectTtlv(Path manifest) throws IOException {
        var artifact = verifyLoadedArtifact(manifest);
        System.out.println(JSON.writeValueAsString(Map.of(
                "library_commit", artifact.libraryCommit(), "artifact_sha256", artifact.artifactSha256())));
        try (var input = new BufferedReader(new InputStreamReader(System.in, java.nio.charset.StandardCharsets.US_ASCII))) {
            String line;
            while ((line = input.readLine()) != null) {
                byte[] wire = HexFormat.of().parseHex(line);
                ArrayNode nodes = JSON.createArrayNode();
                try (var item = TtlvCodec.decode(wire); var view = item.view()) {
                    appendItem(view, nodes);
                } catch (org.kmipkit.KmipException rejected) {
                    nodes = null;
                }
                System.out.println(JSON.writeValueAsString(nodes));
            }
        }
        return System.out.checkError() ? 1 : 0;
    }

    private static void appendItem(TtlvItemView view, ArrayNode nodes) {
        int raw;
        try (var tag = view.tag()) {
            raw = tag.raw();
        }
        var kind = view.itemType();
        var row = nodes.addArray().add(raw).add(kind.code());
        if (kind != TtlvItemType.Structure) {
            row.add(0);
            return;
        }
        try (var value = view.value(); var structure = value.structure()) {
            long count = structure.itemCount();
            row.add(count);
            for (long index = 0; index < count; index++) {
                try (var child = structure.itemAt(index)) {
                    appendItem(child, nodes);
                }
            }
        }
    }

    private static ArtifactProvenance verifyLoadedArtifact(Path manifest) throws IOException {
        var artifact = ArtifactProvenance.loadForCheckout(manifest);
        if (!artifact.artifactSha256().equals(ArtifactProvenance.sha256(loadedJar()))) {
            throw new IllegalArgumentException("The loaded KMIPKit JAR differs from the verified artifact");
        }
        return artifact;
    }

    private static int local(Map<String, String> options, com.fasterxml.jackson.databind.node.ObjectNode result)
            throws IOException {
        if (options.containsKey("--connection")) throw new IllegalArgumentException("Local campaigns do not take a connection");
        var events = JSON.createArrayNode();
        boolean allPassed = true;
        var tags = new HashMap<String, Boolean>();
        var catalog = Path.of("../shared/catalog/kmipkit-2x-matrix-catalog.json");
        try (var reader = Files.newBufferedReader(catalog)) {
            for (JsonNode target : JSON.readTree(reader).path("normative_targets")) {
                if (target.path("kind").asText().equals("tag")) {
                    tags.put(target.path("target_id").asText(), target.path("allocation").asText().equals("assigned"));
                }
            }
        }
        try (var reader = Files.newBufferedReader(Path.of(options.get("--plan")))) {
            String line;
            while ((line = reader.readLine()) != null) {
                var row = JSON.readTree(line);
                if (!"java".equals(row.path("language").asText())
                        || !"construct".equals(row.path("action").asText())
                        || !"required".equals(row.path("execution_status").asText())) {
                    continue;
                }
                String symbol = row.path("public_api_symbol").asText();
                Outcome outcome = localOutcome(row, symbol, tags);
                if (outcome == null) continue;
                String id = row.path("obligation_id").asText();
                if (!id.matches("[0-9a-f]{64}")) throw new IllegalArgumentException("Invalid obligation ID");
                allPassed &= outcome.passed();
                events.addObject().put("obligation_id", id).put("public_api_symbol", symbol)
                        .put("status", outcome.passed() ? "passed" : "failed").put("assertion", outcome.assertion());
            }
        }
        if (events.isEmpty()) throw new IllegalArgumentException("The plan has no supported Java local routes");
        result.set("events", events);
        return allPassed ? 0 : 1;
    }

    private static Outcome localOutcome(JsonNode row, String symbol, Map<String, Boolean> tags) {
        String version = row.path("protocol_version").asText();
        if (version.equals("3.0") && row.path("kind").asText().equals("public_route")) {
            Constructor route = ROUTES.get(symbol);
            return route == null ? null : new Outcome(route.check(), "draft_operation_code");
        }
        if ((!version.equals("2.0") && !version.equals("2.1")) || !row.path("mode").asText().equals("local")
                || !row.path("case_id").asText().equals("test-enum-tag")) {
            return null;
        }
        String target = row.path("target_id").asText();
        Boolean assigned = tags.get(target);
        if (assigned == null || !target.matches("KMIPKIT-ELEM-TAG-[0-9A-Fa-f]{6}")) return null;
        boolean checked = symbol.equals("org.kmipkit.ttlv.RawTag.checked");
        if (!checked && !symbol.equals("org.kmipkit.ttlv.RawTag.fromRaw")) return null;
        int code = Integer.parseInt(target.substring("KMIPKIT-ELEM-TAG-".length()), 16);
        String assertion = checked ? "generic_ttlv_checked_tag_value" : "generic_ttlv_raw_tag_value";
        try (var raw = RawTag.fromRaw(code)) {
            if (!checked) return new Outcome(raw.raw() == code, assertion);
            try (var tag = raw.checked()) {
                return new Outcome(assigned && tag.raw() == code, assertion);
            } catch (InvalidExtensionSchemaException rejected) {
                return new Outcome(!assigned, assertion);
            }
        } catch (RuntimeException rejected) {
            return new Outcome(false, assertion);
        }
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
        if (!Set.of("local", "live").contains(options.getOrDefault("--scope", "")) || !options.containsKey("--plan")
                || !options.containsKey("--evidence") || !options.getOrDefault("--matrix-sha256", "").matches("[0-9a-f]{64}")) {
            throw new IllegalArgumentException("A local campaign requires a plan, fingerprint and evidence path");
        }
        return options;
    }
}
