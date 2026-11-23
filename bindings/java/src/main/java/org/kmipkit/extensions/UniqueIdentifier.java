package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.util.Objects;

import org.kmipkit.InvalidInputException;

/** Exact KMIP Unique Identifier wire value, including its TTLV type. */
public final class UniqueIdentifier {
    static final int MAX_NATIVE_TEXT_BYTES = 16 * 1024 * 1024;

    public enum Kind {
        TEXT_STRING(1),
        ENUMERATION(2),
        INTEGER(3),
        IDENTIFIER(4),
        REFERENCE(5),
        NAME_REFERENCE(6);

        private final int raw;

        Kind(int raw) {
            this.raw = raw;
        }

        public boolean isTextual() {
            return this == TEXT_STRING || this == IDENTIFIER || this == REFERENCE || this == NAME_REFERENCE;
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
        return textual(Kind.TEXT_STRING, value);
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

    public static UniqueIdentifier identifier(String value) {
        return textual(Kind.IDENTIFIER, value);
    }

    public static UniqueIdentifier reference(String value) {
        return textual(Kind.REFERENCE, value);
    }

    public static UniqueIdentifier nameReference(String value) {
        return textual(Kind.NAME_REFERENCE, value);
    }

    private static UniqueIdentifier textual(Kind kind, String value) {
        Objects.requireNonNull(value, "value");
        if (!StandardCharsets.UTF_8.newEncoder().canEncode(value)) {
            throw new InvalidInputException("Unique Identifier is not valid UTF-8 text");
        }
        return new UniqueIdentifier(kind, value, 0);
    }

    /** Returns this identifier's exact KMIP TTLV wire type. */
    public Kind kind() {
        return kind;
    }

    /** Returns the Text String or Identifier value when this identifier uses that wire type. */
    public String text() {
        if (!kind.isTextual()) {
            throw new InvalidInputException("Unique Identifier is not a Text String or Identifier");
        }
        return text;
    }

    /** Returns the unsigned Enumeration value or signed Integer value. */
    public long number() {
        if (kind.isTextual()) {
            throw new InvalidInputException("Unique Identifier is a Text String or Identifier");
        }
        return number;
    }

    public int nativeKind() {
        return kind.raw();
    }

    public byte[] nativeText() {
        if (!kind.isTextual()) {
            return null;
        }
        return ExtensionText.encodeUtf8(text, MAX_NATIVE_TEXT_BYTES, "Unique Identifier");
    }

    public long nativeNumber() {
        return number;
    }

    static UniqueIdentifier fromNative(long kind, byte[] text, long enumeration, long integer) {
        Kind valueKind = Kind.fromRaw(kind);
        return switch (valueKind) {
            case TEXT_STRING -> textString(decodeUtf8(text));
            case ENUMERATION -> enumeration(enumeration);
            case INTEGER -> integer(Math.toIntExact(integer));
            case IDENTIFIER, REFERENCE, NAME_REFERENCE -> textual(valueKind, decodeUtf8(text));
        };
    }

    private static String decodeUtf8(byte[] bytes) {
        if (bytes == null) {
            throw new InvalidInputException("native Unique Identifier bytes are missing");
        }
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
