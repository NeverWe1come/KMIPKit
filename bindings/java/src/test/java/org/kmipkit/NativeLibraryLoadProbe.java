package org.kmipkit;

import java.util.Arrays;
import org.kmipkit.extensions.ClientBatchItem;

/** Child-JVM entry point for exercising default bundled native-library loading. */
public final class NativeLibraryLoadProbe {
    private NativeLibraryLoadProbe() {}

    public static void main(String[] args) {
        boolean abiMismatchProbe = args.length == 1 && args[0].equals("abi-mismatch");
        boolean operationInputLimitProbe = args.length == 1 && args[0].equals("operation-input-limit");
        if (System.getProperty("kmipkit.nativeLibrary") != null
                && !abiMismatchProbe && !operationInputLimitProbe) {
            throw new IllegalStateException("The explicit library override must be absent");
        }
        if (operationInputLimitProbe) {
            byte[] oversizedInput = new byte[16 * 1024 * 1024 + 1];
            Arrays.fill(oversizedInput, (byte) 0x5a);
            requireResourceLimit(() -> ClientBatchItem.certify(null, null, oversizedInput, null, null));
            requireResourceLimit(() -> ClientBatchItem.macVerify(
                    null, null, null, oversizedInput, null, null, null));
            requireResourceLimit(() -> ClientBatchItem.sign(
                    null, null, null, oversizedInput, null, null, null));
            requireResourceLimit(() -> ClientBatchItem.signatureVerify(
                    null, null, null, oversizedInput, null, null, null, null));
            requireResourceLimit(() -> ClientBatchItem.signatureVerify(
                    null, null, null, null, oversizedInput, null, null, null));
            if (oversizedInput[0] != (byte) 0x5a || oversizedInput[oversizedInput.length - 1] != (byte) 0x5a) {
                throw new IllegalStateException("Operation input was modified");
            }
            System.out.println("OPERATION_INPUT_LIMIT_REJECTED_BEFORE_NATIVE_LOAD");
            return;
        }
        if (args.length == 1 && args[0].equals("version")) {
            System.out.println("VERSION=" + RuntimeMetadata.version());
            return;
        }
        NativeExtensionRegistry.ensureLoaded();
        System.out.println("ABI=" + NativeExtensionRegistry.nativeAbiVersion());
    }

    private static void requireResourceLimit(Runnable factory) {
        try {
            factory.run();
        } catch (ResourceLimitException expected) {
            return;
        }
        throw new IllegalStateException("Oversized operation input was accepted");
    }
}
