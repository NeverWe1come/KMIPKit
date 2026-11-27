package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Set Defaults request (§6.1.53). */
public final class SetDefaultsBuilder extends TypedOperationBuilder<SetDefaultsBuilder> {
    private TtlvStructure defaultsInformation;

    SetDefaultsBuilder(KmipClient client, TtlvStructure defaultsInformation) {
        super(client);
        if (defaultsInformation != null) {
            this.defaultsInformation = replace(null, defaultsInformation);
        }
    }

    @Override
    protected SetDefaultsBuilder self() { return this; }

    public SetDefaultsBuilder defaultsInformation(TtlvStructure value) {
        defaultsInformation = replace(defaultsInformation, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.setDefaults(defaultsInformation));
    }
}
