package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.TtlvStructure;
import org.jspecify.annotations.Nullable;

/** Builds a Get request with the full set of optional KMIP selectors. */
public final class GetBuilder extends TypedOperationBuilder<GetBuilder> {
    private @Nullable UniqueIdentifier identifier;
    private @Nullable Long format;
    private @Nullable Long wrap;
    private @Nullable Long compression;
    private @Nullable TtlvStructure wrapping;

    GetBuilder(KmipClient client) {
        super(client);
    }

    public GetBuilder uniqueIdentifier(UniqueIdentifier value) {
        ensureOpen();
        identifier = value;
        return this;
    }

    public GetBuilder keyFormatType(long value) {
        ensureOpen();
        format = unsigned32(value, "Key Format Type");
        return this;
    }

    public GetBuilder keyWrapType(long value) {
        ensureOpen();
        wrap = unsigned32(value, "Key Wrap Type");
        return this;
    }

    public GetBuilder keyCompressionType(long value) {
        ensureOpen();
        compression = unsigned32(value, "Key Compression Type");
        return this;
    }

    public GetBuilder keyWrappingSpecification(TtlvStructure value) {
        wrapping = replace(wrapping, value);
        return this;
    }

    @Override
    protected GetBuilder self() {
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        return validated(ClientBatchItem.get(identifier, format, wrap, compression, wrapping));
    }
}
