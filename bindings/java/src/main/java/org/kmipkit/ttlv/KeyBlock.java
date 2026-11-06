package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated KMIP Key Block with a lossless generic TTLV view. */
public final class KeyBlock extends ManagedTtlvValue {
    private KeyBlock(long nativeHandle) { super(nativeHandle); }

    /** Copies and validates a Key Block without consuming its source Item. */
    public static KeyBlock fromItem(TtlvItem item) {
        if (item == null) throw new InvalidInputException("Key Block input is invalid");
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(
                value -> new KeyBlock(NativeExtensionRegistry.keyBlockFromItem(value)));
    }

}
