from __future__ import annotations

import unittest

import kmipkit.credentials
from kmipkit import errors, ttlv


USERNAME_TAG = 0x0042_0099
ONE_TIME_PASSWORD_TAG = 0x0042_0156
OPAQUE_CHILD_TAG = 0x0054_0001


def append(structure, raw_tag: int, value: ttlv.Value, limits: ttlv.CodecLimits):
    raw = ttlv.raw_tag_from_raw(raw_tag)
    try:
        tag = ttlv.raw_tag_try_checked(raw)
    finally:
        raw.close()
    try:
        item = ttlv.ttlv_item_create(tag, value, limits)
    finally:
        tag.close()
        value.close()
    return ttlv.ttlv_structure_with_item(structure, item, limits)


def credential_value_structure(raw_type: int, *, include_username: bool = True):
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    if raw_type == 4:
        if include_username:
            structure = append(
                structure,
                USERNAME_TAG,
                ttlv.ttlv_value_text_string(b"alice", limits),
                limits,
            )
        structure = append(
            structure,
            ONE_TIME_PASSWORD_TAG,
            ttlv.ttlv_value_text_string(b"credential-value-secret", limits),
            limits,
        )
    else:
        structure = append(
            structure, OPAQUE_CHILD_TAG, ttlv.ttlv_value_integer(7), limits
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


def first_integer(view: ttlv.TtlvStructureView) -> int:
    item = ttlv.ttlv_structure_view_item_at(view, 0)
    try:
        value = ttlv.ttlv_item_view_value(item)
        try:
            if ttlv.ttlv_value_view_type(value) != ttlv.ItemType.Integer:
                raise AssertionError("opaque credential payload is not an Integer")
            return ttlv.ttlv_value_view_integer(value)
        finally:
            value.close()
    finally:
        item.close()


class CredentialValueTests(unittest.TestCase):
    def test_known_extension_and_unknown_values_preserve_raw_type_and_ttlv(self) -> None:
        facade = getattr(kmipkit.credentials, "CredentialValue", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        cases = (
            (4, (USERNAME_TAG, ONE_TIME_PASSWORD_TAG), None),
            (0x8000_4321, (OPAQUE_CHILD_TAG,), 7),
            (0xF123_4567, (OPAQUE_CHILD_TAG,), 7),
        )
        for raw_type, expected_tags, expected_integer in cases:
            with self.subTest(raw_type=raw_type):
                value = facade.from_ttlv(raw_type, credential_value_structure(raw_type))
                self.assertEqual(raw_type, value.credential_type_raw())
                self.assertNotIn("credential-value-secret", repr(value))
                borrowed = []

                def inspect(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
                    borrowed.append(view)
                    with self.assertRaises(errors.InvalidInputError):
                        value.into_ttlv()
                    if expected_integer is not None:
                        self.assertEqual(expected_integer, first_integer(view))
                    return tags(view)

                self.assertEqual(expected_tags, value.with_ttlv(inspect))
                with self.assertRaises(errors.InvalidInputError):
                    ttlv.ttlv_structure_view_item_count(borrowed[0])

                restored = value.into_ttlv()
                view = ttlv.ttlv_structure_view(restored)
                try:
                    self.assertEqual(expected_tags, tags(view))
                    if expected_integer is not None:
                        self.assertEqual(expected_integer, first_integer(view))
                finally:
                    view.close()
                    restored.close()
                value.close()

    def test_known_one_time_password_rejects_missing_username_without_payload(self) -> None:
        facade = getattr(kmipkit.credentials, "CredentialValue", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        with self.assertRaises(errors.CredentialValidationError) as raised:
            facade.from_ttlv(
                4,
                credential_value_structure(4, include_username=False),
            )
        self.assertEqual(errors.CredentialValidationErrorKind.MISSING_FIELD, raised.exception.kind)
        self.assertNotIn("credential-value-secret", str(raised.exception))

    def test_failed_transfer_with_retained_child_view_keeps_value_usable(self) -> None:
        facade = getattr(kmipkit.credentials, "CredentialValue", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        raw_type = 0x8000_4321
        value = facade.from_ttlv(raw_type, credential_value_structure(raw_type))
        retained = value.with_ttlv(
            lambda view: ttlv.ttlv_structure_view_item_at(view, 0)
        )
        try:
            self.assertEqual(ttlv.ItemType.Integer, ttlv.ttlv_item_view_type(retained))
            with self.assertRaises(errors.InvalidInputError):
                value.into_ttlv()
            self.assertEqual(raw_type, value.credential_type_raw())
        finally:
            retained.close()

        restored = value.into_ttlv()
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(1, ttlv.ttlv_structure_view_item_count(view))
        finally:
            view.close()
            restored.close()
            value.close()
