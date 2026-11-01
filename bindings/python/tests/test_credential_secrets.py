"""Scoped ownership and foreign-runtime copy behavior for credential secrets."""

from __future__ import annotations

import unittest

import kmipkit
from kmipkit import errors, ttlv


class CredentialSecretTests(unittest.TestCase):
    def test_secret_bytes_callback_is_read_only_scoped_and_cleared(self) -> None:
        expected = bytes((0x00, 0x7F, 0xFF, 0x00))
        source = bytearray(expected)
        secret = kmipkit.credentials.SecretBytes.from_bytes(source)
        self.assertEqual("SecretBytes([REDACTED])", repr(secret))
        self.assertEqual(expected, bytes(source))

        observed: list[bytearray] = []
        borrowed_views: list[memoryview] = []

        def inspect(value: memoryview) -> bytes:
            self.assertTrue(value.readonly)
            observed.append(value.obj)
            borrowed_views.append(value)
            return value.tobytes()

        self.assertEqual(expected, secret.with_bytes(inspect))
        self.assertEqual(bytearray(len(expected)), observed[-1])
        with self.assertRaises(ValueError):
            borrowed_views[-1].tobytes()

        def fail(value: memoryview) -> None:
            observed.append(value.obj)
            borrowed_views.append(value)
            raise RuntimeError("callback failed")

        with self.assertRaisesRegex(RuntimeError, "callback failed"):
            secret.with_bytes(fail)
        self.assertEqual(bytearray(len(expected)), observed[-1])
        with self.assertRaises(ValueError):
            borrowed_views[-1].tobytes()

        value = secret.into_ttlv_value()
        self.assertTrue(secret.closed)
        view = ttlv.ttlv_value_view(value)
        try:
            self.assertEqual(ttlv.ItemType.ByteString, ttlv.ttlv_value_view_type(view))
            self.assertEqual(
                expected,
                bytes(
                    ttlv.ttlv_value_view_byte_at(view, index)
                    for index in range(ttlv.ttlv_value_view_byte_length(view))
                ),
            )
        finally:
            view.close()
            value.close()
        with self.assertRaises(errors.InvalidInputError):
            secret.with_bytes(inspect)
        secret.close()

    def test_secret_text_is_strict_utf8_and_redacted(self) -> None:
        expected = "snowman \u2603\x00 caf\u00e9"
        secret = kmipkit.credentials.SecretText.from_text(expected)
        self.assertEqual("SecretText([REDACTED])", repr(secret))

        observed: list[str] = []
        self.assertEqual(expected, secret.with_str(observed.append))
        self.assertEqual([expected], observed)

        with self.assertRaises(errors.InvalidInputError):
            kmipkit.credentials.SecretText.from_text("\ud800")

        with secret:
            self.assertFalse(secret.closed)
        self.assertTrue(secret.closed)
        secret.close()


if __name__ == "__main__":
    unittest.main()
