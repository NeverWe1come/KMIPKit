package org.kmipkit.message;

import java.util.Optional;
import java.util.function.Function;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** Read-only opaque Message Extension view that retains its batch item owner. */
public final class MessageExtensionView implements AutoCloseable {
    private static final int CRITICALITY_INDICATOR = 0x420026;
    private static final int VENDOR_EXTENSION = 0x42009C;
    private static final int VENDOR_IDENTIFICATION = 0x42009D;

    private final NativeHandle handle;
    private final Object owner;

    MessageExtensionView(long nativeHandle, Object owner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.MESSAGE_EXTENSION_VIEW);
        this.owner = owner;
    }

    /** Returns the full generic extension value in source order. */
    public TtlvStructure ttlv() {
        return handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.messageExtensionViewTtlv(value), this));
    }

    /** Returns the Vendor Identification text. */
    public Optional<String> vendorIdentification() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.textString(value, VENDOR_IDENTIFICATION);
        }
    }

    /** Returns the Criticality Indicator Boolean. */
    public Optional<Boolean> criticalityIndicator() {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.booleanValue(value, CRITICALITY_INDICATOR);
        }
    }

    /** Invokes the callback with the opaque Vendor Extension Structure. */
    public <T> Optional<T> withVendorExtension(Function<TtlvStructureView, T> callback) {
        try (TtlvStructure value = ttlv()) {
            return MessageTtlvAccess.withStructure(value, VENDOR_EXTENSION, callback);
        }
    }

    @Override
    public void close() {
        handle.close();
    }
}
