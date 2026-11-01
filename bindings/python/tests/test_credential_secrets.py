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

        def mutate_temporary_copy(value: memoryview) -> bytes:
            self.assertTrue(value.readonly)
            self.assertIsInstance(value.obj, bytearray)
            value.obj[0] = 0xAA
            return value.tobytes()

        self.assertNotEqual(expected, secret.with_bytes(mutate_temporary_copy))
        self.assertEqual(expected, secret.with_bytes(bytes))

        def fail(value: memoryview) -> None:
            observed.append(value.obj)
            borrowed_views.append(value)
            raise RuntimeError("callback failed")

        with self.assertRaisesRegex(RuntimeError, "callback failed") as raised:
            secret.with_bytes(fail)
        self.assertEqual(bytearray(len(expected)), observed[-1])
        with self.assertRaises(ValueError):
            borrowed_views[-1].tobytes()
        traceback = raised.exception.__traceback__
        saw_native_callback = False
        while traceback is not None:
            if traceback.tb_frame.f_code.co_name == "invoke":
                self.assertIsNone(traceback.tb_frame.f_locals["data"])
                self.assertEqual(0, traceback.tb_frame.f_locals["length"])
                saw_native_callback = True
            traceback = traceback.tb_next
        # The exception is re-raised only after the C callback has returned,
        # so Python does not retain the native callback frame in its traceback.
        self.assertFalse(saw_native_callback)

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

    def test_secret_bytes_callback_can_reenter_and_close_owner(self) -> None:
        expected = b"scoped value"
        secret = kmipkit.credentials.SecretBytes.from_bytes(expected)

        def reenter_and_close(value: memoryview) -> bytes:
            nested = secret.with_bytes(bytes)
            secret.close()
            return nested + value.tobytes()

        self.assertEqual(expected + expected, secret.with_bytes(reenter_and_close))
        self.assertTrue(secret.closed)

    def test_secret_text_is_strict_utf8_and_redacted(self) -> None:
        expected = "snowman \u2603\x00 caf\u00e9"
        secret = kmipkit.credentials.SecretText.from_text(expected)
        self.assertEqual("SecretText([REDACTED])", repr(secret))

        observed: list[str] = []

        def record_and_return(value: str) -> str:
            observed.append(value)
            return value

        self.assertEqual(expected, secret.with_str(record_and_return))
        self.assertEqual([expected], observed)
        self.assertEqual(expected, secret.with_str(lambda value: value))

        def fail(_value: str) -> None:
            raise RuntimeError("text callback failed")

        with self.assertRaisesRegex(RuntimeError, "text callback failed") as raised:
            secret.with_str(fail)
        traceback = raised.exception.__traceback__
        while traceback is not None:
            if traceback.tb_frame.f_code.co_name == "invoke":
                self.assertIsNone(traceback.tb_frame.f_locals["utf8"])
                self.assertEqual(0, traceback.tb_frame.f_locals["length"])
            traceback = traceback.tb_next

        with self.assertRaises(errors.InvalidInputError):
            kmipkit.credentials.SecretText.from_text("\ud800")

        with secret:
            self.assertFalse(secret.closed)
        self.assertTrue(secret.closed)
        secret.close()


if __name__ == "__main__":
    unittest.main()
