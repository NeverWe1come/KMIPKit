package org.kmipkit;

/** Safe, non-secret runtime metadata exposed by the KMIPKit native ABI. */
public final class RuntimeMetadata {
    static {
        NativeExtensionRegistry.ensureLoaded();
    }

    private RuntimeMetadata() {}

    /** Product and stable ABI versions reported by the loaded native library. */
    public record Version(int major, int minor, int patch, int abiVersion) {}

    /** Safe C ABI error values; integer codes preserve values from future library versions. */
    public record ErrorSnapshot(int status, int category, int cause, int delivery) {}

    /** Returns the loaded native package version and stable ABI revision. */
    public static native Version version();

    /** Returns whether the named capability has an unblocked C mapping. */
    public static native boolean capabilityIsAvailable(String capabilityId);

    /** Returns the calling thread's error metadata without clearing it. */
    public static native ErrorSnapshot lastErrorSnapshot();

    /** Clears the calling thread's last C ABI error metadata. */
    public static native void clearLastError();
}
