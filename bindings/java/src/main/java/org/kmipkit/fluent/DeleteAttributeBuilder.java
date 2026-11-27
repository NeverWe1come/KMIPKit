package org.kmipkit.fluent;

import java.nio.charset.StandardCharsets;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.DeleteAttributeRequest;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;

/** Rust-backed builder for a typed KMIP Delete Attribute request. */
public final class DeleteAttributeBuilder implements AutoCloseable {
    @FunctionalInterface
    private interface ItemAction { void apply(long builder, long item); }

    private final NativeHandle handle;
    private final KmipClient client;
    private RequestOptions options;
    private Long asynchronousIndicator;
    private UniqueIdentifier uniqueIdentifier;
    private AttributeReference attributeReference;

    DeleteAttributeBuilder(KmipClient client) {
        this.client = client;
        NativeFluentRegistry.ensureLoaded();
        handle = NativeHandle.owned(
                NativeFluentRegistry.deleteAttributeBuilderCreate(),
                NativeFluentRegistry.DELETE_ATTRIBUTE_BUILDER);
    }

    public DeleteAttributeBuilder uniqueIdentifier(String value) {
        if (value == null) throw new InvalidInputException("unique identifier is invalid");
        handle.withValue(builder -> {
            NativeFluentRegistry.deleteAttributeBuilderSetUniqueIdentifier(builder, value);
            return null;
        });
        uniqueIdentifier = UniqueIdentifier.textString(value);
        return this;
    }

    private DeleteAttributeBuilder currentAttributeItem(TtlvItem item) {
        if (item == null) throw new InvalidInputException("Current Attribute is invalid");
        NativeHandle.withValues(new NativeHandle[] {handle, item.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.deleteAttributeBuilderSetCurrentAttributeItem(
                    values[0], values[1]);
            return null;
        });
        return this;
    }

    /** Selects one explicitly typed Current Attribute Item. */
    public DeleteAttributeBuilder currentAttribute(TtlvItem item) {
        return currentAttributeItem(item);
    }

    public DeleteAttributeBuilder cataloguedAttribute(Tag tag, TtlvValue value) {
        if (tag == null || value == null) {
            throw new InvalidInputException("catalogued attribute inputs are invalid");
        }
        return cataloguedAttribute(tag.raw(), value);
    }

    public DeleteAttributeBuilder cataloguedAttribute(
            StandardAttribute attribute, TtlvValue value) {
        if (attribute == null || value == null) {
            throw new InvalidInputException("catalogued attribute inputs are invalid");
        }
        return cataloguedAttribute(attribute.tag(), value);
    }

    private DeleteAttributeBuilder cataloguedAttribute(int tag, TtlvValue value) {
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.deleteAttributeBuilderSetCataloguedAttribute(
                    values[0], tag, values[1]);
            return null;
        });
        return this;
    }

    public DeleteAttributeBuilder vendorAttribute(String vendor, String name, TtlvItem value) {
        if (vendor == null || name == null || value == null) {
            throw new InvalidInputException("vendor attribute inputs are invalid");
        }
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.deleteAttributeBuilderSetVendorAttribute(
                    values[0], vendor, name, values[1]);
            return null;
        });
        return this;
    }

    /** Selects a structurally valid KMIP Vendor Attribute as the Current Attribute. */
    public DeleteAttributeBuilder currentVendorAttribute(
            String vendor, String name, TtlvItem value) {
        return vendorAttribute(vendor, name, value);
    }

    public DeleteAttributeBuilder attributeReference(AttributeReference value) {
        if (value == null) throw new InvalidInputException("Attribute Reference is invalid");
        long tag = value.tagValue().orElse(0);
        byte[] vendor = value.vendorIdentification()
                .map(text -> text.getBytes(StandardCharsets.UTF_8)).orElse(null);
        byte[] name = value.attributeName()
                .map(text -> text.getBytes(StandardCharsets.UTF_8)).orElse(null);
        int kind = vendor == null ? 1 : 2;
        handle.withValue(builder -> {
            NativeFluentRegistry.deleteAttributeBuilderSetAttributeReference(
                    builder, kind, tag, vendor, name);
            return null;
        });
        attributeReference = value;
        return this;
    }

    public DeleteAttributeBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }

    public DeleteAttributeBuilder asynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        handle.withValue(builder -> {
            NativeFluentRegistry.deleteAttributeBuilderSetAsynchronousIndicator(builder, value);
            return null;
        });
        asynchronousIndicator = value;
        return this;
    }

    public void validate() {
        handle.withValue(builder -> {
            NativeFluentRegistry.deleteAttributeBuilderValidate(builder);
            return null;
        });
    }

    /** Builds an independently owned typed Delete Attribute request. */
    public DeleteAttributeRequest buildRequest() {
        return handle.withValue(builder -> NativeFluentRegistry.deleteAttributeBuilderBuildRequest(
                builder, uniqueIdentifier, attributeReference));
    }

    public ClientBatchItem build() {
        long item = handle.withValue(NativeFluentRegistry::deleteAttributeBuilderBuildBatchItem);
        return FluentBatchItemAdapter.takeOwnership(item);
    }

    public ClientBatchItem toBatchItem() { return build(); }

    /** Executes Delete Attribute with client limits and throws on server error or Pending. */
    public ClientBatchItemResult execute() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeResult(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Delete Attribute with client limits, returning raw outcomes. */
    public ClientBatchItemResult executeOutcome() throws org.kmipkit.ClientOperationException {
        return BatchExecution.executeOutcome(client, build(), null, options, asynchronousIndicator);
    }

    /** Executes Delete Attribute with explicit limits, returning raw outcomes. */
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
