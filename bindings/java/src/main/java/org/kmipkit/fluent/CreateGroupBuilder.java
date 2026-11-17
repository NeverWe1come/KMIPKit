package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;

/** Typed builder for KMIP 3.0 Create Group. */
public final class CreateGroupBuilder extends Kmip30BuilderBase<CreateGroupBuilder> {
    CreateGroupBuilder(KmipClient client, AttributeSet attributes) {
        super(client, create(client, attributes));
    }
    private static Kmip30OperationHandle create(KmipClient client, AttributeSet attributes) {
        requireClient(client);
        return Kmip30OperationHandle.createGroup(attributes);
    }
    @Override protected CreateGroupBuilder self() { return this; }
}
