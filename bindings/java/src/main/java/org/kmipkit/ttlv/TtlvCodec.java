package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.KmipException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;

/** Bounded decoder for one complete generic TTLV Item. */
public final class TtlvCodec {
    private static final int ERROR_INVALID_INPUT = 4;
    private static final int ERROR_RESOURCE_LIMIT = 6;

    private TtlvCodec() {}

    /** Decodes exactly one complete Item using the default 16 MiB, depth 64, and 100,000 Item limits. */
    public static TtlvItem decode(byte[] wire) {
        return decodeWithLimits(wire, CodecLimits.defaults());
    }

    /** Decodes exactly one complete Item using caller-selected limits. */
    public static TtlvItem decodeWithLimits(byte[] wire, CodecLimits limits) {
        if (wire == null || limits == null) {
            throw new InvalidInputException("TTLV decoder inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] result = NativeTtlvCodec.decode(wire, limits.nativeValues());
        if (result == null || result.length != 4) {
            throw new KmipException("KMIPKit TTLV decoding failed");
        }
        long handle = result[0];
        int kind = Math.toIntExact(result[1]);
        long offset = result[2];
        int status = Math.toIntExact(result[3]);
        if (kind != 0) {
            throw new TtlvDecodeException(kind, offset);
        }
        if (status != 0 || handle == 0) {
            throw statusFailure(status);
        }
        return TtlvItem.fromNative(handle);
    }

    static RuntimeException statusFailure(int status) {
        return switch (status) {
            case ERROR_INVALID_INPUT -> new InvalidInputException("TTLV decoder inputs are invalid");
            case ERROR_RESOURCE_LIMIT -> new ResourceLimitException(
                    "TTLV decoding exceeded a configured resource limit");
            default -> new KmipException("KMIPKit TTLV decoding failed");
        };
    }
}
