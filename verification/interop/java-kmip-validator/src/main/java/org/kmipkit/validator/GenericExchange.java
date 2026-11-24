package org.kmipkit.validator;

import org.kmipkit.message.ResponseMessage;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;
import java.io.ByteArrayOutputStream;
import java.io.EOFException;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.KeyFactory;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.spec.InvalidKeySpecException;
import java.security.spec.PKCS8EncodedKeySpec;
import java.util.Arrays;
import java.util.Base64;
import java.util.HexFormat;

final class GenericExchange {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final int LOCATE = 0x08;
    private static final int QUERY = 0x18;
    private static final int DISCOVER_VERSIONS = 0x1e;
    private static final int RNG_RETRIEVE = 0x25;
    private static final int HASH = 0x27;
    private static final byte[] HASH_INPUT = "abc".getBytes(StandardCharsets.US_ASCII);
    private static final byte[] HASH_DIGEST = HexFormat.of().parseHex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    private static final int PING = 0x3b;

    private GenericExchange() { }

    static boolean supports(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        return "server_exchange".equals(testCase.execution().mode())
                && ("read_only".equals(testCase.execution().mutation()) || operation(testCase, connection) == RNG_RETRIEVE)
                && operation(testCase, connection) != 0
                && "raw_tls".equals(connection.transport()) && "TTLV".equals(connection.encoding())
                && connection.usernamePassword() == null;
    }

    private static int operation(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        String version = connection.protocolVersion();
        if (!"2.0".equals(version) && !"2.1".equals(version)) return 0;
        return switch (testCase.caseId()) {
            case "test-op-locate" -> LOCATE;
            case "test-op-query" -> QUERY;
            case "test-op-discover-versions" -> DISCOVER_VERSIONS;
            case "test-crypto-rng-retrieve-rng-32" -> RNG_RETRIEVE;
            case "test-op-hash", "test-crypto-hash-sha_256-0" -> HASH;
            case "test-op-ping" -> "2.1".equals(version) ? PING : 0;
            default -> 0;
        };
    }

    static JavaKmipDriver.CaseOutcome run(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        long start = System.nanoTime();
        int minor = "2.1".equals(connection.protocolVersion()) ? 1 : 0;
        int operation = operation(testCase, connection);
        try (var request = request(operation, minor); var view = request.view()) {
            return check(exchange(connection, encode(view)), operation, minor, start);
        } catch (GeneralSecurityException | IOException | RuntimeException error) {
            return JavaKmipDriver.CaseOutcome.failed("Generic TTLV exchange or validation failed ("
                    + error.getClass().getSimpleName() + ").", elapsed(start));
        }
    }

    private static TtlvItem request(int operation, int minor) {
        try (var version = version(minor);
                var count = item(0x42000d, TtlvValue.Integer(1));
                var header = structureItem(0x420077, version, count);
                var identifier = item(0x42005c, TtlvValue.Enumeration(operation));
                var payload = switch (operation) {
                    case LOCATE -> structureItem(0x420079, item(0x42004f, TtlvValue.Integer(1)),
                            item(0x4200d4, TtlvValue.Integer(0)));
                    case QUERY -> structureItem(0x420079, item(0x420074, TtlvValue.Enumeration(1)));
                    case DISCOVER_VERSIONS -> structureItem(0x420079, version(minor));
                    case RNG_RETRIEVE -> structureItem(0x420079, item(0x4200c4, TtlvValue.Integer(32)));
                    case HASH -> hashPayload();
                    case PING -> structureItem(0x420079);
                    default -> throw new IllegalArgumentException("Unsupported generic operation");
                };
                var batch = structureItem(0x42000f, identifier, payload)) {
            return structureItem(0x420078, header, batch);
        }
    }

    private static TtlvItem hashPayload() {
        try (var parameters = structureItem(0x42002b, item(0x420038, TtlvValue.Enumeration(6)));
                var data = item(0x4200c2, TtlvValue.ByteString(HASH_INPUT, LIMITS))) {
            return structureItem(0x420079, parameters, data);
        }
    }

    private static TtlvItem version(int minor) {
        try (var major = item(0x42006a, TtlvValue.Integer(2));
                var minorItem = item(0x42006b, TtlvValue.Integer(minor))) {
            return structureItem(0x420069, major, minorItem);
        }
    }

    private static TtlvItem item(int code, TtlvValue value) {
        try (value; var raw = RawTag.fromRaw(code); var tag = raw.checked()) {
            return TtlvItem.create(tag, value, LIMITS);
        }
    }

    // withItem consumes its arguments. Closing already consumed handles is harmless.
    private static TtlvItem structureItem(int code, TtlvItem... items) {
        TtlvStructure structure = TtlvStructure.create();
        try {
            for (var child : items) {
                structure = TtlvStructure.withItem(structure, child, LIMITS);
            }
            return item(code, TtlvValue.structure(structure, LIMITS));
        } finally {
            structure.close();
            for (var child : items) child.close();
        }
    }

    private static byte[] encode(TtlvItemView item) {
        try (var tag = item.tag(); var value = item.value()) {
            byte[] body = switch (item.itemType()) {
                case Structure -> {
                    var children = new ByteArrayOutputStream();
                    try (var structure = value.structure()) {
                        for (long index = 0; index < structure.itemCount(); index++) {
                            try (var child = structure.itemAt(index)) {
                                children.writeBytes(encode(child));
                            }
                        }
                    }
                    yield children.toByteArray();
                }
                case Integer -> ByteBuffer.allocate(4).putInt(value.integer()).array();
                case Enumeration -> ByteBuffer.allocate(4).putInt((int) value.enumeration()).array();
                case ByteString -> value.bytes();
                default -> throw new IllegalArgumentException("Unsupported generic probe item type");
            };
            return ByteBuffer.allocate(8 + ((body.length + 7) & ~7))
                    .putInt((tag.raw() << 8) | item.itemType().code()).putInt(body.length).put(body).array();
        }
    }

    private static byte[] exchange(ConnectionConfig connection, byte[] request)
            throws GeneralSecurityException, IOException {
        var tls = connection.tls();
        var endpoint = new InetSocketAddress(connection.host(), connection.port());
        String peerName = tls.serverNameOverride().isBlank() ? connection.host() : tls.serverNameOverride();
        try (var tcp = new Socket()) {
            tcp.connect(endpoint, Math.multiplyExact(tls.connectTimeoutSeconds(), 1000));
            try (var socket = (SSLSocket) context(tls).getSocketFactory().createSocket(
                    tcp, peerName, connection.port(), true)) {
                socket.setEnabledProtocols(new String[] {"TLSv1.3"});
                var parameters = socket.getSSLParameters();
                parameters.setEndpointIdentificationAlgorithm("HTTPS");
                socket.setSSLParameters(parameters);
                socket.setSoTimeout(Math.multiplyExact(tls.readTimeoutSeconds(), 1000));
                socket.startHandshake();
                socket.getOutputStream().write(request);
                var input = socket.getInputStream();
                byte[] header = input.readNBytes(8);
                if (header.length != 8) throw new EOFException();
                long size = Integer.toUnsignedLong(ByteBuffer.wrap(header).getInt(4));
                long padded = (size + 7) & ~7L;
                if (8 + padded > LIMITS.maxMessageBytes()) throw new IOException("TTLV limit exceeded");
                byte[] body = input.readNBytes((int) padded);
                if (body.length != padded) throw new EOFException();
                return ByteBuffer.allocate(8 + body.length).put(header).put(body).array();
            }
        }
    }

    private static SSLContext context(ConnectionConfig.Tls tls) throws GeneralSecurityException, IOException {
        var certs = CertificateFactory.getInstance("X.509");
        var trust = KeyStore.getInstance("PKCS12");
        trust.load(null);
        try (var input = Files.newInputStream(Path.of(tls.caCertificatePath()))) {
            int index = 0;
            for (var certificate : certs.generateCertificates(input)) {
                trust.setCertificateEntry("ca-" + index++, certificate);
            }
        }
        var identity = KeyStore.getInstance("PKCS12");
        identity.load(null);
        char[] password = {};
        try (var input = Files.newInputStream(Path.of(tls.clientCertificatePath()))) {
            var chain = certs.generateCertificates(input).toArray(Certificate[]::new);
            identity.setKeyEntry("client", readKey(Path.of(tls.clientKeyPath())), password, chain);
        }
        var keys = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        keys.init(identity, password);
        var managers = TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm());
        managers.init(trust);
        var context = SSLContext.getInstance("TLSv1.3");
        context.init(keys.getKeyManagers(), managers.getTrustManagers(), null);
        return context;
    }

    private static PrivateKey readKey(Path path) throws IOException, GeneralSecurityException {
        String pem = Files.readString(path).strip();
        String start = "-----BEGIN PRIVATE KEY-----";
        String end = "-----END PRIVATE KEY-----";
        if (!pem.startsWith(start) || !pem.endsWith(end)) {
            throw new InvalidKeySpecException("A PKCS#8 PEM client key is required");
        }
        byte[] bytes = Base64.getMimeDecoder().decode(pem.substring(start.length(), pem.length() - end.length()));
        try {
            for (String algorithm : new String[] {"RSA", "EC", "Ed25519", "Ed448"}) {
                try {
                    return KeyFactory.getInstance(algorithm).generatePrivate(new PKCS8EncodedKeySpec(bytes));
                } catch (InvalidKeySpecException ignored) {
                    // Try the next available key type.
                }
            }
            throw new InvalidKeySpecException("Client key type is unsupported");
        } finally {
            Arrays.fill(bytes, (byte) 0);
        }
    }

    private static TtlvStructure copyStructure(TtlvStructureView view) {
        TtlvStructure copy = TtlvStructure.create();
        try {
            for (long index = 0; index < view.itemCount(); index++) {
                try (var child = view.itemAt(index); var owned = child.copy()) {
                    copy = TtlvStructure.withItem(copy, owned, LIMITS);
                }
            }
            TtlvStructure result = copy;
            copy = null;
            return result;
        } finally {
            if (copy != null) copy.close();
        }
    }

    private static JavaKmipDriver.CaseOutcome check(byte[] wire, int operation, int minor, long start) {
        try (var root = TtlvCodec.decode(wire); var view = root.view();
                var tag = view.tag(); var value = view.value()) {
            if (tag.raw() != 0x42007b) throw new IllegalArgumentException("Wrong response root");
            try (var structure = value.structure(); var response = ResponseMessage.fromTtlv(copyStructure(structure));
                    var header = response.header()) {
                var version = header.protocolVersion();
                if (version.major() != 2 || version.minor() != minor || response.batchItemCount() != 1) {
                    throw new IllegalArgumentException("Response does not match request version and count");
                }
                try (var batch = response.batchItemAt(0)) {
                    if (batch.operation().orElse(-1) != operation) throw new IllegalArgumentException("Wrong response operation");
                    long status = batch.resultStatus().orElseThrow();
                    Long reason = batch.resultReason().isPresent() ? batch.resultReason().getAsLong() : null;
                    double duration = elapsed(start);
                    if (status == 0 && reason == null && batch.withResponsePayload(
                            payload -> matches(payload, operation, minor)).orElse(false)) {
                        return JavaKmipDriver.CaseOutcome.passed(duration, status, null);
                    }
                    if (status == 1 && reason != null) {
                        if (reason == KmipValues.ResultReason.OPERATION_NOT_SUPPORTED
                                || reason == KmipValues.ResultReason.FEATURE_NOT_SUPPORTED) {
                            return JavaKmipDriver.CaseOutcome.unsupported(duration, status, reason);
                        }
                        if (reason == KmipValues.ResultReason.PERMISSION_DENIED
                                || reason == KmipValues.ResultReason.AUTHENTICATION_NOT_SUCCESSFUL) {
                            return JavaKmipDriver.CaseOutcome.policyRestricted(duration, status, reason);
                        }
                    }
                    return JavaKmipDriver.CaseOutcome.failed("Generic response did not satisfy the case.", duration, status, reason);
                }
            }
        }
    }

    private static boolean matches(TtlvStructureView payload, int operation, int minor) {
        if (operation == LOCATE) return matchesLocate(payload);
        if (operation == PING) return payload.itemCount() == 0;
        if (operation == HASH || operation == RNG_RETRIEVE) {
            if (payload.itemCount() != 1) return false;
            try (var field = payload.itemAt(0); var tag = field.tag(); var value = field.value()) {
                return tag.raw() == 0x4200c2 && field.itemType() == TtlvItemType.ByteString
                        && (operation == RNG_RETRIEVE ? value.byteLength() == 32
                                : Arrays.equals(value.bytes(), HASH_DIGEST));
            }
        }
        if (operation == DISCOVER_VERSIONS) {
            if (payload.itemCount() != 1) return false;
            try (var actual = payload.itemAt(0); var expected = version(minor); var expectedView = expected.view()) {
                return Arrays.equals(encode(actual), encode(expectedView));
            }
        }
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() == 0x42005c && field.itemType() == TtlvItemType.Enumeration
                        && value.enumeration() != 0) return true;
            }
        }
        return false;
    }

    private static boolean matchesLocate(TtlvStructureView payload) {
        Integer total = null;
        int identifiers = 0;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                var type = field.itemType();
                if (tag.raw() == 0x4200d5 && type == TtlvItemType.Integer && total == null) {
                    total = value.integer();
                } else if (tag.raw() == 0x420094 && (type == TtlvItemType.TextString
                        || type == TtlvItemType.Integer || type == TtlvItemType.Enumeration)) {
                    identifiers++;
                } else {
                    return false;
                }
            }
        }
        return identifiers <= 1 && (total == null || total >= identifiers);
    }

    private static double elapsed(long start) {
        return (System.nanoTime() - start) / 1_000_000.0;
    }
}
