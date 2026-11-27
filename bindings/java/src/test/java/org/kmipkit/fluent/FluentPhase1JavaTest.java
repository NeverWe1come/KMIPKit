package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.*;

import java.io.ByteArrayInputStream;
import java.io.DataInputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
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
import java.util.OptionalLong;
import java.util.concurrent.FutureTask;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ServerResultException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValueView;

final class FluentPhase1JavaTest {
    @Test
    void clientConfigurationEmbedsAndUpdatesCodecLimits() {
        try (ClientConfiguration defaults = ClientConfiguration.defaults()) {
            assertEquals(CodecLimits.defaults().maxMessageBytes(), defaults.codecLimits().maxMessageBytes());
            CodecLimits custom = CodecLimits.create(1024 * 1024L, 16L, 2048L);
            try (ClientConfiguration updated = defaults.withCodecLimits(custom);
                    ClientConfiguration v3 = updated.withProtocolVersion(new ProtocolVersion(3, 0))) {
                assertEquals(1024 * 1024L, updated.codecLimits().maxMessageBytes());
                assertEquals(1024 * 1024L, v3.codecLimits().maxMessageBytes());
                assertEquals(new ProtocolVersion(3, 0), v3.protocolVersion());
            }
        }
    }

    @Test
    void connectMtlsValidatesMissingPemPathsWithoutNetwork() {
        assertThrows(InvalidInputException.class, () -> KmipClient.connectMtls(
                "127.0.0.1",
                5696,
                Path.of("/nonexistent/kmipkit/ca.pem"),
                Path.of("/nonexistent/kmipkit/cert.pem"),
                Path.of("/nonexistent/kmipkit/key.pem")));
    }

    @Test
    void serverResultExceptionCapturesPendingAndRejectionMetadata() {
        ServerResultException rejected = ServerResultException.fromServerResult(1L, OptionalLong.of(5L));
        assertTrue(rejected.isServerRejected());
        assertFalse(rejected.isPending());
        assertEquals(1L, rejected.resultStatus());
        assertEquals(OptionalLong.of(5L), rejected.resultReason());

        ServerResultException pending = ServerResultException.fromServerResult(2L, OptionalLong.empty());
        assertTrue(pending.isPending());
        assertFalse(pending.isServerRejected());
        assertEquals(2L, pending.resultStatus());
        assertTrue(pending.resultReason().isEmpty());
    }

    @Test
    void kmip30BuilderExecuteAndExecuteOutcomeOverMtlsAndEmitsAlternativeName() throws Exception {
        byte[] serverCert = fixture("server-cert.pem");
        byte[] serverKey = fixture("server-key.pem");
        byte[] clientCert = fixture("client-cert.pem");
        byte[] clientKey = fixture("client-key.pem");

        SSLServerSocket listener = mtlsListener(serverCert, serverKey, clientCert);
        FutureTask<List<byte[]>> peer = new FutureTask<>(() -> serveCreateResponses(listener));
        Thread peerThread = new Thread(peer, "kmipkit-java-phase1-mtls");
        peerThread.setDaemon(true);
        peerThread.start();

        List<byte[]> requests;
        try (KmipClient client = KmipClient.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .caPem(new byte[][] {serverCert})
                        .clientIdentityPem(new byte[][] {clientCert}, clientKey)
                        .protocolVersion(new ProtocolVersion(3, 0))
                        .codecLimits(CodecLimits.create(2 * 1024 * 1024L, 64L, 4096L))
                        .build()) {
            assertEquals(new ProtocolVersion(3, 0), client.protocolVersion());
            assertEquals(2 * 1024 * 1024L, client.codecLimits().maxMessageBytes());

            try (SymmetricKeyBuilder okBuilder = client.request().createSymmetricKey()
                            .aes256()
                            .usageMask(KmipValues.CryptographicUsageMask.ENCRYPT
                                    | KmipValues.CryptographicUsageMask.DECRYPT)
                            .name("k-30");
                    ClientBatchItemResult created = okBuilder.execute()) {
                assertEquals(0L, created.resultStatus());
                assertEquals(1L, created.operation());
            }

            try (SymmetricKeyBuilder errBuilder = client.request().createSymmetricKey()
                            .aes256()
                            .usageMask(KmipValues.CryptographicUsageMask.ENCRYPT)
                            .name("k-30-fail")) {
                ServerResultException failure = assertThrows(ServerResultException.class, errBuilder::execute);
                assertTrue(failure.isServerRejected());
                assertEquals(1L, failure.resultStatus());
                assertEquals(OptionalLong.of(5L), failure.resultReason());
            }

            try (SymmetricKeyBuilder outcomeBuilder = client.request().createSymmetricKey()
                            .aes256()
                            .usageMask(KmipValues.CryptographicUsageMask.ENCRYPT)
                            .name("k-30-outcome");
                    ClientBatchItemResult outcome = outcomeBuilder.executeOutcome()) {
                assertEquals(1L, outcome.resultStatus());
                assertEquals(OptionalLong.of(5L), outcome.resultReason());
            }

            requests = peer.get();
        } finally {
            listener.close();
        }

        assertEquals(3, requests.size());
        try (TtlvItem decoded = TtlvCodec.decode(requests.get(0));
                TtlvItemView rootView = decoded.view()) {
            assertTrue(containsTag(rootView, 0x4200bfL),
                    "KMIP 3.0 Create request must contain AlternativeName (0x4200bf)");
            assertFalse(containsTag(rootView, 0x420053L),
                    "KMIP 3.0 Create request must not contain legacy Name (0x420053)");
        }
    }

    private static boolean containsTag(TtlvItemView view, long targetTag) {
        if (view.tag().raw() == targetTag) {
            return true;
        }
        try (TtlvValueView val = view.value()) {
            if (val.itemType() != TtlvItemType.Structure) {
                return false;
            }
            try (TtlvStructureView st = val.structure()) {
                for (long i = 0; i < st.itemCount(); i++) {
                    try (TtlvItemView child = st.itemAt(i)) {
                        if (containsTag(child, targetTag)) {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }

    private static byte[] fixture(String filename) throws IOException {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "shared", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IOException("shared mTLS fixture not found: " + filename);
    }

    private static SSLServerSocket mtlsListener(
            byte[] serverCertPem, byte[] serverKeyPem, byte[] clientCertPem) throws Exception {
        char[] password = "kmipkit-test".toCharArray();
        X509Certificate serverCert = certificate(serverCertPem);
        X509Certificate clientCert = certificate(clientCertPem);
        KeyStore identity = KeyStore.getInstance("PKCS12");
        identity.load(null, password);
        identity.setKeyEntry("server", privateKey(serverKeyPem), password,
                new java.security.cert.Certificate[] {serverCert});
        KeyManagerFactory kmf = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        kmf.init(identity, password);
        KeyStore trust = KeyStore.getInstance("PKCS12");
        trust.load(null, password);
        trust.setCertificateEntry("client", clientCert);
        TrustManagerFactory tmf = TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm());
        tmf.init(trust);
        SSLContext ctx = SSLContext.getInstance("TLSv1.3");
        ctx.init(kmf.getKeyManagers(), tmf.getTrustManagers(), null);
        SSLServerSocket listener = (SSLServerSocket) ctx.getServerSocketFactory()
                .createServerSocket(0, 4, java.net.InetAddress.getByName("127.0.0.1"));
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
        String encoded = new String(pem, StandardCharsets.US_ASCII)
                .replaceAll("-----BEGIN [^-]+-----", "")
                .replaceAll("-----END [^-]+-----", "")
                .replaceAll("\\s", "");
        return KeyFactory.getInstance("RSA").generatePrivate(
                new PKCS8EncodedKeySpec(Base64.getDecoder().decode(encoded)));
    }

    private static List<byte[]> serveCreateResponses(SSLServerSocket listener) throws Exception {
        ArrayList<byte[]> requests = new ArrayList<>(3);
        for (int idx = 0; idx < 3; idx++) {
            try (SSLSocket socket = (SSLSocket) listener.accept()) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                socket.setSoTimeout(10_000);
                socket.startHandshake();
                byte[] request = readFrame(socket.getInputStream());
                requests.add(request);
                byte[] batchId = findUniqueBatchItemId(request);
                byte[] response = idx == 0
                        ? createSuccessResponseV3(batchId, "k-30")
                        : createErrorResponseV3(batchId, 1, 5);
                socket.getOutputStream().write(response);
                socket.getOutputStream().flush();
            }
        }
        return List.copyOf(requests);
    }

    private static byte[] createSuccessResponseV3(byte[] uniqueBatchItemId, String idText) {
        byte[] version = structureTtlv(0x420069, integerTtlv(0x42006a, 3), integerTtlv(0x42006b, 0));
        byte[] header = structureTtlv(0x42007a, version, longTtlv(0x420092, 1));
        byte[] payload = structureTtlv(0x42007c, enumTtlv(0x420057, 2), identifierTtlv(0x420094, idText));
        byte[] item = uniqueBatchItemId == null
                ? structureTtlv(0x42000f, enumTtlv(0x42005c, 1), enumTtlv(0x42007f, 0), payload)
                : structureTtlv(0x42000f, enumTtlv(0x42005c, 1), uniqueBatchItemId, enumTtlv(0x42007f, 0), payload);
        return structureTtlv(0x42007b, header, item);
    }

    private static byte[] createErrorResponseV3(byte[] uniqueBatchItemId, int status, int reason) {
        byte[] version = structureTtlv(0x420069, integerTtlv(0x42006a, 3), integerTtlv(0x42006b, 0));
        byte[] header = structureTtlv(0x42007a, version, longTtlv(0x420092, 1));
        byte[] item = uniqueBatchItemId == null
                ? structureTtlv(0x42000f, enumTtlv(0x42005c, 1), enumTtlv(0x42007f, status), enumTtlv(0x42007e, reason))
                : structureTtlv(0x42000f, enumTtlv(0x42005c, 1), uniqueBatchItemId, enumTtlv(0x42007f, status), enumTtlv(0x42007e, reason));
        return structureTtlv(0x42007b, header, item);
    }

    private static byte[] readFrame(java.io.InputStream input) throws IOException {
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

    private static byte[] identifierTtlv(int tag, String value) {
        return ttlv(tag, 12, value.getBytes(StandardCharsets.UTF_8));
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
        byte[] result = new byte[size];
        int offset = 0;
        for (byte[] value : values) {
            System.arraycopy(value, 0, result, offset, value.length);
            offset += value.length;
        }
        return result;
    }
}
