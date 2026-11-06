from __future__ import annotations

from kmipkit import client, ttlv


OPERATION = 0x0042_005C
OBJECT_TYPE = 0x0042_0057
VENDOR_IDENTIFICATION = 0x0042_009D
APPLICATION_NAMESPACE = 0x0042_0003
UNKNOWN_VENDOR_TAG = 0x0054_1234


def _append(structure: ttlv.Structure, tag_raw: int, value: ttlv.Value) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    raw = ttlv.raw_tag_from_raw(tag_raw)
    tag = ttlv.raw_tag_try_checked(raw)
    item = ttlv.ttlv_item_create(tag, value, limits)
    return ttlv.ttlv_structure_with_item(structure, item, limits)


def _query_payload() -> ttlv.Structure:
    payload = ttlv.ttlv_structure_create()
    payload = _append(payload, OPERATION, ttlv.ttlv_value_enumeration(0x3B))
    payload = _append(
        payload,
        APPLICATION_NAMESPACE,
        ttlv.ttlv_value_text_string(b"namespace-a", ttlv.codec_limits_defaults()),
    )
    payload = _append(
        payload,
        UNKNOWN_VENDOR_TAG,
        ttlv.ttlv_value_text_string(b"vendor-extension", ttlv.codec_limits_defaults()),
    )
    payload = _append(payload, OPERATION, ttlv.ttlv_value_enumeration(0xF1234567))
    payload = _append(payload, OBJECT_TYPE, ttlv.ttlv_value_enumeration(7))
    payload = _append(
        payload,
        APPLICATION_NAMESPACE,
        ttlv.ttlv_value_text_string(b"namespace-b", ttlv.codec_limits_defaults()),
    )
    payload = _append(
        payload,
        VENDOR_IDENTIFICATION,
        ttlv.ttlv_value_text_string(b"vendor-a", ttlv.codec_limits_defaults()),
    )
    return payload


def test_typed_query_fields_retain_repeats_unknowns_and_raw_order() -> None:
    payload = _query_payload()
    response = client.QueryResponse._from_payload(payload)
    try:
        assert response.operations == (0x3B, 0xF1234567)
        assert response.object_types == (7,)
        assert response.vendor_identifications == ("vendor-a",)
        assert response.application_namespaces == ("namespace-a", "namespace-b")
        assert response.unknown_field_tags == (UNKNOWN_VENDOR_TAG,)
        assert response.raw_field_tags == (
            OPERATION,
            APPLICATION_NAMESPACE,
            UNKNOWN_VENDOR_TAG,
            OPERATION,
            OBJECT_TYPE,
            APPLICATION_NAMESPACE,
            VENDOR_IDENTIFICATION,
        )
    finally:
        response.close()


def test_typed_query_structure_fields_are_absent_without_losing_raw_payload() -> None:
    payload = _query_payload()
    response = client.QueryResponse._from_payload(payload)
    try:
        assert response.server_information == ()
        assert response.extension_information == ()
        assert response.defaults_information is None
        assert response.protection_storage_masks is None
        with ttlv.ttlv_structure_view(payload) as raw:
            assert ttlv.ttlv_structure_view_item_count(raw) == 7
    finally:
        response.close()

