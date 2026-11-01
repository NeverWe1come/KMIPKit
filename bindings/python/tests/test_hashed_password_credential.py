from __future__ import annotations

import unittest

import kmipkit.credentials
from kmipkit import errors, ttlv


USERNAME_TAG = 0x0042_0099
TIMESTAMP_TAG = 0x0042_0092
HASHING_ALGORITHM_TAG = 0x0042_0038
HASHED_PASSWORD_TAG = 0x0042_0157
UNKNOWN_HASHING_ALGORITHM = 0xE123_4567
HASHED_PASSWORD_BYTES = b"hashed-password-sentinel\x00\xff"


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


def hashed_password_structure(*, algorithm: int | None):
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    structure = append(
        structure,
        USERNAME_TAG,
        ttlv.ttlv_value_text_string(b"alice", limits),
        limits,
    )
    structure = append(
        structure,
        TIMESTAMP_TAG,
        ttlv.ttlv_value_date_time_extended(1_800_000_000),
        limits,
    )
    if algorithm is not None:
        structure = append(
            structure,
            HASHING_ALGORITHM_TAG,
            ttlv.ttlv_value_enumeration(algorithm),
            limits,
        )
    structure = append(
        structure,
        HASHED_PASSWORD_TAG,
        ttlv.ttlv_value_byte_string(HASHED_PASSWORD_BYTES, limits),
        limits,
    )
    limits.close()
    return structure


def tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
    result: list[int] = []
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


def hashed_password_bytes(view: ttlv.TtlvStructureView) -> bytes:
    item = ttlv.ttlv_structure_view_item_at(view, 2 if len(tags(view)) == 3 else 3)
    try:
        with ttlv.ttlv_item_view_value(item) as value:
            if ttlv.ttlv_value_view_type(value) is not ttlv.ItemType.ByteString:
                return b""
            return bytes(
                ttlv.ttlv_value_view_byte_at(value, index)
                for index in range(ttlv.ttlv_value_view_byte_length(value))
            )
    finally:
        item.close()


class HashedPasswordCredentialTests(unittest.TestCase):
    def test_wrapper_preserves_unknown_algorithm_and_secret_ttlv(self) -> None:
        facade = getattr(kmipkit.credentials, "HashedPasswordCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        credential = facade.from_ttlv(
            hashed_password_structure(algorithm=UNKNOWN_HASHING_ALGORITHM)
        )
        self.assertEqual(UNKNOWN_HASHING_ALGORITHM, credential.hashing_algorithm_raw())
        self.assertEqual(
            UNKNOWN_HASHING_ALGORITHM,
            credential.effective_hashing_algorithm_raw(),
        )
        self.assertNotIn("hashed-password-sentinel", repr(credential))
        borrowed: list[ttlv.TtlvStructureView] = []

        def inspect(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
            borrowed.append(view)
            with self.assertRaises(errors.InvalidInputError):
                credential.into_ttlv()
            self.assertEqual(HASHED_PASSWORD_BYTES, hashed_password_bytes(view))
            return tags(view)

        self.assertEqual(
            (USERNAME_TAG, TIMESTAMP_TAG, HASHING_ALGORITHM_TAG, HASHED_PASSWORD_TAG),
            credential.with_ttlv(inspect),
        )
        with self.assertRaises(errors.InvalidInputError):
            ttlv.ttlv_structure_view_item_count(borrowed[0])

        restored = credential.into_ttlv()
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(
                (USERNAME_TAG, TIMESTAMP_TAG, HASHING_ALGORITHM_TAG, HASHED_PASSWORD_TAG),
                tags(view),
            )
            self.assertEqual(HASHED_PASSWORD_BYTES, hashed_password_bytes(view))
        finally:
            view.close()
            restored.close()

    def test_absent_algorithm_remains_absent_with_sha256_effective_default(self) -> None:
        facade = getattr(kmipkit.credentials, "HashedPasswordCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        credential = facade.from_ttlv(hashed_password_structure(algorithm=None))
        self.assertIsNone(credential.hashing_algorithm_raw())
        self.assertEqual(6, credential.effective_hashing_algorithm_raw())
        restored = credential.into_ttlv()
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual((USERNAME_TAG, TIMESTAMP_TAG, HASHED_PASSWORD_TAG), tags(view))
        finally:
            view.close()
            restored.close()

    def test_missing_hashed_password_returns_payload_free_validation_error(self) -> None:
        facade = getattr(kmipkit.credentials, "HashedPasswordCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        limits = ttlv.codec_limits_defaults()
        structure = ttlv.ttlv_structure_create()
        structure = append(
            structure,
            USERNAME_TAG,
            ttlv.ttlv_value_text_string(b"alice", limits),
            limits,
        )
        structure = append(
            structure,
            TIMESTAMP_TAG,
            ttlv.ttlv_value_date_time_extended(1_800_000_000),
            limits,
        )
        limits.close()
        with self.assertRaises(errors.CredentialValidationError) as raised:
            facade.from_ttlv(structure)
        self.assertEqual(
            errors.CredentialValidationErrorKind.MISSING_FIELD,
            raised.exception.kind,
        )
        self.assertNotIn("alice", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
