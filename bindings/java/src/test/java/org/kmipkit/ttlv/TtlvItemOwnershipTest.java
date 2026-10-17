package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class TtlvItemOwnershipTest {
    private static final int PRIVATE_TAG = 0x420001;
    private static final CodecLimits LIMITS = CodecLimits.defaults();

    @Test
    void nullTagDoesNotConsumeTheValue() {
        try (TtlvValue value = TtlvValue.Integer(29)) {
            assertThrows(InvalidInputException.class, () -> TtlvItem.create(null, value, LIMITS));

            assertEquals(29, readInteger(value));
        }
    }

    @Test
    void closedTagDoesNotConsumeTheValue() {
        try (RawTag rawTag = RawTag.fromRaw(PRIVATE_TAG);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.Integer(31)) {
            tag.close();

            assertThrows(InvalidInputException.class, () -> TtlvItem.create(tag, value, LIMITS));

            assertEquals(31, readInteger(value));
        }
    }

    private static int readInteger(TtlvValue value) {
        try (TtlvValueView view = value.view()) {
            return view.integer();
        }
    }
}
