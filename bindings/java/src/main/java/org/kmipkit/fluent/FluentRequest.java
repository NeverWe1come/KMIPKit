package org.kmipkit.fluent;

import org.kmipkit.extensions.UniqueIdentifier;
import java.util.List;

/** Entry point for one task-oriented, typed client operation. */
public final class FluentRequest {
    private final org.kmipkit.client.KmipClient client;
    public FluentRequest(org.kmipkit.client.KmipClient client) {
        if (client == null) throw new org.kmipkit.InvalidInputException("client is required");
        this.client = client;
    }
    public SymmetricKeyBuilder createSymmetricKey() { return new SymmetricKeyBuilder(client); }
    public OperationBuilder get(UniqueIdentifier id) { return new OperationBuilder(client, id, false); }
    public OperationBuilder destroy(UniqueIdentifier id) { return new OperationBuilder(client, id, true); }
    public GetAttributesBuilder getAttributes(
            UniqueIdentifier id, List<org.kmipkit.extensions.AttributeReference> references) {
        return new GetAttributesBuilder(client, id, references);
    }
    public GetAttributeListBuilder getAttributeList() {
        return new GetAttributeListBuilder(client, null);
    }
    public GetAttributeListBuilder getAttributeList(UniqueIdentifier id) {
        return new GetAttributeListBuilder(client, id);
    }
    public EncryptBuilder encrypt() { return new EncryptBuilder(client); }
    public DecryptBuilder decrypt() { return new DecryptBuilder(client); }
    public SignBuilder sign() { return new SignBuilder(client); }
    public SignatureVerifyBuilder signatureVerify() { return new SignatureVerifyBuilder(client); }
    public MacBuilder mac() { return new MacBuilder(client); }
    public MacVerifyBuilder macVerify() { return new MacVerifyBuilder(client); }
    public HashBuilder hash() { return new HashBuilder(client); }
}
