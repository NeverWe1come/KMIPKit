package org.kmipkit.extensions;

/** Local Register attribute-handling context for an Opaque Object. */
public enum OpaqueObjectContext {
    /** Keep generic opaque behavior and do not require cryptographic attributes. */
    GENERIC(1),
    /** Apply Managed Cryptographic Object Register attribute requirements. */
    MANAGED_CRYPTOGRAPHIC_OBJECT(2);

    private final int nativeValue;

    OpaqueObjectContext(int nativeValue) {
        this.nativeValue = nativeValue;
    }

    int nativeValue() {
        return nativeValue;
    }
}
