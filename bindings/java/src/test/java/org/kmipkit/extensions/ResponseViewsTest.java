package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.*;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.CodecLimits;

final class ResponseViewsTest {
    @Test
    void renderersRequireLiveResponseAndExplicitLimits() {
        NativeExtensionRegistry.ensureLoaded();
        long wrong = NativeExtensionRegistry.rawTagCreate(0x420001);
        try (ClientBatchItemResult result = new ClientBatchItemResult(wrong, 100_000, 16_777_216)) {
            assertThrows(InvalidInputException.class, () -> result.ttlvHex(CodecLimits.defaults()));
            assertThrows(InvalidInputException.class, () -> result.json(CodecLimits.defaults()));
            assertThrows(InvalidInputException.class, () -> result.xml(CodecLimits.defaults()));
            assertThrows(InvalidInputException.class, () -> result.json(null));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.RAW_TAG, wrong);
        }
    }
}
