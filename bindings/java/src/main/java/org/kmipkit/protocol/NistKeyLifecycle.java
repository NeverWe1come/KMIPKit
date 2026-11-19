package org.kmipkit.protocol;

import org.kmipkit.NativeExtensionRegistry;

/** NIST SP 800-57 key lifecycle cryptographic usage state validation. */
public final class NistKeyLifecycle {
    private NistKeyLifecycle() {}

    public static boolean canOriginate(long stateEnum) {
        NativeExtensionRegistry.ensureLoaded();
        return NativeExtensionRegistry.nistKeyLifecycleCanOriginate(stateEnum);
    }

    public static boolean canProcess(long stateEnum) {
        NativeExtensionRegistry.ensureLoaded();
        return NativeExtensionRegistry.nistKeyLifecycleCanProcess(stateEnum);
    }
}
