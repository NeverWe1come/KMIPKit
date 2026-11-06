package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated plaintext KMIP Key Value with a generic TTLV view. */
public final class KeyValue extends ManagedTtlvValue {
    private final long formatRaw;

    private KeyValue(long nativeHandle, long formatRaw) {
        super(nativeHandle);
        this.formatRaw = formatRaw;
    }

    /** Copies and validates a plaintext Key Value for the supplied open format. */
    public static KeyValue fromItem(long formatRaw, TtlvItem item) {
        if (formatRaw < 0 || formatRaw > 0xffff_ffffL || item == null) {
            throw new InvalidInputException("Key Value input is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(value -> new KeyValue(
                NativeExtensionRegistry.keyValueFromItem((int) formatRaw, value), formatRaw));
    }

    /** Returns the source-preserving Key Format Type raw value. */
    public long formatRaw() { return formatRaw; }

}
