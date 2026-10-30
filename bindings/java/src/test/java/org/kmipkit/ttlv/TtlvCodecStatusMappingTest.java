package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertInstanceOf;

import org.junit.jupiter.api.Test;
import org.kmipkit.ResourceLimitException;

final class TtlvCodecStatusMappingTest {
    @Test
    void statusOnlyResourceLimitMapsToResourceLimitException() {
        assertInstanceOf(ResourceLimitException.class, TtlvCodec.statusFailure(6));
    }
}
