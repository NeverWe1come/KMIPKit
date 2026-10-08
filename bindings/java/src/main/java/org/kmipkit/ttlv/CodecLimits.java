package org.kmipkit.ttlv;

import org.kmipkit.ResourceLimitException;

/** Immutable limits for generic TTLV construction and decoding. */
public final class CodecLimits {
    private final long maxMessageBytes;
    private final long maxStructureDepth;
    private final long maxElements;

    private CodecLimits(long maxMessageBytes, long maxStructureDepth, long maxElements) {
        this.maxMessageBytes = maxMessageBytes;
        this.maxStructureDepth = maxStructureDepth;
        this.maxElements = maxElements;
    }

    public static CodecLimits defaults() {
        return new CodecLimits(16 * 1024 * 1024L, 64, 100_000);
    }

    public static CodecLimits create(long maxMessageBytes, long maxStructureDepth, long maxElements) {
        if (maxMessageBytes < 0 || maxElements < 0 || maxStructureDepth < 0) {
            throw new org.kmipkit.InvalidInputException("codec limit cannot be negative");
        }
        if (maxStructureDepth > 64) {
            throw new ResourceLimitException("codec depth exceeds the generic model maximum");
        }
        return new CodecLimits(maxMessageBytes, maxStructureDepth, maxElements);
    }

    public long maxMessageBytes() { return maxMessageBytes; }
    public long maxStructureDepth() { return maxStructureDepth; }
    public long maxElements() { return maxElements; }

    long[] nativeValues() {
        return new long[] {maxMessageBytes, maxStructureDepth, maxElements};
    }
}
