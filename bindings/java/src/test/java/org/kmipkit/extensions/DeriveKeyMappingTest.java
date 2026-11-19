package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.AbstractList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
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
                             UniqueIdentifier.textString("source-a"),
                             UniqueIdentifier.textString("é中😀")),
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

    @Test
    void requestBuilderRejectsNullMissingAndOversizedSourceIdentifierCollections() {
        try (TtlvStructure derivationParameters = TtlvStructure.create();
             TtlvStructure attributes = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.deriveKey(
                    2, null, 1, derivationParameters, attributes));
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.deriveKey(
                    2, Arrays.asList(UniqueIdentifier.integer(7), null), 1,
                    derivationParameters, attributes));
            assertThrows(ResourceLimitException.class, () -> ClientBatchItem.deriveKey(
                    2, Collections.nCopies(100_001, UniqueIdentifier.integer(7)), 1,
                    derivationParameters, attributes));
        }
    }

    @Test
    void requestBuilderRejectsSourceIdentifierListThatChangesDuringValidation() {
        List<UniqueIdentifier> changingList = new AbstractList<>() {
            private int sizeReads;

            @Override
            public UniqueIdentifier get(int index) {
                if (index != 0) {
                    throw new IndexOutOfBoundsException(index);
                }
                return UniqueIdentifier.integer(7);
            }

            @Override
            public int size() {
                return sizeReads++ == 0 ? 2 : 1;
            }
        };

        try (TtlvStructure derivationParameters = TtlvStructure.create();
             TtlvStructure attributes = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class, () -> ClientBatchItem.deriveKey(
                    2, changingList, 1, derivationParameters, attributes));
        }
    }

    @Test
    void requestBuilderRejectsTextIdentifiersOverTheInputByteLimit() {
        String oversizedText = "x".repeat(16 * 1024 * 1024 + 1);
        try (TtlvStructure derivationParameters = TtlvStructure.create();
             TtlvStructure attributes = TtlvStructure.create()) {
            assertThrows(ResourceLimitException.class, () -> ClientBatchItem.deriveKey(
                    2, List.of(UniqueIdentifier.textString(oversizedText)), 1,
                    derivationParameters, attributes));
        }
    }
}
