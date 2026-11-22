package org.kmipkit.protocol;

import org.kmipkit.protocol.KmipValues.CryptographicUsageMask;

/** Flags for the KMIP Cryptographic Usage Mask. */
public enum KeyUsage {
    SIGN(CryptographicUsageMask.SIGN),
    VERIFY(CryptographicUsageMask.VERIFY),
    ENCRYPT(CryptographicUsageMask.ENCRYPT),
    DECRYPT(CryptographicUsageMask.DECRYPT),
    WRAP_KEY(CryptographicUsageMask.WRAP_KEY),
    UNWRAP_KEY(CryptographicUsageMask.UNWRAP_KEY),
    MAC_GENERATE(CryptographicUsageMask.MAC_GENERATE),
    MAC_VERIFY(CryptographicUsageMask.MAC_VERIFY),
    DERIVE_KEY(CryptographicUsageMask.DERIVE_KEY),
    KEY_AGREEMENT(CryptographicUsageMask.KEY_AGREEMENT),
    CERTIFICATE_SIGN(CryptographicUsageMask.CERTIFICATE_SIGN),
    CRL_SIGN(CryptographicUsageMask.CRL_SIGN),
    AUTHENTICATE(CryptographicUsageMask.AUTHENTICATE),
    UNRESTRICTED(CryptographicUsageMask.UNRESTRICTED),
    FPE_ENCRYPT(CryptographicUsageMask.FPE_ENCRYPT),
    FPE_DECRYPT(CryptographicUsageMask.FPE_DECRYPT);

    private final int bit;

    KeyUsage(long bit) {
        this.bit = (int) bit;
    }

    /** Returns this flag's bit in the TTLV Integer representation. */
    public int bit() {
        return bit;
    }

    /** Combines flags into the original TTLV Integer bit pattern. */
    public static int mask(KeyUsage... flags) {
        int mask = 0;
        for (KeyUsage flag : flags) {
            mask |= flag.bit;
        }
        return mask;
    }
}
