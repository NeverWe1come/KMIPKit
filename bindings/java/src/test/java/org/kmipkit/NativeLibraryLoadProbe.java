package org.kmipkit;

/** Child-JVM entry point for exercising default bundled native-library loading. */
public final class NativeLibraryLoadProbe {
    private NativeLibraryLoadProbe() {}

    public static void main(String[] args) {
        boolean abiMismatchProbe = args.length == 1 && args[0].equals("abi-mismatch");
        if (System.getProperty("kmipkit.nativeLibrary") != null && !abiMismatchProbe) {
            throw new IllegalStateException("The explicit library override must be absent");
        }
        if (args.length == 1 && args[0].equals("version")) {
            System.out.println("VERSION=" + RuntimeMetadata.version());
            return;
        }
        NativeExtensionRegistry.ensureLoaded();
        System.out.println("ABI=" + NativeExtensionRegistry.nativeAbiVersion());
    }
}
