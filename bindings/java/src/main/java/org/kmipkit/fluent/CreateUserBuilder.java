package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;

/** Typed builder for KMIP 3.0 Create User. */
public final class CreateUserBuilder extends Kmip30BuilderBase<CreateUserBuilder> {
    CreateUserBuilder(KmipClient client, AttributeSet attributes) {
        super(client, create(client, attributes));
    }
    private static Kmip30OperationHandle create(KmipClient client, AttributeSet attributes) {
        requireClient(client);
        return Kmip30OperationHandle.createUser(attributes);
    }
    @Override protected CreateUserBuilder self() { return this; }
}
