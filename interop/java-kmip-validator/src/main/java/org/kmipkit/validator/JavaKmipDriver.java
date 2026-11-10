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
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.WireEncoding;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

import java.nio.file.Path;
import java.time.Duration;
import java.util.List;
import java.util.UUID;

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
        if ("Create".equals(testCase.operation())) {
            return executeCreateWithCleanup(testCase, start);
        }
        ClientBatchItem request = TypedRequestFactory.create(testCase.operation());
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

    private CaseOutcome executeCreateWithCleanup(CatalogLoader.CatalogCase testCase, long start) {
        String objectName = testCase.isolationRecipe().uniqueNamePrefix() + "-" + UUID.randomUUID();
        UniqueIdentifier createdIdentifier = null;
        long createStatus = -1;
        long createReason = -1;
        long createObjectType = -1;
        try (TtlvStructure attributes = createTestKeyAttributes(objectName);
                ClientBatchItem request = ClientBatchItem.create(2, attributes, null)
                        .withUniqueBatchItemId("item-0".getBytes(java.nio.charset.StandardCharsets.US_ASCII));
                ClientBatch batch = ClientBatch.create()) {
            long requestOperation = request.operation();
            batch.add(request);
            if (connection.usernamePassword() != null) {
                batch.setAuthentication(usernamePasswordAuthentication(connection.usernamePassword()));
            }
            try (ClientBatchResponse response = client.execute(batch, CODEC_LIMITS)) {
                if (response.itemCount() != 1) {
                    return CaseOutcome.failed("Create response batch item count did not match the request.",
                            elapsedMillis(start));
                }
                try (ClientBatchItemResult result = response.itemAt(0)) {
                    if (result.operation() != requestOperation) {
                        return CaseOutcome.failed("Create response operation did not match the request.",
                                elapsedMillis(start));
                    }
                    createStatus = result.resultStatus();
                    createReason = result.resultReason().orElse(-1);
                    if (createStatus != 0) {
                        return nonSuccessOutcome(testCase, createStatus, createReason, start);
                    }
                    var payload = result.createResponse();
                    if (payload.isEmpty()) {
                        return CaseOutcome.failed("Create success response omitted its typed payload.",
                                elapsedMillis(start), createStatus, optionalCode(createReason));
                    }
                    var created = payload.orElseThrow();
                    createObjectType = created.objectType();
                    createdIdentifier = created.uniqueIdentifier();
                }
            }
        } catch (Exception exception) {
            return exchangeFailure(exception, start);
        }

        if (createdIdentifier == null) {
            return CaseOutcome.failed("Create success response omitted the object identifier; cleanup could not run.",
                    elapsedMillis(start), createStatus, optionalCode(createReason));
        }

        try (ClientBatchItem request = ClientBatchItem.destroy(createdIdentifier)
                        .withUniqueBatchItemId("item-0".getBytes(java.nio.charset.StandardCharsets.US_ASCII));
                ClientBatch batch = ClientBatch.create()) {
            long requestOperation = request.operation();
            batch.add(request);
            if (connection.usernamePassword() != null) {
                batch.setAuthentication(usernamePasswordAuthentication(connection.usernamePassword()));
            }
            try (ClientBatchResponse response = client.execute(batch, CODEC_LIMITS)) {
                if (response.itemCount() != 1) {
                    return CaseOutcome.failed("Create succeeded, but cleanup response count was invalid.",
                            elapsedMillis(start), createStatus, optionalCode(createReason));
                }
                try (ClientBatchItemResult result = response.itemAt(0)) {
                    if (result.operation() != requestOperation || result.resultStatus() != 0) {
                        return CaseOutcome.failed("Create succeeded, but cleanup Destroy did not succeed.",
                                elapsedMillis(start), optionalCode(result.resultStatus()),
                                optionalCode(result.resultReason().orElse(-1)));
                    }
                    var payload = result.destroyResponse();
                    if (payload.isEmpty()
                            || !sameIdentifier(createdIdentifier, payload.orElseThrow().uniqueIdentifier())) {
                        return CaseOutcome.failed("Create succeeded, but Destroy did not confirm the same object.",
                                elapsedMillis(start), createStatus, optionalCode(createReason));
                    }
                }
            }
        } catch (Exception exception) {
            return CaseOutcome.failed("Create succeeded, but cleanup exchange failed ("
                    + exception.getClass().getSimpleName() + "); remote diagnostic text was suppressed.",
                    elapsedMillis(start), createStatus, optionalCode(createReason));
        }

        if (createObjectType != 2) {
            return CaseOutcome.failed("Create returned an object type different from Symmetric Key.",
                    elapsedMillis(start), createStatus, optionalCode(createReason));
        }
        return CaseOutcome.passed(elapsedMillis(start), createStatus, optionalCode(createReason));
    }

    private static TtlvStructure createTestKeyAttributes(String objectName) {
        TtlvStructure name = TtlvStructure.create();
        TtlvStructure attributes = null;
        try {
            name = appendOwnedValue(name, 0x0042_0055,
                    TtlvValue.TextString(objectName.getBytes(java.nio.charset.StandardCharsets.UTF_8), CODEC_LIMITS));
            name = appendOwnedValue(name, 0x0042_0054, TtlvValue.Enumeration(1));

            attributes = TtlvStructure.create();
            attributes = appendOwnedValue(attributes, 0x0042_0053, TtlvValue.structure(name, CODEC_LIMITS));
            name.close();
            name = null;
            attributes = appendOwnedValue(attributes, 0x0042_0028, TtlvValue.Enumeration(3));
            attributes = appendOwnedValue(attributes, 0x0042_002a, TtlvValue.Integer(256));
            TtlvStructure result = attributes;
            attributes = null;
            return result;
        } finally {
            if (name != null) name.close();
            if (attributes != null) attributes.close();
        }
    }

    private static TtlvStructure appendOwnedValue(TtlvStructure previous, int rawTag, TtlvValue value) {
        try {
            return appendValue(previous, rawTag, value);
        } finally {
            previous.close();
        }
    }

    private static boolean sameIdentifier(UniqueIdentifier left, UniqueIdentifier right) {
        if (left.kind() != right.kind()) return false;
        return left.kind() == UniqueIdentifier.Kind.TEXT_STRING
                ? left.text().equals(right.text()) : left.number() == right.number();
    }

    private static CaseOutcome nonSuccessOutcome(
            CatalogLoader.CatalogCase testCase, long status, long reason, long start) {
        if (reason == KmipValues.ResultReason.OPERATION_NOT_SUPPORTED
                || reason == KmipValues.ResultReason.FEATURE_NOT_SUPPORTED) {
            return CaseOutcome.unsupported(elapsedMillis(start), status, reason);
        }
        if (reason == KmipValues.ResultReason.PERMISSION_DENIED
                || reason == KmipValues.ResultReason.AUTHENTICATION_NOT_SUCCESSFUL) {
            return CaseOutcome.policyRestricted(elapsedMillis(start), status, reason);
        }
        if (expectedReason(testCase, reason)) {
            return CaseOutcome.passed(elapsedMillis(start), status, reason);
        }
        return CaseOutcome.failed("The KMIP server returned a non-success Create result.",
                elapsedMillis(start), status, optionalCode(reason));
    }

    private static CaseOutcome exchangeFailure(Exception exception, long start) {
        return CaseOutcome.failed("KMIP client exchange failed (" + exception.getClass().getSimpleName()
                + "); remote diagnostic text was suppressed.", elapsedMillis(start));
    }

    private boolean verifyTypedResponse(String operation, ClientBatchItemResult result) {
        return switch (operation) {
            case "Ping" -> result.pingResponse().orElse(false);
            case "Query" -> {
                var payload = result.typedQueryResponse();
                if (payload.isEmpty()) yield false;
                try (var query = payload.orElseThrow()) {
                    yield JavaResponseChecks.queryAdvertisesOperations(query.operations());
                }
            }
            case "Log" -> {
                var payload = result.logResponse();
                if (payload.isEmpty()) yield false;
                payload.orElseThrow().close();
                yield true;
            }
            case "Create" -> false;
            case "Get Constraints" -> {
                var payload = result.getConstraintsResponse();
                if (payload.isEmpty()) yield false;
                payload.orElseThrow().close();
                yield true;
            }
            case "Get Attribute List" -> {
                var payload = result.getAttributeListResponse();
                if (payload.isEmpty()) yield false;
                payload.orElseThrow().close();
                yield true;
            }
            case "Discover Versions" -> JavaResponseChecks.advertisesRequestedVersion(
                    result.supportedVersions().orElse(List.of()), toProtocolVersion(connection.protocolVersion()));
            default -> false;
        };
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
