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
import org.kmipkit.ttlv.TtlvStructure;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;
import org.kmipkit.ClientOperationException;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ProfileValidationException;
import org.kmipkit.ttlv.CodecLimits;

final class ProfileValidationMappingTest {
    @Test
    void profileTransportConstantsMatchTheClosedAbiValues() {
        assertEquals(1, ProfileTransport.RAW_TTLV_TLS.abiValue());
        assertEquals(2, ProfileTransport.HTTPS.abiValue());
    }

    @Test
    void selectionAndContextRetainCallerOwnedValues() {
        var selection = new ProfileValidationSelection(List.of("baseline_client"));
        var state = new ProfileObjectState("object-7").withOriginalCreationDatePresent(false);
        var context = new ProfileValidationContext().withObjectState(0, state);

        assertEquals(List.of("baseline_client"), selection.profileIds());
        assertEquals(List.of(new ProfileValidationContext.Entry(0, state)), context.objectStates());
    }

    @Test
    void profileValidationResultExposesTheCompleteTypedValidationSurface() throws Exception {
        var resolver = ProfileValidationSelection.class.getMethod("validate");
        assertEquals(ProfileValidationResult.class, resolver.getReturnType());
        assertEquals(List.class, ProfileValidationResult.class.getMethod("profileIds").getReturnType());
        assertEquals(List.class, ProfileValidationResult.class.getMethod("requirementIds").getReturnType());
        assertEquals(void.class, ProfileValidationResult.class
                .getMethod("validateTransport", ProfileTransport.class).getReturnType());
        assertEquals(void.class, ProfileValidationResult.class
                .getMethod("validateRequest", TtlvStructure.class).getReturnType());
        assertEquals(void.class, ProfileValidationResult.class
                .getMethod("validateRequestWithContext", TtlvStructure.class,
                        ProfileValidationContext.class).getReturnType());
        assertEquals(void.class, ProfileValidationResult.class
                .getMethod("validateResponse", TtlvStructure.class, TtlvStructure.class)
                .getReturnType());
        assertEquals(void.class, ProfileValidationResult.class
                .getMethod("validateResponseWithContext", TtlvStructure.class,
                        TtlvStructure.class, ProfileValidationContext.class).getReturnType());
        assertEquals(void.class, ProfileValidationResult.class.getMethod("close").getReturnType());
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
    void selectionValidationFailsClosedWithRedactedSourceBackedDetails() {
        String selectedBaseline = "KMIPKIT-PROFILE-BASELINE-CLIENT";
        ProfileValidationException failure = assertThrows(
                ProfileValidationException.class,
                () -> new ProfileValidationSelection(List.of(selectedBaseline)).validate());

        var details = failure.profileValidation();
        assertEquals("profile_unavailable", details.errorCode());
        assertEquals(selectedBaseline, details.profileId().orElseThrow());
        assertEquals("KMIPKIT-REQ-SPEC-14.1-001", details.requirementId().orElseThrow());
        assertEquals("KMIPKIT-CLAUSE-SPEC-14.1-001", details.sourceClauseId().orElseThrow());
        assertFalse(details.sourceGapId().isPresent());
        assertFalse(details.sourceGapLocator().isPresent());
        assertFalse(details.batchItemIndex().isPresent());
        assertFalse(failure.getMessage().contains(selectedBaseline));
        assertFalse(failure.toString().contains(selectedBaseline));
    }

    @Test
    void resultSnapshotsOrderedIdsAndRedactsItsStringRepresentation() {
        var bridge = new StubProfileValidationBridge();
        ProfileValidationResult result = new ProfileValidationResult(41, bridge);

        assertEquals(List.of("profile-secret-sentinel", "profile-second"), result.profileIds());
        assertEquals(List.of("requirement-secret-sentinel", "requirement-second"),
                result.requirementIds());
        assertThrows(UnsupportedOperationException.class,
                () -> result.profileIds().add("profile-third"));
        assertThrows(UnsupportedOperationException.class,
                () -> result.requirementIds().add("requirement-third"));
        assertFalse(result.toString().contains("secret-sentinel"));

        result.close();
        assertEquals(1, bridge.releaseCount);
    }

    @Test
    void resultForwardsEveryValidationAndRejectsUnknownTransportBeforeNativeCall() throws Exception {
        var bridge = new StubProfileValidationBridge();
        try (ProfileValidationResult result = new ProfileValidationResult(42, bridge);
                TtlvStructure request = TtlvStructure.create();
                TtlvStructure response = TtlvStructure.create()) {
            var context = new ProfileValidationContext().withObjectState(
                    3, new ProfileObjectState("context-id"));
            var invalidTransportConstructor = ProfileTransport.class.getDeclaredConstructor(int.class);
            invalidTransportConstructor.setAccessible(true);
            ProfileTransport unknownTransport = invalidTransportConstructor.newInstance(99);
            assertThrows(InvalidInputException.class,
                    () -> result.validateTransport(unknownTransport));
            assertEquals(0, bridge.transportCalls);

            result.validateTransport(ProfileTransport.RAW_TTLV_TLS);
            result.validateRequest(request);
            result.validateRequestWithContext(request, context);
            result.validateResponse(request, response);
            result.validateResponseWithContext(request, response, context);

            assertEquals(List.of("transport:1", "request", "request-context", "response",
                    "response-context"), bridge.validationCalls);
        }
        assertEquals(1, bridge.releaseCount);
    }

    @Test
    void resultUseAfterCloseFailsWithoutCallingNativeBridgeAgain() {
        var bridge = new StubProfileValidationBridge();
        var result = new ProfileValidationResult(43, bridge);
        result.close();

        assertThrows(InvalidInputException.class,
                () -> result.validateTransport(ProfileTransport.HTTPS));
        result.close();
        assertEquals(1, bridge.releaseCount);
        assertEquals(0, bridge.transportCalls);
    }

    @Test
    void resultClosesHandleWhenIdentifierSnapshotFails() {
        var bridge = new StubProfileValidationBridge() {
            @Override
            public long requirementIdCount(long handle) {
                return 100_001;
            }
        };

        assertThrows(org.kmipkit.ResourceLimitException.class,
                () -> new ProfileValidationResult(44, bridge));
        assertEquals(1, bridge.releaseCount);
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

    private static class StubProfileValidationBridge implements ProfileValidationResult.Bridge {
        int releaseCount;
        int transportCalls;
        final List<String> validationCalls = new java.util.ArrayList<>();

        @Override public long profileIdCount(long handle) { return 2; }
        @Override public String profileIdAt(long handle, long index) {
            return index == 0 ? "profile-secret-sentinel" : "profile-second";
        }
        @Override public long requirementIdCount(long handle) { return 2; }
        @Override public String requirementIdAt(long handle, long index) {
            return index == 0 ? "requirement-secret-sentinel" : "requirement-second";
        }
        @Override public void validateTransport(long handle, int transport) {
            transportCalls++;
            validationCalls.add("transport:" + transport);
        }
        @Override public void validateRequest(long handle, long request) {
            validationCalls.add("request");
        }
        @Override public void validateRequestWithContext(
                long handle, long request, ProfileValidationContext context) {
            validationCalls.add("request-context");
        }
        @Override public void validateResponse(long handle, long request, long response) {
            validationCalls.add("response");
        }
        @Override public void validateResponseWithContext(
                long handle, long request, long response, ProfileValidationContext context) {
            validationCalls.add("response-context");
        }
        @Override public void release(long handle) { releaseCount++; }
    }
}
