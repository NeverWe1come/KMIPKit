package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class ModifyAttributeJavaFacadeTest {
    @Test
    void rejectsMissingNewAttributeAndNonTextIdentifier() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.modifyAttribute(null, null, null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.modifyAttribute(
                        UniqueIdentifier.integer(17), null, null));
    }

    @Test
    void successfulResponseCopiesIdentifierAndRedactsItsString() {
        ClientBatchItemResult.ModifyAttributeResponse response =
                ClientBatchItemResult.parseModifyAttributeResponse(
                        new Object[] {new long[] {1}, "modified-object-1"}).orElseThrow();

        assertEquals("modified-object-1", response.uniqueIdentifier());
        assertFalse(response.toString().contains("modified-object-1"));
    }

    @Test
    void nonSuccessHasNoTypedResponseAndMalformedSuccessIsRejected() {
        assertEquals(java.util.Optional.empty(),
                ClientBatchItemResult.parseModifyAttributeResponse(
                        new Object[] {new long[] {0}, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseModifyAttributeResponse(
                        new Object[] {new long[] {1}, null}));
    }
}
