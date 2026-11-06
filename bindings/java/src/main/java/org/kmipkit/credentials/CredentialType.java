package org.kmipkit.credentials;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** A raw-preserving KMIP Credential Type discriminator. */
public final class CredentialType {
    public static final long USERNAME_AND_PASSWORD = 1L;
    public static final long DEVICE = 2L;
    public static final long ATTESTATION = 3L;
    public static final long ONE_TIME_PASSWORD = 4L;
    public static final long HASHED_PASSWORD = 5L;
    public static final long TICKET = 6L;

    private final long raw;

    private CredentialType(long raw) {
        this.raw = raw;
    }

    /** Create a typed view of an unsigned 32-bit Credential Type value. */
    public static CredentialType fromRaw(long raw) {
        if (raw < 0 || raw > 0xffff_ffffL) {
            throw new InvalidInputException("Credential Type is outside its unsigned 32-bit range");
        }
        NativeExtensionRegistry.ensureLoaded();
        long canonicalRaw = NativeExtensionRegistry.credentialTypeFromRaw(raw);
        return new CredentialType(canonicalRaw);
    }

    /** Return the exact unsigned 32-bit value, including unknown values. */
    public long raw() {
        return raw;
    }

    /** Return the known, Extensions, or Unknown category for this value. */
    public String kind() {
        return switch ((int) raw) {
            case 1 -> "UsernameAndPassword";
            case 2 -> "Device";
            case 3 -> "Attestation";
            case 4 -> "OneTimePassword";
            case 5 -> "HashedPassword";
            case 6 -> "Ticket";
            default -> raw >= 0x8000_0000L && raw <= 0x8fff_ffffL ? "Extensions" : "Unknown";
        };
    }
}
