package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

final class DeriveKeyMappingTest {
    @Test
    void requestBuilderAcceptsOrderedRepeatedIdentifiersAndUnknownMethod() {
        try (TtlvStructure derivationParameters = TtlvStructure.create();
             TtlvStructure attributes = TtlvStructure.create();
             ClientBatchItem item = ClientBatchItem.deriveKey(
                     2,
                     List.of(
                             UniqueIdentifier.textString("source-a"),
                             UniqueIdentifier.enumeration(0xfeed_beefL),
                             UniqueIdentifier.textString("source-a")),
                     0xfeed_beefL,
                     derivationParameters,
                     attributes)) {
            assertTrue(item != null);
        }
    }

    @Test
    void requestBuilderRejectsMissingSourceIdentifiers() {
        try (TtlvStructure derivationParameters = TtlvStructure.create();
             TtlvStructure attributes = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.deriveKey(
                    2, List.of(), 1, derivationParameters, attributes));
        }
    }
}
