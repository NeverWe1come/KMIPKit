package org.kmipkit.fluent;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;

/** Fluent builder for a typed KMIP Interop request (§6.1.26). */
public final class InteropBuilder extends TypedOperationBuilder<InteropBuilder> {
    private Long interopFunction;
    private byte[] interopIdentifier;

    InteropBuilder(KmipClient client, Long interopFunction, byte[] interopIdentifier) {
        super(client);
        this.interopFunction = interopFunction == null
                ? null : unsigned32(interopFunction, "Interop Function");
        this.interopIdentifier = interopIdentifier == null ? null : interopIdentifier.clone();
    }

    @Override
    protected InteropBuilder self() { return this; }

    public InteropBuilder interopFunction(long value) {
        ensureOpen();
        interopFunction = unsigned32(value, "Interop Function");
        return this;
    }

    public InteropBuilder interopIdentifier(String value) {
        ensureOpen();
        if (value == null) throw new InvalidInputException("interopIdentifier is required");
        interopIdentifier = value.getBytes(StandardCharsets.UTF_8);
        return this;
    }

    public InteropBuilder interopIdentifier(byte[] value) {
        ensureOpen();
        if (value == null) throw new InvalidInputException("interopIdentifier is required");
        interopIdentifier = value.clone();
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (interopFunction == null || interopIdentifier == null) {
            throw new InvalidInputException("Interop requires interopFunction and interopIdentifier");
        }
        return validated(ClientBatchItem.interop(interopFunction, interopIdentifier));
    }
}
