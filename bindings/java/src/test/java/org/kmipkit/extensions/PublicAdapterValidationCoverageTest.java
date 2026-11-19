package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;

final class PublicAdapterValidationCoverageTest {
    @Test
    void clientAndAttributeFactoriesRejectInvalidPublicInputs() {
        assertThrows(InvalidInputException.class, () -> Client.create(null, null));
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.getAttributes(null, null));
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.getAttributes(
                null, Arrays.asList((AttributeReference) null)));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.getAttributeList(UniqueIdentifier.enumeration(1)));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.addAttribute((org.kmipkit.ttlv.TtlvItem) null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.adjustAttribute(null, null, 0, null));
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.adjustAttribute(
                null, AttributeReference.tag(0x0042_0001), -1, null));
        assertThrows(InvalidInputException.class, () -> ClientBatchItem.deleteAttribute(
                UniqueIdentifier.enumeration(1), null, null));
    }

    @Test
    void profileSelectionAndContextEnforceSnapshotBoundsAndRedactState() {
        assertThrows(InvalidInputException.class, () -> new ProfileValidationSelection(null));
        assertThrows(InvalidInputException.class,
                () -> new ProfileValidationSelection(Arrays.asList((String) null)));
        assertThrows(ResourceLimitException.class,
                () -> new ProfileValidationSelection(List.of("x".repeat(4_097))));
        assertThrows(ResourceLimitException.class,
                () -> new ProfileValidationSelection(Collections.nCopies(4_097, "profile")));
        assertEquals("ProfileValidationSelection[profileCount=1]",
                new ProfileValidationSelection(List.of("baseline_client")).toString());

        assertThrows(InvalidInputException.class, () -> new ProfileValidationContext(null));
        assertThrows(InvalidInputException.class,
                () -> new ProfileValidationContext().withObjectState(-1, new ProfileObjectState("id")));
        assertThrows(InvalidInputException.class,
                () -> new ProfileValidationContext.Entry(-1, new ProfileObjectState("id")));
        assertThrows(InvalidInputException.class, () -> new ProfileObjectState(null));
        assertThrows(ResourceLimitException.class,
                () -> new ProfileValidationContext(Collections.nCopies(100_001,
                        new ProfileValidationContext.Entry(0, new ProfileObjectState("id")))));

        ProfileObjectState state = new ProfileObjectState("sensitive-id");
        ProfileValidationContext.Entry entry = new ProfileValidationContext.Entry(3, state);
        assertTrue(state.toString().contains("[REDACTED]"));
        assertTrue(entry.toString().contains("[REDACTED]"));
        assertTrue(new ProfileValidationContext(List.of(entry)).toString().contains("objectStateCount=1"));
    }

    @Test
    void batchResultVersionMappingPreservesEmptyAndRejectsMalformedNativeArrays() {
        assertTrue(ClientBatchItemResult.parseSupportedVersions(null).isEmpty());
        assertEquals(List.of(), ClientBatchItemResult.parseSupportedVersions(new long[0]).orElseThrow());
        assertEquals(List.of(new ProtocolVersion(2, 1)),
                ClientBatchItemResult.parseSupportedVersions(new long[] {2, 1}).orElseThrow());
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseSupportedVersions(new long[] {2}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseSupportedVersions(
                        new long[] {(long) Integer.MAX_VALUE + 1, 1}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMetadata(new long[] {1, 2, 3}));
    }
}
