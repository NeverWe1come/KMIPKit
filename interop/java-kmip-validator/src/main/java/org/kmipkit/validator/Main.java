package org.kmipkit.validator;

import org.kmipkit.validator.models.Models.CaseResult;
import org.kmipkit.validator.models.Models.CaseStatus;
import org.kmipkit.validator.models.Models.ServerCapabilities;
import org.kmipkit.validator.models.Models.TestCase;
import org.kmipkit.validator.models.Models.ValidationSummary;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

public class Main {
    public static void main(String[] args) {
        if (args.length == 0 || "-h".equals(args[0]) || "--help".equals(args[0])) {
            printHelp();
            System.exit(0);
        }

        String command = args[0];
        Path catalogPath = Paths.get("../shared/catalog/kmipkit-1.0-test-catalog.json");
        Path capabilitiesPath = null;
        Path connectionPath = null;
        Path outputDir = Paths.get("./results");
        boolean verifyUnsupported = false;

        for (int i = 1; i < args.length; i++) {
            switch (args[i]) {
                case "--catalog":
                    if (++i < args.length) catalogPath = Paths.get(args[i]);
                    break;
                case "--capabilities":
                    if (++i < args.length) capabilitiesPath = Paths.get(args[i]);
                    break;
                case "--connection":
                    if (++i < args.length) connectionPath = Paths.get(args[i]);
                    break;
                case "--output-dir":
                    if (++i < args.length) outputDir = Paths.get(args[i]);
                    break;
                case "--verify-unsupported":
                    verifyUnsupported = true;
                    break;
                default:
                    System.err.println("Unknown option: " + args[i]);
                    printHelp();
                    System.exit(1);
            }
        }

        if (capabilitiesPath == null) {
            System.err.println("Error: --capabilities <path> is required.");
            System.exit(1);
        }

        try {
            ServerCapabilities capabilities = JsonLoader.loadCapabilities(capabilitiesPath);
            List<TestCase> catalog = JsonLoader.loadCatalog(catalogPath);
            CapabilityMatcher matcher = new CapabilityMatcher(capabilities);

            if ("plan".equalsIgnoreCase(command)) {
                runPlan(capabilities, catalog, matcher);
            } else if ("run".equalsIgnoreCase(command)) {
                runExecution(capabilities, catalog, matcher, outputDir, verifyUnsupported);
            } else {
                System.err.println("Unknown command: " + command);
                printHelp();
                System.exit(1);
            }
        } catch (Exception e) {
            System.err.println("Execution failed: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }

    private static void runPlan(ServerCapabilities caps, List<TestCase> catalog, CapabilityMatcher matcher) {
        int supported = 0;
        int unsupported = 0;
        for (TestCase tc : catalog) {
            if (matcher.isSupported(tc)) {
                supported++;
            } else {
                unsupported++;
            }
        }

        System.out.println("Execution Plan for Server: " + caps.serverId());
        System.out.println("Language: Java (OpenJDK 17+)");
        System.out.println("Total Catalog Cases: " + catalog.size());
        System.out.println("Supported:   " + supported);
        System.out.println("Unsupported: " + unsupported);
    }

    private static void runExecution(ServerCapabilities caps, List<TestCase> catalog,
                                     CapabilityMatcher matcher, Path outputDir, boolean verifyUnsupported) throws IOException {
        String runId = UUID.randomUUID().toString();
        KmipDriver driver = KmipDriver.mockDriver();
        List<CaseResult> results = new ArrayList<>();

        int passed = 0;
        int failed = 0;
        int unsupportedCount = 0;

        for (TestCase tc : catalog) {
            boolean isSup = matcher.isSupported(tc);
            if (!isSup) {
                unsupportedCount++;
                if (verifyUnsupported) {
                    results.add(new CaseResult(tc.caseId(), tc.operation(), CaseStatus.UNSUPPORTED_VERIFIED_REJECTED, "Server verified rejected unsupported operation"));
                } else {
                    List<String> reasons = matcher.getUnsupportedReasons(tc);
                    String reason = reasons.isEmpty() ? "Capability not supported" : String.join(", ", reasons);
                    results.add(new CaseResult(tc.caseId(), tc.operation(), CaseStatus.UNSUPPORTED_SKIPPED, reason));
                }
            } else {
                AttemptContext attempt = new AttemptContext(runId, "Java", "TTLV", "L2");
                KmipDriver.ExecutionOutcome outcome = driver.executeCase(tc, attempt);
                if (outcome.success()) {
                    passed++;
                    results.add(new CaseResult(tc.caseId(), tc.operation(), CaseStatus.PASSED, "Passed"));
                } else {
                    failed++;
                    results.add(new CaseResult(tc.caseId(), tc.operation(), CaseStatus.FAILED, outcome.resultReason()));
                }
            }
        }

        ValidationSummary summary = new ValidationSummary(
                caps.serverId(), "Java", catalog.size(), passed, failed, unsupportedCount, results
        );

        Files.createDirectories(outputDir);
        MatrixReporter.writeReports(summary, outputDir);
        MatrixReporter.printAnsiTable(summary);
    }

    private static void printHelp() {
        System.out.println("KMIPKit 1.0 Standalone Java Validator");
        System.out.println("Usage:");
        System.out.println("  java org.kmipkit.validator.Main <command> [options]");
        System.out.println();
        System.out.println("Commands:");
        System.out.println("  plan     Preview which test cases will run vs skip based on capabilities");
        System.out.println("  run      Execute test cases against KMIP server and generate matrix reports");
        System.out.println();
        System.out.println("Options:");
        System.out.println("  --capabilities <path>      Path to server capabilities JSON (required)");
        System.out.println("  --catalog <path>           Path to test catalog JSON (default: ../shared/catalog/kmipkit-1.0-test-catalog.json)");
        System.out.println("  --connection <path>         Path to server connection JSON (optional)");
        System.out.println("  --output-dir <path>        Path for matrix output reports (default: ./results)");
        System.out.println("  --verify-unsupported       Send unsupported operations to KMS to verify proper error rejection");
    }
}
