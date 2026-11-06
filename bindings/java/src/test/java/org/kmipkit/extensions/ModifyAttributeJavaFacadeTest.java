package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

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
    void allowsOmittedIdentifierAndCurrentAttribute() {
        CodecLimits limits = CodecLimits.defaults();
        try (Tag tag = RawTag.fromRaw(0x0042_00a5).checked();
                TtlvItem newAttribute = TtlvItem.create(
                        tag, TtlvValue.TextString(new byte[] {'n'}, limits), limits);
                ClientBatchItem item = ClientBatchItem.modifyAttribute(null, null, newAttribute)) {
            assertNotNull(item);
            assertNotEquals(0, newAttribute.nativeHandleOwner().get(),
                    "the caller retains its New Attribute Item");
        }
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
                        new Object[] {new long[] {0}, "unexpected-object"}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseModifyAttributeResponse(
                        new Object[] {new long[] {1}, null}));
    }
}
