package org.kmipkit.fluent;

import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.ttlv.CodecLimits;

/** Shared one-item batch execution path for the lifecycle builders. */
final class BatchExecution {
    private BatchExecution() { }

    static ClientBatchResponse execute(
            KmipClient client, ClientBatchItem item, CodecLimits limits,
            RequestOptions options, Long asynchronousIndicator)
            throws org.kmipkit.ClientOperationException {
        try (ClientBatch batch = ClientBatch.create()) {
            batch.add(item);
            if (asynchronousIndicator != null) {
                batch.asynchronousIndicator(asynchronousIndicator);
            }
            return options == null ? client.execute(batch, limits)
                    : client.executeWithOptions(batch, limits, options);
        }
    }
}
