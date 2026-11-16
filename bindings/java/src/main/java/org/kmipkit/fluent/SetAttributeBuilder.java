package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.SetAttributeRequest;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

/** Rust-backed builder for a typed KMIP Set Attribute request. */
public final class SetAttributeBuilder implements AutoCloseable {
    @FunctionalInterface
    private interface ItemAction {
        void apply(long builder, long item);
    }

    @FunctionalInterface
    private interface CataloguedAction {
        void apply(long builder, int tag, long value);
    }

    private final NativeHandle handle;
    private final KmipClient client;
    private RequestOptions options;
    private Long asynchronousIndicator;
    private UniqueIdentifier uniqueIdentifier;

    SetAttributeBuilder(KmipClient client) {
        this.client = client;
        NativeFluentRegistry.ensureLoaded();
        handle = NativeHandle.owned(
                NativeFluentRegistry.setAttributeBuilderCreate(),
                NativeFluentRegistry.SET_ATTRIBUTE_BUILDER);
    }

    public SetAttributeBuilder uniqueIdentifier(String value) {
        if (value == null) throw new InvalidInputException("unique identifier is invalid");
        handle.withValue(builder -> {
            NativeFluentRegistry.setAttributeBuilderSetUniqueIdentifier(builder, value);
            return null;
        });
        uniqueIdentifier = UniqueIdentifier.textString(value);
        return this;
    }

    public SetAttributeBuilder attributeItem(TtlvItem item) {
        return withAttributeItem(item, NativeFluentRegistry::setAttributeBuilderAddAttributeItem);
    }

    private SetAttributeBuilder withAttributeItem(TtlvItem item, ItemAction action) {
        if (item == null) throw new InvalidInputException("attribute item is invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
            action.apply(values[0], values[1]);
            return null;
        });
        return this;
    }

    public SetAttributeBuilder cataloguedAttribute(Tag tag, TtlvValue value) {
        if (tag == null || value == null) {
            throw new InvalidInputException("catalogued attribute inputs are invalid");
        }
        return cataloguedAttribute(tag.raw(), value);
    }

    /** Sets one standard catalogued attribute without requiring a raw KMIP tag. */
    public SetAttributeBuilder cataloguedAttribute(StandardAttribute attribute, TtlvValue value) {
        if (attribute == null || value == null) {
            throw new InvalidInputException("catalogued attribute inputs are invalid");
        }
        return cataloguedAttribute(attribute.tag(), value);
    }

    private SetAttributeBuilder cataloguedAttribute(int tag, TtlvValue value) {
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.setAttributeBuilderAddCataloguedAttribute(
                    values[0], tag, values[1]);
            return null;
        });
        return this;
    }

    public SetAttributeBuilder vendorAttribute(String vendor, String name, TtlvItem value) {
        if (vendor == null || name == null || value == null) {
            throw new InvalidInputException("vendor attribute inputs are invalid");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.setAttributeBuilderAddVendorAttribute(
                    values[0], vendor, name, values[1]);
            return null;
        });
        return this;
    }

    public SetAttributeBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }

    public SetAttributeBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        handle.withValue(builder -> {
            NativeFluentRegistry.setAttributeBuilderSetAsynchronousIndicator(builder, value);
            return null;
        });
        asynchronousIndicator = value;
        return this;
    }

    public void validate() {
        handle.withValue(builder -> {
            NativeFluentRegistry.setAttributeBuilderValidate(builder);
            return null;
        });
    }

    /** Builds an independently owned typed Set Attribute request. */
    public SetAttributeRequest buildRequest() {
        return handle.withValue(builder ->
                NativeFluentRegistry.setAttributeBuilderBuildRequest(builder, uniqueIdentifier));
    }

    public ClientBatchItem build() {
        long item = handle.withValue(NativeFluentRegistry::setAttributeBuilderBuildBatchItem);
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
