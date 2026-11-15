package org.kmipkit.server;

/** User handler invoked once for each received request. */
@FunctionalInterface
public interface ServerRequestHandler {
    ServerReply handle(ServerRequest request);
}
