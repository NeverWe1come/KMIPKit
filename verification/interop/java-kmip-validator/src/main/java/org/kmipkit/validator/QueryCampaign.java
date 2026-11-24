package org.kmipkit.validator;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;

/** Executes the planned typed Query route against a configured KMIP server. */
final class QueryCampaign {
    private static final ObjectMapper JSON = new ObjectMapper();
    private static final String CONSTRUCTOR = "org.kmipkit.extensions.ClientBatchItem.query";
    private static final String EXECUTOR = "org.kmipkit.extensions.Client.execute";
    private static final Path CATALOG = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");

    private QueryCampaign() { }

    static int run(Map<String, String> options, ObjectNode evidence) throws IOException {
        String connectionPath = options.get("--connection");
        if (connectionPath == null) throw new IllegalArgumentException("Live campaigns require a connection");
        ConnectionConfig connection = ConnectionConfig.load(Path.of(connectionPath));
        if (!"typed-protocol".equals(connection.apiLevel())) {
            throw new IllegalArgumentException("The live Query campaign requires the typed protocol route");
        }
        connection.requireTlsFilesExist();
        String obligation = queryObligation(Path.of(options.get("--plan")), connection);
        var query = CatalogLoader.load(CATALOG).cases().stream()
                .filter(testCase -> "test-op-query".equals(testCase.caseId()))
                .findFirst().orElseThrow();
        JavaKmipDriver.CaseOutcome outcome;
        try (var driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(query);
        }

        evidence.putObject("server").put("id", connection.serverId()).put("product", connection.serverProduct())
                .put("version", connection.serverVersion()).put("configuration_fingerprint", connection.configurationFingerprint());
        String status = switch (outcome.status()) {
            case "passed" -> "passed";
            case "policy_restricted", "not_run" -> "blocked";
            default -> "failed";
        };
        var event = evidence.putArray("events").addObject().put("obligation_id", obligation)
                .put("public_api_symbol", CONSTRUCTOR).put("execution_api_symbol", EXECUTOR)
                .put("status", status).put("assertion", "query_operations_and_typed_response")
                .put("encoding", connection.encoding()).put("transport", connection.transport())
                .put("server_compatibility", outcome.serverCompatibility());
        if (outcome.resultStatusCode() != null) event.put("result_status_code", outcome.resultStatusCode());
        if (outcome.resultReasonCode() != null) event.put("result_reason_code", outcome.resultReasonCode());
        return "failed".equals(status) ? 1 : 0;
    }

    private static String queryObligation(Path plan, ConnectionConfig connection) throws IOException {
        Map<String, String> selection = Map.of(
                "language", "java", "case_id", "test-op-query", "target_id", "KMIPKIT-ELEM-OP-C2S-QUERY",
                "public_api_symbol", CONSTRUCTOR, "execution_api_symbol", EXECUTOR, "action", "send",
                "execution_status", "required", "protocol_version", connection.protocolVersion(),
                "encoding", connection.encoding(), "transport", connection.transport());
        String obligation = null;
        try (var reader = Files.newBufferedReader(plan)) {
            String line;
            while ((line = reader.readLine()) != null) {
                JsonNode row = JSON.readTree(line);
                if (!selection.entrySet().stream().allMatch(field ->
                        field.getValue().equals(row.path(field.getKey()).asText()))) {
                    continue;
                }
                if (obligation != null) throw new IllegalArgumentException("Duplicate Query obligation");
                obligation = row.path("obligation_id").asText();
            }
        }
        if (obligation == null || !obligation.matches("[0-9a-f]{64}")) {
            throw new IllegalArgumentException("The plan is missing a valid Query obligation");
        }
        return obligation;
    }
}
