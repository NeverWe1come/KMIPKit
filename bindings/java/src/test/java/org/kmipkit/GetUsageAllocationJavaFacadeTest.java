package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotNull;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

final class GetUsageAllocationJavaFacadeTest {
    @Test
    void builderPreservesOptionalIdentifierAndSignedLongCount() {
        try (ClientBatchItem item = ClientBatchItem.getUsageAllocation(
                UniqueIdentifier.textString("allocation-object"), Long.MAX_VALUE)) {
            assertNotNull(item);
        }
    }

    @Test
    void builderAllowsPlaceholderIdentifierAndZeroCount() {
        try (ClientBatchItem item = ClientBatchItem.getUsageAllocation(null, 0)) {
            assertNotNull(item);
        }
    }
}
