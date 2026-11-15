"""Typed Attestation Credential parity and ownership behavior."""

from __future__ import annotations

import unittest

import kmipkit
from kmipkit import errors, ttlv


NONCE_TAG = 0x0042_00C8
NONCE_ID_TAG = 0x0042_00C9
NONCE_VALUE_TAG = 0x0042_00CA
ATTESTATION_TYPE_TAG = 0x0042_00C7
MEASUREMENT_TAG = 0x0042_00CB
ASSERTION_TAG = 0x0042_00CC


def append(structure: ttlv.Structure, raw_tag_value: int, value: ttlv.Value, limits: ttlv.CodecLimits) -> ttlv.Structure:
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


def attestation_structure(include_measurement: bool, include_assertion: bool) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    nonce = ttlv.ttlv_structure_create()
    nonce = append(
        nonce,
        NONCE_ID_TAG,
        ttlv.ttlv_value_byte_string(b"\x01\x02\xfe", limits),
        limits,
    )
    nonce = append(
        nonce,
        NONCE_VALUE_TAG,
        ttlv.ttlv_value_byte_string(b"\xa1\x00\xff", limits),
        limits,
    )
    structure = ttlv.ttlv_structure_create()
    structure = append(
        structure,
        NONCE_TAG,
        ttlv.ttlv_value_structure(nonce, limits),
        limits,
    )
    structure = append(
        structure,
        ATTESTATION_TYPE_TAG,
        ttlv.ttlv_value_enumeration(0xF123_4567),
        limits,
    )
    if include_measurement:
        structure = append(
            structure,
            MEASUREMENT_TAG,
            ttlv.ttlv_value_byte_string(b"\x10\x20\x30", limits),
            limits,
        )
    if include_assertion:
        structure = append(
            structure,
            ASSERTION_TAG,
            ttlv.ttlv_value_byte_string(b"\x40\x50\x60", limits),
            limits,
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


def byte_string(view: ttlv.TtlvStructureView, index: int) -> bytes:
    item = ttlv.ttlv_structure_view_item_at(view, index)
    try:
        value = ttlv.ttlv_item_view_value(item)
        try:
            length = ttlv.ttlv_value_view_byte_length(value)
            return bytes(
                ttlv.ttlv_value_view_byte_at(value, offset)
                for offset in range(length)
            )
        finally:
            value.close()
    finally:
        item.close()


def attestation_values(view: ttlv.TtlvStructureView) -> tuple[int, bytes, bytes, bytes, bytes]:
    attestation_item = ttlv.ttlv_structure_view_item_at(view, 1)
    try:
        attestation_value = ttlv.ttlv_item_view_value(attestation_item)
        try:
            attestation_type = ttlv.ttlv_value_view_enumeration(attestation_value)
        finally:
            attestation_value.close()
    finally:
        attestation_item.close()

    nonce_item = ttlv.ttlv_structure_view_item_at(view, 0)
    try:
        nonce_value = ttlv.ttlv_item_view_value(nonce_item)
        try:
            nonce_view = ttlv.ttlv_value_view_structure(nonce_value)
            try:
                nonce_id = byte_string(nonce_view, 0)
                nonce_value_bytes = byte_string(nonce_view, 1)
            finally:
                nonce_view.close()
        finally:
            nonce_value.close()
    finally:
        nonce_item.close()
    return (
        attestation_type,
        nonce_id,
        nonce_value_bytes,
        byte_string(view, 2),
        byte_string(view, 3),
    )


class AttestationCredentialTests(unittest.TestCase):
    def test_attestation_credential_preserves_structure_and_scoped_views(self) -> None:
        facade = getattr(kmipkit.credentials, "AttestationCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return

        source = attestation_structure(True, True)
        source_view = ttlv.ttlv_structure_view(source)
        try:
            with self.assertRaises(errors.InvalidInputError):
                facade.from_ttlv(source)
            self.assertEqual(4, ttlv.ttlv_structure_view_item_count(source_view))
        finally:
            source_view.close()

        credential = facade.from_ttlv(source)
        self.assertTrue(source.closed)
        self.assertNotIn("a100ff", repr(credential).lower())
        borrowed = []

        def inspect(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
            borrowed.append(view)
            with self.assertRaises(errors.InvalidInputError):
                credential.into_ttlv()
            return tags(view)

        self.assertEqual(
            (NONCE_TAG, ATTESTATION_TYPE_TAG, MEASUREMENT_TAG, ASSERTION_TAG),
            credential.with_ttlv(inspect),
        )
        with self.assertRaises(errors.InvalidInputError):
            ttlv.ttlv_structure_view_item_count(borrowed[0])

        restored = credential.into_ttlv()
        self.assertTrue(credential.closed)
        view = ttlv.ttlv_structure_view(restored)
        try:
            self.assertEqual(
                (NONCE_TAG, ATTESTATION_TYPE_TAG, MEASUREMENT_TAG, ASSERTION_TAG),
                tags(view),
            )
            self.assertEqual(
                (
                    0xF123_4567,
                    b"\x01\x02\xfe",
                    b"\xa1\x00\xff",
                    b"\x10\x20\x30",
                    b"\x40\x50\x60",
                ),
                attestation_values(view),
            )
        finally:
            view.close()
            restored.close()

    def test_attestation_credential_requires_at_least_one_evidence_field(self) -> None:
        facade = getattr(kmipkit.credentials, "AttestationCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return
        source = attestation_structure(False, False)
        with self.assertRaises(errors.CredentialValidationError) as raised:
            facade.from_ttlv(source)
        self.assertEqual(
            errors.CredentialValidationErrorKind.MISSING_FIELD,
            raised.exception.kind,
        )
        self.assertTrue(source.closed)

    def test_attestation_credential_accepts_either_evidence_field_alone(self) -> None:
        facade = getattr(kmipkit.credentials, "AttestationCredential", None)
        self.assertIsNotNone(facade)
        if facade is None:
            return
        for measurement, assertion in ((True, False), (False, True)):
            credential = facade.from_ttlv(attestation_structure(measurement, assertion))
            try:
                self.assertEqual(3, credential.with_ttlv(ttlv.ttlv_structure_view_item_count))
            finally:
                credential.close()


if __name__ == "__main__":
    unittest.main()
