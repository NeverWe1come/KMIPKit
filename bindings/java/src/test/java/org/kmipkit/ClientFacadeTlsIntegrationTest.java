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
import java.net.InetAddress;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.SecureRandom;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.time.Duration;
import java.util.Base64;
import java.util.List;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
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
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Exercises the T076 client ABI through the Java facade against a local TLS 1.3 peer. */
final class ClientFacadeTlsIntegrationTest {
    private static final char[] STORE_PASSWORD = "kmipkit-test".toCharArray();
    private static final byte[] CORRELATION = {0, (byte) 0xa1, 0x7f};

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
    void executionPreservesOrderedMixedOutcomesOverOneMutualTlsExchange() throws Exception {
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
        FutureTask<byte[]> peer = new FutureTask<>(() -> serveOneResponse(listener));
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
                TtlvStructure splitAttributes = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class,
                    () -> client.executeWithOptions(null, null, null));
            ClientBatch batch = ClientBatch.create();
            batch.asynchronousIndicator(2)
                    .add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(uniqueBatchItemId(0)))
                    .add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(uniqueBatchItemId(1)))
                    .add(ClientBatchItem.discoverVersions().withUniqueBatchItemId(uniqueBatchItemId(2)))
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
                            .withUniqueBatchItemId(uniqueBatchItemId(7)));
            ClientBatchResponse batchResponse = client.executeWithOptions(
                    batch, CodecLimits.defaults(), RequestOptions.defaults());
            try (ClientBatchResponse response = batchResponse;
                    ClientBatchItemResult completed = response.itemAt(0);
                    ClientBatchItemResult pending = response.itemAt(1);
                    ClientBatchItemResult failed = response.itemAt(2);
                    ClientBatchItemResult empty = response.itemAt(3);
                    ClientBatchItemResult created = response.itemAt(4);
                    ClientBatchItemResult creationFailed = response.itemAt(5);
                    ClientBatchItemResult keyPair = response.itemAt(6);
                    ClientBatchItemResult splitKey = response.itemAt(7)) {
                assertEquals(8, response.itemCount());
                assertEquals(0, completed.resultStatus());
                assertTrue(completed.outcomeKind().isCompleted());
                assertArrayEquals(uniqueBatchItemId(0), completed.uniqueBatchItemId().orElseThrow());
                assertTrue(completed.pendingCorrelationValue().isEmpty());
                assertTrue(completed.resultReason().isEmpty());
                assertEquals(List.of(new ProtocolVersion(2, 1), new ProtocolVersion(2, 1)),
                        completed.supportedVersions().orElseThrow());
                assertEquals(2, pending.resultStatus());
                assertTrue(pending.outcomeKind().isPending());
                assertArrayEquals(uniqueBatchItemId(1), pending.uniqueBatchItemId().orElseThrow());
                assertArrayEquals(CORRELATION, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.resultReason().isEmpty());
                assertTrue(pending.supportedVersions().isEmpty());
                assertEquals(1, failed.resultStatus());
                assertTrue(failed.outcomeKind().isCompleted());
                assertArrayEquals(uniqueBatchItemId(2), failed.uniqueBatchItemId().orElseThrow());
                assertTrue(failed.pendingCorrelationValue().isEmpty());
                assertEquals(1, failed.resultReason().orElseThrow());
                assertTrue(failed.supportedVersions().isEmpty());
                assertEquals(0, empty.resultStatus());
                assertTrue(empty.supportedVersions().isPresent());
                assertTrue(empty.supportedVersions().orElseThrow().isEmpty());

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
                assertFalse(creationFailed.metadataV2().toString().contains("création refusée"));

                ClientBatchItemResult.CreateKeyPairResponse keyPairResponse =
                        keyPair.createKeyPairResponse().orElseThrow();
                assertEquals(UniqueIdentifier.Kind.ENUMERATION,
                        keyPairResponse.privateKeyUniqueIdentifier().kind());
                assertEquals(0xffff_ffffL, keyPairResponse.privateKeyUniqueIdentifier().number());
                assertEquals(UniqueIdentifier.Kind.TEXT_STRING,
                        keyPairResponse.publicKeyUniqueIdentifier().kind());
                assertEquals("public-key-α", keyPairResponse.publicKeyUniqueIdentifier().text());

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
            }
            byte[] request = peer.get(12, TimeUnit.SECONDS);
            assertTrue(request.length >= 8);
            assertArrayEquals(new byte[] {0x42, 0x00, 0x78},
                    new byte[] {request[0], request[1], request[2]});
        } finally {
            listener.close();
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
                assertEquals(UniqueIdentifier.integer(0), identifiers.get(0));
                assertEquals(UniqueIdentifier.integer(50_000), identifiers.get(50_000));
                assertEquals(UniqueIdentifier.integer(identifierCount - 1),
                        identifiers.get(identifiers.size() - 1));
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
            assertEquals(2, failure.categoryCode());
            assertEquals(3, failure.deliveryStateCode());
            assertFalse(failure.getMessage().contains("protocol-error-server.p12"));
        } finally {
            listener.close();
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
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(8)));
        byte[] responseVersion = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] responsePayload = structure(0x42007c, responseVersion, responseVersion);
        byte[] successfulItem = structure(0x42000f,
                item(0x42005c, 5, integer(30)),
                item(0x420093, 8, uniqueBatchItemId(0)),
                item(0x42007f, 5, integer(0)), responsePayload);
        byte[] pendingItem = structure(0x42000f,
                item(0x42005c, 5, integer(30)),
                item(0x420093, 8, uniqueBatchItemId(1)),
                item(0x42007f, 5, integer(2)),
                item(0x420006, 8, CORRELATION), structure(0x42007c));
        byte[] failedItem = structure(0x42000f,
                item(0x42005c, 5, integer(30)),
                item(0x420093, 8, uniqueBatchItemId(2)),
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
        return structure(0x42007b, header, successfulItem, pendingItem, failedItem, emptyItem,
                createItem, createFailure, createKeyPairItem, createSplitKeyItem);
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

    private static byte[] uniqueBatchItemId(int index) {
        return ("item-" + index).getBytes(StandardCharsets.US_ASCII);
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

    private static byte[] integer(int value) {
        return ByteBuffer.allocate(Integer.BYTES).putInt(value).array();
    }

    private static byte[] enumeration(int tag, int value) throws Exception {
        return item(tag, 5, integer(value));
    }

    private static byte[] longInteger(long value) {
        return ByteBuffer.allocate(Long.BYTES).putLong(value).array();
    }
}
