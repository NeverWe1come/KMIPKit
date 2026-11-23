package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Typed builder for KMIP 3.0 Deactivate. */
public final class DeactivateBuilder extends Kmip30BuilderBase<DeactivateBuilder> {
    private final Kmip30OperationHandle operation;

    DeactivateBuilder(KmipClient client, UniqueIdentifier identifier) {
        this(requireClient(client), Kmip30OperationHandle.deactivate(identifier));
    }

    private DeactivateBuilder(KmipClient client, Kmip30OperationHandle operation) {
        super(client, operation);
        this.operation = operation;
    }

    public DeactivateBuilder deactivationReason(TtlvStructure reason) {
        operation.setDeactivationReason(reason);
        return this;
    }

    public DeactivateBuilder deactivationDate(long unixSeconds) {
        operation.setDeactivationDate(unixSeconds);
        return this;
    }

    @Override protected DeactivateBuilder self() { return this; }
}
