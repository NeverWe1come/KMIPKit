package org.kmipkit.extensions;

import org.jspecify.annotations.Nullable;

/** Package-private JNI bridge for standalone attribute read requests. */
final class NativeAttributeRequests {
    private NativeAttributeRequests() {}

    static native long create(
            int operation,
            int identifierKind,
            @Nullable byte[] identifierText,
            long identifierNumber,
            int[] referenceKinds,
            long[] referenceTags,
            byte[][] vendorIdentifications,
            byte[][] attributeNames);
}
