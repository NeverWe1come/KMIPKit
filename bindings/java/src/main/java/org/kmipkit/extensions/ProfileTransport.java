package org.kmipkit.extensions;

/** Closed KMIPKit ABI selector for client profile transport validation. */
public final class ProfileTransport {
    /** Raw TTLV records over TLS. */
    public static final ProfileTransport RAW_TTLV_TLS = new ProfileTransport(1);

    /** TTLV over HTTPS using HTTP/1.1. */
    public static final ProfileTransport HTTPS = new ProfileTransport(2);

    private final int abiValue;

    private ProfileTransport(int abiValue) {
        this.abiValue = abiValue;
    }

    int abiValue() {
        return abiValue;
    }
}
