package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MainContractTest {
    @TempDir
    Path temporaryDirectory;

    @Test
    void offlinePlanPreflightAndRunClassifyLocalCasesWithoutServerClaims() throws Exception {
        Path connection = connection();
        Path catalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        Path manifest = Path.of(System.getProperty("kmipkit.artifactManifest"));
        Path output = Path.of(System.getProperty("kmipkit.reportOutputDir",
                temporaryDirectory.resolve("report").toString()));

        String plan = assertCommandOutput(0, "plan", "--connection", connection.toString(),
                "--catalog", catalog.toString());
        assertTrue(plan.contains("Server exchanges executable through selected Java API level: 0"));
        assertCommand(0, "preflight", "--connection", connection.toString(), "--catalog", catalog.toString(),
                "--artifact-manifest", manifest.toString());
        assertCommand(0, "run", "--connection", connection.toString(), "--catalog", catalog.toString(),
                "--artifact-manifest", manifest.toString(), "--output-dir", output.toString());

        JsonNode report = new ObjectMapper().readTree(output.resolve("results.json").toFile());
        assertEquals(212, report.path("summary").path("total").asInt());
        assertEquals(67, report.path("summary").path("passed").asInt());
        assertEquals(132, report.path("summary").path("policy_restricted").asInt());
        assertEquals(13, report.path("summary").path("not_run").asInt());
        for (JsonNode result : report.path("cases")) {
            if ("policy_restricted".equals(result.path("status").asText())) {
                assertEquals("unknown", result.path("server_compatibility").asText());
            }
            if (result.path("case_id").asText().startsWith("test-enum-")
                    || result.path("case_id").asText().startsWith("test-bitmask-")) {
                assertEquals("unknown", result.path("server_compatibility").asText());
            }
        }
        assertTrue(Files.isRegularFile(output.resolve("report.md")));
        assertTrue(Files.isRegularFile(output.resolve("report.html")));
    }

    @Test
    void planDoesNotCountKmip20RoutesBlockedByThePinnedArtifact() throws Exception {
        Path connection = connection("2.0", "typed-protocol");
        String plan = assertCommandOutput(0, "plan", "--connection", connection.toString(),
                "--catalog", "../shared/catalog/kmipkit-1.0-test-catalog.json");
        assertTrue(plan.contains("Server exchanges executable through selected Java API level: 0"));
        assertTrue(plan.contains("KMIP 2.0 live routes are blocked by the pinned artifact"));
    }

    @Test
    void serverInitiatedQueryRemainsPendingWithoutOpeningAClientConnection() throws Exception {
        Path connection = connection("2.1", "typed-protocol");
        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        JsonNode serverQuery = catalog.path("test_cases").findParents("case_id").stream()
                .filter(node -> "test-s2c-query".equals(node.path("case_id").asText()))
                .findFirst().orElseThrow();
        var oneCase = mapper.createArrayNode().add(serverQuery.deepCopy());
        catalog.set("test_cases", oneCase);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", 1);
        Path catalogFile = temporaryDirectory.resolve("server-query-catalog.json");
        mapper.writeValue(catalogFile.toFile(), catalog);
        Path manifest = Path.of(System.getProperty("kmipkit.artifactManifest"));
        Path output = temporaryDirectory.resolve("server-query-report");

        assertCommand(0, "run", "--connection", connection.toString(), "--catalog", catalogFile.toString(),
                "--artifact-manifest", manifest.toString(), "--output-dir", output.toString());

        JsonNode report = mapper.readTree(output.resolve("results.json").toFile());
        JsonNode result = report.path("cases").get(0);
        assertEquals("not_run", result.path("status").asText());
        assertEquals("unknown", result.path("server_compatibility").asText());
        assertTrue(result.path("diagnostic").asText().contains("server-initiated"));
    }

    private Path connection() throws Exception {
        return connection("2.1", "high-level");
    }

    private Path connection(String protocolVersion, String apiLevel) throws Exception {
        Path trust = temporaryDirectory.resolve("trust.pem");
        Path client = temporaryDirectory.resolve("client.pem");
        Path key = temporaryDirectory.resolve("client-key.pem");
        Files.writeString(trust, "preflight-only placeholder");
        Files.writeString(client, "preflight-only placeholder");
        Files.writeString(key, "preflight-only placeholder");
        ObjectNode root = new ObjectMapper().createObjectNode();
        root.put("server_id", "no-network-target");
        root.put("server_configuration_fingerprint", "c".repeat(64));
        root.put("protocol_version", protocolVersion);
        root.put("transport", "raw_tls");
        root.put("encoding", "TTLV");
        root.put("api_level", apiLevel);
        root.putObject("endpoints").putObject("raw_tls")
                .put("host", "no-dns-lookup.invalid").put("port", 5696);
        root.putObject("tls").put("ca_cert_path", trust.toString())
                .put("client_cert_path", client.toString()).put("client_key_path", key.toString());
        root.putObject("environment").put("kind", "dedicated_test")
                .put("dedicated_test_environment_confirmed", true);
        root.putObject("execution_policy").put("allow_mutations", false);
        Path file = temporaryDirectory.resolve("connection.json");
        new ObjectMapper().writeValue(file.toFile(), root);
        return file;
    }

    private static void assertCommand(int expectedStatus, String... args) {
        assertCommandOutput(expectedStatus, args);
    }

    private static String assertCommandOutput(int expectedStatus, String... args) {
        ByteArrayOutputStream output = new ByteArrayOutputStream();
        ByteArrayOutputStream error = new ByteArrayOutputStream();
        int status = Main.execute(args, new PrintStream(output), new PrintStream(error));
        assertEquals(expectedStatus, status, error.toString());
        return output.toString();
    }
}
