package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

final class ClientBatchItemResultFocusedCoverageTest {
    private static final CodecLimits LIMITS = CodecLimits.defaults();

    @Test
    void attributeMutationResponsesExposeSuccessfulIdentifiersAndRedactFormatting() {
        var add = ClientBatchItemResult.parseAddAttributeResponse(
                new Object[] {new long[] {1}, "added-object"}).orElseThrow();
        var adjust = ClientBatchItemResult.parseAdjustAttributeResponse(
                new Object[] {new long[] {1}, "adjusted-object"}).orElseThrow();
        var delete = ClientBatchItemResult.parseDeleteAttributeResponse(
                new Object[] {new long[] {1}, "deleted-object"}).orElseThrow();
        var modify = ClientBatchItemResult.parseModifyAttributeResponse(
                new Object[] {new long[] {1}, "modified-object"}).orElseThrow();
        var set = ClientBatchItemResult.parseSetAttributeResponse(
                new Object[] {new long[] {1}, "set-object"}).orElseThrow();

        assertEquals("added-object", add.uniqueIdentifier());
        assertEquals("adjusted-object", adjust.uniqueIdentifier());
        assertEquals("deleted-object", delete.uniqueIdentifier());
        assertEquals("modified-object", modify.uniqueIdentifier());
        assertEquals("set-object", set.uniqueIdentifier());
        assertFalse(add.toString().contains("added-object"));
        assertFalse(adjust.toString().contains("adjusted-object"));
        assertFalse(delete.toString().contains("deleted-object"));
        assertFalse(modify.toString().contains("modified-object"));
        assertFalse(set.toString().contains("set-object"));

        assertTrue(ClientBatchItemResult.parseAddAttributeResponse(
                new Object[] {new long[] {0}, null}).isEmpty());
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseAddAttributeResponse(
                new Object[] {new long[] {0}, "unexpected"}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseAddAttributeResponse(
                new Object[] {new long[] {1}, null}));
        assertTrue(ClientBatchItemResult.parseAdjustAttributeResponse(
                new Object[] {new long[] {0}, null}).isEmpty());
        assertTrue(ClientBatchItemResult.parseDeleteAttributeResponse(
                new Object[] {new long[] {0}, null}).isEmpty());
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseAdjustAttributeResponse(
                new Object[] {new long[] {0}, "unexpected"}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseAdjustAttributeResponse(
                new Object[] {new long[] {1}, new byte[] {1}}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseDeleteAttributeResponse(
                new Object[] {new long[] {0}, "unexpected"}));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseDeleteAttributeResponse(
                new Object[] {new long[] {1}, null}));
    }

    @Test
    void lifecycleAndRegisterMappingsDistinguishAbsentAndMalformedSuccesses() {
        assertTrue(ClientBatchItemResult.parseLifecycleIdentifierResponse(
                new Object[] {new long[] {0}, null}, "Recover").isEmpty());
        assertThrows(InvalidInputException.class, () ->
                ClientBatchItemResult.parseLifecycleIdentifierResponse(
                        new Object[] {new long[] {0}, "unexpected"}, "Recover"));
        var identifier = ClientBatchItemResult.parseLifecycleIdentifierResponse(
                new Object[] {new long[] {1}, new Object[] {
                    new long[] {1, 1, 0, 0}, "recovered-object".getBytes(StandardCharsets.UTF_8)
                }}, "Recover").orElseThrow();
        assertEquals("recovered-object", identifier.text());

        assertTrue(ClientBatchItemResult.parseRegisterResponse(
                new Object[] {new long[] {0}, null, new long[] {0}}).isEmpty());
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseRegisterResponse(
                new Object[] {new long[] {1}, null, new long[] {0}}));
    }

    @Test
    void successfulCryptoMappingRequiresAndTransfersItsCompleteResponseTtlv() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseHashResponse(
                new Object[] {new long[] {1, 0, 0, 0}, null, null}));

        TtlvStructure source = TtlvStructure.create();
        long handle = source.nativeHandleOwner().transfer();
        var response = ClientBatchItemResult.parseHashResponse(
                new Object[] {new long[] {1, 1, 0, handle}, new byte[] {3, 1, 4}, null})
                .orElseThrow();
        try (TtlvStructure responseTtlv = response.responseTtlv();
                TtlvStructureView view = responseTtlv.view()) {
            assertEquals(0, view.itemCount());
            assertTrue(response.correlationValue().isEmpty());
            assertEquals(3, response.data().orElseThrow().length);
        }
    }

    @Test
    void getAttributeListMapsOrderedReferencesAndReleasesRejectedPayloads() {
        long payloadHandle = emptyStructureHandle();
        var response = ClientBatchItemResult.parseGetAttributeListResponse(new Object[] {
            new long[] {1}, UniqueIdentifier.textString("object-8"),
            new long[] {1, 2}, new long[] {0x0054_0001L, 0},
            new String[] {null, "vendor"}, new String[] {null, "Policy.Name"},
            new long[] {payloadHandle}
        }, LIMITS.maxElements(), LIMITS.maxMessageBytes()).orElseThrow();
        assertEquals("object-8", response.uniqueIdentifier().text());
        assertEquals(0x0054_0001L, response.attributeReferences().get(0).tagValue().orElseThrow());
        assertEquals("vendor", response.attributeReferences().get(1).vendorIdentification().orElseThrow());
        assertEquals("Policy.Name", response.attributeReferences().get(1).attributeName().orElseThrow());
        assertFalse(response.toString().contains("object-8"));
        response.close();

        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetAttributeListResponse(
                new Object[0], -1, LIMITS.maxMessageBytes()));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetAttributeListResponse(
                new Object[] {new long[] {1}, UniqueIdentifier.textString("object-8"), "wrong-kinds",
                    new long[0], new String[0], new String[0], new long[] {0}},
                LIMITS.maxElements(), LIMITS.maxMessageBytes()));
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetAttributeListResponse(
                new Object[] {new long[] {1}, UniqueIdentifier.textString("object-8"), new long[0],
                    new long[0], new Object[0], new String[0], new long[] {0}},
                LIMITS.maxElements(), LIMITS.maxMessageBytes()));
        assertThrows(ResourceLimitException.class, () -> ClientBatchItemResult.parseGetAttributeListResponse(
                new Object[] {new long[] {1}, UniqueIdentifier.textString("object-8"), new long[] {1},
                    new long[] {0x0054_0001L}, new String[] {null}, new String[] {null}, new long[] {0}},
                LIMITS.maxElements(), 15));
        assertThrows(ResourceLimitException.class, () -> ClientBatchItemResult.parseGetAttributeListResponse(
                new Object[] {new long[] {1}, UniqueIdentifier.textString("object-8"), new long[] {1},
                    new long[] {0x0054_0001L}, new String[] {null, null}, new String[] {null},
                    new long[] {0}},
                LIMITS.maxElements(), 16));

        long absentPayload = emptyStructureHandle();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetAttributeListResponse(
                new Object[] {new long[] {0}, null, new long[0], new long[0],
                    new String[0], new String[0], new long[] {absentPayload}},
                LIMITS.maxElements(), LIMITS.maxMessageBytes()));
        ResponseTtlvTestSupport.assertReleased(absentPayload);

        long malformedPayload = emptyStructureHandle();
        assertThrows(InvalidInputException.class, () -> ClientBatchItemResult.parseGetAttributeListResponse(
                new Object[] {new long[] {1}, UniqueIdentifier.textString("object-8"),
                    new long[] {99}, new long[] {0}, new String[] {null},
                    new String[] {null}, new long[] {malformedPayload}},
                LIMITS.maxElements(), LIMITS.maxMessageBytes()));
        ResponseTtlvTestSupport.assertReleased(malformedPayload);
    }

    private static long emptyStructureHandle() {
        try (TtlvStructure structure = TtlvStructure.create()) {
            return structure.nativeHandleOwner().transfer();
        }
    }
}
