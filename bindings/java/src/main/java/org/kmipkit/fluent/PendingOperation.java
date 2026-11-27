package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchResponse;

/** Handle for an asynchronous KMIP operation carrying its correlation value. */
public final class PendingOperation {
    private final byte[] correlationValue;

    public PendingOperation(byte[] correlationValue) {
        if (correlationValue == null || correlationValue.length == 0) {
            throw new InvalidInputException("correlationValue is required");
        }
        this.correlationValue = correlationValue.clone();
    }

    public byte[] correlationValue() {
        return correlationValue.clone();
    }

    public ClientBatchResponse poll(KmipClient client) throws org.kmipkit.ClientOperationException {
        return new PollBuilder(client, correlationValue).execute(client.codecLimits());
    }

    public ClientBatchResponse process(KmipClient client) throws org.kmipkit.ClientOperationException {
        return new ProcessBuilder(client, correlationValue).execute(client.codecLimits());
    }

    public ClientBatchResponse cancel(KmipClient client) throws org.kmipkit.ClientOperationException {
        return new CancelBuilder(client, correlationValue).execute(client.codecLimits());
    }
}
