package org.kmipkit.server;

import static org.junit.jupiter.api.Assertions.assertNotNull;

import org.junit.jupiter.api.Test;

final class ServerListenerApiShapeTest {
    @Test
    void serverInitiatedOperationsHaveTypedRequestAndResponseFactories() {
        assertNotNull(DiscoverVersionsRequest.class);
        assertNotNull(NotifyRequest.class);
        assertNotNull(PutRequest.class);
        assertNotNull(QueryRequest.class);
        assertNotNull(SetEndpointRoleRequest.class);
        assertNotNull(DiscoverVersionsResponse.class);
        assertNotNull(NotifyResponse.class);
        assertNotNull(PutResponse.class);
        assertNotNull(QueryResponse.class);
        assertNotNull(SetEndpointRoleResponse.class);
        assertNotNull(NotifyResponse.class.getMethod("forRequest", NotifyRequest.class));
        assertNotNull(PutResponse.class.getMethod("forRequest", PutRequest.class));
    }

    @Test
    void listenerConfigAndTypedHandlerContractsArePublic() throws ReflectiveOperationException {
        assertNotNull(ServerListenerConfiguration.class.getMethod("builder"));
        assertNotNull(ServerListener.class.getMethod(
                "bindRawTls", ServerListenerConfiguration.class));
        assertNotNull(ServerListener.class.getMethod(
                "bindHttps", ServerListenerConfiguration.class));
        assertNotNull(ServerRequestHandler.class.getMethod("handle", ServerRequest.class));
        assertNotNull(ServerReply.class.getMethod("success", ServerResponse.class));
        assertNotNull(ServerReply.class.getMethod("failure", ServerResultStatus.class,
                ServerResultReason.class, String.class));
    }

    @Test
    void acceptedBatchesExposeRequestsAndConnectionsExposeBoundedExchangeMethods()
            throws ReflectiveOperationException {
        assertNotNull(ServerListener.class.getMethod("accept", java.time.Duration.class));
        assertNotNull(ServerConnection.class.getMethod("receive",
                org.kmipkit.ttlv.CodecLimits.class, java.time.Duration.class));
        assertNotNull(ServerConnection.class.getMethod("send",
                ServerResponseMessage.class, java.time.Duration.class));
        assertNotNull(ServerRequestBatch.class.getMethod("items"));
        assertNotNull(ServerRequestBatch.class.getMethod("responseBuilder"));
        assertNotNull(ServerResponseBuilder.class.getMethod("addSuccess",
                ServerRequest.class, ServerResponse.class));
        assertNotNull(ServerResponseBuilder.class.getMethod("build"));
    }
}
