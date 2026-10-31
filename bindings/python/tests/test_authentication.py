"""Ordered Authentication and callback-scoped CredentialView parity."""

from __future__ import annotations

import unittest

import kmipkit
from kmipkit import errors, ttlv


AUTHENTICATION_CREDENTIAL_TAG = 0x0042_0023
CREDENTIAL_TYPE_TAG = 0x0042_0024
CREDENTIAL_VALUE_TAG = 0x0042_0025
CREDENTIAL_PAYLOAD_TAG = 0x0042_002A


def append(
    structure: ttlv.Structure,
    raw_tag_value: int,
    value: ttlv.Value,
    limits: ttlv.CodecLimits,
) -> ttlv.Structure:
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


def credential_structure(raw_type: int) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    payload = ttlv.ttlv_structure_create()
    try:
        structure = append(
            structure, CREDENTIAL_TYPE_TAG,
            ttlv.ttlv_value_enumeration(raw_type), limits,
        )
        payload = append(
            payload, CREDENTIAL_PAYLOAD_TAG,
            ttlv.ttlv_value_integer(0x13579), limits,
        )
        return append(
            structure, CREDENTIAL_VALUE_TAG,
            ttlv.ttlv_value_structure(payload, limits), limits,
        )
    except BaseException:
        structure.close()
        payload.close()
        raise
    finally:
        limits.close()


def read_tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
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


class AuthenticationTests(unittest.TestCase):
    def test_authentication_preserves_order_and_owner_backed_views(self) -> None:
        facade = getattr(kmipkit.credentials, "Authentication", None)
        view_type = getattr(kmipkit.credentials, "CredentialView", None)
        self.assertIsNotNone(facade, "the Python facade must expose Authentication")
        self.assertIsNotNone(view_type, "the Python facade must expose CredentialView")
        if facade is None or view_type is None:
            return

        raw_types = (0xF123_4567, 0x8000_4321)
        credentials = [
            kmipkit.credentials.Credential.from_ttlv(credential_structure(raw_type))
            for raw_type in raw_types
        ]
        authentication = facade.create(credentials)
        self.assertTrue(all(credential.closed for credential in credentials))
        self.assertEqual(
            (AUTHENTICATION_CREDENTIAL_TAG, AUTHENTICATION_CREDENTIAL_TAG),
            authentication.with_ttlv(read_tags),
        )

        views = authentication.credentials
        try:
            self.assertEqual(raw_types, tuple(view.credential_type_raw() for view in views))
            self.assertTrue(all(isinstance(view, view_type) for view in views))
            self.assertEqual(
                (CREDENTIAL_TYPE_TAG, CREDENTIAL_VALUE_TAG),
                views[0].with_ttlv(read_tags),
            )
            with self.assertRaises(errors.InvalidInputError):
                authentication.into_ttlv()
        finally:
            for view in views:
                view.close()

        restored = authentication.into_ttlv()
        self.assertTrue(authentication.closed)
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(
                (AUTHENTICATION_CREDENTIAL_TAG, AUTHENTICATION_CREDENTIAL_TAG),
                read_tags(view),
            )
        finally:
            view.close()
            restored.close()

    def test_authentication_rejects_empty_credentials(self) -> None:
        facade = getattr(kmipkit.credentials, "Authentication", None)
        self.assertIsNotNone(facade, "the Python facade must expose Authentication")
        if facade is None:
            return
        with self.assertRaises(errors.InvalidInputError):
            facade.create([])


if __name__ == "__main__":
    unittest.main()
