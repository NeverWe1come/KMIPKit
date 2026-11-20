package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.PropertyNamingStrategies;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.Set;

/** Writes sanitized JSON, Markdown, and HTML evidence using the shared report contract. */
public final class MatrixReporter {
    private static final Set<String> STATUSES = Set.of(
            "passed", "failed", "server_unsupported", "policy_restricted", "not_run");
    private static final Set<String> CORRECTNESS = Set.of("passed", "failed", "not_checked");
    private static final Set<String> SERVER_COMPATIBILITY = Set.of(
            "supported", "unsupported", "policy_restricted", "unknown");
    private static final Set<String> PARITY = Set.of("matched", "mismatched", "not_compared");
    private static final ObjectMapper JSON = new ObjectMapper()
            .setPropertyNamingStrategy(PropertyNamingStrategies.SNAKE_CASE);

    private MatrixReporter() { }

    /** Validates the shared contract and renders its canonical JSON shape. */
    public static String toJson(RunReport report) {
        validate(report);
        try {
            return JSON.writerWithDefaultPrettyPrinter().writeValueAsString(report) + "\n";
        } catch (IOException exception) {
            throw new IllegalStateException("Run report could not be serialized", exception);
        }
    }

    /** Writes all three shared report formats to the requested directory. */
    public static void writeReports(RunReport report, Path outputDirectory) throws IOException {
        Files.createDirectories(outputDirectory);
        Files.writeString(outputDirectory.resolve("results.json"), toJson(report));
        Files.writeString(outputDirectory.resolve("report.md"), toMarkdown(report));
        Files.writeString(outputDirectory.resolve("report.html"), toHtml(report));
    }

    public static String toMarkdown(RunReport report) {
        validate(report);
        StringBuilder output = new StringBuilder("# KMIP interoperability results\n\n");
        output.append("- **Server:** `").append(markdown(report.server().id())).append("`\n");
        output.append("- **Protocol:** KMIP ").append(markdown(report.protocolVersion())).append("\n");
        output.append("- **Language:** `Java`\n");
        output.append("- **Library commit:** `").append(report.execution().libraryCommit()).append("`\n\n");
        output.append("| Total | Passed | Failed | Unsupported | Policy restricted | Not run |\n");
        output.append("|---:|---:|---:|---:|---:|---:|\n");
        RunReport.Summary summary = report.summary();
        output.append("| ").append(summary.total()).append(" | ").append(summary.passed()).append(" | ")
                .append(summary.failed()).append(" | ").append(summary.unsupported()).append(" | ")
                .append(summary.policyRestricted()).append(" | ").append(summary.notRun()).append(" |\n\n");
        output.append("| Case | Status | Library correctness | Server compatibility | Parity |\n");
        output.append("|---|---|---|---|---|\n");
        for (RunReport.CaseEvidence evidence : report.cases()) {
            output.append("| `").append(markdown(evidence.caseId())).append("` | ")
                    .append(evidence.status()).append(" | ").append(evidence.libraryCorrectness())
                    .append(" | ").append(evidence.serverCompatibility()).append(" | ")
                    .append(evidence.parity()).append(" |\n");
        }
        return output.toString();
    }

    public static String toHtml(RunReport report) {
        validate(report);
        StringBuilder output = new StringBuilder("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">");
        output.append("<title>KMIP interoperability results</title><h1>KMIP interoperability results</h1>");
        output.append("<p>Server <code>").append(html(report.server().id())).append("</code>; protocol KMIP ")
                .append(html(report.protocolVersion())).append("; Java.</p>");
        output.append("<table><thead><tr><th>Case</th><th>Status</th><th>Library</th><th>Server</th><th>Parity</th>")
                .append("</tr></thead><tbody>");
        for (RunReport.CaseEvidence evidence : report.cases()) {
            output.append("<tr><td><code>").append(html(evidence.caseId())).append("</code></td><td>")
                    .append(evidence.status()).append("</td><td>").append(evidence.libraryCorrectness())
                    .append("</td><td>").append(evidence.serverCompatibility()).append("</td><td>")
                    .append(evidence.parity()).append("</td></tr>");
        }
        return output.append("</tbody></table></html>\n").toString();
    }

    private static void validate(RunReport report) {
        require(report != null, "Run report is required");
        require("2.0".equals(report.schemaVersion()), "Report schema_version must be 2.0");
        text(report.runId(), "run_id");
        try {
            Instant.parse(report.testedAt());
        } catch (RuntimeException exception) {
            throw new IllegalArgumentException("Report tested_at must be an ISO-8601 date-time");
        }
        require("java".equals(report.language()), "Report language must be java");
        require(report.server() != null, "Report server is required");
        text(report.server().id(), "server.id");
        text(report.server().product(), "server.product");
        text(report.server().version(), "server.version");
        digest(report.server().configurationFingerprint(), "server.configuration_fingerprint");
        require(Set.of("2.0", "2.1").contains(report.protocolVersion()), "Report protocol_version is invalid");
        digest(report.catalogSha256(), "catalog_sha256");
        require(report.execution() != null, "Report execution is required");
        require(Set.of("raw_tls", "https").contains(report.execution().transport()), "Report transport is invalid");
        require(Set.of("TTLV", "JSON", "XML").contains(report.execution().encoding()), "Report encoding is invalid");
        require(Set.of("high-level", "typed-protocol", "generic-ttlv").contains(report.execution().apiLevel()),
                "Report api_level is invalid");
        require(report.execution().libraryCommit() != null
                && report.execution().libraryCommit().matches("[0-9a-f]{7,40}"),
                "Report library_commit is invalid");
        digest(report.execution().artifactSha256(), "execution.artifact_sha256");
        require(report.summary() != null, "Report summary is required");
        int total = report.cases().size();
        require(report.summary().total() == total, "Report summary.total does not match cases");
        int passed = 0;
        int failed = 0;
        int unsupported = 0;
        int policyRestricted = 0;
        int notRun = 0;
        for (RunReport.CaseEvidence evidence : report.cases()) {
            text(evidence.caseId(), "case.case_id");
            require(STATUSES.contains(evidence.status()), "Report case status is invalid");
            require(CORRECTNESS.contains(evidence.libraryCorrectness()), "Report library correctness is invalid");
            require(SERVER_COMPATIBILITY.contains(evidence.serverCompatibility()), "Report server compatibility is invalid");
            require(PARITY.contains(evidence.parity()), "Report parity is invalid");
            require(Double.isFinite(evidence.durationMs()) && evidence.durationMs() >= 0,
                    "Report case duration_ms is invalid");
            require(evidence.resultStatusCode() == null || (evidence.resultStatusCode() >= 0
                    && evidence.resultStatusCode() <= 0xffff_ffffL), "Report result_status_code is invalid");
            require(evidence.resultReasonCode() == null || (evidence.resultReasonCode() >= 0
                    && evidence.resultReasonCode() <= 0xffff_ffffL), "Report result_reason_code is invalid");
            switch (evidence.status()) {
                case "passed" -> passed++;
                case "failed" -> failed++;
                case "server_unsupported" -> unsupported++;
                case "policy_restricted" -> policyRestricted++;
                case "not_run" -> notRun++;
                default -> throw new IllegalArgumentException("Report case status is invalid");
            }
        }
        require(report.summary().passed() == passed && report.summary().failed() == failed
                        && report.summary().unsupported() == unsupported
                        && report.summary().policyRestricted() == policyRestricted
                        && report.summary().notRun() == notRun,
                "Report summary counts do not match cases");
    }

    private static String text(String value, String field) {
        require(value != null && !value.isBlank(), "Report " + field + " is required");
        return value;
    }

    private static void digest(String value, String field) {
        require(value != null && value.matches("[a-f0-9]{64}"), "Report " + field + " is invalid");
    }

    private static String markdown(String value) {
        return value.replace("|", "\\|").replace("\n", " ").replace("\r", " ");
    }

    private static String html(String value) {
        return value.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
                .replace("\"", "&quot;").replace("'", "&#39;");
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new IllegalArgumentException(message);
    }
}
