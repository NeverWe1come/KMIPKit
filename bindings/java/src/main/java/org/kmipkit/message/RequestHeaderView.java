package org.kmipkit.message;

import java.util.Optional;
import java.util.OptionalLong;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Read-only request header view that retains its message owner. */
public final class RequestHeaderView implements AutoCloseable {
    private static final int ASYNCHRONOUS_INDICATOR = 0x420007;
    private static final int BATCH_ERROR_CONTINUATION_OPTION = 0x42000E;
    private static final int BATCH_ORDER_OPTION = 0x420010;
    private static final int TIME_STAMP = 0x420092;

    private final NativeHandle handle;
    private final RequestMessage owner;

    RequestHeaderView(long nativeHandle, RequestMessage owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REQUEST_HEADER_VIEW);
        this.owner = owner;
    }

    /** Returns the raw signed Protocol Version pair. */
    public ProtocolVersion protocolVersion() {
        int[] values = handle.withValue(NativeExtensionRegistry::requestHeaderViewProtocolVersion);
        if (values == null || values.length != 2) {
            throw new IllegalStateException("native protocol version result is invalid");
        }
        return new ProtocolVersion(values[0], values[1]);
    }

    /** Returns the validated batch count. */
    public int batchCount() {
        return handle.withValue(NativeExtensionRegistry::requestHeaderViewBatchCount);
    }

    /** Returns the optional raw Asynchronous Indicator Enumeration. */
    public OptionalLong asynchronousIndicator() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.enumeration(value, ASYNCHRONOUS_INDICATOR);
        }
    }

    /** Returns the raw indicator, or the effective OASIS default Prohibited (`3`). */
    public long effectiveAsynchronousIndicator() {
        return asynchronousIndicator().orElse(3);
    }

    /** Returns the optional raw Batch Error Continuation Enumeration. */
    public OptionalLong batchErrorContinuationOption() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.enumeration(value, BATCH_ERROR_CONTINUATION_OPTION);
        }
    }

    /** Returns the raw option, or the effective OASIS default Stop (`2`). */
    public long effectiveBatchErrorContinuationOption() {
        return batchErrorContinuationOption().orElse(2);
    }

    /** Returns the optional Batch Order Option field without materializing an absent value. */
    public Optional<Boolean> batchOrderOption() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.booleanValue(value, BATCH_ORDER_OPTION);
        }
    }

    /** Returns the option, or its effective default of {@code true}. */
    public boolean effectiveBatchOrderOption() {
        return batchOrderOption().orElse(true);
    }

    /** Returns the optional request Date-Time unchanged. */
    public OptionalLong timeStamp() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.dateTime(value, TIME_STAMP);
        }
    }

    /** Returns an owned copy of the complete header Structure in source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.requestHeaderViewTtlv(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
