package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.ResourceLimitException;

final class TtlvCodecStatusMappingTest {
    @Test
    void statusOnlyResourceLimitMapsFromDecoderResult() {
        assertThrows(ResourceLimitException.class,
                () -> TtlvCodec.decodeResult(new long[] {0, 0, 0, 6}));
    }
}
