package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ProcessRequest;

/** Fluent builder for a typed KMIP Process request (§6.1.39). */
public final class ProcessBuilder extends TypedOperationBuilder<ProcessBuilder> {
    private byte[] correlationValue;

    ProcessBuilder(KmipClient client, byte[] correlationValue) {
        super(client);
        this.correlationValue = correlationValue == null ? null : correlationValue.clone();
    }

    @Override
    protected ProcessBuilder self() { return this; }

    public ProcessBuilder correlationValue(byte[] bytes) {
        ensureOpen();
        if (bytes == null) throw new InvalidInputException("correlationValue is required");
        correlationValue = bytes.clone();
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (correlationValue == null) {
            throw new InvalidInputException("Process requires correlationValue");
        }
        try (ProcessRequest req = ProcessRequest.create(correlationValue)) {
            return validated(ClientBatchItem.process(req));
        }
    }
}
