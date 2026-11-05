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
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
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
    void jniProfileReadinessRejectsMalformedNativeInputs() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(null, new String[0], new boolean[0]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate("baseline_client", null, new boolean[0]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate("baseline_client", new String[0], null));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        "baseline_client", new String[] {"requirement"}, new boolean[0]));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        "x".repeat(4_097), new String[0], new boolean[0]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        "baseline_client", new String[] {null}, new boolean[] {true}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        "baseline_client", new String[] {"\ud800"}, new boolean[] {true}));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        "baseline_client", new String[] {"x".repeat(16_777_217)}, new boolean[] {true}));

        String[] tooManyRequirements = new String[100_001];
        boolean[] tooManyResults = new boolean[tooManyRequirements.length];
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.profileReadinessCreate(
                        "baseline_client", tooManyRequirements, tooManyResults));
    }

    @Test
    void jniProfileReadinessRoundTripsAValidEvidenceRecord() {
        NativeExtensionRegistry.ensureLoaded();
        long readiness = NativeExtensionRegistry.profileReadinessCreate(
                "KMIPKIT-PROFILE-BASELINE-CLIENT",
                new String[] {"KMIPKIT-REQ-SPEC-14.1-001"}, new boolean[] {false});
        try {
            String[] view = NativeExtensionRegistry.profileReadinessView(readiness);
            assertEquals(7, view.length);
            assertEquals("KMIPKIT-PROFILE-BASELINE-CLIENT", view[0]);
        } finally {
            NativeExtensionRegistry.profileReadinessRelease(readiness);
        }
    }

    @Test
    void jniProfileSelectionRejectsMalformedNativeInputs() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationSelectionValidate(null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationSelectionValidate(new String[] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationSelectionValidate(new String[] {"\ud800"}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationSelectionValidate(new String[] {"x".repeat(4_097)}));

        String[] tooManyProfiles = new String[4_097];
        java.util.Arrays.fill(tooManyProfiles, "baseline_client");
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationSelectionValidate(tooManyProfiles));
    }

    @Test
    void jniProfileSelectionRejectsAnEmptySelection() {
        NativeExtensionRegistry.ensureLoaded();

        ProfileValidationException failure = assertThrows(ProfileValidationException.class,
                () -> NativeExtensionRegistry.profileValidationSelectionValidate(new String[0]));
        assertEquals("empty_selection", failure.profileValidation().errorCode());
    }

    @Test
    void jniProfileValidationResultMethodsRejectInvalidNativeHandles() {
        NativeExtensionRegistry.ensureLoaded();
        long[] noIndexes = new long[0];
        String[] noIdentifiers = new String[0];
        int[] noKnownFields = new int[0];
        int[] noBooleanValues = new int[0];
        long[] noUsageTotals = new long[0];
        long[] noUsageCounts = new long[0];

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultProfileIdCount(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultProfileIdAt(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultRequirementIdCount(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultRequirementIdAt(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultValidateTransport(0, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultValidateRequest(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultValidateRequestWithContext(
                        0, 0, noIndexes, noIdentifiers, noKnownFields,
                        noBooleanValues, noUsageTotals, noUsageCounts));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultValidateResponse(0, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.profileValidationResultValidateResponseWithContext(
                        0, 0, 0, noIndexes, noIdentifiers, noKnownFields,
                        noBooleanValues, noUsageTotals, noUsageCounts));
        NativeExtensionRegistry.profileValidationResultRelease(0);
    }

    @Test
    void jniProfileValidationContextMethodsInitializeParallelArraysBeforeRejectingWrongKindHandles() {
        NativeExtensionRegistry.ensureLoaded();
        long[] itemIndexes = {0};
        String[] uniqueIdentifiers = {"object-7"};
        int[] knownFields = {1};
        int[] booleanValues = {0};
        long[] usageLimitsTotals = {0};
        long[] usageLimitsCounts = {0};

        try (TtlvStructure request = TtlvStructure.create();
                TtlvStructure response = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.profileValidationResultValidateRequestWithContext(
                            request.handle(), request.handle(), itemIndexes, uniqueIdentifiers,
                            knownFields, booleanValues, usageLimitsTotals, usageLimitsCounts));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.profileValidationResultValidateResponseWithContext(
                            request.handle(), request.handle(), response.handle(), itemIndexes,
                            uniqueIdentifiers, knownFields, booleanValues,
                            usageLimitsTotals, usageLimitsCounts));
        }
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
                    ClientBatch unknownBatch = ClientBatch.create();
                    ClientBatch contextBatch = ClientBatch.create()) {
                batch.add(ClientBatchItem.discoverVersions());
                contextBatch.add(ClientBatchItem.discoverVersions());
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

                ProfileValidationContext context = new ProfileValidationContext().withObjectState(
                        0, new ProfileObjectState("object-7")
                                .withOriginalCreationDatePresent(false)
                                .withUsageLimitsPresent(true)
                                .withProtectionUseStarted(false)
                                .withGetUsageAllocationPerformed(true)
                                .withGetUsageAllocationChangedCount(false)
                                .withUsageLimitsTotal(9L)
                                .withUsageLimitsCount(3L));
                ClientOperationException contextFailure = assertThrows(
                        ClientOperationException.class,
                        () -> client.executeWithProfilesAndContext(contextBatch,
                                CodecLimits.defaults(),
                                new ProfileValidationSelection(
                                        List.of("KMIPKIT-PROFILE-BASELINE-CLIENT")), context));
                assertEquals("profile_unavailable",
                        contextFailure.profileValidation().orElseThrow().errorCode());
                assertFalse(context.toString().contains("object-7"));

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

                long clientHandle = rawClientHandle(client);
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientExecuteWithProfiles(
                                clientHandle, NativeExtensionRegistry.clientBatchCreate(),
                                NativeExtensionRegistry.codecLimitValues(CodecLimits.defaults()),
                                new String[] {"KMIPKIT-PROFILE-BASELINE-CLIENT"},
                                new long[] {0}, new String[] {"object-7"}, new int[0],
                                new int[] {0}, new long[] {0}, new long[] {0}, new long[0]));
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientExecuteWithProfiles(
                                clientHandle, NativeExtensionRegistry.clientBatchCreate(),
                                NativeExtensionRegistry.codecLimitValues(CodecLimits.defaults()),
                                new String[] {"KMIPKIT-PROFILE-BASELINE-CLIENT"},
                                new long[0], null, null, null, null, null, new long[0]));
                long[] invalidTimeouts = new long[12];
                invalidTimeouts[0] = 3;
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientExecuteWithProfiles(
                                clientHandle, NativeExtensionRegistry.clientBatchCreate(),
                                NativeExtensionRegistry.codecLimitValues(CodecLimits.defaults()),
                                new String[] {"KMIPKIT-PROFILE-BASELINE-CLIENT"},
                                null, null, null, null, null, null, invalidTimeouts));
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

    private static long rawClientHandle(Client client) throws ReflectiveOperationException {
        var field = Client.class.getDeclaredField("handle");
        field.setAccessible(true);
        return ((NativeHandle) field.get(client)).get();
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
