package org.kmipkit.validator;

import com.fasterxml.jackson.core.StreamReadFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.PrintStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

/** Entry point for the independent Java interoperability consumer. */
public final class Main {
    private static final String PIN_FILE = "library-pin.json";
    private static final String DEFAULT_CATALOG = "../shared/catalog/kmipkit-1.0-test-catalog.json";
    private static final String DEFAULT_MANIFEST = "build/artifact-manifest.json";
    private static final ObjectMapper JSON = new ObjectMapper(
            com.fasterxml.jackson.core.JsonFactory.builder()
                    .enable(StreamReadFeature.STRICT_DUPLICATE_DETECTION)
                    .build());

    private Main() { }

    public static void main(String[] args) {
        int result = execute(args, System.out, System.err);
        if (result != 0) System.exit(result);
    }

    /** Executes one CLI command, returning a process-style status without exposing sensitive diagnostics. */
    public static int execute(String[] args, PrintStream out, PrintStream err) {
        if (args == null || args.length == 0 || "--help".equals(args[0]) || "-h".equals(args[0])) {
            help(out);
            return 0;
        }
        try {
            Arguments options = Arguments.parse(args);
            return switch (args[0]) {
                case "plan" -> plan(options, out);
                case "preflight" -> preflight(options, out);
                case "run" -> run(options, out);
                default -> throw new IllegalArgumentException("Command must be plan, preflight, or run");
            };
        } catch (Exception exception) {
            err.println("Command failed. Check the local configuration, artifact pin, and sanitized run report.");
            return 2;
        }
    }

    private static int plan(Arguments options, PrintStream out) throws Exception {
        ConnectionConfig connection = ConnectionConfig.load(options.path("--connection"));
        CatalogLoader.Catalog catalog = CatalogLoader.load(options.pathOr("--catalog", DEFAULT_CATALOG));
        Map<String, JavaRouteInventory.Route> routes = JavaRouteInventory.classify(catalog);
        List<CatalogLoader.CatalogCase> applicable = applicableCases(catalog, connection.protocolVersion());
        long local = applicable.stream().filter(testCase -> "local_model".equals(testCase.execution().mode())).count();
        long typed = "typed-protocol".equals(connection.apiLevel())
                ? countCases(applicable, routes, JavaRouteInventory.Status.EXECUTABLE_TYPED) : 0;
        long recipePending = countCases(applicable, routes, JavaRouteInventory.Status.TYPED_API_PENDING);
        long payloadOnly = countCases(applicable, routes, JavaRouteInventory.Status.PAYLOAD_ONLY);
        long apiUnavailable = countCases(applicable, routes, JavaRouteInventory.Status.NO_PUBLIC_ROUTE);
        long mutation = applicable.stream().filter(testCase -> "mutating".equals(testCase.execution().mutation())).count();
        out.println("KMIPKit Java interoperability plan");
        out.println("Server ID: " + connection.serverId());
        out.println("Protocol: KMIP " + connection.protocolVersion());
        out.println("Applicable cases: " + applicable.size());
        out.println("Local TTLV model cases: " + local);
        out.println("Server exchanges executable through selected Java API level: " + typed);
        out.println("Public typed constructors awaiting catalog recipes: " + recipePending);
        out.println("Payload-only and missing public routes: " + (payloadOnly + apiUnavailable));
        out.println("Cases requiring explicit mutation opt-in: " + mutation);
        out.println("Capability profiles are hypotheses and do not filter cases.");
        return 0;
    }

    private static int preflight(Arguments options, PrintStream out) throws Exception {
        ConnectionConfig connection = ConnectionConfig.load(options.path("--connection"));
        CatalogLoader.Catalog catalog = CatalogLoader.load(options.pathOr("--catalog", DEFAULT_CATALOG));
        JavaRouteInventory.classify(catalog);
        ArtifactProvenance artifact = loadArtifact(options.pathOr("--artifact-manifest", DEFAULT_MANIFEST));
        connection.requireTlsFilesExist();
        long applicable = applicableCases(catalog, connection.protocolVersion()).size();
        out.println("Preflight passed without DNS lookup or network sockets.");
        out.println("Server ID: " + connection.serverId());
        out.println("Protocol: KMIP " + connection.protocolVersion());
        out.println("Applicable catalog cases: " + applicable);
        out.println("Library commit: " + artifact.libraryCommit());
        out.println("Installed artifact SHA-256: " + artifact.artifactSha256());
        return 0;
    }

    private static int run(Arguments options, PrintStream out) throws Exception {
        ConnectionConfig connection = ConnectionConfig.load(options.path("--connection"));
        CatalogLoader.Catalog catalog = CatalogLoader.load(options.pathOr("--catalog", DEFAULT_CATALOG));
        Map<String, JavaRouteInventory.Route> routes = JavaRouteInventory.classify(catalog);
        ArtifactProvenance artifact = loadArtifact(options.pathOr("--artifact-manifest", DEFAULT_MANIFEST));
        connection.requireTlsFilesExist();
        List<CatalogLoader.CatalogCase> applicable = applicableCases(catalog, connection.protocolVersion());
        boolean enableMutations = options.flag("--enable-mutations");
        List<RunReport.CaseEvidence> evidence = new ArrayList<>(applicable.size());

        boolean hasPotentialLiveRoute = applicable.stream().anyMatch(testCase ->
                "server_exchange".equals(testCase.execution().mode())
                        && routes.get(testCase.operation()).status() == JavaRouteInventory.Status.EXECUTABLE_TYPED
                        && "typed-protocol".equals(connection.apiLevel())
                        && !"2.0".equals(connection.protocolVersion()));
        JavaKmipDriver driver = hasPotentialLiveRoute ? JavaKmipDriver.connect(connection) : null;
        try {
            for (CatalogLoader.CatalogCase testCase : applicable) {
                JavaKmipDriver.CaseOutcome outcome = executeCase(testCase, connection, enableMutations, driver);
                evidence.add(new RunReport.CaseEvidence(testCase.caseId(), outcome.status(),
                        outcome.libraryCorrectness(), outcome.serverCompatibility(), outcome.parity(),
                        outcome.durationMs(), outcome.resultStatusCode(), outcome.resultReasonCode(),
                        outcome.diagnostic()));
            }
        } finally {
            if (driver != null) driver.close();
        }

        String catalogDigest = ArtifactProvenance.sha256(options.pathOr("--catalog", DEFAULT_CATALOG));
        RunReport.Summary summary = summarize(evidence);
        RunReport report = new RunReport("2.0", "java-" + UUID.randomUUID(), Instant.now().toString(), "java",
                new RunReport.Server(connection.serverId(), connection.serverProduct(), connection.serverVersion(),
                        connection.configurationFingerprint()), connection.protocolVersion(), catalogDigest,
                new RunReport.Execution(connection.transport(), connection.encoding(), connection.apiLevel(),
                        artifact.libraryCommit(), artifact.artifactSha256()), summary, evidence);
        MatrixReporter.writeReports(report, options.pathOr("--output-dir", Path.of("results").toString()));
        out.println("KMIPKit correctness: " + summary.passed() + " passed, " + summary.failed() + " failed");
        out.println("Server compatibility observed: " + summary.unsupported() + " unsupported");
        out.println("Policy restrictions: " + summary.policyRestricted() + " cases withheld or rejected by policy");
        out.println("Coverage pending: " + summary.notRun() + " cases not run");
        out.println("Sanitized reports written to " + options.pathOr("--output-dir", Path.of("results").toString()));
        return summary.failed() == 0 ? 0 : 1;
    }

    private static JavaKmipDriver.CaseOutcome executeCase(CatalogLoader.CatalogCase testCase,
            ConnectionConfig connection, boolean enableMutations, JavaKmipDriver driver) {
        long start = System.nanoTime();
        if ("not_applicable".equals(testCase.execution().status())) {
            return JavaKmipDriver.CaseOutcome.notRun("Catalog case is not applicable: "
                    + testCase.execution().notApplicableReason(), elapsedMillis(start));
        }
        if ("local_model".equals(testCase.execution().mode())) {
            TtlvModelChecks.ModelCheck model = TtlvModelChecks.verifyLocalCase(testCase);
            return model.passed()
                    ? JavaKmipDriver.CaseOutcome.localModelPassed(elapsedMillis(start))
                    : JavaKmipDriver.CaseOutcome.failed("Local TTLV model assertion failed.", elapsedMillis(start));
        }
        if ("mutating".equals(testCase.execution().mutation())) {
            try {
                connection.requireLiveExecution(true, enableMutations);
            } catch (IllegalArgumentException rejected) {
                return new JavaKmipDriver.CaseOutcome("policy_restricted", "not_checked", "unknown",
                        "not_compared", elapsedMillis(start), null, null,
                        "State-changing operation was withheld locally; configuration and --enable-mutations are required.");
            }
        }
        if (!"typed-protocol".equals(connection.apiLevel())) {
            return JavaKmipDriver.CaseOutcome.notRun("The selected Java API level has no server request route.",
                    elapsedMillis(start));
        }
        if ("2.0".equals(connection.protocolVersion())) {
            return JavaKmipDriver.CaseOutcome.notRun(
                    "The pinned release artifact does not yet preserve Java KMIP 2.0 client configuration.",
                    elapsedMillis(start));
        }
        if (!hasTypedRoute(testCase.operation())) {
            return JavaKmipDriver.CaseOutcome.notRun(JavaRouteInventory.routeFor(testCase.operation()).reason(),
                    elapsedMillis(start));
        }
        if (driver == null) {
            return JavaKmipDriver.CaseOutcome.notRun("No Java typed client was initialized for this run.",
                    elapsedMillis(start));
        }
        return driver.execute(testCase);
    }

    private static RunReport.Summary summarize(List<RunReport.CaseEvidence> evidence) {
        int passed = 0;
        int failed = 0;
        int unsupported = 0;
        int policyRestricted = 0;
        int notRun = 0;
        for (RunReport.CaseEvidence result : evidence) {
            switch (result.status()) {
                case "passed" -> passed++;
                case "failed" -> failed++;
                case "server_unsupported" -> unsupported++;
                case "policy_restricted" -> policyRestricted++;
                case "not_run" -> notRun++;
                default -> throw new IllegalStateException("A run case has an unclassified status");
            }
        }
        return new RunReport.Summary(evidence.size(), passed, failed, unsupported, policyRestricted, notRun);
    }

    private static ArtifactProvenance loadArtifact(Path manifestPath) throws Exception {
        Path pinPath = Path.of(PIN_FILE);
        JsonNode pin = JSON.readTree(pinPath.toFile());
        if (pin == null || !"release/1.0.0".equals(pin.path("release_ref").asText())) {
            throw new IllegalArgumentException("The Java package pin must refer to release/1.0.0");
        }
        return ArtifactProvenance.load(manifestPath, pin.path("commit").asText());
    }

    private static List<CatalogLoader.CatalogCase> applicableCases(
            CatalogLoader.Catalog catalog, String version) {
        return catalog.cases().stream().filter(testCase ->
                testCase.execution().protocolVersions().contains(version)).toList();
    }

    private static boolean hasTypedRoute(String operation) {
        return JavaRouteInventory.routeFor(operation).status() == JavaRouteInventory.Status.EXECUTABLE_TYPED;
    }

    private static long countCases(List<CatalogLoader.CatalogCase> cases,
            Map<String, JavaRouteInventory.Route> routes, JavaRouteInventory.Status status) {
        return cases.stream().filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .filter(testCase -> routes.get(testCase.operation()).status() == status).count();
    }

    private static double elapsedMillis(long start) {
        return (System.nanoTime() - start) / 1_000_000.0;
    }

    private static void help(PrintStream output) {
        output.println("KMIPKit Java interoperability validator");
        output.println("Usage: <plan|preflight|run> --connection FILE [--catalog FILE]");
        output.println("Run also accepts --artifact-manifest FILE --output-dir DIR --enable-mutations.");
        output.println("plan and preflight do not resolve DNS or open sockets.");
    }

    private record Arguments(Map<String, String> paths, Set<String> flags) {
        static Arguments parse(String[] args) {
            Map<String, String> paths = new HashMap<>();
            Set<String> flags = new HashSet<>();
            Set<String> knownFlags = Set.of("--enable-mutations");
            for (int index = 1; index < args.length; index++) {
                String option = args[index];
                if (knownFlags.contains(option)) {
                    flags.add(option);
                } else if (Set.of("--connection", "--catalog", "--artifact-manifest", "--output-dir")
                        .contains(option) && index + 1 < args.length) {
                    paths.put(option, args[++index]);
                } else {
                    throw new IllegalArgumentException("Unknown or incomplete command option");
                }
            }
            return new Arguments(Map.copyOf(paths), Set.copyOf(flags));
        }

        Path path(String key) {
            String value = paths.get(key);
            if (value == null || value.isBlank()) throw new IllegalArgumentException(key + " is required");
            return Path.of(value);
        }

        Path pathOr(String key, String fallback) {
            return Path.of(paths.getOrDefault(key, fallback));
        }

        boolean flag(String key) {
            return flags.contains(key);
        }
    }
}
