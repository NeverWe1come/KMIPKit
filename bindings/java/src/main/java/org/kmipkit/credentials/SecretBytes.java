package org.kmipkit.credentials;

import java.util.Arrays;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvValue;

/** Owns caller-provided secret bytes in native zeroizing memory. */
public final class SecretBytes implements AutoCloseable {
    private final NativeHandle handle;

    private SecretBytes(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SECRET_BYTES);
    }

    /** Copy bytes into a native zeroizing owner without changing the caller's array. */
    public static SecretBytes fromBytes(byte[] value) {
        if (value == null) {
            throw new InvalidInputException("secret byte input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        byte[] copy = value.clone();
        try {
            long[] output = new long[1];
            int status = NativeExtensionRegistry.secretBytesCreate(copy, output);
            CredentialInterop.throwIfFailed(status, "secret bytes");
            return new SecretBytes(output[0]);
        } finally {
            Arrays.fill(copy, (byte) 0);
        }
    }

    /** Lend a temporary byte-array copy to a synchronous callback, then clear it. */
    @SuppressWarnings("unchecked")
    public <T> T withBytes(Function<byte[], T> callback) {
        if (callback == null) {
            throw new InvalidInputException("secret byte callback is invalid");
        }
        return handle.withValue(value -> {
            byte[][] scoped = new byte[1][];
            try {
                return (T) NativeExtensionRegistry.secretBytesWithBytes(value, borrowed -> {
                    scoped[0] = borrowed;
                    try {
                        return callback.apply(borrowed);
                    } finally {
                        Arrays.fill(borrowed, (byte) 0);
                        scoped[0] = null;
                    }
                });
            } finally {
                if (scoped[0] != null) {
                    Arrays.fill(scoped[0], (byte) 0);
                }
            }
        });
    }

    /** Move the native secret allocation into a zeroizing TTLV Byte String value. */
    public TtlvValue intoTtlvValue() {
        return NativeHandle.withNativeHandles(() -> {
            try {
                TtlvValue value = NativeExtensionRegistry.secretBytesIntoTtlvValue(handle.get());
                handle.transfer();
                return value;
            } catch (InvalidInputException exception) {
                throw exception;
            } catch (RuntimeException | Error exception) {
                handle.transfer();
                throw exception;
            }
        });
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "SecretBytes[redacted]";
    }
}
