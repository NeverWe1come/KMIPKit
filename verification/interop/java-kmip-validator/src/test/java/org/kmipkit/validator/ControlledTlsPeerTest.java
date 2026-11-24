package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;
import org.junit.jupiter.params.provider.ValueSource;
import org.kmipkit.client.KmipClient;
import org.kmipkit.credentials.CredentialType;
import org.kmipkit.credentials.SecretText;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.fluent.CreateCredentialObject;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetAddress;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyFactory;
import java.security.MessageDigest;
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
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ControlledTlsPeerTest {
    private static final String LOG_MESSAGE = "KMIPKit interoperability diagnostic";
    private static final String CREATED_OBJECT_ID = "controlled-create-object-id";

    @TempDir
    Path temporaryDirectory;

    @ParameterizedTest
    @ValueSource(ints = {0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42})
    void draftTypedAndFluentBuildersSendTheSameVersionedRequest(int operation) throws Exception {
        assertArrayEquals(draftExchange(operation, false), draftExchange(operation, true));
    }

    private static byte[] draftExchange(int operation, boolean fluent) throws Exception {
        byte[] serverPem = fixture("server-cert.pem");
        byte[] clientPem = fixture("client-cert.pem");
        byte[] clientKey = fixture("client-key.pem");
        var limits = CodecLimits.defaults();
        try (var listener = listener(certificate(serverPem), privateKey(fixture("server-key.pem")),
                certificate(clientPem));
                var registry = ClientExtensionRegistry.create(List.of(), ExtensionRegistryLimits.defaults());
                var defaults = ClientConfiguration.create(registry);
                var configuration = defaults.withProtocolVersion(new ProtocolVersion(3, 0));
                var transportBuilder = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientPem}, clientKey)
                        .trustCertificatesPem(new byte[][] {serverPem})
                        .tlsServerName("localhost");
                var transport = transportBuilder.build();
                var client = KmipClient.create(configuration, transport);
                var source = TtlvStructure.create();
                var attrs = AttributeSet.tryNew(source);
                var request = draftRequest(client, attrs, limits, operation, fluent);
                var batch = ClientBatch.create()) {
            var peer = new FutureTask<>(() -> serveOneResponse(listener, draftRejection(operation)));
            Thread thread = new Thread(peer, "java-validator-kmip30-peer");
            thread.setDaemon(true);
            thread.start();
            batch.add(request);
            try (var response = client.execute(batch, limits); var item = response.itemAt(0)) {
                assertEquals(operation, item.operation());
                assertEquals(1, item.resultStatus());
                assertEquals(5, item.resultReason().orElseThrow());
            }
            byte[] wire = peer.get(15, TimeUnit.SECONDS);
            assertTrue(contains(wire, item(0x42006a, 2, intBytes(3))));
            assertTrue(contains(wire, item(0x42005c, 5, intBytes(operation))));
            return wire;
        }
    }

    private static ClientBatchItem draftRequest(KmipClient client, AttributeSet attrs,
            CodecLimits limits, int operation, boolean fluent) throws Exception {
        var id = UniqueIdentifier.identifier("disposable-peer-object");
        byte[] ciphertext = {1, 2, 3};
        var builder = client.request();
        return switch (operation) {
            case 0x3c -> fluent ? builder.createGroup(attrs).build() : ClientBatchItem.createGroupFromAttributes(attrs);
            case 0x3d -> fluent ? builder.obliterate(id).build() : ClientBatchItem.obliterateFromIdentifier(id);
            case 0x3e -> fluent ? builder.createUser(attrs).build() : ClientBatchItem.createUserFromAttributes(attrs);
            case 0x3f -> draftCredential(builder, attrs, limits, fluent);
            case 0x40 -> fluent ? builder.deactivate(id).build() : ClientBatchItem.deactivateFromFields(id, null, null);
            case 0x41 -> fluent ? builder.encapsulate(id).build() : ClientBatchItem.encapsulateFromFields(id, null, null);
            case 0x42 -> fluent ? builder.decapsulate(id, ciphertext).build()
                    : ClientBatchItem.decapsulateFromFields(id, ciphertext, null, null);
            default -> throw new IllegalArgumentException("Unexpected draft operation");
        };
    }

    private static ClientBatchItem draftCredential(org.kmipkit.fluent.FluentRequest request, AttributeSet attrs,
            CodecLimits limits, boolean fluent) throws Exception {
        String fixturePassword = "disposable-peer-credential";
        if (fluent) {
            try (var secret = SecretText.fromString(fixturePassword);
                    var object = CreateCredentialObject.password(secret, null, null, null, null);
                    var builder = request.createCredential(attrs, object)) {
                return builder.build();
            }
        }
        try (var empty = TtlvStructure.create();
                var fieldTag = RawTag.fromRaw(0x4200a1); var field = fieldTag.checked();
                var text = TtlvValue.TextString(fixturePassword.getBytes(StandardCharsets.UTF_8), limits);
                var password = TtlvItem.create(field, text, limits);
                var fields = TtlvStructure.withItem(empty, password, limits);
                var wrapper = TtlvValue.structure(fields, limits);
                var objectTag = RawTag.fromRaw(0x4201a1); var object = objectTag.checked();
                var credential = TtlvItem.create(object, wrapper, limits)) {
            return ClientBatchItem.createCredentialFromFields(
                    CredentialType.fromRaw(CredentialType.PASSWORD), 0x0d, attrs, credential);
        }
    }

    private static byte[] draftRejection(int operation) throws Exception {
        return structure(0x42007b,
                structure(0x42007a,
                        structure(0x420069, item(0x42006a, 2, intBytes(3)), item(0x42006b, 2, intBytes(0))),
                        item(0x420092, 9, longBytes(1))),
                structure(0x42000f, item(0x42005c, 5, intBytes(operation)),
                        item(0x42007f, 5, intBytes(1)), item(0x42007e, 5, intBytes(5))));
    }

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

        ArtifactProvenance.loadForCheckout(manifestPath);
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
        ObjectMapper mapper = new ObjectMapper();
        var plan = temporaryDirectory.resolve("plan.jsonl");
        String obligation = "c".repeat(64);
        var row = mapper.createObjectNode().put("language", "java").put("case_id", "test-op-query")
                .put("target_id", "KMIPKIT-ELEM-OP-C2S-QUERY").put("protocol_version", "2.1")
                .put("action", "send").put("execution_status", "required").put("encoding", "TTLV")
                .put("transport", "raw_tls").put("obligation_id", obligation)
                .put("public_api_symbol", "org.kmipkit.extensions.ClientBatchItem.query")
                .put("execution_api_symbol", "org.kmipkit.extensions.Client.execute");
        Files.writeString(plan, mapper.writeValueAsString(row));
        var evidence = temporaryDirectory.resolve("evidence");
        int exit;
        try {
            exit = DraftCampaign.run(new String[] {
                    "--plan", plan.toString(), "--matrix-sha256", "b".repeat(64), "--scope", "live",
                    "--connection", connectionFile.toString(), "--evidence", evidence.toString(),
            }, Path.of(System.getProperty("kmipkit.artifactManifest")));
        } finally {
            listener.close();
        }
        byte[] request = peer.get(15, TimeUnit.SECONDS);
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 0x18}),
                "The controlled peer must receive the typed Query operation.");
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x74, 0x05, 0, 0, 0, 4, 0, 0, 0, 1}),
                "The Query request must include a non-empty Query Function.");
        var event = mapper.readTree(evidence.toFile()).path("events").get(0);
        assertEquals(0, exit);
        assertEquals(obligation, event.path("obligation_id").asText());
        assertEquals("passed", event.path("status").asText());
        assertEquals("supported", event.path("server_compatibility").asText());
        assertEquals(0, event.path("result_status_code").asInt());
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

    @ParameterizedTest
    @CsvSource({"typed-protocol,0", "fluent,0", "typed-protocol,256", "fluent,256",
            "typed-protocol,128", "fluent,128"})
    void isolatedKeyRecipesReadAndCleanUp(String apiLevel, int returnedLength) throws Exception {
        boolean readLength = returnedLength != 0;
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        SSLServerSocket listener = listener(serverCertificate, privateKey(serverPrivateKeyPem), clientCertificate);
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveCreateCleanup(listener, returnedLength));
        Thread peerThread = new Thread(peer, "java-validator-controlled-create-peer");
        peerThread.setDaemon(true);
        peerThread.start();

        Path connectionFile = temporaryDirectory.resolve("create-connection.json");
        Files.writeString(connectionFile, connectionJson(listener.getLocalPort(), serverCertificatePem,
                clientCertificatePem, clientPrivateKeyPem, true, apiLevel));
        var connection = ConnectionConfig.load(connectionFile);
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var testCase = catalog.cases().stream()
                .filter(value -> value.caseId().equals(readLength ? "test-attr-cryptographic-length" : "test-op-create"))
                .findFirst().orElseThrow();

        JavaKmipDriver.CaseOutcome outcome;
        try (JavaKmipDriver driver = JavaKmipDriver.connect(connection)) {
            outcome = driver.execute(testCase);
        } finally {
            listener.close();
        }
        assertEquals(returnedLength == 128 ? "failed" : "passed", outcome.status(), outcome.toString());
        List<byte[]> requests = peer.get(15, TimeUnit.SECONDS);
        assertEquals(readLength ? 3 : 2, requests.size());
        if (readLength) {
            assertTrue(contains(requests.get(1), item(0x42013b, 5, intBytes(0x42002a))));
            assertTrue(contains(requests.get(1), item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))));
        }
        byte[] cleanup = requests.get(requests.size() - 1);
        assertTrue(contains(requests.get(0), new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4, 0, 0, 0, 1}),
                "The controlled peer must receive Create before cleanup.");
        assertTrue(contains(requests.get(0), new byte[] {0x42, 0x00, 0x53, 1}),
                "The Create request must include an isolated Name attribute.");
        assertTrue(contains(requests.get(0), (readLength ? "kmipkit-attr-cryptographic-length-" : "kmipkit-create-").getBytes(StandardCharsets.UTF_8)),
                "The Create Name must use the catalog's unique prefix.");
        assertTrue(contains(requests.get(0), item(0x420054, 5, intBytes(1))),
                "The Create Name must include the Name Type enumeration.");
        assertTrue(contains(requests.get(0), item(0x420028, 5, intBytes(3))),
                "The isolated object must request AES.");
        assertTrue(contains(requests.get(0), item(0x42002a, 2, intBytes(256))),
                "The isolated object must request a 256-bit key.");
        assertTrue(contains(cleanup, new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4, 0, 0, 0, 0x14}),
                "The created object must be destroyed in a separate exchange.");
        assertTrue(contains(cleanup, item(0x420094, 7,
                CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8))));
    }

    @ParameterizedTest
    @CsvSource({"2.0,typed-protocol", "2.0,fluent", "2.1,typed-protocol", "2.1,fluent",
            "3.0,typed-protocol", "3.0,fluent"})
    void hashAndLocateRunThroughBothApis(String version, String apiLevel) throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        assertEquals("passed", probeReadOnly(catalog, "test-op-locate", 0x08, version, apiLevel,
                item(0x4200d5, 2, intBytes(0))).status());
        byte[] input = "abc".getBytes(StandardCharsets.US_ASCII);
        byte[] digest = MessageDigest.getInstance("SHA-256").digest(input);
        String hash = "test-crypto-hash-sha_256-0";
        assertEquals("passed", probeReadOnly(catalog, hash, 0x27, version, apiLevel,
                item(0x4200c2, 8, digest)).status());
        digest[0] ^= 1;
        assertEquals("failed", probeReadOnly(catalog, hash, 0x27, version, apiLevel,
                item(0x4200c2, 8, digest)).status());
    }

    @Test
    void draftQueryAcceptsOperationsWithoutLegacyStorageMasks() throws Exception {
        var catalog = CatalogLoader.load(Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        var outcome = probeReadOnly(catalog, "test-op-query", 0x18, "3.0", "typed-protocol",
                item(0x42005c, 5, intBytes(0x18)));
        assertEquals("passed", outcome.status(), outcome.toString());
    }

    private JavaKmipDriver.CaseOutcome probeReadOnly(CatalogLoader.Catalog catalog, String caseId,
            int operation, String version, String apiLevel, byte[] responseField) throws Exception {
        var testCase = catalog.cases().stream().filter(value -> value.caseId().equals(caseId))
                .findFirst().orElseThrow();
        byte[] serverPem = fixture("server-cert.pem");
        byte[] clientPem = fixture("client-cert.pem");
        byte[] clientKey = fixture("client-key.pem");
        boolean draft = version.equals("3.0");
        int major = draft ? 3 : 2;
        int minor = version.equals("2.1") ? 1 : 0;
        byte[] versionField = structure(0x420069,
                item(0x42006a, 2, intBytes(major)), item(0x42006b, 2, intBytes(minor)));
        byte[] timestamp = item(0x420092, 9, longBytes(1));
        byte[] header = draft ? structure(0x42007a, versionField, timestamp)
                : structure(0x42007a, versionField, timestamp, item(0x42000d, 2, intBytes(1)));
        byte[] operationField = item(0x42005c, 5, intBytes(operation));
        byte[] status = item(0x42007f, 5, intBytes(0));
        byte[] payload = structure(0x42007c, responseField);
        byte[] batch = draft ? structure(0x42000f, operationField, status, payload)
                : structure(0x42000f, operationField,
                        item(0x420093, 8, "item-0".getBytes(StandardCharsets.US_ASCII)), status, payload);
        byte[] response = structure(0x42007b, header, batch);
        try (var listener = listener(certificate(serverPem), privateKey(fixture("server-key.pem")),
                certificate(clientPem))) {
            var config = (ObjectNode) new ObjectMapper().readTree(connectionJson(listener.getLocalPort(),
                    serverPem, clientPem, clientKey, true, apiLevel));
            Path connectionFile = temporaryDirectory.resolve("probe-connection.json");
            Files.writeString(connectionFile, config.put("protocol_version", version).toString());
            var peer = new FutureTask<>(() -> serveOneResponse(listener, response));
            Thread thread = new Thread(peer, "java-validator-probe");
            thread.setDaemon(true);
            thread.start();
            try (JavaKmipDriver driver = JavaKmipDriver.connect(ConnectionConfig.load(connectionFile))) {
                var outcome = driver.execute(testCase);
                byte[] request = peer.get(15, TimeUnit.SECONDS);
                assertTrue(contains(request, versionField));
                assertTrue(contains(request, operationField));
                if (operation == 0x27) {
                    assertTrue(contains(request, item(0x420038, 5, intBytes(6))));
                    assertTrue(contains(request, item(0x4200c2, 8, "abc".getBytes(StandardCharsets.US_ASCII))));
                } else if (operation == 0x18) {
                    assertTrue(contains(request, item(0x420074, 5, intBytes(1))));
                } else {
                    assertTrue(contains(request, "kmipkit-locate-".getBytes(StandardCharsets.US_ASCII)));
                }
                return outcome;
            }
        }
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey) throws Exception {
        return connectionJson(port, serverCertificate, clientCertificate, clientPrivateKey, false, "typed-protocol");
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey, boolean allowMutations, String apiLevel) throws Exception {
        Path serverCert = temporaryDirectory.resolve("server-cert.pem");
        Path clientCert = temporaryDirectory.resolve("client-cert.pem");
        Path clientKey = temporaryDirectory.resolve("client-key.pem");
        Files.write(serverCert, serverCertificate);
        Files.write(clientCert, clientCertificate);
        Files.write(clientKey, clientPrivateKey);
        ObjectNode root = new ObjectMapper().createObjectNode();
        root.put("server_id", "controlled-test-peer");
        root.put("server_configuration_fingerprint", "a".repeat(64));
        root.put("protocol_version", "2.1");
        root.put("transport", "raw_tls");
        root.put("encoding", "TTLV");
        root.put("api_level", apiLevel);
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
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
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

    private static List<byte[]> serveCreateCleanup(SSLServerSocket listener, int returnedLength) throws Exception {
        var requests = new ArrayList<byte[]>();
        int[] operations = returnedLength != 0 ? new int[] {1, 0x0b, 0x14} : new int[] {1, 0x14};
        for (int exchange = 0; exchange < operations.length; exchange++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readTtlv(socket.getInputStream());
                requests.add(request);
                byte[] operation = new byte[] {0x42, 0x00, 0x5c, 5, 0, 0, 0, 4,
                        0, 0, 0, (byte) operations[exchange]};
                if (!contains(request, operation)) {
                    throw new AssertionError("The controlled create recipe sent an unexpected operation.");
                }
                byte[] response = switch (operations[exchange]) {
                    case 1 -> createResponse();
                    case 0x0b -> lengthResponse(returnedLength);
                    default -> destroyResponse();
                };
                socket.getOutputStream().write(response);
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static byte[] createResponse() throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
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

    private static byte[] lengthResponse(int returnedLength) throws Exception {
        return structure(0x42007b,
                structure(0x42007a,
                        structure(0x420069, item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1))),
                        item(0x420092, 9, longBytes(1)), item(0x42000d, 2, intBytes(1))),
                structure(0x42000f, item(0x42005c, 5, intBytes(0x0b)), item(0x42007f, 5, intBytes(0)),
                        structure(0x42007c,
                                item(0x420094, 7, CREATED_OBJECT_ID.getBytes(StandardCharsets.UTF_8)),
                                structure(0x420125, item(0x42002a, 2, intBytes(returnedLength))))));
    }

    private static byte[] destroyResponse() throws Exception {
        byte[] protocolVersion = structure(0x420069,
                item(0x42006a, 2, intBytes(2)), item(0x42006b, 2, intBytes(1)));
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
        Path path = Path.of("../../../bindings/shared/fixtures/login-mtls", name);
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
}
