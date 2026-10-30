package org.kmipkit.message;

import java.util.Optional;
import java.util.OptionalLong;

import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValueView;

final class MessageTtlvAccess {
    private MessageTtlvAccess() { }

    static OptionalLong enumeration(TtlvStructure structure, int rawTag) {
        return find(structure, rawTag, value -> OptionalLong.of(value.enumeration()));
    }

    static Optional<Boolean> booleanValue(TtlvStructure structure, int rawTag) {
        OptionalLong found = find(structure, rawTag,
                value -> OptionalLong.of(value.booleanValue() ? 1 : 0));
        return found.isPresent() ? Optional.of(found.getAsLong() != 0) : Optional.empty();
    }

    static OptionalLong dateTime(TtlvStructure structure, int rawTag) {
        return find(structure, rawTag, value -> OptionalLong.of(value.dateTime()));
    }

    private static OptionalLong find(
            TtlvStructure structure, int rawTag, ValueReader reader) {
        try (TtlvStructureView view = structure.view()) {
            long count = view.itemCount();
            for (long index = 0; index < count; index++) {
                try (TtlvItemView item = view.itemAt(index);
                        var tag = item.tag()) {
                    if (tag.raw() == rawTag) {
                        try (TtlvValueView value = item.value()) {
                            return reader.read(value);
                        }
                    }
                }
            }
        }
        return OptionalLong.empty();
    }

    @FunctionalInterface
    private interface ValueReader {
        OptionalLong read(TtlvValueView value);
    }
}
