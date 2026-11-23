package org.kmipkit.protocol;

/**
 * KMIP 3.0 Ephemeral Mode enumeration values (OASIS KMIP 3.0 §11.21).
 */
public final class EphemeralMode {
    private EphemeralMode() {}

    /** Omit Data. */
    public static final long DATA = 0x00000001L;

    /** Return an empty payload. */
    public static final long EMPTY = 0x00000002L;

    /** Return only Unique Identifier. */
    public static final long UNIQUE_IDENTIFIER = 0x00000003L;

    /**
     * Returns the assigned catalog name, or null for an unknown raw value.
     */
    public static String name(long raw) {
        if (raw < 0 || raw > 0xFFFF_FFFFL) {
            return null;
        }
        return switch ((int) raw) {
            case 0x00000001 -> "Data";
            case 0x00000002 -> "Empty";
            case 0x00000003 -> "Unique Identifier";
            default -> null;
        };
    }
}
