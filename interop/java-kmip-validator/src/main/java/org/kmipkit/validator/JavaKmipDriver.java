package org.kmipkit.validator;

import org.kmipkit.credentials.Authentication;
import org.kmipkit.credentials.Credential;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.WireEncoding;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.time.Duration;
import java.util.List;

/** Executes the Java public typed client against a KMIP server without logging remote payloads. */
public final class JavaKmipDriver implements AutoCloseable {
    private static final CodecLimits CODEC_LIMITS = CodecLimits.defaults();
    private final Client client;
    private final ConnectionConfig connection;

    private JavaKmipDriver(Client client, ConnectionConfig connection) {
        this.client = client;
        this.connection = connection;
    }

    /** Creates one TLS client for a run; client construction itself performs no exchange. */
    public static JavaKmipDriver connect(ConnectionConfig connection) {
        ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                List.of(), ExtensionRegistryLimits.defaults());
        ClientConfiguration baseConfiguration = ClientConfiguration.create(registry);
        ClientConfiguration encodingConfiguration = baseConfiguration.withWireEncoding(
                WireEncoding.valueOf(connection.encoding()));
        ClientConfiguration protocolConfiguration = encodingConfiguration.withProtocolVersion(
                toProtocolVersion(connection.protocolVersion()));
        TransportConfiguration transport = buildTransport(connection);
        try (registry; baseConfiguration; encodingConfiguration; protocolConfiguration; transport) {
            return new JavaKmipDriver(Client.create(protocolConfiguration, transport), connection);
        }
    }

    /** Executes one available typed operation and records only numeric KMIP result codes. */
    public CaseOutcome execute(CatalogLoader.CatalogCase testCase) {
        long start = System.nanoTime();
        ClientBatchItem request = createRequest(testCase.operation());
        if (request == null) {
            return CaseOutcome.notRun("No Java typed request route is implemented for this catalog operation.",
                    elapsedMillis(start));
        }
        long requestOperation = request.operation();
        try (request; ClientBatch batch = ClientBatch.create()) {
            batch.add(request);
            if (connection.usernamePassword() != null) {
                batch.setAuthentication(usernamePasswordAuthentication(connection.usernamePassword()));
            }
            try (ClientBatchResponse response = client.execute(batch, CODEC_LIMITS)) {
                if (response.itemCount() != 1) {
                    return CaseOutcome.failed("Response batch item count did not match the request.", elapsedMillis(start));
                }
                try (ClientBatchItemResult result = response.itemAt(0)) {
                    if (result.operation() != requestOperation) {
                        return CaseOutcome.failed("Response operation did not match the request.", elapsedMillis(start));
                    }
                    long statusCode = result.resultStatus();
                    long reasonCode = result.resultReason().orElse(-1);
                    if (statusCode == 0) {
                        if (!verifyTypedResponse(testCase.operation(), result)) {
                            return CaseOutcome.failed("The typed response shape did not match the request.",
                                    elapsedMillis(start), statusCode, optionalCode(reasonCode));
                        }
                        return CaseOutcome.passed(elapsedMillis(start), statusCode, optionalCode(reasonCode));
                    }
                    if (reasonCode == KmipValues.ResultReason.OPERATION_NOT_SUPPORTED
                            || reasonCode == KmipValues.ResultReason.FEATURE_NOT_SUPPORTED) {
                        return CaseOutcome.unsupported(elapsedMillis(start), statusCode, reasonCode);
                    }
                    if (reasonCode == KmipValues.ResultReason.PERMISSION_DENIED
                            || reasonCode == KmipValues.ResultReason.AUTHENTICATION_NOT_SUCCESSFUL) {
                        return CaseOutcome.policyRestricted(elapsedMillis(start), statusCode, reasonCode);
                    }
                    if (expectedReason(testCase, reasonCode)) {
                        return CaseOutcome.passed(elapsedMillis(start), statusCode, reasonCode);
                    }
                    return CaseOutcome.failed("The KMIP server returned a non-success result.",
                            elapsedMillis(start), statusCode, optionalCode(reasonCode));
                }
            }
        } catch (Exception exception) {
            return CaseOutcome.failed("KMIP client exchange failed (" + exception.getClass().getSimpleName()
                    + "); remote diagnostic text was suppressed.", elapsedMillis(start));
        }
    }

    private boolean verifyTypedResponse(String operation, ClientBatchItemResult result) {
        return switch (operation) {
            case "Ping" -> result.pingResponse().orElse(false);
            case "Query" -> {
                var payload = result.queryResponse();
                if (payload.isEmpty()) yield false;
                payload.orElseThrow().close();
                yield true;
            }
            case "Get Constraints" -> result.getConstraintsResponse().isPresent();
            case "Get Attribute List" -> result.getAttributeListResponse().isPresent();
            case "Discover Versions" -> true;
            default -> false;
        };
    }

    private static ClientBatchItem createRequest(String operation) {
        ClientBatchItem request = switch (operation) {
            case "Discover Versions" -> ClientBatchItem.discoverVersions();
            case "Ping" -> ClientBatchItem.ping();
            case "Query" -> ClientBatchItem.query(null, null);
            case "Get Constraints" -> ClientBatchItem.getConstraints();
            case "Get Attribute List" -> ClientBatchItem.getAttributeList();
            default -> null;
        };
        return request == null ? null : request.withUniqueBatchItemId(
                "item-0".getBytes(StandardCharsets.US_ASCII));
    }

    private static TransportConfiguration buildTransport(ConnectionConfig connection) {
        var tls = connection.tls();
        try (TransportConfiguration.Builder builder = TransportConfiguration.builder()) {
            if ("raw_tls".equals(connection.transport())) {
                builder.rawTls(connection.host(), connection.port());
            } else {
                builder.https(connection.httpsUri().toString());
            }
            builder.clientIdentityFiles(List.of(Path.of(tls.clientCertificatePath())),
                    Path.of(tls.clientKeyPath()), TransportConfiguration.Encoding.PEM);
            builder.trustCertificateFiles(List.of(Path.of(tls.caCertificatePath())),
                    TransportConfiguration.Encoding.PEM);
            String serverName = tls.serverNameOverride();
            if (serverName.isBlank()) {
                serverName = "raw_tls".equals(connection.transport())
                        ? connection.host() : connection.httpsUri().getHost();
            }
            builder.tlsServerName(serverName);
            Duration connect = Duration.ofSeconds(tls.connectTimeoutSeconds());
            Duration read = Duration.ofSeconds(tls.readTimeoutSeconds());
            builder.timeouts(TransportConfiguration.Timeout.bounded(connect),
                    TransportConfiguration.Timeout.bounded(read),
                    TransportConfiguration.Timeout.bounded(read),
                    TransportConfiguration.Timeout.bounded(read));
            return builder.build();
        }
    }

    private static Authentication usernamePasswordAuthentication(ConnectionConfig.UsernamePassword names) {
        String username = System.getenv(names.usernameEnvironmentName());
        String password = System.getenv(names.passwordEnvironmentName());
        if (username == null || username.isBlank() || password == null || password.isBlank()) {
            throw new IllegalArgumentException("Configured credential environment variables are unavailable");
        }
        // KMIPKit owns and zeroizes its native copies. The JVM's environment Strings cannot be erased.
        TtlvStructure valueStructure = TtlvStructure.create();
        valueStructure = appendSecret(valueStructure, 0x0042_0099, username);
        valueStructure = appendSecret(valueStructure, 0x0042_00a1, password);
        TtlvStructure credentialStructure = TtlvStructure.create();
        credentialStructure = appendValue(credentialStructure, 0x0042_0024,
                TtlvValue.Enumeration(1));
        credentialStructure = appendValue(credentialStructure, 0x0042_0025,
                TtlvValue.structure(valueStructure, CODEC_LIMITS));
        Credential credential = Credential.fromTtlv(credentialStructure);
        return Authentication.create(List.of(credential));
    }

    private static TtlvStructure appendSecret(TtlvStructure structure, int rawTag, String secret) {
        try (SecretText text = SecretText.fromString(secret)) {
            return appendValue(structure, rawTag, text.intoTtlvValue());
        }
    }

    private static TtlvStructure appendValue(TtlvStructure structure, int rawTag, TtlvValue value) {
        try (value; RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            TtlvItem item = TtlvItem.create(tag, value, CODEC_LIMITS);
            return TtlvStructure.withItem(structure, item, CODEC_LIMITS);
        }
    }

    private static boolean expectedReason(CatalogLoader.CatalogCase testCase, long reason) {
        var reasons = testCase.requires().get("expected_result_reasons");
        if (reasons == null || !reasons.isArray()) return false;
        for (var expected : reasons) {
            if (expected.canConvertToLong() && expected.longValue() == reason) return true;
        }
        return false;
    }

    private static ProtocolVersion toProtocolVersion(String version) {
        String[] parts = version.split("\\.");
        return new ProtocolVersion(Integer.parseInt(parts[0]), Integer.parseInt(parts[1]));
    }

    private static Long optionalCode(long code) {
        return code < 0 ? null : code;
    }

    private static double elapsedMillis(long start) {
        return Duration.ofNanos(System.nanoTime() - start).toNanos() / 1_000_000.0;
    }

    @Override
    public void close() {
        client.close();
    }

    public record CaseOutcome(String status, String libraryCorrectness, String serverCompatibility,
                              String parity, double durationMs, Long resultStatusCode,
                              Long resultReasonCode, String diagnostic) {
        static CaseOutcome passed(double elapsed, Long status, Long reason) {
            return new CaseOutcome("passed", "passed", "supported", "not_compared", elapsed,
                    status, reason, null);
        }

        static CaseOutcome localModelPassed(double elapsed) {
            return new CaseOutcome("passed", "passed", "unknown", "not_compared", elapsed,
                    null, null, null);
        }

        static CaseOutcome failed(String diagnostic, double elapsed) {
            return failed(diagnostic, elapsed, null, null);
        }

        static CaseOutcome failed(String diagnostic, double elapsed, Long status, Long reason) {
            return new CaseOutcome("failed", "not_checked", "unknown", "not_compared", elapsed,
                    status, reason, diagnostic);
        }

        static CaseOutcome unsupported(double elapsed, long status, long reason) {
            return new CaseOutcome("server_unsupported", "passed", "unsupported", "not_compared",
                    elapsed, status, reason, "The server rejected the operation as unsupported.");
        }

        static CaseOutcome policyRestricted(double elapsed, long status, long reason) {
            return new CaseOutcome("policy_restricted", "passed", "policy_restricted", "not_compared",
                    elapsed, status, reason, "The server policy rejected the operation.");
        }

        static CaseOutcome notRun(String diagnostic, double elapsed) {
            return new CaseOutcome("not_run", "not_checked", "unknown", "not_compared",
                    elapsed, null, null, diagnostic);
        }
    }
}
