package org.kmipkit.ttlv;

/** Package-private JNI boundary for the public TTLV decoder facade. */
final class NativeTtlvCodec {
    private NativeTtlvCodec() {}

    static native long[] decode(byte[] wire, long[] limits);

    static native long itemViewFromItem(long item);
}
