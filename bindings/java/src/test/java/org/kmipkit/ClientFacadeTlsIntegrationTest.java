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
import java.util.ArrayList;
import java.util.Arrays;
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
                TtlvStructure splitAttributes = TtlvStructure.create();
                TtlvStructure delegatedLoginRights = TtlvStructure.create();
                TtlvStructure derivationParameters = TtlvStructure.create();
                TtlvStructure derivedAttributes = deriveKeyAttributes()) {
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
                            .withUniqueBatchItemId(uniqueBatchItemId(15)));
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
                    ClientBatchItemResult splitKey = response.itemAt(7);
                    ClientBatchItemResult textCreated = response.itemAt(8);
                    ClientBatchItemResult certified = response.itemAt(9);
                    ClientBatchItemResult allocated = response.itemAt(10);
                    ClientBatchItemResult checked = response.itemAt(11);
                    ClientBatchItemResult delegated = response.itemAt(12);
                    ClientBatchItemResult derived = response.itemAt(13);
                    ClientBatchItemResult exported = response.itemAt(14);
                    ClientBatchItemResult exportFailed = response.itemAt(15)) {
                assertEquals(16, response.itemCount());
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
                    assertTrue(exportFailure.objectTypeRaw().isEmpty());
                    assertTrue(exportFailure.uniqueIdentifier().isEmpty());
                    assertTrue(exportFailure.attributes().isEmpty());
                    assertTrue(exportFailure.anyObject().isEmpty());
                    assertEquals(4, exportFailureTtlv.itemCount());
                }
            }
            byte[] request = peer.get(12, TimeUnit.SECONDS);
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
                () -> serveOneResponse(listener, cryptoOperationResponseMessage(operations)));
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
                        boolean validityIndicatorPresent = !cryptoOperationName(operation).endsWith("Verify");
                        invokeCryptoFixture(null, "assertSuccessfulOutput",
                                new Class<?>[] {operation.getClass(), Object.class, boolean.class},
                                operation, typedResponse, validityIndicatorPresent);
                    }
                }
            }
            assertTrue(peer.get(12, TimeUnit.SECONDS).length >= 8);
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
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(operations.size())));
        CryptoOperationsJavaFacadeTest fixture = new CryptoOperationsJavaFacadeTest();
        List<byte[]> responseItems = new ArrayList<>();
        for (int index = 0; index < operations.size(); index++) {
            Object operation = operations.get(index);
            Method successResponse = CryptoOperationsJavaFacadeTest.class.getDeclaredMethod(
                    "successResponse", operation.getClass(), int.class, boolean.class);
            if (!successResponse.trySetAccessible()) {
                throw new AssertionError("crypto success response fixture must remain accessible to TLS tests");
            }
            responseItems.add((byte[]) invokeCryptoFixture(
                    fixture, successResponse, operation, index, false));
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
                item(0x42000d, 2, integer(16)));
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
        return structure(0x42007b, header, successfulItem, pendingItem, failedItem, emptyItem,
                createItem, createFailure, createKeyPairItem, createSplitKeyItem, textCreateItem,
                certifyItem, allocationItem, checkItem, delegatedLoginItem, deriveKeyItem,
                exportItem, exportFailure);
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
