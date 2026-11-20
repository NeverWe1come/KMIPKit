package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.ValidationReport;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Common execution and ownership for the typed lifecycle builders. */
abstract class TypedOperationBuilder<B extends TypedOperationBuilder<B>> implements AutoCloseable {
    private final KmipClient client;
    private final List<AutoCloseable> owned = new ArrayList<>();
    private RequestOptions options;
    private Long asynchronousIndicator;
    private boolean closed;

    TypedOperationBuilder(KmipClient client) {
        this.client = client;
    }

    protected abstract B self();
    public abstract ClientBatchItem build();

    protected final void ensureOpen() {
        if (closed) throw new InvalidInputException("builder is closed");
    }

    protected static long unsigned32(long value, String field) {
        if (value < 0 || value > 0xffff_ffffL) throw new InvalidInputException(field + " is outside uint32");
        return value;
    }

    protected final <T extends AutoCloseable> T own(T resource) {
        ensureOpen();
        owned.add(resource);
        return resource;
    }

    protected final void discard(AutoCloseable resource) {
        if (resource == null) return;
        owned.remove(resource);
        try { resource.close(); }
        catch (RuntimeException error) { throw error; }
        catch (Exception error) { throw new IllegalStateException(error); }
    }

    protected final TtlvStructure replace(TtlvStructure previous, TtlvStructure value) {
        ensureOpen();
        TtlvStructure copy = value == null ? null : value.copy();
        discard(previous);
        return copy == null ? null : own(copy);
    }

    public final B withOptions(RequestOptions value) {
        ensureOpen();
        if (value == null) throw new InvalidInputException("request options are required");
        options = value;
        return self();
    }

    public final B asynchronousIndicator(long value) {
        ensureOpen();
        asynchronousIndicator = unsigned32(value, "Asynchronous Indicator");
        return self();
    }

    protected final ClientBatchItem validated(ClientBatchItem item) {
        try (ValidationReport report = item.validationReport()) {
            if (!report.isValid()) throw new InvalidInputException("typed request validation failed");
            return item;
        } catch (RuntimeException | Error error) {
            item.close();
            throw error;
        }
    }

    public final void validate() {
        try (ClientBatchItem ignored = build()) { }
    }

    public final ClientBatchItem toBatchItem() {
        return build();
    }

    public final ClientBatchResponse execute(CodecLimits limits) throws org.kmipkit.ClientOperationException {
        ensureOpen();
        if (limits == null) throw new InvalidInputException("codec limits are required");
        if (client == null) throw new InvalidInputException("client is required to execute");
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(build());
            if (asynchronousIndicator != null) batch.asynchronousIndicator(asynchronousIndicator);
            return options == null ? client.execute(batch, limits) : client.executeWithOptions(batch, limits, options);
        }
    }

    public final ClientBatchResponse execute(CodecLimits limits, RequestOptions value)
            throws org.kmipkit.ClientOperationException {
        withOptions(value);
        return execute(limits);
    }

    public final ClientBatchResponse executeWithOptions(CodecLimits limits, RequestOptions value)
            throws org.kmipkit.ClientOperationException {
                return execute(limits, value);
            }

    @Override
    public void close() {
        if (closed) return;
        closed = true;
        RuntimeException failure = null;
        for (int index = owned.size() - 1; index >= 0; index--) {
            try {
                owned.get(index).close();
            } catch (Exception error) {
                RuntimeException wrapped = error instanceof RuntimeException
                        ? (RuntimeException) error : new IllegalStateException(error);
                if (failure == null) failure = wrapped;
                else failure.addSuppressed(wrapped);
            }
        }
        owned.clear();
        if (failure != null) throw failure;
    }
}
