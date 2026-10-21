package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.util.Objects;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;

/** Exact KMIP Unique Identifier wire value, including its TTLV type. */
public final class UniqueIdentifier {
    static final int MAX_NATIVE_TEXT_BYTES = 16 * 1024 * 1024;

    public enum Kind {
        TEXT_STRING(1),
        ENUMERATION(2),
        INTEGER(3);

        private final int raw;

        Kind(int raw) {
            this.raw = raw;
        }

        int raw() {
            return raw;
        }

        static Kind fromRaw(long raw) {
            for (Kind kind : values()) {
                if (kind.raw == raw) {
                    return kind;
                }
            }
            throw new InvalidInputException("native Unique Identifier kind is invalid");
        }
    }

    private final Kind kind;
    private final String text;
    private final long number;

    private UniqueIdentifier(Kind kind, String text, long number) {
        this.kind = kind;
        this.text = text;
        this.number = number;
    }

    public static UniqueIdentifier textString(String value) {
        Objects.requireNonNull(value, "value");
        if (!StandardCharsets.UTF_8.newEncoder().canEncode(value)) {
            throw new InvalidInputException("Unique Identifier is not valid UTF-8 text");
        }
        return new UniqueIdentifier(Kind.TEXT_STRING, value, 0);
    }

    public static UniqueIdentifier enumeration(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("Unique Identifier enumeration is outside uint32");
        }
        return new UniqueIdentifier(Kind.ENUMERATION, null, value);
    }

    public static UniqueIdentifier integer(int value) {
        return new UniqueIdentifier(Kind.INTEGER, null, value);
    }

    /** Returns this identifier's exact KMIP TTLV wire type. */
    public Kind kind() {
        return kind;
    }

    /** Returns the Text String value when this identifier uses that wire type. */
    public String text() {
        if (kind != Kind.TEXT_STRING) {
            throw new InvalidInputException("Unique Identifier is not a Text String");
        }
        return text;
    }

    /** Returns the unsigned Enumeration value or signed Integer value. */
    public long number() {
        if (kind == Kind.TEXT_STRING) {
            throw new InvalidInputException("Unique Identifier is a Text String");
        }
        return number;
    }

    int nativeKind() {
        return kind.raw();
    }

    byte[] nativeText() {
        if (kind != Kind.TEXT_STRING) {
            return null;
        }
        return encodeNativeText(text);
    }

    static byte[] encodeNativeText(String text) {
        checkedUtf8Length(text, MAX_NATIVE_TEXT_BYTES, "Unique Identifier");
        return text.getBytes(StandardCharsets.UTF_8);
    }

    static int checkedUtf8Length(String value, int maximumBytes, String inputName) {
        long byteCount = 0;
        for (int index = 0; index < value.length(); index++) {
            char character = value.charAt(index);
            if (character <= 0x7f) {
                byteCount++;
            } else if (character <= 0x7ff) {
                byteCount += 2;
            } else if (Character.isHighSurrogate(character)) {
                if (index + 1 >= value.length()
                        || !Character.isLowSurrogate(value.charAt(index + 1))) {
                    throw new InvalidInputException(inputName + " is not valid Unicode");
                }
                byteCount += 4;
                index++;
            } else if (Character.isLowSurrogate(character)) {
                throw new InvalidInputException(inputName + " is not valid Unicode");
            } else {
                byteCount += 3;
            }
            if (byteCount > maximumBytes) {
                throw new ResourceLimitException(inputName + " exceeds the UTF-8 byte limit");
            }
        }
        return (int) byteCount;
    }

    long nativeNumber() {
        return number;
    }

    static UniqueIdentifier fromNative(long kind, byte[] text, long enumeration, long integer) {
        Kind valueKind = Kind.fromRaw(kind);
        return switch (valueKind) {
            case TEXT_STRING -> textString(decodeUtf8(Objects.requireNonNull(text)));
            case ENUMERATION -> enumeration(enumeration);
            case INTEGER -> integer(Math.toIntExact(integer));
        };
    }

    private static String decodeUtf8(byte[] bytes) {
        try {
            return StandardCharsets.UTF_8.newDecoder()
                    .onMalformedInput(CodingErrorAction.REPORT)
                    .onUnmappableCharacter(CodingErrorAction.REPORT)
                    .decode(ByteBuffer.wrap(bytes)).toString();
        } catch (CharacterCodingException exception) {
            throw new InvalidInputException("native Unique Identifier is not valid UTF-8 text");
        }
    }

    @Override
    public String toString() {
        return "UniqueIdentifier[" + kind + ", redacted]";
    }
}
