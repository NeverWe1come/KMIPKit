package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;
import org.kmipkit.ClientOperationException;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.CodecLimits;

final class ProfileValidationMappingTest {
    @Test
    void selectionAndContextRetainCallerOwnedValues() {
        var selection = new ProfileValidationSelection(List.of("baseline_client"));
        var state = new ProfileObjectState("object-7").withOriginalCreationDatePresent(false);
        var context = new ProfileValidationContext().withObjectState(0, state);

        assertEquals(List.of("baseline_client"), selection.profileIds());
        assertEquals(List.of(new ProfileValidationContext.Entry(0, state)), context.objectStates());
    }

    @Test
    void clientExposesExplicitProfileExchangeMethods() throws Exception {
        assertEquals(4, java.util.Arrays.stream(Client.class.getMethods())
                .filter(method -> method.getName().startsWith("executeWithProfiles"))
                .count());
    }

    @Test
    void jniProfileExecutionSymbolRejectsInvalidHandles() {
        assertThrows(InvalidInputException.class, () ->
                NativeExtensionRegistry.clientExecuteWithProfiles(
                        0, 0, null, null, null, null, null, null, null, null, null));
    }

    @Test
    void baselineSelectionFailsClosedBeforeJniOpensATransportConnection() throws Exception {
        byte[] clientCertificate = testFixture("client-cert.pem");
        byte[] clientPrivateKey = testFixture("client-key.pem");
        byte[] serverCertificate = testFixture("server-cert.pem");
        try (ServerSocket listener = new ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))) {
            listener.setSoTimeout(1_000);
            FutureTask<Boolean> accepted = new FutureTask<>(() -> {
                try (Socket ignored = listener.accept()) {
                    return true;
                } catch (SocketTimeoutException timeout) {
                    return false;
                }
            });
            Thread observer = new Thread(accepted, "kmipkit-profile-no-connect-observer");
            observer.setDaemon(true);
            observer.start();

            try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(), ExtensionRegistryLimits.defaults());
                    ClientConfiguration configuration = ClientConfiguration.create(registry);
                    TransportConfiguration transport = TransportConfiguration.builder()
                            .rawTls("127.0.0.1", listener.getLocalPort())
                            .clientIdentityPem(new byte[][] {clientCertificate}, clientPrivateKey)
                            .trustCertificatesPem(new byte[][] {serverCertificate})
                            .build();
                    Client client = Client.create(configuration, transport);
                    ClientBatch batch = ClientBatch.create();
                    ClientBatch unknownBatch = ClientBatch.create()) {
                batch.add(ClientBatchItem.discoverVersions());
                ClientOperationException failure = assertThrows(ClientOperationException.class,
                        () -> client.executeWithProfiles(batch, CodecLimits.defaults(),
                                new ProfileValidationSelection(
                                        List.of("KMIPKIT-PROFILE-BASELINE-CLIENT"))));
                assertEquals(1, failure.categoryCode());
                assertEquals(1, failure.deliveryStateCode());
                var details = failure.profileValidation().orElseThrow();
                assertEquals("profile_unavailable", details.errorCode());
                assertEquals("KMIPKIT-PROFILE-BASELINE-CLIENT", details.profileId().orElseThrow());
                assertEquals("KMIPKIT-REQ-SPEC-14.1-001", details.requirementId().orElseThrow());
                assertEquals("KMIPKIT-CLAUSE-SPEC-14.1-001", details.sourceClauseId().orElseThrow());
                assertFalse(details.sourceGapId().isPresent());
                assertFalse(details.batchItemIndex().isPresent());
                assertFalse(failure.getMessage().contains("KMIPKIT-PROFILE-BASELINE-CLIENT"));

                unknownBatch.add(ClientBatchItem.discoverVersions());
                String unknownSelector = "unpublished-profile-selector-sentinel";
                ClientOperationException unknownFailure = assertThrows(
                        ClientOperationException.class,
                        () -> client.executeWithProfiles(unknownBatch, CodecLimits.defaults(),
                                new ProfileValidationSelection(List.of(unknownSelector))));
                assertEquals(1, unknownFailure.categoryCode());
                assertEquals(1, unknownFailure.deliveryStateCode());
                var unknownDetails = unknownFailure.profileValidation().orElseThrow();
                assertEquals("unknown_profile", unknownDetails.errorCode());
                assertFalse(unknownDetails.profileId().isPresent());
                assertFalse(unknownDetails.requirementId().isPresent());
                assertFalse(unknownDetails.sourceClauseId().isPresent());
                assertFalse(unknownDetails.sourceGapId().isPresent());
                assertFalse(unknownDetails.sourceGapLocator().isPresent());
                assertFalse(unknownDetails.batchItemIndex().isPresent());
                assertFalse(unknownFailure.getMessage().contains(unknownSelector));
            }

            assertFalse(accepted.get(2, TimeUnit.SECONDS),
                    "incomplete Baseline readiness must make zero TCP connections");
        }
    }

    private static byte[] testFixture(String filename) throws Exception {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(Path.of(
                    "bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IllegalStateException("the shared deterministic mTLS fixture is missing");
    }
}
