package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.ByteBuffer;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValueView;

/** KMIP 2.1 §6.1.5, §6.1.38, §6.1.39, and §6.1.41 request model parity. */
final class AsyncRequestModelsTest {
    private static final int CORRELATION_VALUE_TAG = 0x00420006;
    private static final int CORRELATION_VALUES_TAG = 0x00420176;
    private static final int OPERATIONS_TAG = 0x0042014F;
    private static final int OPERATION_TAG = 0x0042005C;

    @Test
    void cancelPollAndProcessBuildPayloadsWithoutExecutingOperations() {
        byte[] correlation = new byte[] {0, (byte) 0xff, (byte) 0x80};
        try (CancelRequest cancel = CancelRequest.create(correlation);
                PollRequest poll = PollRequest.create(correlation);
                ProcessRequest process = ProcessRequest.create(correlation)) {
            assertCorrelation(cancel::withAsynchronousCorrelationValue, correlation);
            assertCorrelation(poll::withAsynchronousCorrelationValue, correlation);

            try (TtlvStructure payload = cancel.toTtlvPayload()) {
                assertSingleCorrelation(payload, correlation);
            }
            try (TtlvStructure payload = poll.toTtlvPayload()) {
                assertSingleCorrelation(payload, correlation);
            }
            try (TtlvStructure payload = process.toTtlvPayload()) {
                assertSingleCorrelation(payload, correlation);
            }
        }
    }

    @Test
    void queryRequestPreservesAbsentAndPresentOrderedFilters() {
        try (QueryAsyncRequestsRequest empty = QueryAsyncRequestsRequest.create();
                TtlvStructure emptyPayload = empty.toTtlvPayload();
                TtlvStructureView emptyView = emptyPayload.view()) {
            assertEquals(0, emptyView.itemCount());
        }

        byte[][] correlations = new byte[][] {
            {0, (byte) 0xff}, {0x10, 0}, {0, (byte) 0xff}
        };
        List<Long> operations = List.of(0x1aL, 0xf1234567L);
        try (QueryAsyncRequestsRequest query = QueryAsyncRequestsRequest.create();
                QueryAsyncRequestsRequest withCorrelations = query.withCorrelationValues(
                        List.of(correlations));
                QueryAsyncRequestsRequest filtered = withCorrelations.withOperations(operations);
                TtlvStructure payload = filtered.toTtlvPayload();
                TtlvStructureView view = payload.view()) {
            assertEquals(2, view.itemCount());
            assertEquals(CORRELATION_VALUES_TAG, tagAt(view, 0));
            assertEquals(OPERATIONS_TAG, tagAt(view, 1));
            assertNestedTagCount(view, 0, CORRELATION_VALUE_TAG, 3);
            assertNestedTagCount(view, 1, OPERATION_TAG, 2);
            assertEquals(0xf1234567L, nestedEnumerationAt(view, 1, 1));
        }
    }

    private static void assertCorrelation(
            java.util.function.Function<java.util.function.Consumer<ByteBuffer>, Boolean> borrow,
            byte[] expected) {
        assertTrue(borrow.apply(buffer -> {
            assertTrue(buffer.isReadOnly());
            byte[] observed = new byte[buffer.remaining()];
            buffer.get(observed);
            assertEquals(ByteBuffer.wrap(expected), ByteBuffer.wrap(observed));
        }));
    }

    private static void assertSingleCorrelation(TtlvStructure payload, byte[] expected) {
        try (TtlvStructureView view = payload.view()) {
            assertEquals(1, view.itemCount());
            assertEquals(CORRELATION_VALUE_TAG, tagAt(view, 0));
            try (TtlvItemView item = view.itemAt(0); TtlvValueView value = item.value()) {
                assertEquals(expected.length, value.byteLength());
                for (int index = 0; index < expected.length; index++) {
                    assertEquals(Byte.toUnsignedInt(expected[index]),
                            Short.toUnsignedInt(value.byteAt(index)));
                }
            }
        }
    }

    private static int tagAt(TtlvStructureView view, long index) {
        try (TtlvItemView item = view.itemAt(index)) {
            try (org.kmipkit.ttlv.Tag tag = item.tag()) {
                return tag.raw();
            }
        }
    }

    private static void assertNestedTagCount(
            TtlvStructureView parent, long index, int tag, long expectedCount) {
        try (TtlvItemView item = parent.itemAt(index);
                TtlvValueView value = item.value();
                TtlvStructureView nested = value.structure()) {
            assertEquals(expectedCount, nested.itemCount());
            for (long child = 0; child < expectedCount; child++) {
                assertEquals(tag, tagAt(nested, child));
            }
        }
    }

    private static long nestedEnumerationAt(TtlvStructureView parent, long index, long child) {
        try (TtlvItemView item = parent.itemAt(index);
                TtlvValueView value = item.value();
                TtlvStructureView nested = value.structure();
                TtlvItemView operation = nested.itemAt(child);
                TtlvValueView operationValue = operation.value()) {
            return operationValue.enumeration();
        }
    }

}
