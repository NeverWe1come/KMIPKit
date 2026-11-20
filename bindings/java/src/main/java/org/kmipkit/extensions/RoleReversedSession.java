package org.kmipkit.extensions;

import java.time.Duration;
import java.util.Objects;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.message.ResponseMessage;
import org.kmipkit.ttlv.CodecLimits;

/** Connection that transitioned to Role Reversed state. */
public final class RoleReversedSession implements AutoCloseable {
    private final NativeHandle handle;

    private RoleReversedSession(long nativeHandle) {
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.ROLE_REVERSED_SESSION);
    }

    public static RoleReversedSession fromNativeHandle(long nativeHandle) {
        return new RoleReversedSession(nativeHandle);
    }

    public IncomingServerBatch recvBatch(CodecLimits limits, Duration timeout) {
        Objects.requireNonNull(limits, "limits");
        if (timeout == null || timeout.isNegative()) {
            throw new InvalidInputException("timeout must be non-negative");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        long batchHandle = handle.withValue(raw ->
                NativeExtensionRegistry.roleReversedSessionRecvBatch(raw, nativeLimits, timeout.toMillis()));
        return IncomingServerBatch.fromNativeHandle(batchHandle);
    }

    public void sendResponse(ResponseMessage response, Duration timeout) {
        Objects.requireNonNull(response, "response");
        if (timeout == null || timeout.isNegative()) {
            throw new InvalidInputException("timeout must be non-negative");
        }
        NativeExtensionRegistry.ensureLoaded();
        handle.withValue(raw -> {
            long respHandle = response.nativeHandleOwner().get();
            NativeExtensionRegistry.roleReversedSessionSendResponse(raw, respHandle, timeout.toMillis());
            return null;
        });
    }

    public void closeSession() {
        handle.withValue(raw -> {
            NativeExtensionRegistry.roleReversedSessionClose(raw);
            return null;
        });
    }

    @Override
    public void close() { handle.close(); }
}
