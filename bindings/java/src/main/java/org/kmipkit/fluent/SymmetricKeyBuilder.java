package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

/** Rust-backed builder for a typed KMIP Create Symmetric Key request. */
public final class SymmetricKeyBuilder implements AutoCloseable {
    private final NativeHandle handle;
    private final KmipClient client;
    private RequestOptions options;
    private Long asynchronousIndicator;

    SymmetricKeyBuilder(KmipClient client) {
        this.client = client;
        NativeExtensionRegistry.ensureLoaded();
        handle = NativeHandle.owned(
                NativeExtensionRegistry.symmetricKeyBuilderCreate(),
                NativeExtensionRegistry.SYMMETRIC_KEY_BUILDER);
    }

    public SymmetricKeyBuilder algorithm(long value) {
        handle.withValue(builder -> { NativeExtensionRegistry.symmetricKeyBuilderSetAlgorithm(builder, value); return null; });
        return this;
    }

    /** Selects AES with a 256-bit key; the caller must still choose a usage mask. */
    public SymmetricKeyBuilder aes256() {
        return algorithm(org.kmipkit.protocol.KmipValues.CryptographicAlgorithm.AES).length(256);
    }

    public SymmetricKeyBuilder length(int bits) {
        handle.withValue(builder -> { NativeExtensionRegistry.symmetricKeyBuilderSetLength(builder, bits); return null; });
        return this;
    }

    public SymmetricKeyBuilder usageMask(int mask) {
        handle.withValue(builder -> { NativeExtensionRegistry.symmetricKeyBuilderSetUsageMask(builder, mask); return null; });
        return this;
    }

    public SymmetricKeyBuilder name(String value) {
        if (value == null) throw new InvalidInputException("symmetric key name is invalid");
        handle.withValue(builder -> { NativeExtensionRegistry.symmetricKeyBuilderSetName(builder, value); return null; });
        return this;
    }

    public SymmetricKeyBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }

    /** Sets the explicit Request Header Asynchronous Indicator on the containing batch. */
    public SymmetricKeyBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }

    /** Adds a catalogued attribute with its explicit TTLV Value for Rust-side policy checks. */
    public SymmetricKeyBuilder cataloguedAttribute(Tag tag, TtlvValue value) {
        if (tag == null || value == null) throw new InvalidInputException("attribute inputs are invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeExtensionRegistry.symmetricKeyBuilderAddCataloguedAttribute(
                    values[0], tag.raw(), values[1]);
            return null;
        });
        return this;
    }

    /** Adds an explicit generic item; Rust validates known tags and rejects message extensions. */
    public SymmetricKeyBuilder attributeItem(TtlvItem item) {
        if (item == null) throw new InvalidInputException("attribute item is invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
            NativeExtensionRegistry.symmetricKeyBuilderAddAttributeItem(values[0], values[1]);
            return null;
        });
        return this;
    }

    /** Adds a Vendor Attribute whose required structure is assembled and checked in Rust. */
    public SymmetricKeyBuilder vendorAttribute(String vendor, String attributeName, TtlvItem value) {
        if (vendor == null || attributeName == null || value == null) {
            throw new InvalidInputException("vendor attribute inputs are invalid");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeExtensionRegistry.symmetricKeyBuilderAddVendorAttribute(
                    values[0], vendor, attributeName, values[1]);
            return null;
        });
        return this;
    }

    /** Builds the existing typed batch item without consuming this builder. */
    public ClientBatchItem build() {
        long item = handle.withValue(NativeExtensionRegistry::symmetricKeyBuilderBuildBatchItem);
        return FluentBatchItemAdapter.takeOwnership(item);
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }

    public ClientBatchResponse execute(CodecLimits limits) throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        if (client == null) throw new InvalidInputException("client is required to execute this builder");
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) batch.asynchronousIndicator(asynchronousIndicator);
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }

    @Override public void close() { handle.close(); }
}
