package org.kmipkit.fluent;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP Log request (§6.1.29). */
public final class LogBuilder extends TypedOperationBuilder<LogBuilder> {
    private byte[] logMessage;

    LogBuilder(KmipClient client, byte[] logMessage) {
        super(client);
        this.logMessage = logMessage == null ? null : logMessage.clone();
    }

    @Override
    protected LogBuilder self() { return this; }

    public LogBuilder logMessage(String value) {
        ensureOpen();
        if (value == null) throw new InvalidInputException("logMessage is required");
        logMessage = value.getBytes(StandardCharsets.UTF_8);
        return this;
    }

    public LogBuilder logMessage(byte[] value) {
        ensureOpen();
        if (value == null) throw new InvalidInputException("logMessage is required");
        logMessage = value.clone();
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (logMessage == null) throw new InvalidInputException("Log requires logMessage");
        return validated(ClientBatchItem.log(logMessage));
    }
}
