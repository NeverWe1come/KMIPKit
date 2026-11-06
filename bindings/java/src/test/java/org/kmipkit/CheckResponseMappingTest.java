package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.junit.jupiter.api.Test;

final class CheckResponseMappingTest {
    @Test
    void checkBuilderPreservesRawOptionalValues() {
        try (ClientBatchItem item = ClientBatchItem.check(null, Long.MIN_VALUE,
                Integer.MIN_VALUE, 0L)) {
            assertTrue(item != null);
        }
    }

    @Test
    void checkBuilderAllowsEveryOptionalFieldToBeOmitted() {
        try (ClientBatchItem item = ClientBatchItem.check(null, null, null, null)) {
            assertTrue(item != null);
        }
    }

    @Test
    void checkBuilderAcceptsLeaseTimeU32BoundariesAndRejectsOutOfRangeValues() {
        try (ClientBatchItem zero = ClientBatchItem.check(null, null, null, 0L);
             ClientBatchItem maximum = ClientBatchItem.check(null, null, null, 0xffff_ffffL)) {
            assertTrue(zero != null);
            assertTrue(maximum != null);
        }

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.check(null, null, null, -1L));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.check(null, null, null, 0x1_0000_0000L));
    }

    @Test
    void failedResponseMapsPolicyFieldsWithoutIdentifierAndOwnsGenericTtlv() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlvHandle = NativeExtensionRegistry.ttlvStructureCreate();

        try (ClientBatchItemResult.CheckResponse response = ClientBatchItemResult
                .parseCheckResponse(new Object[] {
                    new long[] {1, 0, 1, -9, 1, -1, 1, 0}, null,
                    new long[] {responseTtlvHandle}})
                .orElseThrow()) {
            assertTrue(response.uniqueIdentifier().isEmpty());
            assertEquals(-9L, response.usageLimitsCount().orElseThrow());
            assertEquals(-1, response.cryptographicUsageMask().orElseThrow());
            assertEquals(0L, response.leaseTime().orElseThrow());
            assertTrue(response.responseTtlv().handle() > 0);
            assertFalse(response.toString().contains("-9"));
        }
    }

    @Test
    void absentResponseHasNoTypedFieldsOrOwnedTtlv() {
        assertTrue(ClientBatchItemResult
                .parseCheckResponse(new Object[] {
                    new long[] {0, 0, 0, 0, 0, 0, 0, 0}, null, new long[] {0}})
                .isEmpty());
    }

    @Test
    void absentResponseRejectsTypedFieldsAndGenericTtlv() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCheckResponse(new Object[] {
                    new long[] {0, 0, 1, 0, 0, 0, 0, 0}, null, new long[] {0}
                }));

        NativeExtensionRegistry.ensureLoaded();
        long responseTtlvHandle = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCheckResponse(new Object[] {
                    new long[] {0, 0, 0, 0, 0, 0, 0, 0}, null,
                    new long[] {responseTtlvHandle}
                }));
    }

    @Test
    void absentResponseClosesTtlvWhenTypedFieldsAreInconsistent() {
        NativeExtensionRegistry.ensureLoaded();
        long responseTtlvHandle = NativeExtensionRegistry.ttlvStructureCreate();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCheckResponse(new Object[] {
                    new long[] {0, 0, 1, 0, 0, 0, 0, 0}, null,
                    new long[] {responseTtlvHandle}
                }));

        long viewHandle = 0;
        boolean closed;
        try {
            viewHandle = NativeExtensionRegistry.ttlvStructureView(responseTtlvHandle);
            closed = false;
        } catch (InvalidInputException expected) {
            closed = true;
        } finally {
            if (viewHandle != 0) {
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE_VIEW, viewHandle);
                NativeExtensionRegistry.release(NativeExtensionRegistry.TTLV_STRUCTURE, responseTtlvHandle);
            }
        }

        assertTrue(closed, "the invalid Check response must release its owned TTLV handle");
    }

    @Test
    void presentResponseRequiresOwnedTtlvAndValidPolicyFields() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCheckResponse(new Object[] {
                    new long[] {1, 0, 0, 0, 0, 0, 0, 0}, null, new long[] {0}
                }));

        NativeExtensionRegistry.ensureLoaded();
        long responseTtlvHandle = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCheckResponse(new Object[] {
                    new long[] {1, 0, 0, 0, 1, 0x1_0000_0000L, 0, 0}, null,
                    new long[] {responseTtlvHandle}
                }));
    }

    @Test
    void presentResponseValidatesAbsentOptionalFieldsAndMapsIdentifier() {
        NativeExtensionRegistry.ensureLoaded();
        long absentCountTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCheckResponse(new Object[] {
                    new long[] {1, 0, 0, -1, 0, 0, 0, 0}, null,
                    new long[] {absentCountTtlv}
                }));
        ResponseTtlvTestSupport.assertReleased(absentCountTtlv);

        long absentIdentifierTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        Object[] unexpectedIdentifier = {
            new long[] {1, 0, 0, 0, 0, 0, 0, 0},
            new Object[] {new long[] {1, 3, 0, -9}, null},
            new long[] {absentIdentifierTtlv}
        };
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseCheckResponse(unexpectedIdentifier));
        ResponseTtlvTestSupport.assertReleased(absentIdentifierTtlv);

        long presentIdentifierTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        try (ClientBatchItemResult.CheckResponse response = ClientBatchItemResult
                .parseCheckResponse(new Object[] {
                    new long[] {1, 1, 0, 0, 0, 0, 0, 0},
                    new Object[] {new long[] {1, 3, 0, -9}, null},
                    new long[] {presentIdentifierTtlv}
                })
                .orElseThrow()) {
            assertEquals(UniqueIdentifier.Kind.INTEGER, response.uniqueIdentifier().orElseThrow().kind());
        }
    }
}
