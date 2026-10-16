package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
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

    /** Returns version 2 metadata, including the source operation and exact Result Message. */
    public ClientBatchItemMetadataV2 metadataV2() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseMetadataV2);
        if (values == null || values.length != 2 || !(values[0] instanceof long[] metadata)) {
            throw new InvalidInputException("native batch result metadata v2 is invalid");
        }
        byte[] message = values[1] == null ? null : requireBytes(values[1]);
        return new ClientBatchItemMetadataV2(metadata, message);
    }

    /** Returns the raw unsigned KMIP operation value, preserving unknown operations. */
    public long operation() {
        return metadataV2().operation();
    }

    /** Returns the exact optional server Result Message. */
    public Optional<String> resultMessage() {
        return metadataV2().resultMessage();
    }

    /** Returns the successful Create response, including the identifier's exact wire type. */
    public Optional<CreateResponse> createResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCreate);
        if (values == null || values.length != 2) {
            throw new InvalidInputException("native Create response is invalid");
        }
        long[] fields = requireLongArray(values[0], 5);
        if (fields[0] == 0) {
            return Optional.empty();
        }
        requirePresent(fields[0]);
        byte[] text = values[1] == null ? null : requireBytes(values[1]);
        return Optional.of(new CreateResponse(
                fields[1], identifier(fields[2], fields[3], fields[4], text)));
    }

    /** Returns distinct private and public identifiers for a successful Create Key Pair response. */
    public Optional<CreateKeyPairResponse> createKeyPairResponse() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCreateKeyPair);
        if (values == null || values.length != 3) {
            throw new InvalidInputException("native Create Key Pair response is invalid");
        }
        long[] fields = requireLongArray(values[0], 1);
        if (fields[0] == 0) {
            return Optional.empty();
        }
        requirePresent(fields[0]);
        return Optional.of(new CreateKeyPairResponse(identifierRecord(values[1]), identifierRecord(values[2])));
    }

    /** Returns the one-or-more Create Split Key identifiers in response wire order. */
    public Optional<List<UniqueIdentifier>> createSplitKeyUniqueIdentifiers() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(NativeExtensionRegistry::clientBatchItemResponseCreateSplitKey);
        if (values == null || values.length != 2) {
            throw new InvalidInputException("native Create Split Key response is invalid");
        }
        long[] fields = requireLongArray(values[0], 2);
        if (fields[0] == 0) {
            return Optional.empty();
        }
        requirePresent(fields[0]);
        if (fields[1] < 1 || fields[1] > 16L * 1024L * 1024L
                || !(values[1] instanceof Object[] identifiers)
                || identifiers.length != fields[1]) {
            throw new ResourceLimitException("native Create Split Key identifiers exceed the configured limit");
        }
        List<UniqueIdentifier> result = new ArrayList<>(identifiers.length);
        for (Object identifier : identifiers) {
            result.add(identifierRecord(identifier));
        }
        return Optional.of(List.copyOf(result));
    }

    /** Successful Create response fields. */
    public static final class CreateResponse {
        private final long objectType;
        private final UniqueIdentifier uniqueIdentifier;

        private CreateResponse(long objectType, UniqueIdentifier uniqueIdentifier) {
            this.objectType = objectType;
            this.uniqueIdentifier = uniqueIdentifier;
        }

        public long objectType() { return objectType; }
        public UniqueIdentifier uniqueIdentifier() { return uniqueIdentifier; }

        @Override public String toString() { return "CreateResponse[objectType=" + objectType + ", uniqueIdentifier=redacted]"; }
    }

    /** Successful Create Key Pair response with separate identifiers. */
    public static final class CreateKeyPairResponse {
        private final UniqueIdentifier privateKeyUniqueIdentifier;
        private final UniqueIdentifier publicKeyUniqueIdentifier;

        private CreateKeyPairResponse(UniqueIdentifier privateIdentifier, UniqueIdentifier publicIdentifier) {
            privateKeyUniqueIdentifier = privateIdentifier;
            publicKeyUniqueIdentifier = publicIdentifier;
        }

        public UniqueIdentifier privateKeyUniqueIdentifier() { return privateKeyUniqueIdentifier; }
        public UniqueIdentifier publicKeyUniqueIdentifier() { return publicKeyUniqueIdentifier; }

        @Override public String toString() { return "CreateKeyPairResponse[identifiers=redacted]"; }
    }

    private static UniqueIdentifier identifierRecord(Object value) {
        if (!(value instanceof Object[] values) || values.length != 2) {
            throw new InvalidInputException("native Unique Identifier is invalid");
        }
        long[] fields = requireLongArray(values[0], 4);
        requirePresent(fields[0]);
        byte[] text = values[1] == null ? null : requireBytes(values[1]);
        return identifier(fields[1], fields[2], fields[3], text);
    }

    private static UniqueIdentifier identifier(long kind, long enumeration, long integer, Object value) {
        byte[] text = value == null ? null : requireBytes(value);
        return UniqueIdentifier.fromNative(kind, text, enumeration, integer);
    }

    private static long[] requireLongArray(Object value, int length) {
        if (!(value instanceof long[] values) || values.length != length) {
            throw new InvalidInputException("native creation response fields are invalid");
        }
        return values;
    }

    private static byte[] requireBytes(Object value) {
        if (!(value instanceof byte[] bytes)) {
            throw new InvalidInputException("native byte field is invalid");
        }
        return bytes;
    }

    private static void requirePresent(long value) {
        if (value != 1) {
            throw new InvalidInputException("native response presence flag is invalid");
        }
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
