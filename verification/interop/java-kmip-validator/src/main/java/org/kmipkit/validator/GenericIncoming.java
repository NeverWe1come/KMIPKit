package org.kmipkit.validator;

import org.kmipkit.KmipException;
import org.kmipkit.extensions.IncomingServerBatch;
import org.kmipkit.message.RequestMessage;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLSocket;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.SocketTimeoutException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;

import static org.kmipkit.validator.GenericExchange.item;
import static org.kmipkit.validator.GenericExchange.structureItem;

/** Receives KMIP 2.x raw TLS and constructs replies from native generic TTLV items. */
final class GenericIncoming implements AutoCloseable {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final int[] OPERATIONS = {0x1e, 0x1b, 0x1c, 0x18, 0x32};
    private static final Map<String, Integer> CODES = Map.of(
            "Discover Versions", 0x1e, "Notify", 0x1b, "Put", 0x1c, "Query", 0x18, "Set Endpoint Role", 0x32);
    private final SSLServerSocket listener;
    private final int timeout;
    private final int minor;

    private GenericIncoming(SSLServerSocket listener, int timeout, int minor) {
        this.listener = listener;
        this.timeout = timeout;
        this.minor = minor;
    }

    static boolean isIncoming(CatalogLoader.CatalogCase testCase) {
        return "server_to_client".equals(testCase.requires().path("workflow_type").asText());
    }

    static boolean supports(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        return isIncoming(testCase) && CODES.containsKey(testCase.operation()) && supportsConnection(connection);
    }

    private static boolean supportsConnection(ConnectionConfig connection) {
        return "generic-ttlv".equals(connection.apiLevel()) && "raw_tls".equals(connection.transport())
                && "TTLV".equals(connection.encoding())
                && ("2.0".equals(connection.protocolVersion()) || "2.1".equals(connection.protocolVersion()));
    }

    static GenericIncoming open(ConnectionConfig connection, Map<String, String> options)
            throws IOException, GeneralSecurityException {
        String address = options.get("--listen");
        if (address == null) {
            if (options.containsKey("--listener-cert") || options.containsKey("--listener-key")
                    || options.containsKey("--listener-ready-file")) {
                throw new IllegalArgumentException("Listener options require --listen");
            }
            return null;
        }
        if (!supportsConnection(connection)) throw new IllegalArgumentException("Generic listener requires KMIP 2.x raw TLS");
        int separator = address.lastIndexOf(':');
        if (separator < 1) throw new IllegalArgumentException("Expected listener address and port");
        String host = address.substring(0, separator);
        if (host.startsWith("[") && host.endsWith("]")) host = host.substring(1, host.length() - 1);
        if (!host.matches("[0-9.]+|[0-9a-fA-F:.]*:[0-9a-fA-F:.]*")) {
            throw new IllegalArgumentException("Expected numeric listener address");
        }
        int port = Integer.parseInt(address.substring(separator + 1));
        var tls = connection.tls();
        var cert = Path.of(options.getOrDefault("--listener-cert", tls.clientCertificatePath()));
        var key = Path.of(options.getOrDefault("--listener-key", tls.clientKeyPath()));
        var context = GenericExchange.context(Path.of(tls.caCertificatePath()), cert, key);
        int timeout = Math.multiplyExact(tls.readTimeoutSeconds(), 1000);
        var socket = (SSLServerSocket) context.getServerSocketFactory().createServerSocket();
        try {
            socket.setEnabledProtocols(new String[]{"TLSv1.3"});
            socket.setNeedClientAuth(true);
            socket.bind(new InetSocketAddress(host, port));
            socket.setSoTimeout(timeout);
            if (options.containsKey("--listener-ready-file")) {
                String displayHost = host.contains(":") ? "[" + host + "]" : host;
                Files.writeString(Path.of(options.get("--listener-ready-file")), displayHost + ":" + socket.getLocalPort(),
                        java.nio.file.StandardOpenOption.CREATE_NEW, java.nio.file.StandardOpenOption.WRITE);
            }
            return new GenericIncoming(socket, timeout, "2.1".equals(connection.protocolVersion()) ? 1 : 0);
        } catch (IOException | RuntimeException failure) {
            socket.close();
            throw failure;
        }
    }

    JavaKmipDriver.CaseOutcome exchange(CatalogLoader.CatalogCase testCase) {
        long start = System.nanoTime();
        SSLSocket socket;
        try {
            socket = (SSLSocket) listener.accept();
        } catch (SocketTimeoutException exception) {
            return JavaKmipDriver.CaseOutcome.notRun("Incoming peer did not connect before the listener deadline.", elapsed(start));
        } catch (IOException exception) {
            return JavaKmipDriver.CaseOutcome.failed("Incoming accept failed.", elapsed(start));
        }
        try (socket) {
            socket.setSoTimeout(timeout);
            socket.startHandshake();
            byte[] wire = GenericExchange.readFrame(socket.getInputStream());
            try {
                return reply(socket, wire, CODES.get(testCase.operation()), start);
            } finally {
                Arrays.fill(wire, (byte) 0);
            }
        } catch (IOException | RuntimeException exception) {
            return JavaKmipDriver.CaseOutcome.failed("Incoming request validation or delivery failed.", elapsed(start));
        }
    }

    private JavaKmipDriver.CaseOutcome reply(SSLSocket socket, byte[] wire, int operation, long start) throws IOException {
        try (var root = TtlvCodec.decode(wire); var view = root.view(); var tag = view.tag();
                var value = view.value(); var tree = value.structure()) {
            if (tag.raw() != 0x420078) throw new IOException("Expected Request Message");
            try (var message = RequestMessage.fromTtlv(GenericExchange.copyStructure(tree)); var header = message.header()) {
                var version = header.protocolVersion();
                if (version.major() != 2 || version.minor() != minor || message.batchItemCount() != 1) {
                    throw new IOException("Incoming version or batch count mismatch");
                }
                try (var batch = message.batchItemAt(0)) {
                    if (batch.operation().orElse(-1) != operation) throw new IOException("Wrong incoming operation");
                    return send(socket, wire, operation, header.clientCorrelationValue().orElse(null),
                            batch.uniqueBatchItemId().orElse(null), header.maximumResponseSize().orElse(Integer.MAX_VALUE), start);
                }
            }
        }
    }

    private JavaKmipDriver.CaseOutcome send(SSLSocket socket, byte[] wire, int operation, String correlation,
            byte[] batchId, int maximumResponseSize, long start) throws IOException {
        var payload = payload(wire, operation);
        int status = payload == null ? 1 : 0;
        List<TtlvItem> header = new ArrayList<>();
        header.add(version());
        header.add(item(0x420092, TtlvValue.DateTime(Instant.now().getEpochSecond())));
        if (correlation != null) header.add(item(0x420105, TtlvValue.TextString(correlation.getBytes(StandardCharsets.UTF_8), LIMITS)));
        header.add(item(0x42000d, TtlvValue.Integer(1)));
        List<TtlvItem> batch = new ArrayList<>();
        batch.add(item(0x42005c, TtlvValue.Enumeration(operation)));
        if (batchId != null) batch.add(item(0x420093, TtlvValue.ByteString(batchId, LIMITS)));
        batch.add(item(0x42007f, TtlvValue.Enumeration(status)));
        batch.add(payload == null ? item(0x42007e, TtlvValue.Enumeration(7)) : payload);
        try (var response = structureItem(0x42007b, structureItem(0x42007a, header.toArray(TtlvItem[]::new)),
                structureItem(0x42000f, batch.toArray(TtlvItem[]::new))); var view = response.view()) {
            byte[] encoded = GenericExchange.encode(view);
            try {
                if (encoded.length > maximumResponseSize) throw new IOException("Maximum response size exceeded");
                socket.getOutputStream().write(encoded);
                socket.getOutputStream().flush();
            } finally {
                Arrays.fill(encoded, (byte) 0);
            }
        }
        return status == 0 ? JavaKmipDriver.CaseOutcome.passed(elapsed(start), 0L, null)
                : JavaKmipDriver.CaseOutcome.failed("Peer sent an invalid payload; Invalid Field sent.", elapsed(start), 1L, 7L);
    }

    private TtlvItem payload(byte[] wire, int operation) {
        IncomingServerBatch decoded;
        try {
            decoded = IncomingServerBatch.decodeTtlv(wire, LIMITS);
        } catch (KmipException exception) {
            return null;
        }
        try (decoded) {
            List<TtlvItem> fields = new ArrayList<>();
            if (operation == 0x1e) {
                var offered = decoded.itemDiscoverVersions(0);
                if (offered.isEmpty() || offered.stream().anyMatch(v -> v.major() == 2 && v.minor() == minor)) fields.add(version());
            } else if (operation == 0x18) {
                long flags = decoded.itemQueryFlags(0);
                if ((flags & 1) != 0) for (int code : OPERATIONS) fields.add(item(0x42005c, TtlvValue.Enumeration(code)));
                if ((flags & 4) != 0) {
                    fields.add(item(0x42009d, TtlvValue.TextString("KMIPKit".getBytes(StandardCharsets.US_ASCII), LIMITS)));
                    fields.add(structureItem(0x420088));
                }
            } else if (operation == 0x32) {
                fields.add(item(0x420151, TtlvValue.Enumeration(decoded.itemSetEndpointRole(0))));
            }
            return structureItem(0x42007c, fields.toArray(TtlvItem[]::new));
        }
    }

    private TtlvItem version() {
        return structureItem(0x420069, item(0x42006a, TtlvValue.Integer(2)), item(0x42006b, TtlvValue.Integer(minor)));
    }

    private static double elapsed(long start) {
        return (System.nanoTime() - start) / 1_000_000.0;
    }

    @Override
    public void close() throws IOException {
        listener.close();
    }
}
