package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.AttributeSet;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.FluentBatchItemAdapter;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Rust-backed builder for a typed, generic KMIP Create request. */
public final class CreateBuilder implements AutoCloseable {
    private final NativeHandle handle;
    private final KmipClient client;
    private Long objectType;
    private RequestOptions options;
    private Long asynchronousIndicator;

    CreateBuilder(KmipClient client) {
        this.client = client;
        NativeFluentRegistry.ensureLoaded();
        handle = NativeHandle.owned(
                NativeFluentRegistry.createBuilderCreate(), NativeFluentRegistry.CREATE_BUILDER);
    }

    /** Selects the explicit KMIP Object Type enumeration value. */
    public CreateBuilder objectType(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("Object Type is outside uint32");
        }
        handle.withValue(builder -> {
            NativeFluentRegistry.createBuilderSetObjectType(builder, value);
            return null;
        });
        objectType = value;
        return this;
    }

    /** Copies direct ordered Attributes from the supplied collection. */
    public CreateBuilder attributes(AttributeSet value) {
        if (value == null) throw new InvalidInputException("Create attributes are invalid");
        try (TtlvStructure items = value.asItems()) {
            NativeHandle.withValues(new NativeHandle[] {handle, items.nativeHandleOwner()}, values -> {
                NativeFluentRegistry.createBuilderSetAttributes(values[0], values[1]);
                return null;
            });
        }
        return this;
    }

    /** Copies the optional generic Protection Storage Masks Structure. */
    public CreateBuilder protectionStorageMasks(TtlvStructure value) {
        if (value == null) {
            handle.withValue(builder -> {
                NativeFluentRegistry.createBuilderSetProtectionStorageMasks(builder, 0);
                return null;
            });
            return this;
        }
        NativeHandle.withValues(new NativeHandle[] {handle, value.nativeHandleOwner()}, values -> {
            NativeFluentRegistry.createBuilderSetProtectionStorageMasks(values[0], values[1]);
            return null;
        });
        return this;
    }

    /** Sets the explicit Request Header Asynchronous Indicator for execution. */
    public CreateBuilder withAsynchronousIndicator(long value) {
        if (value < 0 || value > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        handle.withValue(builder -> {
            NativeFluentRegistry.createBuilderSetAsynchronousIndicator(builder, value);
            return null;
        });
        asynchronousIndicator = value;
        return this;
    }

    public CreateBuilder withOptions(RequestOptions value) {
        if (value == null) throw new InvalidInputException("request options are invalid");
        options = value;
        return this;
    }

    /** Validates the configured request without sending it. */
    public void validate() {
        if (objectType == null) throw new InvalidInputException("Create Object Type is required");
        handle.withValue(builder -> {
            NativeFluentRegistry.createBuilderValidate(builder);
            return null;
        });
    }

    /** Builds an independently owned typed Create batch item. */
    public ClientBatchItem build() {
        validate();
        long item = handle.withValue(NativeFluentRegistry::createBuilderBuildBatchItem);
        return FluentBatchItemAdapter.takeOwnership(item);
    }

    /** Alias for {@link #build()}. */
    public ClientBatchItem toBatchItem() { return build(); }

    /** Executes Create with the default codec limits and returns its owned result. */
    public ClientBatchItemResult executeOne() throws org.kmipkit.ClientOperationException {
        return executeOne(CodecLimits.defaults());
    }

    /** Returns the owned Create result, including rejection or Pending outcomes. */
    public ClientBatchItemResult executeOne(CodecLimits limits)
            throws org.kmipkit.ClientOperationException {
        try (ClientBatchResponse response = execute(limits)) {
            return response.itemAt(0);
        }
    }

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
