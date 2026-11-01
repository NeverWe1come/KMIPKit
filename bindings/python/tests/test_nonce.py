"""Server-sourced Nonce ownership and exact TTLV preservation."""

from __future__ import annotations

import unittest

import kmipkit
from kmipkit import errors, ttlv


NONCE_ID_TAG = 0x0042_00C9
NONCE_VALUE_TAG = 0x0042_00CA
NONCE_ID = bytes((0x10, 0x00, 0xFF))
NONCE_VALUE = bytes((0x80, 0x01, 0x7F, 0x00))


def nonce_structure() -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    try:
        for raw, value_bytes in ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)):
            raw_tag = ttlv.raw_tag_from_raw(raw)
            try:
                tag = ttlv.raw_tag_try_checked(raw_tag)
            finally:
                raw_tag.close()
            value = ttlv.ttlv_value_byte_string(value_bytes, limits)
            try:
                item = ttlv.ttlv_item_create(tag, value, limits)
            finally:
                tag.close()
                value.close()
            try:
                structure = ttlv.ttlv_structure_with_item(structure, item, limits)
            finally:
                item.close()
        return structure
    except BaseException:
        structure.close()
        raise
    finally:
        limits.close()


def read_structure(view: ttlv.TtlvStructureView) -> tuple[tuple[int, bytes], ...]:
    result: list[tuple[int, bytes]] = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        item = ttlv.ttlv_structure_view_item_at(view, index)
        try:
            tag = ttlv.ttlv_item_view_tag(item)
            value = ttlv.ttlv_item_view_value(item)
            try:
                payload = bytes(
                    ttlv.ttlv_value_view_byte_at(value, byte_index)
                    for byte_index in range(ttlv.ttlv_value_view_byte_length(value))
                )
                result.append((ttlv.tag_value(tag), payload))
            finally:
                tag.close()
                value.close()
        finally:
            item.close()
    return tuple(result)


class NonceTests(unittest.TestCase):
    def test_nonce_callback_preserves_bytes_and_ttlv_ownership_moves_without_copy(self) -> None:
        facade = getattr(kmipkit.credentials, "Nonce", None)
        self.assertIsNotNone(
            facade,
            "the Python facade must expose the canonical server-sourced Nonce model",
        )
        if facade is None:
            return
        with self.assertRaises(errors.InvalidInputError):
            facade()

        structure = nonce_structure()
        nonce = facade.from_ttlv(structure)
        self.assertTrue(structure.closed, "from_ttlv must transfer the original Structure")
        self.assertNotIn(NONCE_VALUE.hex(), repr(nonce))
        borrowed: list[ttlv.TtlvStructureView] = []

        def inspect(view: ttlv.TtlvStructureView) -> tuple[tuple[int, bytes], ...]:
            borrowed.append(view)
            return read_structure(view)

        self.assertEqual(
            ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
            nonce.with_ttlv(inspect),
        )
        self.assertEqual(1, len(borrowed))
        with self.assertRaises(errors.InvalidInputError):
            ttlv.ttlv_structure_view_item_count(borrowed[0])

        restored = nonce.into_ttlv()
        self.assertTrue(nonce.closed, "into_ttlv must consume the Nonce handle")
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(
                ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
                read_structure(view),
            )
        finally:
            view.close()
            restored.close()

    def test_nonce_rejects_missing_required_value_without_disclosing_nonce_bytes(self) -> None:
        facade = getattr(kmipkit.credentials, "Nonce", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        structure = ttlv.ttlv_structure_create()
        with self.assertRaises(errors.CredentialValidationError) as raised:
            facade.from_ttlv(structure)
        self.assertEqual(
            errors.CredentialValidationErrorKind.MISSING_FIELD,
            raised.exception.kind,
        )
        self.assertNotIn(NONCE_VALUE.hex(), str(raised.exception))
        self.assertTrue(structure.closed)

    def test_failed_nonce_conversion_retains_source_while_its_view_is_live(self) -> None:
        source = nonce_structure()
        view = ttlv.ttlv_structure_view(source)
        try:
            with self.assertRaises(errors.InvalidInputError):
                kmipkit.credentials.Nonce.from_ttlv(source)
            self.assertFalse(source.closed)
            self.assertEqual(
                ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
                read_structure(view),
            )
        finally:
            view.close()

        nonce = kmipkit.credentials.Nonce.from_ttlv(source)
        try:
            self.assertEqual(
                ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
                nonce.with_ttlv(read_structure),
            )
        finally:
            nonce.close()

    def test_nonce_remains_usable_when_consumption_is_attempted_inside_its_scoped_view(self) -> None:
        nonce = kmipkit.credentials.Nonce.from_ttlv(nonce_structure())

        def inspect(view: ttlv.TtlvStructureView) -> None:
            with self.assertRaises(errors.InvalidInputError):
                nonce.into_ttlv()
            self.assertEqual(
                ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
                read_structure(view),
            )

        nonce.with_ttlv(inspect)
        self.assertEqual(
            ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
            nonce.with_ttlv(read_structure),
        )

        restored = nonce.into_ttlv()
        self.assertTrue(nonce.closed)
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(
                ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
                read_structure(view),
            )
        finally:
            view.close()
            restored.close()

    def test_nonce_callback_cannot_reset_the_scoped_view_guard(self) -> None:
        nonce = kmipkit.credentials.Nonce.from_ttlv(nonce_structure())

        def inspect(view: ttlv.TtlvStructureView) -> None:
            for attribute in ("_active_views", "_Nonce__active_views"):
                with self.subTest(attribute=attribute), self.assertRaises(AttributeError):
                    setattr(nonce, attribute, 0)
            kmipkit.credentials._ACTIVE_NONCE_VIEWS[nonce] = 0
            with self.assertRaises(errors.InvalidInputError):
                nonce.into_ttlv()
            self.assertEqual(
                ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
                read_structure(view),
            )

        nonce.with_ttlv(inspect)
        self.assertEqual(
            ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
            nonce.with_ttlv(read_structure),
        )
        restored = nonce.into_ttlv()
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(
                ((NONCE_ID_TAG, NONCE_ID), (NONCE_VALUE_TAG, NONCE_VALUE)),
                read_structure(view),
            )
        finally:
            view.close()
            restored.close()


if __name__ == "__main__":
    unittest.main()
