package org.kmipkit.credentials;

import java.util.OptionalLong;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/**
 * Caller-owned state that enforces strictly increasing Hashed Password timestamps.
 * Keep one sequence for each Username and logical credential producer.
 */
public final class HashedPasswordTimestampSequence implements AutoCloseable {
    private final NativeHandle handle;

    private HashedPasswordTimestampSequence(long nativeHandle) {
        handle = NativeHandle.owned(
                nativeHandle, NativeExtensionRegistry.HASHED_PASSWORD_TIMESTAMP_SEQUENCE);
    }

    /** Create a new sequence or restore one from the caller's persisted last timestamp. */
    public static HashedPasswordTimestampSequence create(OptionalLong lastTimestamp) {
        if (lastTimestamp == null) {
            throw new InvalidInputException("timestamp sequence restore value is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            long[] output = new long[1];
            long restored = lastTimestamp.isPresent() ? lastTimestamp.getAsLong() : 0;
            int status = NativeExtensionRegistry.hashedPasswordTimestampSequenceNew(
                    lastTimestamp.isPresent(), restored, output);
            CredentialInterop.throwIfFailed(status, "hashed password timestamp sequence");
            return new HashedPasswordTimestampSequence(output[0]);
        });
    }

    /** Record a caller-supplied timestamp, rejecting equal or earlier values. */
    public void record(long timestamp) {
        handle.withValue(value -> {
            CredentialInterop.throwIfFailed(
                    NativeExtensionRegistry.hashedPasswordTimestampSequenceRecord(value, timestamp),
                    "hashed password timestamp sequence");
            return null;
        });
    }

    /** Return the last accepted timestamp for caller-managed persistence. */
    public OptionalLong lastTimestamp() {
        return handle.withValue(value -> {
            long[] output = new long[2];
            CredentialInterop.throwIfFailed(
                    NativeExtensionRegistry.hashedPasswordTimestampSequenceLastTimestamp(
                            value, output),
                    "hashed password timestamp sequence");
            return output[0] == 0 ? OptionalLong.empty() : OptionalLong.of(output[1]);
        });
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "HashedPasswordTimestampSequence[redacted]";
    }
}
