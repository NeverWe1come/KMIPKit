package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

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
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.SecureRandom;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.Base64;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ControlledTlsPeerTest {
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

        ArtifactProvenance.load(manifestPath, "9692057c9ade474fb0c4e0b7130539da368eb794");
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
        assertEquals("passed", outcome.status(), outcome.toString());
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x5c, 0x05, 0, 0, 0, 4, 0, 0, 0, 0x18}),
                "The controlled peer must receive the typed Query operation.");
        assertTrue(contains(request, new byte[] {0x42, 0x00, 0x74, 0x05, 0, 0, 0, 4, 0, 0, 0, 1}),
                "The Query request must include a non-empty Query Function.");
    }

    private String connectionJson(int port, byte[] serverCertificate, byte[] clientCertificate,
                                  byte[] clientPrivateKey) throws Exception {
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
        root.putObject("execution_policy").put("allow_mutations", false);
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
                item(0x42007f, 5, intBytes(0)), structure(0x42007c));
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
}
