package org.kmipkit.extensions;

import java.time.Duration;
import java.util.Objects;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.message.ResponseMessage;
import org.kmipkit.ttlv.CodecLimits;

/** Accepted incoming connection on a ClientListener. */
public final class ClientListenerConnection implements AutoCloseable {
    private final NativeHandle handle;

    private ClientListenerConnection(long nativeHandle) {
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT_LISTENER_CONNECTION);
    }

    public static ClientListenerConnection fromNativeHandle(long nativeHandle) {
        return new ClientListenerConnection(nativeHandle);
    }

    public IncomingServerBatch recvBatch(CodecLimits limits, Duration timeout) {
        Objects.requireNonNull(limits, "limits");
        if (timeout == null || timeout.isNegative()) {
            throw new InvalidInputException("timeout must be non-negative");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
        long batchHandle = handle.withValue(raw ->
                NativeExtensionRegistry.clientListenerConnectionRecvBatch(raw, nativeLimits, timeout.toMillis()));
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
            NativeExtensionRegistry.clientListenerConnectionSendResponse(raw, respHandle, timeout.toMillis());
            return null;
        });
    }

    public RoleReversedSession intoRoleReversed() {
        NativeExtensionRegistry.ensureLoaded();
        long sessionHandle = handle.withValue(NativeExtensionRegistry::clientListenerConnectionIntoRoleReversed);
        return RoleReversedSession.fromNativeHandle(sessionHandle);
    }

    public void closeConnection() {
        handle.withValue(raw -> {
            NativeExtensionRegistry.clientListenerConnectionClose(raw);
            return null;
        });
    }

    @Override
    public void close() { handle.close(); }
}
