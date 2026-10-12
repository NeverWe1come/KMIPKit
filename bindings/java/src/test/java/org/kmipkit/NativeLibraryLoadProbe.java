package org.kmipkit;

/** Child-JVM entry point for exercising default bundled native-library loading. */
public final class NativeLibraryLoadProbe {
    private NativeLibraryLoadProbe() {}

    public static void main(String[] args) {
        if (System.getProperty("kmipkit.nativeLibrary") != null) {
            throw new IllegalStateException("The explicit library override must be absent");
        }
        NativeExtensionRegistry.ensureLoaded();
        System.out.println("ABI=" + NativeExtensionRegistry.nativeAbiVersion());
    }
}
