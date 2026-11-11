package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.GetAttributesRequest;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.QueryResponse;
import org.kmipkit.extensions.QueryResponseField;
import org.kmipkit.extensions.QueryResponseFieldKind;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.PrintStream;
import java.net.InetAddress;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.KeyFactory;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.SecureRandom;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.Base64;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ControlledTlsPeerTest {
    private static final String LOG_MESSAGE = "KMIPKit interoperability diagnostic";
    private static final String CREATED_OBJECT_ID = "controlled-create-object-id";
    private static final List<TagRangeFixture> TAG_RANGE_FIXTURES = List.of(
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-001", "KMIPKIT-RANGE-001", 0x000002, "unused"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-002", "KMIPKIT-RANGE-002", 0x420009, "reserved"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-003", "KMIPKIT-RANGE-003", 0x430000, "unused"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-004", "KMIPKIT-RANGE-004", 0x540001, "extension"),
            new TagRangeFixture("test-norm-tag-range-kmipkit-range-005", "KMIPKIT-RANGE-005", 0x550000, "unused"));
    private static final Set<String> GENERIC_ENUMERATION_PARENTS = Set.of(
            "KMIPKIT-ELEM-ENUMERATION-ADJUSTMENT-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-ALTERNATIVE-NAME-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-ASYNCHRONOUS-INDICATOR",
            "KMIPKIT-ELEM-ENUMERATION-ATTESTATION-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-BATCH-ERROR-CONTINUATION-OPTION",
            "KMIPKIT-ELEM-ENUMERATION-CANCELLATION-RESULT",
            "KMIPKIT-ELEM-ENUMERATION-CERTIFICATE-REQUEST-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-CLIENT-REGISTRATION-METHOD",
            "KMIPKIT-ELEM-ENUMERATION-DATA",
            "KMIPKIT-ELEM-ENUMERATION-DERIVATION-METHOD",
            "KMIPKIT-ELEM-ENUMERATION-DESTROY-ACTION",
            "KMIPKIT-ELEM-ENUMERATION-DRBG-ALGORITHM",
            "KMIPKIT-ELEM-ENUMERATION-ENCODING-OPTION",
            "KMIPKIT-ELEM-ENUMERATION-FIPS186-VARIATION",
            "KMIPKIT-ELEM-ENUMERATION-INTEROP-FUNCTION",
            "KMIPKIT-ELEM-ENUMERATION-KEY-COMPRESSION-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-KEY-VALUE-LOCATION-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-KEY-WRAP-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-LINK-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-NAME-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-OBJECT-GROUP-MEMBER",
            "KMIPKIT-ELEM-ENUMERATION-OPERATION",
            "KMIPKIT-ELEM-ENUMERATION-PROCESSING-STAGE",
            "KMIPKIT-ELEM-ENUMERATION-PROFILE-NAME",
            "KMIPKIT-ELEM-ENUMERATION-PUT-FUNCTION",
            "KMIPKIT-ELEM-ENUMERATION-QUERY-FUNCTION",
            "KMIPKIT-ELEM-ENUMERATION-RECOMMENDED-CURVE",
            "KMIPKIT-ELEM-ENUMERATION-RESULT-STATUS",
            "KMIPKIT-ELEM-ENUMERATION-REVOCATION-REASON-CODE",
            "KMIPKIT-ELEM-ENUMERATION-RNG-ALGORITHM",
            "KMIPKIT-ELEM-ENUMERATION-RNG-MODE",
            "KMIPKIT-ELEM-ENUMERATION-ROTATE-NAME-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-SECRET-DATA-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-SPLIT-KEY-METHOD",
            "KMIPKIT-ELEM-ENUMERATION-TICKET-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-UNWRAP-MODE",
            "KMIPKIT-ELEM-ENUMERATION-USAGE-LIMITS-UNIT",
            "KMIPKIT-ELEM-ENUMERATION-VALIDATION-AUTHORITY-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-VALIDATION-TYPE",
            "KMIPKIT-ELEM-ENUMERATION-VALIDITY-INDICATOR",
            "KMIPKIT-ELEM-ENUMERATION-WRAPPING-METHOD");

    @TempDir
    Path temporaryDirectory;

    // OASIS KMIP 2.1 §6.1.16 and §4.1.2: Discover Versions request and response.
    @Test
    void typedDiscoverVersionsUsesInstalledPackageAndVerifiedMutualTls() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, discoverVersionsResponse()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-kmip-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem));
        Path manifestPath = Path.of(System.getProperty("kmipkit.artifactManifest"));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-discover-versions"))
                .findFirst().orElseThrow();

        ArtifactProvenance.load(manifestPath, "9a16cc90a12205101d22a38501a5e74c665df4dc");
        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertEquals("passed", outcome.status(), outcome.toString());
        assertEquals("supported", outcome.serverCompatibility());
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 30}),
                "The controlled peer must receive the typed Discover Versions operation.");
    }

    @Test
    void typedQueryUsesAValidFunctionAndVerifiesTheTypedResponse() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, queryResponse()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-query-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("query-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-query"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 0x18}),
                "The controlled peer must receive the typed Query operation.");
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x74, 0x05, 0, 0, 0, 4, 0, 0, 0, 1}),
                "The Query request must include a non-empty Query Function.");
        assertEquals("passed", outcome.status(), outcome.toString());
    }

    @Test
    void controlledQueryProducesPerRouteEvidenceForBothProtocolVersions() throws Exception {
        Path reportPath = Path.of(System.getProperty("kmipkit.routeEvidenceOutput",
                "build/route-execution-evidence.json"));
        Files.deleteIfExists(reportPath);
        ObjectMapper mapper = new ObjectMapper();
        ArrayNode events = mapper.createArrayNode();
        byte[] queryOperation = queryOperationTtlv();
        String requestTreeHash = normalizedTreeSha256(queryOperation);

        for (String protocolVersion : List.of("2.0", "2.1")) {
            appendLocalQueryRouteEvents(events, protocolVersion, requestTreeHash);
            appendTagRangeRouteEvents(events, protocolVersion);
            appendActivationDateLocalRouteEvents(events, protocolVersion);

            byte[] serverCertificatePem = fixture("server-cert.pem");
            byte[] serverPrivateKeyPem = fixture("server-key.pem");
            byte[] clientCertificatePem = fixture("client-cert.pem");
            byte[] clientPrivateKeyPem = fixture("client-key.pem");
            SSLServerSocket listener = listener(certificate(serverCertificatePem),
                    privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
            byte[] responseBytes = queryResponse("2.0".equals(protocolVersion) ? 0 : 1);
            FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, responseBytes));
            Thread peerThread = new Thread(peer, "java-validator-query-route-evidence-peer-" + protocolVersion);
            peerThread.setDaemon(true);
            peerThread.start();

            Path connectionFile = temporaryDirectory.resolve("route-evidence-" + protocolVersion + ".json");
            Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                    clientCertificatePem, clientPrivateKeyPem, false, protocolVersion));
            try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
                try (ClientBatch batch = ClientBatch.create()) {
                    ClientBatchItem request = ClientBatchItem.query(new int[] {1}, null)
                            .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII));
                    assertEquals(0x18L, request.operation());
                    batch.add(request);
                    try (ClientBatchResponse response = driver.executeBatchForEvidence(batch)) {
                        assertEquals(1, response.itemCount());
                        try (ClientBatchItemResult result = response.itemAt(0)) {
                            assertEquals(0x18L, result.operation());
                            assertEquals(0L, result.resultStatus());
                            try (var genericPayload = result.queryResponse().orElseThrow()) {
                                try (var view = genericPayload.view()) {
                                    assertEquals(2, view.itemCount());
                                }
                            }
                            try (QueryResponse typed = result.typedQueryResponse().orElseThrow()) {
                                assertEquals(2, typed.fieldCount());
                                QueryResponseField operation = typed.fieldAt(0);
                                assertEquals(QueryResponseFieldKind.ENUMERATION, operation.kind());
                                assertEquals(0x3bL, operation.enumeration().orElseThrow());
                            }
                        }
                    }
                }
            } finally {
                listener.close();
            }
            byte[] request = peer.get(15, TimeUnit.SECONDS);
            assertEquals(0x18L, requestOperation(request),
                    "The controlled peer must parse the installed Query request operation.");
            assertTrue(contains(request, item(0x420074, 5, intBytes(1))),
                    "The controlled peer must observe the Query Operations function.");
            assertEquals("2.0".equals(protocolVersion) ? 0 : 1, requestProtocolMinorVersion(request));
            String requestHash = normalizedTreeSha256(request);
            String responseHash = normalizedTreeSha256(responseBytes);
            appendNetworkQueryRouteEvents(events, protocolVersion, requestHash, responseHash);
            appendControlledActivationDateNetworkEvents(events, protocolVersion);
        }

        writeRouteExecutionReport(reportPath, mapper, events);
        assertTrue(Files.isRegularFile(reportPath),
                "Installed Query calls against the controlled peer must emit route execution evidence.");
        JsonNode report = new ObjectMapper().readTree(reportPath.toFile());
        assertEquals("java", report.path("language").asText());
        List<EnumerationFixture> enumerationFixtures = genericEnumerationFixtures();
        assertTrue(enumerationFixtures.stream().noneMatch(fixture -> fixture.caseId().equals(
                        "test-norm-enumeration-value-profile-name-reserved-00000001-00000103")),
                "A finite Profile Name range must remain pending until each named value is represented.");
        int enumerationEventCount = enumerationFixtures.stream()
                .mapToInt(fixture -> fixture.protocolVersions().size() * 4).sum();
        assertEquals(68 + enumerationEventCount, report.path("events").size());
        assertEquals(7 + enumerationFixtures.size(), report.path("case_ids").size());
        assertEquals("test-op-query", report.path("case_ids").get(0).asText());
        for (int index = 1; index < 6; index++) {
            assertEquals("test-norm-tag-range-kmipkit-range-%03d".formatted(index),
                    report.path("case_ids").get(index).asText());
        }
        assertEquals("test-attr-activation-date", report.path("case_ids").get(6).asText());
        assertEquals(2, report.path("protocol_versions").size());
        assertEquals("org.kmipkit.extensions.Client.execute",
                findEvent(report.path("events"), "send", "2.0").path("execution_api_symbol").asText());
        assertEquals("org.kmipkit.extensions.ClientBatchItemResult.queryResponse",
                findRouteEvent(report.path("events"),
                        "java.typed-protocol.function.client_batch_item_response_query", "2.1")
                        .path("public_api_symbol").asText());
    }

    @Test
    void typedClientPreservesKmip20OnTheWire() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem),
                privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, queryResponse(0)));
        Thread peerThread = new Thread(peer, "java-validator-kmip-20-controlled-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip-20-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, "2.0"));
        var connection = ConnectionConfig.load(connectionFile);
        assertEquals("2.0", connection.protocolVersion());
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var query = catalog.cases().stream().filter(value -> value.caseId().equals("test-op-query"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(query);
        } finally {
            listener.close();
        }
        assertEquals("passed", outcome.status(), outcome.toString());
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertTrue(contains(request, structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(0)))),
                "The installed Java client must preserve KMIP 2.0 in the request header: "
                        + java.util.HexFormat.of().formatHex(request));
    }

    @Test
    void asynchronousOperationsUseTheirInstalledDedicatedExecutorsAndTypedResponses() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var operationCases = casesById(catalog, "test-op-cancel", "test-op-poll", "test-op-process",
                "test-op-query-asynchronous-requests");
        var peer = new FutureTask<List<byte[]>>(() -> serveAsynchronousResponses(listener, operationCases.size()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-async-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("async-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        byte[] correlation = {0x4b, 0x4d, 0x49, 0x50, 0x2d, 0x74, 0x65, 0x73, 0x74};
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var testCase : operationCases) {
                var outcome = driver.executeAsynchronousRequest(testCase, correlation);
                assertEquals("passed", outcome.status(), testCase.operation() + ": " + outcome);
            }
        } finally {
            listener.close();
        }

        var requests = peer.get(20, TimeUnit.SECONDS);
        assertEquals(List.of(0x19L, 0x1aL, 0x3aL, 0x39L),
                requests.stream().map(ControlledTlsPeerTest::requestOperation).toList());
        for (byte[] request : requests) {
            assertTrue(contains(request, item(0x420006, 8, correlation)),
                    "The public asynchronous route must send the caller-supplied correlation value.");
        }
    }

    @Test
    void asynchronousCancelAndPollPreserveKmip20WithTheInstalledExecutors() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var operationCases = casesById(catalog, "test-op-cancel", "test-op-poll");
        var peer = new FutureTask<List<byte[]>>(() -> serveAsynchronousResponses(listener, operationCases.size(), 0));
        Thread peerThread = new Thread(peer, "java-validator-controlled-kmip20-async-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip20-async-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true, "2.0"));
        byte[] correlation = {0x20, 0x20, 0x20, 0x30};
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var testCase : operationCases) {
                var outcome = driver.executeAsynchronousRequest(testCase, correlation);
                assertEquals("passed", outcome.status(), testCase.operation() + ": " + outcome);
            }
        } finally {
            listener.close();
        }

        var requests = peer.get(20, TimeUnit.SECONDS);
        assertEquals(List.of(0x19L, 0x1aL),
                requests.stream().map(ControlledTlsPeerTest::requestOperation).toList());
        assertTrue(requests.stream().allMatch(request -> requestProtocolMinorVersion(request) == 0),
                "KMIP 2.0 asynchronous requests must retain the configured version on the wire.");
    }

    @Test
    void processAndQueryAsyncRejectKmip20BeforeOpeningTheTransport() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        listener.setSoTimeout(3_000);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var operationCases = casesById(catalog, "test-v20-reject-process", "test-v20-reject-query-asynchronous-requests");
        Path connectionFile = temporaryDirectory.resolve("kmip20-async-boundary-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, "2.0"));
        byte[] correlation = {0x20, 0x20, 0x20, 0x31};
        FutureTask<Boolean> peer = new FutureTask<>(() -> acceptsConnection(listener));
        Thread peerThread = new Thread(peer, "java-validator-kmip20-async-boundary-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        boolean accepted;
        try {
            try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
                for (var testCase : operationCases) {
                    var outcome = driver.executeAsynchronousRequest(testCase, correlation);
                    assertEquals("failed", outcome.status(), testCase.operation() + ": " + outcome);
                    assertTrue(outcome.diagnostic().contains("InvalidInputException"),
                            testCase.operation() + ": " + outcome);
                }
            }
            accepted = peer.get(5, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }
        assertEquals(false, accepted,
                "KMIP 2.0 must reject Process and Query Asynchronous Requests before transport invocation.");
    }

    @Test
    void typedLogSendsTheRequiredMessageAndVerifiesAnEmptyResponsePayload() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, logResponse()));
        Thread peerThread = new Thread(peer, "java-validator-controlled-log-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("log-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-log"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 0x2c}),
                "The controlled peer must receive the typed Log operation.");
        assertTrue(contains(request, item(0x420141, 7, LOG_MESSAGE.getBytes(StandardCharsets.UTF_8))),
                "The typed Log request must contain the deterministic Log Message.");
        assertEquals("passed", outcome.status(), outcome.toString());
    }

    @Test
    void typedCreateUsesAnIsolatedNameAndDestroysTheCreatedObject() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveCreateCleanup(listener));
        Thread peerThread = new Thread(peer, "java-validator-controlled-create-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("create-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals("test-op-create"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        assertEquals("passed", outcome.status(), outcome.toString());
        List<byte[]> requests = peer.get(15, TimeUnit.SECONDS);
        assertEquals(2, requests.size());
        assertTrue(contains(requests.get(0), new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4, 0, 0, 0, 1}),
                "The controlled peer must receive Create before cleanup.");
        assertTrue(contains(requests.get(0), new byte[] {0x42, 0x00, 0x53, 1}),
                "The Create request must include an isolated Name attribute.");
        assertTrue(contains(requests.get(0), "kmipkit-create-".getBytes(StandardCharsets.UTF_8)),
                "The Create Name must use the catalog's unique prefix.");
        assertTrue(contains(requests.get(0), item(0x420054, 5, intBytes(1))),
                "The Create Name must include the Name Type enumeration.");
        assertTrue(contains(requests.get(0), item(0x420028, 5, intBytes(3))),
                "The isolated object must request AES.");
        assertTrue(contains(requests.get(0), item(0x42002a, 2, intBytes(256))),
                "The isolated object must request a 256-bit key.");
        assertTrue(contains(requests.get(1), new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4, 0, 0, 0, 0x14}),
                "The created object must be destroyed in a separate exchange.");
        assertTrue(contains(requests.get(1), item(0x420094, 7,
                CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))));
    }

    @Test
    void everyPublicClientOperationReachesPeerAndClassifiesUnsupportedResponse() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var availableOperations = TypedRequestFactory.availableFactories().keySet();
        var casesByOperation = new java.util.TreeMap<String, CatalogLoader.CatalogCase>();
        catalog.cases().stream()
                .filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .filter(testCase -> testCase.execution().protocolVersions().contains("2.1"))
                .filter(testCase -> !"server_to_client".equals(testCase.requires().path("workflow_type").asText()))
                .filter(testCase -> availableOperations.contains(testCase.operation()))
                .forEach(testCase -> casesByOperation.putIfAbsent(testCase.operation(), testCase));
        catalog.cases().stream().filter(testCase -> testCase.caseId().equals("test-op-get-attributes"))
                .findFirst().ifPresent(testCase -> casesByOperation.put("Get Attributes", testCase));
        assertEquals(availableOperations.size(), casesByOperation.size(),
                "Every public KMIP 2.x client operation must have a catalog exchange case.");

        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        MatrixPlan matrixPlan = operationMatrixPlan(casesByOperation, 1);
        var peerTrace = new java.util.concurrent.CopyOnWriteArrayList<Long>();
        var peerRequests = new java.util.concurrent.CopyOnWriteArrayList<byte[]>();
        FutureTask<List<Long>> peer = new FutureTask<>(() -> serveOperationMatrixResponses(listener,
                matrixPlan, peerTrace, peerRequests));
        Thread peerThread = new Thread(peer, "java-validator-controlled-operation-matrix-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("operation-matrix-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var entry : casesByOperation.entrySet()) {
                JavaKmipDriver.CaseOutcome outcome = driver.execute(entry.getValue());
                assertPeerHealthy(peer);
                if ("Create".equals(entry.getKey())) {
                    assertEquals("passed", outcome.status(), entry.getKey() + ": " + outcome);
                    assertEquals(0L, outcome.resultStatusCode(), entry.getKey());
                } else {
                    assertEquals("server_unsupported", outcome.status(), entry.getKey() + ": " + outcome
                            + "; peer operations=" + peerTrace);
                    assertEquals("unsupported", outcome.serverCompatibility(), entry.getKey());
                    assertEquals(1L, outcome.resultStatusCode(), entry.getKey());
                    assertEquals(KmipValues.ResultReason.OPERATION_NOT_SUPPORTED,
                            outcome.resultReasonCode(), entry.getKey());
                }
            }
        } finally {
            listener.close();
        }
        List<Long> observedOperations = peer.get(60, TimeUnit.SECONDS);
        assertEquals(matrixPlan.expectedOperations().size(), observedOperations.size());
        byte[] unfilteredGetAttributes = peerRequests.stream()
                .filter(request -> requestOperation(request) == 0x0b)
                .findFirst().orElseThrow();
        assertTrue(!containsTag(unfilteredGetAttributes, 0x42013b),
                "The operation-level Get Attributes case has no Attribute Reference and must request all attributes.");
        for (var entry : casesByOperation.entrySet()) {
            try (ClientBatchItem expected = TypedRequestFactory.create(entry.getKey())) {
                assertTrue(observedOperations.contains(expected.operation()), entry.getKey());
                assertEquals("Operation Not Supported", KmipValues.ResultReason.name(
                        KmipValues.ResultReason.OPERATION_NOT_SUPPORTED));
            }
        }
    }

    // OASIS KMIP 2.0 §§6.1 and 11: every catalogued client operation applicable
    // to 2.0 must be emitted through the installed typed request factories.
    @Test
    void everyPublicClientOperationReachesPeerForKmip20() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var availableOperations = TypedRequestFactory.availableFactories().keySet();
        var casesByOperation = new java.util.TreeMap<String, CatalogLoader.CatalogCase>();
        catalog.cases().stream()
                .filter(testCase -> "server_exchange".equals(testCase.execution().mode()))
                .filter(testCase -> testCase.execution().protocolVersions().contains("2.0"))
                .filter(testCase -> !"server_to_client".equals(testCase.requires().path("workflow_type").asText()))
                .filter(testCase -> availableOperations.contains(testCase.operation()))
                .forEach(testCase -> casesByOperation.putIfAbsent(testCase.operation(), testCase));
        catalog.cases().stream().filter(testCase -> testCase.caseId().equals("test-op-get-attributes"))
                .findFirst().ifPresent(testCase -> casesByOperation.put("Get Attributes", testCase));
        assertTrue(!casesByOperation.isEmpty(), "KMIP 2.0 must have applicable public client operations.");

        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        MatrixPlan matrixPlan = operationMatrixPlan(casesByOperation, 0);
        var peerTrace = new java.util.concurrent.CopyOnWriteArrayList<Long>();
        var peerRequests = new java.util.concurrent.CopyOnWriteArrayList<byte[]>();
        FutureTask<List<Long>> peer = new FutureTask<>(() -> serveOperationMatrixResponses(listener,
                matrixPlan, peerTrace, peerRequests));
        Thread peerThread = new Thread(peer, "java-validator-kmip20-operation-matrix-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip20-operation-matrix-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true, "2.0"));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            for (var entry : casesByOperation.entrySet()) {
                JavaKmipDriver.CaseOutcome outcome = driver.execute(entry.getValue());
                assertPeerHealthy(peer);
                if ("Create".equals(entry.getKey())) {
                    assertEquals("passed", outcome.status(), entry.getKey() + ": " + outcome);
                    assertEquals(0L, outcome.resultStatusCode(), entry.getKey());
                } else {
                    assertEquals("server_unsupported", outcome.status(), entry.getKey() + ": " + outcome
                            + "; peer operations=" + peerTrace);
                    assertEquals("unsupported", outcome.serverCompatibility(), entry.getKey());
                    assertEquals(1L, outcome.resultStatusCode(), entry.getKey());
                    assertEquals(KmipValues.ResultReason.OPERATION_NOT_SUPPORTED,
                            outcome.resultReasonCode(), entry.getKey());
                }
            }
        } finally {
            listener.close();
        }
        List<Long> observedOperations = peer.get(60, TimeUnit.SECONDS);
        assertEquals(matrixPlan.expectedOperations().size(), observedOperations.size());
        byte[] unfilteredGetAttributes = peerRequests.stream()
                .filter(request -> requestOperation(request) == 0x0b)
                .findFirst().orElseThrow();
        assertTrue(!containsTag(unfilteredGetAttributes, 0x42013b),
                "The operation-level Get Attributes case has no Attribute Reference and must request all attributes.");
        for (String operation : casesByOperation.keySet()) {
            try (ClientBatchItem expected = TypedRequestFactory.create(operation)) {
                assertTrue(observedOperations.contains(expected.operation()), operation);
            }
        }
    }

    // OASIS KMIP 2.0 §6.1.32: the consumer CLI must preserve the configured
    // version through request creation, TLS exchange, and response decoding.
    @Test
    void cliRunExecutesQueryAtKmip20AndAcceptsItsTypedResponse() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener, queryResponse(0)));
        Thread peerThread = new Thread(peer, "java-validator-kmip20-cli-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("kmip20-cli-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, "2.0"));
        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        ArrayNode selectedCases = mapper.createArrayNode();
        for (JsonNode testCase : catalog.path("test_cases")) {
            if ("test-op-query".equals(testCase.path("case_id").asText())) {
                selectedCases.add(testCase);
            }
        }
        assertEquals(1, selectedCases.size(), "The KMIP 2.0 Query case must exist in the shared catalog.");
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("kmip20-query-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        Path outputDirectory = temporaryDirectory.resolve("kmip20-run-report");
        var standardOutput = new ByteArrayOutputStream();
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString()},
                    new PrintStream(standardOutput), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        assertEquals(0, exitCode, standardError.toString(StandardCharsets.UTF_8));
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertEquals(0, requestProtocolMinorVersion(request));
        assertEquals(0x18L, requestOperation(request));
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals(1, report.path("summary").path("total").asInt());
        assertEquals("passed", report.path("cases").get(0).path("status").asText());
        assertEquals(0, report.path("cases").get(0).path("result_status_code").asInt());
    }

    @Test
    void cliRunDispatchesGetThroughItsPublicTypedFactory() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveObjectFixtureAndGet(listener));
        Thread peerThread = new Thread(peer, "java-validator-cli-get-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("get-cli-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));
        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        ArrayNode selectedCases = mapper.createArrayNode();
        for (JsonNode testCase : catalog.path("test_cases")) {
            if ("test-op-get".equals(testCase.path("case_id").asText())) selectedCases.add(testCase);
        }
        assertEquals(1, selectedCases.size(), "The Get operation case must exist in the shared catalog.");
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("get-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        Path outputDirectory = temporaryDirectory.resolve("get-run-report");
        var standardOutput = new ByteArrayOutputStream();
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString(), "--enable-mutations"},
                    new PrintStream(standardOutput), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        List<byte[]> requests = peer.get(10, TimeUnit.SECONDS);
        assertEquals(List.of(1L, 10L, 0x14L), requests.stream().map(ControlledTlsPeerTest::requestOperation).toList());
        assertTrue(contains(requests.get(1), item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))),
                "Get must use the identifier returned by the fixture Create response.");
        assertEquals(0, exitCode, standardError.toString(StandardCharsets.UTF_8));
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals("passed", report.path("cases").get(0).path("status").asText());
        assertEquals(0L, report.path("cases").get(0).path("result_status_code").asLong());
    }

    @Test
    void cliRunReadsAttributeListAndValuesThroughTypedFactories() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        Path connectionFile = temporaryDirectory.resolve("attribute-read-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));

        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        Set<String> selectedIds = Set.of(
                "test-attr-cryptographic-algorithm", "test-attr-name");
        ArrayNode selectedCases = mapper.createArrayNode();
        var targetOperations = new ArrayList<Long>();
        var targetAttributeTags = new ArrayList<Long>();
        var loadedCases = CatalogLoader.load(sourceCatalog).cases().stream().collect(
                java.util.stream.Collectors.toMap(CatalogLoader.CatalogCase::caseId, testCase -> testCase));
        for (JsonNode testCase : catalog.path("test_cases")) {
            if (!selectedIds.contains(testCase.path("case_id").asText())) continue;
            selectedCases.add(testCase);
            long attributeTag = loadedCases.get(testCase.path("case_id").asText()).attributeTag();
            targetAttributeTags.add(attributeTag);
            try (ClientBatchItem request = TypedRequestFactory.create(
                    testCase.path("operation").asText(), null, attributeTag)) {
                targetOperations.add(request.operation());
            }
        }
        assertEquals(selectedIds.size(), selectedCases.size(), "Both attribute-specific read cases must exist.");
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("attribute-read-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        FutureTask<List<byte[]>> peer = new FutureTask<>(
                () -> serveAttributeReadCases(listener, targetOperations, targetAttributeTags));
        Thread peerThread = new Thread(peer, "java-validator-attribute-read-peer");
        peerThread.setDaemon(true);
        peerThread.start();
        Path outputDirectory = temporaryDirectory.resolve("attribute-read-run-report");
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString(), "--enable-mutations"},
                    new PrintStream(new ByteArrayOutputStream()), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        List<byte[]> requests = peer.get(10, TimeUnit.SECONDS);
        assertEquals(6, requests.size());
        for (int index = 0; index < targetOperations.size(); index++) {
            byte[] request = requests.get(index * 3 + 1);
            assertEquals(targetOperations.get(index), requestOperation(request));
            assertTrue(contains(request, item(0x420094, 7,
                    CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))),
                    "The request must use the identifier from its setup Create response.");
            assertTrue(contains(request, item(0x42013b, 5,
                            intBytes(Math.toIntExact(targetAttributeTags.get(index))))),
                    "Get Attributes must encode the Attribute Reference tag selected by its catalog case.");
        }
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals(2, report.path("summary").path("passed").asInt(), report.toString());
        assertEquals(0, report.path("summary").path("failed").asInt(), report.toString());
        assertEquals(0, exitCode, standardError.toString(StandardCharsets.UTF_8));
    }

    @Test
    void cliRunRejectsGetAttributesSuccessWithIncorrectAttributeValue() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        Path connectionFile = temporaryDirectory.resolve("attribute-mismatch-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true));

        Path sourceCatalog = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        ObjectMapper mapper = new ObjectMapper();
        ObjectNode catalog = (ObjectNode) mapper.readTree(sourceCatalog.toFile());
        JsonNode selectedCase = null;
        for (JsonNode testCase : catalog.path("test_cases")) {
            if ("test-attr-cryptographic-algorithm".equals(testCase.path("case_id").asText())) {
                selectedCase = testCase;
                break;
            }
        }
        assertTrue(selectedCase != null, "The Cryptographic Algorithm attribute case must exist.");
        ArrayNode selectedCases = mapper.createArrayNode().add(selectedCase);
        catalog.set("test_cases", selectedCases);
        ((ObjectNode) catalog.path("statistics")).put("total_cases", selectedCases.size());
        Path selectedCatalog = temporaryDirectory.resolve("attribute-mismatch-catalog.json");
        Files.writeString(selectedCatalog, mapper.writeValueAsString(catalog));

        long requestedAttributeTag = 0x420028L;
        long returnedAttributeTag = 0x420028L;
        long operation;
        try (ClientBatchItem request = TypedRequestFactory.create("Get Attributes", null, requestedAttributeTag)) {
            operation = request.operation();
        }
        FutureTask<List<byte[]>> peer = new FutureTask<>(
                () -> serveAttributeReadCases(listener, List.of(operation), List.of(requestedAttributeTag),
                        List.of(returnedAttributeTag), List.of(4L)));
        Thread peerThread = new Thread(peer, "java-validator-attribute-mismatch-peer");
        peerThread.setDaemon(true);
        peerThread.start();
        Path outputDirectory = temporaryDirectory.resolve("attribute-mismatch-run-report");
        var standardError = new ByteArrayOutputStream();
        int exitCode;
        try {
            exitCode = Main.execute(new String[] {"run", "--connection", connectionFile.toString(), "--catalog",
                    selectedCatalog.toString(), "--output-dir", outputDirectory.toString(), "--enable-mutations"},
                    new PrintStream(new ByteArrayOutputStream()), new PrintStream(standardError));
        } finally {
            listener.close();
        }
        List<byte[]> requests = peer.get(10, TimeUnit.SECONDS);
        assertEquals(3, requests.size());
        assertTrue(contains(requests.get(1), item(0x42013b, 5, intBytes(Math.toIntExact(requestedAttributeTag)))));
        JsonNode report = mapper.readTree(outputDirectory.resolve("results.json").toFile());
        assertEquals(1, report.path("summary").path("failed").asInt(), report.toString());
        assertEquals(0, report.path("summary").path("passed").asInt(), report.toString());
        assertEquals(1, exitCode, standardError.toString(StandardCharsets.UTF_8));
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey) throws Exception {
        return connectionJson(port, serverCertificate, clientCertificate, clientPrivateKey, false);
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey, boolean allowMutations) throws Exception {
        return connectionJson(port, serverCertificate, clientCertificate, clientPrivateKey,
                allowMutations, "2.1");
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey, boolean allowMutations,
                                  String protocolVersion) throws Exception {
        Path serverCert = temporaryDirectory.resolve("server-cert.pem");
        Path clientCert = temporaryDirectory.resolve("client-cert.pem");
        Path clientKey = temporaryDirectory.resolve("client-key.pem");
        Files.write(serverCert, serverCertificate);
        Files.write(clientCert, clientCertificate);
        Files.write(clientKey, clientPrivateKey);
        ObjectNode root = new ObjectMapper().createObjectNode();
        root.put("server_id", "controlled-test-peer");
        root.put("server_configuration_fingerprint", "a".repeat(64));
        root.put("protocol_version", protocolVersion);
        root.put("transport", "raw_tls");
        root.put("encoding", "TTLV");
        root.put("api_level", "typed-protocol");
        ObjectNode endpoints = root.putObject("endpoints");
        endpoints.putObject("raw_tls").put("host", "127.0.0.1").put("port", port);
        ObjectNode tls = root.putObject("tls");
        tls.put("ca_cert_path", serverCert.toString());
        tls.put("client_cert_path", clientCert.toString());
        tls.put("client_key_path", clientKey.toString());
        tls.put("server_name_override", "localhost");
        tls.put("connect_timeout_seconds", 5);
        tls.put("read_timeout_seconds", 5);
        root.putObject("environment").put("kind", "dedicated_test")
                .put("dedicated_test_environment_confirmed", true);
        root.putObject("execution_policy").put("allow_mutations", allowMutations);
        return new ObjectMapper().writeValueAsString(root);
    }

    private static SSLServerSocket listener(X509Certificate serverCertificate, PrivateKey serverKey,
                                            X509Certificate clientCertificate) throws Exception {
        char[] password = "local-kmip-test".toCharArray();
        KeyStore serverKeys = KeyStore.getInstance("PKCS12");
        serverKeys.load(null, password);
        serverKeys.setKeyEntry("server", serverKey, password, new Certificate[] {serverCertificate});
        KeyManagerFactory keys = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        keys.init(serverKeys, password);
        KeyStore trust = KeyStore.getInstance("PKCS12");
        trust.load(null, password);
        trust.setCertificateEntry("client", clientCertificate);
        TrustManagerFactory trustManagers = TrustManagerFactory.getInstance(
                TrustManagerFactory.getDefaultAlgorithm());
        trustManagers.init(trust);
        SSLContext context = SSLContext.getInstance("TLSv1.3");
        context.init(keys.getKeyManagers(), trustManagers.getTrustManagers(), new SecureRandom());
        SSLServerSocket server = (SSLServerSocket) context.getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        server.setEnabledProtocols(new String[] {"TLSv1.3"});
        server.setNeedClientAuth(true);
        server.setSoTimeout(10_000);
        return server;
    }

    private static byte[] serveOneResponse(SSLServerSocket listener, byte[] response) throws Exception {
        try (SSLSocket socket = (SSLSocket) listener.accept()) {
            socket.setEnabledProtocols(new String[] {"TLSv1.3"});
            socket.setSoTimeout(10_000);
            socket.startHandshake();
            byte[] request = readTtlv(socket.getInputStream());
            OutputStream output = socket.getOutputStream();
            output.write(response);
            output.flush();
            return request;
        }
    }

    private static List<Long> serveUnsupportedResponses(SSLServerSocket listener, int count) throws Exception {
        return serveUnsupportedResponses(listener, count, 1);
    }

    private static List<Long> serveUnsupportedResponses(SSLServerSocket listener, int count,
                                                         int protocolMinorVersion) throws Exception {
        var operations = new ArrayList<Long>();
        for (int index = 0; index < count; index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                long operation = requestOperation(request);
                operations.add(operation);
                socket.getOutputStream().write(unsupportedResponse(operation, protocolMinorVersion));
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(operations);
    }

    private static MatrixPlan operationMatrixPlan(
            java.util.Map<String, CatalogLoader.CatalogCase> casesByOperation, int protocolMinorVersion)
            throws Exception {
        var expectedOperations = new ArrayList<Long>();
        var responses = new ArrayList<byte[]>();
        for (var entry : casesByOperation.entrySet()) {
            String operationName = entry.getKey();
            CatalogLoader.CatalogCase testCase = entry.getValue();
            boolean fixture = requiresSymmetricKeyFixture(testCase);
            if ("Create".equals(operationName)) {
                expectedOperations.add(1L);
                responses.add(createResponse(protocolMinorVersion));
                expectedOperations.add(0x14L);
                responses.add(destroyResponse(protocolMinorVersion));
                continue;
            }
            if (fixture) {
                expectedOperations.add(1L);
                responses.add(createResponse(protocolMinorVersion));
            }
            try (ClientBatchItem request = TypedRequestFactory.create(operationName)) {
                expectedOperations.add(request.operation());
            }
            responses.add(unsupportedResponse(expectedOperations.get(expectedOperations.size() - 1),
                    protocolMinorVersion));
            if (fixture) {
                expectedOperations.add(0x14L);
                responses.add(destroyResponse(protocolMinorVersion));
            }
        }

        return new MatrixPlan(List.copyOf(expectedOperations), List.copyOf(responses));
    }

    private static List<Long> serveOperationMatrixResponses(SSLServerSocket listener,
            MatrixPlan plan, List<Long> peerTrace, List<byte[]> peerRequests) throws Exception {
        var observedOperations = new ArrayList<Long>();
        for (int index = 0; index < plan.expectedOperations().size(); index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                peerTrace.add(-1L);
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                peerRequests.add(request);
                long operation = requestOperation(request);
                observedOperations.add(operation);
                peerTrace.add(operation);
                if (operation != plan.expectedOperations().get(index)) {
                    throw new AssertionError("The operation matrix sent an unexpected operation at exchange " + index);
                }
                socket.getOutputStream().write(plan.responses().get(index));
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(observedOperations);
    }

    private static boolean requiresSymmetricKeyFixture(CatalogLoader.CatalogCase testCase) {
        return testCase.isolationRecipe().setupSteps().stream()
                .anyMatch(step -> step.startsWith("Create unique SymmetricKey"))
                || ("Get Attributes".equals(testCase.operation())
                        && (testCase.attributeTag() == 0x420028L || testCase.attributeTag() == 0x420053L));
    }

    private static void assertPeerHealthy(FutureTask<?> peer) throws Exception {
        if (!peer.isDone()) return;
        try {
            peer.get();
        } catch (java.util.concurrent.ExecutionException exception) {
            throw new AssertionError("The controlled peer rejected the request sequence.", exception.getCause());
        }
    }

    private static long requestOperation(byte[] request) {
        byte[] header = new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4};
        for (int offset = 0; offset <= request.length - header.length - 4; offset++) {
            boolean matches = true;
            for (int index = 0; index < header.length; index++) {
                if (request[offset + index] != header[index]) {
                    matches = false;
                    break;
                }
            }
            if (matches) return Integer.toUnsignedLong(ByteBuffer.wrap(request, offset + header.length, 4).getInt());
        }
        throw new AssertionError("The outgoing KMIP request omitted its Operation field.");
    }

    private static void appendLocalQueryRouteEvents(ArrayNode events, String protocolVersion,
            String operationTreeHash) throws Exception {
        try (RawTag rawTag = RawTag.fromRaw(0x42005c);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.Enumeration(0x18);
                TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
            try (var view = item.view(); var valueView = view.value()) {
                assertEquals(0x42005c, view.tag().raw());
                assertEquals(TtlvItemType.Enumeration, view.itemType());
                assertEquals(0x18L, valueView.enumeration());
            }
            appendRouteEvent(events, "test-op-query", "java.generic-ttlv.ttlv_item.create",
                    "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.ttlv.TtlvItem.create", null, null,
                    "construct", protocolVersion, null, operationTreeHash, null);
            appendRouteEvent(events, "test-op-query", "java.generic-ttlv.function.ttlv_item_view_from_item",
                    "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.ttlv.TtlvItem.view", null, null,
                    "decode", protocolVersion, null, null, operationTreeHash);
        }

        try (TtlvItem decoded = TtlvCodec.decode(queryOperationTtlv())) {
            assertTrue(decoded != null, "The installed TTLV decoder must return a value.");
        }
        appendRouteEvent(events, "test-op-query", "java.generic-ttlv.ttlv_codec.decode",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, operationTreeHash);
    }

    private static void appendTagRangeRouteEvents(ArrayNode events, String protocolVersion) throws Exception {
        for (TagRangeFixture fixture : TAG_RANGE_FIXTURES) {
            byte[] wire = item(fixture.rawTag(), 5, intBytes(0));
            String treeHash = normalizedTreeSha256(wire);
            if ("extension".equals(fixture.allocation())) {
                appendAllocatedTagRangeEvents(events, fixture, protocolVersion, wire, treeHash);
            } else {
                appendRejectedTagRangeEvent(events, fixture, protocolVersion, wire);
            }
        }
    }

    private static List<EnumerationFixture> genericEnumerationFixtures() throws Exception {
        Path catalogPath = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        JsonNode catalog = new ObjectMapper().readTree(catalogPath.toFile());
        Map<String, JsonNode> targets = new HashMap<>();
        for (JsonNode target : catalog.path("normative_targets")) {
            targets.put(target.path("target_id").asText(), target);
        }

        var fixtures = new ArrayList<EnumerationFixture>();
        for (JsonNode testCase : catalog.path("test_cases")) {
            JsonNode requires = testCase.path("requires");
            if (!"enumeration_value".equals(requires.path("target_kind").asText())) continue;
            JsonNode target = targets.get(requires.path("normative_target_id").asText());
            if (target == null || !target.path("parent_element_ids").isArray()
                    || target.path("parent_element_ids").isEmpty()) continue;
            String parentId = target.path("parent_element_ids").get(0).asText();
            if (!GENERIC_ENUMERATION_PARENTS.contains(parentId)) continue;
            String parentName = parentId.replace("KMIPKIT-ELEM-ENUMERATION-", "");
            JsonNode tagTarget = targets.values().stream()
                    .filter(candidate -> "tag".equals(candidate.path("kind").asText()))
                    .filter(candidate -> normalizeTargetName(candidate.path("name").asText())
                            .equals(normalizeTargetName(parentName)))
                    .findFirst().orElse(null);
            if (tagTarget == null) continue;

            var versions = new ArrayList<String>();
            for (JsonNode version : testCase.path("execution").path("protocol_versions")) {
                versions.add(version.asText());
            }
            if (versions.isEmpty()) continue;
            String wireValue = target.path("wire_value").asText();
            long enumValue = "extension".equals(target.path("allocation").asText())
                    ? 0x8000_0000L : parseFirstEnumerationWireValue(wireValue);
            String tagWireValue = tagTarget.path("wire_value").asText().replaceFirst("^0[xX]", "");
            int rawTag = Math.toIntExact(Long.parseLong(tagWireValue, 16));
            fixtures.add(new EnumerationFixture(testCase.path("case_id").asText(),
                    target.path("target_id").asText(), rawTag, enumValue, List.copyOf(versions)));
        }
        return List.copyOf(fixtures);
    }

    private static long parseFirstEnumerationWireValue(String wireValue) {
        String firstValue = wireValue.split("-", 2)[0];
        return Long.parseUnsignedLong(firstValue, 16);
    }

    private static String normalizeTargetName(String value) {
        return value.replaceAll("[^A-Za-z0-9]", "").toUpperCase(Locale.ROOT);
    }

    private static void appendActivationDateLocalRouteEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        byte[] activationDateWire = item(0x420001, 9, longBytes(1));
        byte[] getAttributesOperationWire = item(0x42005c, 5, intBytes(0x0b));
        String activationDateHash = normalizedTreeSha256(activationDateWire);
        try (TtlvValue value = TtlvValue.DateTime(1)) {
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.generic-ttlv.model.org.kmipkit.ttlv.ttlvvalue.datetime",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.ttlv.TtlvValue.DateTime",
                    null, null, "construct", protocolVersion, null, activationDateHash, null);
            appendLocalTtlvRoundtripEvents(events, "test-attr-activation-date",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", 0x420001, TtlvItemType.DateTime,
                    value, activationDateWire, protocolVersion);
        }
        try (TtlvValue value = TtlvValue.Enumeration(0x0b)) {
            appendLocalTtlvRoundtripEvents(events, "test-attr-activation-date",
                    "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES", 0x42005c, TtlvItemType.Enumeration,
                    value, getAttributesOperationWire, protocolVersion);
        }

        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue value = TtlvValue.DateTime(1);
                TypedAttribute typed = TypedAttribute.create(StandardAttribute.ACTIVATION_DATE, value);
                TtlvItem item = typed.asItem();
                TypedAttribute decoded = TypedAttribute.fromItem(StandardAttribute.ACTIVATION_DATE, item)) {
            assertEquals(0x420001, StandardAttribute.ACTIVATION_DATE.tag());
            try (var view = decoded.view(); var decodedValue = view.value()) {
                assertEquals(1L, decodedValue.dateTime());
            }
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.create",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.ttlv.TypedAttribute.create",
                    null, null, "construct", protocolVersion, null, activationDateHash, null);
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.typed-protocol.model.org.kmipkit.ttlv.typedattribute.fromitem",
                    "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.ttlv.TypedAttribute.fromItem",
                    null, null, "construct", protocolVersion, null, activationDateHash, null);

            try (TtlvStructure empty = TtlvStructure.create();
                    TtlvStructure attributeItems = TtlvStructure.withItem(empty, item, limits);
                    AttributeSet attributes = AttributeSet.tryNew(attributeItems)) {
                assertEquals(1, attributes.len());
                appendRouteEvent(events, "test-attr-activation-date",
                        "java.typed-protocol.model.org.kmipkit.extensions.attributeset.trynew",
                        "KMIPKIT-ELEM-ATTRIBUTE-ACTIVATION-DATE", "org.kmipkit.extensions.AttributeSet.tryNew",
                        null, null, "construct", protocolVersion, null,
                        normalizedTreeSha256(structure(0x420125, activationDateWire)), null);
            }
        }

        UniqueIdentifier identifier = UniqueIdentifier.textString("object-7");
        byte[] requestPayloadWire = structure(0x42007c,
                item(0x420094, 7, "object-7".getBytes(StandardCharsets.UTF_8)),
                item(0x42013b, 5, intBytes(0x420001)));
        try (GetAttributesRequest request = GetAttributesRequest.create(identifier,
                        List.of(AttributeReference.tag(0x420001)));
                TtlvStructure payload = request.toTtlvPayload();
                var view = payload.view()) {
            assertEquals(2, view.itemCount());
            appendRouteEvent(events, "test-attr-activation-date",
                    "java.typed-protocol.function.attribute_request_get_attributes_to_ttlv_payload",
                    "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES",
                    "org.kmipkit.extensions.GetAttributesRequest.toTtlvPayload", null, null,
                    "encode", protocolVersion, null, normalizedTreeSha256(requestPayloadWire), null);
        }
    }

    private static void appendLocalTtlvRoundtripEvents(ArrayNode events, String caseId, String targetId,
            int rawTag, TtlvItemType expectedType, TtlvValue value, byte[] wire, String protocolVersion)
            throws Exception {
        String wireHash = normalizedTreeSha256(wire);
        try (RawTag raw = RawTag.fromRaw(rawTag);
                Tag tag = raw.checked();
                TtlvItem created = TtlvItem.create(tag, value, CodecLimits.defaults());
                var view = created.view();
                var viewValue = view.value()) {
            assertEquals(rawTag, view.tag().raw());
            assertEquals(expectedType, view.itemType());
            if (expectedType == TtlvItemType.DateTime) assertEquals(1L, viewValue.dateTime());
            else assertEquals(0x0bL, viewValue.enumeration());
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_item.create", targetId,
                "org.kmipkit.ttlv.TtlvItem.create", null, null, "construct", protocolVersion,
                null, wireHash, null);
        appendRouteEvent(events, caseId, "java.generic-ttlv.function.ttlv_item_view_from_item", targetId,
                "org.kmipkit.ttlv.TtlvItem.view", null, null, "decode", protocolVersion,
                null, null, wireHash);
        try (TtlvItem decoded = TtlvCodec.decode(wire);
                var view = decoded.view();
                var decodedValue = view.value()) {
            assertEquals(rawTag, view.tag().raw());
            assertEquals(expectedType, view.itemType());
            if (expectedType == TtlvItemType.DateTime) assertEquals(1L, decodedValue.dateTime());
            else assertEquals(0x0bL, decodedValue.enumeration());
        }
        appendRouteEvent(events, caseId, "java.generic-ttlv.ttlv_codec.decode", targetId,
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null, "decode", protocolVersion,
                null, null, wireHash);
    }

    private void appendControlledActivationDateNetworkEvents(ArrayNode events, String protocolVersion)
            throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        int minor = "2.0".equals(protocolVersion) ? 0 : 1;
        SSLServerSocket listener = listener(certificate(serverCertificatePem),
                privateKey(serverPrivateKeyPem), certificate(clientCertificatePem));
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveActivationDateRequests(listener, minor));
        Thread peerThread = new Thread(peer, "java-validator-activation-date-peer-" + protocolVersion);
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("activation-date-" + protocolVersion + ".json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, false, protocolVersion));
        try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
            try (ClientBatchItem request = ClientBatchItem.getAttributes(
                    UniqueIdentifier.textString("object-7"), List.of(AttributeReference.tag(0x420001)))
                    .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII))) {
                executeGetAttributesForEvidence(driver, request, true);
            }
            try (GetAttributesRequest request = GetAttributesRequest.create(
                    UniqueIdentifier.textString("object-7"), List.of(AttributeReference.tag(0x420001)));
                    ClientBatchItem batchItem = request.toBatchItem()
                            .withUniqueBatchItemId("item-0".getBytes(StandardCharsets.US_ASCII))) {
                executeGetAttributesForEvidence(driver, batchItem, false);
            }
        } finally {
            listener.close();
        }

        List<byte[]> requests = peer.get(15, TimeUnit.SECONDS);
        assertEquals(2, requests.size());
        for (int index = 0; index < requests.size(); index++) {
            byte[] observedRequest = requests.get(index);
            assertEquals(0x0bL, requestOperation(observedRequest));
            assertEquals(minor, requestProtocolMinorVersion(observedRequest));
            assertTrue(contains(observedRequest, item(0x42013b, 5, intBytes(0x420001))),
                    "The controlled peer must receive the requested Activation Date reference.");
            String routeId = index == 0 ? "java.typed-protocol.client_batch_item.get_attributes"
                    : "java.typed-protocol.function.attribute_request_get_attributes_to_batch_item";
            String symbol = index == 0 ? "org.kmipkit.extensions.ClientBatchItem.getAttributes"
                    : "org.kmipkit.extensions.GetAttributesRequest.toBatchItem";
            String requestHash = normalizedTreeSha256(observedRequest);
            String responseHash = normalizedTreeSha256(activationDateResponse(minor));
            appendTypedNetworkRouteEvents(events, "test-attr-activation-date", routeId,
                    "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES", symbol, protocolVersion,
                    requestHash, responseHash);
        }
        appendRouteEvent(events, "test-attr-activation-date",
                "java.typed-protocol.function.client_batch_item_response_get_attributes",
                "KMIPKIT-ELEM-OP-C2S-GET-ATTRIBUTES",
                "org.kmipkit.extensions.ClientBatchItemResult.getAttributesResponse", null, null,
                "decode", protocolVersion, null, null,
                normalizedTreeSha256(activationDateResponse(minor)));
    }

    private static void executeGetAttributesForEvidence(JavaKmipDriver driver, ClientBatchItem request,
            boolean readTypedResponse) throws Exception {
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(request);
            try (ClientBatchResponse response = driver.executeBatchForEvidence(batch);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(0x0bL, result.operation());
                assertEquals(0L, result.resultStatus());
                if (readTypedResponse) {
                    try (var typed = result.getAttributesResponse().orElseThrow();
                            TtlvStructure attributes = typed.attributes().orElseThrow();
                            var view = attributes.view();
                            var attribute = view.itemAt(0);
                            var value = attribute.value()) {
                        assertEquals("object-7", typed.uniqueIdentifier());
                        assertEquals(1, view.itemCount());
                        assertEquals(0x420001, attribute.tag().raw());
                        assertEquals(TtlvItemType.DateTime, attribute.itemType());
                        assertEquals(1L, value.dateTime());
                    }
                }
            }
        }
    }

    private static List<byte[]> serveActivationDateRequests(SSLServerSocket listener, int minor)
            throws Exception {
        var observedRequests = new ArrayList<byte[]>();
        byte[] response = activationDateResponse(minor);
        for (int index = 0; index < 2; index++) {
            byte[] request = serveOneAndCapture(listener, response);
            if (requestOperation(request) != 0x0bL || requestProtocolMinorVersion(request) != minor) {
                throw new AssertionError("The controlled peer observed an unexpected Get Attributes request.");
            }
            observedRequests.add(request);
        }
        return List.copyOf(observedRequests);
    }

    private static byte[] activationDateResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] attributes = structure(0x420125, item(0x420001, 9, longBytes(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "object-7".getBytes(StandardCharsets.UTF_8)), attributes);
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x0b)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static void appendAllocatedTagRangeEvents(ArrayNode events, TagRangeFixture fixture,
            String protocolVersion, byte[] wire, String treeHash) throws Exception {
        try (RawTag raw = RawTag.fromRaw(fixture.rawTag());
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Enumeration(0);
                TtlvItem created = TtlvItem.create(tag, value, CodecLimits.defaults());
                var view = created.view();
                var viewValue = view.value()) {
            assertEquals(fixture.rawTag(), view.tag().raw());
            assertEquals(TtlvItemType.Enumeration, view.itemType());
            assertEquals(0L, viewValue.enumeration());
            appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_item.create",
                    fixture.targetId(), "org.kmipkit.ttlv.TtlvItem.create", null, null,
                    "construct", protocolVersion, null, treeHash, null);
            appendRouteEvent(events, fixture.caseId(),
                    "java.generic-ttlv.function.ttlv_item_view_from_item", fixture.targetId(),
                    "org.kmipkit.ttlv.TtlvItem.view", null, null,
                    "decode", protocolVersion, null, null, treeHash);
        }
        try (TtlvItem decoded = TtlvCodec.decode(wire); var decodedView = decoded.view()) {
            assertEquals(fixture.rawTag(), decodedView.tag().raw());
            assertEquals(TtlvItemType.Enumeration, decodedView.itemType());
        }
        appendRouteEvent(events, fixture.caseId(), "java.generic-ttlv.ttlv_codec.decode",
                fixture.targetId(), "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null, null, treeHash);
    }

    private static void appendRejectedTagRangeEvent(ArrayNode events, TagRangeFixture fixture,
            String protocolVersion, byte[] wire) throws Exception {
        TtlvDecodeException rejection = assertThrows(
                TtlvDecodeException.class, () -> TtlvCodec.decode(wire));
        int expectedKind = "reserved".equals(fixture.allocation())
                ? TtlvDecodeErrorKind.RESERVED_TAG : TtlvDecodeErrorKind.UNALLOCATED_TAG;
        assertEquals(expectedKind, rejection.kind());
        ObjectNode event = newRouteEvent(events, fixture.caseId(),
                "java.generic-ttlv.ttlv_codec.decode", fixture.targetId(),
                "org.kmipkit.ttlv.TtlvCodec.decode", null, null,
                "decode", protocolVersion, null);
        event.put("decode_outcome", "rejected");
        event.put("decode_rejection_code", "reserved".equals(fixture.allocation())
                ? "tag_reserved" : "tag_not_allocated");
        event.put("decode_input_sha256", sha256(wire));
    }

    private static void appendNetworkQueryRouteEvents(ArrayNode events, String protocolVersion,
            String requestTreeHash, String responseTreeHash) {
        String routeId = "java.typed-protocol.client_batch_item.query";
        String symbol = "org.kmipkit.extensions.ClientBatchItem.query";
        appendTypedNetworkRouteEvents(events, "test-op-query", routeId,
                "KMIPKIT-ELEM-OP-C2S-QUERY", symbol, protocolVersion, requestTreeHash, responseTreeHash);
        appendRouteEvent(events, "test-op-query", "java.typed-protocol.function.client_batch_item_response_query",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.extensions.ClientBatchItemResult.queryResponse",
                null, null, "decode", protocolVersion, null, null, responseTreeHash);
        appendRouteEvent(events, "test-op-query", "java.typed-protocol.function.client_batch_item_response_query_field_at",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.extensions.QueryResponse.fieldAt",
                null, null, "decode", protocolVersion, null, null, responseTreeHash);
        appendRouteEvent(events, "test-op-query", "java.typed-protocol.function.client_batch_item_response_query_field_count",
                "KMIPKIT-ELEM-OP-C2S-QUERY", "org.kmipkit.extensions.QueryResponse.fieldCount",
                null, null, "decode", protocolVersion, null, null, responseTreeHash);
    }

    private static void appendTypedNetworkRouteEvents(ArrayNode events, String caseId, String routeId,
            String targetId, String publicApiSymbol, String protocolVersion,
            String requestTreeHash, String responseTreeHash) {
        String executionRoute = "java.execute.typed-batch";
        String executionSymbol = "org.kmipkit.extensions.Client.execute";
        appendRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRoute, executionSymbol, "construct", protocolVersion, "raw_tls",
                requestTreeHash, null);
        ObjectNode sent = appendRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRoute, executionSymbol, "send", protocolVersion, "raw_tls",
                requestTreeHash, null);
        sent.put("request_delivery_state", "response_received");
        sent.put("transport_invocation_count", 1);
        appendRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRoute, executionSymbol, "decode", protocolVersion, "raw_tls",
                null, responseTreeHash);
    }

    private static ObjectNode appendRouteEvent(ArrayNode events, String caseId, String routeId,
            String targetId, String publicApiSymbol, String executionRouteId, String executionApiSymbol,
            String action, String protocolVersion, String transport, String requestHash,
            String responseHash) {
        ObjectNode event = newRouteEvent(events, caseId, routeId, targetId, publicApiSymbol,
                executionRouteId, executionApiSymbol, action, protocolVersion, transport);
        if (requestHash != null) event.put("request_tree_sha256", requestHash);
        if (responseHash != null) event.put("response_tree_sha256", responseHash);
        return event;
    }

    private static ObjectNode newRouteEvent(ArrayNode events, String caseId, String routeId,
            String targetId, String publicApiSymbol, String executionRouteId, String executionApiSymbol,
            String action, String protocolVersion, String transport) {
        ObjectNode event = events.addObject();
        event.put("case_id", caseId);
        event.put("route_id", routeId);
        event.put("target_id", targetId);
        event.put("public_api_symbol", publicApiSymbol);
        if (executionRouteId == null) event.putNull("execution_route_id");
        else event.put("execution_route_id", executionRouteId);
        if (executionApiSymbol == null) event.putNull("execution_api_symbol");
        else event.put("execution_api_symbol", executionApiSymbol);
        event.put("invocation_id", UUID.randomUUID().toString());
        event.put("action", action);
        event.put("protocol_version", protocolVersion);
        event.put("encoding", "TTLV");
        if (transport == null) event.putNull("transport");
        else event.put("transport", transport);
        event.put("status", "passed");
        event.put("invocation_count", 1);
        return event;
    }

    private static void writeRouteExecutionReport(Path reportPath, ObjectMapper mapper,
            ArrayNode events) throws Exception {
        Path catalogPath = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");
        Path manifestPath = Path.of("../shared/coverage/route-capabilities.json");
        Path pinPath = Path.of("library-pin.json");
        String libraryCommit = mapper.readTree(pinPath.toFile()).path("commit").asText();
        var artifact = ArtifactProvenance.load(Path.of(System.getProperty("kmipkit.artifactManifest")), libraryCommit);
        Path loadedJar = Path.of(ClientBatchItem.class.getProtectionDomain().getCodeSource()
                .getLocation().toURI()).toAbsolutePath().normalize();
        assertEquals(artifact.artifactPath(), loadedJar,
                "All route events must come from the independently installed package JAR.");

        ObjectNode report = mapper.createObjectNode();
        report.put("schema_version", "1.0");
        report.put("run_id", "java-controlled-query-" + UUID.randomUUID());
        report.put("language", "java");
        ObjectNode server = report.putObject("server");
        server.put("id", "java-controlled-tls-peer");
        server.put("product", "KMIPKit Java validator controlled fixture");
        server.put("version", "fixture-1");
        server.put("configuration_fingerprint", sha256("java-controlled-query-peer-fixture-v1"));
        report.put("library_commit", artifact.libraryCommit());
        report.put("artifact_sha256", artifact.artifactSha256());
        report.put("catalog_sha256", ArtifactProvenance.sha256(catalogPath));
        report.put("route_manifest_sha256", ArtifactProvenance.sha256(manifestPath));
        ArrayNode caseIds = report.putArray("case_ids").add("test-op-query");
        TAG_RANGE_FIXTURES.forEach(fixture -> caseIds.add(fixture.caseId()));
        caseIds.add("test-attr-activation-date");
        report.putArray("protocol_versions").add("2.0").add("2.1");
        ObjectNode combination = mapper.createObjectNode();
        combination.put("encoding", "TTLV");
        combination.put("transport", "raw_tls");
        report.putArray("network_combinations").add(combination);
        report.set("events", events);
        Files.createDirectories(reportPath.toAbsolutePath().normalize().getParent());
        mapper.writerWithDefaultPrettyPrinter().writeValue(reportPath.toFile(), report);
    }

    private static JsonNode findEvent(JsonNode events, String action, String protocolVersion) {
        for (JsonNode event : events) {
            if (action.equals(event.path("action").asText())
                    && protocolVersion.equals(event.path("protocol_version").asText())
                    && "test-op-query".equals(event.path("case_id").asText())
                    && event.hasNonNull("execution_api_symbol")) return event;
        }
        throw new AssertionError("The expected controlled Query route event is absent.");
    }

    private static JsonNode findRouteEvent(JsonNode events, String routeId, String protocolVersion) {
        for (JsonNode event : events) {
            if (routeId.equals(event.path("route_id").asText())
                    && protocolVersion.equals(event.path("protocol_version").asText())) return event;
        }
        throw new AssertionError("The expected installed Java API route event is absent.");
    }

    private static byte[] queryOperationTtlv() throws Exception {
        return item(0x42005c, 5, intBytes(0x18));
    }

    private static String normalizedTreeSha256(byte[] wire) throws Exception {
        StringBuilder normalized = new StringBuilder();
        int next = appendNormalizedItem(wire, 0, normalized);
        if (next != wire.length) throw new IllegalArgumentException("TTLV fixture has trailing bytes");
        return sha256(normalized.toString().getBytes(StandardCharsets.UTF_8));
    }

    private static int appendNormalizedItem(byte[] wire, int offset, StringBuilder normalized) {
        if (offset < 0 || wire.length - offset < 8) {
            throw new IllegalArgumentException("TTLV fixture item header is incomplete");
        }
        int tag = (Byte.toUnsignedInt(wire[offset]) << 16)
                | (Byte.toUnsignedInt(wire[offset + 1]) << 8)
                | Byte.toUnsignedInt(wire[offset + 2]);
        int type = Byte.toUnsignedInt(wire[offset + 3]);
        int length = ByteBuffer.wrap(wire, offset + 4, 4).getInt();
        if (length < 0) throw new IllegalArgumentException("TTLV fixture item length is invalid");
        int padded = Math.addExact(length, 7) & ~7;
        int valueOffset = Math.addExact(offset, 8);
        int valueEnd = Math.addExact(valueOffset, length);
        int itemEnd = Math.addExact(valueOffset, padded);
        if (itemEnd > wire.length || valueEnd > itemEnd) {
            throw new IllegalArgumentException("TTLV fixture item is incomplete");
        }
        if (tag != 0x420092) {
            normalized.append(String.format(java.util.Locale.ROOT, "%06x:%02x:", tag, type));
            if (type == 1) {
                int child = valueOffset;
                while (child < valueEnd) child = appendNormalizedItem(wire, child, normalized);
            } else {
                normalized.append(HexFormat.of().formatHex(wire, valueOffset, valueEnd));
            }
            normalized.append(';');
        }
        return itemEnd;
    }

    private static String sha256(String value) throws Exception {
        return sha256(value.getBytes(StandardCharsets.UTF_8));
    }

    private static String sha256(byte[] value) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(value));
    }

    private static int requestProtocolMinorVersion(byte[] request) {
        byte[] minorHeader = new byte[] {0x42, 0x00, 0x6b, 2, 0, 0, 0, 4};
        for (int offset = 0; offset <= request.length - minorHeader.length - 4; offset++) {
            boolean matches = true;
            for (int index = 0; index < minorHeader.length; index++) {
                if (request[offset + index] != minorHeader[index]) {
                    matches = false;
                    break;
                }
            }
            if (matches) return ByteBuffer.wrap(request, offset + minorHeader.length, 4).getInt();
        }
        throw new AssertionError("The outgoing KMIP request omitted ProtocolVersionMinor.");
    }

    private static byte[] unsupportedResponse(long operation) throws Exception {
        return unsupportedResponse(operation, 1);
    }

    private static byte[] unsupportedResponse(long operation, int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(1)),
                item(0x42007e, 5, intBytes(Math.toIntExact(KmipValues.ResultReason.OPERATION_NOT_SUPPORTED))));
        return structure(0x42007b, header, responseItem);
    }

    private static List<byte[]> serveAsynchronousResponses(SSLServerSocket listener, int count) throws Exception {
        return serveAsynchronousResponses(listener, count, 1);
    }

    private static List<byte[]> serveAsynchronousResponses(SSLServerSocket listener, int count,
            int protocolMinorVersion) throws Exception {
        var requests = new ArrayList<byte[]>();
        for (int index = 0; index < count; index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                requests.add(request);
                socket.getOutputStream().write(asynchronousResponse(requestOperation(request),
                        requestCorrelation(request), protocolMinorVersion));
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static boolean acceptsConnection(SSLServerSocket listener) throws Exception {
        try (SSLSocket ignored = (SSLSocket) listener.accept()) {
            return true;
        } catch (java.net.SocketTimeoutException timeout) {
            return false;
        }
    }

    private static List<CatalogLoader.CatalogCase> casesById(CatalogLoader.Catalog catalog, String... caseIds) {
        return java.util.Arrays.stream(caseIds)
                .map(caseId -> catalog.cases().stream().filter(testCase -> testCase.caseId().equals(caseId))
                        .findFirst().orElseThrow())
                .toList();
    }

    private static byte[] asynchronousResponse(long operation, byte[] correlation, int protocolMinorVersion)
            throws Exception {
        byte[] version = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, version,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        List<byte[]> fields = new ArrayList<>();
        fields.add(item(0x42005c, 5, intBytes(Math.toIntExact(operation))));
        fields.add(item(0x42007f, 5, intBytes(operation == 0x19 ? 0 : 2)));
        if (operation == 0x19) {
            fields.add(structure(0x42007c, item(0x420006, 8, correlation),
                    item(0x420012, 5, intBytes(1))));
        } else {
            fields.add(item(0x420006, 8, correlation));
            if (operation != 0x1a) fields.add(structure(0x42007c));
        }
        return structure(0x42007b, header, structure(0x42000f, fields.toArray(byte[][]::new)));
    }

    private static byte[] requestCorrelation(byte[] request) {
        for (int offset = 0; offset <= request.length - 8; offset++) {
            if (request[offset] == 0x42 && request[offset + 1] == 0x00 && request[offset + 2] == 0x06
                    && request[offset + 3] == 8) {
                int length = ByteBuffer.wrap(request, offset + 4, 4).getInt();
                if (length < 1 || length > 1024 || offset + 8 + length > request.length) break;
                return java.util.Arrays.copyOfRange(request, offset + 8, offset + 8 + length);
            }
        }
        throw new AssertionError("The asynchronous request omitted its correlation value.");
    }

    private static byte[] readTtlv(InputStream input) throws Exception {
        DataInputStream data = new DataInputStream(input);
        byte[] header = data.readNBytes(8);
        if (header.length != 8) throw new IllegalStateException("TTLV request header is incomplete");
        long payloadLength = Integer.toUnsignedLong(ByteBuffer.wrap(header, 4, 4).getInt());
        if (payloadLength > 16 * 1024 * 1024L) throw new IllegalStateException("TTLV request is oversized");
        int paddedLength = Math.toIntExact((payloadLength + 7) & ~7L);
        byte[] payload = data.readNBytes(paddedLength);
        if (payload.length != paddedLength) throw new IllegalStateException("TTLV request is incomplete");
        byte[] frame = new byte[header.length + payload.length];
        System.arraycopy(header, 0, frame, 0, header.length);
        System.arraycopy(payload, 0, frame, header.length, payload.length);
        return frame;
    }

    private static byte[] discoverVersionsResponse() throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = structure(0x42007c, protocolVersion);
        byte[] responseItem = structure(0x42000f, item(0x42005c, 5, intBytes(30)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] queryResponse() throws Exception {
        return queryResponse(1);
    }

    private static byte[] queryResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x18)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)),
                structure(0x42007c,
                        item(0x42005c, 5, intBytes(0x3b)),
                        structure(0x42015f)));
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] getResponse() throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] keyMaterial = new byte[32];
        java.util.Arrays.fill(keyMaterial, (byte) 0x5a);
        byte[] keyBlock = structure(0x420040,
                item(0x420042, 5, intBytes(1)),
                structure(0x420043, item(0x420044, 8, keyMaterial)),
                item(0x420028, 5, intBytes(3)),
                item(0x42002a, 2, intBytes(256)));
        byte[] payload = structure(0x42007c,
                item(0x420057, 5, intBytes(2)),
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)),
                structure(0x42008f, keyBlock));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(10)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] identifierResponse(long operation) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] identifier = item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8));
        byte[] payload = operation == 0x0c
                ? structure(0x42007c, identifier, item(0x42013b, 5, intBytes(0x42002f)))
                : structure(0x42007c, identifier);
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] logResponse() throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x2c)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), structure(0x42007c));
        return structure(0x42007b, header, responseItem);
    }

    private static List<byte[]> serveCreateCleanup(SSLServerSocket listener) throws Exception {
        var requests = new ArrayList<byte[]>();
        for (int exchange = 0; exchange < 2; exchange++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                requests.add(request);
                byte[] operation = new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4,
                        0, 0, 0, (byte) (exchange == 0 ? 1 : 0x14)};
                if (!contains(request, operation)) {
                    throw new AssertionError("The controlled create recipe sent an unexpected operation.");
                }
                byte[] response = exchange == 0 ? createResponse() : destroyResponse();
                socket.getOutputStream().write(response);
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static List<byte[]> serveObjectFixtureAndGet(SSLServerSocket listener) throws Exception {
        var requests = new ArrayList<byte[]>();
        byte[][] responses = {createResponse(), getResponse(), destroyResponse()};
        for (byte[] response : responses) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                requests.add(readTtlv(socket.getInputStream()));
                socket.getOutputStream().write(response);
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static List<byte[]> serveAttributeReadCases(SSLServerSocket listener,
            List<Long> targetOperations, List<Long> targetAttributeTags) throws Exception {
        return serveAttributeReadCases(listener, targetOperations, targetAttributeTags, targetAttributeTags,
                java.util.Collections.nCopies(targetOperations.size(), 3L));
    }

    private static List<byte[]> serveAttributeReadCases(SSLServerSocket listener,
            List<Long> targetOperations, List<Long> targetAttributeTags, List<Long> responseAttributeTags)
            throws Exception {
        return serveAttributeReadCases(listener, targetOperations, targetAttributeTags, responseAttributeTags,
                java.util.Collections.nCopies(targetOperations.size(), 3L));
    }

    private static List<byte[]> serveAttributeReadCases(SSLServerSocket listener,
            List<Long> targetOperations, List<Long> targetAttributeTags, List<Long> responseAttributeTags,
            List<Long> responseAlgorithmValues) throws Exception {
        var requests = new ArrayList<byte[]>();
        for (int index = 0; index < targetOperations.size(); index++) {
            long targetOperation = targetOperations.get(index);
            byte[] createRequest = serveOneAndCapture(listener, createResponse());
            requests.add(createRequest);
            String fixtureName = requestTextString(createRequest, 0x420055);
            requests.add(serveOneAndCapture(listener, getAttributesResponse(targetOperation,
                    responseAttributeTags.get(index), responseAlgorithmValues.get(index), fixtureName)));
            requests.add(serveOneAndCapture(listener, destroyResponse()));
        }
        return List.copyOf(requests);
    }

    private static byte[] serveOneAndCapture(SSLServerSocket listener, byte[] response) throws Exception {
        try (SSLSocket socket = (SSLSocket) listener.accept()) {
            socket.setEnabledProtocols(new String[] {"TLSv1.3"});
            socket.setSoTimeout(10_000);
            socket.startHandshake();
            byte[] request = readTtlv(socket.getInputStream());
            socket.getOutputStream().write(response);
            socket.getOutputStream().flush();
            return request;
        }
    }

    private static String requestTextString(byte[] request, int tag) {
        byte[] header = new byte[] {(byte) ((tag >>> 16) & 0xff), (byte) ((tag >>> 8) & 0xff),
                (byte) (tag & 0xff), 7, 0, 0, 0, 0};
        for (int offset = 0; offset <= request.length - header.length; offset++) {
            boolean matches = true;
            for (int index = 0; index < 4; index++) {
                if (request[offset + index] != header[index]) {
                    matches = false;
                    break;
                }
            }
            if (!matches) continue;
            int length = ByteBuffer.wrap(request, offset + 4, 4).getInt();
            if (length < 0 || offset + 8 + length > request.length) {
                throw new AssertionError("The Create request contained an invalid Name Value length.");
            }
            return new String(request, offset + 8, length, StandardCharsets.UTF_8);
        }
        throw new AssertionError("The Create request omitted its Name Value.");
    }

    private static byte[] getAttributesResponse(long operation, long attributeTag, long algorithm,
            String fixtureName) throws Exception {
        byte[] value;
        if (attributeTag == 0x420028L) {
            value = item(0x420028, 5, intBytes(Math.toIntExact(algorithm)));
        } else if (attributeTag == 0x420053L) {
            value = structure(0x420053,
                    item(0x420055, 7, fixtureName.getBytes(StandardCharsets.UTF_8)),
                    item(0x420054, 5, intBytes(1)));
        } else {
            throw new IllegalArgumentException("No controlled attribute fixture is defined.");
        }
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] attributes = structure(0x420125, value);
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)), attributes);
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(Math.toIntExact(operation))),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] createResponse() throws Exception {
        return createResponse(1);
    }

    private static byte[] createResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = structure(0x42007c,
                item(0x420057, 5, intBytes(2)),
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(1)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] destroyResponse() throws Exception {
        return destroyResponse(1);
    }

    private static byte[] destroyResponse(int protocolMinorVersion) throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(protocolMinorVersion)));
        byte[] header = structure(0x42007a, protocolVersion,
                item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, intBytes(0x14)),
                item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)),
                item(0x42007f, 5, intBytes(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] structure(int tag, byte[]... children) throws Exception {
        ByteArrayOutputStream body = new ByteArrayOutputStream();
        for (byte[] child : children) body.write(child);
        return item(tag, 1, body.toByteArray());
    }

    private static byte[] item(int tag, int type, byte[] value) throws Exception {
        ByteArrayOutputStream item = new ByteArrayOutputStream();
        item.write((tag >>> 16) & 0xff);
        item.write((tag >>> 8) & 0xff);
        item.write(tag & 0xff);
        item.write(type);
        item.write(ByteBuffer.allocate(4).putInt(value.length).array());
        item.write(value);
        int padding = (8 - value.length % 8) % 8;
        item.write(new byte[padding]);
        return item.toByteArray();
    }

    private static byte[] intBytes(int value) {
        return ByteBuffer.allocate(4).putInt(value).array();
    }

    private static byte[] longBytes(long value) {
        return ByteBuffer.allocate(8).putLong(value).array();
    }

    private static X509Certificate certificate(byte[] pem) throws Exception {
        return (X509Certificate) CertificateFactory.getInstance("X.509")
                .generateCertificate(new java.io.ByteArrayInputStream(pem));
    }

    private static PrivateKey privateKey(byte[] pem) throws Exception {
        String encoded = new String(pem, StandardCharsets.US_ASCII)
                .replace("-----BEGIN PRIVATE KEY-----", "")
                .replace("-----END PRIVATE KEY-----", "").replaceAll("\\s", "");
        return KeyFactory.getInstance("RSA").generatePrivate(
                new PKCS8EncodedKeySpec(Base64.getDecoder().decode(encoded)));
    }

    private static byte[] fixture(String name) throws Exception {
        Path path = Path.of("../../bindings/python/tests/fixtures/login-mtls", name);
        return Files.readAllBytes(path);
    }

    private static boolean contains(byte[] data, byte[] target) {
        outer: for (int index = 0; index <= data.length - target.length; index++) {
            for (int offset = 0; offset < target.length; offset++) {
                if (data[index + offset] != target[offset]) continue outer;
            }
            return true;
        }
        return false;
    }

    private static boolean containsTag(byte[] data, int tag) {
        byte[] tagBytes = new byte[] {(byte) ((tag >>> 16) & 0xff), (byte) ((tag >>> 8) & 0xff),
                (byte) (tag & 0xff)};
        outer: for (int offset = 0; offset <= data.length - 4; offset++) {
            for (int index = 0; index < tagBytes.length; index++) {
                if (data[offset + index] != tagBytes[index]) continue outer;
            }
            if (data[offset + 3] == 5) return true;
        }
        return false;
    }

    private record TagRangeFixture(String caseId, String targetId, int rawTag, String allocation) {}

    private record EnumerationFixture(String caseId, String targetId, int rawTag, long value,
            List<String> protocolVersions) {}

    private record MatrixPlan(List<Long> expectedOperations, List<byte[]> responses) {}
}
