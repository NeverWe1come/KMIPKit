package org.kmipkit.message;

import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValueView;

final class MessageTtlvAccess {
    private MessageTtlvAccess() { }

    static OptionalLong enumeration(TtlvStructure structure, int rawTag) {
        Optional<Long> found = find(structure, rawTag,
                value -> value.itemType() == TtlvItemType.Enumeration ? value.enumeration() : null);
        return found.isPresent() ? OptionalLong.of(found.get()) : OptionalLong.empty();
    }

    static Optional<Boolean> booleanValue(TtlvStructure structure, int rawTag) {
        return find(structure, rawTag, value -> value.itemType() == TtlvItemType.Boolean
                ? value.booleanValue() : null);
    }

    static OptionalInt integer(TtlvStructure structure, int rawTag) {
        Optional<Integer> found = find(structure, rawTag,
                value -> value.itemType() == TtlvItemType.Integer ? value.integer() : null);
        return found.isPresent() ? OptionalInt.of(found.get()) : OptionalInt.empty();
    }

    static OptionalLong dateTime(TtlvStructure structure, int rawTag) {
        Optional<Long> found = find(structure, rawTag,
                value -> value.itemType() == TtlvItemType.DateTime ? value.dateTime() : null);
        return found.isPresent() ? OptionalLong.of(found.get()) : OptionalLong.empty();
    }

    static Optional<String> textString(TtlvStructure structure, int rawTag) {
        return find(structure, rawTag, value -> value.itemType() == TtlvItemType.TextString
                ? decodeText(copyBytes(value)) : null);
    }

    static Optional<byte[]> byteString(TtlvStructure structure, int rawTag) {
        return find(structure, rawTag, value -> value.itemType() == TtlvItemType.ByteString
                ? copyBytes(value) : null);
    }

    static List<Long> repeatedEnumerations(TtlvStructure structure, int rawTag) {
        List<Long> values = new ArrayList<>();
        try (TtlvStructureView view = structure.view()) {
            for (long index = 0, count = view.itemCount(); index < count; index++) {
                try (TtlvItemView item = view.itemAt(index);
                        var tag = item.tag()) {
                    if (tag.raw() != rawTag) {
                        continue;
                    }
                    try (TtlvValueView value = item.value()) {
                        if (value.itemType() == TtlvItemType.Enumeration) {
                            values.add(value.enumeration());
                        }
                    }
                }
            }
        }
        return List.copyOf(values);
    }

    /** Invokes a callback with a temporary, read-only Structure view. */
    static <T> Optional<T> withStructure(
            TtlvStructure structure, int rawTag, Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("KMIP message callback is invalid");
        }
        return find(structure, rawTag, value -> {
            if (value.itemType() != TtlvItemType.Structure) {
                return null;
            }
            try (TtlvStructureView nested = value.structure()) {
                return callback.apply(nested);
            }
        });
    }

    /** Invokes a callback with a byte copy that KMIPKit clears when the callback returns. */
    static <T> Optional<T> withByteString(
            TtlvStructure structure, int rawTag, Function<byte[], T> callback) {
        if (callback == null) {
            throw new InvalidInputException("KMIP message callback is invalid");
        }
        return find(structure, rawTag, value -> {
            if (value.itemType() != TtlvItemType.ByteString) {
                return null;
            }
            byte[] copy = copyBytes(value);
            try {
                return callback.apply(copy);
            } finally {
                Arrays.fill(copy, (byte) 0);
            }
        });
    }

    private static <T> Optional<T> find(
            TtlvStructure structure, int rawTag, Function<TtlvValueView, T> reader) {
        try (TtlvStructureView view = structure.view()) {
            for (long index = 0, count = view.itemCount(); index < count; index++) {
                try (TtlvItemView item = view.itemAt(index);
                        var tag = item.tag()) {
                    if (tag.raw() == rawTag) {
                        try (TtlvValueView value = item.value()) {
                            return Optional.ofNullable(reader.apply(value));
                        }
                    }
                }
            }
        }
        return Optional.empty();
    }

    private static byte[] copyBytes(TtlvValueView value) {
        long length = value.byteLength();
        if (length < 0 || length > Integer.MAX_VALUE) {
            throw new ResourceLimitException("KMIP message field exceeds Java array limits");
        }
        byte[] bytes = new byte[(int) length];
        for (int index = 0; index < bytes.length; index++) {
            bytes[index] = (byte) value.byteAt(index);
        }
        return bytes;
    }

    private static String decodeText(byte[] bytes) {
        try {
            return StandardCharsets.UTF_8.newDecoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .decode(ByteBuffer.wrap(bytes))
                    .toString();
        } catch (CharacterCodingException exception) {
            throw new InvalidInputException("KMIP message text field is not valid UTF-8");
        } finally {
            Arrays.fill(bytes, (byte) 0);
        }
    }
}
