package org.kmipkit.fluent;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ServerResultException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.ttlv.CodecLimits;

/** Shared one-item batch execution path for the fluent builders. */
final class BatchExecution {
    private BatchExecution() { }

    static CodecLimits resolveLimits(KmipClient client, CodecLimits explicitLimits) {
        if (explicitLimits != null) {
            return explicitLimits;
        }
        if (client != null) {
            return client.codecLimits();
        }
        throw new InvalidInputException("client is required to execute");
    }

    static ClientBatchResponse execute(
            KmipClient client, ClientBatchItem item, CodecLimits limits,
            RequestOptions options, Long asynchronousIndicator)
            throws org.kmipkit.ClientOperationException {
        if (client == null) {
            throw new InvalidInputException("client is required to execute");
        }
        if (limits == null) {
            throw new InvalidInputException("codec limits are required");
        }
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(item);
            if (asynchronousIndicator != null) {
                batch.asynchronousIndicator(asynchronousIndicator);
            }
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }

    static ClientBatchItemResult executeOutcome(
            KmipClient client, ClientBatchItem item, CodecLimits limits,
            RequestOptions options, Long asynchronousIndicator)
            throws org.kmipkit.ClientOperationException {
        CodecLimits effectiveLimits = resolveLimits(client, limits);
        try (ClientBatchResponse response = execute(
                client, item, effectiveLimits, options, asynchronousIndicator)) {
            return response.itemAt(0);
        }
    }

    static ClientBatchItemResult executeResult(
            KmipClient client, ClientBatchItem item, CodecLimits limits,
            RequestOptions options, Long asynchronousIndicator)
            throws org.kmipkit.ClientOperationException {
        ClientBatchItemResult result = executeOutcome(
                client, item, limits, options, asynchronousIndicator);
        long status = result.resultStatus();
        if (status != 0L) {
            var reason = result.resultReason();
            result.close();
            throw ServerResultException.fromServerResult(status, reason);
        }
        return result;
    }
}
