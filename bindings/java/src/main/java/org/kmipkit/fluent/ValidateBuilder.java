package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Validate request. */
public final class ValidateBuilder extends TypedOperationBuilder<ValidateBuilder> {
    private final List<TtlvStructure> certificates = new ArrayList<>();
    private final List<UniqueIdentifier> identifiers = new ArrayList<>();
    private Long validityDate;

    ValidateBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected ValidateBuilder self() { return this; }

    public ValidateBuilder certificate(TtlvStructure cert) {
        ensureOpen();
        if (cert == null) throw new InvalidInputException("certificate is required");
        certificates.add(own(cert.copy()));
        return this;
    }

    public ValidateBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        if (id == null) throw new InvalidInputException("uniqueIdentifier is required");
        identifiers.add(id);
        return this;
    }

    public ValidateBuilder validityDate(long unixSeconds) {
        ensureOpen();
        validityDate = unixSeconds;
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.validate(certificates, identifiers, validityDate));
    }
}
