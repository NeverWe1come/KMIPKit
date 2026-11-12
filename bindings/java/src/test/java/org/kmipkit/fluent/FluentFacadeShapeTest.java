package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.InputStream;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyFactory;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.ArrayList;
import java.util.Base64;
import java.util.List;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ClientOperationException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.client.SymmetricKeyRequest;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.protocol.KmipValues.CryptographicUsageMask;
import org.junit.jupiter.api.Test;

/** Checks the Java package shape while preserving the established extensions API. */
final class FluentFacadeShapeTest {
    @Test
    void exposesTaskEntryPointAndRetainsLegacyClientType() throws Exception {
        assertNotNull(Class.forName("org.kmipkit.client.KmipClient"));
        assertNotNull(Class.forName("org.kmipkit.client.SymmetricKeyRequest"));
        assertNotNull(Class.forName("org.kmipkit.fluent.SymmetricKeyBuilder"));
        assertNotNull(Class.forName("org.kmipkit.extensions.Client"));
    }

    @Test
    void buildersExposeBuildBatchAndExecutionMethods() throws Exception {
        assertNotNull(SymmetricKeyBuilder.class.getMethod("build"));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("toBatchItem"));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("withOptions",
                org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("asynchronousIndicator", long.class));
        assertNotNull(OperationBuilder.class.getMethod("asynchronousIndicator", long.class));
        assertNotNull(OperationBuilder.class.getMethod("execute",
                org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(Class.forName("org.kmipkit.fluent.GetAttributesBuilder")
                .getMethod("execute", org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(Class.forName("org.kmipkit.fluent.GetAttributeListBuilder")
                .getMethod("execute", org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(Class.forName("org.kmipkit.fluent.FluentRequest")
                .getMethod("getAttributes", org.kmipkit.extensions.UniqueIdentifier.class,
                        java.util.List.class));
        assertNotNull(Class.forName("org.kmipkit.fluent.FluentRequest")
                .getMethod("getAttributeList", org.kmipkit.extensions.UniqueIdentifier.class));
    }

    @Test
    void attributeReadBuildersValidateAndBuildOrderedTypedBatchItems() throws Exception {
        UniqueIdentifier id = UniqueIdentifier.textString("attribute-read-key");
        List<AttributeReference> references = List.of(
                AttributeReference.tag(0x42002A),
                AttributeReference.name("KMIPKit", "Comment"));
        GetAttributesBuilder attributes = new GetAttributesBuilder(null, id, references);
        GetAttributeListBuilder list = new GetAttributeListBuilder(null, null);
        attributes.withOptions(RequestOptions.defaults()).asynchronousIndicator(1).validate();
        list.withOptions(RequestOptions.defaults()).asynchronousIndicator(2).validate();
        try (ClientBatchItem item = attributes.toBatchItem();
                ClientBatchItem listItem = list.toBatchItem()) {
            assertEquals(11, item.operation());
            assertEquals(12, listItem.operation());
        }
        assertThrows(InvalidInputException.class,
                () -> new GetAttributeListBuilder(null, UniqueIdentifier.integer(7)));
        assertThrows(InvalidInputException.class,
                () -> list.asynchronousIndicator(-1));
        assertThrows(InvalidInputException.class,
                () -> list.asynchronousIndicator(0x1_0000_0000L));
        assertThrows(InvalidInputException.class,
                () -> attributes.asynchronousIndicator(0x1_0000_0000L));
        AttributeReference duplicate = AttributeReference.tag(0x42002A);
        GetAttributesBuilder invalid = new GetAttributesBuilder(
                null, id, List.of(duplicate, duplicate));
        assertThrows(InvalidInputException.class, invalid::build);
        assertThrows(InvalidInputException.class,
                () -> new GetAttributesBuilder(null, id, null));
        assertThrows(InvalidInputException.class,
                () -> new GetAttributesBuilder(
                        null, id, java.util.Arrays.asList((AttributeReference) null)));
    }

    @Test
    void attributeReadBuildersExecuteOneTypedExchangePerOperationOverMtls() throws Exception {
        byte[] serverCertificatePem = fixture("server-cert.pem");
        byte[] serverPrivateKeyPem = fixture("server-key.pem");
        byte[] clientCertificatePem = fixture("client-cert.pem");
        byte[] clientPrivateKeyPem = fixture("client-key.pem");
        SSLServerSocket listener = mtlsListener(
                serverCertificatePem, serverPrivateKeyPem, clientCertificatePem);
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveAttributeReads(listener));
        Thread peerThread = new Thread(peer, "kmipkit-java-fluent-attribute-read-mtls");
        peerThread.setDaemon(true);
        peerThread.start();

        List<byte[]> requests;
        CodecLimits limits = CodecLimits.defaults();
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificatePem}, clientPrivateKeyPem)
                        .trustCertificatesPem(new byte[][] {serverCertificatePem})
                        .build();
                KmipClient client = KmipClient.create(configuration, transport)) {
            try (ClientBatchResponse response = client.request().getAttributes(
                            UniqueIdentifier.textString("attribute-read-key"),
                            List.of(AttributeReference.tag(0x42002a),
                                    AttributeReference.name("KMIPKit", "Comment")))
                            .withOptions(RequestOptions.defaults()).execute(limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(11, result.operation());
                assertEquals(0, result.resultStatus());
                assertEquals("attribute-read-key",
                        result.getAttributesResponse().orElseThrow().uniqueIdentifier());
            }
            try (ClientBatchItem omittedIdentifier = client.request().getAttributeList().toBatchItem()) {
                assertEquals(12, omittedIdentifier.operation());
            }
            try (ClientBatchResponse response = client.request().getAttributeList(
                            UniqueIdentifier.textString("attribute-read-key"))
                            .asynchronousIndicator(2).execute(limits);
                    ClientBatchItemResult result = response.itemAt(0)) {
                assertEquals(12, result.operation());
                assertEquals(0, result.resultStatus());
                assertEquals("attribute-read-key",
                        result.getAttributeListResponse().orElseThrow()
                                .uniqueIdentifier().text());
            }
            requests = peer.get(12, TimeUnit.SECONDS);
        } finally {
            listener.close();
        }
        assertEquals(2, requests.size());
        assertTrue(contains(requests.get(0), enumTtlv(0x42005c, 11)));
        assertTrue(contains(requests.get(0), textTtlv(0x420094, "attribute-read-key")));
        byte[] tagReference = enumTtlv(0x42013b, 0x42002a);
        byte[] nameReference = structureTtlv(0x42013b,
                textTtlv(0x42009d, "KMIPKit"), textTtlv(0x42000a, "Comment"));
        assertTrue(contains(requests.get(0), tagReference));
        assertTrue(contains(requests.get(0), nameReference));
        assertTrue(indexOf(requests.get(0), tagReference) < indexOf(requests.get(0), nameReference));
        assertTrue(contains(requests.get(1), enumTtlv(0x420007, 2)));
        assertTrue(contains(requests.get(1), enumTtlv(0x42005c, 12)));
        assertTrue(peer.isDone());
    }

    @Test
    void kmipClientExposesNamedOperationConveniences() throws Exception {
        Class<?> client = Class.forName("org.kmipkit.client.KmipClient");
        Class<?> request = Class.forName("org.kmipkit.client.SymmetricKeyRequest");
        assertNotNull(client.getMethod("createSymmetricKey", request,
                org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("createSymmetricKey", request,
                org.kmipkit.ttlv.CodecLimits.class, org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(client.getMethod("getById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("getById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class,
                org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(client.getMethod("destroyById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("destroyById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class,
                org.kmipkit.extensions.RequestOptions.class));
    }

    @Test
    void createBuilderUsesRustValidationAndProducesTypedCreateItem() {
        try (SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            assertUnsentValidation(assertThrows(ClientOperationException.class, builder::build), 4);
            assertUnsentValidation(assertThrows(ClientOperationException.class,
                    () -> builder.aes256().build()), 4);
            assertThrows(InvalidInputException.class, () -> builder.algorithm(-1));
            assertThrows(InvalidInputException.class, () -> builder.name(null));
            assertThrows(InvalidInputException.class, () -> builder.withOptions(null));
            assertThrows(InvalidInputException.class, () -> new OperationBuilder(null, null, false));
        }
        try (SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) (CryptographicUsageMask.ENCRYPT | CryptographicUsageMask.DECRYPT))
                    .name("payments-key");
            try (ClientBatchItem item = builder.toBatchItem()) {
                assertNotNull(item);
                assertEquals(1, item.operation());
            }
            builder.close();
            assertThrows(InvalidInputException.class, builder::build);
        }
    }

    @Test
    void expertPositionValidationIsTypedAndNotSentBeforeAnyConnection() throws Exception {
        // KMIPKIT-0096-FR-003; KMIP v2.1 sections 4-5 and ADR-0012.
        try (ServerSocket listener = new ServerSocket(0);
                ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {fixture("client-cert.pem")}, fixture("client-key.pem"))
                        .trustCertificatesPem(new byte[][] {fixture("server-cert.pem")}).build();
                KmipClient client = KmipClient.create(configuration, transport);
                RawTag raw = RawTag.fromRaw(0x540001);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(
                        "position-private-value".getBytes(java.nio.charset.StandardCharsets.UTF_8), CodecLimits.defaults());
                TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
            listener.setSoTimeout(100);
            for (int[] positions : new int[][] {{99}, {0, 0}}) {
                try (SymmetricKeyBuilder builder = client.request().createSymmetricKey()) {
                    builder.aes256().usageMask((int) CryptographicUsageMask.ENCRYPT);
                    for (int position : positions) builder.attributeItemAt(position, item);
                    assertUnsentValidation(assertThrows(ClientOperationException.class, builder::build), 4);
                    assertUnsentValidation(assertThrows(ClientOperationException.class,
                            () -> builder.execute(CodecLimits.defaults())), 4);
                    assertThrows(SocketTimeoutException.class, listener::accept);
                }
            }
        }
    }

    @Test
    void expertInsertionReportsNativeTypedValidationAndPreservesOtherArgumentErrors() {
        // Catalogued Cryptographic Algorithm requires Enumeration, not TextString.
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag raw = RawTag.fromRaw(0x420028);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(
                        "position-private-value".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            assertUnsentValidation(assertThrows(ClientOperationException.class,
                    () -> builder.attributeItemAt(0, item)), 13);
            assertThrows(InvalidInputException.class,
                    () -> NativeFluentRegistry.symmetricKeyBuilderInsertAttributeItem(0, -1, 0));
            assertThrows(InvalidInputException.class,
                    () -> org.kmipkit.NativeExtensionRegistry.symmetricKeyBuilderSetAlgorithm(0, 3));
            assertThrows(InvalidInputException.class, () -> builder.attributeItemAt(-1, item));
            assertThrows(InvalidInputException.class, () -> builder.attributeItemAt(0, null));
        }
    }

    private static void assertUnsentValidation(ClientOperationException failure, int status) {
        assertEquals(status, failure.statusCode());
        assertEquals(1, failure.categoryCode());
        assertEquals(1, failure.causeCode());
        assertEquals(1, failure.deliveryStateCode());
        assertNull(failure.getCause());
        assertFalse(failure.toString().contains("position-private-value"));
    }

    @Test
    void builderAcceptsCataloguedUnknownAndVendorAttributes() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag descriptionRaw = RawTag.fromRaw(StandardAttribute.DESCRIPTION.tag());
                Tag descriptionTag = descriptionRaw.checked();
                TtlvValue description = TtlvValue.TextString(
                        "managed without TTLV tags".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                RawTag expertRaw = RawTag.fromRaw(0x540001);
                Tag expertTag = expertRaw.checked();
                TtlvValue expertValue = TtlvValue.TextString(
                        "expert value".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                TtlvItem expertItem = TtlvItem.create(expertTag, expertValue, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) (CryptographicUsageMask.ENCRYPT | CryptographicUsageMask.DECRYPT))
                    .cataloguedAttribute(descriptionTag, description)
                    .attributeItem(expertItem)
                    .vendorAttribute("Vendor_Name.1", "region", expertItem)
                    .withOptions(RequestOptions.defaults())
                    .asynchronousIndicator(1);
            assertThrows(InvalidInputException.class,
                    () -> builder.vendorAttribute(null, "region", expertItem));
            try (ClientBatchItem item = builder.build()) {
                assertEquals(1, item.operation());
            }
            try (TtlvValue wrongValue = TtlvValue.Integer(1)) {
                assertThrows(InvalidInputException.class,
                        () -> builder.cataloguedAttribute(descriptionTag, wrongValue));
            }
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(0x1_0000_0000L));
        }
    }

    @Test
    void builderCanInsertAnExpertItemAtAnExplicitFinalAttributePosition() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag raw = RawTag.fromRaw(0x54F001);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(17);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) CryptographicUsageMask.ENCRYPT)
                    .attributeItemAt(0, item);
            try (ClientBatchItem batchItem = builder.build()) {
                assertEquals(1, batchItem.operation());
            }
            assertThrows(InvalidInputException.class,
                    () -> builder.attributeItemAt(-1, item));
        }

        try (RawTag raw = RawTag.fromRaw(0x54F001);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(17);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) CryptographicUsageMask.ENCRYPT)
                    .attributeItemAt(99, item);
            assertUnsentValidation(assertThrows(ClientOperationException.class, builder::build), 4);
        }
    }

    @Test
    void facadeConveniencesBuildAndExecuteTypedCreateGetAndDestroy() throws Exception {
        int unusedPort;
        try (ServerSocket reservation = new ServerSocket(0)) {
            unusedPort = reservation.getLocalPort();
        }
        byte[] clientCertificate = fixture("client-cert.pem");
        byte[] clientPrivateKey = fixture("client-key.pem");
        byte[] serverCertificate = fixture("server-cert.pem");

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", unusedPort)
                        .clientIdentityPem(new byte[][] {clientCertificate}, clientPrivateKey)
                        .trustCertificatesPem(new byte[][] {serverCertificate})
                        .build();
                KmipClient client = KmipClient.create(configuration, transport)) {
            CodecLimits limits = CodecLimits.defaults();
            UniqueIdentifier id = UniqueIdentifier.textString("fluent-id");
            try (ClientBatchItem create = client.request().createSymmetricKey().aes256()
                            .usageMask((int) CryptographicUsageMask.ENCRYPT).build();
                    ClientBatchItem get = client.request().get(id).build();
                    ClientBatchItem destroy = client.request().destroy(id).build();
                    ClientBatchItem getCopy = client.request().get(id).toBatchItem()) {
                assertEquals(1, create.operation());
                assertEquals(10, get.operation());
                assertEquals(20, destroy.operation());
                assertEquals(10, getCopy.operation());
            }
            assertThrows(ClientOperationException.class, () -> client.createSymmetricKey(
                    SymmetricKeyRequest.named(3, 256, (int) CryptographicUsageMask.ENCRYPT, "named"), limits));
            assertThrows(ClientOperationException.class, () -> client.createSymmetricKey(
                    SymmetricKeyRequest.unnamed(3, 256, (int) CryptographicUsageMask.ENCRYPT), limits,
                    RequestOptions.defaults()));
            assertThrows(ClientOperationException.class, () -> client.getById(id, limits));
            assertThrows(ClientOperationException.class,
                    () -> client.getById(id, limits, RequestOptions.defaults()));
            assertThrows(ClientOperationException.class, () -> client.destroyById(id, limits));
            assertThrows(ClientOperationException.class,
                    () -> client.destroyById(id, limits, RequestOptions.defaults()));
            assertThrows(InvalidInputException.class, () -> client.createSymmetricKey(null, limits));
            assertThrows(InvalidInputException.class,
                    () -> client.createSymmetricKey(null, limits, RequestOptions.defaults()));
            assertThrows(InvalidInputException.class, () -> client.getById(null, limits));
            assertThrows(InvalidInputException.class, () -> client.destroyById(id, null));
            assertThrows(InvalidInputException.class, () -> client.destroyById(null, limits));

            assertThrows(ClientOperationException.class,
                    () -> client.request().createSymmetricKey().aes256()
                            .usageMask((int) CryptographicUsageMask.ENCRYPT).execute(limits));
            assertThrows(ClientOperationException.class,
                    () -> client.request().createSymmetricKey().aes256()
                            .usageMask((int) CryptographicUsageMask.ENCRYPT)
                            .withOptions(RequestOptions.defaults()).execute(limits));
            assertThrows(ClientOperationException.class,
                    () -> client.request().get(id).asynchronousIndicator(1).execute(limits));
            assertThrows(ClientOperationException.class,
                    () -> client.request().destroy(id).withOptions(RequestOptions.defaults())
                            .asynchronousIndicator(1).execute(limits));
            assertThrows(InvalidInputException.class,
                    () -> client.request().get(id).asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> client.request().destroy(id).asynchronousIndicator(0x1_0000_0000L));

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.discoverVersions());
                assertThrows(ClientOperationException.class, () -> client.execute(batch, limits));
            }
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.discoverVersions());
                assertThrows(ClientOperationException.class,
                        () -> client.executeWithOptions(batch, limits, RequestOptions.defaults()));
            }
        }
    }

    private static byte[] fixture(String filename) throws IOException {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IOException("shared mTLS fixture not found: " + filename);
    }

    private static SSLServerSocket mtlsListener(
            byte[] serverCertificatePem, byte[] serverPrivateKeyPem, byte[] clientCertificatePem)
            throws Exception {
        char[] password = "kmipkit-test".toCharArray();
        X509Certificate serverCertificate = certificate(serverCertificatePem);
        X509Certificate clientCertificate = certificate(clientCertificatePem);
        KeyStore identity = KeyStore.getInstance("PKCS12");
        identity.load(null, password);
        identity.setKeyEntry("server", privateKey(serverPrivateKeyPem), password,
                new java.security.cert.Certificate[] {serverCertificate});
        KeyManagerFactory keyManagers = KeyManagerFactory.getInstance(
                KeyManagerFactory.getDefaultAlgorithm());
        keyManagers.init(identity, password);
        KeyStore trustedClients = KeyStore.getInstance("PKCS12");
        trustedClients.load(null, password);
        trustedClients.setCertificateEntry("client", clientCertificate);
        TrustManagerFactory trustManagers = TrustManagerFactory.getInstance(
                TrustManagerFactory.getDefaultAlgorithm());
        trustManagers.init(trustedClients);
        SSLContext context = SSLContext.getInstance("TLSv1.3");
        context.init(keyManagers.getKeyManagers(), trustManagers.getTrustManagers(), null);
        SSLServerSocket listener = (SSLServerSocket) context.getServerSocketFactory()
                .createServerSocket(0, 2, java.net.InetAddress.getByName("127.0.0.1"));
        listener.setEnabledProtocols(new String[] {"TLSv1.3"});
        listener.setNeedClientAuth(true);
        listener.setSoTimeout(10_000);
        return listener;
    }

    private static X509Certificate certificate(byte[] pem) throws Exception {
        return (X509Certificate) CertificateFactory.getInstance("X.509")
                .generateCertificate(new ByteArrayInputStream(pem));
    }

    private static PrivateKey privateKey(byte[] pem) throws Exception {
        String encoded = new String(pem, java.nio.charset.StandardCharsets.US_ASCII)
                .replaceAll("-----BEGIN [^-]+-----", "")
                .replaceAll("-----END [^-]+-----", "")
                .replaceAll("\\s", "");
        return KeyFactory.getInstance("RSA").generatePrivate(
                new PKCS8EncodedKeySpec(Base64.getDecoder().decode(encoded)));
    }

    private static List<byte[]> serveAttributeReads(SSLServerSocket listener) throws Exception {
        ArrayList<byte[]> requests = new ArrayList<>(2);
        for (int index = 0; index < 2; index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readFrame(socket.getInputStream());
                requests.add(request);
                byte[] response = index == 0
                        ? getAttributesResponse(findUniqueBatchItemId(request))
                        : getAttributeListResponse(findUniqueBatchItemId(request));
                socket.getOutputStream().write(response);
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static byte[] readFrame(InputStream input) throws IOException {
        DataInputStream data = new DataInputStream(input);
        byte[] header = data.readNBytes(8);
        if (header.length != 8) throw new IOException("incomplete TTLV header");
        int valueLength = ByteBuffer.wrap(header, 4, 4).getInt();
        int paddedLength = Math.toIntExact((Integer.toUnsignedLong(valueLength) + 7) & ~7L);
        byte[] body = data.readNBytes(paddedLength);
        if (body.length != paddedLength) throw new IOException("incomplete TTLV body");
        byte[] frame = new byte[header.length + body.length];
        System.arraycopy(header, 0, frame, 0, header.length);
        System.arraycopy(body, 0, frame, header.length, body.length);
        return frame;
    }

    private static byte[] findUniqueBatchItemId(byte[] request) {
        for (int index = 0; index <= request.length - 8; index++) {
            if (request[index] == 0x42 && request[index + 1] == 0x00
                    && request[index + 2] == (byte) 0x93 && request[index + 3] == 8) {
                int length = ByteBuffer.wrap(request, index + 4, 4).getInt();
                if (length <= 0 || length > 64) return null;
                int paddedLength = (length + 7) & ~7;
                byte[] item = new byte[8 + paddedLength];
                System.arraycopy(request, index, item, 0, item.length);
                return item;
            }
        }
        return null;
    }

    private static byte[] getAttributesResponse(byte[] uniqueBatchItemId) {
        byte[] version = structureTtlv(0x420069,
                integerTtlv(0x42006a, 2), integerTtlv(0x42006b, 1));
        byte[] header = structureTtlv(0x42007a, version,
                longTtlv(0x420092, 1), integerTtlv(0x42000d, 1));
        byte[] attributes = structureTtlv(0x420125, integerTtlv(0x42002a, 256));
        byte[] payload = structureTtlv(0x42007c,
                textTtlv(0x420094, "attribute-read-key"), attributes);
        byte[] operation = enumTtlv(0x42005c, 11);
        byte[] status = enumTtlv(0x42007f, 0);
        byte[] responseItem = uniqueBatchItemId == null
                ? structureTtlv(0x42000f, operation, status, payload)
                : structureTtlv(0x42000f, operation, uniqueBatchItemId, status, payload);
        return structureTtlv(0x42007b, header, responseItem);
    }

    private static byte[] getAttributeListResponse(byte[] uniqueBatchItemId) {
        byte[] version = structureTtlv(0x420069,
                integerTtlv(0x42006a, 2), integerTtlv(0x42006b, 1));
        byte[] header = structureTtlv(0x42007a, version,
                longTtlv(0x420092, 1), integerTtlv(0x42000d, 1));
        byte[] payload = structureTtlv(0x42007c,
                textTtlv(0x420094, "attribute-read-key"), enumTtlv(0x42013b, 0x42002a));
        byte[] operation = enumTtlv(0x42005c, 12);
        byte[] status = enumTtlv(0x42007f, 0);
        byte[] responseItem = uniqueBatchItemId == null
                ? structureTtlv(0x42000f, operation, status, payload)
                : structureTtlv(0x42000f, operation, uniqueBatchItemId, status, payload);
        return structureTtlv(0x42007b, header, responseItem);
    }

    private static byte[] structureTtlv(int tag, byte[]... children) {
        return ttlv(tag, 1, concatenate(children));
    }

    private static byte[] integerTtlv(int tag, int value) {
        return ttlv(tag, 2, ByteBuffer.allocate(4).putInt(value).array());
    }

    private static byte[] enumTtlv(int tag, int value) {
        return ttlv(tag, 5, ByteBuffer.allocate(4).putInt(value).array());
    }

    private static byte[] longTtlv(int tag, long value) {
        return ttlv(tag, 9, ByteBuffer.allocate(8).putLong(value).array());
    }

    private static byte[] textTtlv(int tag, String value) {
        return ttlv(tag, 7, value.getBytes(java.nio.charset.StandardCharsets.UTF_8));
    }

    private static byte[] ttlv(int tag, int type, byte[] value) {
        int paddedLength = (value.length + 7) & ~7;
        ByteBuffer item = ByteBuffer.allocate(8 + paddedLength);
        item.put((byte) (tag >>> 16)).put((byte) (tag >>> 8)).put((byte) tag);
        item.put((byte) type).putInt(value.length).put(value);
        return item.array();
    }

    private static byte[] concatenate(byte[][] values) {
        int size = 0;
        for (byte[] value : values) size = Math.addExact(size, value.length);
        ByteBuffer result = ByteBuffer.allocate(size);
        for (byte[] value : values) result.put(value);
        return result.array();
    }

    private static boolean contains(byte[] bytes, byte[] sequence) {
        return indexOf(bytes, sequence) >= 0;
    }

    private static int indexOf(byte[] bytes, byte[] sequence) {
        outer: for (int start = 0; start <= bytes.length - sequence.length; start++) {
            for (int index = 0; index < sequence.length; index++) {
                if (bytes[start + index] != sequence[index]) continue outer;
            }
            return start;
        }
        return -1;
    }
}
