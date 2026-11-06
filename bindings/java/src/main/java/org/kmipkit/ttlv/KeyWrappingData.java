package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned, validated KMIP Key Wrapping Data with a lossless generic TTLV view. */
public final class KeyWrappingData extends ManagedTtlvValue {
    private KeyWrappingData(long nativeHandle) { super(nativeHandle); }

    /** Copies and validates Key Wrapping Data without consuming the source Item. */
    public static KeyWrappingData fromItem(TtlvItem item) {
        if (item == null) throw new InvalidInputException("Key Wrapping Data input is invalid");
        NativeExtensionRegistry.ensureLoaded();
        return item.nativeHandleOwner().withValue(
                value -> new KeyWrappingData(NativeExtensionRegistry.keyWrappingDataFromItem(value)));
    }

}
