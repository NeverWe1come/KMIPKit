package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.PollRequest;

/** Fluent builder for a typed KMIP Poll request (§6.1.38). */
public final class PollBuilder extends TypedOperationBuilder<PollBuilder> {
    private byte[] correlationValue;

    PollBuilder(KmipClient client, byte[] correlationValue) {
        super(client);
        this.correlationValue = correlationValue == null ? null : correlationValue.clone();
    }

    @Override
    protected PollBuilder self() { return this; }

    public PollBuilder correlationValue(byte[] bytes) {
        ensureOpen();
        if (bytes == null) throw new InvalidInputException("correlationValue is required");
        correlationValue = bytes.clone();
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (correlationValue == null) {
            throw new InvalidInputException("Poll requires correlationValue");
        }
        try (PollRequest req = PollRequest.create(correlationValue)) {
            return validated(ClientBatchItem.poll(req));
        }
    }
}
