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
    /** Configures KEM parameters from a curated KMIP 3.0 {@link KemSpec}. */
    public DecapsulateBuilder withKemSpec(KemSpec spec) {
        if (spec == null) throw new org.kmipkit.InvalidInputException("KemSpec is required");
        try (CryptographicParameters params = spec.toCryptographicParameters()) {
            return cryptographicParameters(params);
        }
    }
    @Override protected DecapsulateBuilder self() { return this; }
}
