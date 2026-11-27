package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.AddAttributeRequest;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

/** Rust-backed builder for a typed KMIP Add Attribute request. */
public final class AddAttributeBuilder implements AutoCloseable {
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

    AddAttributeBuilder(KmipClient client) {
        this.client = client;
        NativeFluentRegistry.ensureLoaded();
        handle = NativeHandle.owned(
                NativeFluentRegistry.addAttributeBuilderCreate(),
                NativeFluentRegistry.ADD_ATTRIBUTE_BUILDER);
    }

    public AddAttributeBuilder uniqueIdentifier(String value) {
        if (value == null) throw new InvalidInputException("unique identifier is invalid");
        handle.withValue(builder -> {
            NativeFluentRegistry.addAttributeBuilderSetUniqueIdentifier(builder, value);
            return null;
        });
        uniqueIdentifier = UniqueIdentifier.textString(value);
        return this;
    }

    public AddAttributeBuilder attributeItem(TtlvItem item) {
        return withAttributeItem(item, NativeFluentRegistry::addAttributeBuilderAddAttributeItem);
    }

    private AddAttributeBuilder withAttributeItem(TtlvItem item, ItemAction action) {
        if (item == null) throw new InvalidInputException("attribute item is invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
            action.apply(values[0], values[1]);
            return null;
        });
        return this;
    }

    public AddAttributeBuilder cataloguedAttribute(Tag tag, TtlvValue value) {
        if (tag == null || value == null) {
            throw new InvalidInputException("catalogued attribute inputs are invalid");
        }
        return cataloguedAttribute(tag.raw(), value);
    }

    /** Sets one standard catalogued attribute without requiring a raw KMIP tag. */
    public AddAttributeBuilder cataloguedAttribute(StandardAttribute attribute, TtlvValue value) {
        if (attribute == null || value == null) {
            throw new InvalidInputException("catalogued attribute inputs are invalid");
        }
        return cataloguedAttribute(attribute.tag(), value);
    }

    private AddAttributeBuilder cataloguedAttribute(int tag, TtlvValue value) {
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.addAttributeBuilderAddCataloguedAttribute(
                    values[0], tag, values[1]);
            return null;
        });
        return this;
    }

    public AddAttributeBuilder vendorAttribute(String vendor, String name, TtlvItem value) {
        if (vendor == null || name == null || value == null) {
            throw new InvalidInputException("vendor attribute inputs are invalid");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.addAttributeBuilderAddVendorAttribute(
                    values[0], vendor, name, values[1]);
            return null;
        });
        return this;
    }

    public AddAttributeBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }

    public AddAttributeBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        handle.withValue(builder -> {
            NativeFluentRegistry.addAttributeBuilderSetAsynchronousIndicator(builder, value);
            return null;
        });
        asynchronousIndicator = value;
        return this;
    }

    public void validate() {
        handle.withValue(builder -> {
            NativeFluentRegistry.addAttributeBuilderValidate(builder);
            return null;
        });
    }

    /** Builds an independently owned typed Add Attribute request. */
    public AddAttributeRequest buildRequest() {
        return handle.withValue(builder ->
                NativeFluentRegistry.addAttributeBuilderBuildRequest(builder, uniqueIdentifier));
    }

    public ClientBatchItem build() {
        long item = handle.withValue(NativeFluentRegistry::addAttributeBuilderBuildBatchItem);
        return FluentBatchItemAdapter.takeOwnership(item);
    }

    public ClientBatchItem toBatchItem() { return build(); }

    /** Executes Add Attribute with client limits and throws on server error or Pending. */
    public ClientBatchItemResult execute() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeResult(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Add Attribute with client limits, returning raw outcomes. */
    public ClientBatchItemResult executeOutcome() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeOutcome(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Add Attribute with explicit limits, returning raw outcomes. */
    public ClientBatchItemResult executeOutcome(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        if (limits == null) throw new InvalidInputException("codec limits are invalid");
        return BatchExecution.executeOutcome(client, build(), limits, options, asynchronousIndicator);
    }

    public ClientBatchResponse execute(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        return BatchExecution.execute(client, build(), limits, options, asynchronousIndicator);
    }

    @Override public void close() { handle.close(); }
}
