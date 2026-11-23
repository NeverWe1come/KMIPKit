"""Named flags for the KMIP Cryptographic Usage Mask."""

from enum import IntFlag

from .protocol_values import CryptographicUsageMask


class KeyUsage(IntFlag):
    """Combine usage flags with the bitwise OR operator."""

    SIGN = CryptographicUsageMask.SIGN
    VERIFY = CryptographicUsageMask.VERIFY
    ENCRYPT = CryptographicUsageMask.ENCRYPT
    DECRYPT = CryptographicUsageMask.DECRYPT
    WRAP_KEY = CryptographicUsageMask.WRAP_KEY
    UNWRAP_KEY = CryptographicUsageMask.UNWRAP_KEY
    MAC_GENERATE = CryptographicUsageMask.MAC_GENERATE
    MAC_VERIFY = CryptographicUsageMask.MAC_VERIFY
    DERIVE_KEY = CryptographicUsageMask.DERIVE_KEY
    KEY_AGREEMENT = CryptographicUsageMask.KEY_AGREEMENT
    CERTIFICATE_SIGN = CryptographicUsageMask.CERTIFICATE_SIGN
    CRL_SIGN = CryptographicUsageMask.CRL_SIGN
    AUTHENTICATE = CryptographicUsageMask.AUTHENTICATE
    UNRESTRICTED = CryptographicUsageMask.UNRESTRICTED
    FPE_ENCRYPT = CryptographicUsageMask.FPE_ENCRYPT
    FPE_DECRYPT = CryptographicUsageMask.FPE_DECRYPT


__all__ = ["KeyUsage"]
