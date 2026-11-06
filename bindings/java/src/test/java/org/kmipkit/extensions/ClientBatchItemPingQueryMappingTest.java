package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

final class ClientBatchItemPingQueryMappingTest {
    @Test
    void pingFactoryCreatesAnOwnedBatchItem() {
        try (ClientBatchItem item = ClientBatchItem.ping()) {
            assertEquals(0, item.extensionCount());
            assertTrue(item.extensions().isEmpty());
        }
    }

    @Test
    void queryFactoryAcceptsRawValuesAndDistinguishesAbsentFromEmptyObjectGroups() {
        try (ClientBatchItem populated = ClientBatchItem.query(
                        new int[] {1, 0xf123_4567, 1}, new String[] {"group-a", "group-a"});
                ClientBatchItem absent = ClientBatchItem.query(new int[] {14}, null);
                ClientBatchItem presentEmpty = ClientBatchItem.query(new int[] {14}, new String[0])) {
            assertEquals(0, populated.extensionCount());
            assertEquals(0, absent.extensionCount());
            assertEquals(0, presentEmpty.extensionCount());
        }
    }

    @Test
    void queryFactoryAcceptsMoreThanOneHundredThousandFunctionsAndGroups() {
        int[] functions = new int[100_001];
        String[] groups = new String[100_001];
        java.util.Arrays.fill(functions, 1);
        java.util.Arrays.fill(groups, "group");

        try (ClientBatchItem item = ClientBatchItem.queryWithLimits(
                functions, groups, CodecLimits.create(8 * 1024 * 1024L, 64, 400_000))) {
            assertEquals(0, item.extensionCount());
        }
    }

    @Test
    void queryFactoryRequiresLimitsAndEnforcesElementLimit() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.queryWithLimits(new int[] {14}, null, null));
        assertThrows(ResourceLimitException.class, () -> ClientBatchItem.queryWithLimits(
                new int[] {14}, null, CodecLimits.create(1024, 64, 1)));
    }

    @Test
    void queryFactoryAcceptsSupplementaryUnicodeAndRejectsLoneLowSurrogate() {
        try (ClientBatchItem item = ClientBatchItem.query(
                new int[] {14}, new String[] {"group-🔐"})) {
            assertEquals(0, item.extensionCount());
        }

        String loneLowSurrogate = new String(new char[] {'\udc00'});
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.query(new int[] {14}, new String[] {loneLowSurrogate}));
    }

    @Test
    void queryFactoryAcceptsExactTtlvPayloadLimitForPresentEmptyObjectGroups() {
        try (ClientBatchItem item = ClientBatchItem.queryWithLimits(
                new int[] {1}, new String[0], CodecLimits.create(32, 4, 3))) {
            assertEquals(0, item.extensionCount());
        }
    }

    @Test
    void queryFactoryAcceptsObjectGroupLargerThanDefaultMessageLimit() {
        String objectGroup = "x".repeat(16 * 1024 * 1024 + 1);

        try (ClientBatchItem item = ClientBatchItem.queryWithLimits(
                new int[] {1}, new String[] {objectGroup},
                CodecLimits.create(32 * 1024 * 1024L, 64, 100_000))) {
            assertEquals(0, item.extensionCount());
        }
    }

    @Test
    void queryFactoryRejectsDefaultLimitExcessBeforeEncodingObjectGroups() {
        String objectGroup = "x".repeat(1024 * 1024);
        assertThrows(ResourceLimitException.class, () -> ClientBatchItem.queryWithLimits(
                new int[] {1}, new String[] {objectGroup}, CodecLimits.create(32, 64, 100)));
    }

    @Test
    void queryFactoryRejectsInsufficientStructureDepth() {
        assertThrows(ResourceLimitException.class, () -> ClientBatchItem.queryWithLimits(
                new int[] {1}, new String[] {"group"}, CodecLimits.create(1024, 3, 100)));
        assertThrows(ResourceLimitException.class, () -> ClientBatchItem.queryWithLimits(
                new int[] {1}, null, CodecLimits.create(1024, 2, 100)));
    }

    @Test
    void queryTextStringLengthMatchesTheTtlvItemLengthField() {
        assertEquals(0xffff_ffffL, ClientBatchItem.requireTtlvItemLength(0xffff_ffffL));
        assertThrows(ResourceLimitException.class,
                () -> ClientBatchItem.requireTtlvItemLength(0x1_0000_0000L));
    }

    @Test
    void queryJniBridgeRejectsInsufficientDepthBeforeInspectingGroupBytes() {
        assertThrows(ResourceLimitException.class, () ->
                NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                        new int[] {1}, new byte[][] {null}, new long[] {1024, 3, 100}));
    }

    @Test
    void queryJniBridgeAcceptsRawUtf8GroupsAndPreservesAbsentVersusEmptyGroups() {
        byte[][] populatedGroups = {"grupo-ñ".getBytes(StandardCharsets.UTF_8), new byte[0]};
        long populated = NativeExtensionRegistry.clientBatchItemQuery(
                new int[] {1, 0xf123_4567}, populatedGroups);
        long absentGroups = NativeExtensionRegistry.clientBatchItemQuery(new int[] {14}, null);
        long emptyGroups = NativeExtensionRegistry.clientBatchItemQuery(new int[] {14}, new byte[0][]);
        try {
            assertNotEquals(0, populated);
            assertNotEquals(0, absentGroups);
            assertNotEquals(0, emptyGroups);
            assertEquals(0, NativeExtensionRegistry.clientBatchItemExtensionCount(populated));
            assertEquals(0, NativeExtensionRegistry.clientBatchItemExtensionCount(absentGroups));
            assertEquals(0, NativeExtensionRegistry.clientBatchItemExtensionCount(emptyGroups));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, populated);
            NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, absentGroups);
            NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, emptyGroups);
        }
    }

    @Test
    void queryJniBridgeRejectsNullAndEmptyFunctionArrays() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQuery(null, null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQuery(new int[0], null));
    }

    @Test
    void queryJniBridgeRejectsNullGroupBytesAfterLimitsPassPreflight() {
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                new int[] {1}, new byte[][] {null}, new long[] {1024, 4, 100}));
    }

    @Test
    void defaultQueryJniBridgeRejectsElementLimitBeforeInspectingGroupBytes() {
        int[] functions = new int[100_001];
        assertThrows(ResourceLimitException.class, () ->
                NativeExtensionRegistry.clientBatchItemQuery(functions, new byte[][] {null}));
    }

    @Test
    void queryFactoryRequiresAtLeastOneFunction() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.query(null, null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.query(new int[0], null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.query(new int[] {14}, new String[] {null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.query(new int[] {14}, new String[] {new String(new char[] {'\ud800'})}));
    }

    @Test
    void pingResponsePreservesOptionalPresence() {
        assertEquals(Optional.of(Boolean.TRUE), ClientBatchItemResult.parsePingResponse(new long[] {1}));
        assertTrue(ClientBatchItemResult.parsePingResponse(new long[] {0}).isEmpty());
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parsePingResponse(new long[] {2}));
    }

    @Test
    void queryResponseTransfersAnOwnedGenericTtlvAndPreservesAbsence() {
        TtlvStructure payload = TtlvStructure.create();
        long payloadHandle = payload.nativeHandleOwner().transfer();
        TtlvStructure response = ClientBatchItemResult.parseQueryResponse(
                new Object[] {new long[] {1}, new long[] {payloadHandle}}).orElseThrow();
        try (response) {
            assertEquals(0, response.view().itemCount());
        }

        assertTrue(ClientBatchItemResult.parseQueryResponse(
                new Object[] {new long[] {0}, new long[] {0}}).isEmpty());
        ResponseTtlvTestSupport.assertReleased(payloadHandle);
    }

    @Test
    void queryResponseReleasesOwnedPayloadWhenPresenceMappingIsMalformed() {
        TtlvStructure malformed = TtlvStructure.create();
        long payloadHandle = malformed.nativeHandleOwner().transfer();

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseQueryResponse(
                new Object[] {new long[] {0}, new long[] {payloadHandle}}));

        ResponseTtlvTestSupport.assertReleased(payloadHandle);
    }

    @Test
    void queryResponseRejectsPresentPayloadWithoutTtlvHandle() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseQueryResponse(
                new Object[] {new long[] {1}, new long[] {0}}));
    }
}
