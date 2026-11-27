package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.credentials.Authentication;

/** Fluent multi-operation batch builder with ID Placeholder ordering and header controls. */
public final class FluentBatchBuilder implements AutoCloseable {
    private final KmipClient client;
    private final ClientBatch batch;
    private RequestOptions options;

    public FluentBatchBuilder(KmipClient client) {
        this.client = client;
        this.batch = ClientBatch.create();
    }

    public FluentBatchBuilder add(ClientBatchItem item) {
        if (item == null) throw new InvalidInputException("batch item is required");
        batch.add(item);
        return this;
    }

    public FluentBatchBuilder batchOrder(boolean ordered) {
        batch.ordered(ordered);
        return this;
    }

    public FluentBatchBuilder ordered() {
        return batchOrder(true);
    }

    public FluentBatchBuilder authentication(Authentication authentication) {
        batch.setAuthentication(authentication);
        return this;
    }

    public FluentBatchBuilder errorContinuation(long value) {
        batch.addErrorContinuationValue(value);
        return this;
    }

    public FluentBatchBuilder asynchronousIndicator(long value) {
        batch.asynchronousIndicator(value);
        return this;
    }

    public FluentBatchBuilder withOptions(RequestOptions options) {
        this.options = options;
        return this;
    }

    public ClientBatch build() {
        return batch;
    }

    public ClientBatchResponse execute() throws org.kmipkit.ClientOperationException {
        if (client == null) throw new InvalidInputException("client is required for execution");
        return options == null
                ? client.execute(batch, client.codecLimits())
                : client.executeWithOptions(batch, client.codecLimits(), options);
    }

    @Override
    public void close() {
        batch.close();
    }
}
