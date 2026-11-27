package org.kmipkit.fluent;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.QueryAsyncRequestsRequest;

/** Fluent builder for a typed KMIP Query Asynchronous Requests payload (§6.1.41). */
public final class QueryAsyncRequestsBuilder
        extends TypedOperationBuilder<QueryAsyncRequestsBuilder> {
    private List<byte[]> correlationValues;
    private List<Long> operations;

    QueryAsyncRequestsBuilder(KmipClient client) {
        super(client);
    }

    @Override
    protected QueryAsyncRequestsBuilder self() { return this; }

    public QueryAsyncRequestsBuilder correlationValue(byte[] bytes) {
        ensureOpen();
        if (bytes == null) throw new InvalidInputException("correlationValue is required");
        if (correlationValues == null) correlationValues = new ArrayList<>();
        correlationValues.add(bytes.clone());
        return this;
    }

    public QueryAsyncRequestsBuilder correlationValues(List<byte[]> values) {
        ensureOpen();
        if (values == null) throw new InvalidInputException("correlationValues are required");
        correlationValues = new ArrayList<>(values.size());
        for (byte[] item : values) {
            if (item == null) throw new InvalidInputException("correlation filter cannot contain null");
            correlationValues.add(item.clone());
        }
        return this;
    }

    public QueryAsyncRequestsBuilder operation(long opCode) {
        ensureOpen();
        if (operations == null) operations = new ArrayList<>();
        operations.add(unsigned32(opCode, "Operation"));
        return this;
    }

    public QueryAsyncRequestsBuilder operations(List<Long> opCodes) {
        ensureOpen();
        if (opCodes == null) throw new InvalidInputException("operations are required");
        operations = new ArrayList<>(opCodes.size());
        for (Long opCode : opCodes) {
            if (opCode == null) throw new InvalidInputException("operation cannot be null");
            operations.add(unsigned32(opCode, "Operation"));
        }
        return this;
    }

    @Override
    public ClientBatchItem build() {
        ensureOpen();
        QueryAsyncRequestsRequest req = QueryAsyncRequestsRequest.create();
        try {
            if (correlationValues != null) {
                QueryAsyncRequestsRequest next = req.withCorrelationValues(correlationValues);
                req.close();
                req = next;
            }
            if (operations != null) {
                QueryAsyncRequestsRequest next = req.withOperations(operations);
                req.close();
                req = next;
            }
            return validated(ClientBatchItem.queryAsyncRequests(req));
        } finally {
            req.close();
        }
    }
}
