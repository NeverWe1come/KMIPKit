package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.protocol.KmipValues;

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
import java.util.List;
import java.util.Set;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ControlledTlsPeerTest {
    private static final String LOG_MESSAGE = "KMIPKit interoperability diagnostic";
    private static final String CREATED_OBJECT_ID = "controlled-create-object-id";

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
        FutureTask<List<Long>> peer = new FutureTask<>(() -> serveOperationMatrixResponses(listener,
                matrixPlan, peerTrace));
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
        assertTrue(!casesByOperation.isEmpty(), "KMIP 2.0 must have applicable public client operations.");

        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = listener(certificate(serverCertificatePem), privateKey(serverPrivateKeyPem),
                certificate(clientCertificatePem));
        MatrixPlan matrixPlan = operationMatrixPlan(casesByOperation, 0);
        var peerTrace = new java.util.concurrent.CopyOnWriteArrayList<Long>();
        FutureTask<List<Long>> peer = new FutureTask<>(() -> serveOperationMatrixResponses(listener,
                matrixPlan, peerTrace));
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
                () -> serveAttributeReadCases(listener, targetOperations));
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
            MatrixPlan plan, List<Long> peerTrace) throws Exception {
        var observedOperations = new ArrayList<Long>();
        for (int index = 0; index < plan.expectedOperations().size(); index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                peerTrace.add(-1L);
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
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
            List<Long> targetOperations) throws Exception {
        var requests = new ArrayList<byte[]>();
        for (long targetOperation : targetOperations) {
            byte[][] responses = {
                    createResponse(), identifierResponse(targetOperation), destroyResponse()
            };
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
        }
        return List.copyOf(requests);
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

    private record MatrixPlan(List<Long> expectedOperations, List<byte[]> responses) {}
}
