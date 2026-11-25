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
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Predicate;

final class GenericExchange {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final int CREATE = 0x01;
    private static final int CHECK = 0x09;
    private static final int GET = 0x0a;
    private static final int GET_ATTRIBUTES = 0x0b;
    private static final int GET_ATTRIBUTE_LIST = 0x0c;
    private static final int ACTIVATE = 0x12;
    private static final int REVOKE = 0x13;
    private static final int DESTROY = 0x14;
    private static final int ARCHIVE = 0x15;
    private static final int RECOVER = 0x16;
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
        int operation = operation(testCase, connection);
        return "server_exchange".equals(testCase.execution().mode()) && operation != 0
                && (usesOwnedKey(operation) ? "mutating".equals(testCase.execution().mutation())
                        : ("read_only".equals(testCase.execution().mutation()) || operation == RNG_RETRIEVE
                            || JavaKmipDriver.multipartHash(testCase)))
                && "raw_tls".equals(connection.transport()) && "TTLV".equals(connection.encoding())
                && connection.usernamePassword() == null;
    }

    private static int operation(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        String version = connection.protocolVersion();
        if (!"2.0".equals(version) && !"2.1".equals(version)) return 0;
        if (keyAttributeReference(testCase.caseId()) != 0) return GET_ATTRIBUTES;
        return switch (testCase.caseId()) {
            case "test-op-create" -> CREATE;
            case "test-op-check" -> CHECK;
            case "test-op-get" -> GET;
            case "test-op-get-attribute-list" -> GET_ATTRIBUTE_LIST;
            case "test-op-activate" -> ACTIVATE;
            case "test-op-revoke" -> REVOKE;
            case "test-op-destroy" -> DESTROY;
            case "test-op-archive" -> ARCHIVE;
            case "test-op-recover" -> RECOVER;
            case "test-op-locate" -> LOCATE;
            case "test-op-query" -> QUERY;
            case "test-op-discover-versions" -> DISCOVER_VERSIONS;
            case "test-crypto-rng-retrieve-rng-32" -> RNG_RETRIEVE;
            case "test-op-hash", "test-crypto-hash-sha_256-0", "test-multipart-hash-sha256" -> HASH;
            case "test-op-ping" -> "2.1".equals(version) ? PING : 0;
            default -> 0;
        };
    }

    static JavaKmipDriver.CaseOutcome run(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        long start = System.nanoTime();
        int minor = "2.1".equals(connection.protocolVersion()) ? 1 : 0;
        if (JavaKmipDriver.multipartHash(testCase)) return multipartHash(connection, minor, start);
        int operation = operation(testCase, connection);
        int attributeTag = keyAttributeReference(testCase.caseId());
        if (usesOwnedKey(operation)) return runOwnedKey(connection, operation, minor, attributeTag, start);
        return execute(connection, operation, minor, null, payload -> matches(payload, operation, minor), start);
    }

    private static int keyAttributeReference(String caseId) {
        return switch (caseId) {
            case "test-attr-cryptographic-algorithm" -> 0x420028;
            case "test-op-get-attributes", "test-attr-cryptographic-length" -> 0x42002a;
            case "test-attr-cryptographic-usage-mask" -> 0x42002c;
            case "test-attr-object-type" -> 0x420057;
            case "test-attr-state" -> 0x42008d;
            case "test-attr-sensitive" -> 0x420120;
            case "test-attr-always-sensitive" -> 0x420121;
            case "test-attr-extractable" -> 0x420122;
            case "test-attr-never-extractable" -> 0x420123;
            default -> 0;
        };
    }

    private static boolean usesOwnedKey(int operation) {
        return switch (operation) {
            case CREATE, CHECK, GET, GET_ATTRIBUTES, GET_ATTRIBUTE_LIST, ACTIVATE, REVOKE, DESTROY, ARCHIVE, RECOVER -> true;
            default -> false;
        };
    }

    private static JavaKmipDriver.CaseOutcome runOwnedKey(ConnectionConfig connection, int operation,
            int minor, int attributeTag, long start) {
        var createdId = new AtomicReference<byte[]>();
        var create = execute(connection, CREATE, minor, null, attributeTag, payload -> {
            byte[] id = textIdentifier(payload);
            createdId.set(id);
            return id != null && hasField(payload, 0x420057, TtlvItemType.Enumeration, 2);
        }, start);
        byte[] id = createdId.get();
        if (id == null) {
            if (operation == CREATE || "failed".equals(create.status())) return create;
            return JavaKmipDriver.CaseOutcome.notRun("Server rejected isolated Create setup.", elapsed(start));
        }
        Predicate<TtlvStructureView> sameId = payload -> Arrays.equals(id, textIdentifier(payload));
        if ("passed".equals(create.status()) && operation == DESTROY) {
            return execute(connection, DESTROY, minor, id, sameId, start);
        }

        var target = create;
        if ("passed".equals(create.status()) && operation == GET) {
            target = execute(connection, GET, minor, id, payload -> sameId.test(payload)
                    && hasField(payload, 0x420057, TtlvItemType.Enumeration, 2)
                    && hasField(payload, 0x42008f, TtlvItemType.Structure, 0), start);
        }
        if ("passed".equals(create.status()) && operation == GET_ATTRIBUTES) {
            target = execute(connection, GET_ATTRIBUTES, minor, id, attributeTag,
                    payload -> sameId.test(payload) && hasKeyAttribute(payload, attributeTag), start);
        }
        if ("passed".equals(create.status()) && operation == GET_ATTRIBUTE_LIST) {
            target = execute(connection, GET_ATTRIBUTE_LIST, minor, id,
                    payload -> sameId.test(payload) && hasLengthReference(payload), start);
        }
        if ("passed".equals(create.status()) && (operation == ACTIVATE || operation == REVOKE || operation == CHECK)) {
            var activate = execute(connection, ACTIVATE, minor, id, sameId, start);
            target = activate;
            if ("passed".equals(activate.status()) && operation == CHECK) {
                target = execute(connection, CHECK, minor, id,
                        payload -> sameId.test(payload) && matchesCheckUsage(payload), start);
            }
            if (activate.resultStatusCode() == null || activate.resultStatusCode() == 0) {
                var revoke = execute(connection, REVOKE, minor, id, sameId, start);
                if ("passed".equals(activate.status())) {
                    if (operation == REVOKE) target = revoke;
                    else if (!"passed".equals(revoke.status())) {
                        target = JavaKmipDriver.CaseOutcome.failed("Cleanup Revoke failed for the activated key.", elapsed(start));
                    }
                }
            } else if (operation != ACTIVATE) {
                target = JavaKmipDriver.CaseOutcome.notRun("Server rejected isolated Activate setup.", elapsed(start));
            }
        }
        if ("passed".equals(create.status()) && (operation == ARCHIVE || operation == RECOVER)) {
            var archive = execute(connection, ARCHIVE, minor, id, sameId, start);
            target = archive;
            if (Long.valueOf(0).equals(archive.resultStatusCode())) {
                var recover = execute(connection, RECOVER, minor, id, sameId, start);
                if ("passed".equals(archive.status())) {
                    target = operation == RECOVER ? recover : archive;
                    if (operation == ARCHIVE && !"passed".equals(recover.status())) {
                        target = JavaKmipDriver.CaseOutcome.failed("Cleanup Recover failed.", elapsed(start));
                    }
                }
            } else if (operation == RECOVER && !"failed".equals(archive.status())) {
                target = JavaKmipDriver.CaseOutcome.notRun("Server rejected Archive setup.", elapsed(start));
            }
        }

        var cleanup = execute(connection, DESTROY, minor, id, sameId, start);
        return "passed".equals(cleanup.status()) ? target
                : JavaKmipDriver.CaseOutcome.failed("Cleanup Destroy failed for the isolated key.", elapsed(start));
    }

    private record HashFrame(int index, byte[] chunk, byte[] correlation) { }

    private static JavaKmipDriver.CaseOutcome multipartHash(ConnectionConfig connection, int minor, long start) {
        byte[] expected = HexFormat.of().parseHex("484e69bd8ae45ed2440bf93f35e464f86ed9557258f0377c0c070a4f786b5b0a");
        var correlation = new AtomicReference<byte[]>();
        var chunks = List.of("KMIP ", "Hash", "!");
        try {
            JavaKmipDriver.CaseOutcome outcome = null;
            for (int index = 0; index < chunks.size(); index++) {
                var frame = new HashFrame(index, chunks.get(index).getBytes(StandardCharsets.US_ASCII), correlation.get());
                outcome = execute(connection, HASH, minor, null, 0, frame, payload -> {
                    if (frame.index() == 1) return payload.itemCount() == 0;
                    if (payload.itemCount() != 1) return false;
                    try (var field = payload.itemAt(0); var tag = field.tag(); var value = field.value()) {
                        if (field.itemType() != TtlvItemType.ByteString) return false;
                        if (frame.index() == 0 && tag.raw() == 0x4200d6) {
                            byte[] bytes = value.bytes();
                            correlation.set(bytes);
                            return bytes.length != 0;
                        }
                        return frame.index() == 2 && tag.raw() == 0x4200c2
                                && Arrays.equals(value.bytes(), expected);
                    }
                }, start);
                if (!"passed".equals(outcome.status())) return outcome;
            }
            return outcome;
        } finally {
            byte[] bytes = correlation.get();
            if (bytes != null) Arrays.fill(bytes, (byte) 0);
        }
    }

    private static JavaKmipDriver.CaseOutcome execute(ConnectionConfig connection, int operation, int minor,
            byte[] identifier, Predicate<TtlvStructureView> expected, long start) {
        return execute(connection, operation, minor, identifier, 0, expected, start);
    }

    private static JavaKmipDriver.CaseOutcome execute(ConnectionConfig connection, int operation, int minor,
            byte[] identifier, int attributeTag, Predicate<TtlvStructureView> expected, long start) {
        return execute(connection, operation, minor, identifier, attributeTag, null, expected, start);
    }

    private static JavaKmipDriver.CaseOutcome execute(ConnectionConfig connection, int operation, int minor,
            byte[] identifier, int attributeTag, HashFrame frame, Predicate<TtlvStructureView> expected, long start) {
        try (var request = request(operation, minor, identifier, attributeTag, frame); var view = request.view()) {
            byte[] wire = exchange(connection, encode(view));
            try {
                return check(wire, operation, minor, start, expected);
            } finally {
                Arrays.fill(wire, (byte) 0);
            }
        } catch (GeneralSecurityException | IOException | RuntimeException error) {
            return JavaKmipDriver.CaseOutcome.failed("Generic TTLV exchange or validation failed ("
                    + error.getClass().getSimpleName() + ").", elapsed(start));
        }
    }

    private static TtlvItem request(int operation, int minor, byte[] objectId, int attributeTag, HashFrame frame) {
        try (var version = version(minor);
                var count = item(0x42000d, TtlvValue.Integer(1));
                var header = structureItem(0x420077, version, count);
                var identifier = item(0x42005c, TtlvValue.Enumeration(operation));
                var payload = switch (operation) {
                    case CREATE -> createPayload(attributeTag);
                    case CHECK -> structureItem(0x420079,
                            item(0x420094, TtlvValue.TextString(objectId, LIMITS)),
                            item(0x42002c, TtlvValue.Integer(4)));
                    case GET_ATTRIBUTES -> structureItem(0x420079,
                            item(0x420094, TtlvValue.TextString(objectId, LIMITS)),
                            item(0x42013b, TtlvValue.Enumeration(attributeTag)));
                    case REVOKE -> revokePayload(objectId);
                    case GET, GET_ATTRIBUTE_LIST, ACTIVATE, DESTROY, ARCHIVE, RECOVER -> structureItem(0x420079,
                            item(0x420094, TtlvValue.TextString(objectId, LIMITS)));
                    case LOCATE -> structureItem(0x420079, item(0x42004f, TtlvValue.Integer(1)),
                            item(0x4200d4, TtlvValue.Integer(0)));
                    case QUERY -> structureItem(0x420079, item(0x420074, TtlvValue.Enumeration(1)));
                    case DISCOVER_VERSIONS -> structureItem(0x420079, version(minor));
                    case RNG_RETRIEVE -> structureItem(0x420079, item(0x4200c4, TtlvValue.Integer(32)));
                    case HASH -> hashPayload(frame);
                    case PING -> structureItem(0x420079);
                    default -> throw new IllegalArgumentException("Unsupported generic operation");
                };
                var batch = structureItem(0x42000f, identifier, payload)) {
            return structureItem(0x420078, header, batch);
        }
    }

    private static TtlvItem revokePayload(byte[] objectId) {
        try (var code = item(0x420082, TtlvValue.Enumeration(6));
                var reason = structureItem(0x420081, code);
                var identifier = item(0x420094, TtlvValue.TextString(objectId, LIMITS))) {
            return structureItem(0x420079, identifier, reason);
        }
    }

    private static TtlvItem createPayload(int attributeTag) {
        try (var algorithm = item(0x420028, TtlvValue.Enumeration(3));
                var length = item(0x42002a, TtlvValue.Integer(256));
                var usage = item(0x42002c, TtlvValue.Integer(4));
                var sensitive = item(0x420120, TtlvValue.Boolean(true));
                var extractable = item(0x420122, TtlvValue.Boolean(false));
                var attributes = attributeTag >= 0x420120 && attributeTag <= 0x420123
                        ? structureItem(0x420125, algorithm, length, usage, sensitive, extractable)
                        : structureItem(0x420125, algorithm, length, usage);
                var objectType = item(0x420057, TtlvValue.Enumeration(2))) {
            return structureItem(0x420079, objectType, attributes);
        }
    }

    private static TtlvItem hashPayload(HashFrame frame) {
        try (var parameters = structureItem(0x42002b, item(0x420038, TtlvValue.Enumeration(6)));
                var data = item(0x4200c2, TtlvValue.ByteString(frame == null ? HASH_INPUT : frame.chunk(), LIMITS))) {
            if (frame == null) return structureItem(0x420079, parameters, data);
            if (frame.index() == 0) {
                try (var init = item(0x4200d7, TtlvValue.Boolean(true))) {
                    return structureItem(0x420079, parameters, data, init);
                }
            }
            try (var correlation = item(0x4200d6, TtlvValue.ByteString(frame.correlation(), LIMITS))) {
                if (frame.index() == 1) return structureItem(0x420079, parameters, data, correlation);
                try (var end = item(0x4200d8, TtlvValue.Boolean(true))) {
                    return structureItem(0x420079, parameters, data, correlation, end);
                }
            }
        }
    }

    private static TtlvItem version(int minor) {
        try (var major = item(0x42006a, TtlvValue.Integer(2));
                var minorItem = item(0x42006b, TtlvValue.Integer(minor))) {
            return structureItem(0x420069, major, minorItem);
        }
    }

    static TtlvItem item(int code, TtlvValue value) {
        try (value; var raw = RawTag.fromRaw(code); var tag = raw.checked()) {
            return TtlvItem.create(tag, value, LIMITS);
        }
    }

    // withItem consumes its arguments. Closing already consumed handles is harmless.
    static TtlvItem structureItem(int code, TtlvItem... items) {
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

    static byte[] encode(TtlvItemView item) {
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
                case LongInteger -> ByteBuffer.allocate(8).putLong(value.longInteger()).array();
                case BigInteger -> {
                    byte[] integer = value.bytes();
                    if (integer.length == 0 || integer.length % 8 != 0) {
                        throw new IllegalArgumentException("Invalid Big Integer size");
                    }
                    yield integer;
                }
                case Boolean -> ByteBuffer.allocate(8).putLong(value.booleanValue() ? 1 : 0).array();
                case ByteString, TextString -> value.bytes();
                case DateTime -> ByteBuffer.allocate(8).putLong(value.dateTime()).array();
                case Interval -> ByteBuffer.allocate(4).putInt((int) value.interval()).array();
                case DateTimeExtended -> ByteBuffer.allocate(8).putLong(value.dateTimeExtended()).array();
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
                return readFrame(socket.getInputStream());
            }
        }
    }

    static byte[] readFrame(java.io.InputStream input) throws IOException {
        byte[] header = input.readNBytes(8);
        if (header.length != 8) throw new EOFException();
        long size = Integer.toUnsignedLong(ByteBuffer.wrap(header).getInt(4));
        long padded = (size + 7) & ~7L;
        if (8 + padded > LIMITS.maxMessageBytes()) throw new IOException("TTLV limit exceeded");
        byte[] body = input.readNBytes((int) padded);
        try {
            if (body.length != padded) throw new EOFException();
            return ByteBuffer.allocate(8 + body.length).put(header).put(body).array();
        } finally {
            Arrays.fill(body, (byte) 0);
        }
    }

    private static SSLContext context(ConnectionConfig.Tls tls) throws GeneralSecurityException, IOException {
        return context(Path.of(tls.caCertificatePath()), Path.of(tls.clientCertificatePath()), Path.of(tls.clientKeyPath()));
    }

    static SSLContext context(Path ca, Path certificatePath, Path key) throws GeneralSecurityException, IOException {
        var certs = CertificateFactory.getInstance("X.509");
        var trust = KeyStore.getInstance("PKCS12");
        trust.load(null);
        try (var input = Files.newInputStream(ca)) {
            int index = 0;
            for (var certificate : certs.generateCertificates(input)) {
                trust.setCertificateEntry("ca-" + index++, certificate);
            }
        }
        var identity = KeyStore.getInstance("PKCS12");
        identity.load(null);
        char[] password = {};
        try (var input = Files.newInputStream(certificatePath)) {
            var chain = certs.generateCertificates(input).toArray(Certificate[]::new);
            identity.setKeyEntry("client", readKey(key), password, chain);
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

    static TtlvStructure copyStructure(TtlvStructureView view) {
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

    private static JavaKmipDriver.CaseOutcome check(byte[] wire, int operation, int minor, long start,
            Predicate<TtlvStructureView> expected) {
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
                    if (status == 0 && reason == null && batch.withResponsePayload(expected::test).orElse(false)) {
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

    private static byte[] textIdentifier(TtlvStructureView payload) {
        byte[] id = null;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag()) {
                if (tag.raw() != 0x420094) continue;
                if (id != null || field.itemType() != TtlvItemType.TextString) return null;
                try (var value = field.value()) {
                    if (value.byteLength() == 0) return null;
                    id = value.bytes();
                }
            }
        }
        return id;
    }

    private static boolean hasField(TtlvStructureView payload, int wanted, TtlvItemType type, int expected) {
        boolean found = false;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() != wanted) continue;
                if (found || field.itemType() != type) return false;
                found = switch (type) {
                    case Enumeration -> value.enumeration() == expected;
                    case Integer -> value.integer() == expected;
                    case Boolean -> value.booleanValue() == (expected != 0);
                    default -> true;
                };
                if (!found) return false;
            }
        }
        return found;
    }

    private static boolean matchesCheckUsage(TtlvStructureView payload) {
        boolean found = false;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() != 0x42002c) continue;
                if (found || field.itemType() != TtlvItemType.Integer || (value.integer() & 4) == 0) return false;
                found = true;
            }
        }
        return true;
    }

    private static boolean hasLengthReference(TtlvStructureView payload) {
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                if (tag.raw() == 0x42013b && field.itemType() == TtlvItemType.Enumeration
                        && value.enumeration() == 0x42002a) return true;
            }
        }
        return false;
    }

    private static boolean hasKeyAttribute(TtlvStructureView payload, int attributeTag) {
        if (!hasField(payload, 0x420125, TtlvItemType.Structure, 0)) return false;
        for (long index = 0; index < payload.itemCount(); index++) {
            try (var field = payload.itemAt(index); var tag = field.tag()) {
                if (tag.raw() != 0x420125) continue;
                try (var value = field.value(); var attrs = value.structure()) {
                    if (attrs.itemCount() != 1) return false;
                    return switch (attributeTag) {
                        case 0x420028 -> hasField(attrs, attributeTag, TtlvItemType.Enumeration, 3);
                        case 0x42002a -> hasField(attrs, attributeTag, TtlvItemType.Integer, 256);
                        case 0x42002c -> hasField(attrs, attributeTag, TtlvItemType.Integer, 4);
                        case 0x420057 -> hasField(attrs, attributeTag, TtlvItemType.Enumeration, 2);
                        case 0x42008d -> hasField(attrs, attributeTag, TtlvItemType.Enumeration, 1);
                        case 0x420120, 0x420121, 0x420123 -> hasField(attrs, attributeTag, TtlvItemType.Boolean, 1);
                        case 0x420122 -> hasField(attrs, attributeTag, TtlvItemType.Boolean, 0);
                        default -> false;
                    };
                }
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
