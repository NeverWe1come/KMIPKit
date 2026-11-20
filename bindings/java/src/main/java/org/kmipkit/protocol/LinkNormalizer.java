package org.kmipkit.protocol;

import java.util.Objects;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvItem;

/** OASIS KMIP link normalizer between KMIP 2.x and 3.0 representations. */
public final class LinkNormalizer {
    private LinkNormalizer() {}

    /** Converts a KMIP 2.x Link TTLV item to its normalized KMIP 3.0 form. Consumes input item. */
    public static TtlvItem v2ToV3(TtlvItem item) {
        Objects.requireNonNull(item, "item");
        NativeExtensionRegistry.ensureLoaded();
        try (item) {
            return item.nativeHandleOwner().withValue(input ->
                    TtlvItem.fromNativeHandle(NativeExtensionRegistry.linkNormalizerV2ToV3(input)));
        }
    }

    /** Converts a KMIP 3.0 Link TTLV item to its normalized KMIP 2.x form. Consumes input item. */
    public static TtlvItem v3ToV2(TtlvItem item) {
        Objects.requireNonNull(item, "item");
        NativeExtensionRegistry.ensureLoaded();
        try (item) {
            return item.nativeHandleOwner().withValue(input ->
                    TtlvItem.fromNativeHandle(NativeExtensionRegistry.linkNormalizerV3ToV2(input)));
        }
    }
}
