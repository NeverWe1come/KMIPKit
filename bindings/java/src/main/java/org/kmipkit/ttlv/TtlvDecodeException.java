package org.kmipkit.ttlv;

import org.kmipkit.KmipException;

/** A redacted generic TTLV decoding error with a stable category and byte offset. */
public final class TtlvDecodeException extends KmipException {
    private static final long serialVersionUID = 1L;

    private final int kind;
    private final long byteOffset;

    TtlvDecodeException(int kind, long byteOffset) {
        super("KMIPKit TTLV decoding failed (kind=" + kind + ", byteOffset=" + byteOffset + ")");
        this.kind = kind;
        this.byteOffset = byteOffset;
    }

    /** Returns the stable decoder category code, preserving unknown future values. */
    public int kind() {
        return kind;
    }

    /** Returns the byte offset at which decoding detected the error. */
    public long byteOffset() {
        return byteOffset;
    }
}
