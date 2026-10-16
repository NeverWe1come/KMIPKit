package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Common, operation-independent metadata for one response batch item. */
public final class ClientBatchItemResult implements AutoCloseable {
    private final NativeHandle handle;

    ClientBatchItemResult(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT_BATCH_ITEM_RESPONSE);
    }

    /** Returns the forward-compatible completed or Pending outcome discriminator. */
    public BatchOutcomeKind outcomeKind() {
        return new BatchOutcomeKind(metadata()[0]);
    }

    /** Returns the raw unsigned KMIP Result Status value, preserving unknown values. */
    public long resultStatus() {
        return metadata()[1];
    }

    /** Returns the optional raw unsigned KMIP Result Reason value. */
    public OptionalLong resultReason() {
        long[] values = metadata();
        return values[2] == 0 ? OptionalLong.empty() : OptionalLong.of(values[3]);
    }

    /** Returns the copied Unique Batch Item ID, when the server supplied one. */
    public Optional<byte[]> uniqueBatchItemId() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] value = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseUniqueBatchItemId);
        return value == null ? Optional.empty() : Optional.of(value.clone());
    }

    /** Returns the exact pending correlation bytes, when this result is Pending. */
    public Optional<byte[]> pendingCorrelationValue() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] value = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponsePendingCorrelationValue);
        return value == null ? Optional.empty() : Optional.of(value.clone());
    }

    /**
     * Returns the ordered Protocol Version list for a successful Discover Versions result.
     * An empty present list is distinct from Pending, failure, or another operation.
     */
    public Optional<List<ProtocolVersion>> supportedVersions() {
        NativeExtensionRegistry.ensureLoaded();
        long[] values = handle.withValue(
                NativeExtensionRegistry::clientBatchItemResponseDiscoverVersionsSupportedVersions);
        if (values == null) {
            return Optional.empty();
        }
        if ((values.length & 1) != 0) {
            throw new InvalidInputException("native Discover Versions list is invalid");
        }
        List<ProtocolVersion> versions = new ArrayList<>(values.length / 2);
        for (int index = 0; index < values.length; index += 2) {
            versions.add(new ProtocolVersion(
                    checkedVersionComponent(values[index]),
                    checkedVersionComponent(values[index + 1])));
        }
        return Optional.of(List.copyOf(versions));
    }

    private long[] metadata() {
        NativeExtensionRegistry.ensureLoaded();
        long[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseMetadata);
        if (values == null || values.length != 4) {
            throw new InvalidInputException("native batch result metadata is invalid");
        }
        return values;
    }

    private static int checkedVersionComponent(long value) {
        if (value < Integer.MIN_VALUE || value > Integer.MAX_VALUE) {
            throw new InvalidInputException("native Discover Versions component is invalid");
        }
        return (int) value;
    }

    @Override
    public void close() {
        handle.close();
    }
}
