package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.DataOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.net.InetAddress;
import java.net.SocketTimeoutException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.KeyFactory;
import java.security.PrivateKey;
import java.security.SecureRandom;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.security.spec.PKCS8EncodedKeySpec;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Base64;
import java.util.List;
import java.util.HexFormat;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;
import org.kmipkit.extensions.BatchOutcomeKind;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.RegisterRequest;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.RevokeRequest;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.SplitKeyReference;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.UsageAllocationContext;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

/** Exercises the T076 client ABI through the Java facade against a local TLS 1.3 peer. */
final class ClientFacadeTlsIntegrationTest {
    private static final char[] STORE_PASSWORD = "kmipkit-test".toCharArray();
    private static final byte[] CORRELATION = {0, (byte) 0xa1, 0x7f};
    private static final byte[] LIFECYCLE_DESTROY_CORRELATION = {
        0, (byte) 0xd3, 0x51, (byte) 0xff
    };
    private static final byte[] LIFECYCLE_RECOVER_CORRELATION = {
        (byte) 0x80, 0x52, 0, (byte) 0xff
    };
    private static final String RNG_RETRIEVED_SENTINEL = "KMIP_RNG_RETRIEVE_SECRET_SENTINEL";
    private static final int LARGE_RESPONSE_DATA_BYTES = 16 * 1024 * 1024 + 1;

    @TempDir
    Path temporaryDirectory;

    @Test
    void nativeAdapterAcceptsCredentialModesAndHttpsTlsNameOverride() throws Exception {
        Path keyStorePath = temporaryDirectory.resolve("native-adapter.p12");
        createKeyStore(keyStorePath, "native-adapter");
        KeyStore keyStore = loadKeyStore(keyStorePath);
        byte[] certificateDer = certificate(keyStore, "native-adapter").getEncoded();
        byte[] privateKeyDer = ((PrivateKey) keyStore.getKey("native-adapter", STORE_PASSWORD)).getEncoded();
        byte[] certificatePem = pem("CERTIFICATE", certificateDer);
        byte[] privateKeyPem = pem("PRIVATE KEY", privateKeyDer);
        Path certificatePemPath = temporaryDirectory.resolve("native-adapter-cert.pem");
        Path privateKeyPemPath = temporaryDirectory.resolve("native-adapter-key.pem");
        Path trustPemPath = temporaryDirectory.resolve("native-adapter-trust.pem");
        Path certificateDerPath = temporaryDirectory.resolve("native-adapter-cert.der");
        Path privateKeyDerPath = temporaryDirectory.resolve("native-adapter-key.der");
        Path trustDerPath = temporaryDirectory.resolve("native-adapter-trust.der");
        Files.write(certificatePemPath, certificatePem);
        Files.write(privateKeyPemPath, privateKeyPem);
        Files.write(trustPemPath, certificatePem);
        Files.write(certificateDerPath, certificateDer);
        Files.write(privateKeyDerPath, privateKeyDer);
        Files.write(trustDerPath, certificateDer);
        byte[] crlDer = testCrlDer();
        byte[] crlPem = pem("X509 CRL", crlDer);

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry)) {
            List<TransportConfiguration.Builder> configurations = List.of(
                    TransportConfiguration.builder().rawTls("127.0.0.1", 5696)
                            .clientIdentityPem(new byte[][] {certificatePem}, privateKeyPem)
                            .trustCertificatesPem(new byte[][] {certificatePem}),
                    TransportConfiguration.builder().rawTls("127.0.0.1", 5696)
                            .clientIdentityDer(new byte[][] {certificateDer}, privateKeyDer)
                            .trustCertificatesDer(new byte[][] {certificateDer}),
                    TransportConfiguration.builder().rawTls("127.0.0.1", 5696)
                            .clientIdentityFiles(List.of(certificatePemPath), privateKeyPemPath,
                                    TransportConfiguration.Encoding.PEM)
                            .trustCertificateFiles(List.of(trustPemPath),
                                    TransportConfiguration.Encoding.PEM),
                    TransportConfiguration.builder().rawTls("127.0.0.1", 5696)
                            .clientIdentityFiles(List.of(certificateDerPath), privateKeyDerPath,
                                    TransportConfiguration.Encoding.DER)
                            .trustCertificateFiles(List.of(trustDerPath),
                                    TransportConfiguration.Encoding.DER),
                    TransportConfiguration.builder().https("https://127.0.0.1:5696/kmip")
                            .clientIdentityPem(new byte[][] {certificatePem}, privateKeyPem)
                            .trustCertificatesPem(new byte[][] {certificatePem})
                            .tlsServerName("localhost"),
                    TransportConfiguration.builder().rawTls("127.0.0.1", 5696)
                            .clientIdentity(
                                    List.of(TransportConfiguration.TransportInput.memory(
                                            certificatePem, TransportConfiguration.Encoding.PEM)),
                                    TransportConfiguration.TransportInput.file(
                                            privateKeyDerPath, TransportConfiguration.Encoding.DER))
                            .trustCertificates(List.of(
                                    TransportConfiguration.TransportInput.file(
                                            trustPemPath, TransportConfiguration.Encoding.PEM),
                                    TransportConfiguration.TransportInput.memory(
                                            certificateDer, TransportConfiguration.Encoding.DER)))
                            .revocationLists(List.of(
                                    TransportConfiguration.TransportInput.memory(
                                            crlPem, TransportConfiguration.Encoding.PEM),
                                    TransportConfiguration.TransportInput.memory(
                                            crlDer, TransportConfiguration.Encoding.DER))));

            for (TransportConfiguration.Builder builder : configurations) {
                try (TransportConfiguration transport = builder.build();
                        Client client = Client.create(extensions, transport)) {
                    assertNotNull(client);
                }
            }
        }
    }

    @Test
    void pingQueryAndLogoutRoundTripThroughNativeJniOverDeterministicTls13Peer() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate serverCertificate = testCertificate(serverCertificatePem);
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);

        KeyStore serverStore = KeyStore.getInstance("PKCS12");
        serverStore.load(null, STORE_PASSWORD);
        serverStore.setKeyEntry(
                "kmipkit-test-server",
                testPrivateKey(serverPrivateKeyPem),
                STORE_PASSWORD,
                new Certificate[] {serverCertificate});
        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, pingQueryLogoutResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-ping-query-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        byte[] pingId = "ping-fixture".getBytes(StandardCharsets.US_ASCII);
        byte[] queryId = "query-fixture".getBytes(StandardCharsets.US_ASCII);
        byte[] logoutId = "logout-fixture".getBytes(StandardCharsets.US_ASCII);
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.ping().withUniqueBatchItemId(pingId));
            batch.add(ClientBatchItem.query(
                            new int[] {1, 2, 1}, new String[] {"group-b", "group-a"})
                    .withUniqueBatchItemId(queryId));
            try (TtlvStructure ticket = logoutTicket()) {
                batch.add(ClientBatchItem.logout(ticket).withUniqueBatchItemId(logoutId));
            }
            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult pingResult = response.itemAt(0);
                    ClientBatchItemResult queryResult = response.itemAt(1);
                    ClientBatchItemResult logoutResult = response.itemAt(2)) {
                assertEquals(3, response.itemCount());
                assertEquals(0x3b, pingResult.operation());
                assertEquals(Boolean.TRUE, pingResult.pingResponse().orElseThrow());
                assertEquals(0x18, queryResult.operation());
                assertEquals(0x2e, logoutResult.operation());
                assertEquals(Boolean.TRUE, logoutResult.logoutResponse().orElseThrow());
                try (TtlvStructure queryPayload = queryResult.queryResponse().orElseThrow();
                        TtlvStructureView payloadView = queryPayload.view();
                        TtlvItemView masksItem = payloadView.itemAt(0);
                        Tag masksTag = masksItem.tag()) {
                    assertEquals(1, payloadView.itemCount());
                    assertEquals(0x42015f, masksTag.raw());
                    assertEquals(TtlvItemType.Structure, masksItem.itemType());
                    try (TtlvValueView masksValue = masksItem.value();
                            TtlvStructureView masks = masksValue.structure()) {
                        assertEquals(0, masks.itemCount());
                    }
                }
            }
            request = peer.get(15, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        assertContains(request, enumeration(0x42005c, 0x3b));
        assertContains(request, enumeration(0x42005c, 0x18));
        assertContains(request, item(0x420093, 8, pingId));
        assertContains(request, item(0x420093, 8, queryId));
        assertContains(request, enumeration(0x42005c, 0x2e));
        assertContains(request, item(0x420093, 8, logoutId));
        assertContains(request, enumeration(0x420074, 1));
        assertContains(request, enumeration(0x420074, 2));
        assertContains(request, item(0x420056, 7, "group-b".getBytes(StandardCharsets.UTF_8)));
        assertContains(request, item(0x420056, 7, "group-a".getBytes(StandardCharsets.UTF_8)));
        assertTrue(peer.isDone());
    }

    @ParameterizedTest
    @ValueSource(strings = {"nonempty", "absent", "presentEmpty"})
    void pkcs11ResponseMatchesSharedFixtureThroughNativeJniOverMutualTls(
            String correlationCase) throws Exception {
        String fixture = Files.readString(pkcs11ParityFixturePath(), StandardCharsets.UTF_8);
        List<Long> expectedTags = new ArrayList<>(fixtureLongArray(fixture, "responseTags"));
        String expectedOutput = fixtureObject(fixture, "expectedOutput");
        String typedResponse = fixtureObject(expectedOutput, "typedResponse");
        String expectedFunction = fixtureObject(typedResponse, "function");
        String expectedCorrelation = fixtureObject(typedResponse, "correlationValue");
        String expectedOutputParameters = fixtureObject(typedResponse, "outputParameters");
        String expectedPkcs11ReturnCode = fixtureObject(typedResponse, "returnCode");
        if (correlationCase.equals("absent")) {
            expectedTags.remove(fixtureHexField(expectedCorrelation, "tag"));
        }
        String vendorExtension = fixtureObject(expectedOutput, "vendorExtension");
        long expectedReturnCode = fixtureHexField(expectedPkcs11ReturnCode, "childValue");
        long expectedVendorTag = fixtureHexField(vendorExtension, "tag");
        int expectedVendorValue = Math.toIntExact(fixtureLongField(vendorExtension, "integer"));
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        byte[] responseMessage = pkcs11ResponseMessage(fixture, correlationCase);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, responseMessage));
        Thread peerThread = new Thread(peer, "kmipkit-java-pkcs11-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        byte[] correlation = "pkcs11-correlation".getBytes(StandardCharsets.US_ASCII);
        byte[] request;
        ClientBatchItemResult result;
        ClientBatchResponse batchResponse;
        CodecLimits limits = CodecLimits.defaults();
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create();
                TtlvStructure function = pkcs11Structure(0x42015a, TtlvValue.Integer(-12345), limits);
                TtlvStructure interfaceStructure = pkcs11Structure(0x420159,
                        TtlvValue.TextString("test-interface".getBytes(StandardCharsets.US_ASCII), limits),
                        limits);
                TtlvStructure inputParameters = pkcs11Structure(0x42015b,
                        TtlvValue.ByteString("opaque-input".getBytes(StandardCharsets.US_ASCII), limits),
                        limits)) {
            try (ClientBatchItem item = ClientBatchItem.pkcs11(
                    function, interfaceStructure, correlation, inputParameters)) {
                batch.add(item);
            }
            batchResponse = client.execute(batch, limits);
            assertEquals(1, batchResponse.itemCount());
            result = batchResponse.itemAt(0);
            assertEquals(0x33, result.operation());
            assertEquals(0, result.resultStatus());
            request = peer.get(15, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        byte[] interfaceField = structure(0x420159,
                item(0x420159, 7, "test-interface".getBytes(StandardCharsets.US_ASCII)));
        byte[] functionField = structure(0x42015a,
                item(0x42015a, 2, integer(-12345)));
        byte[] correlationField = item(0x4200d6, 8, correlation);
        byte[] inputField = structure(0x42015b,
                item(0x42015b, 8, "opaque-input".getBytes(StandardCharsets.US_ASCII)));
        assertEncodedInOrder(request, interfaceField, functionField, correlationField, inputField);

        ClientBatchItemResult.Pkcs11Response pkcs11Response = result.pkcs11Response().orElseThrow();
        result.close();
        batchResponse.close();
        assertTrue(pkcs11Response.interfaceTtlv().isEmpty(), "absent Interface stays absent");
        if (correlationCase.equals("absent")) {
            assertTrue(pkcs11Response.correlationValue().isEmpty());
        } else if (correlationCase.equals("presentEmpty")) {
            assertArrayEquals(new byte[0], pkcs11Response.correlationValue().orElseThrow());
        } else {
            assertArrayEquals(fixtureHexBytes(expectedCorrelation, "hex"),
                    pkcs11Response.correlationValue().orElseThrow());
        }
        try (TtlvStructure functionTtlv = pkcs11Response.functionTtlv().orElseThrow();
                TtlvStructureView functionView = functionTtlv.view();
                TtlvItemView functionItem = functionView.itemAt(0);
                TtlvValueView functionValue = functionItem.value()) {
            assertEquals(fixtureHexField(expectedFunction, "childTag"), tagAt(functionView, 0));
            assertEquals(Math.toIntExact(fixtureLongField(expectedFunction, "childValue")),
                    functionValue.integer());
        }
        try (TtlvStructure outputTtlv = pkcs11Response.outputParametersTtlv().orElseThrow();
                TtlvStructureView outputView = outputTtlv.view()) {
            assertBytesAt(outputView, 0, fixtureHexField(expectedOutputParameters, "childTag"),
                    fixtureHexBytes(expectedOutputParameters, "childHex"));
        }
        try (TtlvStructure returnTtlv = pkcs11Response.returnCodeTtlv().orElseThrow();
                TtlvStructureView returnView = returnTtlv.view();
                TtlvItemView codeItem = returnView.itemAt(0);
                TtlvValueView codeValue = codeItem.value()) {
            assertEquals(fixtureHexField(expectedPkcs11ReturnCode, "childTag"),
                    tagAt(returnView, 0));
            assertEquals(expectedReturnCode, codeValue.enumeration());
        }
        try (pkcs11Response;
                TtlvStructureView responseView = pkcs11Response.responseTtlv().view();
                TtlvItemView payloadItem = responseView.itemAt(2);
                TtlvValueView payloadValue = payloadItem.value();
                TtlvStructureView payloadView = payloadValue.structure()) {
            assertTagOrder(responseView, 0x42005c, 0x42007f, 0x42007c);
            assertEquals(expectedTags.size(), payloadView.itemCount());
            assertTagOrder(payloadView, expectedTags.stream().mapToLong(Long::longValue).toArray());
            assertFalse(expectedTags.contains(0x420159L), "absent Interface remains absent");
            assertFalse(expectedTags.contains(0x42015bL), "absent Input Parameters remain absent");
            int fieldOffset = correlationCase.equals("absent") ? 0 : 1;
            assertNestedInteger(payloadView, 0, fixtureHexField(expectedFunction, "tag"),
                    Math.toIntExact(fixtureLongField(expectedFunction, "childValue")));
            if (!correlationCase.equals("absent")) {
                assertBytesAt(payloadView, 1, fixtureHexField(expectedCorrelation, "tag"),
                        correlationCase.equals("presentEmpty") ? new byte[0]
                                : fixtureHexBytes(expectedCorrelation, "hex"));
            }
            assertNestedBytes(payloadView, 1 + fieldOffset,
                    fixtureHexField(expectedOutputParameters, "tag"),
                    fixtureHexBytes(expectedOutputParameters, "childHex"));
            try (TtlvItemView returnItem = payloadView.itemAt(2 + fieldOffset);
                    TtlvValueView returnValue = returnItem.value();
                    TtlvStructureView returnCode = returnValue.structure();
                    TtlvItemView codeItem = returnCode.itemAt(0);
                    TtlvValueView codeValue = codeItem.value()) {
                assertEquals(fixtureHexField(expectedPkcs11ReturnCode, "childTag"),
                        tagAt(returnCode, 0));
                assertEquals(expectedReturnCode, codeValue.enumeration());
            }
            try (TtlvItemView vendorItem = payloadView.itemAt(3 + fieldOffset);
                    Tag vendorTag = vendorItem.tag();
                    TtlvValueView vendorValue = vendorItem.value()) {
                assertEquals(expectedVendorTag, vendorTag.raw());
                assertEquals(expectedVendorValue, vendorValue.integer());
            }
            assertTrue(pkcs11Response.toString().contains("redacted"));
            assertFalse(pkcs11Response.toString().contains("opaque-input"));
        }
        assertTrue(peer.isDone(), "the peer handles exactly one exchange");
    }

    @Test
    void largeCryptoResponsesSurviveSourceCloseAndLargeQueryUsesElevatedElementLimit() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);

        byte[] expectedData = largeResponseData();
        byte[] encryptResponse = largeCryptoResponseMessage(0x1f, expectedData);
        byte[] decryptResponse = largeCryptoResponseMessage(0x20, expectedData);
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> List.of(
                serveOneResponse(listener, encryptResponse),
                serveOneResponse(listener, decryptResponse),
                serveOneResponse(listener, largeQueryResponseMessage())));
        Thread peerThread = new Thread(peer, "kmipkit-java-large-crypto-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        byte[] encryptData;
        byte[] decryptData;
        byte[] queryRequest;
        CodecLimits limits = CodecLimits.create(LARGE_RESPONSE_DATA_BYTES + 1024L, 64, 100_000);
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(10)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(10)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(30)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(45)))
                        .build();
                Client client = Client.create(extensions, transport)) {
            encryptData = executeCryptoDataAfterSourceClose(client, 0x1f, limits);
            decryptData = executeCryptoDataAfterSourceClose(client, 0x20, limits);

            int[] functions = new int[100_001];
            String[] objectGroups = new String[100_001];
            for (int index = 0; index < functions.length; index++) {
                functions[index] = (index & 1) + 1;
                objectGroups[index] = "group-" + index;
            }
            CodecLimits queryLimits = CodecLimits.create(16 * 1024 * 1024L, 64, 400_000);
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.queryWithLimits(functions, objectGroups, queryLimits)
                        .withUniqueBatchItemId("large-query".getBytes(StandardCharsets.US_ASCII)));
                try (ClientBatchResponse response = client.execute(batch, queryLimits);
                        ClientBatchItemResult result = response.itemAt(0);
                        TtlvStructure queryPayload = result.queryResponse().orElseThrow()) {
                    assertEquals(0x18, result.operation());
                    assertNotNull(queryPayload);
                }
            }

            List<byte[]> requests = peer.get(60, TimeUnit.SECONDS);
            assertEquals(3, requests.size());
            queryRequest = requests.get(2);
        } finally {
            listener.close();
        }

        assertArrayEquals(expectedData, encryptData);
        assertArrayEquals(expectedData, decryptData);
        assertTrue(queryRequest.length > 3_000_000,
                "the TLS peer must receive the encoded 100,001-function and 100,001-group Query");
        assertContains(queryRequest, enumeration(0x420074, 1));
        assertContains(queryRequest, item(0x420056, 7,
                "group-100000".getBytes(StandardCharsets.UTF_8)));
    }

    @Test
    void executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange() throws Exception {
        String fixture = Files.readString(SharedCapabilityParityFixtureTest.sharedFixturePath());
        String batchFixture = SharedCapabilityParityFixtureTest.caseById(fixture,
                "batch.ordered-completed-pending-failed.v1");
        String discoverVersionsCase = SharedCapabilityParityFixtureTest.caseById(fixture,
                "discover-versions.supported-versions.v1");
        String discoverVersionsExpected = SharedCapabilityParityFixtureTest.objectField(
                discoverVersionsCase, "expectedOutput");
        List<ProtocolVersion> expectedSupportedVersions = SharedCapabilityParityFixtureTest
                .objectArrayFields(discoverVersionsExpected, "supportedVersions").stream()
                .map(version -> new ProtocolVersion(
                        Math.toIntExact(SharedCapabilityParityFixtureTest.numberField(version, "major")),
                        Math.toIntExact(SharedCapabilityParityFixtureTest.numberField(version, "minor"))))
                .toList();
        List<ProtocolVersion> expectedEmptyVersions = SharedCapabilityParityFixtureTest
                .objectArrayFields(discoverVersionsExpected, "emptySuccessfulResponseVersions").stream()
                .map(version -> new ProtocolVersion(
                        Math.toIntExact(SharedCapabilityParityFixtureTest.numberField(version, "major")),
                        Math.toIntExact(SharedCapabilityParityFixtureTest.numberField(version, "minor"))))
                .toList();
        assertTrue(Boolean.parseBoolean(SharedCapabilityParityFixtureTest.stringOrBooleanField(
                discoverVersionsExpected, "preservesSourceOrderAndRepetitions")));
        assertFalse(Boolean.parseBoolean(SharedCapabilityParityFixtureTest.stringOrBooleanField(
                discoverVersionsExpected, "automaticFollowUp")));
        String fixtureInput = SharedCapabilityParityFixtureTest.objectField(batchFixture, "input");
        String fixtureExpected = SharedCapabilityParityFixtureTest.objectField(batchFixture, "expectedOutput");
        List<String> requestIdsHex = SharedCapabilityParityFixtureTest.stringArrayField(
                fixtureInput, "uniqueBatchItemIds");
        List<byte[]> requestIds = requestIdsHex.stream().map(java.util.HexFormat.of()::parseHex).toList();
        List<String> expectedStatuses = SharedCapabilityParityFixtureTest.stringArrayField(
                fixtureExpected, "resultStatuses");
        List<String> expectedIdsHex = SharedCapabilityParityFixtureTest.stringArrayField(
                fixtureExpected, "uniqueBatchItemIds");
        List<byte[]> expectedIds = expectedIdsHex.stream().map(java.util.HexFormat.of()::parseHex).toList();
        byte[] expectedCorrelation = java.util.HexFormat.of().parseHex(
                SharedCapabilityParityFixtureTest.stringField(fixtureExpected, "correlationValueHex"));

        Path serverStorePath = temporaryDirectory.resolve("server.p12");
        Path clientStorePath = temporaryDirectory.resolve("client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener,
                        mixedResponseMessage(requestIds, expectedCorrelation, expectedSupportedVersions)));
        Thread peerThread = new Thread(peer, "kmipkit-java-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                TtlvStructure createAttributes = TtlvStructure.create();
                TtlvStructure commonAttributes = TtlvStructure.create();
                TtlvStructure privateAttributes = TtlvStructure.create();
                TtlvStructure publicAttributes = TtlvStructure.create();
                TtlvStructure splitAttributes = TtlvStructure.create();
                TtlvStructure delegatedLoginRights = TtlvStructure.create();
                TtlvStructure derivationParameters = TtlvStructure.create();
                TtlvStructure derivedAttributes = deriveKeyAttributes();
                TtlvStructure importAttributes = TtlvStructure.create();
                TtlvStructure importAnyObject = importAnyObjectStructure()) {
            assertThrows(InvalidInputException.class,
                    () -> client.executeWithOptions(null, null, null));
            assertNativeRequestOptionsRejected(client, new long[11]);
            long[] invalidTimeoutKind = new long[12];
            invalidTimeoutKind[0] = 3;
            assertNativeRequestOptionsRejected(client, invalidTimeoutKind);
            long[] invalidTimeoutNanoseconds = new long[12];
            invalidTimeoutNanoseconds[2] = 1_000_000_000L;
            assertNativeRequestOptionsRejected(client, invalidTimeoutNanoseconds);
            long[] unboundedTimeoutWithDuration = new long[12];
            unboundedTimeoutWithDuration[0] = 2;
            unboundedTimeoutWithDuration[1] = 1;
            assertNativeRequestOptionsRejected(client, unboundedTimeoutWithDuration);

            ClientBatch batch = ClientBatch.create();
            batch.asynchronousIndicator(2)
                    .add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(requestIds.get(0)))
                    .add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(requestIds.get(1)))
                    .add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(requestIds.get(2)))
                    .add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(uniqueBatchItemId(3)))
                    .add(ClientBatchItem.create(7, createAttributes, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(4)))
                    .add(ClientBatchItem.create(7, createAttributes, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(5)))
                    .add(ClientBatchItem.createKeyPair(commonAttributes, privateAttributes,
                                    publicAttributes, null, null, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(6)))
                    .add(ClientBatchItem.createSplitKey(7, UniqueIdentifier.integer(-7), -3, 2,
                                    3, new byte[] {0, 0, 0, 0, 0, 0, 0, 17}, splitAttributes, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(7)))
                    .add(ClientBatchItem.create(7, createAttributes, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(8)))
                    .add(ClientBatchItem.certify(UniqueIdentifier.textString("public-key"),
                                    1L, null, null, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(9)))
                    .add(ClientBatchItem.getUsageAllocation(
                                    UniqueIdentifier.textString("allocation-request"), 3L)
                            .withUniqueBatchItemId(uniqueBatchItemId(10)))
                    .add(ClientBatchItem.check(UniqueIdentifier.textString("check-key"),
                                    3L, 2, 60L)
                            .withUniqueBatchItemId(uniqueBatchItemId(11)))
                    .add(ClientBatchItem.delegatedLogin(delegatedLoginRights,
                                    60L, null, 3, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(12)))
                    .add(ClientBatchItem.deriveKey(7,
                                    List.of(UniqueIdentifier.textString("source-key"),
                                            UniqueIdentifier.enumeration(0xfedc_ba98L),
                                            UniqueIdentifier.integer(-17)),
                                    0xfeed_beefL, derivationParameters, derivedAttributes)
                            .withUniqueBatchItemId(uniqueBatchItemId(13)))
                    .add(ClientBatchItem.export(UniqueIdentifier.textString("export-source"),
                                    0xfedc_ba98L, 0xfedc_ba99L, 0xfedc_ba9aL, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(14)))
                    .add(ClientBatchItem.export(null, null, null, null, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(15)))
                    .add(ClientBatchItem.getConstraints()
                            .withUniqueBatchItemId(uniqueBatchItemId(16)))
                    .add(ClientBatchItem.getConstraints()
                            .withUniqueBatchItemId(uniqueBatchItemId(17)))
                    .add(ClientBatchItem.get(null, null, null, null, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(18)))
                    .add(ClientBatchItem.locate(null, null, null, null, createAttributes)
                            .withUniqueBatchItemId(uniqueBatchItemId(19)))
                    .add(ClientBatchItem.get(UniqueIdentifier.textString("failed-get-request"),
                                    null, null, null, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(20)))
                    .add(ClientBatchItem.locate(null, null, null, null, createAttributes)
                            .withUniqueBatchItemId(uniqueBatchItemId(21)))
                    .add(ClientBatchItem.importObject(UniqueIdentifier.textString("import-source"),
                                    7, Boolean.FALSE, null, importAttributes, importAnyObject)
                            .withUniqueBatchItemId(uniqueBatchItemId(22)))
                    .add(ClientBatchItem.importObject(UniqueIdentifier.textString("import-source"),
                                    7, Boolean.FALSE, null, importAttributes, importAnyObject)
                            .withUniqueBatchItemId(uniqueBatchItemId(23)))
                    .add(ClientBatchItem.interop(0xf1234567L,
                                    "interop-integration-test".getBytes(StandardCharsets.UTF_8))
                            .withUniqueBatchItemId(uniqueBatchItemId(24)))
                    .add(ClientBatchItem.log(
                                    "log-message-private-sentinel".getBytes(StandardCharsets.UTF_8))
                            .withUniqueBatchItemId(uniqueBatchItemId(25)))
                    .add(ClientBatchItem.login(null, null, null, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(26)));
            ClientBatchResponse batchResponse;
            try {
                batchResponse = client.executeWithOptions(
                        batch, CodecLimits.defaults(), RequestOptions.defaults());
            } catch (ClientOperationException exception) {
                throw new AssertionError(
                        "batch execution failed with native status=" + exception.statusCode()
                                + ", category=" + exception.categoryCode()
                                + ", cause=" + exception.causeCode()
                                + ", delivery=" + exception.deliveryStateCode(),
                        exception);
            }
            try (ClientBatchResponse response = batchResponse;
                    ClientBatchItemResult completed = response.itemAt(0);
                    ClientBatchItemResult pending = response.itemAt(1);
                    ClientBatchItemResult failed = response.itemAt(2);
                    ClientBatchItemResult empty = response.itemAt(3);
                    ClientBatchItemResult created = response.itemAt(4);
                    ClientBatchItemResult creationFailed = response.itemAt(5);
                    ClientBatchItemResult keyPair = response.itemAt(6);
                    ClientBatchItemResult splitKey = response.itemAt(7);
                    ClientBatchItemResult textCreated = response.itemAt(8);
                    ClientBatchItemResult certified = response.itemAt(9);
                    ClientBatchItemResult allocated = response.itemAt(10);
                    ClientBatchItemResult checked = response.itemAt(11);
                    ClientBatchItemResult delegated = response.itemAt(12);
                    ClientBatchItemResult derived = response.itemAt(13);
                    ClientBatchItemResult exported = response.itemAt(14);
                    ClientBatchItemResult exportFailed = response.itemAt(15);
                    ClientBatchItemResult constraints = response.itemAt(16);
                    ClientBatchItemResult constraintsFailed = response.itemAt(17);
                    ClientBatchItemResult getMapped = response.itemAt(18);
                    ClientBatchItemResult locateMapped = response.itemAt(19);
                    ClientBatchItemResult getFailed = response.itemAt(20);
                    ClientBatchItemResult locateFailed = response.itemAt(21);
                    ClientBatchItemResult imported = response.itemAt(22);
                    ClientBatchItemResult importFailed = response.itemAt(23);
                    ClientBatchItemResult interop = response.itemAt(24);
                    ClientBatchItemResult log = response.itemAt(25);
                    ClientBatchItemResult login = response.itemAt(26)) {
                assertEquals(27, response.itemCount());
                assertTrue(completed.getResponse().isEmpty());
                assertTrue(completed.locateResponse().isEmpty());
                assertEquals(resultStatusCode(expectedStatuses.get(0)), completed.resultStatus());
                assertEquals("Success".equals(expectedStatuses.get(0)),
                        completed.outcomeKind().isCompleted());
                assertArrayEquals(expectedIds.get(0), completed.uniqueBatchItemId().orElseThrow());
                assertTrue(completed.pendingCorrelationValue().isEmpty());
                assertTrue(completed.resultReason().isEmpty());
                assertEquals(expectedSupportedVersions, completed.supportedVersions().orElseThrow());
                assertEquals(resultStatusCode(expectedStatuses.get(1)), pending.resultStatus());
                assertEquals("Operation Pending".equals(expectedStatuses.get(1)),
                        pending.outcomeKind().isPending());
                assertArrayEquals(expectedIds.get(1), pending.uniqueBatchItemId().orElseThrow());
                assertArrayEquals(expectedCorrelation, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.resultReason().isEmpty());
                assertEquals(Boolean.parseBoolean(SharedCapabilityParityFixtureTest.stringOrBooleanField(
                        discoverVersionsExpected, "pendingAndFailureHaveNoSupportedVersions")),
                        pending.supportedVersions().isEmpty());
                assertEquals(resultStatusCode(expectedStatuses.get(2)), failed.resultStatus());
                assertEquals("Operation Failed".equals(expectedStatuses.get(2)),
                        failed.outcomeKind().isCompleted());
                assertArrayEquals(expectedIds.get(2), failed.uniqueBatchItemId().orElseThrow());
                assertTrue(failed.pendingCorrelationValue().isEmpty());
                assertEquals(1, failed.resultReason().orElseThrow());
                assertEquals(Boolean.parseBoolean(SharedCapabilityParityFixtureTest.stringOrBooleanField(
                        discoverVersionsExpected, "pendingAndFailureHaveNoSupportedVersions")),
                        failed.supportedVersions().isEmpty());
                long pendingCount = List.of(completed, pending, failed).stream()
                        .filter(result -> result.outcomeKind().isPending()).count();
                assertEquals(SharedCapabilityParityFixtureTest.numberField(fixtureExpected, "pendingCount"),
                        pendingCount);
                assertEquals(0, empty.resultStatus());
                assertEquals(Boolean.parseBoolean(SharedCapabilityParityFixtureTest.stringOrBooleanField(
                        discoverVersionsExpected, "emptySuccessfulResponseIsPresent")),
                        empty.supportedVersions().isPresent());
                assertEquals(expectedEmptyVersions, empty.supportedVersions().orElseThrow());

                assertEquals(1, created.operation());
                ClientBatchItemResult.CreateResponse createResponse =
                        created.createResponse().orElseThrow();
                assertEquals(7, createResponse.objectType());
                assertEquals(UniqueIdentifier.Kind.INTEGER, createResponse.uniqueIdentifier().kind());
                assertEquals(-8, createResponse.uniqueIdentifier().number());
                assertTrue(created.resultMessage().isEmpty());

                assertEquals(1, creationFailed.resultStatus());
                assertEquals(5, creationFailed.resultReason().orElseThrow());
                assertEquals(1, creationFailed.operation());
                assertEquals(java.util.Optional.of("création refusée ✓"), creationFailed.resultMessage());
                assertTrue(creationFailed.createResponse().isEmpty());
                assertTrue(creationFailed.createKeyPairResponse().isEmpty());
                assertTrue(creationFailed.createSplitKeyUniqueIdentifiers().isEmpty());
                assertFalse(creationFailed.metadataV2().toString().contains("création refusée"));

                ClientBatchItemResult.CreateKeyPairResponse keyPairResponse =
                        keyPair.createKeyPairResponse().orElseThrow();
                assertEquals(UniqueIdentifier.Kind.ENUMERATION,
                        keyPairResponse.privateKeyUniqueIdentifier().kind());
                assertEquals(0xffff_ffffL, keyPairResponse.privateKeyUniqueIdentifier().number());
                assertEquals(UniqueIdentifier.Kind.TEXT_STRING,
                        keyPairResponse.publicKeyUniqueIdentifier().kind());
                assertEquals("public-key-α", keyPairResponse.publicKeyUniqueIdentifier().text());
                assertFalse(keyPairResponse.toString().contains("public-key-α"));

                List<UniqueIdentifier> splitIdentifiers =
                        splitKey.createSplitKeyUniqueIdentifiers().orElseThrow();
                assertEquals(4, splitIdentifiers.size());
                assertEquals(UniqueIdentifier.Kind.TEXT_STRING, splitIdentifiers.get(0).kind());
                assertEquals("part-a", splitIdentifiers.get(0).text());
                assertEquals(UniqueIdentifier.Kind.ENUMERATION, splitIdentifiers.get(1).kind());
                assertEquals(0xffff_ffffL, splitIdentifiers.get(1).number());
                assertEquals(UniqueIdentifier.Kind.INTEGER, splitIdentifiers.get(2).kind());
                assertEquals(-7, splitIdentifiers.get(2).number());
                assertEquals("part-a", splitIdentifiers.get(3).text());

                ClientBatchItemResult.CreateResponse textCreateResponse =
                        textCreated.createResponse().orElseThrow();
                assertEquals(UniqueIdentifier.Kind.TEXT_STRING, textCreateResponse.uniqueIdentifier().kind());
                assertEquals("create-text-id-secret", textCreateResponse.uniqueIdentifier().text());
                assertFalse(textCreateResponse.toString().contains("create-text-id-secret"));

                try (ClientBatchItemResult.CertifyResponse certifyResponse =
                                certified.certifyResponse().orElseThrow()) {
                    certified.close();
                    try (TtlvStructureView certifyTtlv = certifyResponse.responseTtlv().view()) {
                        assertEquals(UniqueIdentifier.Kind.TEXT_STRING,
                                certifyResponse.uniqueIdentifier().kind());
                        assertEquals("certify-text-id-secret", certifyResponse.uniqueIdentifier().text());
                        assertTrue(certifyTtlv.itemCount() > 0);
                        assertTrue(containsByteString(certifyTtlv, 0x540001,
                                "KMIPKit-certify-sentinel".getBytes(StandardCharsets.US_ASCII)));
                        assertFalse(certifyResponse.toString().contains("certify-text-id-secret"));
                    }
                }

                try (ClientBatchItemResult.GetUsageAllocationResponse allocationResponse =
                                allocated.getUsageAllocationResponse().orElseThrow();
                        TtlvStructureView allocationTtlv = allocationResponse.responseTtlv().view()) {
                    assertEquals(0, allocated.resultStatus());
                    assertEquals(UniqueIdentifier.Kind.TEXT_STRING,
                            allocationResponse.uniqueIdentifier().kind());
                    assertEquals("allocation-object", allocationResponse.uniqueIdentifier().text());
                    assertTrue(containsByteString(allocationTtlv, 0x540001,
                            "KMIPKit-allocation-sentinel".getBytes(StandardCharsets.US_ASCII)));
                    assertFalse(allocationResponse.toString().contains("allocation-object"));
                }

                try (UsageAllocationContext allocationContext =
                                allocated.createUsageAllocationContext(5, 0xfedc_ba98L)) {
                    assertEquals(0xfedc_ba98L, allocationContext.usageLimitsUnit());
                    assertEquals(5, allocationContext.allocatedUnits());
                    assertEquals(5, allocationContext.availableUnits());
                    assertEquals(0, allocationContext.reservedUnits());
                    assertEquals(UniqueIdentifier.Kind.TEXT_STRING,
                            allocationContext.uniqueIdentifier().kind());
                    assertEquals("allocation-object", allocationContext.uniqueIdentifier().text());
                    assertFalse(allocationContext.toString().contains("allocation-object"));
                }
                assertThrows(InvalidInputException.class,
                        () -> allocated.createUsageAllocationContext(0, 1));
                assertThrows(InvalidInputException.class,
                        () -> creationFailed.createUsageAllocationContext(5, 1));

                try (ClientBatchItemResult.CheckResponse checkResponse =
                                checked.checkResponse().orElseThrow();
                        TtlvStructureView checkTtlv = checkResponse.responseTtlv().view()) {
                    assertTrue(checkResponse.uniqueIdentifier().isEmpty());
                    assertEquals(3L, checkResponse.usageLimitsCount().orElseThrow());
                    assertEquals(2L, checkResponse.cryptographicUsageMask().orElseThrow());
                    assertEquals(60L, checkResponse.leaseTime().orElseThrow());
                    assertTrue(checkTtlv.itemCount() > 0);
                    assertFalse(checkResponse.toString().contains("check-key"));
                }

                try (ClientBatchItemResult.DelegatedLoginResponse delegatedResponse =
                                delegated.delegatedLoginResponse().orElseThrow()) {
                    assertEquals(0, delegated.resultStatus());
                    assertEquals(0xfeed_beefL,
                            delegatedResponse.ticketTypeRaw().orElseThrow());
                    assertArrayEquals("delegated-ticket-secret"
                                    .getBytes(StandardCharsets.US_ASCII),
                            delegatedResponse.ticketValue().orElseThrow());
                    assertTrue(delegatedResponse.ticketTtlv().isPresent());
                    assertFalse(delegatedResponse.toString().contains("delegated-ticket-secret"));
                }
                ClientBatchItemResult.LoginResponse loginResponse =
                        login.loginResponse().orElseThrow();
                TtlvStructure loginTicket = loginResponse.ticketTtlv().orElseThrow();
                try (loginResponse) {
                    assertEquals(0, login.resultStatus());
                    assertEquals(0xfeed_beefL, loginResponse.ticketTypeRaw().orElseThrow());
                    byte[] callerTicket = loginResponse.ticketValue().orElseThrow();
                    assertArrayEquals("login-ticket-native-secret"
                                    .getBytes(StandardCharsets.US_ASCII), callerTicket);
                    assertFalse(loginResponse.toString().contains("login-ticket-native-secret"));
                    try (TtlvStructureView ticketView = loginTicket.view()) {
                        assertTagOrder(ticketView, 0x0042_014a, 0x0042_014b);
                    }
                    Arrays.fill(callerTicket, (byte) 0);
                }
                assertTrue(loginResponse.ticketValue().isEmpty());
                assertTrue(loginResponse.ticketTtlv().isEmpty());
                try (ClientBatchItemResult.DeriveKeyResponse deriveKeyResponse =
                                derived.deriveKeyResponse().orElseThrow();
                        TtlvStructureView deriveKeyTtlv = deriveKeyResponse.responseTtlv().view()) {
                    assertEquals(0, derived.resultStatus());
                    assertEquals(UniqueIdentifier.Kind.TEXT_STRING,
                            deriveKeyResponse.uniqueIdentifier().kind());
                    assertEquals("derived-key", deriveKeyResponse.uniqueIdentifier().text());
                    assertTagOrder(deriveKeyTtlv,
                            0x0042_005c, 0x0042_0093, 0x0042_007f, 0x0042_007c);
                    try (TtlvItemView responsePayload = deriveKeyTtlv.itemAt(3);
                            TtlvValueView responsePayloadValue = responsePayload.value();
                            TtlvStructureView payload = responsePayloadValue.structure()) {
                        assertTagOrder(payload, 0x0042_0094, 0x0054_0001);
                    }
                    assertFalse(deriveKeyResponse.toString().contains("derived-key"));
                }
                assertEquals(0, imported.resultStatus());
                ClientBatchItemResult.ImportResponse importResponse =
                        imported.importResponse().orElseThrow();
                imported.close();
                try (importResponse;
                        TtlvStructureView importTtlv = importResponse.responseTtlv().view()) {
                    assertEquals("imported-object-secret-id",
                            importResponse.uniqueIdentifier().text());
                    assertTagOrder(importTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
                    try (TtlvItemView payload = importTtlv.itemAt(3);
                            TtlvValueView payloadValue = payload.value();
                            TtlvStructureView payloadView = payloadValue.structure()) {
                        assertTagOrder(payloadView, 0x420094, 0x540001);
                        assertTrue(containsByteString(payloadView, 0x540001,
                                "KMIPKit-import-sentinel".getBytes(StandardCharsets.US_ASCII)));
                    }
                    assertFalse(importResponse.toString().contains("imported-object-secret-id"));
                }
                assertEquals(1, importFailed.resultStatus());
                assertTrue(importFailed.importResponse().isEmpty());
                try (ClientBatchItemResult.InteropResponse interopResponse =
                                interop.interopResponse().orElseThrow();
                        TtlvStructureView interopTtlv = interopResponse.responseTtlv().view()) {
                    assertEquals(0x34, interop.operation());
                    assertEquals(0, interop.resultStatus());
                    assertTagOrder(interopTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
                    try (TtlvItemView payload = interopTtlv.itemAt(3);
                            TtlvValueView payloadValue = payload.value();
                            TtlvStructureView payloadView = payloadValue.structure()) {
                        assertEquals(0, payloadView.itemCount());
                    }
                }
                try (ClientBatchItemResult.LogResponse logResponse =
                                log.logResponse().orElseThrow();
                        TtlvStructureView logTtlv = logResponse.responseTtlv().view()) {
                    assertEquals(0x2c, log.operation());
                    assertEquals(0, log.resultStatus());
                    assertTagOrder(logTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
                    try (TtlvItemView payload = logTtlv.itemAt(3);
                            TtlvValueView payloadValue = payload.value();
                            TtlvStructureView payloadView = payloadValue.structure()) {
                        assertEquals(0, payloadView.itemCount());
                    }
                    assertFalse(logResponse.toString().contains("log-message-private-sentinel"));
                }
                try (ClientBatchItemResult.ExportResponse exportResponse =
                                exported.exportResponse().orElseThrow();
                        TtlvStructureView exportTtlv = exportResponse.responseTtlv().view();
                        TtlvStructureView exportAttributes =
                                exportResponse.attributes().orElseThrow().view();
                        TtlvStructureView exportObject = exportResponse.anyObject().orElseThrow().view()) {
                    assertEquals(0, exported.resultStatus());
                    assertEquals(0xfedc_ba98L, exportResponse.objectTypeRaw().orElseThrow());
                    assertEquals("export-secret-id", exportResponse.uniqueIdentifier().orElseThrow().text());
                    assertTagOrder(exportTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
                    assertTagOrder(exportAttributes, 0x42001d);
                    assertTagOrder(exportObject, 0x42008f);
                    assertTrue(containsByteString(exportObject, 0x420045,
                            "KMIPKit-export-sentinel".getBytes(StandardCharsets.US_ASCII)));
                    assertFalse(exportResponse.toString().contains("export-secret-id"));
                }
                try (ClientBatchItemResult.ExportResponse exportFailure =
                                exportFailed.exportResponse().orElseThrow();
                        TtlvStructureView exportFailureTtlv = exportFailure.responseTtlv().view()) {
                    assertEquals(1, exportFailed.resultStatus());
                    assertEquals(5, exportFailed.resultReason().orElseThrow());
                    assertTrue(exportFailure.objectTypeRaw().isEmpty());
                    assertTrue(exportFailure.uniqueIdentifier().isEmpty());
                    assertTrue(exportFailure.attributes().isEmpty());
                    assertTrue(exportFailure.anyObject().isEmpty());
                    assertTagOrder(exportFailureTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007e);
                }
                try (ClientBatchItemResult.GetConstraintsResponse constraintsResponse =
                                constraints.getConstraintsResponse().orElseThrow();
                        TtlvStructureView constraintsTtlv = constraintsResponse.responseTtlv().view()) {
                    assertEquals(0, constraints.resultStatus());
                    assertTagOrder(constraintsTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
                    try (TtlvItemView payload = constraintsTtlv.itemAt(3);
                            TtlvValueView payloadValue = payload.value();
                            TtlvStructureView payloadView = payloadValue.structure()) {
                        assertTagOrder(payloadView, 0x420168);
                        try (TtlvItemView constraintsItem = payloadView.itemAt(0);
                                TtlvValueView constraintsValue = constraintsItem.value();
                                TtlvStructureView constraintsView = constraintsValue.structure()) {
                            assertTrue(containsByteString(constraintsView, 0x540001,
                                    "KMIPKit-constraints-sentinel".getBytes(StandardCharsets.US_ASCII)));
                        }
                    }
                    assertFalse(constraintsResponse.toString().contains("constraints-sentinel"));
                }
                assertEquals(1, constraintsFailed.resultStatus());
                assertTrue(constraintsFailed.getConstraintsResponse().isEmpty());

                try (ClientBatchItemResult.GetResponse getResponse =
                                getMapped.getResponse().orElseThrow();
                        TtlvStructureView getObject = getResponse.object().orElseThrow().view()) {
                    assertEquals(2, getResponse.objectTypeRaw().orElseThrow());
                    assertEquals("get-response-secret-id",
                            getResponse.uniqueIdentifier().orElseThrow().text());
                    assertTagOrder(getObject, 0x42008f);
                    assertTrue(containsByteString(getObject, 0x420045,
                            "KMIPKit-get-response-sentinel".getBytes(StandardCharsets.US_ASCII)));
                    assertFalse(getResponse.toString().contains("get-response-secret-id"));
                }
                ClientBatchItemResult.LocateResponse locateResponse =
                        locateMapped.locateResponse().orElseThrow();
                assertEquals(3, locateResponse.locatedItems().orElseThrow());
                assertEquals(List.of("locate-first", "locate-second", "locate-first"),
                        locateResponse.uniqueIdentifiers().stream()
                                .map(UniqueIdentifier::text).toList());
                try (ClientBatchItemResult.GetResponse failedGetResponse =
                        getFailed.getResponse().orElseThrow()) {
                    ClientBatchItemResult.LocateResponse failedLocateResponse =
                            locateFailed.locateResponse().orElseThrow();
                    assertEquals(1, getFailed.resultStatus());
                    assertTrue(failedGetResponse.objectTypeRaw().isEmpty());
                    assertTrue(failedGetResponse.uniqueIdentifier().isEmpty());
                    assertTrue(failedGetResponse.object().isEmpty());
                    assertEquals(1, locateFailed.resultStatus());
                    assertTrue(failedLocateResponse.locatedItems().isEmpty());
                    assertTrue(failedLocateResponse.uniqueIdentifiers().isEmpty());
                }
            }
            byte[] request = peer.get(12, TimeUnit.SECONDS);
            assertEquals(1L, SharedCapabilityParityFixtureTest.numberField(
                    fixtureExpected, "transportExchangeCount"));
            assertEquals("false", SharedCapabilityParityFixtureTest.stringOrBooleanField(
                    fixtureExpected, "automaticFollowUp"));
            assertTrue(request.length >= 8);
            assertArrayEquals(new byte[] {0x42, 0x00, 0x78},
                    new byte[] {request[0], request[1], request[2]});
            assertTrue(containsOrderedDeriveKeyIdentifiersInBatchItem(request,
                    item(0x420094, 7, "source-key".getBytes(StandardCharsets.UTF_8)),
                    enumeration(0x420094, (int) 0xfedc_ba98L),
                    item(0x420094, 2, integer(-17))));
        } finally {
            listener.close();
        }
    }

    @Test
    void usageAllocationContextGuardsJavaProtectionExecutionLocally() throws Exception {
        String fixture = Files.readString(SharedCapabilityParityFixtureTest.sharedFixturePath());
        String usageFixture = SharedCapabilityParityFixtureTest.caseById(fixture,
                "usage-allocation.guard-accounting.v1");
        String fixtureInput = SharedCapabilityParityFixtureTest.objectField(usageFixture, "input");
        String fixtureExpected = SharedCapabilityParityFixtureTest.objectField(usageFixture, "expectedOutput");
        String allocationIdentifier = SharedCapabilityParityFixtureTest.stringField(
                fixtureInput, "allocationIdentifier");
        long allocationCount = SharedCapabilityParityFixtureTest.numberField(fixtureInput, "allocationCount");
        long usageLimitsUnit = SharedCapabilityParityFixtureTest.numberField(fixtureInput, "usageLimitsUnit");
        long successfulCost = SharedCapabilityParityFixtureTest.numberField(fixtureInput, "successfulCost");
        long overBudgetCost = SharedCapabilityParityFixtureTest.numberField(fixtureInput, "overBudgetCost");
        assertEquals("Encrypt", SharedCapabilityParityFixtureTest.stringField(fixtureInput, "operation"));
        assertEquals("invalid_input", SharedCapabilityParityFixtureTest.stringField(
                usageFixture, "errorCategory"));

        Path serverStorePath = temporaryDirectory.resolve("usage-guard-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("usage-guard-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");
        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);
        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);

        byte[] allocationItemId = uniqueBatchItemId(0);
        byte[] encryptionItemId = uniqueBatchItemId(1);
        byte[] directGuardItemId = uniqueBatchItemId(2);
        CodecLimits limits = CodecLimits.defaults();
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveResponses(listener,
                usageAllocationResponseMessage(allocationItemId, allocationIdentifier),
                guardedEncryptResponseMessage(encryptionItemId, allocationIdentifier),
                guardedEncryptResponseMessage(directGuardItemId, allocationIdentifier)));
        Thread peerThread = new Thread(peer, "kmipkit-java-usage-allocation-guard-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport)) {
            try (ClientBatch allocationBatch = ClientBatch.create()) {
                allocationBatch.add(ClientBatchItem.getUsageAllocation(
                        UniqueIdentifier.textString(allocationIdentifier), allocationCount)
                        .withUniqueBatchItemId(allocationItemId));
                try (ClientBatchResponse allocationResponse = client.execute(allocationBatch, limits);
                        ClientBatchItemResult allocation = allocationResponse.itemAt(0);
                        UsageAllocationContext context =
                                allocation.createUsageAllocationContext(allocationCount, usageLimitsUnit)) {
                    String initial = SharedCapabilityParityFixtureTest.objectField(fixtureExpected, "initial");
                    assertEquals(allocationIdentifier, context.uniqueIdentifier().text());
                    assertEquals(usageLimitsUnit, context.usageLimitsUnit());
                    assertEquals(SharedCapabilityParityFixtureTest.numberField(initial, "allocated"),
                            context.allocatedUnits());
                    assertEquals(SharedCapabilityParityFixtureTest.numberField(initial, "available"),
                            context.availableUnits());
                    assertEquals(SharedCapabilityParityFixtureTest.numberField(initial, "reserved"),
                            context.reservedUnits());
                    String afterSuccess = SharedCapabilityParityFixtureTest.objectField(
                            fixtureExpected, "afterSuccessfulProtection");

                    try (TtlvStructure parameters = TtlvStructure.create();
                            TtlvValue data = TtlvValue.ByteString(new byte[] {1, 2, 3}, limits);
                            ClientBatchItem encryption = ClientBatchItem.encrypt(
                                            UniqueIdentifier.textString(allocationIdentifier),
                                            parameters, data, null, null, null, null, null)
                                    .withUniqueBatchItemId(encryptionItemId)) {
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItemWithAsynchronousIndicator(
                                        context, encryption, successfulCost, -1L, limits));
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItemWithAsynchronousIndicator(
                                        context, encryption, successfulCost, 0x1_0000_0000L, limits));
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItem(null, encryption, successfulCost, limits));
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItem(context, null, successfulCost, limits));
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItem(context, encryption, 0L, limits));
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItem(context, encryption, successfulCost, null));

                        try (ClientBatchResponse guarded = client.executeGuardedProtectionItemWithAsynchronousIndicator(
                                    context, encryption, successfulCost, 2L, limits);
                                ClientBatchItemResult guardedItem = guarded.itemAt(0);
                                ClientBatchItemResult.EncryptResponse encrypted =
                                        guardedItem.encryptResponse().orElseThrow()) {
                            assertEquals(0L, guardedItem.resultStatus());
                            assertEquals(allocationIdentifier, encrypted.uniqueIdentifier().text());
                            assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "allocated"),
                                    context.allocatedUnits());
                            assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "reserved"),
                                    context.reservedUnits());
                            assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "available"),
                                    context.availableUnits());
                        }
                    }

                    try (TtlvStructure parameters = TtlvStructure.create();
                            TtlvValue data = TtlvValue.ByteString(new byte[] {4, 5, 6}, limits);
                            ClientBatchItem overBudget = ClientBatchItem.encrypt(
                                    UniqueIdentifier.textString(allocationIdentifier),
                                    parameters, data, null, null, null, null, null)) {
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItem(
                                        context, overBudget, overBudgetCost, limits));
                        String rejected = SharedCapabilityParityFixtureTest.objectField(
                                fixtureExpected, "overBudgetAttempt");
                        assertEquals("false", SharedCapabilityParityFixtureTest.stringOrBooleanField(
                                rejected, "accepted"));
                        assertEquals("true", SharedCapabilityParityFixtureTest.stringOrBooleanField(
                                rejected, "stateUnchanged"));
                        assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "allocated"),
                                context.allocatedUnits());
                    }

                    try (TtlvStructure parameters = TtlvStructure.create();
                            TtlvValue data = TtlvValue.ByteString(new byte[] {1}, limits);
                            ClientBatchItem mismatched = ClientBatchItem.encrypt(
                                    UniqueIdentifier.textString("another-object"),
                                    parameters, data, null, null, null, null, null)) {
                        assertThrows(InvalidInputException.class,
                                () -> client.executeGuardedProtectionItem(
                                        context, mismatched, 1L, limits));
                        assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "allocated"),
                                context.allocatedUnits());
                    }
                    assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "allocated"),
                            context.allocatedUnits());
                    assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "reserved"),
                            context.reservedUnits());
                    assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "available"),
                            context.availableUnits());
                    assertEquals("true", SharedCapabilityParityFixtureTest.stringOrBooleanField(
                            fixtureExpected, "localOnlyAccounting"));

                    // Exercise the convenience overload in addition to the fixture's async-indicator path.
                    try (TtlvStructure parameters = TtlvStructure.create();
                            TtlvValue data = TtlvValue.ByteString(new byte[] {7, 8, 9}, limits);
                            ClientBatchItem directGuard = ClientBatchItem.encrypt(
                                            UniqueIdentifier.textString(allocationIdentifier),
                                            parameters, data, null, null, null, null, null)
                                    .withUniqueBatchItemId(directGuardItemId);
                            ClientBatchResponse guarded = client.executeGuardedProtectionItem(
                                    context, directGuard, successfulCost, limits);
                            ClientBatchItemResult guardedItem = guarded.itemAt(0)) {
                        assertEquals(0L, guardedItem.resultStatus());
                        assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "allocated"),
                                context.allocatedUnits());
                        assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "reserved")
                                        + successfulCost,
                                context.reservedUnits());
                        assertEquals(SharedCapabilityParityFixtureTest.numberField(afterSuccess, "available")
                                        - successfulCost,
                                context.availableUnits());
                    }
                }
            }
        } finally {
            listener.close();
        }
        assertEquals(SharedCapabilityParityFixtureTest.numberField(fixtureExpected, "transportExchangeCount") + 1,
                peer.get(10, TimeUnit.SECONDS).size());
    }

    @Test
    void deriveKeyWireAssertionRequiresTheExactOrderedIdentifierList() throws Exception {
        byte[] textIdentifier = item(0x420094, 7,
                "source-key".getBytes(StandardCharsets.UTF_8));
        byte[] enumerationIdentifier = enumeration(0x420094, (int) 0xfedc_ba98L);
        byte[] integerIdentifier = item(0x420094, 2, integer(-17));
        byte[][] expected = {textIdentifier, enumerationIdentifier, integerIdentifier};
        byte[] extraIdentifier = item(0x420094, 2, integer(-18));
        byte[] operation = enumeration(0x42005c, 5);

        byte[] exactRequest = structure(0x420078,
                structure(0x42000f, operation,
                        structure(0x420079, textIdentifier, enumerationIdentifier, integerIdentifier)));
        byte[] requestWithExtraIdentifier = structure(0x420078,
                structure(0x42000f, operation,
                        structure(0x420079, textIdentifier, extraIdentifier,
                                enumerationIdentifier, integerIdentifier)));

        assertTrue(containsOrderedDeriveKeyIdentifiersInBatchItem(exactRequest, expected));
        assertFalse(containsOrderedDeriveKeyIdentifiersInBatchItem(requestWithExtraIdentifier, expected));
    }

    @Test
    void cryptoOperationResponsesConvertSuccessfullyOverMutualTls() throws Exception {
        executeCryptoOperationsOverMutualTls(true);
    }

    @Test
    void cryptoOperationResponsesPreserveOmittedOptionalFieldsOverMutualTls() throws Exception {
        executeCryptoOperationsOverMutualTls(false);
    }

    private void executeCryptoOperationsOverMutualTls(boolean includeOptionalResponseFields) throws Exception {
        List<?> operations = cryptoOperations();
        Path serverStorePath = temporaryDirectory.resolve("crypto-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("crypto-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener,
                        cryptoOperationResponseMessage(operations, includeOptionalResponseFields)));
        Thread peerThread = new Thread(peer, "kmipkit-java-crypto-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            int index = 0;
            for (Object operation : operations) {
                batch.add(cryptoOperationItem(operation)
                        .withUniqueBatchItemId(uniqueBatchItemId(index++)));
            }

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults())) {
                assertEquals(operations.size(), response.itemCount());
                for (int operationIndex = 0; operationIndex < operations.size(); operationIndex++) {
                    Object operation = operations.get(operationIndex);
                    try (ClientBatchItemResult result = response.itemAt(operationIndex)) {
                        assertEquals(0, result.resultStatus());
                        Object typedResponse = cryptoTypedResponse(cryptoOperationName(operation), result);
                        if (includeOptionalResponseFields) {
                            boolean validityIndicatorPresent = !cryptoOperationName(operation).endsWith("Verify");
                            invokeCryptoFixture(null, "assertSuccessfulOutput",
                                    new Class<?>[] {operation.getClass(), Object.class, boolean.class},
                                    operation, typedResponse, validityIndicatorPresent);
                        } else {
                            assertCryptoOptionalFieldsAbsent(cryptoOperationName(operation), typedResponse);
                        }
                    }
                }
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    private static void assertCryptoOptionalFieldsAbsent(String operationName, Object response) {
        switch (operationName) {
            case "encrypt" -> {
                ClientBatchItemResult.EncryptResponse encrypt =
                        (ClientBatchItemResult.EncryptResponse) response;
                try (encrypt) {
                    assertTrue(encrypt.data().isPresent());
                    assertTrue(encrypt.ivCounterNonce().isEmpty());
                    assertTrue(encrypt.correlationValue().isEmpty());
                    assertTrue(encrypt.authenticatedEncryptionTag().isEmpty());
                }
            }
            case "decrypt" -> {
                ClientBatchItemResult.DecryptResponse decrypt =
                        (ClientBatchItemResult.DecryptResponse) response;
                try (decrypt) {
                    assertTrue(decrypt.data().isPresent());
                    assertTrue(decrypt.correlationValue().isEmpty());
                }
            }
            case "hash" -> {
                ClientBatchItemResult.HashResponse hash = (ClientBatchItemResult.HashResponse) response;
                assertTrue(hash.data().isPresent());
                assertTrue(hash.correlationValue().isEmpty());
            }
            case "mac" -> {
                ClientBatchItemResult.MacResponse mac = (ClientBatchItemResult.MacResponse) response;
                assertTrue(mac.macData().isPresent());
                assertTrue(mac.correlationValue().isEmpty());
            }
            case "macVerify" -> {
                ClientBatchItemResult.MacVerifyResponse macVerify =
                        (ClientBatchItemResult.MacVerifyResponse) response;
                assertTrue(macVerify.validityIndicatorRaw().isEmpty());
                assertTrue(macVerify.correlationValue().isEmpty());
            }
            case "sign" -> {
                ClientBatchItemResult.SignResponse sign = (ClientBatchItemResult.SignResponse) response;
                assertTrue(sign.signatureData().isPresent());
                assertTrue(sign.correlationValue().isEmpty());
            }
            case "signatureVerify" -> {
                ClientBatchItemResult.SignatureVerifyResponse signatureVerify =
                        (ClientBatchItemResult.SignatureVerifyResponse) response;
                assertTrue(signatureVerify.validityIndicatorRaw().isEmpty());
                assertTrue(signatureVerify.recoveredData().isPresent());
                assertTrue(signatureVerify.correlationValue().isEmpty());
            }
            default -> throw new IllegalArgumentException("unknown crypto operation: " + operationName);
        }
    }

    @Test
    void discoverVersionsResponseCanExceedTheDefaultElementLimitWhenConfigured() throws Exception {
        int supportedVersionCount = 100_001;
        Path serverStorePath = temporaryDirectory.resolve("large-discover-versions-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("large-discover-versions-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, discoverVersionsResponseMessage(supportedVersionCount)));
        Thread peerThread = new Thread(peer, "kmipkit-java-large-discover-versions-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        CodecLimits limits = CodecLimits.create(16 * 1024 * 1024L, 64, 400_000);
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport)) {
            ClientBatch batch = ClientBatch.create();
            batch.add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(uniqueBatchItemId(0)));
            try (ClientBatchResponse response = client.execute(batch, limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                List<ProtocolVersion> versions = result.supportedVersions().orElseThrow();
                assertEquals(supportedVersionCount, versions.size());
                assertEquals(new ProtocolVersion(2, 1), versions.get(0));
                assertEquals(new ProtocolVersion(2, 1), versions.get(versions.size() - 1));
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    @Test
    void createSplitKeyResponsePreservesMoreThanOneHundredThousandIdentifiersOverTls()
            throws Exception {
        int identifierCount = 100_001;
        Path serverStorePath = temporaryDirectory.resolve("large-split-key-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("large-split-key-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, createSplitKeyResponseMessage(identifierCount)));
        Thread peerThread = new Thread(peer, "kmipkit-java-large-split-key-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        CodecLimits limits = CodecLimits.create(16 * 1024 * 1024L, 64, 400_000);
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                TtlvStructure attributes = TtlvStructure.create()) {
            ClientBatch batch = ClientBatch.create();
            batch.add(ClientBatchItem.createSplitKey(7, null, 2, 2, 1, null, attributes, null));
            try (ClientBatchResponse response = client.execute(batch, limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                List<UniqueIdentifier> identifiers = result.createSplitKeyUniqueIdentifiers()
                        .orElseThrow();
                assertEquals(identifierCount, identifiers.size());
                assertEquals(UniqueIdentifier.Kind.INTEGER, identifiers.get(0).kind());
                assertEquals(0, identifiers.get(0).number());
                assertEquals(50_000, identifiers.get(50_000).number());
                assertEquals(identifierCount - 1,
                        identifiers.get(identifiers.size() - 1).number());
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    @Test
    void joinSplitKeyResponseUsesNativeMappingAndRetainsOwnedTtlvOverTls() throws Exception {
        Path serverStorePath = temporaryDirectory.resolve("join-split-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("join-split-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, joinSplitKeyResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-join-split-response-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            try (SplitKeyReference reference = new SplitKeyReference(
                    UniqueIdentifier.textString("source-fragment"), 1)) {
                batch.add(ClientBatchItem.joinSplitKey(4, List.of(reference))
                        .withUniqueBatchItemId(uniqueBatchItemId(0)));
            }

            ClientBatchItemResult.JoinSplitKeyResponse joined;
            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(1, response.itemCount());
                assertEquals(0, result.resultStatus());
                joined = result.joinSplitKeyResponse().orElseThrow();
            }

            try (joined; TtlvStructureView responseTtlv = joined.responseTtlv().view()) {
                assertEquals("joined-output-id", joined.uniqueIdentifier().text());
                assertFalse(joined.toString().contains("joined-output-id"));
                assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
                try (TtlvItemView payload = responseTtlv.itemAt(3);
                        TtlvValueView payloadValue = payload.value();
                        TtlvStructureView payloadView = payloadValue.structure()) {
                    assertTagOrder(payloadView, 0x420094, 0x540001);
                }
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    // OASIS KMIP Specification v2.1 §6.1.35, Tables 268–270.
    @Test
    void recertifyResponseUsesNativeJniBridgeAndRetainsOrderedOwnedTtlvOverTls() throws Exception {
        Path serverStorePath = temporaryDirectory.resolve("recertify-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("recertify-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, recertifyResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-recertify-response-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.RecertifyResponse recertified;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.recertify(
                    UniqueIdentifier.textString("certificate-to-recertify"), null,
                    null, null, null, null, null)
                    .withUniqueBatchItemId(uniqueBatchItemId(0)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult item = response.itemAt(0)) {
                assertEquals(1, response.itemCount());
                assertEquals(0x07, item.operation());
                assertEquals(0, item.resultStatus());
                recertified = item.recertifyResponse().orElseThrow();
            }
        } finally {
            listener.close();
        }

        try (recertified; TtlvStructureView responseTtlv = recertified.responseTtlv().view()) {
            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, recertified.uniqueIdentifier().kind());
            assertEquals("replacement-certificate", recertified.uniqueIdentifier().text());
            assertEquals(4, responseTtlv.itemCount());
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertEquals(2, payloadStructure.itemCount());
                assertTagOrder(payloadStructure, 0x420094, 0x541234);
                assertTrue(containsByteString(payloadStructure, 0x541234,
                        new byte[] {0x01, 0x00, (byte) 0xff}));
            }
            assertFalse(recertified.toString().contains("replacement-certificate"));
        }
        assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
    }

    // OASIS KMIP Specification v2.1 §6.1.48, Tables 313–315.
    @Test
    void parameterlessReprovisionPreservesOptionalIdentifierAndDoesNotPoll() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, reprovisionResponseWithOptionalIdentifiersMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-reprovision-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.ReProvisionResponse withIdentifier;
        ClientBatchItemResult.ReProvisionResponse reprovisioned;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create();
                ClientBatchItem firstItem = ClientBatchItem.reprovision(null, null)
                        .withUniqueBatchItemId(uniqueBatchItemId(0));
                ClientBatchItem secondItem = ClientBatchItem.reprovision(null, null)
                        .withUniqueBatchItemId(uniqueBatchItemId(1))) {
            batch.add(firstItem);
            batch.add(secondItem);
            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult resultWithIdentifier = response.itemAt(0);
                    ClientBatchItemResult resultWithoutIdentifier = response.itemAt(1)) {
                assertEquals(2, response.itemCount());
                assertEquals(0x35, resultWithIdentifier.operation());
                assertEquals(0, resultWithIdentifier.resultStatus());
                assertEquals(0x35, resultWithoutIdentifier.operation());
                assertEquals(0, resultWithoutIdentifier.resultStatus());
                assertNativeReprovisionTtlvBridge(resultWithIdentifier, true);
                assertNativeReprovisionTtlvBridge(resultWithoutIdentifier, true);
                withIdentifier = resultWithIdentifier.reprovisionResponse().orElseThrow();
                reprovisioned = resultWithoutIdentifier.reprovisionResponse().orElseThrow();
            }
            request = peer.get(15, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        try (withIdentifier;
                TtlvStructureView identifierTtlv = withIdentifier.responseTtlv().view();
                reprovisioned; TtlvStructureView responseTtlv = reprovisioned.responseTtlv().view()) {
            assertEquals(
                    "server-created-certificate", withIdentifier.uniqueIdentifier().orElseThrow().text());
            assertTrue(reprovisioned.uniqueIdentifier().isEmpty());
            assertTagOrder(identifierTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            assertEquals(4, responseTtlv.itemCount());
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertEquals(0, payloadStructure.itemCount());
            }
            assertTrue(reprovisioned.toString().contains("uniqueIdentifier=absent"));
        }
        assertEquals(2, countOccurrences(request, enumeration(0x42005c, 0x35)));
        assertEquals(0, countOccurrences(request, enumeration(0x42005c, 0x20)));
        assertTrue(peer.isDone());
    }

    // OASIS KMIP Specification v2.1 §6.1.43, Tables 291–294.
    @Test
    void registerResponseUsesNativeJniBridgeAndPreservesOwnedOrderedTtlv() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, registerResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-register-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.RegisterResponse registered;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create();
                RegisterRequest registerRequest = RegisterRequest.create(
                        8, TtlvStructure.create(), registerOpaqueObject());
                ClientBatchItem item = ClientBatchItem.register(registerRequest)
                        .withUniqueBatchItemId(uniqueBatchItemId(0))) {
            batch.add(item);
            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(1, response.itemCount());
                assertEquals(0x03, result.operation());
                assertEquals(0, result.resultStatus());
                registered = result.registerResponse().orElseThrow();
            }
            request = peer.get(15, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        try (registered; TtlvStructureView responseTtlv = registered.responseTtlv().view()) {
            assertEquals("registered-server-id", registered.uniqueIdentifier().text());
            assertEquals(4, responseTtlv.itemCount());
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertEquals(2, payloadStructure.itemCount());
                assertTagOrder(payloadStructure, 0x420094, 0x541234);
            }
            assertFalse(registered.toString().contains("registered-server-id"));
        }
        assertEquals(1, countOccurrences(request, enumeration(0x42005c, 0x03)));
        assertEquals(0, countOccurrences(request, enumeration(0x42005c, 0x20)));
        assertTrue(peer.isDone());
    }

    // OASIS KMIP Specification v2.1 §§4.47, 6.1.44, and 11.48.
    @Test
    void revokeResponseUsesNativeJniBridgeAndPreservesSuccessFailureAndPending() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, revokeResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-revoke-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.RevokeResponse successful;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.asynchronousIndicator(1);
            batch.add(ClientBatchItem.revoke(RevokeRequest.create(
                    UniqueIdentifier.textString("existing-revoke-target"),
                    RevocationReason.create(2, "operator audit"),
                    1_700_000_000L)).withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.revoke(RevokeRequest.create(
                    UniqueIdentifier.textString("missing-revoke-target"),
                    RevocationReason.create(4, null), null))
                    .withUniqueBatchItemId(uniqueBatchItemId(1)));
            batch.add(ClientBatchItem.revoke(RevokeRequest.create(
                    UniqueIdentifier.textString("pending-revoke-target"),
                    RevocationReason.create(3, null), null))
                    .withUniqueBatchItemId(uniqueBatchItemId(2)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult success = response.itemAt(0);
                    ClientBatchItemResult failure = response.itemAt(1);
                    ClientBatchItemResult pending = response.itemAt(2)) {
                assertEquals(3, response.itemCount());
                assertEquals(0x13, success.operation());
                assertEquals(0, success.resultStatus());
                assertTrue(success.outcomeKind().isCompleted());
                successful = success.revokeResponse().orElseThrow();

                assertEquals(0x13, failure.operation());
                assertEquals(1, failure.resultStatus());
                assertEquals(5, failure.resultReason().orElseThrow());
                assertEquals(java.util.Optional.of("revoke rejected"), failure.resultMessage());
                assertTrue(failure.outcomeKind().isCompleted());
                assertTrue(failure.revokeResponse().isEmpty());

                assertEquals(0x13, pending.operation());
                assertEquals(2, pending.resultStatus());
                assertTrue(pending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.revokeResponse().isEmpty());
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        try (successful; TtlvStructureView responseTtlv = successful.responseTtlv().view()) {
            assertEquals("revoked-server-id", successful.uniqueIdentifier().text());
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertTagOrder(payloadStructure, 0x420094, 0x541234);
                assertTrue(containsByteString(payloadStructure, 0x541234,
                        new byte[] {0x01, 0x00, (byte) 0xff}));
            }
            assertFalse(successful.toString().contains("revoked-server-id"));
        }
        assertContains(request, enumeration(0x42005c, 0x13));
        assertEquals(3, countOccurrences(request, enumeration(0x42005c, 0x13)));
        assertContains(request, structure(0x420081,
                enumeration(0x420082, 2),
                item(0x420080, 7, "operator audit".getBytes(StandardCharsets.UTF_8))));
        assertContains(request, item(0x420021, 9, longInteger(1_700_000_000L)));
        assertTrue(peer.isDone());
    }

    @Test
    void lifecycleRequestsPreserveIdentifiersAndFailurePendingResponsesWithoutFollowUp()
            throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveLifecycleResponseWithoutFollowUp(
                        listener, lifecycleFailurePendingResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-lifecycle-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.asynchronousIndicator(1);
            batch.add(ClientBatchItem.activate(UniqueIdentifier.textString("activate-target"))
                    .withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.archive(UniqueIdentifier.enumeration(0x8fed_cba9L))
                    .withUniqueBatchItemId(uniqueBatchItemId(1)));
            batch.add(ClientBatchItem.destroy(UniqueIdentifier.integer(-723))
                    .withUniqueBatchItemId(uniqueBatchItemId(2)));
            batch.add(ClientBatchItem.recover(UniqueIdentifier.textString("recover-target"))
                    .withUniqueBatchItemId(uniqueBatchItemId(3)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult activate = response.itemAt(0);
                    ClientBatchItemResult archive = response.itemAt(1);
                    ClientBatchItemResult destroy = response.itemAt(2);
                    ClientBatchItemResult recover = response.itemAt(3)) {
                assertEquals(4, response.itemCount());

                assertEquals(0x12, activate.operation());
                assertEquals(1, activate.resultStatus());
                assertEquals(5, activate.resultReason().orElseThrow());
                assertEquals(java.util.Optional.of("activate rejected"), activate.resultMessage());
                assertTrue(activate.outcomeKind().isCompleted());
                assertTrue(activate.activateResponse().isEmpty());

                assertEquals(0x15, archive.operation());
                assertEquals(1, archive.resultStatus());
                assertEquals(5, archive.resultReason().orElseThrow());
                assertEquals(java.util.Optional.of("archive rejected"), archive.resultMessage());
                assertTrue(archive.outcomeKind().isCompleted());
                assertTrue(archive.archiveResponse().isEmpty());

                assertEquals(0x14, destroy.operation());
                assertEquals(2, destroy.resultStatus());
                assertTrue(destroy.outcomeKind().isPending());
                assertArrayEquals(LIFECYCLE_DESTROY_CORRELATION,
                        destroy.pendingCorrelationValue().orElseThrow());
                assertTrue(destroy.destroyResponse().isEmpty());

                assertEquals(0x16, recover.operation());
                assertEquals(2, recover.resultStatus());
                assertTrue(recover.outcomeKind().isPending());
                assertArrayEquals(LIFECYCLE_RECOVER_CORRELATION,
                        recover.pendingCorrelationValue().orElseThrow());
                assertTrue(recover.recoverResponse().isEmpty());
            }
            request = peer.get(12, TimeUnit.SECONDS);
            listener.setSoTimeout(250);
            assertThrows(SocketTimeoutException.class, () -> listener.accept());
        } finally {
            listener.close();
        }

        assertContains(request, structure(0x42000f,
                enumeration(0x42005c, 0x12),
                item(0x420093, 8, uniqueBatchItemId(0)),
                structure(0x420079,
                        item(0x420094, 7, "activate-target".getBytes(StandardCharsets.UTF_8)))));
        assertContains(request, structure(0x42000f,
                enumeration(0x42005c, 0x15),
                item(0x420093, 8, uniqueBatchItemId(1)),
                structure(0x420079,
                        enumeration(0x420094, (int) 0x8fed_cba9L))));
        assertContains(request, structure(0x42000f,
                enumeration(0x42005c, 0x14),
                item(0x420093, 8, uniqueBatchItemId(2)),
                structure(0x420079, item(0x420094, 2, integer(-723)))));
        assertContains(request, structure(0x42000f,
                enumeration(0x42005c, 0x16),
                item(0x420093, 8, uniqueBatchItemId(3)),
                structure(0x420079,
                        item(0x420094, 7, "recover-target".getBytes(StandardCharsets.UTF_8)))));
        assertTrue(peer.isDone());
    }

    @Test
    void lifecycleSuccessResponsesExposeTypedIdentifiersAndRedactThem() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, lifecycleSuccessResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-lifecycle-success-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.activate(UniqueIdentifier.textString("activate-request"))
                    .withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.archive(UniqueIdentifier.textString("archive-request"))
                    .withUniqueBatchItemId(uniqueBatchItemId(1)));
            batch.add(ClientBatchItem.destroy(UniqueIdentifier.textString("destroy-request"))
                    .withUniqueBatchItemId(uniqueBatchItemId(2)));
            batch.add(ClientBatchItem.recover(UniqueIdentifier.textString("recover-request"))
                    .withUniqueBatchItemId(uniqueBatchItemId(3)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult activate = response.itemAt(0);
                    ClientBatchItemResult archive = response.itemAt(1);
                    ClientBatchItemResult destroy = response.itemAt(2);
                    ClientBatchItemResult recover = response.itemAt(3)) {
                assertEquals(4, response.itemCount());
                assertEquals("activate-result-secret",
                        activate.activateResponse().orElseThrow().uniqueIdentifier().text());

                ClientBatchItemResult.ArchiveResponse archiveResponse =
                        archive.archiveResponse().orElseThrow();
                assertEquals(UniqueIdentifier.Kind.ENUMERATION,
                        archiveResponse.uniqueIdentifier().kind());
                assertEquals(0x8fed_cba9L, archiveResponse.uniqueIdentifier().number());

                ClientBatchItemResult.DestroyResponse destroyResponse =
                        destroy.destroyResponse().orElseThrow();
                assertEquals(UniqueIdentifier.Kind.INTEGER,
                        destroyResponse.uniqueIdentifier().kind());
                assertEquals(-723, destroyResponse.uniqueIdentifier().number());

                ClientBatchItemResult.RecoverResponse recoverResponse =
                        recover.recoverResponse().orElseThrow();
                assertEquals("recover-result-secret", recoverResponse.uniqueIdentifier().text());

                assertTrue(activate.activateResponse().orElseThrow().toString().contains("redacted"));
                assertFalse(activate.activateResponse().orElseThrow().toString()
                        .contains("activate-result-secret"));
                assertTrue(archiveResponse.toString().contains("redacted"));
                assertTrue(destroyResponse.toString().contains("redacted"));
                assertTrue(recoverResponse.toString().contains("redacted"));
                assertFalse(recoverResponse.toString().contains("recover-result-secret"));
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        assertContains(request, item(0x42005c, 5, integer(0x12)));
        assertContains(request, item(0x42005c, 5, integer(0x15)));
        assertContains(request, item(0x42005c, 5, integer(0x14)));
        assertContains(request, item(0x42005c, 5, integer(0x16)));
        assertTrue(peer.isDone());
    }

    // OASIS KMIP Specification v2.1 §6.1.52, Tables 325–327.
    @Test
    void setConstraintsResponseUsesNativeJniBridgeAndExposesTtlvOnlyOnSuccess() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, setConstraintsResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-set-constraints-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.SetConstraintsResponse successful;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create();
                TtlvStructure constraints = TtlvStructure.create()) {
            batch.asynchronousIndicator(1);
            batch.add(ClientBatchItem.setConstraints(constraints)
                    .withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.setConstraints(constraints)
                    .withUniqueBatchItemId(uniqueBatchItemId(1)));
            batch.add(ClientBatchItem.setConstraints(constraints)
                    .withUniqueBatchItemId(uniqueBatchItemId(2)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult success = response.itemAt(0);
                    ClientBatchItemResult failure = response.itemAt(1);
                    ClientBatchItemResult pending = response.itemAt(2)) {
                assertEquals(3, response.itemCount());
                assertEquals(0x37, success.operation());
                assertEquals(0, success.resultStatus());
                successful = success.setConstraintsResponse().orElseThrow();

                assertEquals(0x37, failure.operation());
                assertEquals(1, failure.resultStatus());
                assertEquals(5, failure.resultReason().orElseThrow());
                assertEquals(java.util.Optional.of("constraints rejected"), failure.resultMessage());
                assertTrue(failure.setConstraintsResponse().isEmpty());

                assertEquals(0x37, pending.operation());
                assertEquals(2, pending.resultStatus());
                assertTrue(pending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.setConstraintsResponse().isEmpty());
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        try (successful; TtlvStructureView responseTtlv = successful.responseTtlv().view()) {
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertEquals(0, payloadStructure.itemCount());
            }
            assertFalse(successful.toString().contains("response_ttlv"));
        }
        assertEquals(3, countOccurrences(request, enumeration(0x42005c, 0x37)));
        assertContains(request, structure(0x420168));
        assertTrue(peer.isDone());
    }

    // OASIS KMIP Specification v2.1 §6.1.53, Tables 328–330.
    @Test
    void setDefaultsResponseUsesNativeJniBridgeAndExposesTtlvOnlyOnSuccess() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, setDefaultsResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-set-defaults-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.SetDefaultsResponse successful;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create();
                TtlvStructure defaultsInformation = TtlvStructure.create()) {
            batch.asynchronousIndicator(1);
            batch.add(ClientBatchItem.setDefaults()
                    .withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.setDefaults(defaultsInformation)
                    .withUniqueBatchItemId(uniqueBatchItemId(1)));
            batch.add(ClientBatchItem.setDefaults()
                    .withUniqueBatchItemId(uniqueBatchItemId(2)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult success = response.itemAt(0);
                    ClientBatchItemResult failure = response.itemAt(1);
                    ClientBatchItemResult pending = response.itemAt(2)) {
                assertEquals(3, response.itemCount());
                assertEquals(0x36, success.operation());
                assertEquals(0, success.resultStatus());
                successful = success.setDefaultsResponse().orElseThrow();

                assertEquals(0x36, failure.operation());
                assertEquals(1, failure.resultStatus());
                assertEquals(7, failure.resultReason().orElseThrow());
                assertEquals(java.util.Optional.of("defaults rejected"), failure.resultMessage());
                assertTrue(failure.setDefaultsResponse().isEmpty());

                assertEquals(0x36, pending.operation());
                assertEquals(2, pending.resultStatus());
                assertTrue(pending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.setDefaultsResponse().isEmpty());
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        try (successful; TtlvStructureView responseTtlv = successful.responseTtlv().view()) {
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertEquals(0, payloadStructure.itemCount());
            }
            assertFalse(successful.toString().contains("response_ttlv"));
        }
        assertContains(request, enumeration(0x42005c, 0x36));
        assertEquals(3, countOccurrences(request, enumeration(0x42005c, 0x36)));
        assertContains(request, structure(0x420152));
        assertTrue(peer.isDone());
    }

    // OASIS KMIP Specification v2.1 §6.1.57, Tables 341–342.
    @Test
    void validateResponseUsesNativeJniBridgeAndPreservesOpenValidityIndicator() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        SSLServerSocket listener = deterministicMtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificate);
        long rawValidityIndicator = 0xf123_4567L;
        byte[] responseSentinel =
                "validate-response-secret-sentinel".getBytes(StandardCharsets.UTF_8);
        byte[] certificateBytes = "validate-certificate-request-sentinel"
                .getBytes(StandardCharsets.UTF_8);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(
                        listener, validateResponseMessage(rawValidityIndicator, responseSentinel)));
        Thread peerThread = new Thread(peer, "kmipkit-java-validate-tls-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            try (TtlvStructure certificate = validateCertificate(certificateBytes)) {
                batch.add(ClientBatchItem.validate(List.of(certificate),
                        List.of(UniqueIdentifier.textString("validate-target")), null)
                        .withUniqueBatchItemId(uniqueBatchItemId(0)));
            }

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult result = response.itemAt(0);
                    ClientBatchItemResult.ValidateResponse validateResponse =
                            result.validateResponse().orElseThrow();
                    TtlvStructureView responseTtlv = validateResponse.responseTtlv().view()) {
                assertEquals(1, response.itemCount());
                assertEquals(0x17, result.operation());
                assertEquals(0, result.resultStatus());
                assertEquals(rawValidityIndicator, validateResponse.validityIndicatorRaw());
                assertTrue(responseTtlv.itemCount() > 0);
                assertTrue(containsByteString(responseTtlv, 0x541234, responseSentinel));
                assertFalse(validateResponse.toString().contains("validate-response-secret-sentinel"));
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        assertContains(request, enumeration(0x42005c, 0x17));
        assertContains(request, item(0x420094, 7,
                "validate-target".getBytes(StandardCharsets.UTF_8)));
        assertContains(request, structure(0x420013,
                enumeration(0x42001d, 1),
                item(0x42001e, 8, certificateBytes)));
        assertTrue(peer.isDone());
    }

    private static TtlvStructure validateCertificate(byte[] value) {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvStructure empty = TtlvStructure.create();
                RawTag typeRaw = RawTag.fromRaw(0x0042_001d);
                Tag typeTag = typeRaw.checked();
                TtlvValue typeValue = TtlvValue.Enumeration(1);
                TtlvItem typeItem = TtlvItem.create(typeTag, typeValue, limits)) {
            TtlvStructure withType = TtlvStructure.withItem(empty, typeItem, limits);
            try (withType;
                    RawTag valueRaw = RawTag.fromRaw(0x0042_001e);
                    Tag valueTag = valueRaw.checked();
                    TtlvValue certificateValue = TtlvValue.ByteString(value, limits);
                    TtlvItem valueItem = TtlvItem.create(valueTag, certificateValue, limits)) {
                return TtlvStructure.withItem(withType, valueItem, limits);
            }
        }
    }

    @Test
    void rekeyResponseUsesNativeJniBridgeAndRestrictsNonSuccessPayloads() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate serverCertificate = testCertificate(serverCertificatePem);
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);

        KeyStore serverStore = KeyStore.getInstance("PKCS12");
        serverStore.load(null, STORE_PASSWORD);
        serverStore.setKeyEntry("kmipkit-test-server", testPrivateKey(serverPrivateKeyPem),
                STORE_PASSWORD, new Certificate[] {serverCertificate});
        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, rekeyResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-rekey-response-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.RekeyResponse successful;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.asynchronousIndicator(1);
            batch.add(ClientBatchItem.rekey(UniqueIdentifier.textString("existing-success"),
                    null, null, null).withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.rekey(UniqueIdentifier.textString("existing-failure"),
                    null, null, null).withUniqueBatchItemId(uniqueBatchItemId(1)));
            batch.add(ClientBatchItem.rekey(UniqueIdentifier.textString("existing-pending"),
                    null, null, null).withUniqueBatchItemId(uniqueBatchItemId(2)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult success = response.itemAt(0);
                    ClientBatchItemResult failure = response.itemAt(1);
                    ClientBatchItemResult pending = response.itemAt(2)) {
                assertEquals(3, response.itemCount());
                assertEquals(0x04, success.operation());
                assertEquals(0, success.resultStatus());
                assertNativeReprovisionTtlvBridge(success, false);
                successful = success.rekeyResponse().orElseThrow();

                assertEquals(0x04, failure.operation());
                assertEquals(1, failure.resultStatus());
                assertEquals(5, failure.resultReason().orElseThrow());
                assertNativeReprovisionTtlvBridge(failure, false);
                assertTrue(failure.rekeyResponse().isEmpty());

                assertEquals(0x04, pending.operation());
                assertEquals(2, pending.resultStatus());
                assertTrue(pending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.rekeyResponse().isEmpty());
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        try (successful; TtlvStructureView responseTtlv = successful.responseTtlv().view()) {
            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, successful.uniqueIdentifier().kind());
            assertEquals("replacement-key", successful.uniqueIdentifier().text());
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertTagOrder(payloadStructure, 0x420094, 0x541234);
                assertTrue(containsByteString(payloadStructure, 0x541234,
                        new byte[] {0x01, 0x00, (byte) 0xff}));
            }
            assertFalse(successful.toString().contains("replacement-key"));
        }
        assertContains(request, enumeration(0x42005c, 0x04));
        assertEquals(3, countOccurrences(request, enumeration(0x42005c, 0x04)));
        assertTrue(peer.isDone());
    }

    @Test
    void rngRetrieveAndSeedResponsesCrossNativeJniBridge() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate serverCertificate = testCertificate(serverCertificatePem);
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);
        byte[] seed = {0x00, (byte) 0x81, (byte) 0xff};

        KeyStore serverStore = KeyStore.getInstance("PKCS12");
        serverStore.load(null, STORE_PASSWORD);
        serverStore.setKeyEntry("kmipkit-test-server", testPrivateKey(serverPrivateKeyPem),
                STORE_PASSWORD, new Certificate[] {serverCertificate});
        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, rngOperationResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-rng-response-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.RngRetrieveResponse retrieveResponse;
        ClientBatchItemResult.RngSeedResponse seedResponse;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.rngRetrieve(-17).withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.rngSeed(seed).withUniqueBatchItemId(uniqueBatchItemId(1)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult retrieve = response.itemAt(0);
                    ClientBatchItemResult seedResult = response.itemAt(1)) {
                assertEquals(2, response.itemCount());
                assertEquals(0x25, retrieve.operation());
                assertEquals(0, retrieve.resultStatus());
                retrieveResponse = retrieve.rngRetrieveResponse().orElseThrow();
                assertEquals(0x26, seedResult.operation());
                assertEquals(0, seedResult.resultStatus());
                seedResponse = seedResult.rngSeedResponse().orElseThrow();
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        try (retrieveResponse; seedResponse;
                TtlvStructureView retrieveTtlv = retrieveResponse.responseTtlv().orElseThrow().view();
                TtlvStructureView seedTtlv = seedResponse.responseTtlv().orElseThrow().view()) {
            byte[] returned = retrieveResponse.data().orElseThrow();
            try {
            assertTrue(Arrays.equals(
                    RNG_RETRIEVED_SENTINEL.getBytes(StandardCharsets.US_ASCII), returned),
                        "RNG Retrieve must preserve returned bytes");
            } finally {
                Arrays.fill(returned, (byte) 0);
            }
            assertFalse(retrieveResponse.toString().contains(RNG_RETRIEVED_SENTINEL));
            assertEquals(0, seedResponse.dataLength().orElseThrow());
            assertTagOrder(retrieveTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            assertTagOrder(seedTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            assertTrue(peer.isDone());
        }

        assertContains(request, enumeration(0x42005c, 0x25));
        assertContains(request, item(0x4200d5, 2, integer(-17)));
        assertContains(request, enumeration(0x42005c, 0x26));
        assertTrue(containsTtlvItem(request, item(0x4200c2, 8, seed)),
                "RNG Seed must preserve caller-supplied bytes");
    }

    @Test
    void rngFailureAndPendingResponsesCrossNativeJniBridgeWithoutTypedValues() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate serverCertificate = testCertificate(serverCertificatePem);
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);

        KeyStore serverStore = KeyStore.getInstance("PKCS12");
        serverStore.load(null, STORE_PASSWORD);
        serverStore.setKeyEntry("kmipkit-test-server", testPrivateKey(serverPrivateKeyPem),
                STORE_PASSWORD, new Certificate[] {serverCertificate});
        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, rngFailurePendingResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-rng-failure-pending-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.asynchronousIndicator(1);
            batch.add(ClientBatchItem.rngRetrieve(8).withUniqueBatchItemId(uniqueBatchItemId(0)));
            batch.add(ClientBatchItem.rngRetrieve(8).withUniqueBatchItemId(uniqueBatchItemId(1)));
            batch.add(ClientBatchItem.rngSeed(new byte[] {0x00, (byte) 0xff})
                    .withUniqueBatchItemId(uniqueBatchItemId(2)));
            batch.add(ClientBatchItem.rngSeed(new byte[] {0x01})
                    .withUniqueBatchItemId(uniqueBatchItemId(3)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult retrieveFailure = response.itemAt(0);
                    ClientBatchItemResult retrievePending = response.itemAt(1);
                    ClientBatchItemResult seedFailure = response.itemAt(2);
                    ClientBatchItemResult seedPending = response.itemAt(3)) {
                assertEquals(4, response.itemCount());
                assertEquals(0x25, retrieveFailure.operation());
                assertEquals(1, retrieveFailure.resultStatus());
                assertTrue(retrieveFailure.outcomeKind().isCompleted());
                assertEquals(5, retrieveFailure.resultReason().orElseThrow());
                assertTrue(retrieveFailure.rngRetrieveResponse().isEmpty());

                assertEquals(0x25, retrievePending.operation());
                assertEquals(2, retrievePending.resultStatus());
                assertTrue(retrievePending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION,
                        retrievePending.pendingCorrelationValue().orElseThrow());
                assertTrue(retrievePending.rngRetrieveResponse().isEmpty());

                assertEquals(0x26, seedFailure.operation());
                assertEquals(1, seedFailure.resultStatus());
                assertTrue(seedFailure.outcomeKind().isCompleted());
                assertEquals(5, seedFailure.resultReason().orElseThrow());
                assertTrue(seedFailure.rngSeedResponse().isEmpty());

                assertEquals(0x26, seedPending.operation());
                assertEquals(2, seedPending.resultStatus());
                assertTrue(seedPending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, seedPending.pendingCorrelationValue().orElseThrow());
                assertTrue(seedPending.rngSeedResponse().isEmpty());
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length > 0);
        } finally {
            listener.close();
        }
    }

    @Test
    void rekeyKeyPairResponseUsesNativeJniBridgeAndPreservesBothIdentifiers() throws Exception {
        byte[] serverCertificatePem = testFixture("server-cert.pem");
        byte[] serverPrivateKeyPem = testFixture("server-key.pem");
        byte[] clientCertificatePem = testFixture("client-cert.pem");
        byte[] clientPrivateKeyPem = testFixture("client-key.pem");
        X509Certificate serverCertificate = testCertificate(serverCertificatePem);
        X509Certificate clientCertificate = testCertificate(clientCertificatePem);

        KeyStore serverStore = KeyStore.getInstance("PKCS12");
        serverStore.load(null, STORE_PASSWORD);
        serverStore.setKeyEntry("kmipkit-test-server", testPrivateKey(serverPrivateKeyPem),
                STORE_PASSWORD, new Certificate[] {serverCertificate});
        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, rekeyKeyPairResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-rekey-key-pair-response-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        ClientBatchItemResult.RekeyKeyPairResponse successful;
        byte[] request;
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create();
                TtlvStructure createAttributes = TtlvStructure.create();
                ClientBatchItem create = ClientBatchItem.create(4, createAttributes, null)) {
            batch.asynchronousIndicator(1);
            batch.add(create.withUniqueBatchItemId(uniqueBatchItemId(0)));
            createAttributes.close();

            try (TtlvStructure commonAttributes = requestStructure(0x541101, "common-attributes");
                    TtlvStructure privateAttributes = requestStructure(0x541102, "private-attributes");
                    TtlvStructure publicAttributes = requestStructure(0x541103, "public-attributes");
                    TtlvStructure commonMasks = requestStructure(0x541104, "common-masks");
                    TtlvStructure privateMasks = requestStructure(0x541105, "private-masks");
                    TtlvStructure publicMasks = requestStructure(0x541106, "public-masks")) {
                batch.add(ClientBatchItem.rekeyKeyPair(null, 86_400L, commonAttributes,
                        privateAttributes, publicAttributes, commonMasks, privateMasks, publicMasks)
                        .withUniqueBatchItemId(uniqueBatchItemId(1)));
            }
            batch.add(ClientBatchItem.rekeyKeyPair(null, null, null, null, null, null, null, null)
                    .withUniqueBatchItemId(uniqueBatchItemId(2)));
            batch.add(ClientBatchItem.rekeyKeyPair(null, null, null, null, null, null, null, null)
                    .withUniqueBatchItemId(uniqueBatchItemId(3)));

            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult created = response.itemAt(0);
                    ClientBatchItemResult success = response.itemAt(1);
                    ClientBatchItemResult failure = response.itemAt(2);
                    ClientBatchItemResult pending = response.itemAt(3)) {
                assertEquals(4, response.itemCount());
                assertEquals(1, created.operation());
                assertEquals(0, created.resultStatus());
                assertEquals(0x1d, success.operation());
                assertEquals(0, success.resultStatus());
                assertTrue(success.outcomeKind().isCompleted());
                successful = success.rekeyKeyPairResponse().orElseThrow();
                assertEquals("replacement-private", successful.privateKeyUniqueIdentifier().text());
                assertEquals("replacement-public", successful.publicKeyUniqueIdentifier().text());

                assertEquals(0x1d, failure.operation());
                assertEquals(1, failure.resultStatus());
                assertTrue(failure.outcomeKind().isCompleted());
                assertEquals(5, failure.resultReason().orElseThrow());
                assertTrue(failure.rekeyKeyPairResponse().isEmpty());

                assertEquals(0x1d, pending.operation());
                assertEquals(2, pending.resultStatus());
                assertTrue(pending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.rekeyKeyPairResponse().isEmpty());
            }
            request = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }

        assertRekeyKeyPairRequest(request);
        try (successful; TtlvStructureView responseTtlv = successful.responseTtlv().view()) {
            assertEquals(4, responseTtlv.itemCount());
            assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
            try (TtlvItemView payload = responseTtlv.itemAt(3);
                    TtlvValueView payloadValue = payload.value();
                    TtlvStructureView payloadStructure = payloadValue.structure()) {
                assertEquals(3, payloadStructure.itemCount());
                assertTagOrder(payloadStructure, 0x420066, 0x42006f, 0x541234);
                assertTrue(containsByteString(payloadStructure, 0x541234,
                        new byte[] {0x01, 0x00, (byte) 0xff}));
            }
            assertFalse(successful.toString().contains("replacement-private"));
            assertFalse(successful.toString().contains("replacement-public"));
        }
        assertContains(request, enumeration(0x42005c, 0x1d));
        assertEquals(3, countOccurrences(request, enumeration(0x42005c, 0x1d)));
        assertEquals(0, countOccurrences(request, new byte[] {0x42, 0x00, 0x66}));
        assertTrue(peer.isDone());
    }

    @Test
    void rekeyKeyPairPeerClosePreservesPossiblySentDeliveryState() throws Exception {
        Path serverStorePath = temporaryDirectory.resolve("rekey-pair-close-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("rekey-pair-close-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = peerThatClosesAfterOneRequest(listener);
        Thread peerThread = new Thread(peer, "kmipkit-java-rekey-key-pair-close-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.rekeyKeyPair(UniqueIdentifier.textString("delivery-key"),
                    null, null, null, null, null, null, null));

            TransportException failure = assertThrows(
                    TransportException.class, () -> client.execute(batch, CodecLimits.defaults()));
            assertEquals(9, failure.statusCode());
            assertEquals(3, failure.categoryCode());
            assertEquals(2, failure.deliveryStateCode());
            byte[] request = peer.get(12, TimeUnit.SECONDS);
            assertContains(request, enumeration(0x42005c, 0x1d));
            assertContains(request, item(0x420066, 7, "delivery-key".getBytes(StandardCharsets.US_ASCII)));
        } finally {
            listener.close();
        }
    }

    @Test
    void obtainLeaseResponseUsesNativeJniBridgeAndRetainsOwnedTtlvOverTls() throws Exception {
        Path serverStorePath = temporaryDirectory.resolve("obtain-lease-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("obtain-lease-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, obtainLeaseResponseMessage()));
        Thread peerThread = new Thread(peer, "kmipkit-java-obtain-lease-response-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.obtainLease(UniqueIdentifier.textString("lease-request-id"))
                    .withUniqueBatchItemId(uniqueBatchItemId(0)));

            ClientBatchItemResult.ObtainLeaseResponse lease;
            try (ClientBatchResponse response = client.execute(batch, CodecLimits.defaults());
                    ClientBatchItemResult item = response.itemAt(0)) {
                assertEquals(1, response.itemCount());
                assertEquals(0x10, item.operation());
                assertEquals(0, item.resultStatus());
                lease = item.obtainLeaseResponse().orElseThrow();
            }

            try (lease; TtlvStructureView responseTtlv = lease.responseTtlv().view()) {
                assertEquals(UniqueIdentifier.Kind.TEXT_STRING, lease.uniqueIdentifier().kind());
                assertEquals("leased-object", lease.uniqueIdentifier().text());
                assertEquals(0, lease.leaseTime());
                assertEquals(-7, lease.lastChangeDate());
                assertTagOrder(responseTtlv, 0x42005c, 0x420093, 0x42007f, 0x42007c);
                try (TtlvItemView payload = responseTtlv.itemAt(3);
                        TtlvValueView payloadValue = payload.value();
                        TtlvStructureView payloadStructure = payloadValue.structure()) {
                    assertTagOrder(payloadStructure, 0x420094, 0x420049, 0x420048, 0x541234);
                    assertTrue(containsByteString(payloadStructure, 0x541234,
                            new byte[] {0x01, 0x00, (byte) 0xff}));
                }
                assertFalse(lease.toString().contains("leased-object"));
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    // OASIS KMIP Specification v2.1 §6.1.28, Tables 248–249.
    @Test
    void locateResponseRejectsMoreThanOneHundredThousandIdentifiersAtJavaBoundaryOverTls()
            throws Exception {
        int identifierCount = 100_001;
        Path serverStorePath = temporaryDirectory.resolve("large-locate-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("large-locate-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = new FutureTask<>(
                () -> serveOneResponse(listener, locateResponseMessage(identifierCount)));
        Thread peerThread = new Thread(peer, "kmipkit-java-large-locate-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        CodecLimits limits = CodecLimits.create(16 * 1024 * 1024L, 64, 400_000);
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create();
                TtlvStructure attributes = TtlvStructure.create()) {
            batch.add(ClientBatchItem.locate(null, null, null, null, attributes));
            try (ClientBatchResponse response = client.execute(batch, limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertTrue(result.uniqueBatchItemId().isEmpty());
                assertThrows(ResourceLimitException.class, result::locateResponse);
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    @Test
    void peerClosingAfterRequestSurfacesSanitizedTypedTransportFailure() throws Exception {
        Path serverStorePath = temporaryDirectory.resolve("transport-error-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("transport-error-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        FutureTask<byte[]> peer = peerThatClosesAfterOneRequest(listener);
        Thread peerThread = new Thread(peer, "kmipkit-java-tls-transport-error-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.discoverVersions());

            TransportException failure = assertThrows(
                    TransportException.class, () -> client.execute(batch, CodecLimits.defaults()));

            assertEquals(9, failure.statusCode());
            assertEquals(3, failure.categoryCode());
            assertEquals(2, failure.deliveryStateCode());
            assertFalse(failure.getMessage().contains("transport-error-server.p12"));
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
        } finally {
            listener.close();
        }
    }

    @Test
    void malformedTtlvResponseSurfacesSanitizedTypedProtocolFailure() throws Exception {
        Path serverStorePath = temporaryDirectory.resolve("protocol-error-server.p12");
        Path clientStorePath = temporaryDirectory.resolve("protocol-error-client.p12");
        createKeyStore(serverStorePath, "server");
        createKeyStore(clientStorePath, "client");

        KeyStore serverStore = loadKeyStore(serverStorePath);
        KeyStore clientStore = loadKeyStore(clientStorePath);
        X509Certificate serverCertificate = certificate(serverStore, "server");
        X509Certificate clientCertificate = certificate(clientStore, "client");
        PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        byte[] malformedTtlv = {0x42, 0x00, 0x7b, 1, 0, 0, 0, 0};
        FutureTask<byte[]> peer = peerThatRespondsAfterOneRequest(listener, malformedTtlv);
        Thread peerThread = new Thread(peer, "kmipkit-java-tls-protocol-error-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration extensions = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                        .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(extensions, transport);
                ClientBatch batch = ClientBatch.create()) {
            batch.add(ClientBatchItem.discoverVersions());

            ClientOperationException failure = assertThrows(
                    ClientOperationException.class, () -> client.execute(batch, CodecLimits.defaults()));
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);

            assertEquals(8, failure.statusCode());
            assertTrue(failure instanceof ProtocolException);
            assertTrue(((ProtocolException) failure).responseTtlv().isEmpty());
            assertEquals(2, failure.categoryCode());
            assertEquals(3, failure.deliveryStateCode());
            assertFalse(failure.getMessage().contains("protocol-error-server.p12"));
        } finally {
            listener.close();
        }
    }

    @Test
    void typedResponseShapeFailureRetainsGenericTtlvForDefaultAndOptionsExecution() throws Exception {
        for (boolean withOptions : new boolean[] {false, true}) {
            Path serverStorePath = temporaryDirectory.resolve(
                    withOptions ? "shape-options-server.p12" : "shape-default-server.p12");
            Path clientStorePath = temporaryDirectory.resolve(
                    withOptions ? "shape-options-client.p12" : "shape-default-client.p12");
            createKeyStore(serverStorePath, "server");
            createKeyStore(clientStorePath, "client");

            KeyStore serverStore = loadKeyStore(serverStorePath);
            KeyStore clientStore = loadKeyStore(clientStorePath);
            X509Certificate serverCertificate = certificate(serverStore, "server");
            X509Certificate clientCertificate = certificate(clientStore, "client");
            PrivateKey clientKey = (PrivateKey) clientStore.getKey("client", STORE_PASSWORD);

            SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                    .getServerSocketFactory()
                    .createServerSocket(0, 1, InetAddress.getByName("127.0.0.1"));
            listener.setEnabledProtocols(new String[] {"TLSv1.3"});
            listener.setNeedClientAuth(true);
            listener.setSoTimeout(10_000);
            byte[] response = macResponseMissingUniqueIdentifierWithSentinel();
            FutureTask<byte[]> peer = peerThatRespondsAfterOneRequest(listener, response);
            Thread peerThread = new Thread(peer, "kmipkit-java-shape-error-fixture");
            peerThread.setDaemon(true);
            peerThread.start();

            try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(), ExtensionRegistryLimits.defaults());
                    ClientConfiguration extensions = ClientConfiguration.create(registry);
                    TransportConfiguration transport = TransportConfiguration.builder()
                            .rawTls("127.0.0.1", listener.getLocalPort())
                            .clientIdentityDer(new byte[][] {clientCertificate.getEncoded()}, clientKey.getEncoded())
                            .trustCertificatesDer(new byte[][] {serverCertificate.getEncoded()})
                            .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                    TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                    TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                    TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                            .build();
                    Client client = Client.create(extensions, transport);
                    ClientBatch batch = ClientBatch.create()) {
                CodecLimits limits = CodecLimits.defaults();
                try (TtlvValue macData = TtlvValue.ByteString(new byte[] {0x41, 0x42}, limits)) {
                    batch.add(ClientBatchItem.mac(null, null, macData, null, null, null)
                            .withUniqueBatchItemId(uniqueBatchItemId(0)));
                }

                ProtocolException failure = assertThrows(ProtocolException.class, () -> {
                    if (withOptions) {
                        client.executeWithOptions(batch, limits, RequestOptions.defaults());
                    } else {
                        client.execute(batch, limits);
                    }
                });
                assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
                assertFalse(failure.getMessage().contains("KMIPKit-response-sentinel"));
                assertFalse(failure.toString().contains("KMIPKit-response-sentinel"));

                byte[] sentinel = "KMIPKit-response-sentinel".getBytes(StandardCharsets.US_ASCII);
                try (TtlvStructure retained = failure.responseTtlv().orElseThrow();
                        TtlvStructureView retainedView = retained.view()) {
                    assertTrue(containsByteString(retainedView, 0x540001, sentinel));
                }
            } finally {
                listener.close();
            }
        }
    }

    private static byte[] macResponseMissingUniqueIdentifierWithSentinel() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] sentinel = "KMIPKit-response-sentinel".getBytes(StandardCharsets.US_ASCII);
        byte[] payload = structure(0x42007c,
                item(0x4200c4, 8, new byte[] {0x5a}), item(0x540001, 8, sentinel));
        byte[] responseItem = structure(0x42000f, item(0x42005c, 5, integer(35)),
                item(0x420093, 8, uniqueBatchItemId(0)), item(0x42007f, 5, integer(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static boolean containsByteString(
            TtlvStructureView structure, int rawTag, byte[] expected) {
        for (long index = 0; index < structure.itemCount(); index++) {
            try (TtlvItemView item = structure.itemAt(index);
                    Tag tag = item.tag()) {
                if (tag.raw() == rawTag && item.itemType() == TtlvItemType.ByteString) {
                    try (TtlvValueView value = item.value()) {
                        if (value.byteLength() == expected.length) {
                            boolean matches = true;
                            for (int byteIndex = 0; byteIndex < expected.length; byteIndex++) {
                                if ((byte) value.byteAt(byteIndex) != expected[byteIndex]) {
                                    matches = false;
                                    break;
                                }
                            }
                            if (matches) return true;
                        }
                    }
                }
                if (item.itemType() == TtlvItemType.Structure) {
                    try (TtlvValueView value = item.value();
                            TtlvStructureView nested = value.structure()) {
                        if (containsByteString(nested, rawTag, expected)) return true;
                    }
                }
            }
        }
        return false;
    }

    private static boolean containsOrderedDeriveKeyIdentifiersInBatchItem(
            byte[] bytes, byte[]... identifiers) throws Exception {
        TtlvSpan message = parseItem(bytes, 0, bytes.length);
        if (message.end() != bytes.length || message.tag() != 0x420078 || message.type() != 1) {
            return false;
        }

        for (TtlvSpan batchItem : children(bytes, message)) {
            if (batchItem.tag() != 0x42000f || batchItem.type() != 1) continue;
            boolean isDeriveKey = false;
            TtlvSpan requestPayload = null;
            for (TtlvSpan field : children(bytes, batchItem)) {
                if (field.tag() == 0x42005c && field.type() == 5 && field.valueLength() == 4) {
                    isDeriveKey = ByteBuffer.wrap(bytes, field.valueStart(), field.valueLength())
                            .getInt() == 5;
                } else if (field.tag() == 0x420079 && field.type() == 1) {
                    requestPayload = field;
                }
            }
            if (!isDeriveKey || requestPayload == null) continue;

            List<byte[]> actualIdentifiers = new ArrayList<>();
            for (TtlvSpan field : children(bytes, requestPayload)) {
                if (field.tag() == 0x420094) {
                    actualIdentifiers.add(Arrays.copyOfRange(bytes, field.offset(), field.end()));
                }
            }
            if (actualIdentifiers.size() != identifiers.length) continue;
            boolean exactMatch = true;
            for (int index = 0; index < identifiers.length; index++) {
                if (!Arrays.equals(actualIdentifiers.get(index), identifiers[index])) {
                    exactMatch = false;
                    break;
                }
            }
            if (exactMatch) return true;
        }
        return false;
    }

    private static void assertRekeyKeyPairRequest(byte[] bytes) {
        TtlvSpan message = parseItem(bytes, 0, bytes.length);
        assertEquals(0x420078, message.tag());
        assertEquals(1, message.type());
        assertEquals(bytes.length, message.end());

        TtlvSpan requestPayload = null;
        for (TtlvSpan batchItem : children(bytes, message)) {
            if (batchItem.tag() != 0x42000f || batchItem.type() != 1) continue;
            boolean isRekeyKeyPair = false;
            TtlvSpan candidatePayload = null;
            for (TtlvSpan field : children(bytes, batchItem)) {
                if (field.tag() == 0x42005c && field.type() == 5 && field.valueLength() == 4) {
                    isRekeyKeyPair = ByteBuffer.wrap(bytes, field.valueStart(), field.valueLength())
                            .getInt() == 0x1d;
                } else if (field.tag() == 0x420079 && field.type() == 1) {
                    candidatePayload = field;
                }
            }
            if (isRekeyKeyPair) {
                requestPayload = candidatePayload;
                break;
            }
        }
        assertNotNull(requestPayload, "request did not contain a Re-key Key Pair payload");

        List<TtlvSpan> fields = children(bytes, requestPayload);
        int[] expectedTags = {0x420058, 0x420126, 0x420127, 0x420128, 0x420163, 0x420164, 0x420165};
        assertEquals(expectedTags.length, fields.size());
        for (int index = 0; index < expectedTags.length; index++) {
            assertEquals(expectedTags[index], fields.get(index).tag(),
                    "Table 310 request field order at index " + index);
        }
        TtlvSpan offset = fields.get(0);
        assertEquals(10, offset.type());
        assertEquals(4, offset.valueLength());
        assertEquals(86_400, ByteBuffer.wrap(bytes, offset.valueStart(), offset.valueLength()).getInt());

        int[] markerTags = {0x541101, 0x541102, 0x541103, 0x541104, 0x541105, 0x541106};
        byte[][] markers = {
            "common-attributes".getBytes(StandardCharsets.US_ASCII),
            "private-attributes".getBytes(StandardCharsets.US_ASCII),
            "public-attributes".getBytes(StandardCharsets.US_ASCII),
            "common-masks".getBytes(StandardCharsets.US_ASCII),
            "private-masks".getBytes(StandardCharsets.US_ASCII),
            "public-masks".getBytes(StandardCharsets.US_ASCII)
        };
        for (int index = 0; index < markerTags.length; index++) {
            TtlvSpan group = fields.get(index + 1);
            assertEquals(1, group.type());
            List<TtlvSpan> children = children(bytes, group);
            assertEquals(1, children.size());
            TtlvSpan marker = children.get(0);
            assertEquals(markerTags[index], marker.tag(), "request field group marker tag");
            assertEquals(8, marker.type());
            assertArrayEquals(markers[index], Arrays.copyOfRange(bytes, marker.valueStart(),
                    marker.valueStart() + marker.valueLength()));
        }
    }

    private static TtlvStructure requestStructure(int markerTag, String marker) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure empty = TtlvStructure.create();
        try (RawTag rawTag = RawTag.fromRaw(markerTag);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.ByteString(marker.getBytes(StandardCharsets.US_ASCII), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(empty, item, limits);
        } finally {
            empty.close();
        }
    }

    private static TtlvItem registerOpaqueObject() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure fields = TtlvStructure.create();
        TtlvStructure withType;
        try (fields) {
            withType = TtlvStructure.withItem(
                    fields,
                    registerTtlvItem(
                            0x420059, TtlvValue.Enumeration(0x80000001L), limits),
                    limits);
        }
        TtlvStructure withValue;
        try (withType) {
            withValue = TtlvStructure.withItem(
                    withType,
                    registerTtlvItem(
                            0x42005A,
                            TtlvValue.ByteString(
                                    "register-object-fixture".getBytes(StandardCharsets.US_ASCII),
                                    limits),
                            limits),
                    limits);
        }
        try (withValue) {
            return registerTtlvItem(
                    0x42005B, TtlvValue.structure(withValue, limits), limits);
        }
    }

    private static TtlvItem registerTtlvItem(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static List<TtlvSpan> children(byte[] bytes, TtlvSpan parent) {
        if (parent.type() != 1) throw new IllegalArgumentException("TTLV parent must be a Structure");
        int end = parent.valueStart() + parent.valueLength();
        List<TtlvSpan> children = new ArrayList<>();
        int offset = parent.valueStart();
        while (offset < end) {
            TtlvSpan child = parseItem(bytes, offset, end);
            children.add(child);
            offset = child.end();
        }
        if (offset != end) throw new IllegalArgumentException("TTLV Structure has invalid child boundaries");
        return children;
    }

    private static TtlvSpan parseItem(byte[] bytes, int offset, int parentEnd) {
        if (offset < 0 || parentEnd > bytes.length || offset > parentEnd - 8) {
            throw new IllegalArgumentException("TTLV item header is truncated");
        }
        int tag = ((bytes[offset] & 0xff) << 16)
                | ((bytes[offset + 1] & 0xff) << 8)
                | (bytes[offset + 2] & 0xff);
        int type = bytes[offset + 3] & 0xff;
        int valueLength = ByteBuffer.wrap(bytes, offset + 4, 4).getInt();
        int valueStart = offset + 8;
        if (valueLength < 0 || valueLength > parentEnd - valueStart) {
            throw new IllegalArgumentException("TTLV item value exceeds its parent");
        }
        long paddedLength = ((long) valueLength + 7L) & ~7L;
        long itemEnd = (long) valueStart + paddedLength;
        if (itemEnd > parentEnd) throw new IllegalArgumentException("TTLV item padding exceeds its parent");
        return new TtlvSpan(tag, type, offset, valueStart, valueLength, (int) itemEnd);
    }

    private record TtlvSpan(int tag, int type, int offset, int valueStart, int valueLength, int end) {}

    private static long tagAt(TtlvStructureView structure, long index) {
        try (TtlvItemView item = structure.itemAt(index); Tag tag = item.tag()) {
            return tag.raw();
        }
    }

    private static void assertTagOrder(TtlvStructureView structure, long... expectedTags) {
        assertEquals(expectedTags.length, structure.itemCount());
        for (int index = 0; index < expectedTags.length; index++) {
            assertEquals(expectedTags[index], tagAt(structure, index));
        }
    }

    private static TtlvStructure deriveKeyAttributes() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure attributes = TtlvStructure.create();
        try (Tag tag = RawTag.fromRaw(0x42002a).checked();
                TtlvValue length = TtlvValue.Integer(128);
                TtlvItem item = TtlvItem.create(tag, length, limits)) {
            return TtlvStructure.withItem(attributes, item, limits);
        }
    }

    private static TtlvStructure importAnyObjectStructure() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure body = TtlvStructure.create();
        TtlvStructure object = TtlvStructure.create();
        try (Tag tag = RawTag.fromRaw(0x42008f).checked();
                TtlvValue value = TtlvValue.structure(body, limits);
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(object, item, limits);
        } finally {
            object.close();
            body.close();
        }
    }

    private static List<?> cryptoOperations() throws Exception {
        Field operations = CryptoOperationsJavaFacadeTest.class.getDeclaredField("OPERATIONS");
        if (!operations.trySetAccessible()) {
            throw new AssertionError("crypto operation table must remain accessible to TLS integration tests");
        }
        return (List<?>) operations.get(null);
    }

    private static void assertNativeRequestOptionsRejected(Client client, long[] requestTimeouts)
            throws ReflectiveOperationException {
        Field clientHandleField = Client.class.getDeclaredField("handle");
        if (!clientHandleField.trySetAccessible()) {
            throw new AssertionError("client handle must remain accessible to JNI boundary tests");
        }
        NativeHandle handle = (NativeHandle) clientHandleField.get(client);
        long nativeBatch = NativeExtensionRegistry.clientBatchCreate();
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientExecuteWithOptions(
                handle.get(), nativeBatch, NativeExtensionRegistry.codecLimitValues(CodecLimits.defaults()),
                requestTimeouts));
    }

    private static void assertNativeReprovisionTtlvBridge(
            ClientBatchItemResult result, boolean expectedPresent)
            throws ReflectiveOperationException {
        Field resultHandleField = ClientBatchItemResult.class.getDeclaredField("handle");
        if (!resultHandleField.trySetAccessible()) {
            throw new AssertionError(
                    "batch item result handle must remain accessible to JNI bridge tests");
        }
        NativeHandle handle = (NativeHandle) resultHandleField.get(result);
        Object[] tuple = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseReprovisionTtlv);
        assertEquals(2, tuple.length);
        long[] fields = (long[]) tuple[0];
        long[] ttlvHandle = (long[]) tuple[1];
        try {
            assertEquals(expectedPresent ? 1 : 0, fields[0]);
            if (expectedPresent) {
                assertTrue(ttlvHandle[0] > 0);
            } else {
                assertEquals(0, ttlvHandle[0]);
            }
        } finally {
            if (ttlvHandle[0] > 0) TtlvStructure.fromNative(ttlvHandle[0], null).close();
        }
    }

    private static ClientBatchItem cryptoOperationItem(Object operation) throws Exception {
        String operationName = cryptoOperationName(operation);
        byte[] previousPartCorrelation = new byte[] {0x41, 0x00, (byte) 0xf2};
        if ("macVerify".equals(operationName)) {
            return ClientBatchItem.macVerify(null, null, null, null,
                    previousPartCorrelation, Boolean.FALSE, Boolean.TRUE);
        }
        if ("signatureVerify".equals(operationName)) {
            return ClientBatchItem.signatureVerify(null, null, null, null, null,
                    previousPartCorrelation, Boolean.FALSE, Boolean.TRUE);
        }
        Class<?> valueKindType = Class.forName(
                CryptoOperationsJavaFacadeTest.class.getName() + "$TtlvValueKind");
        Object byteStringKind = null;
        for (Object constant : valueKindType.getEnumConstants()) {
            if ("BYTE_STRING".equals(constant.toString())) {
                byteStringKind = constant;
                break;
            }
        }
        if (byteStringKind == null) {
            throw new AssertionError("crypto operation input fixture must expose BYTE_STRING");
        }
        Method factory = CryptoOperationsJavaFacadeTest.class.getDeclaredMethod(
                "newItem", operation.getClass(), valueKindType, boolean.class, boolean.class);
        if (!factory.trySetAccessible()) {
            throw new AssertionError("crypto operation item fixture must remain accessible to TLS tests");
        }
        return (ClientBatchItem) invokeCryptoFixture(null, factory,
                operation, byteStringKind, false, true);
    }

    private static byte[] cryptoOperationResponseMessage(List<?> operations) throws Exception {
        return cryptoOperationResponseMessage(operations, true);
    }

    private static byte[] cryptoOperationResponseMessage(
            List<?> operations, boolean includeOptionalResponseFields) throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(operations.size())));
        CryptoOperationsJavaFacadeTest fixture = new CryptoOperationsJavaFacadeTest();
        List<byte[]> responseItems = new ArrayList<>();
        for (int index = 0; index < operations.size(); index++) {
            Object operation = operations.get(index);
            Method successResponse = CryptoOperationsJavaFacadeTest.class.getDeclaredMethod(
                    "successResponse", operation.getClass(), int.class, boolean.class, boolean.class);
            if (!successResponse.trySetAccessible()) {
                throw new AssertionError("crypto success response fixture must remain accessible to TLS tests");
            }
            responseItems.add((byte[]) invokeCryptoFixture(
                    fixture, successResponse, operation, index, false, includeOptionalResponseFields));
        }
        List<byte[]> messageItems = new ArrayList<>();
        messageItems.add(header);
        messageItems.addAll(responseItems);
        return structure(0x42007b, messageItems.toArray(byte[][]::new));
    }

    private static String cryptoOperationName(Object operation) throws Exception {
        return (String) invokeCryptoFixture(operation, "name", new Class<?>[0]);
    }

    private static Object cryptoTypedResponse(String operationName, ClientBatchItemResult result) {
        return switch (operationName) {
            case "encrypt" -> result.encryptResponse().orElseThrow();
            case "decrypt" -> result.decryptResponse().orElseThrow();
            case "hash" -> result.hashResponse().orElseThrow();
            case "mac" -> result.macResponse().orElseThrow();
            case "macVerify" -> result.macVerifyResponse().orElseThrow();
            case "sign" -> result.signResponse().orElseThrow();
            case "signatureVerify" -> result.signatureVerifyResponse().orElseThrow();
            default -> throw new IllegalArgumentException("unknown crypto operation: " + operationName);
        };
    }

    private static Object invokeCryptoFixture(Object receiver, String name, Class<?>[] parameterTypes,
            Object... arguments) throws Exception {
        Class<?> owner = receiver == null ? CryptoOperationsJavaFacadeTest.class : receiver.getClass();
        Method method = owner.getDeclaredMethod(name, parameterTypes);
        return invokeCryptoFixture(receiver, method, arguments);
    }

    private static Object invokeCryptoFixture(Object receiver, Method method, Object... arguments)
            throws Exception {
        if (!method.trySetAccessible()) {
            throw new AssertionError("crypto operation fixture must remain accessible to TLS integration tests");
        }
        try {
            return method.invoke(receiver, arguments);
        } catch (InvocationTargetException failure) {
            Throwable cause = failure.getCause();
            if (cause instanceof Exception exception) {
                throw exception;
            }
            if (cause instanceof Error error) {
                throw error;
            }
            throw new AssertionError("crypto operation fixture failed", cause);
        }
    }

    private static void createKeyStore(Path path, String alias) throws Exception {
        String executableName = System.getProperty("os.name").startsWith("Windows")
                ? "keytool.exe" : "keytool";
        Path keytool = Path.of(System.getProperty("java.home"), "bin", executableName);
        Process process = new ProcessBuilder(keytool.toString(), "-genkeypair", "-noprompt",
                "-alias", alias, "-keyalg", "RSA", "-keysize", "2048", "-validity", "365",
                "-dname", "CN=localhost", "-ext", "SAN=IP:127.0.0.1,DNS:localhost",
                "-storetype", "PKCS12", "-keystore", path.toString(),
                "-storepass", new String(STORE_PASSWORD), "-keypass", new String(STORE_PASSWORD))
                .redirectErrorStream(true)
                .redirectOutput(ProcessBuilder.Redirect.DISCARD)
                .start();
        assertTrue(process.waitFor(30, TimeUnit.SECONDS), "keytool must finish creating the fixture");
        assertEquals(0, process.exitValue(), "keytool must create the local TLS fixture");
    }

    private static byte[] testFixture(String filename) throws Exception {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IllegalStateException("shared deterministic mTLS fixture is missing");
    }

    private static SSLServerSocket deterministicMtlsListener(
            byte[] serverCertificatePem, byte[] serverPrivateKeyPem, X509Certificate clientCertificate)
            throws Exception {
        X509Certificate serverCertificate = testCertificate(serverCertificatePem);
        KeyStore serverStore = KeyStore.getInstance("PKCS12");
        serverStore.load(null, STORE_PASSWORD);
        serverStore.setKeyEntry("kmipkit-large-response-server", testPrivateKey(serverPrivateKeyPem),
                STORE_PASSWORD, new Certificate[] {serverCertificate});
        SSLServerSocket listener = (SSLServerSocket) serverContext(serverStore, clientCertificate)
                .getServerSocketFactory()
                .createServerSocket(0, 2, InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(30_000);
        return listener;
    }

    private static X509Certificate testCertificate(byte[] pem) throws Exception {
        return (X509Certificate) CertificateFactory.getInstance("X.509")
                .generateCertificate(new java.io.ByteArrayInputStream(pem));
    }

    private static PrivateKey testPrivateKey(byte[] pem) throws Exception {
        String encoded = new String(pem, StandardCharsets.US_ASCII)
                .replace("-----BEGIN PRIVATE KEY-----", "")
                .replace("-----END PRIVATE KEY-----", "")
                .replaceAll("\\s", "");
        byte[] der = Base64.getDecoder().decode(encoded);
        return KeyFactory.getInstance("RSA").generatePrivate(new PKCS8EncodedKeySpec(der));
    }

    private static KeyStore loadKeyStore(Path path) throws Exception {
        KeyStore store = KeyStore.getInstance("PKCS12");
        try (InputStream input = Files.newInputStream(path)) {
            store.load(input, STORE_PASSWORD);
        }
        return store;
    }

    private static X509Certificate certificate(KeyStore store, String alias) throws Exception {
        Certificate value = store.getCertificate(alias);
        return (X509Certificate) CertificateFactory.getInstance("X.509")
                .generateCertificate(new java.io.ByteArrayInputStream(value.getEncoded()));
    }

    private static byte[] pem(String label, byte[] der) {
        String encoded = Base64.getMimeEncoder(64, new byte[] {'\n'}).encodeToString(der);
        return ("-----BEGIN " + label + "-----\n" + encoded + "\n-----END " + label + "-----\n")
                .getBytes(java.nio.charset.StandardCharsets.US_ASCII);
    }

    private static byte[] testCrlDer() throws Exception {
        try (InputStream input = ClientFacadeTlsIntegrationTest.class
                .getResourceAsStream("/t076-test-crl.der")) {
            if (input == null) {
                throw new IllegalStateException("test CRL fixture is missing");
            }
            return input.readAllBytes();
        }
    }

    private static SSLContext serverContext(KeyStore serverStore, X509Certificate clientCertificate)
            throws Exception {
        KeyManagerFactory keys = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        keys.init(serverStore, STORE_PASSWORD);
        KeyStore trust = KeyStore.getInstance("PKCS12");
        trust.load(null, STORE_PASSWORD);
        trust.setCertificateEntry("client", clientCertificate);
        TrustManagerFactory trustManagers = TrustManagerFactory.getInstance(
                TrustManagerFactory.getDefaultAlgorithm());
        trustManagers.init(trust);
        SSLContext context = SSLContext.getInstance("TLSv1.3");
        context.init(keys.getKeyManagers(), trustManagers.getTrustManagers(), new SecureRandom());
        return context;
    }

    private static byte[] serveOneResponse(SSLServerSocket listener) throws Exception {
        return serveOneResponse(listener, mixedResponseMessage());
    }

    private static byte[] serveOneResponse(SSLServerSocket listener, byte[] response) throws Exception {
        return serveOneResponse(listener, response, false);
    }

    private static byte[] serveLifecycleResponseWithoutFollowUp(
            SSLServerSocket listener, byte[] response) throws Exception {
        return serveOneResponse(listener, response, true);
    }

    private static byte[] serveOneResponse(
            SSLServerSocket listener, byte[] response, boolean assertNoFollowUp) throws Exception {
        try (SSLSocket socket = (SSLSocket) listener.accept()) {
            socket.setEnabledProtocols(new String[] {"TLSv1.3"});
            socket.setSoTimeout(10_000);
            socket.startHandshake();
            byte[] request = readFrame(socket.getInputStream());
            OutputStream output = socket.getOutputStream();
            output.write(response);
            output.flush();

            if (assertNoFollowUp) {
                socket.setSoTimeout(250);
                try {
                    int followUp = socket.getInputStream().read();
                    if (followUp >= 0) {
                        throw new AssertionError("unexpected lifecycle follow-up request");
                    }
                } catch (SocketTimeoutException expected) {
                    // No bytes arrived on the original TLS connection after its response.
                }
            }
            return request;
        }
    }

    private static List<byte[]> serveResponses(SSLServerSocket listener, byte[]... responses)
            throws Exception {
        List<byte[]> requests = new ArrayList<>(responses.length);
        for (byte[] response : responses) {
            requests.add(serveOneResponse(listener, response));
        }
        return List.copyOf(requests);
    }

    private static byte[] usageAllocationResponseMessage(byte[] itemId) throws Exception {
        return usageAllocationResponseMessage(itemId, "allocation-object");
    }

    private static byte[] pkcs11ResponseMessage(String fixture, String correlationCase)
            throws Exception {
        String expectedOutput = fixtureObject(fixture, "expectedOutput");
        String typedResponse = fixtureObject(expectedOutput, "typedResponse");
        String function = fixtureObject(typedResponse, "function");
        String correlationValue = fixtureObject(typedResponse, "correlationValue");
        String outputParameters = fixtureObject(typedResponse, "outputParameters");
        String returnCode = fixtureObject(typedResponse, "returnCode");
        String vendorExtension = fixtureObject(expectedOutput, "vendorExtension");
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] functionField = structure(Math.toIntExact(fixtureHexField(function, "tag")),
                item(Math.toIntExact(fixtureHexField(function, "childTag")), 2,
                        integer(Math.toIntExact(fixtureLongField(function, "childValue")))));
        long correlationTag = fixtureHexField(correlationValue, "tag");
        byte[] correlationBytes = switch (correlationCase) {
            case "nonempty" -> fixtureHexBytes(correlationValue, "hex");
            case "presentEmpty" -> new byte[0];
            case "absent" -> null;
            default -> throw new AssertionError("unsupported correlation presence case");
        };
        byte[] correlationField = correlationBytes == null ? null
                : item(Math.toIntExact(correlationTag), 8, correlationBytes);
        byte[] outputField = structure(Math.toIntExact(fixtureHexField(outputParameters, "tag")),
                item(Math.toIntExact(fixtureHexField(outputParameters, "childTag")), 8,
                        fixtureHexBytes(outputParameters, "childHex")));
        byte[] returnCodeField = structure(Math.toIntExact(fixtureHexField(returnCode, "tag")),
                enumeration(Math.toIntExact(fixtureHexField(returnCode, "childTag")),
                        (int) fixtureHexField(returnCode, "childValue")));
        byte[] vendorField = item(Math.toIntExact(fixtureHexField(vendorExtension, "tag")), 2,
                integer(Math.toIntExact(fixtureLongField(vendorExtension, "integer"))));
        List<byte[]> responseFields = new ArrayList<>();
        for (long tag : fixtureLongArray(expectedOutput, "responseTags")) {
            if (correlationCase.equals("absent") && tag == correlationTag) continue;
            if (tag == fixtureHexField(function, "tag")) responseFields.add(functionField);
            else if (tag == correlationTag) responseFields.add(correlationField);
            else if (tag == fixtureHexField(outputParameters, "tag")) responseFields.add(outputField);
            else if (tag == fixtureHexField(returnCode, "tag")) responseFields.add(returnCodeField);
            else if (tag == fixtureHexField(vendorExtension, "tag")) responseFields.add(vendorField);
            else throw new AssertionError("fixture has an unmapped PKCS#11 response tag");
        }
        byte[] payload = structure(0x42007c, responseFields.toArray(byte[][]::new));
        byte[] responseItem = structure(0x42000f,
                enumeration(0x42005c, 0x33), enumeration(0x42007f, 0), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static TtlvStructure pkcs11Structure(int tagValue, TtlvValue value, CodecLimits limits) {
        TtlvStructure structure = TtlvStructure.create();
        try (Tag tag = RawTag.fromRaw(tagValue).checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        } catch (RuntimeException failure) {
            structure.close();
            throw failure;
        }
    }

    private static Path pkcs11ParityFixturePath() throws Exception {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve("tests/fixtures/api/pkcs11-parity.json");
            if (Files.isRegularFile(candidate)) return candidate;
            directory = directory.getParent();
        }
        throw new IllegalStateException("shared PKCS#11 parity fixture is missing");
    }

    private static List<Long> fixtureLongArray(String source, String name) {
        Matcher field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*\\[([^\\]]*)\\]").matcher(source);
        if (!field.find()) throw new AssertionError("PKCS#11 parity fixture is missing " + name);
        Matcher values = Pattern.compile("\\\"0x([0-9a-fA-F]+)\\\"").matcher(field.group(1));
        List<Long> result = new ArrayList<>();
        while (values.find()) result.add(Long.parseUnsignedLong(values.group(1), 16));
        return List.copyOf(result);
    }

    private static long fixtureHexField(String source, String name) {
        Matcher field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*\\\"0x([0-9a-fA-F]+)\\\"").matcher(source);
        if (!field.find()) throw new AssertionError("PKCS#11 parity fixture is missing " + name);
        return Long.parseUnsignedLong(field.group(1), 16);
    }

    private static long fixtureLongField(String source, String name) {
        Matcher field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*(-?\\d+)").matcher(source);
        if (!field.find()) throw new AssertionError("PKCS#11 parity fixture is missing " + name);
        return Long.parseLong(field.group(1));
    }

    private static String fixtureObject(String source, String name) {
        Matcher field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*\\{").matcher(source);
        if (!field.find()) throw new AssertionError("PKCS#11 parity fixture is missing " + name);
        int start = field.end() - 1;
        int depth = 0;
        for (int index = start; index < source.length(); index++) {
            if (source.charAt(index) == '{') depth++;
            if (source.charAt(index) == '}' && --depth == 0) {
                return source.substring(start, index + 1);
            }
        }
        throw new AssertionError("PKCS#11 parity fixture has an unterminated " + name);
    }

    private static byte[] fixtureHexBytes(String source, String name) {
        Matcher field = Pattern.compile("\\\"" + Pattern.quote(name)
                + "\\\"\\s*:\\s*\\\"([0-9a-fA-F]*)\\\"").matcher(source);
        if (!field.find()) throw new AssertionError("PKCS#11 parity fixture is missing " + name);
        return HexFormat.of().parseHex(field.group(1));
    }

    private static void assertEncodedInOrder(byte[] encoded, byte[]... expectedItems)
            throws Exception {
        int previousEnd = -1;
        for (byte[] expected : expectedItems) {
            int offset = indexOf(encoded, expected);
            assertTrue(offset > previousEnd, "request fields retain their KMIP wire order");
            previousEnd = offset + expected.length - 1;
        }
    }

    private static int indexOf(byte[] value, byte[] expected) {
        for (int offset = 0; offset <= value.length - expected.length; offset++) {
            if (Arrays.equals(expected, Arrays.copyOfRange(value, offset, offset + expected.length))) {
                return offset;
            }
        }
        return -1;
    }

    private static void assertNestedInteger(
            TtlvStructureView structure, long index, long expectedTag, int expectedValue) {
        try (TtlvItemView outer = structure.itemAt(index);
                Tag outerTag = outer.tag();
                TtlvValueView outerValue = outer.value();
                TtlvStructureView nested = outerValue.structure();
                TtlvItemView inner = nested.itemAt(0);
                Tag innerTag = inner.tag();
                TtlvValueView innerValue = inner.value()) {
            assertEquals(expectedTag, outerTag.raw());
            assertEquals(1, nested.itemCount());
            assertEquals(expectedTag, innerTag.raw());
            assertEquals(expectedValue, innerValue.integer());
        }
    }

    private static void assertNestedBytes(
            TtlvStructureView structure, long index, long expectedTag, byte[] expectedBytes) {
        try (TtlvItemView outer = structure.itemAt(index);
                Tag outerTag = outer.tag();
                TtlvValueView outerValue = outer.value();
                TtlvStructureView nested = outerValue.structure();
                TtlvItemView inner = nested.itemAt(0);
                Tag innerTag = inner.tag();
                TtlvValueView innerValue = inner.value()) {
            assertEquals(expectedTag, outerTag.raw());
            assertEquals(1, nested.itemCount());
            assertEquals(expectedTag, innerTag.raw());
            assertArrayEquals(expectedBytes, bytesFrom(innerValue));
        }
    }

    private static void assertBytesAt(
            TtlvStructureView structure, long index, long expectedTag, byte[] expectedBytes) {
        try (TtlvItemView item = structure.itemAt(index);
                Tag tag = item.tag();
                TtlvValueView value = item.value()) {
            assertEquals(expectedTag, tag.raw());
            assertArrayEquals(expectedBytes, bytesFrom(value));
        }
    }

    private static byte[] bytesFrom(TtlvValueView value) {
        byte[] bytes = new byte[Math.toIntExact(value.byteLength())];
        for (int index = 0; index < bytes.length; index++) {
            bytes[index] = (byte) value.byteAt(index);
        }
        return bytes;
    }

    private static byte[] usageAllocationResponseMessage(byte[] itemId, String allocationIdentifier)
            throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, allocationIdentifier.getBytes(StandardCharsets.UTF_8)));
        byte[] responseItem = structure(0x42000f,
                enumeration(0x42005c, 17), item(0x420093, 8, itemId),
                enumeration(0x42007f, 0), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] guardedEncryptResponseMessage(byte[] itemId) throws Exception {
        return guardedEncryptResponseMessage(itemId, "allocation-object");
    }

    private static byte[] guardedEncryptResponseMessage(byte[] itemId, String allocationIdentifier)
            throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, allocationIdentifier.getBytes(StandardCharsets.UTF_8)),
                item(0x4200c2, 8, "guarded-encrypt-result".getBytes(StandardCharsets.US_ASCII)));
        byte[] responseItem = structure(0x42000f,
                enumeration(0x42005c, 0x1f), item(0x420093, 8, itemId),
                enumeration(0x42007f, 0), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static FutureTask<byte[]> peerThatClosesAfterOneRequest(SSLServerSocket listener) {
        return new FutureTask<>(() -> {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                return readFrame(socket.getInputStream());
            }
        });
    }

    private static FutureTask<byte[]> peerThatRespondsAfterOneRequest(SSLServerSocket listener, byte[] response) {
        return new FutureTask<>(() -> {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readFrame(socket.getInputStream());
                OutputStream output = socket.getOutputStream();
                output.write(response);
                output.flush();
                return request;
            }
        });
    }

    private static byte[] readFrame(InputStream input) throws Exception {
        DataInputStream data = new DataInputStream(input);
        byte[] header = data.readNBytes(8);
        if (header.length != 8) throw new IllegalStateException("request frame header is incomplete");
        long valueLength = Integer.toUnsignedLong(ByteBuffer.wrap(header, 4, 4).getInt());
        if (valueLength > 16 * 1024 * 1024L) throw new IllegalStateException("request frame is oversized");
        int paddedLength = Math.toIntExact((valueLength + 7) & ~7L);
        byte[] body = data.readNBytes(paddedLength);
        if (body.length != paddedLength) throw new IllegalStateException("request frame body is incomplete");
        byte[] frame = new byte[header.length + body.length];
        System.arraycopy(header, 0, frame, 0, header.length);
        System.arraycopy(body, 0, frame, header.length, body.length);
        return frame;
    }

    private static byte[] mixedResponseMessage() throws Exception {
        return mixedResponseMessage(List.of(uniqueBatchItemId(0), uniqueBatchItemId(1),
                uniqueBatchItemId(2)), CORRELATION);
    }

    private static byte[] mixedResponseMessage(List<byte[]> fixtureItemIds, byte[] correlation)
            throws Exception {
        return mixedResponseMessage(fixtureItemIds, correlation,
                List.of(new ProtocolVersion(2, 1), new ProtocolVersion(2, 1)));
    }

    private static byte[] mixedResponseMessage(List<byte[]> fixtureItemIds, byte[] correlation,
            List<ProtocolVersion> supportedVersions) throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(27)));
        ByteArrayOutputStream responseVersionValues = new ByteArrayOutputStream();
        for (ProtocolVersion supportedVersion : supportedVersions) {
            responseVersionValues.write(structure(0x420069,
                    item(0x42006a, 2, integer(supportedVersion.major())),
                    item(0x42006b, 2, integer(supportedVersion.minor()))));
        }
        byte[] responsePayload = structure(0x42007c, responseVersionValues.toByteArray());
        byte[] successfulItem = structure(0x42000f,
                item(0x42005c, 5, integer(30)),
                item(0x420093, 8, fixtureItemIds.get(0)),
                item(0x42007f, 5, integer(0)), responsePayload);
        byte[] pendingItem = structure(0x42000f,
                item(0x42005c, 5, integer(30)),
                item(0x420093, 8, fixtureItemIds.get(1)),
                item(0x42007f, 5, integer(2)),
                item(0x420006, 8, correlation), structure(0x42007c));
        byte[] failedItem = structure(0x42000f,
                item(0x42005c, 5, integer(30)),
                item(0x420093, 8, fixtureItemIds.get(2)),
                item(0x42007f, 5, integer(1)),
                enumeration(0x42007e, 1));
        byte[] emptyItem = structure(0x42000f,
                item(0x42005c, 5, integer(30)),
                item(0x420093, 8, uniqueBatchItemId(3)),
                item(0x42007f, 5, integer(0)), structure(0x42007c));
        byte[] createPayload = structure(0x42007c,
                enumeration(0x420057, 7), item(0x420094, 2, integer(-8)));
        byte[] createItem = structure(0x42000f,
                enumeration(0x42005c, 1), item(0x420093, 8, uniqueBatchItemId(4)),
                enumeration(0x42007f, 0), createPayload);
        byte[] createFailure = structure(0x42000f,
                enumeration(0x42005c, 1), item(0x420093, 8, uniqueBatchItemId(5)),
                enumeration(0x42007f, 1), enumeration(0x42007e, 5),
                item(0x42007d, 7, "création refusée ✓".getBytes(StandardCharsets.UTF_8)));
        byte[] createKeyPairPayload = structure(0x42007c,
                enumeration(0x420066, 0xffff_ffff),
                item(0x42006f, 7, "public-key-α".getBytes(StandardCharsets.UTF_8)));
        byte[] createKeyPairItem = structure(0x42000f,
                enumeration(0x42005c, 2), item(0x420093, 8, uniqueBatchItemId(6)),
                enumeration(0x42007f, 0), createKeyPairPayload);
        byte[] createSplitKeyPayload = structure(0x42007c,
                item(0x420094, 7, "part-a".getBytes(StandardCharsets.UTF_8)),
                enumeration(0x420094, 0xffff_ffff),
                item(0x420094, 2, integer(-7)),
                item(0x420094, 7, "part-a".getBytes(StandardCharsets.UTF_8)));
        byte[] createSplitKeyItem = structure(0x42000f,
                enumeration(0x42005c, 40), item(0x420093, 8, uniqueBatchItemId(7)),
                enumeration(0x42007f, 0), createSplitKeyPayload);
        byte[] textCreatePayload = structure(0x42007c,
                enumeration(0x420057, 7), item(0x420094, 7,
                        "create-text-id-secret".getBytes(StandardCharsets.UTF_8)));
        byte[] textCreateItem = structure(0x42000f,
                enumeration(0x42005c, 1), item(0x420093, 8, uniqueBatchItemId(8)),
                enumeration(0x42007f, 0), textCreatePayload);
        byte[] certifyPayload = structure(0x42007c,
                item(0x420094, 7, "certify-text-id-secret".getBytes(StandardCharsets.UTF_8)),
                item(0x540001, 8, "KMIPKit-certify-sentinel".getBytes(StandardCharsets.US_ASCII)));
        byte[] certifyItem = structure(0x42000f,
                enumeration(0x42005c, 6), item(0x420093, 8, uniqueBatchItemId(9)),
                enumeration(0x42007f, 0), certifyPayload);
        byte[] allocationPayload = structure(0x42007c,
                item(0x420094, 7, "allocation-object".getBytes(StandardCharsets.UTF_8)),
                item(0x540001, 8, "KMIPKit-allocation-sentinel".getBytes(StandardCharsets.US_ASCII)));
        byte[] allocationItem = structure(0x42000f,
                enumeration(0x42005c, 17), item(0x420093, 8, uniqueBatchItemId(10)),
                enumeration(0x42007f, 0), allocationPayload);
        byte[] checkPayload = structure(0x42007c,
                item(0x420096, TtlvItemType.LongInteger.code(), longInteger(3)),
                item(0x42002c, TtlvItemType.Integer.code(), integer(2)),
                item(0x420049, TtlvItemType.Interval.code(), integer(60)));
        byte[] checkItem = structure(0x42000f,
                enumeration(0x42005c, 9), item(0x420093, 8, uniqueBatchItemId(11)),
                enumeration(0x42007f, 1), enumeration(0x42007e, 0x29), checkPayload);
        byte[] ticket = structure(0x420149,
                enumeration(0x42014a, 0xfeed_beef),
                item(0x42014b, 8, "delegated-ticket-secret".getBytes(StandardCharsets.US_ASCII)));
        byte[] delegatedLoginPayload = structure(0x42007c, ticket);
        byte[] delegatedLoginItem = structure(0x42000f,
                enumeration(0x42005c, 0x2f), item(0x420093, 8, uniqueBatchItemId(12)),
                enumeration(0x42007f, 0), delegatedLoginPayload);
        byte[] deriveKeyPayload = structure(0x42007c,
                item(0x420094, 7, "derived-key".getBytes(StandardCharsets.UTF_8)),
                item(0x540001, 8, "KMIPKit-derive-key-sentinel".getBytes(StandardCharsets.US_ASCII)));
        byte[] deriveKeyItem = structure(0x42000f,
                enumeration(0x42005c, 5), item(0x420093, 8, uniqueBatchItemId(13)),
                enumeration(0x42007f, 0), deriveKeyPayload);
        byte[] exportAttributes = structure(0x0042_0125,
                enumeration(0x0042_001d, 1));
        byte[] exportObject = structure(0x0042_008f,
                structure(0x0042_0040,
                        item(0x0042_0045, 8, "KMIPKit-export-sentinel".getBytes(StandardCharsets.US_ASCII))));
        byte[] exportPayload = structure(0x42007c,
                enumeration(0x0042_0057, 0xfedc_ba98),
                item(0x0042_0094, 7, "export-secret-id".getBytes(StandardCharsets.UTF_8)),
                exportAttributes, exportObject);
        byte[] exportItem = structure(0x42000f,
                enumeration(0x0042_005c, 43), item(0x0042_0093, 8, uniqueBatchItemId(14)),
                enumeration(0x0042_007f, 0), exportPayload);
        byte[] exportFailure = structure(0x42000f,
                enumeration(0x0042_005c, 43), item(0x0042_0093, 8, uniqueBatchItemId(15)),
                enumeration(0x0042_007f, 1), enumeration(0x0042_007e, 5));
        byte[] constraintsPayload = structure(0x42007c,
                structure(0x420168,
                        item(0x540001, 8, "KMIPKit-constraints-sentinel".getBytes(StandardCharsets.US_ASCII))));
        byte[] constraintsItem = structure(0x42000f,
                enumeration(0x42005c, 0x38), item(0x420093, 8, uniqueBatchItemId(16)),
                enumeration(0x42007f, 0), constraintsPayload);
        byte[] constraintsFailure = structure(0x42000f,
                enumeration(0x42005c, 0x38), item(0x420093, 8, uniqueBatchItemId(17)),
                enumeration(0x42007f, 1), enumeration(0x42007e, 5));
        byte[] getObject = structure(0x42008f,
                structure(0x420040,
                        item(0x420045, 8, "KMIPKit-get-response-sentinel".getBytes(StandardCharsets.US_ASCII))));
        byte[] getPayload = structure(0x42007c,
                enumeration(0x420057, 2),
                item(0x420094, 7, "get-response-secret-id".getBytes(StandardCharsets.UTF_8)),
                getObject);
        byte[] getItem = structure(0x42000f,
                enumeration(0x42005c, 10), item(0x420093, 8, uniqueBatchItemId(18)),
                enumeration(0x42007f, 0), getPayload);
        byte[] locatePayload = structure(0x42007c,
                item(0x4200d5, 2, integer(3)),
                item(0x420094, 7, "locate-first".getBytes(StandardCharsets.UTF_8)),
                item(0x420094, 7, "locate-second".getBytes(StandardCharsets.UTF_8)),
                item(0x420094, 7, "locate-first".getBytes(StandardCharsets.UTF_8)));
        byte[] locateItem = structure(0x42000f,
                enumeration(0x42005c, 8), item(0x420093, 8, uniqueBatchItemId(19)),
                enumeration(0x42007f, 0), locatePayload);
        byte[] getFailure = structure(0x42000f,
                enumeration(0x42005c, 10), item(0x420093, 8, uniqueBatchItemId(20)),
                enumeration(0x42007f, 1), enumeration(0x42007e, 5));
        byte[] locateFailure = structure(0x42000f,
                enumeration(0x42005c, 8), item(0x420093, 8, uniqueBatchItemId(21)),
                enumeration(0x42007f, 1), enumeration(0x42007e, 5));
        byte[] importPayload = structure(0x42007c,
                item(0x420094, 7, "imported-object-secret-id".getBytes(StandardCharsets.UTF_8)),
                item(0x540001, 8, "KMIPKit-import-sentinel".getBytes(StandardCharsets.US_ASCII)));
        byte[] importItem = structure(0x42000f,
                enumeration(0x42005c, 42), item(0x420093, 8, uniqueBatchItemId(22)),
                enumeration(0x42007f, 0), importPayload);
        byte[] importFailure = structure(0x42000f,
                enumeration(0x42005c, 42), item(0x420093, 8, uniqueBatchItemId(23)),
                enumeration(0x42007f, 1), enumeration(0x42007e, 5));
        byte[] interopItem = structure(0x42000f,
                enumeration(0x42005c, 0x34), item(0x420093, 8, uniqueBatchItemId(24)),
                enumeration(0x42007f, 0), structure(0x42007c));
        byte[] logItem = structure(0x42000f,
                enumeration(0x42005c, 0x2c), item(0x420093, 8, uniqueBatchItemId(25)),
                enumeration(0x42007f, 0), structure(0x42007c));
        byte[] loginTicket = structure(0x420149,
                enumeration(0x42014a, 0xfeed_beef),
                item(0x42014b, 8, "login-ticket-native-secret".getBytes(StandardCharsets.US_ASCII)));
        byte[] loginItem = structure(0x42000f,
                enumeration(0x42005c, 0x2d), item(0x420093, 8, uniqueBatchItemId(26)),
                enumeration(0x42007f, 0), structure(0x42007c, loginTicket));
        return structure(0x42007b, header, successfulItem, pendingItem, failedItem, emptyItem,
                createItem, createFailure, createKeyPairItem, createSplitKeyItem, textCreateItem,
                certifyItem, allocationItem, checkItem, delegatedLoginItem, deriveKeyItem,
                exportItem, exportFailure, constraintsItem, constraintsFailure, getItem, locateItem,
                getFailure, locateFailure, importItem, importFailure, interopItem, logItem, loginItem);
    }

    private static byte[] pingQueryLogoutResponseMessage() throws Exception {
        byte[] pingId = "ping-fixture".getBytes(StandardCharsets.US_ASCII);
        byte[] queryId = "query-fixture".getBytes(StandardCharsets.US_ASCII);
        byte[] logoutId = "logout-fixture".getBytes(StandardCharsets.US_ASCII);
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(3)));
        byte[] pingItem = structure(0x42000f,
                enumeration(0x42005c, 0x3b),
                item(0x420093, 8, pingId),
                enumeration(0x42007f, 0),
                structure(0x42007c));
        byte[] queryPayload = structure(0x42007c, structure(0x42015f));
        byte[] queryItem = structure(0x42000f,
                enumeration(0x42005c, 0x18),
                item(0x420093, 8, queryId),
                enumeration(0x42007f, 0),
                queryPayload);
        byte[] logoutItem = structure(0x42000f,
                enumeration(0x42005c, 0x2e),
                item(0x420093, 8, logoutId),
                enumeration(0x42007f, 0),
                structure(0x42007c));
        return structure(0x42007b, header, pingItem, queryItem, logoutItem);
    }

    private static TtlvStructure logoutTicket() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure empty = TtlvStructure.create();
        try (empty;
                RawTag typeRawTag = RawTag.fromRaw(0x42014a);
                Tag typeTag = typeRawTag.checked();
                TtlvValue type = TtlvValue.Enumeration(1);
                TtlvItem typeItem = TtlvItem.create(typeTag, type, limits)) {
            TtlvStructure withType = TtlvStructure.withItem(empty, typeItem, limits);
            try (withType;
                    RawTag valueRawTag = RawTag.fromRaw(0x42014b);
                    Tag valueTag = valueRawTag.checked();
                    TtlvValue value = TtlvValue.ByteString(new byte[] {1, 2, 3}, limits);
                    TtlvItem valueItem = TtlvItem.create(valueTag, value, limits)) {
                return TtlvStructure.withItem(withType, valueItem, limits);
            }
        }
    }

    private static byte[] largeCryptoResponseMessage(int operation, byte[] data) throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "large-response-key".getBytes(StandardCharsets.UTF_8)),
                item(0x4200c2, 8, data));
        byte[] responseItem = structure(0x42000f,
                enumeration(0x42005c, operation),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] largeQueryResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c, structure(0x42015f));
        byte[] responseItem = structure(0x42000f,
                enumeration(0x42005c, 0x18),
                item(0x420093, 8, "large-query".getBytes(StandardCharsets.US_ASCII)),
                enumeration(0x42007f, 0), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] largeResponseData() {
        byte[] data = new byte[LARGE_RESPONSE_DATA_BYTES];
        for (int index = 0; index < data.length; index++) {
            data[index] = (byte) ((index * 31 + 17) & 0xff);
        }
        return data;
    }

    private static byte[] executeCryptoDataAfterSourceClose(Client client, int operation, CodecLimits limits)
            throws Exception {
        if (operation == 0x1f) {
            ClientBatchItemResult.EncryptResponse copied;
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(largeCryptoRequestItem(operation));
                try (ClientBatchResponse response = client.execute(batch, limits)) {
                    assertEquals(1, response.itemCount());
                    try (ClientBatchItemResult source = response.itemAt(0)) {
                        copied = source.encryptResponse().orElseThrow();
                    }
                }
            }
            try (copied) {
                return copied.data().orElseThrow();
            }
        }
        if (operation == 0x20) {
            ClientBatchItemResult.DecryptResponse copied;
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(largeCryptoRequestItem(operation));
                try (ClientBatchResponse response = client.execute(batch, limits)) {
                    assertEquals(1, response.itemCount());
                    try (ClientBatchItemResult source = response.itemAt(0)) {
                        copied = source.decryptResponse().orElseThrow();
                    }
                }
            }
            try (copied) {
                return copied.data().orElseThrow();
            }
        }
        throw new IllegalArgumentException("unsupported large-response crypto operation");
    }

    private static ClientBatchItem largeCryptoRequestItem(int operation) {
        UniqueIdentifier identifier = UniqueIdentifier.textString("large-response-key");
        try (TtlvStructure parameters = TtlvStructure.create();
                TtlvValue data = TtlvValue.ByteString(new byte[] {0x01, 0x02}, CodecLimits.defaults())) {
            ClientBatchItem item = operation == 0x1f
                    ? ClientBatchItem.encrypt(identifier, parameters, data, null, null, null, null, null)
                    : ClientBatchItem.decrypt(identifier, parameters, data, null, null, null, null, null, null);
            return item.withUniqueBatchItemId(uniqueBatchItemId(0));
        }
    }

    private static byte[] discoverVersionsResponseMessage(int supportedVersionCount) throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        ByteArrayOutputStream supportedVersions = new ByteArrayOutputStream();
        for (int index = 0; index < supportedVersionCount; index++) {
            supportedVersions.write(version);
        }
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c, supportedVersions.toByteArray());
        byte[] responseItem = structure(0x42000f, item(0x42005c, 5, integer(30)),
                item(0x420093, 8, uniqueBatchItemId(0)), item(0x42007f, 5, integer(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] createSplitKeyResponseMessage(int identifierCount) throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        ByteArrayOutputStream identifiers = new ByteArrayOutputStream();
        for (int index = 0; index < identifierCount; index++) {
            identifiers.write(item(0x420094, 2, integer(index)));
        }
        byte[] payload = structure(0x42007c, identifiers.toByteArray());
        byte[] responseItem = structure(0x42000f, item(0x42005c, 5, integer(40)),
                item(0x42007f, 5, integer(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] joinSplitKeyResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "joined-output-id".getBytes(StandardCharsets.UTF_8)),
                item(0x540001, 8, new byte[] {0x01, 0x00, (byte) 0xff}));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, integer(0x29)),
                item(0x420093, 8, uniqueBatchItemId(0)),
                item(0x42007f, 5, integer(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] obtainLeaseResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "leased-object".getBytes(StandardCharsets.UTF_8)),
                item(0x420049, TtlvItemType.Interval.code(), integer(0)),
                item(0x420048, TtlvItemType.DateTime.code(), longInteger(-7)),
                item(0x541234, 8, new byte[] {0x01, 0x00, (byte) 0xff}));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, integer(0x10)),
                item(0x420093, 8, uniqueBatchItemId(0)),
                item(0x42007f, 5, integer(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] recertifyResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "replacement-certificate".getBytes(StandardCharsets.UTF_8)),
                item(0x541234, 8, new byte[] {0x01, 0x00, (byte) 0xff}));
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, integer(0x07)),
                item(0x420093, 8, uniqueBatchItemId(0)),
                item(0x42007f, 5, integer(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] registerResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "registered-server-id".getBytes(StandardCharsets.UTF_8)),
                item(0x541234, 8, new byte[] {0x01, 0x00, (byte) 0xff}));
        byte[] responseItem = structure(0x42000f,
                enumeration(0x42005c, 0x03),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] revokeResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(3)));
        byte[] successPayload = structure(0x42007c,
                item(0x420094, 7, "revoked-server-id".getBytes(StandardCharsets.UTF_8)),
                item(0x541234, 8, new byte[] {0x01, 0x00, (byte) 0xff}));
        byte[] success = structure(0x42000f,
                enumeration(0x42005c, 0x13),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0), successPayload);
        byte[] failure = structure(0x42000f,
                enumeration(0x42005c, 0x13),
                item(0x420093, 8, uniqueBatchItemId(1)),
                enumeration(0x42007f, 1),
                enumeration(0x42007e, 5),
                item(0x42007d, 7, "revoke rejected".getBytes(StandardCharsets.UTF_8)));
        byte[] pending = structure(0x42000f,
                enumeration(0x42005c, 0x13),
                item(0x420093, 8, uniqueBatchItemId(2)),
                enumeration(0x42007f, 2),
                item(0x420006, 8, CORRELATION),
                structure(0x42007c));
        return structure(0x42007b, header, success, failure, pending);
    }

    private static byte[] lifecycleFailurePendingResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(4)));
        byte[] activateFailure = structure(0x42000f,
                enumeration(0x42005c, 0x12),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 1),
                enumeration(0x42007e, 5),
                item(0x42007d, 7, "activate rejected".getBytes(StandardCharsets.UTF_8)));
        byte[] archiveFailure = structure(0x42000f,
                enumeration(0x42005c, 0x15),
                item(0x420093, 8, uniqueBatchItemId(1)),
                enumeration(0x42007f, 1),
                enumeration(0x42007e, 5),
                item(0x42007d, 7, "archive rejected".getBytes(StandardCharsets.UTF_8)));
        byte[] destroyPending = structure(0x42000f,
                enumeration(0x42005c, 0x14),
                item(0x420093, 8, uniqueBatchItemId(2)),
                enumeration(0x42007f, 2),
                item(0x420006, 8, LIFECYCLE_DESTROY_CORRELATION),
                structure(0x42007c));
        byte[] recoverPending = structure(0x42000f,
                enumeration(0x42005c, 0x16),
                item(0x420093, 8, uniqueBatchItemId(3)),
                enumeration(0x42007f, 2),
                item(0x420006, 8, LIFECYCLE_RECOVER_CORRELATION),
                structure(0x42007c));
        return structure(0x42007b, header,
                activateFailure, archiveFailure, destroyPending, recoverPending);
    }

    private static byte[] lifecycleSuccessResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(4)));
        byte[] activate = structure(0x42000f,
                enumeration(0x42005c, 0x12),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0),
                structure(0x42007c,
                        item(0x420094, 7, "activate-result-secret".getBytes(StandardCharsets.UTF_8))));
        byte[] archive = structure(0x42000f,
                enumeration(0x42005c, 0x15),
                item(0x420093, 8, uniqueBatchItemId(1)),
                enumeration(0x42007f, 0),
                structure(0x42007c, enumeration(0x420094, (int) 0x8fed_cba9L)));
        byte[] destroy = structure(0x42000f,
                enumeration(0x42005c, 0x14),
                item(0x420093, 8, uniqueBatchItemId(2)),
                enumeration(0x42007f, 0),
                structure(0x42007c, item(0x420094, 2, integer(-723))));
        byte[] recover = structure(0x42000f,
                enumeration(0x42005c, 0x16),
                item(0x420093, 8, uniqueBatchItemId(3)),
                enumeration(0x42007f, 0),
                structure(0x42007c,
                        item(0x420094, 7, "recover-result-secret".getBytes(StandardCharsets.UTF_8))));
        return structure(0x42007b, header, activate, archive, destroy, recover);
    }

    private static byte[] setConstraintsResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(3)));
        byte[] success = structure(0x42000f,
                enumeration(0x42005c, 0x37),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0), structure(0x42007c));
        byte[] failure = structure(0x42000f,
                enumeration(0x42005c, 0x37),
                item(0x420093, 8, uniqueBatchItemId(1)),
                enumeration(0x42007f, 1),
                enumeration(0x42007e, 5),
                item(0x42007d, 7, "constraints rejected".getBytes(StandardCharsets.UTF_8)));
        byte[] pending = structure(0x42000f,
                enumeration(0x42005c, 0x37),
                item(0x420093, 8, uniqueBatchItemId(2)),
                enumeration(0x42007f, 2),
                item(0x420006, 8, CORRELATION),
                structure(0x42007c));
        return structure(0x42007b, header, success, failure, pending);
    }

    private static byte[] setDefaultsResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(3)));
        byte[] success = structure(0x42000f,
                enumeration(0x42005c, 0x36),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0), structure(0x42007c));
        byte[] failure = structure(0x42000f,
                enumeration(0x42005c, 0x36),
                item(0x420093, 8, uniqueBatchItemId(1)),
                enumeration(0x42007f, 1),
                enumeration(0x42007e, 7),
                item(0x42007d, 7, "defaults rejected".getBytes(StandardCharsets.UTF_8)));
        byte[] pending = structure(0x42000f,
                enumeration(0x42005c, 0x36),
                item(0x420093, 8, uniqueBatchItemId(2)),
                enumeration(0x42007f, 2),
                item(0x420006, 8, CORRELATION),
                structure(0x42007c));
        return structure(0x42007b, header, success, failure, pending);
    }

    private static byte[] reprovisionResponseWithOptionalIdentifiersMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(2)));
        byte[] identifierPayload = structure(0x42007c,
                item(0x420094, 7, "server-created-certificate".getBytes(StandardCharsets.UTF_8)));
        byte[] responseWithIdentifier = structure(0x42000f,
                enumeration(0x42005c, 0x35),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0), identifierPayload);
        byte[] responseWithoutIdentifier = structure(0x42000f,
                enumeration(0x42005c, 0x35),
                item(0x420093, 8, uniqueBatchItemId(1)),
                enumeration(0x42007f, 0), structure(0x42007c));
        return structure(0x42007b, header, responseWithIdentifier, responseWithoutIdentifier);
    }

    private static byte[] validateResponseMessage(long validityIndicator, byte[] responseSentinel)
            throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] payload = structure(0x42007c,
                enumeration(0x42009b, (int) validityIndicator),
                item(0x541234, 8, responseSentinel));
        byte[] responseItem = structure(0x42000f,
                enumeration(0x42005c, 0x17),
                item(0x420093, 8, uniqueBatchItemId(0)),
                enumeration(0x42007f, 0), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] rekeyResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(3)));
        byte[] payload = structure(0x42007c,
                item(0x420094, 7, "replacement-key".getBytes(StandardCharsets.UTF_8)),
                item(0x541234, 8, new byte[] {0x01, 0x00, (byte) 0xff}));
        byte[] success = structure(0x42000f, enumeration(0x42005c, 0x04),
                item(0x420093, 8, uniqueBatchItemId(0)), enumeration(0x42007f, 0), payload);
        byte[] failure = structure(0x42000f, enumeration(0x42005c, 0x04),
                item(0x420093, 8, uniqueBatchItemId(1)), enumeration(0x42007f, 1),
                enumeration(0x42007e, 5));
        byte[] pending = structure(0x42000f, enumeration(0x42005c, 0x04),
                item(0x420093, 8, uniqueBatchItemId(2)), enumeration(0x42007f, 2),
                item(0x420006, 8, CORRELATION), structure(0x42007c));
        return structure(0x42007b, header, success, failure, pending);
    }

    private static byte[] rngOperationResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(2)));
        byte[] retrievePayload = structure(0x42007c,
                item(0x4200c2, 8, RNG_RETRIEVED_SENTINEL.getBytes(StandardCharsets.US_ASCII)));
        byte[] retrieve = structure(0x42000f, enumeration(0x42005c, 0x25),
                item(0x420093, 8, uniqueBatchItemId(0)), enumeration(0x42007f, 0),
                retrievePayload);
        byte[] seedPayload = structure(0x42007c, item(0x4200d5, 2, integer(0)));
        byte[] seed = structure(0x42000f, enumeration(0x42005c, 0x26),
                item(0x420093, 8, uniqueBatchItemId(1)), enumeration(0x42007f, 0), seedPayload);
        return structure(0x42007b, header, retrieve, seed);
    }

    private static byte[] rngFailurePendingResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(4)));
        byte[] retrieveFailure = structure(0x42000f, enumeration(0x42005c, 0x25),
                item(0x420093, 8, uniqueBatchItemId(0)), enumeration(0x42007f, 1),
                enumeration(0x42007e, 5));
        byte[] retrievePending = structure(0x42000f, enumeration(0x42005c, 0x25),
                item(0x420093, 8, uniqueBatchItemId(1)), enumeration(0x42007f, 2),
                item(0x420006, 8, CORRELATION), structure(0x42007c));
        byte[] seedFailure = structure(0x42000f, enumeration(0x42005c, 0x26),
                item(0x420093, 8, uniqueBatchItemId(2)), enumeration(0x42007f, 1),
                enumeration(0x42007e, 5));
        byte[] seedPending = structure(0x42000f, enumeration(0x42005c, 0x26),
                item(0x420093, 8, uniqueBatchItemId(3)), enumeration(0x42007f, 2),
                item(0x420006, 8, CORRELATION), structure(0x42007c));
        return structure(0x42007b, header, retrieveFailure, retrievePending,
                seedFailure, seedPending);
    }

    private static byte[] rekeyKeyPairResponseMessage() throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(4)));
        byte[] createPayload = structure(0x42007c, enumeration(0x420057, 4),
                item(0x420094, 7, "existing-private".getBytes(StandardCharsets.UTF_8)));
        byte[] create = structure(0x42000f, enumeration(0x42005c, 1),
                item(0x420093, 8, uniqueBatchItemId(0)), enumeration(0x42007f, 0), createPayload);
        byte[] pairPayload = structure(0x42007c,
                item(0x420066, 7, "replacement-private".getBytes(StandardCharsets.UTF_8)),
                item(0x42006f, 7, "replacement-public".getBytes(StandardCharsets.UTF_8)),
                item(0x541234, 8, new byte[] {0x01, 0x00, (byte) 0xff}));
        byte[] success = structure(0x42000f, enumeration(0x42005c, 0x1d),
                item(0x420093, 8, uniqueBatchItemId(1)), enumeration(0x42007f, 0), pairPayload);
        byte[] failure = structure(0x42000f, enumeration(0x42005c, 0x1d),
                item(0x420093, 8, uniqueBatchItemId(2)), enumeration(0x42007f, 1),
                enumeration(0x42007e, 5));
        byte[] pending = structure(0x42000f, enumeration(0x42005c, 0x1d),
                item(0x420093, 8, uniqueBatchItemId(3)), enumeration(0x42007f, 2),
                item(0x420006, 8, CORRELATION), structure(0x42007c));
        return structure(0x42007b, header, create, success, failure, pending);
    }

    private static byte[] locateResponseMessage(int identifierCount) throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        ByteArrayOutputStream identifiers = new ByteArrayOutputStream();
        byte[] identifier = item(0x420094, 7, new byte[] {'x'});
        for (int index = 0; index < identifierCount; index++) {
            identifiers.write(identifier);
        }
        byte[] payload = structure(0x42007c,
                item(0x4200d5, 2, integer(identifierCount)), identifiers.toByteArray());
        byte[] responseItem = structure(0x42000f,
                item(0x42005c, 5, integer(8)),
                item(0x42007f, 5, integer(0)), payload);
        return structure(0x42007b, header, responseItem);
    }

    private static byte[] uniqueBatchItemId(int index) {
        return ("item-" + index).getBytes(StandardCharsets.US_ASCII);
    }

    private static int resultStatusCode(String fixtureName) {
        return switch (fixtureName) {
            case "Success" -> 0;
            case "Operation Pending" -> 2;
            case "Operation Failed" -> 1;
            default -> throw new AssertionError("unknown shared batch result status: " + fixtureName);
        };
    }

    private static byte[] structure(int tag, byte[]... items) throws Exception {
        ByteArrayOutputStream output = new ByteArrayOutputStream();
        for (byte[] item : items) output.write(item);
        return item(tag, 1, output.toByteArray());
    }

    private static byte[] item(int tag, int type, byte[] value) throws Exception {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        DataOutputStream output = new DataOutputStream(bytes);
        output.writeByte(tag >>> 16);
        output.writeByte(tag >>> 8);
        output.writeByte(tag);
        output.writeByte(type);
        output.writeInt(value.length);
        output.write(value);
        output.write(new byte[(8 - value.length % 8) % 8]);
        return bytes.toByteArray();
    }

    private static int countOccurrences(byte[] actual, byte[] expected) {
        int count = 0;
        for (int offset = 0; offset <= actual.length - expected.length; offset++) {
            if (Arrays.equals(expected, Arrays.copyOfRange(actual, offset, offset + expected.length))) {
                count++;
                offset += expected.length - 1;
            }
        }
        return count;
    }

    private static byte[] integer(int value) {
        return ByteBuffer.allocate(Integer.BYTES).putInt(value).array();
    }

    private static byte[] enumeration(int tag, int value) throws Exception {
        return item(tag, 5, integer(value));
    }

    private static void assertContains(byte[] actual, byte[] expected) {
        for (int offset = 0; offset <= actual.length - expected.length; offset++) {
            if (Arrays.equals(expected, Arrays.copyOfRange(actual, offset, offset + expected.length))) {
                return;
            }
        }
        assertTrue(false, "request did not contain the expected encoded TTLV item");
    }

    private static boolean containsTtlvItem(byte[] actual, byte[] expected) {
        for (int offset = 0; offset <= actual.length - expected.length; offset++) {
            if (Arrays.equals(expected, Arrays.copyOfRange(actual, offset, offset + expected.length))) {
                return true;
            }
        }
        return false;
    }

    private static byte[] longInteger(long value) {
        return ByteBuffer.allocate(Long.BYTES).putLong(value).array();
    }
}
