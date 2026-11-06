"""Four-language Table 59 parameter construction and message Authentication."""

from __future__ import annotations

import kmipkit
import pytest
from kmipkit import client, credentials, errors, extensions, ttlv


TABLE_59_TAGS = (
    0x0042_0011, 0x0042_005F, 0x0042_0038, 0x0042_0083, 0x0042_00AE,
    0x0042_0028, 0x0042_00C5, 0x0042_00CD, 0x0042_00CE, 0x0042_00CF,
    0x0042_00D2, 0x0042_00D0, 0x0042_00D1, 0x0042_0100, 0x0042_0101,
    0x0042_0102, 0x0042_0103, 0x0042_0104,
)


def _credential_structure() -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    credential = ttlv.ttlv_structure_create()
    payload = ttlv.ttlv_structure_create()

    def append(structure: ttlv.Structure, tag_value: int, value: ttlv.Value) -> ttlv.Structure:
        raw = ttlv.raw_tag_from_raw(tag_value)
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

    credential = append(
        credential, 0x0042_0024, ttlv.ttlv_value_enumeration(0x8000_0001)
    )
    payload = append(payload, 0x0042_002A, ttlv.ttlv_value_integer(1))
    credential = append(
        credential, 0x0042_0025, ttlv.ttlv_value_structure(payload, limits)
    )
    limits.close()
    return credential


def _read_tags(view: ttlv.TtlvStructureView) -> tuple[int, ...]:
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


def test_builds_all_table59_fields_in_normative_order_and_preserves_future_enums():
    parameters = kmipkit.cryptographic_parameters.CryptographicParameters.builder()
    parameters.trailer_field(1).p_source(b"Q\x00\xff")
    parameters.mask_generator_hashing_algorithm(0x8000_0003)
    parameters.mask_generator(0x8000_0002).salt_length(16)
    parameters.initial_counter_value(1).counter_length(32)
    parameters.invocation_field_length(4).fixed_field_length(2)
    parameters.tag_length(16).iv_length(8).random_iv(True)
    parameters.cryptographic_algorithm(0xE123_0001)
    parameters.digital_signature_algorithm(0x8000_5678).key_role_type(0x8000_1234)
    parameters.hashing_algorithm(0xF123_4567).padding_method(0x8000_0001)
    parameters.block_cipher_mode(0xE123_4567)
    with parameters.build() as typed, typed.into_ttlv() as structure:
        with ttlv.ttlv_structure_view(structure) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 18
            assert _read_tags(view) == TABLE_59_TAGS


def test_reads_every_typed_table59_field_without_consuming_the_model():
    builder = kmipkit.cryptographic_parameters.CryptographicParameters.builder()
    builder.block_cipher_mode(0xE123_4567).padding_method(0x8000_0001)
    builder.hashing_algorithm(0xF123_4567).key_role_type(0x8000_1234)
    builder.digital_signature_algorithm(0x8000_5678).cryptographic_algorithm(0xE123_0001)
    builder.random_iv(False).iv_length(8).tag_length(16).fixed_field_length(2)
    builder.invocation_field_length(4).counter_length(32).initial_counter_value(1)
    builder.salt_length(16).mask_generator(0x8000_0002)
    builder.mask_generator_hashing_algorithm(0x8000_0003)
    builder.p_source(b"Q\x00\xff").trailer_field(1)
    with builder.build() as parameters:
        assert parameters.block_cipher_mode() == 0xE123_4567
        assert parameters.padding_method() == 0x8000_0001
        assert parameters.hashing_algorithm() == 0xF123_4567
        assert parameters.key_role_type() == 0x8000_1234
        assert parameters.digital_signature_algorithm() == 0x8000_5678
        assert parameters.cryptographic_algorithm() == 0xE123_0001
        assert parameters.random_iv() is False
        assert parameters.iv_length() == 8
        assert parameters.tag_length() == 16
        assert parameters.fixed_field_length() == 2
        assert parameters.invocation_field_length() == 4
        assert parameters.counter_length() == 32
        assert parameters.initial_counter_value() == 1
        assert parameters.salt_length() == 16
        assert parameters.mask_generator() == 0x8000_0002
        assert parameters.mask_generator_hashing_algorithm() == 0x8000_0003
        assert parameters.p_source() == b"Q\x00\xff"
        assert parameters.trailer_field() == 1


def test_distinguishes_absent_and_present_empty_p_source():
    model = kmipkit.cryptographic_parameters.CryptographicParameters
    with model.builder().random_iv(False).build() as absent:
        assert absent.p_source() is None
    with model.builder().random_iv(False).p_source(b"").build() as present_empty:
        assert present_empty.p_source() == b""


def test_authentication_is_attached_to_the_request_batch_and_owned_on_success():
    credential = credentials.Credential.from_ttlv(_credential_structure())
    authentication = credentials.Authentication.create([credential])
    batch = client.Batch.create()
    try:
        batch.set_authentication(authentication)
        assert authentication.closed
        batch.add_item(extensions.client_batch_item_ping())
        batch.add_item(extensions.client_batch_item_ping())
        assert batch.item_count == 2
    finally:
        batch.close()
        authentication.close()


def test_failed_authentication_attachment_keeps_the_caller_owner_live():
    credential = credentials.Credential.from_ttlv(_credential_structure())
    authentication = credentials.Authentication.create([credential])
    batch = client.Batch.create()
    batch.close()
    try:
        with pytest.raises(errors.InvalidInputError):
            batch.set_authentication(authentication)
        assert not authentication.closed
        assert len(authentication.credentials) == 1
    finally:
        authentication.close()

