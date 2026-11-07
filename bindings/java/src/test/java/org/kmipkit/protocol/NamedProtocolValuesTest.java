package org.kmipkit.protocol;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

import java.util.List;
import org.junit.jupiter.api.Test;

final class NamedProtocolValuesTest {
    // OASIS KMIP Specification v2.1 §§11.12 and 12.1; KMIPKIT-0047-FR-001–004.
    @Test
    void namedValuesRetainUnknownRawValuesAndBits() {
        assertEquals(3L, KmipValues.CryptographicAlgorithm.AES);
        assertEquals("AES", KmipValues.CryptographicAlgorithm.name(3L));
        assertNull(KmipValues.CryptographicAlgorithm.name(0xffff_ffffL));
        assertNull(KmipValues.CryptographicAlgorithm.name(0x1_0000_0003L));

        long raw = KmipValues.CryptographicUsageMask.ENCRYPT | 0x8000_0000L;
        assertEquals(KmipValues.CryptographicUsageMask.ENCRYPT,
                KmipValues.CryptographicUsageMask.knownBits(raw));
        assertEquals(0x8000_0000L, KmipValues.CryptographicUsageMask.unknownBits(raw));
        assertEquals(0x8000_0000L,
                KmipValues.CryptographicUsageMask.unknownBits(raw | 0x1_0000_0000L));
        assertEquals(List.of("Encrypt"), KmipValues.CryptographicUsageMask.names(raw));
    }
}
