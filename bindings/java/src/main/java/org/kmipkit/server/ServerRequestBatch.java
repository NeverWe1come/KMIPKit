package org.kmipkit.server;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.internal.NativeHandle;

/** Decoded, owned request batch received from a KMIP server. */
public final class ServerRequestBatch implements AutoCloseable {
    private static final int DISCOVER_VERSIONS = 30;
    private static final int NOTIFY = 27;
    private static final int PUT = 28;
    private static final int QUERY = 24;
    private static final int SET_ENDPOINT_ROLE = 50;

    private final NativeHandle handle;
    @SuppressWarnings("unused")
    private final ServerConnection owner;
    private final ProtocolVersion protocolVersion;

    ServerRequestBatch(long nativeHandle, ServerConnection owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.SERVER_REQUEST_BATCH);
        this.owner = owner;
        int[] version = ServerNative.batchProtocolVersion(nativeHandle);
        protocolVersion = new ProtocolVersion(version[0], version[1]);
    }

    public ProtocolVersion protocolVersion() { return protocolVersion; }
    boolean belongsTo(ServerConnection connection) { return owner == connection; }

    public List<ServerRequest> items() {
        long count = handle.withValue(ServerNative::batchCount);
        List<ServerRequest> requests = new ArrayList<>((int) count);
        for (long index = 0; index < count; index++) {
            long batchIndex = index;
            long item = handle.withValue(raw -> ServerNative.batchItemAt(raw, batchIndex));
            int operation = ServerNative.requestOperation(item);
            requests.add(switch (operation) {
                case DISCOVER_VERSIONS -> new DiscoverVersionsRequest(item, this);
                case NOTIFY -> new NotifyRequest(item, this);
                case PUT -> new PutRequest(item, this);
                case QUERY -> new QueryRequest(item, this);
                case SET_ENDPOINT_ROLE -> new SetEndpointRoleRequest(item, this);
                default -> new ServerRequest(item, this, operation);
            });
        }
        return List.copyOf(requests);
    }

    public ServerResponseBuilder responseBuilder() {
        return new ServerResponseBuilder(
                ServerNative.responseBuilderCreate(handle.get()), this);
    }

    public ServerResponseMessage respond(ServerRequestHandler handler) {
        if (handler == null) throw new IllegalArgumentException("handler is required");
        List<ServerRequest> requests = items();
        List<ServerResponse> responses = new ArrayList<>();
        try (ServerResponseBuilder builder = responseBuilder()) {
            for (ServerRequest request : requests) {
                ServerReply reply = handler.handle(request);
                if (reply == null) throw new IllegalArgumentException("handler returned null");
                if (reply.response() != null) {
                    responses.add(reply.response());
                    builder.addSuccess(request, reply.response());
                }
                else builder.addFailure(request, reply);
            }
            return builder.build();
        } finally {
            responses.forEach(ServerResponse::close);
            requests.forEach(ServerRequest::close);
        }
    }

    @Override
    public void close() { handle.close(); }
}
