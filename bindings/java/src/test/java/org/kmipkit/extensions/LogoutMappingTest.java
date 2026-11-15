package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Optional;
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

final class LogoutMappingTest {
    @Test
    void requestBuilderMapsRequiredTicketAndKeepsCallerStructureUsable() {
        try (TtlvStructure ticket = ticket();
             TtlvStructureView view = ticket.view();
             ClientBatchItem item = ClientBatchItem.logout(ticket)) {
            assertEquals(2, view.itemCount());
            assertTrue(item != null);
        }
    }

    @Test
    void requestBuilderRejectsMissingTicket() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.logout(null));
    }

    @Test
    void responseMappingPreservesOptionalPresence() {
        assertEquals(Optional.of(Boolean.TRUE),
                ClientBatchItemResult.parseLogoutResponse(new long[] {1}));
        assertTrue(ClientBatchItemResult.parseLogoutResponse(new long[] {0}).isEmpty());
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseLogoutResponse(new long[] {2}));
    }

    @Test
    void nativeResponseBridgeRejectsAnInvalidItemHandle() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseLogout(0));
    }

    private static TtlvStructure ticket() {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure empty = TtlvStructure.create();
        try (empty;
             RawTag typeRawTag = RawTag.fromRaw(0x42014a);
             Tag typeTag = typeRawTag.checked();
             TtlvValue type = TtlvValue.Enumeration(1);
             TtlvItem typeItem = TtlvItem.create(typeTag, type, limits)) {
            TtlvStructure withType = TtlvStructure.withItem(empty, typeItem, limits);
            try (withType;
                 RawTag valueRawTag = RawTag.fromRaw(0x42014b);
                 Tag valueTag = valueRawTag.checked();
                 TtlvValue value = TtlvValue.ByteString(new byte[] {1, 2, 3}, limits);
                 TtlvItem valueItem = TtlvItem.create(valueTag, value, limits)) {
                return TtlvStructure.withItem(withType, valueItem, limits);
            }
        }
    }
}
