package org.kmipkit.fluent;

import org.kmipkit.protocol.KeyUsage;
import org.kmipkit.InvalidInputException;
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

/** Rust-backed builder for a typed KMIP Create Key Pair request. */
public final class CreateKeyPairBuilder implements AutoCloseable {
    @FunctionalInterface
    private interface ItemAction {
        void apply(long builder, long item);
    }

    @FunctionalInterface
    private interface CataloguedAction {
        void apply(long builder, int tag, long value);
    }

    @FunctionalInterface
    private interface VendorAction {
        void apply(long builder, String vendor, String name, long value);
    }

    private final NativeHandle handle;
    private final KmipClient client;
    private RequestOptions options;
    private Long asynchronousIndicator;

    CreateKeyPairBuilder(KmipClient client) {
        this.client = client;
        NativeFluentRegistry.ensureLoaded();
        handle = NativeHandle.owned(
                NativeFluentRegistry.createKeyPairBuilderCreate(),
                NativeFluentRegistry.CREATE_KEY_PAIR_BUILDER);
        if (client != null) {
            withProtocolVersion(client.protocolVersion());
        }
    }

    /** Sets the target KMIP protocol version for attribute emission (`Name` vs `AlternativeName`). */
    public CreateKeyPairBuilder withProtocolVersion(ProtocolVersion version) {
        if (version == null) throw new InvalidInputException("protocol version is invalid");
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetProtocolVersion(
                    builder, version.major(), version.minor());
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder algorithm(long value) {
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetAlgorithm(builder, value);
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder length(int bits) {
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetLength(builder, bits);
            return null;
        });
        return this;
    }

    /** Configures asymmetric or KMIP 3.0 PQC algorithm and length from a curated {@link KeySpec}. */
    public CreateKeyPairBuilder withKeySpec(KeySpec spec) {
        if (spec == null || !spec.isAsymmetric()) {
            throw new InvalidInputException("an asymmetric KeySpec is required");
        }
        return algorithm(spec.algorithm()).length(spec.length());
    }

    /** Selects named KMIP key usage flags. */
    public CreateKeyPairBuilder privateUsageMask(KeyUsage... flags) {
        return privateUsageMask(KeyUsage.mask(flags));
    }

    public CreateKeyPairBuilder privateUsageMask(int mask) {
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetPrivateUsageMask(builder, mask);
            return null;
        });
        return this;
    }

    /** Selects exact 32-bit private Cryptographic Usage Mask bits. */
    public CreateKeyPairBuilder privateUsageMask(long mask) {
        if (mask < Integer.MIN_VALUE || mask > 0xffff_ffffL) {
            throw new InvalidInputException("cryptographic usage mask is outside 32-bit range");
        }
        return privateUsageMask((int) mask);
    }

    /** Selects named KMIP key usage flags. */
    public CreateKeyPairBuilder publicUsageMask(KeyUsage... flags) {
        return publicUsageMask(KeyUsage.mask(flags));
    }

    public CreateKeyPairBuilder publicUsageMask(int mask) {
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetPublicUsageMask(builder, mask);
            return null;
        });
        return this;
    }

    /** Selects exact 32-bit public Cryptographic Usage Mask bits. */
    public CreateKeyPairBuilder publicUsageMask(long mask) {
        if (mask < Integer.MIN_VALUE || mask > 0xffff_ffffL) {
            throw new InvalidInputException("cryptographic usage mask is outside 32-bit range");
        }
        return publicUsageMask((int) mask);
    }

    public CreateKeyPairBuilder privateName(String value) {
        if (value == null) throw new InvalidInputException("private key name is invalid");
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetPrivateName(builder, value);
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder publicName(String value) {
        if (value == null) throw new InvalidInputException("public key name is invalid");
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetPublicName(builder, value);
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder commonAttributeItem(TtlvItem item) {
        return withAttributeItem(item, NativeFluentRegistry::createKeyPairBuilderAddCommonAttributeItem);
    }

    public CreateKeyPairBuilder privateAttributeItem(TtlvItem item) {
        return withAttributeItem(item, NativeFluentRegistry::createKeyPairBuilderAddPrivateAttributeItem);
    }

    public CreateKeyPairBuilder publicAttributeItem(TtlvItem item) {
        return withAttributeItem(item, NativeFluentRegistry::createKeyPairBuilderAddPublicAttributeItem);
    }

    private CreateKeyPairBuilder withAttributeItem(TtlvItem item, ItemAction action) {
        if (item == null) throw new InvalidInputException("attribute item is invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
            action.apply(values[0], values[1]);
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder commonCataloguedAttribute(Tag tag, TtlvValue value) {
        return withCataloguedAttribute(tag, value,
                NativeFluentRegistry::createKeyPairBuilderAddCommonCataloguedAttribute);
    }

    public CreateKeyPairBuilder privateCataloguedAttribute(Tag tag, TtlvValue value) {
        return withCataloguedAttribute(tag, value,
                NativeFluentRegistry::createKeyPairBuilderAddPrivateCataloguedAttribute);
    }

    public CreateKeyPairBuilder publicCataloguedAttribute(Tag tag, TtlvValue value) {
        return withCataloguedAttribute(tag, value,
                NativeFluentRegistry::createKeyPairBuilderAddPublicCataloguedAttribute);
    }

    private CreateKeyPairBuilder withCataloguedAttribute(
            Tag tag, TtlvValue value, CataloguedAction action) {
        if (tag == null || value == null) {
            throw new InvalidInputException("catalogued attribute inputs are invalid");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            action.apply(values[0], tag.raw(), values[1]);
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder commonVendorAttribute(String vendor, String name, TtlvItem value) {
        return withVendorAttribute(vendor, name, value,
                NativeFluentRegistry::createKeyPairBuilderAddCommonVendorAttribute);
    }

    public CreateKeyPairBuilder privateVendorAttribute(String vendor, String name, TtlvItem value) {
        return withVendorAttribute(vendor, name, value,
                NativeFluentRegistry::createKeyPairBuilderAddPrivateVendorAttribute);
    }

    public CreateKeyPairBuilder publicVendorAttribute(String vendor, String name, TtlvItem value) {
        return withVendorAttribute(vendor, name, value,
                NativeFluentRegistry::createKeyPairBuilderAddPublicVendorAttribute);
    }

    private CreateKeyPairBuilder withVendorAttribute(
            String vendor, String name, TtlvItem value, VendorAction action) {
        if (vendor == null || name == null || value == null) {
            throw new InvalidInputException("vendor attribute inputs are invalid");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            action.apply(values[0], vendor, name, values[1]);
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }

    public CreateKeyPairBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        asynchronousIndicator = value;
        return this;
    }

    public void validate() {
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderValidate(builder);
            return null;
        });
    }

    public ClientBatchItem build() {
        long item = handle.withValue(NativeFluentRegistry::createKeyPairBuilderBuildBatchItem);
        return FluentBatchItemAdapter.takeOwnership(item);
    }

    public ClientBatchItem toBatchItem() { return build(); }

    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return BatchExecution.execute(client, build(), limits, options, asynchronousIndicator);
    }

    /** Executes Create Key Pair with configured limits and throws on server error or Pending. */
    public ClientBatchItemResult execute() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeResult(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Create Key Pair with configured limits, returning rejection or Pending outcomes without throwing. */
    public ClientBatchItemResult executeOutcome() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeOutcome(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Create Key Pair with explicit limits, returning rejection or Pending outcomes without throwing. */
    public ClientBatchItemResult executeOutcome(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        return BatchExecution.executeOutcome(client, build(), limits, options, asynchronousIndicator);
    }

    @Override public void close() { handle.close(); }
}
