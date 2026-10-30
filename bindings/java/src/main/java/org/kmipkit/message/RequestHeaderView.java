package org.kmipkit.message;

import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import java.util.function.Function;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** Read-only request header view that retains its message owner. */
public final class RequestHeaderView implements AutoCloseable {
    private static final int ASYNCHRONOUS_INDICATOR = 0x420007;
    private static final int ATTESTATION_CAPABLE_INDICATOR = 0x4200D3;
    private static final int ATTESTATION_TYPE = 0x4200C7;
    private static final int AUTHENTICATION = 0x42000C;
    private static final int BATCH_ERROR_CONTINUATION_OPTION = 0x42000E;
    private static final int BATCH_ORDER_OPTION = 0x420010;
    private static final int CLIENT_CORRELATION_VALUE = 0x420105;
    private static final int MAXIMUM_RESPONSE_SIZE = 0x420050;
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

    /** Returns the optional Maximum Response Size Integer. */
    public OptionalInt maximumResponseSize() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.integer(value, MAXIMUM_RESPONSE_SIZE);
        }
    }

    /** Returns the optional Client Correlation Value text. */
    public Optional<String> clientCorrelationValue() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.textString(value, CLIENT_CORRELATION_VALUE);
        }
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

    /** Returns the optional Attestation Capable Indicator. */
    public Optional<Boolean> attestationCapableIndicator() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.booleanValue(value, ATTESTATION_CAPABLE_INDICATOR);
        }
    }

    /** Returns the indicator, or the effective OASIS default {@code false}. */
    public boolean effectiveAttestationCapableIndicator() {
        return attestationCapableIndicator().orElse(false);
    }

    /** Returns raw Attestation Type Enumeration values in source order. */
    public List<Long> attestationTypes() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.repeatedEnumerations(value, ATTESTATION_TYPE);
        }
    }

    /** Invokes the callback with the optional Authentication Structure. */
    public <T> Optional<T> withAuthentication(Function<TtlvStructureView, T> callback) {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.withStructure(value, AUTHENTICATION, callback);
        }
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
