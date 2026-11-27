package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;

/** Fluent builder for a typed KMIP Export request. */
public final class ExportBuilder extends TypedOperationBuilder<ExportBuilder> {
    private UniqueIdentifier uniqueIdentifier;
    private Long keyFormatType;
    private Long keyWrapType;
    private Long keyCompressionType;
    private TtlvStructure keyWrappingSpecification;

    ExportBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected ExportBuilder self() { return this; }

    public ExportBuilder uniqueIdentifier(UniqueIdentifier id) {
        ensureOpen();
        uniqueIdentifier = id;
        return this;
    }

    public ExportBuilder keyFormatType(long value) {
        ensureOpen();
        keyFormatType = unsigned32(value, "Key Format Type");
        return this;
    }

    public ExportBuilder keyWrapType(long value) {
        ensureOpen();
        keyWrapType = unsigned32(value, "Key Wrap Type");
        return this;
    }

    public ExportBuilder keyCompressionType(long value) {
        ensureOpen();
        keyCompressionType = unsigned32(value, "Key Compression Type");
        return this;
    }

    public ExportBuilder keyWrappingSpecification(TtlvStructure spec) {
        keyWrappingSpecification = replace(keyWrappingSpecification, spec);
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.export(
                uniqueIdentifier, keyFormatType, keyWrapType, keyCompressionType, keyWrappingSpecification));
    }
}
