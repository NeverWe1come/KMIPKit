package org.kmipkit.validator;

import org.kmipkit.validator.models.Models.CaseResult;
import org.kmipkit.validator.models.Models.CaseStatus;
import org.kmipkit.validator.models.Models.ValidationSummary;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

public class MatrixReporter {
    public static void printAnsiTable(ValidationSummary summary) {
        renderAnsi(summary);
    }

    public static void writeReports(ValidationSummary summary, Path outputDir) throws IOException {
        renderMarkdown(summary, outputDir.resolve("report.md"));
        renderHtml(summary, outputDir.resolve("report.html"));
        renderJson(summary, outputDir.resolve("results.json"));
    }

    public static void renderAnsi(ValidationSummary summary) {
        System.out.printf("%n=== KMIP 2.1 Validation Matrix: Server [%s] (Lang: %s) ===%n",
                summary.serverId(), summary.language());
        System.out.printf("Total: %d | ✅ Passed: %d | ❌ Failed: %d | ➖ Unsupported: %d%n",
                summary.totalCases(), summary.passed(), summary.failed(), summary.unsupported());
        System.out.println("--------------------------------------------------------------------------------");
        System.out.printf("%-35s | %-25s | %-10s%n", "Case ID", "Operation", "Status");
        System.out.println("--------------------------------------------------------------------------------");

        for (CaseResult r : summary.results()) {
            String sym = switch (r.status()) {
                case PASSED -> "✅ PASS";
                case FAILED -> "❌ FAIL";
                case UNSUPPORTED_SKIPPED, UNSUPPORTED_VERIFIED_REJECTED -> "➖ UNSUPP";
                case UNEXPECTED_SUCCESS -> "⚠️ UNEXP";
            };
            System.out.printf("%-35s | %-25s | %-10s%n", r.caseId(), r.operation(), sym);
        }
        System.out.println("--------------------------------------------------------------------------------");
    }

    public static void renderMarkdown(ValidationSummary summary, Path path) throws IOException {
        StringBuilder sb = new StringBuilder();
        sb.append(String.format("# KMIP 2.1 Conformance Matrix: `%s`%n%n", summary.serverId()));
        sb.append(String.format("- **Language**: `%s`%n", summary.language()));
        sb.append(String.format("- **Total Cases**: %d%n%n", summary.totalCases()));
        sb.append("### Summary\n\n| Metric | Count |\n|---|---:|\n");
        sb.append(String.format("| Total Cases | %d |%n", summary.totalCases()));
        sb.append(String.format("| ✅ Passed | %d |%n", summary.passed()));
        sb.append(String.format("| ❌ Failed | %d |%n", summary.failed()));
        sb.append(String.format("| ➖ Unsupported | %d |%n%n", summary.unsupported()));

        sb.append("### Detailed Operations\n\n| Case ID | Operation | Status |\n|---|---|:---:|\n");
        for (CaseResult r : summary.results()) {
            String sym = (r.status() == CaseStatus.PASSED) ? "✅" : (r.status() == CaseStatus.FAILED) ? "❌" : "➖";
            sb.append(String.format("| `%s` | %s | %s |%n", r.caseId(), r.operation(), sym));
        }

        Files.createDirectories(path.getParent() != null ? path.getParent() : Path.of("."));
        Files.writeString(path, sb.toString());
    }

    public static void renderHtml(ValidationSummary summary, Path path) throws IOException {
        StringBuilder sb = new StringBuilder();
        sb.append("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n<title>KMIP 2.1 Validation Report</title>\n");
        sb.append("<style>\nbody { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; background: #f8fafc; color: #1e293b; }\n");
        sb.append(".card { background: white; border-radius: 8px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; }\n");
        sb.append("table { width: 100%; border-collapse: collapse; margin-top: 1rem; }\n");
        sb.append("th, td { text-align: left; padding: 0.75rem 1rem; border-bottom: 1px solid #e2e8f0; }\n");
        sb.append("th { background: #f1f5f9; font-weight: 600; }\n");
        sb.append(".pass { color: #16a34a; font-weight: bold; }\n.fail { color: #dc2626; font-weight: bold; }\n.unsupp { color: #64748b; font-weight: bold; }\n</style>\n</head>\n<body>\n");
        sb.append(String.format("<div class=\"card\">\n<h1>KMIP 2.1 Validation Matrix: %s</h1>\n", summary.serverId()));
        sb.append(String.format("<p><strong>Language:</strong> %s</p>\n", summary.language()));
        sb.append(String.format("<p>✅ <strong>Passed:</strong> %d | ❌ <strong>Failed:</strong> %d | ➖ <strong>Unsupported:</strong> %d</p>\n</div>\n",
                summary.passed(), summary.failed(), summary.unsupported()));
        sb.append("<div class=\"card\">\n<table>\n<thead>\n<tr><th>Case ID</th><th>Operation</th><th>Status</th></tr>\n</thead>\n<tbody>\n");

        for (CaseResult r : summary.results()) {
            String cls = (r.status() == CaseStatus.PASSED) ? "pass" : (r.status() == CaseStatus.FAILED) ? "fail" : "unsupp";
            String st = (r.status() == CaseStatus.PASSED) ? "✅ Passed" : (r.status() == CaseStatus.FAILED) ? "❌ Failed" : "➖ Unsupported";
            sb.append(String.format("<tr><td><code>%s</code></td><td>%s</td><td class=\"%s\">%s</td></tr>\n",
                    r.caseId(), r.operation(), cls, st));
        }

        sb.append("</tbody>\n</table>\n</div>\n</body>\n</html>\n");
        Files.createDirectories(path.getParent() != null ? path.getParent() : Path.of("."));
        Files.writeString(path, sb.toString());
    }

    public static void renderJson(ValidationSummary summary, Path path) throws IOException {
        StringBuilder sb = new StringBuilder();
        sb.append("{\n  \"run_id\": \"run-java\",\n  \"language\": \"java\",\n");
        sb.append(String.format("  \"server_id\": \"%s\",\n", summary.serverId()));
        sb.append(String.format("  \"summary\": {\n    \"total_cases\": %d,\n    \"passed\": %d,\n    \"failed\": %d,\n    \"unsupported_by_server\": %d\n  },\n",
                summary.totalCases(), summary.passed(), summary.failed(), summary.unsupported()));
        sb.append("  \"results\": [\n");

        for (int i = 0; i < summary.results().size(); i++) {
            CaseResult r = summary.results().get(i);
            String st = (r.status() == CaseStatus.PASSED) ? "passed" : (r.status() == CaseStatus.FAILED) ? "failed" : "unsupported_skipped";
            sb.append(String.format("    {\n      \"case_id\": \"%s\",\n      \"operation\": \"%s\",\n      \"status\": \"%s\"\n    }%s\n",
                    r.caseId(), r.operation(), st, (i + 1 < summary.results().size()) ? "," : ""));
        }
        sb.append("  ]\n}\n");

        Files.createDirectories(path.getParent() != null ? path.getParent() : Path.of("."));
        Files.writeString(path, sb.toString());
    }
}
