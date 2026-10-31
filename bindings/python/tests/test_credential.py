"""Generic Credential envelope ownership and raw type preservation."""

from __future__ import annotations

import unittest

import kmipkit
from kmipkit import errors, ttlv


CREDENTIAL_TYPE_TAG = 0x0042_0024
CREDENTIAL_VALUE_TAG = 0x0042_0025
CREDENTIAL_PAYLOAD_TAG = 0x0042_002A
CREDENTIAL_PAYLOAD_VALUE = 0x13579


def read_tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
    tags = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        item = ttlv.ttlv_structure_view_item_at(view, index)
        try:
            tag = ttlv.ttlv_item_view_tag(item)
            try:
                tags.append(ttlv.tag_value(tag))
            finally:
                tag.close()
        finally:
            item.close()
    return tuple(tags)


def read_payload(view: ttlv.TtlvStructureView) -> tuple[int, int]:
    outer_item = ttlv.ttlv_structure_view_item_at(view, 1)
    try:
        outer_value = ttlv.ttlv_item_view_value(outer_item)
        try:
            payload = ttlv.ttlv_value_view_structure(outer_value)
            try:
                payload_item = ttlv.ttlv_structure_view_item_at(payload, 0)
                try:
                    tag = ttlv.ttlv_item_view_tag(payload_item)
                    try:
                        payload_tag = ttlv.tag_value(tag)
                    finally:
                        tag.close()
                    value = ttlv.ttlv_item_view_value(payload_item)
                    try:
                        payload_value = ttlv.ttlv_value_view_integer(value)
                    finally:
                        value.close()
                    return payload_tag, payload_value
                finally:
                    payload_item.close()
            finally:
                payload.close()
        finally:
            outer_value.close()
    finally:
        outer_item.close()


def credential_structure(raw_type: int) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    try:
        raw_tag = ttlv.raw_tag_from_raw(CREDENTIAL_TYPE_TAG)
        try:
            tag = ttlv.raw_tag_try_checked(raw_tag)
        finally:
            raw_tag.close()
        value = ttlv.ttlv_value_enumeration(raw_type)
        try:
            item = ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            value.close()
        structure = ttlv.ttlv_structure_with_item(structure, item, limits)

        value_structure = ttlv.ttlv_structure_create()
        payload_tag = ttlv.raw_tag_from_raw(CREDENTIAL_PAYLOAD_TAG)
        try:
            checked_payload_tag = ttlv.raw_tag_try_checked(payload_tag)
        finally:
            payload_tag.close()
        payload_value = ttlv.ttlv_value_integer(CREDENTIAL_PAYLOAD_VALUE)
        try:
            payload_item = ttlv.ttlv_item_create(
                checked_payload_tag, payload_value, limits
            )
        finally:
            checked_payload_tag.close()
            payload_value.close()
        value_structure = ttlv.ttlv_structure_with_item(
            value_structure, payload_item, limits
        )
        value = ttlv.ttlv_value_structure(value_structure, limits)
        raw_tag = ttlv.raw_tag_from_raw(CREDENTIAL_VALUE_TAG)
        try:
            tag = ttlv.raw_tag_try_checked(raw_tag)
        finally:
            raw_tag.close()
        try:
            item = ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            value.close()
        return ttlv.ttlv_structure_with_item(structure, item, limits)
    except BaseException:
        structure.close()
        raise
    finally:
        limits.close()


class CredentialTests(unittest.TestCase):
    def test_credential_preserves_unknown_type_and_original_ordered_ttlv(self) -> None:
        facade = getattr(kmipkit.credentials, "Credential", None)
        self.assertIsNotNone(
            facade,
            "the Python facade must expose the generic validated Credential envelope",
        )
        if facade is None:
            return

        raw_type = 0xF123_4567
        source = credential_structure(raw_type)
        credential = facade.from_ttlv(source)
        self.assertTrue(source.closed, "from_ttlv must consume the original Structure")
        self.assertEqual(raw_type, credential.credential_type_raw())
        self.assertEqual(raw_type, credential.credential_type.raw())
        self.assertNotIn(f"{CREDENTIAL_VALUE_TAG:08x}", repr(credential))

        borrowed: list[ttlv.TtlvStructureView] = []

        def inspect(view: ttlv.TtlvStructureView) -> tuple[tuple[int, ...], tuple[int, int]]:
            borrowed.append(view)
            return read_tags(view), read_payload(view)

        self.assertEqual(
            (
                (CREDENTIAL_TYPE_TAG, CREDENTIAL_VALUE_TAG),
                (CREDENTIAL_PAYLOAD_TAG, CREDENTIAL_PAYLOAD_VALUE),
            ),
            credential.with_ttlv(inspect),
        )
        self.assertEqual(1, len(borrowed))
        with self.assertRaises(errors.InvalidInputError):
            ttlv.ttlv_structure_view_item_count(borrowed[0])

        restored = credential.into_ttlv()
        self.assertTrue(credential.closed)
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual((CREDENTIAL_TYPE_TAG, CREDENTIAL_VALUE_TAG), read_tags(view))
            self.assertEqual(
                (CREDENTIAL_PAYLOAD_TAG, CREDENTIAL_PAYLOAD_VALUE), read_payload(view)
            )
        finally:
            view.close()
            restored.close()

    def test_credential_cannot_be_consumed_during_a_scoped_view(self) -> None:
        facade = getattr(kmipkit.credentials, "Credential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return
        credential = facade.from_ttlv(credential_structure(1))

        def inspect(view: ttlv.TtlvStructureView) -> None:
            with self.assertRaises(errors.InvalidInputError):
                credential.into_ttlv()
            self.assertEqual(2, ttlv.ttlv_structure_view_item_count(view))

        credential.with_ttlv(inspect)
        self.assertEqual(2, credential.with_ttlv(ttlv.ttlv_structure_view_item_count))
        restored = credential.into_ttlv()
        self.assertTrue(credential.closed)
        restored.close()

    def test_failed_credential_conversion_retains_source_while_its_view_is_live(self) -> None:
        facade = getattr(kmipkit.credentials, "Credential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return
        source = credential_structure(1)
        view = ttlv.ttlv_structure_view(source)
        try:
            with self.assertRaises(errors.InvalidInputError):
                facade.from_ttlv(source)
            self.assertEqual(2, ttlv.ttlv_structure_view_item_count(view))
        finally:
            view.close()

        credential = facade.from_ttlv(source)
        try:
            self.assertEqual(1, credential.credential_type_raw())
        finally:
            credential.close()

    def test_credential_rejects_malformed_input_without_disclosing_contents(self) -> None:
        facade = getattr(kmipkit.credentials, "Credential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return
        source = ttlv.ttlv_structure_create()
        marker = b"credential-private-marker"
        raw_tag = ttlv.raw_tag_from_raw(CREDENTIAL_TYPE_TAG)
        try:
            tag = ttlv.raw_tag_try_checked(raw_tag)
        finally:
            raw_tag.close()
        limits = ttlv.codec_limits_defaults()
        value = ttlv.ttlv_value_text_string(marker, limits)
        try:
            item = ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()
            value.close()
            limits.close()
        limits = ttlv.codec_limits_defaults()
        try:
            source = ttlv.ttlv_structure_with_item(source, item, limits)
        finally:
            limits.close()
        with self.assertRaises(errors.InvalidInputError) as raised:
            facade.from_ttlv(source)
        self.assertNotIn(marker.decode(), str(raised.exception))


if __name__ == "__main__":
    unittest.main()
