package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
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

    public CreateKeyPairBuilder privateUsageMask(int mask) {
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetPrivateUsageMask(builder, mask);
            return null;
        });
        return this;
    }

    public CreateKeyPairBuilder publicUsageMask(int mask) {
        handle.withValue(builder -> {
            NativeFluentRegistry.createKeyPairBuilderSetPublicUsageMask(builder, mask);
            return null;
        });
        return this;
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
