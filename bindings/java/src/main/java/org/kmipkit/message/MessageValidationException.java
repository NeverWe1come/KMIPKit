package org.kmipkit.message;

import java.util.OptionalLong;

import org.kmipkit.KmipException;

/** A redacted structural validation failure with stable numeric metadata only. */
public final class MessageValidationException extends KmipException {
    private static final long serialVersionUID = 1L;

    private final long kindCode;
    private final OptionalLong structureIndex;
    private final OptionalLong fieldIndex;

    MessageValidationException(long kindCode, long structureIndex, long fieldIndex) {
        super("KMIP message structure is invalid; payload details are redacted");
        this.kindCode = kindCode;
        this.structureIndex = optionalIndex(structureIndex);
        this.fieldIndex = optionalIndex(fieldIndex);
    }

    /** Returns the raw forward-compatible validation kind code. */
    public long kindCode() {
        return kindCode;
    }

    /** Returns the top-level Structure item index when the validator provides one. */
    public OptionalLong structureIndex() {
        return structureIndex;
    }

    /** Returns the field index within a Structure when the validator provides one. */
    public OptionalLong fieldIndex() {
        return fieldIndex;
    }

    private static OptionalLong optionalIndex(long value) {
        return value < 0 ? OptionalLong.empty() : OptionalLong.of(value);
    }
}
