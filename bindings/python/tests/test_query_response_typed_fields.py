from __future__ import annotations

import pytest

from kmipkit import client, errors, ttlv


OPERATION = 0x0042_005C
OBJECT_TYPE = 0x0042_0057
VENDOR_IDENTIFICATION = 0x0042_009D
APPLICATION_NAMESPACE = 0x0042_0003
UNKNOWN_VENDOR_TAG = 0x0054_1234
SERVER_INFORMATION = 0x0042_0088
EXTENSION_INFORMATION = 0x0042_00A4
ATTESTATION_TYPE = 0x0042_00C7
RNG_PARAMETERS = 0x0042_00D9
PROFILE_INFORMATION = 0x0042_00EB
VALIDATION_INFORMATION = 0x0042_00DF
CAPABILITY_INFORMATION = 0x0042_00F7
CLIENT_REGISTRATION_METHOD = 0x0042_00F6
DEFAULTS_INFORMATION = 0x0042_0152
PROTECTION_STORAGE_MASKS = 0x0042_015F


def _append(
    structure: ttlv.Structure,
    tag_raw: int,
    value: ttlv.Value,
    limits: ttlv.CodecLimits,
) -> ttlv.Structure:
    raw = ttlv.raw_tag_from_raw(tag_raw)
    try:
        tag = ttlv.raw_tag_try_checked(raw)
        try:
            item = ttlv.ttlv_item_create(tag, value, limits)
            return ttlv.ttlv_structure_with_item(structure, item, limits)
        finally:
            tag.close()
    finally:
        raw.close()


def _query_payload() -> ttlv.Structure:
    with ttlv.codec_limits_defaults() as limits:
        payload = ttlv.ttlv_structure_create()
        payload = _append(payload, OPERATION, ttlv.ttlv_value_enumeration(0x3B), limits)
        payload = _append(payload, APPLICATION_NAMESPACE,
                          ttlv.ttlv_value_text_string("namespace-ñ".encode(), limits), limits)
        payload = _append(payload, UNKNOWN_VENDOR_TAG,
                          ttlv.ttlv_value_text_string(b"vendor-extension", limits), limits)
        payload = _append(payload, OPERATION,
                          ttlv.ttlv_value_enumeration(0xF1234567), limits)
        payload = _append(payload, OBJECT_TYPE, ttlv.ttlv_value_enumeration(7), limits)
        server = ttlv.ttlv_structure_create()
        server = _append(server, 0x0054_0001,
                         ttlv.ttlv_value_text_string(b"server-extension", limits), limits)
        payload = _append(payload, SERVER_INFORMATION,
                          ttlv.ttlv_value_structure(server, limits), limits)
        payload = _append(payload, APPLICATION_NAMESPACE,
                          ttlv.ttlv_value_text_string(b"namespace-b", limits), limits)
        payload = _append(payload, EXTENSION_INFORMATION,
                          ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits)
        payload = _append(payload, ATTESTATION_TYPE,
                          ttlv.ttlv_value_enumeration(0xF1234567), limits)
        payload = _append(payload, RNG_PARAMETERS,
                          ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits)
        profile = ttlv.ttlv_structure_create()
        profile = _append(profile, 0x0042_00EC,
                          ttlv.ttlv_value_enumeration(0x80000001), limits)
        payload = _append(payload, PROFILE_INFORMATION,
                          ttlv.ttlv_value_structure(profile, limits), limits)
        payload = _append(payload, VALIDATION_INFORMATION,
                          ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits)
        payload = _append(payload, CAPABILITY_INFORMATION,
                          ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits)
        payload = _append(payload, CLIENT_REGISTRATION_METHOD,
                          ttlv.ttlv_value_enumeration(0x80000001), limits)
        payload = _append(payload, DEFAULTS_INFORMATION,
                          ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits)
        payload = _append(payload, PROTECTION_STORAGE_MASKS,
                          ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits)
        payload = _append(payload, VENDOR_IDENTIFICATION,
                          ttlv.ttlv_value_text_string(b"vendor-a", limits), limits)
        return payload


def _sparse_query_payload() -> ttlv.Structure:
    with ttlv.codec_limits_defaults() as limits:
        return _append(
            ttlv.ttlv_structure_create(),
            OPERATION,
            ttlv.ttlv_value_enumeration(0x3B),
            limits,
        )


def _classified_fields() -> tuple[client._QueryFieldData, ...]:
    return (
        client._QueryFieldData(OPERATION, client.QueryFieldKind.ENUMERATION, 0x3B),
        client._QueryFieldData(
            APPLICATION_NAMESPACE, client.QueryFieldKind.TEXT_STRING, 0
        ),
        client._QueryFieldData(UNKNOWN_VENDOR_TAG, client.QueryFieldKind.UNKNOWN, 0),
        client._QueryFieldData(OPERATION, client.QueryFieldKind.ENUMERATION, 0xF1234567),
        client._QueryFieldData(OBJECT_TYPE, client.QueryFieldKind.ENUMERATION, 7),
        client._QueryFieldData(SERVER_INFORMATION, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(
            APPLICATION_NAMESPACE, client.QueryFieldKind.TEXT_STRING, 0
        ),
        client._QueryFieldData(EXTENSION_INFORMATION, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(ATTESTATION_TYPE, client.QueryFieldKind.ENUMERATION, 0xF1234567),
        client._QueryFieldData(RNG_PARAMETERS, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(PROFILE_INFORMATION, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(VALIDATION_INFORMATION, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(CAPABILITY_INFORMATION, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(
            CLIENT_REGISTRATION_METHOD, client.QueryFieldKind.ENUMERATION, 0x80000001
        ),
        client._QueryFieldData(DEFAULTS_INFORMATION, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(PROTECTION_STORAGE_MASKS, client.QueryFieldKind.STRUCTURE, 0),
        client._QueryFieldData(
            VENDOR_IDENTIFICATION, client.QueryFieldKind.TEXT_STRING, 0
        ),
    )


def test_typed_query_fields_retain_repeats_unknowns_and_raw_order() -> None:
    payload = _query_payload()
    response = client.QueryResponse(payload, _classified_fields())
    try:
        assert response.field_count == 17
        assert response.field_at(0).kind is client.QueryFieldKind.ENUMERATION
        assert response.field_at(0).enumeration == 0x3B
        assert response.field_at(2).kind is client.QueryFieldKind.UNKNOWN
        assert response.field_at(2).tag == UNKNOWN_VENDOR_TAG
        unknown_item = response.field_at(2).item()
        assert unknown_item is not None
        unknown_item.close()
        text_field = response.field_at(16)
        assert text_field.kind is client.QueryFieldKind.TEXT_STRING
        text_item = text_field.item()
        assert text_item is not None
        text_item.close()
        with pytest.raises(errors.InvalidInputError):
            response.field_at(17)
        assert response.operations == (0x3B, 0xF1234567)
        assert response.object_types == (7,)
        assert response.vendor_identifications == ("vendor-a",)
        assert response.application_namespaces == ("namespace-ñ", "namespace-b")
        with ttlv.ttlv_structure_view(payload) as raw:
            item = ttlv.ttlv_structure_view_item_at(raw, 1)
            try:
                value = ttlv.ttlv_item_view_value(item)
                try:
                    assert ttlv.ttlv_value_view_bytes(value) == "namespace-ñ".encode()
                finally:
                    value.close()
            finally:
                item.close()
        assert response.attestation_types == (0xF1234567,)
        assert response.client_registration_methods == (0x80000001,)
        assert response.unknown_field_tags == (UNKNOWN_VENDOR_TAG,)
        assert response.raw_field_tags == (
            OPERATION,
            APPLICATION_NAMESPACE,
            UNKNOWN_VENDOR_TAG,
            OPERATION,
            OBJECT_TYPE,
            SERVER_INFORMATION,
            APPLICATION_NAMESPACE,
            EXTENSION_INFORMATION,
            ATTESTATION_TYPE,
            RNG_PARAMETERS,
            PROFILE_INFORMATION,
            VALIDATION_INFORMATION,
            CAPABILITY_INFORMATION,
            CLIENT_REGISTRATION_METHOD,
            DEFAULTS_INFORMATION,
            PROTECTION_STORAGE_MASKS,
            VENDOR_IDENTIFICATION,
        )
        for structure_items in (
            response.extension_informations,
            response.rng_parameters,
            response.profile_information,
            response.validation_information,
            response.capability_information,
        ):
            assert len(structure_items) == 1
            for item in structure_items:
                item.close()
        defaults = response.defaults_information
        assert defaults is not None
        defaults.close()
        masks = response.protection_storage_masks
        assert masks is not None
        masks.close()
        server_information = response.server_informations
        assert len(server_information) == 1
        server_item = server_information[0]
        try:
            value = ttlv.ttlv_item_view_value(server_item)
            try:
                server_fields = ttlv.ttlv_value_view_structure(value)
                try:
                    assert ttlv.ttlv_structure_view_item_count(server_fields) == 1
                    field = ttlv.ttlv_structure_view_item_at(server_fields, 0)
                    try:
                        tag = ttlv.ttlv_item_view_tag(field)
                        try:
                            assert ttlv.tag_value(tag) == 0x0054_0001
                        finally:
                            tag.close()
                    finally:
                        field.close()
                finally:
                    server_fields.close()
            finally:
                value.close()
        finally:
            server_item.close()
    finally:
        response.close()


def test_typed_query_structure_fields_are_absent_without_losing_raw_payload() -> None:
    payload = _sparse_query_payload()
    response = client.QueryResponse(
        payload,
        (client._QueryFieldData(OPERATION, client.QueryFieldKind.ENUMERATION, 0x3B),),
    )
    try:
        assert response.server_informations == ()
        assert response.extension_informations == ()
        assert response.defaults_information is None
        assert response.protection_storage_masks is None
        with ttlv.ttlv_structure_view(payload) as raw:
            assert ttlv.ttlv_structure_view_item_count(raw) == 1
    finally:
        response.close()

