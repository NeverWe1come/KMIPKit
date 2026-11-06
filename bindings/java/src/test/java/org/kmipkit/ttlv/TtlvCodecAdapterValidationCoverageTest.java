package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.KmipException;
import org.kmipkit.ResourceLimitException;

final class TtlvCodecAdapterValidationCoverageTest {
    @Test
    void decodeFacadeAndNativeResultValidateEveryFailureCategory() {
        assertThrows(InvalidInputException.class, () -> TtlvCodec.decodeWithLimits(null, null));
        assertThrows(KmipException.class, () -> TtlvCodec.decodeResult(null));
        assertThrows(KmipException.class, () -> TtlvCodec.decodeResult(new long[3]));
        assertThrows(TtlvDecodeException.class,
                () -> TtlvCodec.decodeResult(new long[] {0, 2, 9, 0}));
        assertThrows(InvalidInputException.class,
                () -> TtlvCodec.decodeResult(new long[] {0, 0, 0, 4}));
        assertThrows(ResourceLimitException.class,
                () -> TtlvCodec.decodeResult(new long[] {0, 0, 0, 6}));
        assertThrows(KmipException.class,
                () -> TtlvCodec.decodeResult(new long[] {0, 0, 0, 99}));
    }
}
