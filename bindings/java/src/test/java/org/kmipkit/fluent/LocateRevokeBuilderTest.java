package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

final class LocateRevokeBuilderTest {
    @Test
    void locateBuildsTypedItemWithExplicitFilters() {
        try (TtlvStructure empty = TtlvStructure.create();
                AttributeSet attributes = AttributeSet.tryNew(empty)) {
            LocateBuilder builder = new LocateBuilder(null, attributes)
                    .maximumItems(12)
                    .offsetItems(3)
                    .storageStatusMask(2)
                    .objectGroupMember(1)
                    .asynchronousIndicator(5);
            builder.validate();
            try (ClientBatchItem item = builder.toBatchItem()) {
                assertEquals(8, item.operation());
            }
        }
    }

    @Test
    void revokeBuildsTypedItemAndRejectsDateForOtherReasonCodes() {
        RevokeBuilder builder = new RevokeBuilder(null, 2, "key material compromised")
                .uniqueIdentifier(UniqueIdentifier.textString("lifecycle-key"))
                .compromiseOccurrenceDate(1_700_000_000L)
                .asynchronousIndicator(7);
        builder.validate();
        try (ClientBatchItem item = builder.toBatchItem()) {
            assertEquals(23, item.operation());
        }
        RevokeBuilder invalid = new RevokeBuilder(null, 1, null)
                .compromiseOccurrenceDate(1_700_000_000L);
        assertThrows(InvalidInputException.class, invalid::build);
    }
}
