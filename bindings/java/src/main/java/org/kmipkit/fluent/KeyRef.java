package org.kmipkit.fluent;

import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.UniqueIdentifier;

/** Object-scoped fluent handle returned by {@link KmipClient#key(String)}. */
public final class KeyRef {
    private final FluentRequest req;
    private final UniqueIdentifier id;

    public KeyRef(KmipClient client, UniqueIdentifier id) {
        if (id == null) throw new InvalidInputException("UniqueIdentifier is required");
        this.req = new FluentRequest(client);
        this.id = id;
    }

    public UniqueIdentifier id() { return id; }
    public OperationBuilder get() { return req.get(id); }
    public OperationBuilder activate() { return req.activate(id); }
    public DeactivateBuilder deactivate() { return req.deactivate(id); }
    public ObliterateBuilder obliterate() { return req.obliterate(id); }
    public RevokeBuilder revoke(RevocationReason reason) { return req.revoke(id, reason); }
    public OperationBuilder destroy() { return req.destroy(id); }
    public OperationBuilder archive() { return req.archive(id); }
    public OperationBuilder recover() { return req.recover(id); }
    public RekeyBuilder rekey() { return req.rekey().uniqueIdentifier(id); }
    public RekeyKeyPairBuilder rekeyKeyPair() { return req.rekeyKeyPair().privateKeyUniqueIdentifier(id); }
    public ExportBuilder export() { return req.export().uniqueIdentifier(id); }
    public ObtainLeaseBuilder obtainLease() { return req.obtainLease().uniqueIdentifier(id); }
    public EncryptBuilder encrypt() { return req.encrypt().uniqueIdentifier(id); }
    public DecryptBuilder decrypt() { return req.decrypt().uniqueIdentifier(id); }
    public SignBuilder sign() { return req.sign().uniqueIdentifier(id); }
    public SignatureVerifyBuilder verify() { return req.signatureVerify().uniqueIdentifier(id); }
    public SignatureVerifyBuilder signatureVerify() { return verify(); }
    public MacBuilder mac() { return req.mac().uniqueIdentifier(id); }
    public MacVerifyBuilder macVerify() { return req.macVerify().uniqueIdentifier(id); }
    public EncapsulateBuilder encapsulate() { return req.encapsulate(id); }
    public DecapsulateBuilder decapsulate(byte[] ciphertext) { return req.decapsulate(id, ciphertext); }
    public GetAttributesBuilder attributes(List<AttributeReference> refs) { return req.getAttributes(id, refs); }
    public GetAttributeListBuilder attributeList() { return req.getAttributeList(id); }
    public AddAttributeBuilder addAttribute() { return req.addAttribute().uniqueIdentifier(id.text()); }
    public SetAttributeBuilder setAttribute() { return req.setAttribute().uniqueIdentifier(id.text()); }
    public ModifyAttributeBuilder modifyAttribute() { return req.modifyAttribute().uniqueIdentifier(id); }
    public AdjustAttributeBuilder adjustAttribute() { return req.adjustAttribute().uniqueIdentifier(id); }
    public DeleteAttributeBuilder deleteAttribute() { return req.deleteAttribute().uniqueIdentifier(id.text()); }
}
