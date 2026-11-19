package org.kmipkit.extensions;

import java.util.Optional;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

/** Shared lifecycle and field-copy behavior for typed attribute request models. */
abstract class AttributeRequest implements AutoCloseable {
    static final int OPERATION_GET_ATTRIBUTES = 5;
    static final int OPERATION_GET_ATTRIBUTE_LIST = 6;

    private final NativeHandle handle;
    private final int operation;
    private final UniqueIdentifier uniqueIdentifier;
    private final AttributeReference attributeReference;

    AttributeRequest(
            long nativeHandle,
            int handleKind,
            int operation,
            UniqueIdentifier uniqueIdentifier,
            AttributeReference attributeReference) {
        handle = NativeHandle.owned(nativeHandle, handleKind);
        this.operation = operation;
        this.uniqueIdentifier = uniqueIdentifier;
        this.attributeReference = attributeReference;
    }

    /** Returns the exact optional Unique Identifier value. */
    public final Optional<UniqueIdentifier> uniqueIdentifier() {
        handle.get();
        return Optional.ofNullable(uniqueIdentifier);
    }

    final AttributeReference attributeReferenceValue() {
        handle.get();
        return attributeReference;
    }

    /** Returns an owned generic TTLV copy of this Request Payload. */
    public final TtlvStructure toTtlvPayload() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.attributeRequestToTtlvPayload(operation, value), this));
    }

    /** Creates an independent batch item while leaving this request open. */
    public final ClientBatchItem toBatchItem() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> ClientBatchItem.fromRequestHandle(
                NativeExtensionRegistry.clientBatchItemFromAttributeRequest(operation, value)));
    }

    final TtlvItem item(int field) {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value ->
                NativeExtensionRegistry.attributeRequestTtlvItem(operation, value, field));
    }

    final TtlvValue value() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> NativeExtensionRegistry.attributeRequestTtlvValue(
                operation, value));
    }

    final void ensureOpen() {
        handle.get();
    }

    static long createNative(
            int operation,
            UniqueIdentifier identifier,
            TtlvItem currentAttribute,
            TtlvItem newAttribute,
            AttributeReference reference,
            long adjustmentType,
            TtlvValue adjustmentValue) {
        if (identifier != null && identifier.kind() != UniqueIdentifier.Kind.TEXT_STRING) {
            throw new InvalidInputException("attribute request identifier must be text");
        }
        if (adjustmentType < 0 || adjustmentType > 0xffff_ffffL) {
            throw new InvalidInputException("Adjustment Type is outside uint32");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> NativeExtensionRegistry.attributeRequestCreate(
                operation,
                identifier == null ? 0 : identifier.nativeKind(),
                identifier == null ? null : identifier.nativeText(),
                identifier == null ? 0 : identifier.nativeNumber(),
                currentAttribute == null ? 0 : currentAttribute.nativeHandleOwner().get(),
                newAttribute == null ? 0 : newAttribute.nativeHandleOwner().get(),
                reference == null ? 0 : reference.nativeKind(),
                reference == null ? 0 : reference.nativeTagValue(),
                reference == null ? null : reference.nativeVendorIdentification(),
                reference == null ? null : reference.nativeAttributeName(),
                adjustmentType,
                adjustmentValue == null ? 0 : adjustmentValue.nativeHandleOwner().get()));
    }

    static long createReadNative(
            int operation, UniqueIdentifier identifier, List<AttributeReference> references) {
        if (identifier != null && identifier.kind() != UniqueIdentifier.Kind.TEXT_STRING) {
            throw new InvalidInputException("attribute request identifier must be text");
        }
        if (references == null) {
            throw new InvalidInputException("Attribute References are required");
        }
        int[] kinds = new int[references.size()];
        long[] tags = new long[references.size()];
        byte[][] vendors = new byte[references.size()][];
        byte[][] names = new byte[references.size()][];
        for (int index = 0; index < references.size(); index++) {
            AttributeReference reference = references.get(index);
            if (reference == null) {
                throw new InvalidInputException("Attribute Reference is required");
            }
            kinds[index] = reference.nativeKind();
            tags[index] = reference.nativeTagValue();
            vendors[index] = reference.nativeVendorIdentification();
            names[index] = reference.nativeAttributeName();
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeAttributeRequests.create(
                operation,
                identifier == null ? 0 : identifier.nativeKind(),
                identifier == null ? null : identifier.nativeText(),
                identifier == null ? 0 : identifier.nativeNumber(),
                kinds,
                tags,
                vendors,
                names);
    }

    @Override
    public final void close() {
        handle.close();
    }
}
