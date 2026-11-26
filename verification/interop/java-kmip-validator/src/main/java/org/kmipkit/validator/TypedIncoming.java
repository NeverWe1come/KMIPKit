package org.kmipkit.validator;

import org.kmipkit.KmipException;
import org.kmipkit.extensions.ClientListener;
import org.kmipkit.extensions.ListenerTransportConfiguration;
import org.kmipkit.extensions.S2cCapabilities;
import org.kmipkit.extensions.S2cOperationResult;
import org.kmipkit.extensions.ServerRequestDispatcher;
import org.kmipkit.extensions.ServerRequestHandler;
import org.kmipkit.protocol.VersionGate.ProtocolVersion;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvCodec;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.time.Duration;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Set;

/** Exercises the installed client listener and server-request dispatcher. */
final class TypedIncoming implements AutoCloseable {
    private final ClientListener listener;
    private final Duration timeout;
    private final int minor;

    private TypedIncoming(ClientListener listener, Duration timeout, int minor) {
        this.listener = listener;
        this.timeout = timeout;
        this.minor = minor;
    }

    static boolean supports(CatalogLoader.CatalogCase testCase, ConnectionConfig connection) {
        return GenericIncoming.isIncoming(testCase)
                && Set.of("Notify", "Put").contains(testCase.operation()) && supportsConnection(connection);
    }

    static boolean supportsConnection(ConnectionConfig connection) {
        return "typed-protocol".equals(connection.apiLevel()) && "raw_tls".equals(connection.transport())
                && "TTLV".equals(connection.encoding())
                && Set.of("2.0", "2.1").contains(connection.protocolVersion());
    }

    static TypedIncoming open(ConnectionConfig connection, Map<String, String> options) throws IOException {
        String address = options.get("--listen");
        if (address == null) return null;
        int separator = address.lastIndexOf(':');
        if (separator < 1) throw new IllegalArgumentException("Expected listener address and port");
        String host = address.substring(0, separator);
        if (host.startsWith("[") && host.endsWith("]")) host = host.substring(1, host.length() - 1);
        if (!host.matches("[0-9.]+|[0-9a-fA-F:.]*:[0-9a-fA-F:.]*")) {
            throw new IllegalArgumentException("Expected numeric listener address");
        }
        int port = Integer.parseInt(address.substring(separator + 1));
        var tls = connection.tls();
        byte[] key = Files.readAllBytes(Path.of(options.getOrDefault("--listener-key", tls.clientKeyPath())));
        var timeout = Duration.ofSeconds(tls.readTimeoutSeconds());
        ClientListener listener;
        try (var builder = ListenerTransportConfiguration.builder().bind(host, port)
                .serverIdentityPem(new byte[][] {Files.readAllBytes(
                        Path.of(options.getOrDefault("--listener-cert", tls.clientCertificatePath())))}, key)
                .trustCertificatesPem(new byte[][] {Files.readAllBytes(Path.of(tls.caCertificatePath()))})
                .readTimeout(timeout).writeTimeout(timeout);
                var config = builder.build()) {
            listener = ClientListener.bindRawTls(config);
        } finally {
            Arrays.fill(key, (byte) 0);
        }
        try {
            if (options.containsKey("--listener-ready-file")) {
                String displayHost = host.contains(":") ? "[" + host + "]" : host;
                Files.writeString(Path.of(options.get("--listener-ready-file")), displayHost + ":" + listener.port(),
                        StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE);
            }
            return new TypedIncoming(listener, timeout, "2.1".equals(connection.protocolVersion()) ? 1 : 0);
        } catch (IOException | RuntimeException failure) {
            listener.close();
            throw failure;
        }
    }

    JavaKmipDriver.CaseOutcome exchange(CatalogLoader.CatalogCase testCase) {
        long start = System.nanoTime();
        int expected = "Notify".equals(testCase.operation()) ? 0x1b : 0x1c;
        try (var peer = listener.accept(timeout);
                var batch = peer.recvBatch(CodecLimits.defaults(), timeout);
                var dispatcher = ServerRequestDispatcher.create()) {
            var version = batch.protocolVersion();
            if (version.major() != 2 || version.minor() != minor || batch.itemCount() != 1
                    || batch.itemOperation(0) != expected) {
                throw new IllegalArgumentException("Unexpected incoming batch");
            }
            try (var response = dispatcher.dispatch(batch, new Handler()); var result = response.batchItemAt(0)) {
                long status = result.resultStatus().orElseThrow();
                peer.sendResponse(response, timeout);
                double elapsed = (System.nanoTime() - start) / 1_000_000.0;
                return status == 0 && result.operation().orElse(-1) == expected
                        ? JavaKmipDriver.CaseOutcome.passed(elapsed, 0L, null)
                        : JavaKmipDriver.CaseOutcome.failed("Typed incoming handler rejected the payload.", elapsed,
                                status, result.resultReason().isPresent() ? result.resultReason().getAsLong() : null);
            }
        } catch (KmipException | IllegalArgumentException failure) {
            return JavaKmipDriver.CaseOutcome.failed("Typed incoming request validation or delivery failed.",
                    (System.nanoTime() - start) / 1_000_000.0);
        }
    }

    private static final class Handler implements ServerRequestHandler {
        @Override
        public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> offered,
                List<ProtocolVersion> selected) {
            return S2cOperationResult.failure(5);
        }

        @Override
        public S2cOperationResult handleQuery(ItemView item, long flags, S2cCapabilities capabilities) {
            return S2cOperationResult.failure(5);
        }

        @Override
        public S2cOperationResult handleNotify(ItemView item, byte[] identifier, byte[] attributes) {
            return identifier != null && identifier.length != 0
                    ? S2cOperationResult.success() : S2cOperationResult.failure(7);
        }

        @Override
        public S2cOperationResult handlePut(ItemView item, long function, byte[] identifier, byte[] replaced,
                byte[] object, byte[] attributes) {
            if (identifier == null || identifier.length == 0 || object == null || object.length == 0
                    || (function != 1 && function != 2) || (function == 2 && (replaced == null || replaced.length == 0))) {
                return S2cOperationResult.failure(7);
            }
            try (var decoded = TtlvCodec.decode(object); var view = decoded.view(); var value = view.value()) {
                return value.itemType() == org.kmipkit.ttlv.TtlvItemType.Structure
                        ? S2cOperationResult.success() : S2cOperationResult.failure(7);
            }
        }
    }

    @Override
    public void close() {
        listener.close();
    }
}
