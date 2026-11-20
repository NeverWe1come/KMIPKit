package org.kmipkit.validator;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

class TtlvModelChecksTest {
    @Test
    void preservesUnknownEnumerationValuesAndBitmaskBits() {
        assertEquals(0xf0000001L, TtlvModelChecks.constructAndReadEnumeration(0xf0000001L));
        assertEquals(0x80000001, TtlvModelChecks.constructAndReadIntegerMask(0x80000001));
        assertEquals(0xf0000001L, TtlvModelChecks.decodeEnumeration(0xf0000001L));
        assertEquals(0x80000001, TtlvModelChecks.decodeIntegerMask(0x80000001));
    }

    @Test
    void preservesGenericStructureItemOrder() {
        assertArrayEquals(new int[] {0x420001, 0x420002}, TtlvModelChecks.constructAndReadTagOrder());
    }
}
