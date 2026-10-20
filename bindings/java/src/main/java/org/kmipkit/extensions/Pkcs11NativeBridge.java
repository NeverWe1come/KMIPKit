package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.jspecify.annotations.Nullable;

/** Private JNI bridge for the owner-gated PKCS#11 C ABI mapping. */
final class Pkcs11NativeBridge {
    private Pkcs11NativeBridge() {}

    static long create(
            TtlvStructure function,
            @Nullable TtlvStructure interfaceStructure,
            @Nullable byte[] correlationValue,
            @Nullable TtlvStructure inputParameters) {
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> createNative(
                function.handle(),
                interfaceStructure == null ? 0 : interfaceStructure.handle(),
                correlationValue,
                inputParameters == null ? 0 : inputParameters.handle()));
    }

    static long responseTtlv(long itemResponse) {
        NativeExtensionRegistry.ensureLoaded();
        return responseTtlvNative(itemResponse);
    }

    private static native long createNative(
            long function,
            long interfaceStructure,
            @Nullable byte[] correlationValue,
            long inputParameters);

    private static native long responseTtlvNative(long itemResponse);
}
