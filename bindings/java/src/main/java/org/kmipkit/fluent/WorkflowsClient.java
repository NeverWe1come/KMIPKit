package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.protocol.KeyUsage;

/** High-level multi-step workflows combining common KMIP sequences safely. */
public final class WorkflowsClient {
    private final KmipClient client;

    public WorkflowsClient(KmipClient client) {
        if (client == null) throw new InvalidInputException("client is required");
        this.client = client;
    }

    /** Creates a symmetric key and immediately activates it, returning its {@link UniqueIdentifier}. */
    public UniqueIdentifier createActiveSymmetricKey(KeySpec spec, String name, KeyUsage... usage)
            throws org.kmipkit.ClientOperationException {
        try (SymmetricKeyBuilder b = new SymmetricKeyBuilder(client)
                .withKeySpec(spec)
                .usageMask(usage)) {
            if (name != null) b.name(name);
            try (ClientBatchResponse resp = b.execute(client.codecLimits());
                 ClientBatchItemResult item = resp.itemAt(0)) {
                UniqueIdentifier id = item.createIdentifier()
                        .orElseThrow(() -> new InvalidInputException("Create omitted UniqueIdentifier"));
                new OperationBuilder(client, id, OperationBuilder.Kind.ACTIVATE)
                        .execute(client.codecLimits())
                        .close();
                return id;
            }
        }
    }

    /** Revokes an object and immediately destroys it. */
    public void revokeAndDestroy(UniqueIdentifier id, RevocationReason reason)
            throws org.kmipkit.ClientOperationException {
        if (id == null || reason == null) throw new InvalidInputException("id and reason are required");
        new FluentRequest(client).revoke(id, reason).execute(client.codecLimits()).close();
        new OperationBuilder(client, id, true).execute(client.codecLimits()).close();
    }
}
