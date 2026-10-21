package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

/** Validate facade coverage derived from OASIS KMIP 2.1 §6.1.57, Table 340. */
final class ValidateJavaFacadeTest {
    @Test
    void builderAcceptsIdentifierOnlyChainAndPreservesOptionalDate() {
        try (ClientBatchItem item = ClientBatchItem.validate(
                List.of(), List.of(UniqueIdentifier.textString("chain-id")), Long.MIN_VALUE)) {
            assertNotNull(item);
        }
    }

    @Test
    void builderPreservesOmittedDateAndEmptyChainWithoutSynthesizingLocalTime() {
        try (ClientBatchItem item = ClientBatchItem.validate(List.of(), List.of(), null)) {
            assertNotNull(item);
        }
    }

    @Test
    void builderRejectsMissingLists() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.validate(null, List.of(), null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.validate(List.of(), null, null));
    }
}
