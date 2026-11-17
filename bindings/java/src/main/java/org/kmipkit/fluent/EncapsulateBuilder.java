package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.protocol.CryptographicParameters;

/** Typed builder for KMIP 3.0 Encapsulate. */
public final class EncapsulateBuilder extends Kmip30BuilderBase<EncapsulateBuilder> {
    private final Kmip30OperationHandle operation;
    EncapsulateBuilder(KmipClient client, UniqueIdentifier identifier) {
        this(requireClient(client), Kmip30OperationHandle.encapsulate(identifier));
    }
    private EncapsulateBuilder(KmipClient client, Kmip30OperationHandle operation) {
        super(client, operation);
        this.operation = operation;
    }
    public EncapsulateBuilder attributes(AttributeSet value) { operation.setAttributes(value); return this; }
    public EncapsulateBuilder cryptographicParameters(CryptographicParameters value) {
        operation.setCryptographicParameters(value); return this;
    }
    @Override protected EncapsulateBuilder self() { return this; }
}
