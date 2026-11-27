package org.kmipkit.fluent;

import java.util.List;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.RevocationReason;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Domain-grouped fluent namespaces exposed on {@link KmipClient}. */
public final class KmipNamespaces {
    private KmipNamespaces() {}

    /** Key generation, import/export, derivation, split keys, and lookup. */
    public static final class KeysClient {
        private final FluentRequest req;
        public KeysClient(KmipClient client) { this.req = new FluentRequest(client); }
        public FluentRequest raw() { return req; }
        public SymmetricKeyBuilder createSymmetric() { return req.createSymmetricKey(); }
        public CreateKeyPairBuilder createKeyPair() { return req.createKeyPair(); }
        public CreateBuilder create() { return req.create(); }
        public CreateSplitKeyBuilder createSplitKey() { return req.createSplitKey(); }
        public JoinSplitKeyBuilder joinSplitKey() { return req.joinSplitKey(); }
        public DeriveKeyBuilder derive() { return req.deriveKey(); }
        public RegisterBuilder register() { return req.register(); }
        public ImportBuilder importObject() { return req.importObject(); }
        public ExportBuilder export() { return req.export(); }
        public RekeyBuilder rekey() { return req.rekey(); }
        public RekeyKeyPairBuilder rekeyKeyPair() { return req.rekeyKeyPair(); }
        public LocateBuilder locate(AttributeSet attributes) { return req.locate(attributes); }
        public OperationBuilder get(UniqueIdentifier id) { return req.get(id); }
        public OperationBuilder activate(UniqueIdentifier id) { return req.activate(id); }
        public RevokeBuilder revoke(UniqueIdentifier id, RevocationReason reason) { return req.revoke(id, reason); }
        public OperationBuilder destroy(UniqueIdentifier id) { return req.destroy(id); }
    }

    /** Cryptographic, hashing, RNG, and KMIP 3.0 KEM operations. */
    public static final class CryptoClient {
        private final FluentRequest req;
        public CryptoClient(KmipClient client) { this.req = new FluentRequest(client); }
        public EncryptBuilder encrypt() { return req.encrypt(); }
        public DecryptBuilder decrypt() { return req.decrypt(); }
        public SignBuilder sign() { return req.sign(); }
        public SignatureVerifyBuilder signatureVerify() { return req.signatureVerify(); }
        public MacBuilder mac() { return req.mac(); }
        public MacVerifyBuilder macVerify() { return req.macVerify(); }
        public HashBuilder hash() { return req.hash(); }
        public RngRetrieveBuilder rngRetrieve(int dataLength) { return req.rngRetrieve().dataLength(dataLength); }
        public RngSeedBuilder rngSeed(byte[] data) { return req.rngSeed().data(data); }
        public EncapsulateBuilder encapsulate(UniqueIdentifier publicKeyId) { return req.encapsulate(publicKeyId); }
        public DecapsulateBuilder decapsulate(UniqueIdentifier privateKeyId, byte[] ciphertext) {
            return req.decapsulate(privateKeyId, ciphertext);
        }
    }

    /** Certificate issuance, re-certification, and validation. */
    public static final class CertificatesClient {
        private final FluentRequest req;
        public CertificatesClient(KmipClient client) { this.req = new FluentRequest(client); }
        public CertifyBuilder certify() { return req.certify(); }
        public RecertifyBuilder recertify() { return req.recertify(); }
        public ValidateBuilder validate() { return req.validate(); }
        public OperationBuilder get(UniqueIdentifier id) { return req.get(id); }
        public LocateBuilder locate(AttributeSet attributes) { return req.locate(attributes); }
        public RevokeBuilder revoke(UniqueIdentifier id, RevocationReason reason) { return req.revoke(id, reason); }
        public OperationBuilder destroy(UniqueIdentifier id) { return req.destroy(id); }
    }

    /** Secret data, archival, recovery, and usage allocation. */
    public static final class SecretsClient {
        private final FluentRequest req;
        public SecretsClient(KmipClient client) { this.req = new FluentRequest(client); }
        public RegisterBuilder register() { return req.register(); }
        public OperationBuilder get(UniqueIdentifier id) { return req.get(id); }
        public OperationBuilder archive(UniqueIdentifier id) { return req.archive(id); }
        public OperationBuilder recover(UniqueIdentifier id) { return req.recover(id); }
        public ObtainLeaseBuilder obtainLease(UniqueIdentifier id) { return req.obtainLease().uniqueIdentifier(id); }
        public GetUsageAllocationBuilder getUsageAllocation(UniqueIdentifier id) {
            return req.getUsageAllocation().uniqueIdentifier(id);
        }
        public OperationBuilder destroy(UniqueIdentifier id) { return req.destroy(id); }
    }

    /** Object attribute and policy operations. */
    public static final class AttributesClient {
        private final FluentRequest req;
        public AttributesClient(KmipClient client) { this.req = new FluentRequest(client); }
        public GetAttributesBuilder get(UniqueIdentifier id, List<AttributeReference> references) {
            return req.getAttributes(id, references);
        }
        public GetAttributeListBuilder getList(UniqueIdentifier id) { return req.getAttributeList(id); }
        public AddAttributeBuilder add(UniqueIdentifier id) { return req.addAttribute().uniqueIdentifier(id.text()); }
        public SetAttributeBuilder set(UniqueIdentifier id) { return req.setAttribute().uniqueIdentifier(id.text()); }
        public ModifyAttributeBuilder modify(UniqueIdentifier id) { return req.modifyAttribute().uniqueIdentifier(id); }
        public AdjustAttributeBuilder adjust(UniqueIdentifier id) { return req.adjustAttribute().uniqueIdentifier(id); }
        public DeleteAttributeBuilder delete(UniqueIdentifier id) { return req.deleteAttribute().uniqueIdentifier(id.text()); }
        public GetConstraintsBuilder getConstraints() { return req.getConstraints(); }
        public SetConstraintsBuilder setConstraints() { return req.setConstraints(); }
        public SetDefaultsBuilder setDefaults() { return req.setDefaults(); }
    }

    /** Sessions, endpoint roles, and KMIP 3.0 users/groups/credentials. */
    public static final class SessionsClient {
        private final FluentRequest req;
        public SessionsClient(KmipClient client) { this.req = new FluentRequest(client); }
        public LoginBuilder login() { return req.login(); }
        public LogoutBuilder logout(TtlvStructure ticket) { return req.logout(ticket); }
        public DelegatedLoginBuilder delegatedLogin() { return req.delegatedLogin(); }
        public SetEndpointRoleBuilder setEndpointRole(long role) { return req.setEndpointRole(role); }
        public CreateUserBuilder createUser(AttributeSet attributes) { return req.createUser(attributes); }
        public CreateGroupBuilder createGroup(AttributeSet attributes) { return req.createGroup(attributes); }
        public CreateCredentialBuilder createCredential(AttributeSet attributes, CreateCredentialObject object) {
            return req.createCredential(attributes, object);
        }
    }

    /** Server capabilities, auditing, PKCS#11, and KMIP 3.0 decommissioning. */
    public static final class AdminClient {
        private final FluentRequest req;
        public AdminClient(KmipClient client) { this.req = new FluentRequest(client); }
        public PingBuilder ping() { return req.ping(); }
        public QueryBuilder query() { return req.query(); }
        public DiscoverVersionsBuilder discoverVersions() { return req.discoverVersions(); }
        public CheckBuilder check() { return req.check(); }
        public LogBuilder log(String message) { return req.log(message); }
        public Pkcs11Builder pkcs11() { return req.pkcs11(); }
        public InteropBuilder interop(long fn, String id) { return req.interop(fn, id); }
        public ReProvisionBuilder reprovision() { return req.reprovision(); }
        public DeactivateBuilder deactivate(UniqueIdentifier id) { return req.deactivate(id); }
        public ObliterateBuilder obliterate(UniqueIdentifier id) { return req.obliterate(id); }
    }

    /** Asynchronous job polling, processing, and cancellation. */
    public static final class AsyncOpsClient {
        private final FluentRequest req;
        public AsyncOpsClient(KmipClient client) { this.req = new FluentRequest(client); }
        public PollBuilder poll(byte[] correlationValue) { return req.poll(correlationValue); }
        public CancelBuilder cancel(byte[] correlationValue) { return req.cancel(correlationValue); }
        public ProcessBuilder process(byte[] correlationValue) { return req.process(correlationValue); }
        public QueryAsyncRequestsBuilder query() { return req.queryAsyncRequests(); }
    }
}
