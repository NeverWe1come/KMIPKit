package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ClientRequestMessageExtension;
import org.kmipkit.extensions.Compatibility;
import org.kmipkit.extensions.Discriminator;
import org.kmipkit.extensions.DuplicateExtensionKeyException;
import org.kmipkit.extensions.ExtensionChildRule;
import org.kmipkit.extensions.ExtensionCompatibilityException;
import org.kmipkit.extensions.ExtensionDefinition;
import org.kmipkit.extensions.ExtensionIdentity;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ExtensionSchema;
import org.kmipkit.extensions.InvalidExtensionIdentityException;
import org.kmipkit.extensions.RegisteredExtensionValue;
import org.kmipkit.extensions.TtlvPath;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class NativeBridgeCoverageTest {
    private static final long[] CODEC_LIMITS = {16_777_216, 64, 100_000};
    private static final long[] REGISTRY_LIMITS = {
            256, 16_384, 256, 4_096, 1_048_576, 4_096,
            1_048_576, 256, 16_384, 200_000, 1_048_576, 64
    };
    private static final int CRITICALITY_EXTENSION_TAG = 0x420001;
    private static final CodecLimits DEFAULT_CODEC_LIMITS = CodecLimits.defaults();

    @Test
    void nativeAbiVersionMatchesTheExpectedVersion() {
        NativeExtensionRegistry.ensureLoaded();

        assertEquals(1, NativeExtensionRegistry.nativeAbiVersion());
    }

    @Test
    void nativeStringBridgeRoundTripsUnicodeAndRejectsInvalidJavaText() {
        String text = "two-byte é, three-byte €, four-byte 🔐";
        long identity = NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", text, "1");
        try {
            assertEquals(text, NativeExtensionRegistry.extensionIdentityName(identity));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.IDENTITY, identity);
        }

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", null, "1"));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "\ud800", "1"));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "\udc00", "1"));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "x".repeat(4_097), "1"));
    }

    @Test
    void nativeArrayReadersRejectNullMalformedAndOverLimitInputs() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueTextString(null, CODEC_LIMITS));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueTextString(new byte[0], null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueBigInteger(new byte[0], new long[] {1, 2}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueBigInteger(new byte[0], new long[] {-1, 2, 3}));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.ttlvValueByteString(new byte[2], new long[] {1, 1, 1}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionSchemaStructure(null, new long[0], false));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionSchemaStructure(new long[] {0}, new long[0], false));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.codecLimitsCreate(-1, 1, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.release(-1, 1));
    }

    @Test
    void nativeStatusFailuresBecomeTypedErrorsForNullHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExtensionCount(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExtensionIdentityAt(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExtensionCriticalityIndicatorAt(0, 0));
    }

    @Test
    void nativeBatchItemCriticalityReaderReturnsBothCallerSelectedValues() {
        try (RawTag rawTag = RawTag.fromRaw(CRITICALITY_EXTENSION_TAG);
                Tag tag = rawTag.checked();
                ExtensionIdentity identity = ExtensionIdentity.create("coverage.vendor", "criticality", "1");
                Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "0.0.0", "99.0.0");
                TtlvPath discriminatorPath = TtlvPath.create(tag);
                TtlvValue discriminatorValue = TtlvValue.TextString(
                        "marker".getBytes(StandardCharsets.UTF_8), DEFAULT_CODEC_LIMITS);
                ExtensionSchema markerSchema = ExtensionSchema.scalar(TtlvItemType.TextString);
                ExtensionChildRule childRule = ExtensionChildRule.required(tag, markerSchema);
                ExtensionSchema schema = ExtensionSchema.structure(List.of(childRule), List.of(), false);
                Discriminator discriminator = Discriminator.create(discriminatorPath, discriminatorValue);
                TtlvStructure payload = markerPayload()) {
            ExtensionDefinition definition = ExtensionDefinition.create(
                    identity, compatibility, discriminator, schema);
            try (definition;
                    ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(definition), ExtensionRegistryLimits.defaults())) {
                for (boolean criticalityIndicator : new boolean[] {false, true}) {
                    try (RegisteredExtensionValue currentValue = registry.validateExtensionValue(
                                    identity, payload, DEFAULT_CODEC_LIMITS);
                            ClientRequestMessageExtension extension = ClientRequestMessageExtension.create(
                                    currentValue, criticalityIndicator);
                            ClientBatchItem baseItem = ClientBatchItem.discoverVersions()) {
                        ClientBatchItem itemWithExtension = baseItem.withExtension(extension);
                        try (itemWithExtension) {
                            assertEquals(criticalityIndicator,
                                    itemWithExtension.extensionCriticalityIndicatorAt(0));
                        }
                    }
                }
            }
        }
    }

    private static TtlvStructure markerPayload() {
        try (RawTag rawTag = RawTag.fromRaw(CRITICALITY_EXTENSION_TAG);
                Tag tag = rawTag.checked();
                TtlvStructure empty = TtlvStructure.create();
                TtlvValue value = TtlvValue.TextString(
                        "marker".getBytes(StandardCharsets.UTF_8), DEFAULT_CODEC_LIMITS);
                TtlvItem item = TtlvItem.create(tag, value, DEFAULT_CODEC_LIMITS)) {
            return TtlvStructure.withItem(empty, item, DEFAULT_CODEC_LIMITS);
        }
    }

    @Test
    void nativeTypedResponseBridgesRejectNullItemHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMetadataV2(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreate(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCertify(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCheck(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDelegatedLogin(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetUsageAllocation(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetConstraints(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseExport(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, -1, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, 1, -1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, 1, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDeriveKey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateKeyPair(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateSplitKey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseHash(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMac(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMacVerify(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseSign(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseSignatureVerify(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDiscoverVersionsSupportedVersions(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseJoinSplitKey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvStructureView(0));
    }

    @Test
    void nativeResponseBridgesRejectNullHandlesForRemainingOperations() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseRekey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponsePing(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseQueryTtlv(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponsePkcs11(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateSplitKey(0));
    }

    @Test
    void usageAllocationContextBridgesRejectNullHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextUsageLimitsUnit(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextAllocatedUnits(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextAvailableUnits(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextReservedUnits(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextUniqueIdentifier(0));
    }

    @Test
    void nativeQueryBridgesRejectMalformedObjectGroupText() {
        byte[][] malformedObjectGroups = {new byte[] {(byte) 0xff}};
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQuery(new int[] {1}, malformedObjectGroups));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                        new int[] {1}, malformedObjectGroups, CODEC_LIMITS));
    }

    @Test
    void nativeQueryWithLimitsEnforcesElementBudgetBeforeAllocation() {
        long[] limitsBelowMinimumQuerySize = {16_777_216, 64, 1};
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                        new int[] {1}, null, limitsBelowMinimumQuerySize));
    }

    @Test
    void nativeValidationBridgesRejectMalformedBoundaryInputs() {
        byte[][] nullObjectGroups = new byte[1][];
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQuery(new int[] {1}, nullObjectGroups));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                        new int[] {1}, nullObjectGroups, CODEC_LIMITS));

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.splitKeyReferenceCreate(0, null, 0, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemJoinSplitKey(
                        -1, new long[] {1}, false, 0, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemJoinSplitKey(
                        1, new long[0], false, 0, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemJoinSplitKey(
                        1, new long[] {0}, false, 0, 0, 0));

        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.ttlvValueEnumeration(-1));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.ttlvValueInterval(-1));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientCreate(0, 0));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientBatchAddItem(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchResponseItemAt(0, -1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextCreate(0, 1, 0));
    }

    @Test
    void nativeAttributeRequestBridgesRejectMalformedReferenceArrays() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {1}, new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {1}, new long[] {-1},
                        new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {2}, new long[] {0},
                        new byte[][] {null}, new byte[][] {"Comment".getBytes(StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {99}, new long[] {0},
                        new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {1}, new long[] {0x1_0000_0000L},
                        new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {2}, new long[] {1},
                        new byte[][] {"KMIPKit".getBytes(StandardCharsets.UTF_8)},
                        new byte[][] {"Comment".getBytes(StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {2}, new long[] {0},
                        new byte[][] {new byte[] {(byte) 0xff}},
                        new byte[][] {"Comment".getBytes(StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        99, null, 0, new int[0], new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, null, new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        2, null, 1, new int[0], new long[0], new byte[0][], new byte[0][]));
        int[] tooManyKinds = new int[100_001];
        long[] tooManyTags = new long[tooManyKinds.length];
        byte[][] tooManyVendors = new byte[tooManyKinds.length][];
        byte[][] tooManyNames = new byte[tooManyKinds.length][];
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, tooManyKinds, tooManyTags, tooManyVendors, tooManyNames));
    }

    @Test
    void nativeAttributeRequestCreateRejectsInvalidOperationsAndFields() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        99, 0, null, 0, 0, 0, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        0, 99, null, 0, 0, 1, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        0, 0, null, 0, 0, 0, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        1, 0, null, 0, 0, 0, 0, 0, null, null, 0x1_0000_0000L, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        0, 0, null, 0, 0, 1, 0, 0, null, null, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        1, 0, null, 0, 0, 0, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        2, 0, null, 0, 0, 1, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        3, 0, null, 0, 0, 1, 1, 1, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        4, 0, null, 0, 1, 1, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        2, 2, null, 1, 0, 0, 0, 0, null, null, 0, 0));
    }

    @Test
    void nativeNamedAttributeAdjustAndDeleteReachTheirAbiValidationPaths() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] vendor = "KMIPKit".getBytes(StandardCharsets.UTF_8);
        byte[] name = "Comment".getBytes(StandardCharsets.UTF_8);

        long adjusted = NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                0, null, 0, 2, 0, vendor, name, 1, 0);
        assertNotEquals(0, adjusted);
        NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, adjusted);

        long deleted = NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                0, null, 0, 0, 2, 0, vendor, name);
        assertNotEquals(0, deleted);
        NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, deleted);

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 99, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 1, -1, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 1, 0, null, null, 0x1_0000_0000L, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 1, 0, null, name, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        99, null, 0, 2, 0, vendor, name, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        2, null, 1, 2, 0, vendor, name, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 2, 0, new byte[] {(byte) 0xff}, name, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        0, null, 0, 0, 99, 0, null, null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        0, null, 0, 0, 1, 0, vendor, null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        2, null, 1, 0, 0, 0, null, null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        99, null, 0, 0, 2, 0, vendor, name));

        long deletedTag = NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                0, null, 0, 0, 1, 0x0042_000a, null, null);
        assertNotEquals(0, deletedTag);
        NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, deletedTag);
    }

    @Test
    void nativePkcs11BridgeRejectsMissingTtlvInputs() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemPkcs11(0, 0, null, 0));
    }

    @Test
    void nativeRekeyBridgeRejectsInvalidIdentifiersAndOffsets() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemRekey(99, null, 0, false, 0, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemRekey(
                        1, new byte[] {'k'}, 0, true, -1, 0, 0));
    }

    @Test
    void nativeClientOperationBridgesRejectMalformedIdentifiersAndAbsentValues() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCheck(
                        4, null, 0, false, 0, false, 0, false, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCheck(
                        0, null, 0, false, 0, false, 0, false, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetUsageAllocation(4, null, 0, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                        1, 2, 1, 1, 3, null, (long) Integer.MAX_VALUE + 1,
                        null, 0, 0));
    }

    @Test
    void nativeReleaseIgnoresNullAndForgedPositiveJniHandles() {
        assertDoesNotThrow(() -> NativeExtensionRegistry.release(NativeExtensionRegistry.TAG, 0));
        assertDoesNotThrow(() -> NativeExtensionRegistry.release(NativeExtensionRegistry.TAG, 0x1000L));
    }

    @Test
    void nativeErrorClearResetsOnlyCallingThreadSnapshot() throws Exception {
        RuntimeMetadata.clearLastError();
        ExecutorService worker = Executors.newSingleThreadExecutor();
        try {
            assertThrows(ResourceLimitException.class,
                    () -> RuntimeMetadata.capabilityIsAvailable("x".repeat(129)));
            RuntimeMetadata.ErrorSnapshot callerError = new RuntimeMetadata.ErrorSnapshot(6, 1, 10, 0);
            assertEquals(callerError, RuntimeMetadata.lastErrorSnapshot());

            worker.submit(() -> {
                assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientBatchItemExtensionCount(0));
                assertEquals(new RuntimeMetadata.ErrorSnapshot(4, 1, 1, 0), RuntimeMetadata.lastErrorSnapshot());

                RuntimeMetadata.clearLastError();
                assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
            }).get(10, TimeUnit.SECONDS);

            assertEquals(callerError, RuntimeMetadata.lastErrorSnapshot());
            RuntimeMetadata.clearLastError();
            assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
        } finally {
            worker.shutdownNow();
            RuntimeMetadata.clearLastError();
        }
    }

    @Test
    void nativeRegistryLookupsAndOptionalInformationUseTypedResults() {
        long identity = 0;
        long compatibility = 0;
        long rootRawTag = 0;
        long rootTag = 0;
        long childRawTag = 0;
        long childTag = 0;
        long rootPath = 0;
        long path = 0;
        long scalar = 0;
        long discriminator = 0;
        long scalarSchema = 0;
        long childRule = 0;
        long childSchema = 0;
        long rootRule = 0;
        long schema = 0;
        long definition = 0;
        long registry = 0;
        long definitionAt = 0;
        long definitionForIdentity = 0;
        long codecLimits = 0;
        try {
            identity = NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "registry", "1");
            compatibility = NativeExtensionRegistry.compatibilityCreate(
                    2, 1, 2, 1, "0.0.0", "99.0.0");
            rootRawTag = NativeExtensionRegistry.rawTagCreate(0x420001);
            rootTag = NativeExtensionRegistry.rawTagChecked(rootRawTag);
            childRawTag = NativeExtensionRegistry.rawTagCreate(0x420002);
            childTag = NativeExtensionRegistry.rawTagChecked(childRawTag);
            rootPath = NativeExtensionRegistry.ttlvPathCreate(rootTag);
            path = NativeExtensionRegistry.ttlvPathWithChildTag(rootPath, childTag);
            scalar = NativeExtensionRegistry.ttlvValueTextString(
                    "marker".getBytes(StandardCharsets.UTF_8), CODEC_LIMITS);
            discriminator = NativeExtensionRegistry.discriminatorCreate(path, scalar);
            scalarSchema = NativeExtensionRegistry.extensionSchemaScalar(TtlvItemType.TextString.code());
            childRule = NativeExtensionRegistry.extensionChildRuleRequired(childTag, scalarSchema);
            childSchema = NativeExtensionRegistry.extensionSchemaStructure(
                    new long[] {childRule}, new long[0], false);
            rootRule = NativeExtensionRegistry.extensionChildRuleRequired(rootTag, childSchema);
            schema = NativeExtensionRegistry.extensionSchemaStructure(new long[] {rootRule}, new long[0], false);
            definition = NativeExtensionRegistry.extensionDefinitionCreate(
                    identity, compatibility, discriminator, schema);
            registry = NativeExtensionRegistry.clientExtensionRegistryCreate(
                    new long[] {definition}, REGISTRY_LIMITS);

            definitionAt = NativeExtensionRegistry.clientExtensionRegistryDefinitionAt(registry, 0);
            definitionForIdentity = NativeExtensionRegistry.clientExtensionRegistryDefinitionForIdentity(
                    registry, identity);
            assertNotEquals(0, definitionAt);
            assertNotEquals(0, definitionForIdentity);
            assertEquals(0, NativeExtensionRegistry.extensionDefinitionInformation(definition));
            codecLimits = NativeExtensionRegistry.codecLimitsCreate(1_024, 10, 100);
            assertNotEquals(0, codecLimits);

            long duplicateDefinition = definition;
            assertThrows(DuplicateExtensionKeyException.class,
                    () -> NativeExtensionRegistry.clientExtensionRegistryCreate(
                            new long[] {duplicateDefinition, duplicateDefinition}, REGISTRY_LIMITS));
            assertThrows(ExtensionCompatibilityException.class,
                    () -> NativeExtensionRegistry.compatibilityCreate(
                            2, 2, 2, 1, "0.0.0", "99.0.0"));
            assertThrows(InvalidExtensionIdentityException.class,
                    () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "", "1"));
        } finally {
            release(26, codecLimits);
            release(NativeExtensionRegistry.DEFINITION, definitionForIdentity);
            release(NativeExtensionRegistry.DEFINITION, definitionAt);
            release(NativeExtensionRegistry.REGISTRY, registry);
            release(NativeExtensionRegistry.DEFINITION, definition);
            release(NativeExtensionRegistry.SCHEMA, schema);
            release(NativeExtensionRegistry.CHILD_RULE, rootRule);
            release(NativeExtensionRegistry.SCHEMA, childSchema);
            release(NativeExtensionRegistry.CHILD_RULE, childRule);
            release(NativeExtensionRegistry.SCHEMA, scalarSchema);
            release(NativeExtensionRegistry.DISCRIMINATOR, discriminator);
            release(NativeExtensionRegistry.TTLV_VALUE, scalar);
            release(NativeExtensionRegistry.PATH, path);
            release(NativeExtensionRegistry.PATH, rootPath);
            release(NativeExtensionRegistry.TAG, childTag);
            release(NativeExtensionRegistry.RAW_TAG, childRawTag);
            release(NativeExtensionRegistry.TAG, rootTag);
            release(NativeExtensionRegistry.RAW_TAG, rootRawTag);
            release(NativeExtensionRegistry.COMPATIBILITY, compatibility);
            release(NativeExtensionRegistry.IDENTITY, identity);
        }
    }

    private static void release(int handleKind, long handle) {
        if (handle != 0) {
            NativeExtensionRegistry.release(handleKind, handle);
        }
    }
}
