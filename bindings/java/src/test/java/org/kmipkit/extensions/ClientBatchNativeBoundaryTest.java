package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

/** Exercises fixed-width validation at the Java-to-JNI boundary. */
final class ClientBatchNativeBoundaryTest {
    @Test
    void everyNativeResponseAccessorRejectsANullResponseHandle() {
        var responseAccessors = Arrays.stream(NativeExtensionRegistry.class.getDeclaredMethods())
                .filter(method -> Modifier.isNative(method.getModifiers()))
                .filter(method -> method.getName().startsWith("clientBatchItemResponse"))
                .filter(method -> Arrays.equals(method.getParameterTypes(), new Class<?>[] {long.class}))
                .toList();

        assertFalse(responseAccessors.isEmpty());
        for (Method accessor : responseAccessors) {
            InvocationTargetException failure = assertThrows(
                    InvocationTargetException.class,
                    () -> accessor.invoke(null, 0L),
                    accessor.getName());
            assertInstanceOf(InvalidInputException.class, failure.getCause(), accessor.getName());
        }
    }

    @Test
    void asynchronousIndicatorRejectsNullHandlesAndValuesOutsideUint32() {
        try (ClientBatch batch = ClientBatch.create()) {
            long nativeBatch = batch.nativeHandleOwner().get();

            assertThrows(
                    InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(0, 1));
            assertThrows(
                    InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(nativeBatch, -1));
            assertThrows(
                    InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(
                            nativeBatch, 0x1_0000_0000L));
        }
    }

    @Test
    void createFactoriesRejectMissingRequiredAttributesAndOutOfRangeUnsignedValues() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.create(7, null, null));

        try (TtlvStructure attributes = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.create(0x1_0000_0000L, attributes, null));
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.createSplitKey(7, null, 2, 2,
                            0x1_0000_0000L, null, attributes, null));
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.createSplitKey(7, null, 2, 2,
                            1, null, null, null));
        }
    }

    @Test
    void createJniRejectsObjectTypesOutsideUint32() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCreate(-1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCreate(0x1_0000_0000L, 0, 0));
    }

    @Test
    void exportJniRejectsUnknownIdentifierKinds() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExport(
                        77, null, 0, false, 0, false, 0, false, 0, 0));
    }

    @Test
    void exportJniRejectsIdentifierPayloadWhenIdentifierIsAbsent() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExport(
                        0, new byte[] {1}, 0, false, 0, false, 0, false, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExport(
                        0, null, 1, false, 0, false, 0, false, 0, 0));
    }

    @Test
    void importExportAndSplitKeyJniAcceptOptionalBoundariesAndRejectMismatchedPresence() {
        try (TtlvStructure attributes = TtlvStructure.create();
                TtlvStructure anyObject = anyObjectWithSingleItem()) {
            long attributesHandle = attributes.handle();
            long anyObjectHandle = anyObject.handle();
            long imported = 0;
            long exported = 0;
            long splitKey = 0;
            try {
                byte[] importId = "import-native-id".getBytes(java.nio.charset.StandardCharsets.UTF_8);
                imported = NativeExtensionRegistry.clientBatchItemImport(
                        UniqueIdentifier.Kind.TEXT_STRING.raw(), importId, 0,
                        0xffff_ffffL, false, false, true, 0xffff_ffffL,
                        attributesHandle, anyObjectHandle);
                assertNotEquals(0, imported);
                assertEquals(0, NativeExtensionRegistry.clientBatchItemExtensionCount(imported));

                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientBatchItemImport(
                                UniqueIdentifier.Kind.TEXT_STRING.raw(), importId, 0,
                                7, false, true, false, 0, attributesHandle, anyObjectHandle));
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientBatchItemImport(
                                UniqueIdentifier.Kind.TEXT_STRING.raw(), importId, 0,
                                7, false, false, true, 0x1_0000_0000L,
                                attributesHandle, anyObjectHandle));

                exported = NativeExtensionRegistry.clientBatchItemExport(
                        UniqueIdentifier.Kind.TEXT_STRING.raw(),
                        "export-native-id".getBytes(java.nio.charset.StandardCharsets.UTF_8), 0,
                        true, 0xffff_ffffL, false, 0, true, 0xffff_ffffL, 0);
                assertNotEquals(0, exported);
                assertEquals(0, NativeExtensionRegistry.clientBatchItemExtensionCount(exported));
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientBatchItemExport(
                                UniqueIdentifier.Kind.TEXT_STRING.raw(), importId, 0,
                                false, 1, false, 0, false, 0, 0));

                splitKey = NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                        7, 4, 2, 1, UniqueIdentifier.Kind.TEXT_STRING.raw(),
                        "split-native-id".getBytes(java.nio.charset.StandardCharsets.UTF_8), 0,
                        new byte[0], attributesHandle, 0);
                assertNotEquals(0, splitKey);
                assertEquals(0, NativeExtensionRegistry.clientBatchItemExtensionCount(splitKey));
            } finally {
                if (splitKey != 0) NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, splitKey);
                if (exported != 0) NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, exported);
                if (imported != 0) NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, imported);
            }
        }
    }

    private static TtlvStructure anyObjectWithSingleItem() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure object = TtlvStructure.create();
        TtlvStructure anyObject = TtlvStructure.create();
        try (Tag tag = RawTag.fromRaw(0x0042_008f).checked();
                TtlvItem objectItem = TtlvItem.create(tag, TtlvValue.structure(object, limits), limits)) {
            return TtlvStructure.withItem(anyObject, objectItem, limits);
        } finally {
            object.close();
            anyObject.close();
        }
    }

    @Test
    void extensionIdentityLookupRejectsNegativeIndex() {
        try (TtlvStructure attributes = TtlvStructure.create();
                ClientBatchItem item = ClientBatchItem.create(7, attributes, null)) {
            long nativeItem = item.nativeHandleOwner().get();
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemExtensionIdentityAt(
                            nativeItem, -1));
        }
    }

    @Test
    void extensionCriticalityLookupRejectsNegativeIndex() {
        try (TtlvStructure attributes = TtlvStructure.create();
                ClientBatchItem item = ClientBatchItem.create(7, attributes, null)) {
            assertThrows(InvalidInputException.class,
                    () -> item.extensionCriticalityIndicatorAt(-1));
        }
    }

    @Test
    void cryptoResponseJniRejectsNullNativeResponseHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseEncrypt(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDecrypt(0));
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
    }

    @Test
    void objectResponseJniRejectsNullNativeResponseHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseImport(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseLocate(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateSplitKey(0));
    }

    @Test
    void splitKeyFactoryMarshalsTextAndEnumerationIdentifiers() {
        try (TtlvStructure attributes = TtlvStructure.create();
                ClientBatchItem textIdentifier = ClientBatchItem.createSplitKey(
                        7, UniqueIdentifier.textString("request-key-id"), 2, 2, 1,
                        null, attributes, null);
                ClientBatchItem enumerationIdentifier = ClientBatchItem.createSplitKey(
                        7, UniqueIdentifier.enumeration(0xffff_ffffL), 2, 2, 1,
                        null, attributes, null)) {
            assertEquals(0, textIdentifier.extensionCount());
            assertEquals(0, enumerationIdentifier.extensionCount());
        }
    }

    @Test
    void splitKeyJniAcceptsEnumerationIdentifiersAndRejectsOutOfRangeNativeNumbers() {
        long attributes = NativeExtensionRegistry.ttlvStructureCreate();
        long item = 0;
        try {
            item = NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                    7, 2, 2, 1, UniqueIdentifier.Kind.ENUMERATION.raw(), null,
                    0xffff_ffffL, null, attributes, 0);
            assertNotEquals(0, item);
            NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, item);
            item = 0;

            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                            7, 2, 2, 1, UniqueIdentifier.Kind.ENUMERATION.raw(), null,
                            0x1_0000_0000L, null, attributes, 0));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                            7, 2, 2, 1, UniqueIdentifier.Kind.INTEGER.raw(), null,
                            Integer.MIN_VALUE - 1L, null, attributes, 0));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                            7, 2, 2, 1, UniqueIdentifier.Kind.INTEGER.raw(), null,
                            Integer.MAX_VALUE + 1L, null, attributes, 0));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                            7, 2, 2, 1, 4, null, 0, null, attributes, 0));
        } finally {
            if (item != 0) {
                NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, item);
            }
            NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, attributes);
        }
    }

    @Test
    void splitKeyJniRejectsMissingTextAndMismatchedIdentifierPayloads() {
        long attributes = NativeExtensionRegistry.ttlvStructureCreate();
        try {
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                            7, 2, 2, 1, UniqueIdentifier.Kind.TEXT_STRING.raw(), null,
                            0, null, attributes, 0));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                            7, 2, 2, 1, UniqueIdentifier.Kind.ENUMERATION.raw(), new byte[] {1},
                            1, null, attributes, 0));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, attributes);
        }
    }

    @Test
    void executeWithOptionsReleasesBatchWhenJniRejectsTheClientHandle() {
        long batch = NativeExtensionRegistry.clientBatchCreate();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientExecuteWithOptions(0, batch, new long[0], new long[0]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchAddErrorContinuationValue(batch, 0));
    }

    @Test
    void responseAccessorsRejectNullNativeHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchResponseItemCount(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchResponseItemAt(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchResponseItemAt(0, -1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMetadata(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMetadataV2(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseUniqueBatchItemId(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponsePendingCorrelationValue(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDiscoverVersionsSupportedVersions(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreate(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCertify(0));
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
                () -> NativeExtensionRegistry.clientBatchItemResponseGetUsageAllocation(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseExport(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDeriveKey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCheck(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDelegatedLogin(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGet(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseLocate(0));
    }

    @Test
    void responseAccessorsRejectLiveHandlesOfTheWrongNativeKind() {
        long structureHandle = NativeExtensionRegistry.ttlvStructureCreate();
        try {
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchResponseItemCount(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchResponseItemAt(structureHandle, 0));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseMetadata(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseMetadataV2(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseUniqueBatchItemId(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponsePendingCorrelationValue(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseDiscoverVersionsSupportedVersions(
                            structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseCreate(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseCertify(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseCreateKeyPair(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseCreateSplitKey(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseHash(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseMac(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseMacVerify(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseSign(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseSignatureVerify(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseGetUsageAllocation(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseExport(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseDeriveKey(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseCheck(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseDelegatedLogin(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseGet(structureHandle));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemResponseLocate(structureHandle));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, structureHandle);
        }
    }

    @Test
    void getAndLocateFactoriesRejectMalformedOptionalValues() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGet(99, null, 0,
                        false, 0, false, 0, false, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGet(0, null, 0,
                        true, -1, false, 0, false, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGet(0, null, 0,
                        true, 0x1_0000_0000L, false, 0, false, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGet(0, null, 0,
                        false, 1, false, 0, false, 0, 0));

        long attributes = NativeExtensionRegistry.ttlvStructureCreate();
        try {
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemLocate(
                            false, 1, false, 0, false, 0, false, 0, attributes));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemLocate(
                            false, 0, false, 1, false, 0, false, 0, attributes));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemLocate(
                            false, 0, false, 0, true, 0x1_0000_0000L,
                            false, 0, attributes));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchItemLocate(
                            false, 0, false, 0, false, 0,
                            true, 0x1_0000_0000L, attributes));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, attributes);
        }
    }

    @Test
    void deriveKeyJniRejectsMalformedArraysIdentifiersAndLimits() {
        int[] textKind = {UniqueIdentifier.Kind.TEXT_STRING.raw()};
        byte[][] textIdentifier = {new byte[] {0x41}};
        long[] noNumber = {0};

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, null, textIdentifier, noNumber, 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        -1, textKind, textIdentifier, noNumber, 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, textKind, textIdentifier, noNumber, 0x1_0000_0000L, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, new int[0], new byte[0][], new long[0], 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, textKind, new byte[0][], noNumber, 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, textKind, new byte[][] {null}, noNumber, 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, textKind, textIdentifier, new long[] {1}, 1, 0, 0));

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, new int[] {UniqueIdentifier.Kind.ENUMERATION.raw()},
                        new byte[][] {new byte[] {0x41}}, new long[] {1}, 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, new int[] {UniqueIdentifier.Kind.ENUMERATION.raw()},
                        new byte[][] {null}, new long[] {0x1_0000_0000L}, 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, new int[] {UniqueIdentifier.Kind.INTEGER.raw()},
                        new byte[][] {null}, new long[] {Integer.MAX_VALUE + 1L}, 1, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, new int[] {4}, new byte[][] {null}, new long[] {0}, 1, 0, 0));

        int[] tooManyKinds = new int[100_001];
        byte[][] tooManyTexts = new byte[tooManyKinds.length][];
        long[] tooManyNumbers = new long[tooManyKinds.length];
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, tooManyKinds, tooManyTexts, tooManyNumbers, 1, 0, 0));

        byte[] oversizedText = new byte[16_777_217];
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeriveKey(
                        7, textKind, new byte[][] {oversizedText}, noNumber, 1, 0, 0));
    }

    @Test
    void directExecuteRejectsNullClientAndBatchHandles() {
        long batch = NativeExtensionRegistry.clientBatchCreate();
        try {
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientExecute(0, batch, new long[0]));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientExecute(1, 0, new long[0]));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.CLIENT_BATCH, batch);
        }
    }

    @Test
    void cryptoJniReadersRejectMalformedOptionalIdentifiersAndIndicators() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemHash(0, 0, null, 3, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(0, new byte[0], 0, 0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(0, null, 1, 0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(1, null, 0, 0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(1, new byte[] {0x41}, 1, 0, 0,
                        null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(2, new byte[] {0x41}, 1, 0, 0,
                        null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(2, null, -1, 0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(2, null, 0x1_0000_0000L,
                        0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(3, new byte[] {0x41}, 1,
                        0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(3, null, Integer.MIN_VALUE - 1L,
                        0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(3, null, Integer.MAX_VALUE + 1L,
                        0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(77, null, 0, 0, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemSignatureVerify(0, null, 0,
                        0, 0, null, null, null, 0, -1));
    }

    @Test
    void cryptoJniFactoriesRejectInvalidIndicatorsBeforeNativeHandleLookup() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemHash(0, 0, null, 3, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMac(0, null, 0,
                        0, 0, null, 0, 3));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemMacVerify(0, null, 0,
                        0, 0, null, null, 3, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemSign(0, null, 0,
                        0, 0, null, null, 0, 3));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemSignatureVerify(0, null, 0,
                        0, 0, null, null, null, 3, 0));
    }

    @Test
    void certifyJniReaderRejectsMalformedOptionalIdentifierAndRequestTypeFields() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        0, new byte[] {0x41}, 0, false, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        1, null, 0, false, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        2, new byte[] {0x41}, 1, false, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        2, null, -1, false, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        3, null, Integer.MAX_VALUE + 1L, false, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        77, null, 0, false, 0, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        0, null, 0, false, 0x1_0000_0000L, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCertify(
                        0, null, 0, false, 1, null, 0, 0));
    }
}
