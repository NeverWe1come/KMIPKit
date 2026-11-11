package org.kmipkit.fluent;

import org.kmipkit.NativeExtensionRegistry;
import org.jspecify.annotations.Nullable;

/** Package-private JNI bridge so fluent builder handles do not expand public exports. */
final class NativeFluentRegistry {
    static final int CRYPTO_OPERATION_BUILDER = 72;

    private NativeFluentRegistry() {}

    static void ensureLoaded() {
        NativeExtensionRegistry.ensureLoaded();
    }

    static native long cryptoOperationBuilderCreate(int operation);

    static native void cryptoOperationBuilderSetUniqueIdentifier(
            long builder, int kind, @Nullable byte[] text, long number);

    static native void cryptoOperationBuilderSetBytes(long builder, int field, byte[] value);

    static native void cryptoOperationBuilderSetDataValue(long builder, long value);

    static native void cryptoOperationBuilderSetCryptographicParameters(
            long builder, long parameters);

    static native void cryptoOperationBuilderSetCryptographicParametersStructure(
            long builder, long parameters);

    static native void cryptoOperationBuilderSetIndicator(long builder, int indicator, boolean value);

    static native long cryptoOperationBuilderBuildBatchItem(long builder);
}
