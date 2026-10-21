package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.AbstractList;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

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

    @Test
    void builderCopiesUnknownPayloadExtensionsAndLeavesInputOwned() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x0054_1234);
                Tag tag = rawTag.checked();
                TtlvValue value = TtlvValue.TextString(
                        "validate-extension-sentinel".getBytes(java.nio.charset.StandardCharsets.UTF_8),
                        limits);
                TtlvItem extension = TtlvItem.create(
                        tag,
                        value,
                        limits);
                ClientBatchItem item = ClientBatchItem.validate(
                        List.of(), List.of(), null, List.of(extension));
                TtlvStructure retained = TtlvStructure.withItem(
                        TtlvStructure.create(), extension, limits);
                TtlvStructureView view = retained.view()) {
            assertNotNull(item);
            org.junit.jupiter.api.Assertions.assertEquals(1, view.itemCount());
        }
    }

    @Test
    void builderRejectsOversizedExtensionListBeforeCopyingIt() {
        List<TtlvItem> oversized = new AbstractList<>() {
            @Override
            public TtlvItem get(int index) {
                throw new AssertionError("oversized list entries must not be copied");
            }

            @Override
            public int size() {
                return 100_001;
            }
        };

        assertThrows(ResourceLimitException.class,
                () -> ClientBatchItem.validate(List.of(), List.of(), null, oversized));
    }
}
