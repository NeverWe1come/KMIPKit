package org.kmipkit.validator;

import com.fasterxml.jackson.core.StreamReadFeature;
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
            if (args.length == 1 && Set.of("--inspect-oasis-ttlv", "--construct-oasis-ttlv").contains(args[0])) {
                return DraftCampaign.inspectTtlv(Path.of(DEFAULT_MANIFEST), "--construct-oasis-ttlv".equals(args[0]));
            }
            if (args.length == 1 && "--inspect-table-key-material".equals(args[0])) {
                return DraftCampaign.inspectKeyMaterial(Path.of(DEFAULT_MANIFEST));
            }
            if (args.length == 1 && "--inspect-table-version".equals(args[0])) {
                return DraftCampaign.inspectVersion(Path.of(DEFAULT_MANIFEST));
            }
            if ("--plan".equals(args[0])) {
                return DraftCampaign.run(args, Path.of(DEFAULT_MANIFEST));
            }
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
        JavaRouteInventory.classify(catalog);
        List<CatalogLoader.CatalogCase> applicable = applicableCases(catalog, connection.protocolVersion());
        long local = applicable.stream().filter(testCase -> "local_model".equals(testCase.execution().mode())).count();
        long typed = applicable.stream().filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .filter(testCase -> supportsRoute(testCase, connection)).count();
        long recipePending = countCases(applicable, JavaRouteInventory.Status.TYPED_API_PENDING);
        long payloadOnly = countCases(applicable, JavaRouteInventory.Status.PAYLOAD_ONLY);
        long apiUnavailable = countCases(applicable, JavaRouteInventory.Status.NO_PUBLIC_ROUTE);
        long workflowUnavailable = applicable.stream().filter(GenericIncoming::isIncoming)
                .filter(testCase -> !GenericIncoming.supports(testCase, connection)).count();
        long mutation = applicable.stream().filter(testCase -> "mutating".equals(testCase.execution().mutation())).count();
        out.println("KMIPKit Java interoperability plan");
        out.println("Server ID: " + connection.serverId());
        out.println("Protocol: KMIP " + connection.protocolVersion());
        out.println("Applicable cases: " + applicable.size());
        out.println("Local TTLV model cases: " + local);
        out.println("Server exchanges executable through selected Java API level: " + typed);
        out.println("Public typed constructors awaiting catalog recipes: " + recipePending);
        out.println("Payload-only and missing public routes: " + (payloadOnly + apiUnavailable));
        out.println("Server-initiated workflows without a recipe for the selected API: " + workflowUnavailable);
        out.println("Cases requiring explicit mutation opt-in: " + mutation);
        out.println("Capability profiles are hypotheses and do not filter cases.");
        return 0;
    }

    private static int preflight(Arguments options, PrintStream out) throws Exception {
        ConnectionConfig connection = ConnectionConfig.load(options.path("--connection"));
        CatalogLoader.Catalog catalog = CatalogLoader.load(options.pathOr("--catalog", DEFAULT_CATALOG));
        JavaRouteInventory.classify(catalog);
        ArtifactProvenance artifact = ArtifactProvenance.loadForCheckout(options.pathOr("--artifact-manifest", DEFAULT_MANIFEST));
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
        JavaRouteInventory.classify(catalog);
        List<CatalogLoader.CatalogCase> applicable = applicableCases(catalog, connection.protocolVersion());
        String caseId = options.values().get("--case");
        if (caseId != null) {
            applicable = applicable.stream().filter(testCase -> caseId.equals(testCase.caseId())).toList();
            if (applicable.size() != 1) {
                out.println("No unique case for the selected protocol version.");
                return 2;
            }
        }
        ArtifactProvenance artifact = ArtifactProvenance.loadForCheckout(options.pathOr("--artifact-manifest", DEFAULT_MANIFEST));
        connection.requireTlsFilesExist();
        boolean enableMutations = options.flag("--enable-mutations");
        List<RunReport.CaseEvidence> evidence = new ArrayList<>(applicable.size());

        boolean hasPotentialLiveRoute = applicable.stream().anyMatch(testCase ->
                "server_exchange".equals(testCase.execution().mode())
                        && !GenericIncoming.isIncoming(testCase)
                        && supportsRoute(testCase, connection)
                        && (!"mutating".equals(testCase.execution().mutation())
                                || (connection.allowMutations() && enableMutations)));
        JavaKmipDriver driver = hasPotentialLiveRoute && !"generic-ttlv".equals(connection.apiLevel()) ? JavaKmipDriver.connect(connection) : null;
        boolean nativeIncoming = TypedIncoming.supportsConnection(connection) && options.values().containsKey("--listen");
        try (var incoming = nativeIncoming ? null : GenericIncoming.open(connection, options.values());
                var typedIncoming = nativeIncoming ? TypedIncoming.open(connection, options.values()) : null) {
            for (CatalogLoader.CatalogCase testCase : applicable) {
                JavaKmipDriver.CaseOutcome outcome = executeCase(
                        testCase, connection, enableMutations, driver, incoming, typedIncoming);
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
            ConnectionConfig connection, boolean enableMutations, JavaKmipDriver driver, GenericIncoming incoming,
            TypedIncoming typedIncoming) {
        long start = System.nanoTime();
        if ("not_applicable".equals(testCase.execution().status())) {
            return JavaKmipDriver.CaseOutcome.notRun("Catalog case is not applicable: "
                    + testCase.execution().notApplicableReason(), elapsedMillis(start));
        }
        if ("local_model".equals(testCase.execution().mode())) {
            if (!TtlvModelChecks.supports(testCase)) {
                return JavaKmipDriver.CaseOutcome.notRun(
                        "No Java local model recipe implements the selected assertions.", elapsedMillis(start));
            }
            TtlvModelChecks.ModelCheck model = TtlvModelChecks.verifyLocalCase(testCase);
            return model.passed()
                    ? JavaKmipDriver.CaseOutcome.localModelPassed(elapsedMillis(start))
                    : JavaKmipDriver.CaseOutcome.failed("Local TTLV model assertion failed.", elapsedMillis(start));
        }
        if (!supportsRoute(testCase, connection)) {
            if (!"typed-protocol".equals(connection.apiLevel())) {
                return JavaKmipDriver.CaseOutcome.notRun("The selected Java API level has no equivalent recipe.",
                        elapsedMillis(start));
            }
            return JavaKmipDriver.CaseOutcome.notRun(JavaRouteInventory.routeFor(testCase).reason(),
                    elapsedMillis(start));
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
        if (TypedIncoming.supports(testCase, connection)) {
            return typedIncoming == null
                    ? JavaKmipDriver.CaseOutcome.notRun("Incoming exchange requires --listen and a peer trigger.", elapsedMillis(start))
                    : typedIncoming.exchange(testCase);
        }
        if (GenericIncoming.supports(testCase, connection)) {
            return incoming == null
                    ? JavaKmipDriver.CaseOutcome.notRun("Incoming exchange requires --listen and a peer trigger.", elapsedMillis(start))
                    : incoming.exchange(testCase);
        }
        if ("generic-ttlv".equals(connection.apiLevel())) {
            return GenericExchange.run(testCase, connection);
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

    private static List<CatalogLoader.CatalogCase> applicableCases(
            CatalogLoader.Catalog catalog, String version) {
        return catalog.cases().stream().filter(testCase ->
                testCase.execution().protocolVersions().contains(version)).toList();
    }

    private static boolean supportsRoute(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        String apiLevel = connection.apiLevel();
        if ("generic-ttlv".equals(apiLevel)) {
            return GenericIncoming.supports(testCase, connection) || GenericExchange.supports(testCase, connection);
        }
        if (TypedIncoming.supports(testCase, connection)) return true;
        return hasTypedRoute(testCase) && ("typed-protocol".equals(apiLevel)
                || "fluent".equals(apiLevel) && JavaKmipDriver.supportsFluent(testCase));
    }

    private static boolean hasTypedRoute(CatalogLoader.CatalogCase testCase) {
        return JavaRouteInventory.routeFor(testCase).status() == JavaRouteInventory.Status.EXECUTABLE_TYPED;
    }

    private static long countCases(List<CatalogLoader.CatalogCase> cases, JavaRouteInventory.Status status) {
        return cases.stream().filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .filter(testCase -> JavaRouteInventory.routeFor(testCase).status() == status).count();
    }

    private static double elapsedMillis(long start) {
        return (System.nanoTime() - start) / 1_000_000.0;
    }

    private static void help(PrintStream output) {
        output.println("KMIPKit Java interoperability validator");
        output.println("Usage: <plan|preflight|run> --connection FILE [--catalog FILE]");
        output.println("Run also accepts --case ID --artifact-manifest FILE --output-dir DIR --enable-mutations.");
        output.println("Incoming cases accept --listen ADDRESS:PORT --listener-cert FILE --listener-key FILE --listener-ready-file FILE.");
        output.println("plan and preflight do not resolve DNS or open sockets.");
    }

    private record Arguments(Map<String, String> values, Set<String> flags) {
        static Arguments parse(String[] args) {
            Map<String, String> values = new HashMap<>();
            Set<String> flags = new HashSet<>();
            Set<String> knownFlags = Set.of("--enable-mutations");
            for (int index = 1; index < args.length; index++) {
                String option = args[index];
                if (knownFlags.contains(option)) {
                    flags.add(option);
                } else if (Set.of("--connection", "--catalog", "--artifact-manifest", "--output-dir")
                        .contains(option) && index + 1 < args.length) {
                    values.put(option, args[++index]);
                } else if ("run".equals(args[0]) && Set.of("--case", "--listen", "--listener-cert",
                        "--listener-key", "--listener-ready-file").contains(option)
                        && !values.containsKey(option) && index + 1 < args.length) {
                    values.put(option, args[++index]);
                } else {
                    throw new IllegalArgumentException("Unknown or incomplete command option");
                }
            }
            return new Arguments(Map.copyOf(values), Set.copyOf(flags));
        }

        Path path(String key) {
            String value = values.get(key);
            if (value == null || value.isBlank()) throw new IllegalArgumentException(key + " is required");
            return Path.of(value);
        }

        Path pathOr(String key, String fallback) {
            return Path.of(values.getOrDefault(key, fallback));
        }

        boolean flag(String key) {
            return flags.contains(key);
        }
    }
}
