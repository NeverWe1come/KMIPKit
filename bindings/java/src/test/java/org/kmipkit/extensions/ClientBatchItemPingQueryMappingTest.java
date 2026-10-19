package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Optional;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.TtlvStructure;

final class ClientBatchItemPingQueryMappingTest {
    @Test
    void pingFactoryCreatesAnOwnedBatchItem() {
        try (ClientBatchItem item = ClientBatchItem.ping()) {
            assertEquals(0, item.extensionCount());
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
    void queryFactoryRequiresAtLeastOneFunction() {
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.query(null, null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.query(new int[0], null));
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
}
