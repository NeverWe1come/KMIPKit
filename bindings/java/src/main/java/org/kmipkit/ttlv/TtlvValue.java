package org.kmipkit.ttlv;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;

import java.util.Arrays;
import java.util.function.Function;
import org.kmipkit.internal.NativeHandle;

/**
 * An owned generic TTLV value. KMIPKit clears its temporary Java byte-array
 * copies after native calls; copies internal to the JVM remain outside its control.
 */
public final class TtlvValue implements AutoCloseable {
    private final NativeHandle handle;

    private TtlvValue(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.TTLV_VALUE);
    }

    static TtlvValue fromNative(long nativeHandle) {
        return new TtlvValue(nativeHandle);
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
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        return withZeroizedCopy(value, copy -> new TtlvValue(
                NativeExtensionRegistry.ttlvValueBigInteger(copy, nativeLimits)));
    }

    public static TtlvValue TextString(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        return withZeroizedCopy(value, copy -> new TtlvValue(
                NativeExtensionRegistry.ttlvValueTextString(copy, nativeLimits)));
    }

    public static TtlvValue ByteString(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        return withZeroizedCopy(value, copy -> new TtlvValue(
                NativeExtensionRegistry.ttlvValueByteString(copy, nativeLimits)));
    }

    public static TtlvValue Identifier(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        return withZeroizedCopy(value, copy -> new TtlvValue(
                NativeExtensionRegistry.ttlvValueIdentifier(copy, nativeLimits)));
    }

    public static TtlvValue Identifier(String value, CodecLimits limits) {
        if (value == null) {
            throw new InvalidInputException("Identifier string is invalid");
        }
        return Identifier(value.getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
    }

    public static TtlvValue Reference(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        return withZeroizedCopy(value, copy -> new TtlvValue(
                NativeExtensionRegistry.ttlvValueReference(copy, nativeLimits)));
    }

    public static TtlvValue Reference(String value, CodecLimits limits) {
        if (value == null) {
            throw new InvalidInputException("Reference string is invalid");
        }
        return Reference(value.getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
    }

    public static TtlvValue NameReference(byte[] value, CodecLimits limits) {
        requireWithinMessageLimit(value, limits);
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        return withZeroizedCopy(value, copy -> new TtlvValue(
                NativeExtensionRegistry.ttlvValueNameReference(copy, nativeLimits)));
    }

    public static TtlvValue NameReference(String value, CodecLimits limits) {
        if (value == null) {
            throw new InvalidInputException("NameReference string is invalid");
        }
        return NameReference(value.getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
    }

    static <T> T withZeroizedCopy(byte[] value, Function<byte[], T> nativeCall) {
        byte[] copy = value.clone();
        try {
            return nativeCall.apply(copy);
        } finally {
            Arrays.fill(copy, (byte) 0);
        }
    }

    private static void requireWithinMessageLimit(byte[] value, CodecLimits limits) {
        if (value == null || limits == null) {
            throw new InvalidInputException("TTLV byte input is invalid");
        }
        long maximum = limits.maxMessageBytes();
        if (value.length > maximum) {
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
    @Override
    public void close() {
        handle.close();
    }
}
