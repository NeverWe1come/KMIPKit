package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;

/** Typed builder for KMIP 3.0 Create Credential. */
public final class CreateCredentialBuilder extends Kmip30BuilderBase<CreateCredentialBuilder> {
    CreateCredentialBuilder(KmipClient client, AttributeSet attributes, CreateCredentialObject object) {
        super(client, create(requireClient(client), attributes, object));
    }

    private static Kmip30OperationHandle create(KmipClient client, AttributeSet attributes, CreateCredentialObject object) {
        if (attributes == null || object == null) {
            throw new InvalidInputException("Create Credential inputs are invalid");
        }
        return Kmip30OperationHandle.createCredential(attributes, object.nativeObject());
    }

    @Override protected CreateCredentialBuilder self() { return this; }
}
