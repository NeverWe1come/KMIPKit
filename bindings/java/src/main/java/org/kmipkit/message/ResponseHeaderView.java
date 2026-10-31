package org.kmipkit.message;

import java.util.List;
import java.util.Optional;
import java.util.function.Function;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** Read-only response header view that retains its message owner. */
public final class ResponseHeaderView implements AutoCloseable {
    private static final int ATTESTATION_TYPE = 0x4200C7;
    private static final int CLIENT_CORRELATION_VALUE = 0x420105;
    private static final int NONCE = 0x4200C8;
    private static final int SERVER_CORRELATION_VALUE = 0x420106;
    private static final int SERVER_HASHED_PASSWORD = 0x420155;

    private final NativeHandle handle;
    private final ResponseMessage owner;

    ResponseHeaderView(long nativeHandle, ResponseMessage owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.RESPONSE_HEADER_VIEW);
        this.owner = owner;
    }

    /** Returns the raw signed Protocol Version pair. */
    public ProtocolVersion protocolVersion() {
        int[] values = handle.withValue(NativeExtensionRegistry::responseHeaderViewProtocolVersion);
        if (values == null || values.length != 2) {
            throw new IllegalStateException("native protocol version result is invalid");
        }
        return new ProtocolVersion(values[0], values[1]);
    }

    /** Returns the required response Date-Time unchanged. */
    public long timeStamp() {
        return handle.withValue(NativeExtensionRegistry::responseHeaderViewTimeStamp);
    }

    /** Returns the validated batch count. */
    public int batchCount() {
        return handle.withValue(NativeExtensionRegistry::responseHeaderViewBatchCount);
    }

    /** Invokes the callback with the optional Nonce Structure. */
    public <T> Optional<T> withNonce(Function<TtlvStructureView, T> callback) {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.withStructure(value, NONCE, callback);
        }
    }

    /** Returns raw Attestation Type Enumeration values in source order. */
    public List<Long> attestationTypes() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.repeatedEnumerations(value, ATTESTATION_TYPE);
        }
    }

    /** Returns the optional Client Correlation Value text. */
    public Optional<String> clientCorrelationValue() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.textString(value, CLIENT_CORRELATION_VALUE);
        }
    }

    /** Returns the optional Server Correlation Value text. */
    public Optional<String> serverCorrelationValue() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.textString(value, SERVER_CORRELATION_VALUE);
        }
    }

    /**
     * Invokes the callback with a temporary copy of Server Hashed Password bytes.
     * KMIPKit clears the byte array after the callback returns. Copies made by the
     * callback remain subject to JVM memory-management limits.
     */
    public <T> Optional<T> withServerHashedPassword(Function<byte[], T> callback) {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.withByteString(value, SERVER_HASHED_PASSWORD, callback);
        }
    }

    /** Returns an owned copy of the complete header Structure in source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.responseHeaderViewTtlv(value), this));
    }

    @Override
    public void close() {
        handle.close();
    }
}
