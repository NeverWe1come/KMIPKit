package org.kmipkit.protocol;

/**
 * KMIP 3.0 Ephemeral Mode enumeration values (OASIS KMIP 3.0 §3.44).
 */
public final class EphemeralMode {
    private EphemeralMode() {}

    /** No ephemeral handling. */
    public static final long NO = 0x00000001L;

    /** Destroy immediately after request completes. */
    public static final long IMMEDIATE = 0x00000002L;

    /** Destroy on success; retain on failure for diagnostics. */
    public static final long RETAIN_ON_FAILURE = 0x00000003L;

    /**
     * Returns the assigned catalog name, or null for an unknown raw value.
     */
    public static String name(long raw) {
        if (raw < 0 || raw > 0xFFFF_FFFFL) {
            return null;
        }
        return switch ((int) raw) {
            case 0x00000001 -> "No";
            case 0x00000002 -> "Immediate";
            case 0x00000003 -> "Retain On Failure";
            default -> null;
        };
    }
}
