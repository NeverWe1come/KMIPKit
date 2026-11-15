"""Typed Device Credential parity and ownership behavior."""

from __future__ import annotations

import unittest

import kmipkit
from kmipkit import errors, ttlv


DEVICE_SERIAL_NUMBER_TAG = 0x0042_00B0
PASSWORD_TAG = 0x0042_00A1
DEVICE_IDENTIFIER_TAG = 0x0042_00A2
NETWORK_IDENTIFIER_TAG = 0x0042_00AB
MACHINE_IDENTIFIER_TAG = 0x0042_00A9
MEDIA_IDENTIFIER_TAG = 0x0042_00AA
EXTENSION_CHILD_TAG = 0x0054_0001


def append(structure, raw_tag_value, value, limits):
    raw_tag = ttlv.raw_tag_from_raw(raw_tag_value)
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


def device_structure(include_identifiers):
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    if include_identifiers:
        structure = append(
            structure,
            DEVICE_SERIAL_NUMBER_TAG,
            ttlv.ttlv_value_text_string(b"", limits),
            limits,
        )
    structure = append(
        structure,
        PASSWORD_TAG,
        ttlv.ttlv_value_text_string(b"sensitive-sentinel", limits),
        limits,
    )
    structure = append(
        structure,
        DEVICE_IDENTIFIER_TAG,
        ttlv.ttlv_value_text_string(b"device-id", limits),
        limits,
    )
    if include_identifiers:
        for tag, value in (
            (NETWORK_IDENTIFIER_TAG, b"network-id"),
            (MACHINE_IDENTIFIER_TAG, b"machine-id"),
            (MEDIA_IDENTIFIER_TAG, b"media-id"),
        ):
            structure = append(
                structure, tag, ttlv.ttlv_value_text_string(value, limits), limits
            )
        structure = append(
            structure, EXTENSION_CHILD_TAG, ttlv.ttlv_value_integer(7), limits
        )
    limits.close()
    return structure


def tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
    result = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        item = ttlv.ttlv_structure_view_item_at(view, index)
        try:
            tag = ttlv.ttlv_item_view_tag(item)
            try:
                result.append(ttlv.tag_value(tag))
            finally:
                tag.close()
        finally:
            item.close()
    return tuple(result)


def values_match(view: ttlv.TtlvStructureView) -> bool:
    expected_text = (
        b"",
        b"sensitive-sentinel",
        b"device-id",
        b"network-id",
        b"machine-id",
        b"media-id",
    )
    for index, expected in enumerate(expected_text):
        item = ttlv.ttlv_structure_view_item_at(view, index)
        try:
            with ttlv.ttlv_item_view_value(item) as value:
                if ttlv.ttlv_value_view_type(value) is not ttlv.ItemType.TextString:
                    return False
                actual_length = ttlv.ttlv_value_view_byte_length(value)
                if actual_length != len(expected):
                    return False
                actual = bytes(
                    ttlv.ttlv_value_view_byte_at(value, offset)
                    for offset in range(actual_length)
                )
                if actual != expected:
                    return False
        finally:
            item.close()

    item = ttlv.ttlv_structure_view_item_at(view, 6)
    try:
        with ttlv.ttlv_item_view_value(item) as value:
            return (
                ttlv.ttlv_value_view_type(value) is ttlv.ItemType.Integer
                and ttlv.ttlv_value_view_integer(value) == 7
            )
    finally:
        item.close()


class DeviceCredentialTests(unittest.TestCase):
    def test_device_credential_preserves_table_412_fields_and_scoped_views(self) -> None:
        facade = getattr(kmipkit.credentials, "DeviceCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        source = device_structure(True)
        source_view = ttlv.ttlv_structure_view(source)
        try:
            with self.assertRaises(errors.InvalidInputError):
                facade.from_ttlv(source)
            self.assertEqual(7, ttlv.ttlv_structure_view_item_count(source_view))
        finally:
            source_view.close()

        credential = facade.from_ttlv(source)
        self.assertTrue(source.closed)
        self.assertNotIn("sensitive-sentinel", repr(credential))
        borrowed = []

        def inspect(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
            borrowed.append(view)
            with self.assertRaises(errors.InvalidInputError):
                credential.into_ttlv()
            self.assertTrue(values_match(view))
            return tags(view)

        expected = (
            DEVICE_SERIAL_NUMBER_TAG,
            PASSWORD_TAG,
            DEVICE_IDENTIFIER_TAG,
            NETWORK_IDENTIFIER_TAG,
            MACHINE_IDENTIFIER_TAG,
            MEDIA_IDENTIFIER_TAG,
            EXTENSION_CHILD_TAG,
        )
        self.assertEqual(expected, credential.with_ttlv(inspect))
        with self.assertRaises(errors.InvalidInputError):
            ttlv.ttlv_structure_view_item_count(borrowed[0])

        restored = credential.into_ttlv()
        self.assertTrue(credential.closed)
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(expected, tags(view))
            self.assertTrue(values_match(view))
        finally:
            view.close()
            restored.close()

    def test_device_credential_requires_one_of_four_identifier_fields(self) -> None:
        facade = getattr(kmipkit.credentials, "DeviceCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return
        with self.assertRaises(errors.CredentialValidationError) as raised:
            facade.from_ttlv(device_structure(False))
        self.assertEqual(
            errors.CredentialValidationErrorKind.MISSING_FIELD,
            raised.exception.kind,
        )
        self.assertNotIn("sensitive-sentinel", str(raised.exception))

    def test_credential_validation_error_preserves_unknown_kind_values(self) -> None:
        error = errors.CredentialValidationError(0x7F)

        self.assertEqual(0x7F, error.kind)
        self.assertNotIn("sensitive-sentinel", str(error))


if __name__ == "__main__":
    unittest.main()
