package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ClientOperationException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

import java.util.List;

final class LocateRevokeBuilderTest {
    @Test
    void locateBuildsTypedItemWithExplicitFilters() {
        try (TtlvStructure empty = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(empty);
                LocateBuilder builder = new LocateBuilder(null, attributes)
                    .maximumItems(12)
                    .offsetItems(3)
                    .storageStatusMask(2)
                    .objectGroupMember(1)
                    .asynchronousIndicator(5)) {
            builder.validate();
            try (ClientBatchItem item = builder.toBatchItem()) {
                assertEquals(8, item.operation());
            }
        }
    }

    @Test
    void revokeBuildsTypedItemAndRejectsDateForOtherReasonCodes() {
        RevokeBuilder builder = new RevokeBuilder(null, 2, "key material compromised")
                .uniqueIdentifier(UniqueIdentifier.textString("lifecycle-key"))
                .compromiseOccurrenceDate(1_700_000_000L)
                .asynchronousIndicator(7);
        builder.validate();
        try (ClientBatchItem item = builder.toBatchItem()) {
            assertEquals(19, item.operation());
        }
        RevokeBuilder invalid = new RevokeBuilder(null, 1, null)
                .compromiseOccurrenceDate(1_700_000_000L);
        assertThrows(InvalidInputException.class, invalid::build);
    }

    @Test
    void buildersValidateOptionsAndExecutionArgumentsBeforeUsingTheClient() {
        RequestOptions options = RequestOptions.defaults();
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvStructure empty = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(empty);
                LocateBuilder locate = new LocateBuilder(null, attributes).withOptions(options)) {
            assertThrows(InvalidInputException.class, () -> locate.withOptions(null));
            assertThrows(InvalidInputException.class, () -> locate.asynchronousIndicator(0x1_0000_0000L));
            assertThrows(InvalidInputException.class, () -> locate.execute(null));
            assertThrows(InvalidInputException.class, () -> locate.execute(limits));
            assertThrows(InvalidInputException.class, () -> locate.execute(limits, null));
            assertThrows(InvalidInputException.class, () -> locate.execute(limits, options));
            assertThrows(InvalidInputException.class, () -> locate.executeWithOptions(limits, options));
        }

        RevokeBuilder revoke = new RevokeBuilder(null, 2, null).withOptions(options);
        assertThrows(InvalidInputException.class, () -> revoke.withOptions(null));
        assertThrows(InvalidInputException.class, () -> revoke.asynchronousIndicator(0x1_0000_0000L));
        assertThrows(InvalidInputException.class, () -> revoke.execute(null));
        assertThrows(InvalidInputException.class, () -> revoke.execute(limits));
        assertThrows(InvalidInputException.class, () -> revoke.execute(limits, null));
        assertThrows(InvalidInputException.class, () -> revoke.execute(limits, options));
        assertThrows(InvalidInputException.class, () -> revoke.executeWithOptions(limits, options));
        assertThrows(InvalidInputException.class, () -> new RevokeBuilder(null, -1, null));
    }

    @Test
    void requestFacadeCreatesLocateAndRevokeBuildersWithoutConnecting() throws IOException {
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", 5696)
                        .clientIdentityPem(
                                new byte[][] {fixture("client-cert.pem")}, fixture("client-key.pem"))
                        .trustCertificatesPem(new byte[][] {fixture("server-cert.pem")})
                        .build();
                KmipClient client = KmipClient.create(configuration, transport);
                TtlvStructure empty = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(empty);
                LocateBuilder locate = client.request().locate(attributes)) {
            try (ClientBatchItem locateItem = locate.build();
                    ClientBatchItem revokeItem = client.request().revoke(2).build()) {
                assertEquals(8, locateItem.operation());
                assertEquals(19, revokeItem.operation());
            }
        }
    }

    @Test
    void executionOverloadsReachTheClientAndPropagateLocalTlsFailure() throws Exception {
        byte[] serverCertificate = fixture("server-cert.pem");
        byte[] clientCertificate = fixture("client-cert.pem");
        byte[] clientKey = fixture("client-key.pem");
        try (ServerSocket listener = new ServerSocket(
                0, 1, InetAddress.getByName("127.0.0.1"))) {
            listener.setSoTimeout(15_000);
            FutureTask<Void> peer = new FutureTask<>(() -> {
                try (Socket ignored = listener.accept()) {
                    // Closing the TCP peer makes TLS fail before a response can be trusted.
                }
                return null;
            });
            Thread peerThread = new Thread(peer, "kmipkit-java-locate-revoke-failure-peer");
            peerThread.setDaemon(true);
            peerThread.start();

            try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(), ExtensionRegistryLimits.defaults());
                    ClientConfiguration configuration = ClientConfiguration.create(registry);
                    TransportConfiguration transport = TransportConfiguration.builder()
                            .rawTls("127.0.0.1", listener.getLocalPort())
                            .clientIdentityPem(
                                    new byte[][] {clientCertificate}, clientKey)
                            .trustCertificatesPem(new byte[][] {serverCertificate})
                            .build();
                    KmipClient client = KmipClient.create(configuration, transport);
                    TtlvStructure empty = TtlvStructure.create();
                    AttributeSet attributes = AttributeSet.tryNew(empty)) {
                CodecLimits limits = CodecLimits.defaults();
                RequestOptions options = RequestOptions.defaults();
                try (LocateBuilder locate = client.request().locate(attributes)
                        .asynchronousIndicator(3)) {
                    assertThrows(ClientOperationException.class, () -> locate.execute(limits));
                    assertThrows(ClientOperationException.class, () -> locate.execute(limits, options));
                    assertThrows(
                            ClientOperationException.class,
                            () -> locate.executeWithOptions(limits, options));
                }
                RevokeBuilder revoke = client.request().revoke(2, "controlled TLS failure")
                        .asynchronousIndicator(4);
                assertThrows(ClientOperationException.class, () -> revoke.execute(limits));
                assertThrows(ClientOperationException.class, () -> revoke.execute(limits, options));
                assertThrows(
                        ClientOperationException.class,
                        () -> revoke.executeWithOptions(limits, options));
            }
            peer.get(15, TimeUnit.SECONDS);
        }
    }

    private static byte[] fixture(String filename) throws IOException {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "shared", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IOException("shared mTLS fixture not found");
    }
}
