package org.kmipkit.protocol;

import java.util.Objects;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvItem;

/** OASIS KMIP link normalizer between KMIP 2.x and 3.0 representations. */
public final class LinkNormalizer {
    private LinkNormalizer() {}

    /** Converts a KMIP 2.x Link TTLV item to its normalized KMIP 3.0 form. Consumes input item. */
    public static TtlvItem v2ToV3(TtlvItem item) {
        Objects.requireNonNull(item, "item");
        NativeExtensionRegistry.ensureLoaded();
        long output = NativeHandle.withNativeHandles(() -> {
            long inputHandle = item.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.linkNormalizerV2ToV3(inputHandle);
        });
        return TtlvItem.fromNativeHandle(output);
    }

    /** Converts a KMIP 3.0 Link TTLV item to its normalized KMIP 2.x form. Consumes input item. */
    public static TtlvItem v3ToV2(TtlvItem item) {
        Objects.requireNonNull(item, "item");
        NativeExtensionRegistry.ensureLoaded();
        long output = NativeHandle.withNativeHandles(() -> {
            long inputHandle = item.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.linkNormalizerV3ToV2(inputHandle);
        });
        return TtlvItem.fromNativeHandle(output);
    }
}
