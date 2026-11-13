package org.kmipkit.server;

import static org.junit.jupiter.api.Assertions.assertThrows;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class ServerListenerValidationTest {
    private static Path fixture(String name) {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(Path.of("bindings", "python", "tests", "fixtures",
                    "server-listener-mtls", name));
            if (Files.isRegularFile(candidate)) return candidate;
            directory = directory.getParent();
        }
        throw new IllegalStateException("server listener TLS fixtures are missing");
    }

    private static byte[] read(String name) throws IOException {
        return Files.readAllBytes(fixture(name));
    }

    @Test
    void builderRejectsInvalidValuesAndIncompleteTlsConfiguration() throws Exception {
        ServerListenerConfiguration.Builder builder = ServerListenerConfiguration.builder();
        assertThrows(InvalidInputException.class, () -> builder.transport(null));
        assertThrows(InvalidInputException.class, () -> builder.wireEncoding(null));
        assertThrows(InvalidInputException.class, () -> builder.certificateEncoding(null));
        assertThrows(InvalidInputException.class, () -> builder.bindHost(null));
        assertThrows(InvalidInputException.class, () -> builder.bindHost("  "));
        assertThrows(InvalidInputException.class, () -> builder.bindHost("bad\u0000host"));
        assertThrows(InvalidInputException.class, () -> builder.bindPort(-1));
        assertThrows(InvalidInputException.class, () -> builder.bindPort(65_536));
        assertThrows(InvalidInputException.class, () -> builder.listenerCertificateChain(null));
        assertThrows(InvalidInputException.class, () -> builder.listenerCertificateChain(List.of()));
        assertThrows(InvalidInputException.class,
                () -> builder.listenerCertificateChain(Collections.nCopies(4097, new byte[] {1})));
        assertThrows(InvalidInputException.class,
                () -> builder.listenerCertificateChain(Collections.singletonList(null)));
        assertThrows(InvalidInputException.class,
                () -> builder.listenerCertificateChain(List.of(new byte[0])));
        assertThrows(InvalidInputException.class, () -> builder.listenerPrivateKey(null));
        assertThrows(InvalidInputException.class, () -> builder.listenerPrivateKey(new byte[0]));
        assertThrows(InvalidInputException.class, () -> builder.trustCertificates(null));
        assertThrows(InvalidInputException.class, () -> builder.trustCertificates(List.of()));
        assertThrows(InvalidInputException.class,
                () -> builder.revocationLists(Collections.singletonList(null)));
        assertThrows(InvalidInputException.class,
                () -> builder.revocationLists(List.of(new byte[0])));
        assertThrows(InvalidInputException.class, () -> builder.targetPath(null));
        assertThrows(InvalidInputException.class, () -> builder.targetPath("/bad\u0000path"));
        assertThrows(InvalidInputException.class, () -> builder.maxRequestBytes(-1));
        assertThrows(InvalidInputException.class, builder::build);

        builder.transport(ListenerTransport.RAW_TLS).targetPath("/kmip")
                .listenerCertificateChain(List.of(read("server-cert.pem")))
                .listenerPrivateKey(read("server-key.pem"))
                .trustCertificates(List.of(read("ca.pem")));
        assertThrows(InvalidInputException.class, builder::build);
    }

    @Test
    void bindFactoriesEnforceTransportAndBindHttpsConfigurations() throws Exception {
        assertThrows(InvalidInputException.class, () -> ServerListener.bindRawTls(null));
        assertThrows(InvalidInputException.class, () -> ServerListener.bindHttps(null));

        ServerListenerConfiguration raw = configuration(ListenerTransport.RAW_TLS, "");
        ServerListenerConfiguration https = configuration(ListenerTransport.HTTPS, "/kmip");
        try (raw; https) {
            assertThrows(IllegalArgumentException.class, () -> ServerListener.bindHttps(raw));
            assertThrows(IllegalArgumentException.class, () -> ServerListener.bindRawTls(https));
            try (ServerListener listener = ServerListener.bindHttps(https)) {
                if (listener.port() <= 0) throw new AssertionError("ephemeral HTTPS listener has a port");
            }
        }
    }

    private static ServerListenerConfiguration configuration(
            ListenerTransport transport, String targetPath) throws IOException {
        return ServerListenerConfiguration.builder()
                .transport(transport)
                .targetPath(targetPath)
                .bindHost("127.0.0.1")
                .bindPort(0)
                .listenerCertificateChain(List.of(read("server-cert.pem")))
                .listenerPrivateKey(read("server-key.pem"))
                .trustCertificates(List.of(read("ca.pem")))
                .build();
    }
}
