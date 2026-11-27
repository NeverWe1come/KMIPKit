package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Set Constraints request (§6.1.52). */
public final class SetConstraintsBuilder extends TypedOperationBuilder<SetConstraintsBuilder> {
    private TtlvStructure constraints;

    SetConstraintsBuilder(KmipClient client, TtlvStructure constraints) {
        super(client);
        if (constraints != null) {
            this.constraints = replace(null, constraints);
        }
    }

    @Override
    protected SetConstraintsBuilder self() { return this; }

    public SetConstraintsBuilder constraints(TtlvStructure value) {
        if (value == null) throw new InvalidInputException("constraints is required");
        constraints = replace(constraints, value);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (constraints == null) {
            throw new InvalidInputException("Set Constraints requires constraints");
        }
        return validated(ClientBatchItem.setConstraints(constraints));
    }
}
