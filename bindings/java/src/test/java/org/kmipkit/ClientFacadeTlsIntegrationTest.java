package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayOutputStream;
import java.io.DataInputStream;
import java.io.DataOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetAddress;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.SecureRandom;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.time.Duration;
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
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.ttlv.CodecLimits;

/** Exercises the T076 client ABI through the Java facade against a local TLS 1.3 peer. */
final class ClientFacadeTlsIntegrationTest {
    private static final char[] STORE_PASSWORD = "kmipkit-test".toCharArray();
    private static final byte[] CORRELATION = {0, 0x41, (byte) 0xff, 0x42};

    @TempDir
    Path temporaryDirectory;

    @Test
    void executionPreservesOrderedPendingAndCompletedMetadataOverMutualTls() throws Exception {
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
        FutureTask<byte[][]> peer = new FutureTask<>(() -> serveTwoResponses(listener));
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
                Client client = Client.create(extensions, transport)) {
            ClientBatch pendingBatch = ClientBatch.create();
            pendingBatch.asynchronousIndicator(2).add(ClientBatchItem.discoverVersions());
            ClientBatchResponse pendingResponse;
            try {
                pendingResponse = client.execute(pendingBatch, CodecLimits.defaults());
            } catch (ClientOperationException exception) {
                throw new AssertionError("Pending exchange failed with status " + exception.statusCode()
                        + ", category " + exception.categoryCode() + ", cause " + exception.causeCode()
                        + ", delivery " + exception.deliveryStateCode(), exception);
            }
            try (ClientBatchResponse response = pendingResponse;
                    ClientBatchItemResult pending = response.itemAt(0)) {
                assertEquals(1, response.itemCount());
                assertEquals(2, pending.resultStatus());
                assertTrue(pending.outcomeKind().isPending());
                assertArrayEquals(CORRELATION, pending.pendingCorrelationValue().orElseThrow());
                assertTrue(pending.resultReason().isEmpty());
            }

            ClientBatch completedBatch = ClientBatch.create();
            completedBatch.add(ClientBatchItem.discoverVersions());
            try (ClientBatchResponse response = client.execute(completedBatch, CodecLimits.defaults());
                    ClientBatchItemResult completed = response.itemAt(0)) {
                assertEquals(1, response.itemCount());
                assertEquals(0, completed.resultStatus());
                assertTrue(completed.outcomeKind().isCompleted());
                assertTrue(completed.pendingCorrelationValue().isEmpty());
                assertTrue(completed.resultReason().isEmpty());
            }
            byte[][] requests = peer.get(12, TimeUnit.SECONDS);
            assertEquals(2, requests.length);
            for (byte[] request : requests) {
                assertTrue(request.length >= 8);
                assertArrayEquals(new byte[] {0x42, 0x00, 0x78},
                        new byte[] {request[0], request[1], request[2]});
            }
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

    private static byte[][] serveTwoResponses(SSLServerSocket listener) throws Exception {
        byte[][] requests = new byte[2][];
        for (int index = 0; index < requests.length; index++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                requests[index] = readFrame(socket.getInputStream());
                OutputStream output = socket.getOutputStream();
                output.write(responseMessage(index == 0));
                output.flush();
            }
        }
        return requests;
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

    private static byte[] responseMessage(boolean pendingResult) throws Exception {
        byte[] version = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] header = structure(0x42007a, version, item(0x420092, 9, longInteger(1)),
                item(0x42000d, 2, integer(1)));
        byte[] responseVersion = structure(0x420069, item(0x42006a, 2, integer(2)),
                item(0x42006b, 2, integer(1)));
        byte[] responsePayload = structure(0x42007c, responseVersion);
        byte[] batchResult = pendingResult
                ? structure(0x42000f, item(0x42005c, 5, integer(30)), item(0x42007f, 5, integer(2)),
                        item(0x420006, 8, CORRELATION), responsePayload)
                : structure(0x42000f, item(0x42005c, 5, integer(30)), item(0x42007f, 5, integer(0)),
                        responsePayload);
        return structure(0x42007b, header, batchResult);
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

    private static byte[] longInteger(long value) {
        return ByteBuffer.allocate(Long.BYTES).putLong(value).array();
    }
}
