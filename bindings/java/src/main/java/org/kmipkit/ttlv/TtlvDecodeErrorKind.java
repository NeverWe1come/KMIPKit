package org.kmipkit.ttlv;

/** Stable numeric categories returned by the bounded generic TTLV decoder. */
public final class TtlvDecodeErrorKind {
    public static final int MESSAGE_TOO_LARGE = 1;
    public static final int TRUNCATED_HEADER = 2;
    public static final int TRUNCATED_VALUE = 3;
    public static final int INVALID_ITEM_LENGTH = 4;
    public static final int EMPTY_BIG_INTEGER = 5;
    public static final int INVALID_UTF8 = 6;
    public static final int INVALID_BOOLEAN = 7;
    public static final int UNSUPPORTED_ITEM_TYPE = 8;
    public static final int TRAILING_BYTES = 9;
    public static final int STRUCTURE_BOUNDARY = 10;
    public static final int RESERVED_TAG = 11;
    public static final int UNALLOCATED_TAG = 12;
    public static final int INVALID_PADDING_EXTENT = 13;
    public static final int STRUCTURE_DEPTH_EXCEEDED = 14;
    public static final int ELEMENT_LIMIT_EXCEEDED = 15;
    public static final int ALLOCATION_FAILED = 16;
    public static final int MODEL_CONSTRAINT = 17;

    private TtlvDecodeErrorKind() {}
}
