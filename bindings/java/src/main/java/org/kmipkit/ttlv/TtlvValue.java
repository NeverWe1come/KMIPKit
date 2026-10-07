package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;

/** An owned generic TTLV value. Byte inputs are copied by the native adapter. */
public final class TtlvValue {
    private final NativeHandle handle;

    private TtlvValue(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_VALUE);
    }

    public static TtlvValue Integer(int value) {
        return new TtlvValue(NativeExtensionRegistry.ttlvValueInteger(value));
    }

    public static TtlvValue LongInteger(long value) {
        return new TtlvValue(NativeExtensionRegistry.ttlvValueLongInteger(value));
    }

    public static TtlvValue Enumeration(long value) {
        return new TtlvValue(NativeExtensionRegistry.ttlvValueEnumeration(value));
    }

    public static TtlvValue Boolean(boolean value) {
        return new TtlvValue(NativeExtensionRegistry.ttlvValueBoolean(value));
    }

    public static TtlvValue DateTime(long value) {
        return new TtlvValue(NativeExtensionRegistry.ttlvValueDateTime(value));
    }

    public static TtlvValue Interval(long value) {
        return new TtlvValue(NativeExtensionRegistry.ttlvValueInterval(value));
    }

    public static TtlvValue DateTimeExtended(long value) {
        return new TtlvValue(NativeExtensionRegistry.ttlvValueDateTimeExtended(value));
    }

    public static TtlvValue BigInteger(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        return new TtlvValue(NativeExtensionRegistry.ttlvValueBigInteger(
                value.clone(), NativeExtensionRegistry.codecLimitValues(limits)));
    }

    public static TtlvValue TextString(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        return new TtlvValue(NativeExtensionRegistry.ttlvValueTextString(
                value.clone(), NativeExtensionRegistry.codecLimitValues(limits)));
    }

    public static TtlvValue ByteString(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        return new TtlvValue(NativeExtensionRegistry.ttlvValueByteString(
                value.clone(), NativeExtensionRegistry.codecLimitValues(limits)));
    }

    private static void requireWithinMessageLimit(byte[] value, CodecLimits limits) {
        long maximum = limits.maxMessageBytes();
        if (value != null && value.length > maximum) {
            throw new ResourceLimitException("byte input exceeds configured limits");
        }
    }

    public static TtlvValue structure(TtlvStructure structure, CodecLimits limits) {
        NativeExtensionRegistry.ensureLoaded();
        long[] limitValues = NativeExtensionRegistry.codecLimitValues(limits);
        long structureHandle = structure.nativeHandleOwner().transfer();
        return new TtlvValue(NativeExtensionRegistry.ttlvValueStructure(
                structureHandle, limitValues));
    }

    public TtlvValueView view() {
        return handle.withValue(value -> TtlvValueView.fromNative(
                NativeExtensionRegistry.ttlvValueView(value), this));
    }

    public long handle() {
        return handle.get();
    }

    /** Internal owner token used to atomically transfer this value to a consuming facade call. */
    public NativeHandle nativeHandleOwner() {
        return handle;
    }
}
