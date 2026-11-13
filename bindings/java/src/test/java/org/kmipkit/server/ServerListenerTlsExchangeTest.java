package org.kmipkit.server;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.DataOutputStream;
import java.net.InetAddress;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyFactory;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.security.spec.PKCS8EncodedKeySpec;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Base64;
import java.util.List;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;
import org.junit.jupiter.api.Test;
import org.kmipkit.ProtocolException;
import org.kmipkit.ClientOperationException;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

final class ServerListenerTlsExchangeTest {
    private static final char[] PASSWORD = "kmipkit-test".toCharArray();

    @Test
    void acceptHonorsItsTimeoutAndListenerCloseIsIdempotent() throws Exception {
        Path fixtures = fixtureDirectory();
        byte[] authority = read(fixtures.resolve("ca.pem"));
        ServerListenerConfiguration configuration = ServerListenerConfiguration.builder()
                .bindHost("127.0.0.1").bindPort(0)
                .listenerCertificateChain(List.of(read(fixtures.resolve("server-cert.pem"))))
                .listenerPrivateKey(read(fixtures.resolve("server-key.pem")))
                .trustCertificates(List.of(authority)).build();
        try (configuration; ServerListener listener = ServerListener.bindRawTls(configuration)) {
            assertThrows(ClientOperationException.class,
                    () -> listener.accept(Duration.ofMillis(10)));
            listener.close();
            listener.close();
        }
    }

    @Test
    void servesTypedDiscoverVersionsOverTls13MutualAuthentication() throws Exception {
        byte[] response = exchange(request -> {
            assertTrue(request instanceof DiscoverVersionsRequest);
            DiscoverVersionsRequest discover = (DiscoverVersionsRequest) request;
            assertEquals(List.of(), discover.requestedVersions());
            return ServerReply.success(DiscoverVersionsResponse.forRequest(
                    discover, List.of(new ProtocolVersion(2, 1))));
        });
        assertNotNull(response);
        assertTrue(contains(response, hex("42007f050000000400000000")), "response should be successful");
        assertTrue(contains(response, hex("42006a020000000400000002")), "major version should be 2");
        assertTrue(contains(response, hex("42006b020000000400000001")), "minor version should be 1");
    }

    @Test
    void servesAllFiveTypedOperationsOverTls13MutualAuthentication() throws Exception {
        List<Class<?>> seen = new ArrayList<>();
        byte[] request = requestBatch(
                batchItem(30, "batch-01", structure(0x420079,
                        structure(0x420069, integer(0x42006a, 2), integer(0x42006b, 1)))),
                batchItem(27, "batch-02", structure(0x420079,
                        item(0x420094, 7, "object-01".getBytes(StandardCharsets.US_ASCII)),
                        structure(0x420125, integer(0x42002a, 256)),
                        structure(0x420126, enumeration(0x42000a, 0x420028)),
                        structure(0x420126,
                                item(0x42009d, 7, "KMIPKit".getBytes(StandardCharsets.US_ASCII)),
                                item(0x42000a, 7, "Comment".getBytes(StandardCharsets.US_ASCII))))),
                batchItem(28, "batch-03", structure(0x420079,
                        enumeration(0x420070, 1),
                        item(0x420094, 7, "object-02".getBytes(StandardCharsets.US_ASCII)),
                        structure(0x42008f,
                                structure(0x420040, enumeration(0x420042, 1))))),
                batchItem(24, "batch-04", structure(0x420079, enumeration(0x420074, 1))),
                batchItem(50, "batch-05", structure(0x420079, enumeration(0x420151, 2))));
        byte[] response = exchange(request, requestItem -> {
            seen.add(requestItem.getClass());
            if (requestItem instanceof DiscoverVersionsRequest discover) {
                assertEquals(List.of(new ProtocolVersion(2, 1)), discover.requestedVersions());
                return ServerReply.success(DiscoverVersionsResponse.forRequest(
                        discover, List.of(new ProtocolVersion(2, 1))));
            }
            if (requestItem instanceof NotifyRequest notify) {
                assertEquals(List.of(
                        org.kmipkit.extensions.AttributeReference.tag(0x420028),
                        org.kmipkit.extensions.AttributeReference.name("KMIPKit", "Comment")),
                        notify.deletedAttributeReferences());
                return ServerReply.success(NotifyResponse.forRequest(notify));
            }
            if (requestItem instanceof PutRequest put) {
                assertEquals(1, put.putFunction());
                try (PutObject object = put.putObject()) {
                    assertEquals(0x42008f, object.tag());
                }
                return ServerReply.success(PutResponse.forRequest(put));
            }
            if (requestItem instanceof QueryRequest query) {
                assertEquals(List.of(1), query.functions());
                return ServerReply.success(QueryResponse.forRequest(
                        query, List.of(24, 30), List.of(2), List.of(), "KMIPKit"));
            }
            SetEndpointRoleRequest role = (SetEndpointRoleRequest) requestItem;
            assertEquals(2, role.endpointRole());
            return ServerReply.success(SetEndpointRoleResponse.forRequest(role, 1, null));
        });

        assertEquals(List.of(DiscoverVersionsRequest.class, NotifyRequest.class, PutRequest.class,
                QueryRequest.class, SetEndpointRoleRequest.class), seen);
        assertEquals(5, countOccurrences(response, hex("42007f050000000400000000")));
        assertTrue(contains(response, hex("420151050000000400000001")),
                "typed Set Endpoint Role response should be encoded");
    }

    @Test
    void sendsAnExplicitTypedHandlerFailureOverTls13MutualAuthentication() throws Exception {
        byte[] response = exchange(request -> ServerReply.failure(
                ServerResultStatus.OPERATION_FAILED,
                ServerResultReason.OPERATION_NOT_SUPPORTED,
                "unsupported"));
        assertTrue(contains(response, hex("42007f050000000400000001")), "failure status should be encoded");
        assertTrue(contains(response, hex("42007e050000000400000005")), "KMIP reason 5 should be encoded");
    }

    @Test
    void preservesNotifyChangedAttributesOverTls13MutualAuthentication() throws Exception {
        byte[] response = exchange(notifyRequest(), request -> {
            assertTrue(request instanceof NotifyRequest);
            NotifyRequest notify = (NotifyRequest) request;
            AttributeSet attributes = notify.attributes();
            assertNotNull(attributes);
            try (attributes; TtlvStructure items = attributes.asItems();
                    TtlvStructureView view = items.view()) {
                assertEquals(1, view.itemCount());
            }
            return ServerReply.success(NotifyResponse.forRequest(notify));
        });
        assertTrue(contains(response, hex("42007f050000000400000000")),
                "Notify response should be successful");
    }

    @Test
    void rejectsMalformedTtlvOverTls13MutualAuthentication() throws Exception {
        Path fixtures = fixtureDirectory();
        byte[] authority = read(fixtures.resolve("ca.pem"));
        ServerListenerConfiguration configuration = ServerListenerConfiguration.builder()
                .bindHost("127.0.0.1").bindPort(0)
                .listenerCertificateChain(List.of(read(fixtures.resolve("server-cert.pem"))))
                .listenerPrivateKey(read(fixtures.resolve("server-key.pem")))
                .trustCertificates(List.of(authority)).build();
        try (configuration; ServerListener listener = ServerListener.bindRawTls(configuration)) {
            int port = listener.port();
            FutureTask<Boolean> server = new FutureTask<>(() -> {
                try (ServerConnection connection = listener.accept(Duration.ofSeconds(5))) {
                    try (ServerRequestBatch batch = connection.receive(
                            CodecLimits.defaults(), Duration.ofSeconds(5))) {
                        assertNotNull(batch);
                        return false;
                    } catch (ProtocolException expected) {
                        return true;
                    }
                }
            });
            Thread worker = new Thread(server, "kmipkit-java-malformed-listener-fixture");
            worker.setDaemon(true);
            worker.start();
            try (SSLSocket client = (SSLSocket) clientContext(authority,
                    read(fixtures.resolve("client-cert.pem")), read(fixtures.resolve("client-key.pem")))
                    .getSocketFactory().createSocket("127.0.0.1", port)) {
                client.setEnabledProtocols(new String[] {"TLSv1.3"});
                client.setSoTimeout(5000);
                client.startHandshake();
                client.getOutputStream().write(hex("42007801000000010000000000000000"));
                client.getOutputStream().flush();
                assertTrue(server.get(8, TimeUnit.SECONDS), "malformed TTLV should be rejected");
            }
        }
    }

    private static byte[] exchange(ServerRequestHandler handler) throws Exception {
        return exchange(discoverVersionsRequest(), handler);
    }

    private static byte[] exchange(byte[] requestBytes, ServerRequestHandler handler)
            throws Exception {
        Path fixtures = fixtureDirectory();
        byte[] serverCertificate = read(fixtures.resolve("server-cert.pem"));
        byte[] serverKey = read(fixtures.resolve("server-key.pem"));
        byte[] authority = read(fixtures.resolve("ca.pem"));
        byte[] clientCertificate = read(fixtures.resolve("client-cert.pem"));
        byte[] clientKey = read(fixtures.resolve("client-key.pem"));
        ServerListenerConfiguration configuration = ServerListenerConfiguration.builder()
                .bindHost("127.0.0.1")
                .bindPort(0)
                .listenerCertificateChain(List.of(serverCertificate))
                .listenerPrivateKey(serverKey)
                .trustCertificates(List.of(authority))
                .build();

        try (configuration; ServerListener listener = ServerListener.bindRawTls(configuration)) {
            int port = listener.port();
            FutureTask<Throwable> server = new FutureTask<>(() -> {
                try (ServerConnection connection = listener.accept(Duration.ofSeconds(5));
                        ServerRequestBatch batch = connection.receive(
                                CodecLimits.defaults(), Duration.ofSeconds(5));
                        ServerResponseMessage response = batch.respond(handler)) {
                    connection.send(response, Duration.ofSeconds(5));
                }
                return null;
            });
            Thread worker = new Thread(server, "kmipkit-java-server-listener-fixture");
            worker.setDaemon(true);
            worker.start();

            byte[] response;
            try (SSLSocket client = (SSLSocket) clientContext(authority,
                    clientCertificate, clientKey).getSocketFactory()
                    .createSocket("127.0.0.1", port)) {
                client.setEnabledProtocols(new String[] {"TLSv1.3"});
                client.setSoTimeout(5000);
                client.startHandshake();
                client.getOutputStream().write(requestBytes);
                client.getOutputStream().flush();
                assertTrue(server.get(8, TimeUnit.SECONDS) == null, "listener worker should complete");
                response = readFrame(client);
            }
            return response;
        }
    }

    private static SSLContext clientContext(byte[] authority, byte[] certificate, byte[] key)
            throws Exception {
        CertificateFactory factory = CertificateFactory.getInstance("X.509");
        X509Certificate trustCertificate = (X509Certificate) factory.generateCertificate(
                new java.io.ByteArrayInputStream(authority));
        X509Certificate clientCertificate = (X509Certificate) factory.generateCertificate(
                new java.io.ByteArrayInputStream(certificate));
        PrivateKey privateKey = privateKey(key);
        KeyStore trustStore = KeyStore.getInstance("PKCS12");
        trustStore.load(null, PASSWORD);
        trustStore.setCertificateEntry("ca", trustCertificate);
        TrustManagerFactory trustManagers = TrustManagerFactory.getInstance(
                TrustManagerFactory.getDefaultAlgorithm());
        trustManagers.init(trustStore);
        KeyStore identityStore = KeyStore.getInstance("PKCS12");
        identityStore.load(null, PASSWORD);
        identityStore.setKeyEntry("client", privateKey, PASSWORD,
                new Certificate[] {clientCertificate});
        KeyManagerFactory keyManagers = KeyManagerFactory.getInstance(
                KeyManagerFactory.getDefaultAlgorithm());
        keyManagers.init(identityStore, PASSWORD);
        SSLContext context = SSLContext.getInstance("TLSv1.3");
        context.init(keyManagers.getKeyManagers(), trustManagers.getTrustManagers(), null);
        return context;
    }

    private static PrivateKey privateKey(byte[] pem) throws Exception {
        String encoded = new String(pem, StandardCharsets.US_ASCII)
                .replace("-----BEGIN PRIVATE KEY-----", "")
                .replace("-----END PRIVATE KEY-----", "")
                .replaceAll("\\s", "");
        return KeyFactory.getInstance("RSA").generatePrivate(
                new PKCS8EncodedKeySpec(Base64.getDecoder().decode(encoded)));
    }

    private static byte[] discoverVersionsRequest() throws Exception {
        byte[] version = structure(0x420069, integer(0x42006a, 2), integer(0x42006b, 1));
        byte[] header = structure(0x420077, version, integer(0x42000d, 1));
        byte[] batch = structure(0x42000f, enumeration(0x42005c, 30),
                item(0x420093, 8, "batch-01".getBytes(StandardCharsets.US_ASCII)),
                structure(0x420079));
        return structure(0x420078, header, batch);
    }

    private static byte[] requestBatch(byte[]... batchItems) throws Exception {
        byte[] version = structure(0x420069, integer(0x42006a, 2), integer(0x42006b, 1));
        byte[] header = structure(0x420077, version, integer(0x42000d, batchItems.length));
        byte[][] children = new byte[batchItems.length + 1][];
        children[0] = header;
        System.arraycopy(batchItems, 0, children, 1, batchItems.length);
        return structure(0x420078, children);
    }

    private static byte[] batchItem(int operation, String batchId, byte[] payload)
            throws Exception {
        return structure(0x42000f,
                enumeration(0x42005c, operation),
                item(0x420093, 8, batchId.getBytes(StandardCharsets.US_ASCII)),
                payload);
    }

    private static byte[] notifyRequest() throws Exception {
        byte[] payload = structure(0x420079,
                item(0x420094, 7, "object-01".getBytes(StandardCharsets.US_ASCII)),
                structure(0x420125, integer(0x42002a, 256)));
        byte[] header = structure(0x420077,
                structure(0x420069, integer(0x42006a, 2), integer(0x42006b, 1)),
                integer(0x42000d, 1));
        byte[] batch = structure(0x42000f, enumeration(0x42005c, 27),
                item(0x420093, 8, "batch-01".getBytes(StandardCharsets.US_ASCII)), payload);
        return structure(0x420078, header, batch);
    }

    private static byte[] readFrame(SSLSocket socket) throws Exception {
        DataInputStream input = new DataInputStream(socket.getInputStream());
        byte[] header = input.readNBytes(8);
        if (header.length != 8) throw new AssertionError("response header was incomplete");
        long valueLength = Integer.toUnsignedLong(ByteBuffer.wrap(header, 4, 4)
                .order(ByteOrder.BIG_ENDIAN).getInt());
        int paddedLength = Math.toIntExact((valueLength + 7) & ~7L);
        byte[] body = input.readNBytes(paddedLength);
        if (body.length != paddedLength) throw new AssertionError("response body was incomplete");
        ByteArrayOutputStream result = new ByteArrayOutputStream(header.length + body.length);
        result.write(header);
        result.write(body);
        return result.toByteArray();
    }

    private static byte[] structure(int tag, byte[]... children) throws Exception {
        ByteArrayOutputStream value = new ByteArrayOutputStream();
        for (byte[] child : children) value.write(child);
        return item(tag, 1, value.toByteArray());
    }

    private static byte[] integer(int tag, int value) throws Exception {
        return item(tag, 2, ByteBuffer.allocate(4).putInt(value).array());
    }

    private static byte[] enumeration(int tag, int value) throws Exception {
        return item(tag, 5, ByteBuffer.allocate(4).putInt(value).array());
    }

    private static byte[] item(int tag, int type, byte[] value) throws Exception {
        int padding = (-value.length) & 7;
        ByteArrayOutputStream result = new ByteArrayOutputStream();
        DataOutputStream output = new DataOutputStream(result);
        output.writeByte(tag >>> 16);
        output.writeByte(tag >>> 8);
        output.writeByte(tag);
        output.writeByte(type);
        output.writeInt(value.length);
        output.write(value);
        output.write(new byte[padding]);
        return result.toByteArray();
    }

    private static Path fixtureDirectory() throws Exception {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(Path.of("bindings", "python", "tests", "fixtures",
                    "server-listener-mtls"));
            if (Files.isDirectory(candidate)) return candidate;
            directory = directory.getParent();
        }
        throw new IllegalStateException("server listener TLS fixtures are missing");
    }

    private static byte[] read(Path path) throws Exception { return Files.readAllBytes(path); }

    private static byte[] hex(String value) {
        byte[] result = new byte[value.length() / 2];
        for (int index = 0; index < result.length; index++) {
            result[index] = (byte) Integer.parseInt(value.substring(index * 2, index * 2 + 2), 16);
        }
        return result;
    }

    private static boolean contains(byte[] value, byte[] needle) {
        outer: for (int index = 0; index <= value.length - needle.length; index++) {
            for (int offset = 0; offset < needle.length; offset++) {
                if (value[index + offset] != needle[offset]) continue outer;
            }
            return true;
        }
        return false;
    }

    private static int countOccurrences(byte[] value, byte[] needle) {
        int count = 0;
        for (int start = 0; start <= value.length - needle.length; start++) {
            if (containsAt(value, needle, start)) count++;
        }
        return count;
    }

    private static boolean containsAt(byte[] value, byte[] needle, int start) {
        for (int offset = 0; offset < needle.length; offset++) {
            if (value[start + offset] != needle[offset]) return false;
        }
        return true;
    }
}
