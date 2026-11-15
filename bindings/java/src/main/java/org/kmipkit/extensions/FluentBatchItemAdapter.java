package org.kmipkit.extensions;

/** Internal bridge used to wrap batch items returned by fluent native builders. */
public final class FluentBatchItemAdapter {
    private FluentBatchItemAdapter() {}

    /** Takes ownership of a batch item handle returned by the KMIPKit C ABI. */
    public static ClientBatchItem takeOwnership(long handle) {
        return ClientBatchItem.fromRequestHandle(handle);
    }
}
