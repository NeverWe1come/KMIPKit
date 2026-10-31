package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class AddAttributeJavaFacadeTest {
    @Test
    void successfulResponseCopiesTheRequiredIdentifierWithoutFormattingIt() {
        ClientBatchItemResult.AddAttributeResponse response = ClientBatchItemResult
                .parseAddAttributeResponse(new Object[] {new long[] {1}, "object-7"})
                .orElseThrow();

        assertEquals("object-7", response.uniqueIdentifier());
        assertFalse(response.toString().contains("object-7"));
    }

    @Test
    void nonSuccessHasNoTypedAddAttributeResponse() {
        assertEquals(
                java.util.Optional.empty(),
                ClientBatchItemResult.parseAddAttributeResponse(
                        new Object[] {new long[] {0}, null}));
    }

    @Test
    void malformedSuccessfulResponseRejectsMissingOrNonTextIdentifier() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseAddAttributeResponse(
                        new Object[] {new long[] {1}, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseAddAttributeResponse(
                        new Object[] {new long[] {1}, new byte[] {0x01}}));
    }
}
