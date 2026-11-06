package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class AsyncOperationMetadataV1Test {
    @Test
    void preservesUnsignedMetadataAndRedactsMessageFromFormatting() {
        long[] values = {0x1a, 2, 2, 1, 0xdead_beefL, 1, 0xf000_0001L, 1};
        byte[] message = "server-marker".getBytes(java.nio.charset.StandardCharsets.UTF_8);

        AsyncOperationMetadataV1 metadata = new AsyncOperationMetadataV1(values, message);
        values[4] = 0;
        message[0] = 0;

        assertEquals(0x1a, metadata.operation());
        assertTrue(metadata.outcomeKind().isPending());
        assertEquals(2, metadata.resultStatus());
        assertEquals(0xdead_beefL, metadata.resultReason().orElseThrow());
        assertEquals(0xf000_0001L, metadata.cancellationResult().orElseThrow());
        assertEquals("server-marker", metadata.resultMessage().orElseThrow());
        assertFalse(metadata.toString().contains("server-marker"));
    }

    @Test
    void rejectsInconsistentOptionalMessagePresence() {
        long[] values = {0x1a, 1, 0, 0, 0, 0, 0, 0};
        assertThrows(InvalidInputException.class,
                () -> new AsyncOperationMetadataV1(values, new byte[0]));
    }
}
