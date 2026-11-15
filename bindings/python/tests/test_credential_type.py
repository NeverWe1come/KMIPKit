"""Open Credential Type values retain their raw KMIP enumeration."""

from __future__ import annotations

import unittest

import kmipkit


class CredentialTypeTests(unittest.TestCase):
    def test_from_raw_preserves_known_extension_and_unknown_values(self) -> None:
        credential_type = getattr(kmipkit, "CredentialType", None)
        self.assertIsNotNone(
            credential_type,
            "the Python facade must expose the canonical CredentialType model",
        )
        self.assertEqual(1, credential_type.USERNAME_AND_PASSWORD)
        self.assertEqual(2, credential_type.DEVICE)
        self.assertEqual(3, credential_type.ATTESTATION)
        self.assertEqual(4, credential_type.ONE_TIME_PASSWORD)
        self.assertEqual(5, credential_type.HASHED_PASSWORD)
        self.assertEqual(6, credential_type.TICKET)

        cases = (
            (1, "UsernameAndPassword"),
            (2, "Device"),
            (3, "Attestation"),
            (4, "OneTimePassword"),
            (5, "HashedPassword"),
            (6, "Ticket"),
            (0x8000_0009, "Extensions"),
            (7, "Unknown"),
            (0xFFFF_FFFF, "Unknown"),
        )
        for raw, expected_kind in cases:
            with self.subTest(raw=raw):
                value = credential_type.from_raw(raw)
                self.assertEqual(value.raw(), raw)
                self.assertEqual(value.kind, expected_kind)

    def test_from_raw_rejects_non_uint32_values(self) -> None:
        credential_type = getattr(kmipkit, "CredentialType", None)
        self.assertIsNotNone(credential_type)
        for raw in (-1, 0x1_0000_0000, True):
            with self.subTest(raw=raw), self.assertRaises(kmipkit.errors.InvalidInputError):
                credential_type.from_raw(raw)


if __name__ == "__main__":
    unittest.main()
