package org.kmipkit.server;

import org.kmipkit.InvalidInputException;

/** One explicit success or failure decision returned by a server request handler. */
public final class ServerReply {
    private final ServerResponse response;
    private final ServerResultStatus status;
    private final ServerResultReason reason;
    private final String message;

    private ServerReply(ServerResponse response, ServerResultStatus status,
            ServerResultReason reason, String message) {
        this.response = response;
        this.status = status;
        this.reason = reason;
        this.message = message;
    }

    public static ServerReply success(ServerResponse response) {
        if (response == null) throw new InvalidInputException("response is required");
        return new ServerReply(response, null, null, null);
    }

    public static ServerReply failure(ServerResultStatus status,
            ServerResultReason reason, String message) {
        if (status == null || (message != null && message.indexOf('\0') >= 0)) {
            throw new InvalidInputException("failure result is invalid");
        }
        return new ServerReply(null, status, reason, message);
    }

    public ServerResponse response() { return response; }
    public ServerResultStatus status() { return status; }
    public ServerResultReason reason() { return reason; }
    public String message() { return message; }
}
