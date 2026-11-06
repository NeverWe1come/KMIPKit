package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.junit.jupiter.api.Test;

/**
 * Obtain Lease parity for OASIS KMIP 2.1 §6.1.35, Tables 268–270.
 * Traces KMIPKIT-REQ-SPEC-6.1.35-001-001/-001-002/-002/-005 and
 * owner requirements KMIPKIT-0036-FR-001 through KMIPKIT-0036-FR-008.
 */
final class ObtainLeaseMappingTest {
    @Test
    void requestBuilderSupportsPlaceholderAndExplicitIdentifier() {
        try (ClientBatchItem omitted = ClientBatchItem.obtainLease(null);
                ClientBatchItem explicit = ClientBatchItem.obtainLease(
                        UniqueIdentifier.textString("leased-object"))) {
            assertNotNull(omitted);
            assertNotNull(explicit);
        }
    }

    @Test
    void successfulResponsePreservesZeroLeaseAndSignedDateAndRedactsIdentifier() {
        long ttlvHandle = responseTtlv().nativeHandleOwner().transfer();
        Object[] identifier = {new long[] {1, 1, 0, 0},
                "leased-object".getBytes(StandardCharsets.UTF_8)};
        ClientBatchItemResult.ObtainLeaseResponse response = ClientBatchItemResult
                .parseObtainLeaseResponse(new Object[] {
                    new long[] {1, 0, -7}, identifier, new long[] {ttlvHandle}
                })
                .orElseThrow();

        try (response) {
            assertEquals(UniqueIdentifier.Kind.TEXT_STRING, response.uniqueIdentifier().kind());
            assertEquals("leased-object", response.uniqueIdentifier().text());
            assertEquals(0, response.leaseTime());
            assertEquals(-7, response.lastChangeDate());
            assertFalse(response.toString().contains("leased-object"));
        }
    }

    @Test
    void absentResponseDoesNotCreateSuccessFields() {
        assertTrue(ClientBatchItemResult
                .parseObtainLeaseResponse(new Object[] {
                    new long[] {0, 0, 0}, null, new long[] {0}
                })
                .isEmpty());
    }

    @Test
    void successfulResponseOwnsOrderedTtlvAfterSourceRelease() {
        TtlvStructure source = responseTtlv();
        TtlvStructure copy = responseTtlv();
        long copiedHandle = copy.nativeHandleOwner().transfer();
        Object[] identifier = {new long[] {1, 1, 0, 0},
                "leased-object".getBytes(StandardCharsets.UTF_8)};
        ClientBatchItemResult.ObtainLeaseResponse response = ClientBatchItemResult
                .parseObtainLeaseResponse(new Object[] {
                    new long[] {1, 0, -7}, identifier, new long[] {copiedHandle}
                })
                .orElseThrow();

        source.close();
        try (response; TtlvStructureView view = response.responseTtlv().view()) {
            assertEquals(4, view.itemCount());
            assertEquals(0x0042_0094, tagAt(view, 0));
            assertEquals(0x0042_0049, tagAt(view, 1));
            assertEquals(0x0042_0048, tagAt(view, 2));
            assertEquals(0x0054_1234, tagAt(view, 3));
        }
    }

    @Test
    void malformedResponseReleasesOwnedTtlvAndRejectsMissingRequiredFields() {
        NativeExtensionRegistry.ensureLoaded();
        long ttlvHandle = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseObtainLeaseResponse(new Object[] {
                    new long[] {0, 0, 0}, null, new long[] {ttlvHandle}
                }));
        ResponseTtlvTestSupport.assertReleased(ttlvHandle);

        long missingIdentifierHandle = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseObtainLeaseResponse(new Object[] {
                    new long[] {1, 0, -7}, null, new long[] {missingIdentifierHandle}
                }));
        ResponseTtlvTestSupport.assertReleased(missingIdentifierHandle);

        long malformedIdentifierHandle = NativeExtensionRegistry.ttlvStructureCreate();
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseObtainLeaseResponse(new Object[] {
                    new long[] {1, 0, -7}, new Object[] {new long[] {1}, null},
                    new long[] {malformedIdentifierHandle}
                }));
        ResponseTtlvTestSupport.assertReleased(malformedIdentifierHandle);
    }

    private static TtlvStructure responseTtlv() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure payload = TtlvStructure.create();
        payload = append(payload, 0x0042_0094,
                TtlvValue.TextString("leased-object".getBytes(StandardCharsets.UTF_8), limits), limits);
        payload = append(payload, 0x0042_0049, TtlvValue.Interval(0), limits);
        payload = append(payload, 0x0042_0048, TtlvValue.DateTime(-7), limits);
        payload = append(payload, 0x0054_1234, TtlvValue.Integer(-7), limits);
        return payload;
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (Tag tag = RawTag.fromRaw(rawTag).checked();
                TtlvItem item = TtlvItem.create(tag, value, limits)) {
            return TtlvStructure.withItem(structure, item, limits);
        }
    }

    private static long tagAt(TtlvStructureView view, long index) {
        try (var item = view.itemAt(index); Tag tag = item.tag()) {
            return tag.raw();
        }
    }
}
