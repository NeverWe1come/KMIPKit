package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;
import org.kmipkit.protocol.KmipValues;

final class AsyncBatchItemTest {
    @Test
    void convertsAsyncRequestModelsToMatchingBatchItemsWithoutConsumingTheModels() {
        byte[] correlation = {0, (byte) 0xff, 0x41};

        try (CancelRequest request = CancelRequest.create(correlation);
             ClientBatchItem item = ClientBatchItem.cancel(request)) {
            assertEquals(KmipValues.Operation.CANCEL, item.operation());
        }
        try (PollRequest request = PollRequest.create(correlation);
             ClientBatchItem item = ClientBatchItem.poll(request)) {
            assertEquals(KmipValues.Operation.POLL, item.operation());
        }
        try (ProcessRequest request = ProcessRequest.create(correlation);
             ClientBatchItem item = ClientBatchItem.process(request)) {
            assertEquals(KmipValues.Operation.PROCESS, item.operation());
        }
        try (QueryAsyncRequestsRequest request = QueryAsyncRequestsRequest.create();
             ClientBatchItem item = ClientBatchItem.queryAsyncRequests(request)) {
            assertEquals(KmipValues.Operation.QUERY_ASYNCHRONOUS_REQUESTS, item.operation());
        }
    }
}
