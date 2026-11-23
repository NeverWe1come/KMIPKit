package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.protocol.CryptographicParameters;

/** Typed builder for KMIP 3.0 Decapsulate. It exposes no shared-secret response field. */
public final class DecapsulateBuilder extends Kmip30BuilderBase<DecapsulateBuilder> {
    private final Kmip30OperationHandle operation;
    DecapsulateBuilder(KmipClient client, UniqueIdentifier identifier, byte[] ciphertext) {
        this(requireClient(client), Kmip30OperationHandle.decapsulate(identifier, ciphertext));
    }
    private DecapsulateBuilder(KmipClient client, Kmip30OperationHandle operation) {
        super(client, operation);
        this.operation = operation;
    }
    public DecapsulateBuilder attributes(AttributeSet value) { operation.setAttributes(value); return this; }
    public DecapsulateBuilder cryptographicParameters(CryptographicParameters value) {
        operation.setCryptographicParameters(value); return this;
    }
    @Override protected DecapsulateBuilder self() { return this; }
}
