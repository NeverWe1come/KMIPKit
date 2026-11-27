package org.kmipkit.fluent;

import org.kmipkit.protocol.KeyUsage;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

/** Rust-backed builder for a typed KMIP Create Symmetric Key request. */
public final class SymmetricKeyBuilder implements AutoCloseable {
    @FunctionalInterface
    private interface AttributeItemAction {
        void apply(long builder, long item);
    }

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
        if (client != null) {
            withProtocolVersion(client.protocolVersion());
        }
    }

    /** Sets the target KMIP protocol version for attribute emission (`Name` vs `AlternativeName`). */
    public SymmetricKeyBuilder withProtocolVersion(ProtocolVersion version) {
        if (version == null) throw new InvalidInputException("protocol version is invalid");
        handle.withValue(builder -> {
            NativeExtensionRegistry.symmetricKeyBuilderSetProtocolVersion(
                    builder, version.major(), version.minor());
            return null;
        });
        return this;
    }

    public SymmetricKeyBuilder algorithm(long value) {
        handle.withValue(builder -> { NativeExtensionRegistry.symmetricKeyBuilderSetAlgorithm(builder, value); return null; });
        return this;
    }

    /** Selects AES with a 256-bit key; the caller must still choose a usage mask. */
    public SymmetricKeyBuilder aes256() {
        return algorithm(org.kmipkit.protocol.KmipValues.CryptographicAlgorithm.AES).length(256);
    }

    /** Configures algorithm and bit length from a curated symmetric {@link KeySpec}. */
    public SymmetricKeyBuilder withKeySpec(KeySpec spec) {
        if (spec == null || !spec.isSymmetric()) {
            throw new InvalidInputException("a symmetric KeySpec is required");
        }
        return algorithm(spec.algorithm()).length(spec.length());
    }

    public SymmetricKeyBuilder length(int bits) {
        handle.withValue(builder -> { NativeExtensionRegistry.symmetricKeyBuilderSetLength(builder, bits); return null; });
        return this;
    }

    /** Selects named KMIP key usage flags. */
    public SymmetricKeyBuilder usageMask(KeyUsage... flags) {
        return usageMask(KeyUsage.mask(flags));
    }

    public SymmetricKeyBuilder usageMask(int mask) {
        handle.withValue(builder -> { NativeExtensionRegistry.symmetricKeyBuilderSetUsageMask(builder, mask); return null; });
        return this;
    }

    /** Selects exact 32-bit Cryptographic Usage Mask bits (including values above 0x7fffffff). */
    public SymmetricKeyBuilder usageMask(long mask) {
        if (mask < Integer.MIN_VALUE || mask > 0xffff_ffffL) {
            throw new InvalidInputException("cryptographic usage mask is outside 32-bit range");
        }
        return usageMask((int) mask);
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
        return withAttributeItem(item, NativeExtensionRegistry::symmetricKeyBuilderAddAttributeItem);
    }

    private SymmetricKeyBuilder withAttributeItem(TtlvItem item, AttributeItemAction action) {
        if (item == null) throw new InvalidInputException("attribute item is invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
            action.apply(values[0], values[1]);
            return null;
        });
        return this;
    }

    /** Inserts an explicit expert item at its zero-based position in final Object Attributes. */
    public SymmetricKeyBuilder attributeItemAt(int index, TtlvItem item) {
        if (index < 0) throw new InvalidInputException("attribute position is invalid");
        return withAttributeItem(item, (builder, rawItem) ->
                NativeFluentRegistry.symmetricKeyBuilderInsertAttributeItem(
                        builder, index, rawItem));
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

    /** Executes Create with configured limits and throws on server error or Pending. */
    public ClientBatchItemResult execute() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeResult(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Create with configured limits, returning rejection or Pending outcomes without throwing. */
    public ClientBatchItemResult executeOutcome() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeOutcome(client, build(), null, options, asynchronousIndicator);
    }

    /** Returns the owned Create result with explicit limits, including rejection or Pending outcomes. */
    public ClientBatchItemResult executeOutcome(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        return BatchExecution.executeOutcome(client, build(), limits, options, asynchronousIndicator);
    }

    /** Executes Create with the default codec limits and returns its owned result. */
    public ClientBatchItemResult executeOne() throws org.kmipkit.ClientOperationException {
        return executeOutcome();
    }

    /** Returns the owned Create result, including rejection or Pending outcomes. */
    public ClientBatchItemResult executeOne(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return executeOutcome(limits);
    }

    public ClientBatchResponse execute(CodecLimits limits) throws org.kmipkit.ClientOperationException {
        return BatchExecution.execute(client, build(), limits, options, asynchronousIndicator);
    }

    @Override public void close() { handle.close(); }
}
