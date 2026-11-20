from __future__ import annotations

import unittest

import kmipkit.credentials
from kmipkit import errors, ttlv


USERNAME_TAG = 0x0042_0099
PASSWORD_TAG = 0x0042_00A1
EXTENSION_CHILD_TAG = 0x0054_0001


def append(structure, raw_tag, value, limits):
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


def username_password_structure(include_username: bool):
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    if include_username:
        structure = append(
            structure, USERNAME_TAG, ttlv.ttlv_value_text_string(b"alice", limits), limits
        )
    structure = append(
        structure,
        PASSWORD_TAG,
        ttlv.ttlv_value_text_string(b"sensitive-sentinel", limits),
        limits,
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
    expected = (b"alice", b"sensitive-sentinel")
    for index, wanted in enumerate(expected):
        item = ttlv.ttlv_structure_view_item_at(view, index)
        try:
            with ttlv.ttlv_item_view_value(item) as value:
                if ttlv.ttlv_value_view_type(value) is not ttlv.ItemType.TextString:
                    return False
                length = ttlv.ttlv_value_view_byte_length(value)
                actual = bytes(
                    ttlv.ttlv_value_view_byte_at(value, offset)
                    for offset in range(length)
                )
                if actual != wanted:
                    return False
        finally:
            item.close()
    return True


class UsernameAndPasswordCredentialTests(unittest.TestCase):
    def test_wrapper_preserves_table_411_values_and_scoped_ttlv(self) -> None:
        facade = getattr(kmipkit.credentials, "UsernameAndPasswordCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        credential = facade.from_ttlv(username_password_structure(True))
        self.assertNotIn("sensitive-sentinel", repr(credential))
        borrowed = []

        def inspect(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
            borrowed.append(view)
            with self.assertRaises(errors.InvalidInputError):
                credential.into_ttlv()
            self.assertTrue(values_match(view))
            return tags(view)

        expected = (USERNAME_TAG, PASSWORD_TAG, EXTENSION_CHILD_TAG)
        self.assertEqual(expected, credential.with_ttlv(inspect))
        with self.assertRaises(errors.InvalidInputError):
            ttlv.ttlv_structure_view_item_count(borrowed[0])

        restored = credential.into_ttlv()
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(expected, tags(view))
            self.assertTrue(values_match(view))
        finally:
            view.close()
            restored.close()

    def test_wrapper_requires_username_and_redacts_validation_error(self) -> None:
        facade = getattr(kmipkit.credentials, "UsernameAndPasswordCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        with self.assertRaises(errors.CredentialValidationError) as raised:
            facade.from_ttlv(username_password_structure(False))
        self.assertEqual(
            errors.CredentialValidationErrorKind.MISSING_FIELD,
            raised.exception.kind,
        )
        self.assertNotIn("sensitive-sentinel", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
