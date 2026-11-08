package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.function.Consumer;
import org.junit.jupiter.api.Test;

final class NativeResponseHandleValidationTest {
    private record ResponseBridge(String name, Consumer<Long> invoke) {}

    @Test
    void responseBridgesRejectAValidHandleOfTheWrongKind() {
        NativeExtensionRegistry.ensureLoaded();
        long rawTag = NativeExtensionRegistry.rawTagCreate(0x420001);
        assertTrue(rawTag > 0);
        try {
            for (ResponseBridge bridge : responseBridges()) {
                assertThrows(InvalidInputException.class, () -> bridge.invoke().accept(rawTag), bridge.name());
            }
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(
                            rawTag, 100_000, 16_777_216),
                    "getAttributeList");
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.RAW_TAG, rawTag);
        }
    }

    @Test
    void getAttributeListRejectsNegativeLimitsBeforeReadingTheHandle() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, -1, 16_777_216));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, 100_000, -1));
    }

    @Test
    void profileExecutionRejectsMalformedSelectionAndContextArraysBeforeTransport() throws Exception {
        NativeExtensionRegistry.ensureLoaded();
        long registry = 0;
        long configuration = 0;
        long transport = 0;
        long client = 0;
        long batch = 0;
        try {
            registry = NativeExtensionRegistry.clientExtensionRegistryCreate(new long[0], new long[] {
                    256, 16_384, 256, 4_096, 1_048_576, 4_096,
                    1_048_576, 256, 16_384, 200_000, 1_048_576, 64
            });
            configuration = NativeExtensionRegistry.clientConfigurationCreate(registry);
            byte[] clientCertificate = fixture("client-cert.pem");
            byte[] clientPrivateKey = fixture("client-key.pem");
            byte[] serverCertificate = fixture("server-cert.pem");
            transport = NativeExtensionRegistry.transportConfigCreate(
                    1, "127.0.0.1", 1, 2,
                    new int[] {1}, new int[] {1}, new byte[][] {clientCertificate},
                    1, 1, clientPrivateKey,
                    new int[] {1}, new int[] {1}, new byte[][] {serverCertificate},
                    new int[0], new int[0], new byte[0][],
                    "localhost", null, new long[12], 16_777_216);
            long consumedTransport = transport;
            transport = 0;
            client = NativeExtensionRegistry.clientCreate(configuration, consumedTransport);
            batch = NativeExtensionRegistry.clientBatchCreate();
            long liveClient = client;
            long liveBatch = batch;
            long wrongKind = NativeExtensionRegistry.rawTagCreate(0x420002);
            try {
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientExecuteWithProfiles(
                                wrongKind, wrongKind, new long[] {16_777_216, 64, 100_000},
                                null, null, null, null, null, null, null, new long[12]),
                        "a live wrong-kind client or batch handle must be rejected");
            } finally {
                NativeExtensionRegistry.release(NativeExtensionRegistry.RAW_TAG, wrongKind);
            }

            assertThrows(InvalidInputException.class,
                    () -> executeProfiles(liveClient, liveBatch, new String[4_097],
                            null, null, null, null, null, null),
                    "profile selections over the JNI count limit must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> executeProfiles(liveClient, liveBatch, new String[] {null},
                            null, null, null, null, null, null),
                    "null profile identifiers must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> executeProfiles(liveClient, liveBatch, new String[] {"x".repeat(4_097)},
                            null, null, null, null, null, null),
                    "oversized profile identifiers must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> executeProfiles(liveClient, liveBatch, new String[] {"valid-profile"},
                            new long[0], null, null, null, null, null),
                    "partial profile context arrays must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> executeProfiles(liveClient, liveBatch, new String[] {"valid-profile"},
                            new long[] {0}, new String[] {"object"}, new int[0],
                            new int[] {0}, new long[] {0}, new long[] {0}),
                    "profile context arrays with different lengths must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> executeProfiles(liveClient, liveBatch, new String[] {"valid-profile"},
                            new long[] {0}, new String[] {null}, new int[] {0},
                            new int[] {0}, new long[] {0}, new long[] {0}),
                    "null object identifiers in profile context must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientExecuteWithProfiles(
                            liveClient, liveBatch, new long[] {16_777_216, 64},
                            new String[] {"valid-profile"}, null, null, null, null, null, null, new long[12]),
                    "malformed codec-limit spans must be rejected");
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientExecuteWithProfiles(
                            liveClient, liveBatch, new long[] {16_777_216, 64, 100_000},
                            new String[] {"valid-profile"}, null, null, null, null, null, null, new long[11]),
                    "malformed timeout spans must be rejected");
        } finally {
            if (batch > 0) NativeExtensionRegistry.release(NativeExtensionRegistry.CLIENT_BATCH, batch);
            if (client > 0) NativeExtensionRegistry.release(NativeExtensionRegistry.CLIENT, client);
            if (transport > 0) {
                NativeExtensionRegistry.release(NativeExtensionRegistry.TRANSPORT_CONFIGURATION, transport);
            }
            if (configuration > 0) {
                NativeExtensionRegistry.release(NativeExtensionRegistry.CONFIGURATION, configuration);
            }
            if (registry > 0) NativeExtensionRegistry.release(NativeExtensionRegistry.REGISTRY, registry);
        }
    }

    private static long executeProfiles(
            long client, long batch, String[] profileIds, long[] itemIndexes,
            String[] uniqueIdentifiers, int[] knownFields, int[] booleanValues,
            long[] usageLimitsTotals, long[] usageLimitsCounts) {
        return NativeExtensionRegistry.clientExecuteWithProfiles(
                client, batch, new long[] {16_777_216, 64, 100_000}, profileIds,
                itemIndexes, uniqueIdentifiers, knownFields, booleanValues,
                usageLimitsTotals, usageLimitsCounts, new long[12]);
    }

    private static byte[] fixture(String filename) throws IOException {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IOException("shared deterministic mTLS fixture is missing");
    }

    private static List<ResponseBridge> responseBridges() {
        return List.of(
                new ResponseBridge("metadata", NativeExtensionRegistry::clientBatchItemResponseMetadata),
                new ResponseBridge("metadataV2", NativeExtensionRegistry::clientBatchItemResponseMetadataV2),
                new ResponseBridge("uniqueBatchItemId",
                        NativeExtensionRegistry::clientBatchItemResponseUniqueBatchItemId),
                new ResponseBridge("extensionCount", NativeExtensionRegistry::clientBatchItemResponseExtensionCount),
                new ResponseBridge("extensionTtlvAt", value ->
                        NativeExtensionRegistry.clientBatchItemResponseExtensionTtlvAt(value, 0)),
                new ResponseBridge("pendingCorrelationValue",
                        NativeExtensionRegistry::clientBatchItemResponsePendingCorrelationValue),
                new ResponseBridge("discoverVersionsSupportedVersions",
                        NativeExtensionRegistry::clientBatchItemResponseDiscoverVersionsSupportedVersions),
                new ResponseBridge("create", NativeExtensionRegistry::clientBatchItemResponseCreate),
                new ResponseBridge("certify", NativeExtensionRegistry::clientBatchItemResponseCertify),
                new ResponseBridge("recertify", NativeExtensionRegistry::clientBatchItemResponseRecertify),
                new ResponseBridge("rekey", NativeExtensionRegistry::clientBatchItemResponseRekey),
                new ResponseBridge("rekeyKeyPair", NativeExtensionRegistry::clientBatchItemResponseRekeyKeyPair),
                new ResponseBridge("reprovision", NativeExtensionRegistry::clientBatchItemResponseReprovision),
                new ResponseBridge("reprovisionTtlv", NativeExtensionRegistry::clientBatchItemResponseReprovisionTtlv),
                new ResponseBridge("register", NativeExtensionRegistry::clientBatchItemResponseRegister),
                new ResponseBridge("revoke", NativeExtensionRegistry::clientBatchItemResponseRevoke),
                new ResponseBridge("rngRetrieve", NativeExtensionRegistry::clientBatchItemResponseRngRetrieve),
                new ResponseBridge("rngSeed", NativeExtensionRegistry::clientBatchItemResponseRngSeed),
                new ResponseBridge("activate", NativeExtensionRegistry::clientBatchItemResponseActivate),
                new ResponseBridge("archive", NativeExtensionRegistry::clientBatchItemResponseArchive),
                new ResponseBridge("destroy", NativeExtensionRegistry::clientBatchItemResponseDestroy),
                new ResponseBridge("recover", NativeExtensionRegistry::clientBatchItemResponseRecover),
                new ResponseBridge("check", NativeExtensionRegistry::clientBatchItemResponseCheck),
                new ResponseBridge("delegatedLogin",
                        NativeExtensionRegistry::clientBatchItemResponseDelegatedLogin),
                new ResponseBridge("login", NativeExtensionRegistry::clientBatchItemResponseLogin),
                new ResponseBridge("logout", NativeExtensionRegistry::clientBatchItemResponseLogout),
                new ResponseBridge("getUsageAllocation",
                        NativeExtensionRegistry::clientBatchItemResponseGetUsageAllocation),
                new ResponseBridge("obtainLease", NativeExtensionRegistry::clientBatchItemResponseObtainLease),
                new ResponseBridge("import", NativeExtensionRegistry::clientBatchItemResponseImport),
                new ResponseBridge("joinSplitKey", NativeExtensionRegistry::clientBatchItemResponseJoinSplitKey),
                new ResponseBridge("get", NativeExtensionRegistry::clientBatchItemResponseGet),
                new ResponseBridge("locate", NativeExtensionRegistry::clientBatchItemResponseLocate),
                new ResponseBridge("getConstraints",
                        NativeExtensionRegistry::clientBatchItemResponseGetConstraints),
                new ResponseBridge("getAttributes",
                        NativeExtensionRegistry::clientBatchItemResponseGetAttributes),
                new ResponseBridge("addAttribute",
                        NativeExtensionRegistry::clientBatchItemResponseAddAttribute),
                new ResponseBridge("adjustAttribute",
                        NativeExtensionRegistry::clientBatchItemResponseAdjustAttribute),
                new ResponseBridge("deleteAttribute",
                        NativeExtensionRegistry::clientBatchItemResponseDeleteAttribute),
                new ResponseBridge("modifyAttribute",
                        NativeExtensionRegistry::clientBatchItemResponseModifyAttribute),
                new ResponseBridge("setAttribute",
                        NativeExtensionRegistry::clientBatchItemResponseSetAttribute),
                new ResponseBridge("setConstraints",
                        NativeExtensionRegistry::clientBatchItemResponseSetConstraints),
                new ResponseBridge("setDefaults",
                        NativeExtensionRegistry::clientBatchItemResponseSetDefaults),
                new ResponseBridge("validate", NativeExtensionRegistry::clientBatchItemResponseValidate),
                new ResponseBridge("ping", NativeExtensionRegistry::clientBatchItemResponsePing),
                new ResponseBridge("queryTtlv", NativeExtensionRegistry::clientBatchItemResponseQueryTtlv),
                new ResponseBridge("rawTtlv", NativeExtensionRegistry::clientBatchItemResponseRawTtlv),
                new ResponseBridge("responseHeaderTtlv",
                        NativeExtensionRegistry::clientBatchItemResponseHeaderTtlv),
                new ResponseBridge("interop", NativeExtensionRegistry::clientBatchItemResponseInterop),
                new ResponseBridge("pkcs11", NativeExtensionRegistry::clientBatchItemResponsePkcs11),
                new ResponseBridge("log", NativeExtensionRegistry::clientBatchItemResponseLog),
                new ResponseBridge("export", NativeExtensionRegistry::clientBatchItemResponseExport),
                new ResponseBridge("deriveKey", NativeExtensionRegistry::clientBatchItemResponseDeriveKey),
                new ResponseBridge("createKeyPair",
                        NativeExtensionRegistry::clientBatchItemResponseCreateKeyPair),
                new ResponseBridge("createSplitKey",
                        NativeExtensionRegistry::clientBatchItemResponseCreateSplitKey),
                new ResponseBridge("hash", NativeExtensionRegistry::clientBatchItemResponseHash),
                new ResponseBridge("mac", NativeExtensionRegistry::clientBatchItemResponseMac),
                new ResponseBridge("macVerify", NativeExtensionRegistry::clientBatchItemResponseMacVerify),
                new ResponseBridge("sign", NativeExtensionRegistry::clientBatchItemResponseSign),
                new ResponseBridge("signatureVerify",
                        NativeExtensionRegistry::clientBatchItemResponseSignatureVerify),
                new ResponseBridge("decrypt", NativeExtensionRegistry::clientBatchItemResponseDecrypt),
                new ResponseBridge("encrypt", NativeExtensionRegistry::clientBatchItemResponseEncrypt));
    }
}
