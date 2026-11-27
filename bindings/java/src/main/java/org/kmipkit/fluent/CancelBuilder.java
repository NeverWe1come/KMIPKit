package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.CancelRequest;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP Cancel request (§6.1.5). */
public final class CancelBuilder extends TypedOperationBuilder<CancelBuilder> {
    private byte[] correlationValue;

    CancelBuilder(KmipClient client, byte[] correlationValue) {
        super(client);
        this.correlationValue = correlationValue == null ? null : correlationValue.clone();
    }

    @Override
    protected CancelBuilder self() { return this; }

    public CancelBuilder correlationValue(byte[] bytes) {
        ensureOpen();
        if (bytes == null) throw new InvalidInputException("correlationValue is required");
        correlationValue = bytes.clone();
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (correlationValue == null) {
            throw new InvalidInputException("Cancel requires correlationValue");
        }
        try (CancelRequest req = CancelRequest.create(correlationValue)) {
            return validated(ClientBatchItem.cancel(req));
        }
    }
}
