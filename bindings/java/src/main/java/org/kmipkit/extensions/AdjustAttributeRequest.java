package org.kmipkit.extensions;

import java.util.Optional;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvValue;

/** Owned KMIP 2.1 Adjust Attribute Request model. */
public final class AdjustAttributeRequest extends AttributeRequest {
    private final long adjustmentType;

    private AdjustAttributeRequest(
            long handle, UniqueIdentifier identifier, AttributeReference reference, long type) {
        super(handle, NativeExtensionRegistry.ADJUST_ATTRIBUTE_REQUEST,
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_ADJUST, identifier, reference);
        adjustmentType = type;
    }

    /** Copies the required reference and optional value, preserving raw Adjustment Type. */
    public static AdjustAttributeRequest create(
            UniqueIdentifier identifier,
            AttributeReference reference,
            long adjustmentType,
            TtlvValue adjustmentValue) {
        if (reference == null) {
            throw new InvalidInputException("Attribute Reference is required");
        }
        long handle = AttributeRequest.createNative(
                NativeExtensionRegistry.ATTRIBUTE_OPERATION_ADJUST, identifier,
                null, null, reference, adjustmentType, adjustmentValue);
        return new AdjustAttributeRequest(handle, identifier, reference, adjustmentType);
    }

    /** Returns the raw unsigned Adjustment Type, including future values. */
    public long adjustmentType() {
        ensureOpen();
        return adjustmentType;
    }

    /** Returns the required Attribute Reference, preserving its tag or name form. */
    public AttributeReference attributeReference() {
        return attributeReferenceValue();
    }

    /** Returns an independently owned copy of the optional Adjustment Value. */
    public Optional<TtlvValue> adjustmentValue() {
        return Optional.ofNullable(value());
    }
}
