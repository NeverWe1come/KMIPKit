package org.kmipkit.server;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Builds one KMIP response for every request in a received batch. */
public final class ServerResponseBuilder implements AutoCloseable {
    private final NativeHandle handle;
    private final ServerRequestBatch owner;

    ServerResponseBuilder(long nativeHandle, ServerRequestBatch owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_RESPONSE_BUILDER);
        this.owner = owner;
    }

    public void addSuccess(ServerRequest request, ServerResponse response) {
        requireOwner(request);
        if (response == null || response.request() != request) {
            throw new IllegalArgumentException("typed response belongs to another request");
        }
        NativeHandle.withValues(new NativeHandle[] {
                handle, request.handleForTransfer(), response.handleForTransfer()}, handles -> {
            ServerNative.responseBuilderAddSuccess(handles[0], handles[1], handles[2]);
            return null;
        });
    }

    public void addFailure(ServerRequest request, ServerResultStatus status,
            ServerResultReason reason, String message) {
        requireOwner(request);
        ServerReply reply = ServerReply.failure(status, reason, message);
        NativeHandle.withValues(new NativeHandle[] {handle, request.handleForTransfer()}, handles -> {
            ServerNative.responseBuilderAddFailure(handles[0], handles[1],
                    reply.status().nativeValue, reply.reason() != null,
                    reply.reason() == null ? 0 : reply.reason().nativeValue, reply.message());
            return null;
        });
    }

    void addFailure(ServerRequest request, ServerReply reply) {
        addFailure(request, reply.status(), reply.reason(), reply.message());
    }

    public ServerResponseMessage build() {
        return new ServerResponseMessage(handle.withValue(ServerNative::responseBuilderBuild), owner);
    }

    private void requireOwner(ServerRequest request) {
        if (request == null || request.owner() != owner) {
            throw new IllegalArgumentException("request belongs to another batch");
        }
    }

    @Override public void close() { handle.close(); }
}
