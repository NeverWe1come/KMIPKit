package org.kmipkit;

import java.util.List;

/** Safe, non-secret runtime metadata exposed by the KMIPKit native ABI. */
public final class RuntimeMetadata {
    static {
        NativeExtensionRegistry.ensureLoaded();
    }

    private RuntimeMetadata() {}

    /** Product and stable ABI versions reported by the loaded native library. */
    public record Version(int major, int minor, int patch, int abiVersion) {}

    /** Internal C ABI error values; integer codes preserve future native values. */
    record ErrorSnapshot(int status, int category, int cause, int delivery) {}

    /** Returns the loaded native package version and stable ABI revision. */
    public static native Version version();

    /** Returns whether the named capability has an unblocked C mapping.
     * @throws InvalidInputException if the identifier contains an unpaired surrogate
     * @throws ResourceLimitException if the UTF-8 identifier exceeds 128 bytes
     */
    public static native boolean capabilityIsAvailable(String capabilityId);

    /**
     * Returns the local API-level taxonomy in canonical order. This metadata query performs no
     * network call, is distinct from the KMIP Discover Versions operation, and does not claim that
     * every KMIP operation is mapped or complete.
     */
    public static List<String> apiLevels() {
        return List.of(apiLevelsNative());
    }

    private static native String[] apiLevelsNative();

    static native ErrorSnapshot lastErrorSnapshot();

    static native void clearLastError();

}
