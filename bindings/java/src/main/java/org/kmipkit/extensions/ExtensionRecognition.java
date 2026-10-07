package org.kmipkit.extensions;

import java.util.Optional;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructureView;

/** Result of inspecting one received vendor extension. */
public final class ExtensionRecognition {
    private final NativeHandle handle;
    private final ClientExtensionRegistry registryOwner;

    ExtensionRecognition(long nativeHandle, ClientExtensionRegistry registryOwner) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.RECOGNITION);
        this.registryOwner = registryOwner;
    }

    public static boolean isRecognized(ExtensionRecognition recognition) {
        NativeExtensionRegistry.ensureLoaded();
        return recognition.handle.withValue(NativeExtensionRegistry::extensionRecognitionIsRecognized);
    }

    public static Optional<ValidatedExtensionValue> validatedValue(ExtensionRecognition recognition) {
        NativeExtensionRegistry.ensureLoaded();
        return recognition.handle.withValue(handle -> {
            long value = NativeExtensionRegistry.extensionRecognitionValidatedValue(handle);
            if (value == 0) {
                return Optional.empty();
            }
            long identityHandle = NativeExtensionRegistry.validatedExtensionValueIdentity(value);
            return Optional.of(new ValidatedExtensionValue(
                    value, ExtensionIdentity.fromNative(identityHandle), recognition));
        });
    }

    public static TtlvStructureView genericValue(ExtensionRecognition recognition) {
        NativeExtensionRegistry.ensureLoaded();
        return recognition.handle.withValue(value -> TtlvStructureView.fromNative(
                NativeExtensionRegistry.extensionRecognitionGenericValue(value), recognition));
    }

    @Override
    public String toString() {
        return "ExtensionRecognition([REDACTED])";
    }
}
