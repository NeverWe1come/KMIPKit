package org.kmipkit.fluent;

import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.RevocationReason;
import java.nio.charset.StandardCharsets;
import java.util.List;
import org.kmipkit.ttlv.TtlvStructure;

/** Entry point for one task-oriented, typed client operation. */
public final class FluentRequest {
    private final org.kmipkit.client.KmipClient client;
    public FluentRequest(org.kmipkit.client.KmipClient client) {
        if (client == null) throw new org.kmipkit.InvalidInputException("client is required");
        this.client = client;
    }
    public PingBuilder ping() { return new PingBuilder(client); }
    public CreateBuilder create() { return new CreateBuilder(client); }
    public CreateSplitKeyBuilder createSplitKey() { return new CreateSplitKeyBuilder(client); }
    public RegisterBuilder register() { return new RegisterBuilder(client); }
    public ActivateBuilder activate() { return new ActivateBuilder(client); }
    public ArchiveBuilder archive() { return new ArchiveBuilder(client); }
    public RecoverBuilder recover() { return new RecoverBuilder(client); }
    public RekeyBuilder rekey() { return new RekeyBuilder(client); }
    public RekeyKeyPairBuilder rekeyKeyPair() { return new RekeyKeyPairBuilder(client); }
    public RecertifyBuilder recertify() { return new RecertifyBuilder(client); }
    public SymmetricKeyBuilder createSymmetricKey() { return new SymmetricKeyBuilder(client); }
    public CreateKeyPairBuilder createKeyPair() { return new CreateKeyPairBuilder(client); }
    public AddAttributeBuilder addAttribute() { return new AddAttributeBuilder(client); }
    public SetAttributeBuilder setAttribute() { return new SetAttributeBuilder(client); }
    public DeleteAttributeBuilder deleteAttribute() { return new DeleteAttributeBuilder(client); }
    public LocateBuilder locate(AttributeSet attributes) {
        return new LocateBuilder(client, attributes);
    }
    public RevokeBuilder revoke(long reasonCode) {
        return new RevokeBuilder(client, reasonCode, null);
    }
    public RevokeBuilder revoke(long reasonCode, String revocationMessage) {
        return new RevokeBuilder(client, reasonCode, revocationMessage);
    }
    public RevokeBuilder revoke(UniqueIdentifier id, RevocationReason reason) {
        if (reason == null) throw new org.kmipkit.InvalidInputException("revocation reason is required");
        RevokeBuilder builder = new RevokeBuilder(client, reason.rawCode(), reason.message().orElse(null));
        return builder.uniqueIdentifier(id);
    }
    public OperationBuilder get(UniqueIdentifier id) { return new OperationBuilder(client, id, false); }
    public GetBuilder get() { return new GetBuilder(client); }
    public OperationBuilder destroy(UniqueIdentifier id) { return new OperationBuilder(client, id, true); }
    public OperationBuilder activate(UniqueIdentifier id) {
        return new OperationBuilder(client, id, OperationBuilder.Kind.ACTIVATE);
    }
    public OperationBuilder archive(UniqueIdentifier id) {
        return new OperationBuilder(client, id, OperationBuilder.Kind.ARCHIVE);
    }
    public OperationBuilder recover(UniqueIdentifier id) {
        return new OperationBuilder(client, id, OperationBuilder.Kind.RECOVER);
    }
    public CreateCredentialBuilder createCredential(
            AttributeSet attributes, CreateCredentialObject object) {
        return new CreateCredentialBuilder(client, attributes, object);
    }
    public CreateGroupBuilder createGroup(AttributeSet attributes) {
        return new CreateGroupBuilder(client, attributes);
    }
    public CreateUserBuilder createUser(AttributeSet attributes) {
        return new CreateUserBuilder(client, attributes);
    }
    public DeactivateBuilder deactivate(UniqueIdentifier identifier) {
        return new DeactivateBuilder(client, identifier);
    }
    public DecapsulateBuilder decapsulate(UniqueIdentifier identifier, byte[] ciphertext) {
        return new DecapsulateBuilder(client, identifier, ciphertext);
    }
    public EncapsulateBuilder encapsulate(UniqueIdentifier identifier) {
        return new EncapsulateBuilder(client, identifier);
    }
    public ObliterateBuilder obliterate(UniqueIdentifier identifier) {
        return new ObliterateBuilder(client, identifier);
    }
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
    public QueryBuilder query() { return new QueryBuilder(client); }
    public DiscoverVersionsBuilder discoverVersions() { return new DiscoverVersionsBuilder(client); }
    public CheckBuilder check() { return new CheckBuilder(client); }
    public ExportBuilder export() { return new ExportBuilder(client); }
    public ImportBuilder importObject() { return new ImportBuilder(client); }
    public RngRetrieveBuilder rngRetrieve() { return new RngRetrieveBuilder(client); }
    public RngSeedBuilder rngSeed() { return new RngSeedBuilder(client); }
    public CertifyBuilder certify() { return new CertifyBuilder(client); }
    public DeriveKeyBuilder deriveKey() { return new DeriveKeyBuilder(client); }
    public ValidateBuilder validate() { return new ValidateBuilder(client); }
    public JoinSplitKeyBuilder joinSplitKey() { return new JoinSplitKeyBuilder(client); }
    public ObtainLeaseBuilder obtainLease() { return new ObtainLeaseBuilder(client); }
    public GetUsageAllocationBuilder getUsageAllocation() { return new GetUsageAllocationBuilder(client); }
    public ModifyAttributeBuilder modifyAttribute() { return new ModifyAttributeBuilder(client); }
    public AdjustAttributeBuilder adjustAttribute() { return new AdjustAttributeBuilder(client); }
    public GetConstraintsBuilder getConstraints() { return new GetConstraintsBuilder(client); }
    public CancelBuilder cancel() { return new CancelBuilder(client, null); }
    public CancelBuilder cancel(byte[] correlationValue) { return new CancelBuilder(client, correlationValue); }
    public PollBuilder poll() { return new PollBuilder(client, null); }
    public PollBuilder poll(byte[] correlationValue) { return new PollBuilder(client, correlationValue); }
    public ProcessBuilder process() { return new ProcessBuilder(client, null); }
    public ProcessBuilder process(byte[] correlationValue) { return new ProcessBuilder(client, correlationValue); }
    public QueryAsyncRequestsBuilder queryAsyncRequests() { return new QueryAsyncRequestsBuilder(client); }
    public LoginBuilder login() { return new LoginBuilder(client); }
    public LogoutBuilder logout() { return new LogoutBuilder(client, null); }
    public LogoutBuilder logout(TtlvStructure ticket) { return new LogoutBuilder(client, ticket); }
    public DelegatedLoginBuilder delegatedLogin() { return new DelegatedLoginBuilder(client, null); }
    public DelegatedLoginBuilder delegatedLogin(TtlvStructure rights) {
        return new DelegatedLoginBuilder(client, rights);
    }
    public SetEndpointRoleBuilder setEndpointRole() { return new SetEndpointRoleBuilder(client, null, null); }
    public SetEndpointRoleBuilder setEndpointRole(long role) {
        return new SetEndpointRoleBuilder(client, role, null);
    }
    public SetEndpointRoleBuilder setEndpointRole(long role, String vendorIdentification) {
        return new SetEndpointRoleBuilder(client, role, vendorIdentification);
    }
    public Pkcs11Builder pkcs11() { return new Pkcs11Builder(client, null); }
    public Pkcs11Builder pkcs11(TtlvStructure function) { return new Pkcs11Builder(client, function); }
    public InteropBuilder interop() { return new InteropBuilder(client, null, null); }
    public InteropBuilder interop(long interopFunction, String interopIdentifier) {
        byte[] encoded = interopIdentifier == null ? null : interopIdentifier.getBytes(StandardCharsets.UTF_8);
        return new InteropBuilder(client, interopFunction, encoded);
    }
    public InteropBuilder interop(long interopFunction, byte[] interopIdentifier) {
        return new InteropBuilder(client, interopFunction, interopIdentifier);
    }
    public ReProvisionBuilder reprovision() { return new ReProvisionBuilder(client); }
    public SetDefaultsBuilder setDefaults() { return new SetDefaultsBuilder(client, null); }
    public SetDefaultsBuilder setDefaults(TtlvStructure defaultsInformation) {
        return new SetDefaultsBuilder(client, defaultsInformation);
    }
    public SetConstraintsBuilder setConstraints() { return new SetConstraintsBuilder(client, null); }
    public SetConstraintsBuilder setConstraints(TtlvStructure constraints) {
        return new SetConstraintsBuilder(client, constraints);
    }
    public LogBuilder log() { return new LogBuilder(client, null); }
    public LogBuilder log(String logMessage) {
        byte[] encoded = logMessage == null ? null : logMessage.getBytes(StandardCharsets.UTF_8);
        return new LogBuilder(client, encoded);
    }
    public LogBuilder log(byte[] logMessage) { return new LogBuilder(client, logMessage); }
}
