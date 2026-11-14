package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.UniqueIdentifier;

/** Typed builder for KMIP 3.0 Obliterate. */
public final class ObliterateBuilder extends Kmip30BuilderBase<ObliterateBuilder> {
    ObliterateBuilder(KmipClient client, UniqueIdentifier identifier) {
        super(client, create(client, identifier));
    }
    private static Kmip30OperationHandle create(KmipClient client, UniqueIdentifier identifier) {
        requireClient(client);
        return Kmip30OperationHandle.obliterate(identifier);
    }
    @Override protected ObliterateBuilder self() { return this; }
}
