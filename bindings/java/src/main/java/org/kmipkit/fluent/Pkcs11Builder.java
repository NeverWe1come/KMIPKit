package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP PKCS#11 request (§6.1.37). */
public final class Pkcs11Builder extends TypedOperationBuilder<Pkcs11Builder> {
    private TtlvStructure function;
    private TtlvStructure interfaceStructure;
    private byte[] correlationValue;
    private TtlvStructure inputParameters;
    private final List<TtlvItem> extensions = new ArrayList<>();

    Pkcs11Builder(KmipClient client, TtlvStructure function) {
        super(client);
        if (function != null) {
            this.function = replace(null, function);
        }
    }

    @Override
    protected Pkcs11Builder self() { return this; }

    public Pkcs11Builder function(TtlvStructure value) {
        if (value == null) throw new InvalidInputException("PKCS#11 function is required");
        function = replace(function, value);
        return this;
    }

    public Pkcs11Builder interfaceStructure(TtlvStructure value) {
        interfaceStructure = replace(interfaceStructure, value);
        return this;
    }

    public Pkcs11Builder correlationValue(byte[] bytes) {
        ensureOpen();
        correlationValue = bytes == null ? null : bytes.clone();
        return this;
    }

    public Pkcs11Builder inputParameters(TtlvStructure value) {
        inputParameters = replace(inputParameters, value);
        return this;
    }

    public Pkcs11Builder extension(TtlvItem item) {
        ensureOpen();
        if (item == null) throw new InvalidInputException("extension item is required");
        extensions.add(item);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        if (function == null) throw new InvalidInputException("PKCS#11 function is required");
        return validated(ClientBatchItem.pkcs11(
                function, interfaceStructure, correlationValue, inputParameters, extensions));
    }
}
