"""Captured Table 238 and Table 370 request wires through the Python client."""

from __future__ import annotations

import ssl
import struct
from pathlib import Path

from _test_support import (
    mtls_transport_config,
    read_ttlv_frame,
    start_tls13_mtls_peer,
    ttlv_enumeration,
    ttlv_item,
    ttlv_structure,
)
from kmipkit import client, extensions, ttlv

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"
_BATCH_IDS = tuple(f"wrapping-item-{index}".encode() for index in range(5))
_WRAPPED_MATERIAL = b"synthetic-wrapped-key-value"


def _single_child_structure(
    child_tag: int, value: ttlv.Value, limits: ttlv.CodecLimits
) -> ttlv.Structure:
    with ttlv.raw_tag_from_raw(child_tag) as raw_tag:
        with ttlv.raw_tag_try_checked(raw_tag) as tag:
            with value:
                with ttlv.ttlv_item_create(tag, value, limits) as item:
                    with ttlv.ttlv_structure_create() as empty:
                        return ttlv.ttlv_structure_with_item(empty, item, limits)


def _two_child_structure(
    first_tag: int,
    first_value: ttlv.Value,
    second_tag: int,
    second_value: ttlv.Value,
    limits: ttlv.CodecLimits,
) -> ttlv.Structure:
    with ttlv.raw_tag_from_raw(first_tag) as first_raw:
        with ttlv.raw_tag_try_checked(first_raw) as first_checked:
            with first_value:
                with ttlv.ttlv_item_create(first_checked, first_value, limits) as first_item:
                    with ttlv.ttlv_structure_create() as empty:
                        with ttlv.ttlv_structure_with_item(empty, first_item, limits) as first:
                            with ttlv.raw_tag_from_raw(second_tag) as second_raw:
                                with ttlv.raw_tag_try_checked(second_raw) as second_checked:
                                    with second_value:
                                        with ttlv.ttlv_item_create(
                                            second_checked, second_value, limits
                                        ) as second_item:
                                            return ttlv.ttlv_structure_with_item(
                                                first, second_item, limits
                                            )


def _structure_from_fields(
    fields: tuple[tuple[int, ttlv.Value], ...], limits: ttlv.CodecLimits
) -> ttlv.Structure:
    result = ttlv.ttlv_structure_create()
    try:
        for child_tag, value in fields:
            with ttlv.raw_tag_from_raw(child_tag) as raw_tag:
                with ttlv.raw_tag_try_checked(raw_tag) as tag:
                    with value:
                        with ttlv.ttlv_item_create(tag, value, limits) as item:
                            updated = ttlv.ttlv_structure_with_item(result, item, limits)
            result.close()
            result = updated
        return result
    except Exception:
        result.close()
        raise


def _wrapped_any_object(limits: ttlv.CodecLimits) -> ttlv.Structure:
    with _single_child_structure(
        0x420094, ttlv.ttlv_value_text_string(b"wrapping-key", limits), limits
    ) as key_information:
        with _two_child_structure(
            0x42009E,
            ttlv.ttlv_value_enumeration(1),
            0x420036,
            ttlv.ttlv_value_structure(key_information, limits),
            limits,
        ) as key_wrapping_data:
            with _structure_from_fields(
                (
                    (0x420042, ttlv.ttlv_value_enumeration(1)),
                    (0x420045, ttlv.ttlv_value_byte_string(_WRAPPED_MATERIAL, limits)),
                    (0x420046, ttlv.ttlv_value_structure(key_wrapping_data, limits)),
                ),
                limits,
            ) as key_block:
                with _single_child_structure(
                    0x420040, ttlv.ttlv_value_structure(key_block, limits), limits
                ) as key_block_item:
                    with ttlv.ttlv_value_structure(key_block_item, limits) as object_value:
                        return _single_child_structure(0x42008F, object_value, limits)


def _batch_item(operation: int, batch_id: bytes, *payload: bytes) -> bytes:
    return ttlv_structure(
        0x42000F,
        ttlv_enumeration(0x42005C, operation),
        ttlv_item(0x420093, 8, batch_id),
        ttlv_structure(0x420079, *payload),
    )


def _response_message() -> bytes:
    version = ttlv_structure(
        0x420069,
        ttlv_item(0x42006A, 2, struct.pack(">i", 2)),
        ttlv_item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = ttlv_structure(
        0x42007A,
        version,
        ttlv_item(0x420092, 9, struct.pack(">q", 1)),
        ttlv_item(0x42000D, 2, struct.pack(">i", len(_BATCH_IDS))),
    )
    operations = (42, 42, 42, 43, 10)
    results = tuple(
        ttlv_structure(
            0x42000F,
            ttlv_enumeration(0x42005C, operation),
            ttlv_item(0x420093, 8, batch_id),
            ttlv_enumeration(0x42007F, 1),
            ttlv_enumeration(0x42007E, 5),
        )
        for operation, batch_id in zip(operations, _BATCH_IDS, strict=True)
    )
    return ttlv_structure(0x42007B, header, *results)


def test_import_and_get_export_wrapping_options_are_encoded_over_mtls() -> None:
    """OASIS KMIP 2.1 §6.1.25 Table 238; §6.1.18 Table 217; §6.1.19 Table 220; §7.18 Table 370."""
    requests: list[bytes] = []

    def handle(stream: ssl.SSLSocket) -> None:
        requests.append(read_ttlv_frame(stream))
        stream.sendall(_response_message())

    port, failures, peer = start_tls13_mtls_peer(_FIXTURES, handle)
    configuration = extensions.create_client_configuration(
        extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
    )
    limits = ttlv.codec_limits_defaults()
    batch = None
    result = None
    try:
        with (
            configuration,
            client.Client.create(configuration, mtls_transport_config(port, _FIXTURES)) as kmip,
            _wrapped_any_object(limits) as any_object,
            _single_child_structure(0x4200FD, ttlv.ttlv_value_text_string(b"import-attribute", limits), limits) as attributes,
            _single_child_structure(0x420094, ttlv.ttlv_value_text_string(b"wrapping-key-id", limits), limits) as key_info,
            _two_child_structure(
                0x42009E,
                ttlv.ttlv_value_enumeration(1),
                0x420036,
                ttlv.ttlv_value_structure(key_info, limits),
                limits,
            ) as wrapping_specification,
        ):
            batch = client.Batch.create()
            identifiers = (
                extensions.UniqueIdentifier.text_string("import-omitted"),
                extensions.UniqueIdentifier.text_string("import-false"),
                extensions.UniqueIdentifier.text_string("import-true"),
                extensions.UniqueIdentifier.text_string("export-wrapped"),
                extensions.UniqueIdentifier.text_string("get-wrapped"),
            )
            items = (
                extensions.client_batch_item_import(
                    identifiers[0], 2, attributes, any_object, key_wrap_type=0xF1234568
                ),
                extensions.client_batch_item_import(
                    identifiers[1], 2, attributes, any_object,
                    replace_existing=False, key_wrap_type=0xF1234568,
                ),
                extensions.client_batch_item_import(
                    identifiers[2], 2, attributes, any_object,
                    replace_existing=True, key_wrap_type=0xF1234568,
                ),
                extensions.client_batch_item_export(
                    identifiers[3], key_wrapping_specification=wrapping_specification
                ),
                extensions.client_batch_item_get(
                    identifiers[4], key_wrapping_specification=wrapping_specification
                ),
            )
            for item, batch_id in zip(items, _BATCH_IDS, strict=True):
                batch.add_item(extensions.with_unique_batch_item_id(item, batch_id))
            result = kmip.execute(batch, limits)
        assert failures.empty(), "the TLS 1.3 peer completes the wrapping request exchange"
        assert len(requests) == 1
        assert result is not None and len(result.items) == len(_BATCH_IDS)

        wrapped_key = ttlv_structure(
            0x42008F,
            ttlv_structure(
                0x420040,
                ttlv_enumeration(0x420042, 1),
                ttlv_item(0x420045, 8, _WRAPPED_MATERIAL),
                ttlv_structure(
                    0x420046,
                    ttlv_enumeration(0x42009E, 1),
                    ttlv_structure(
                        0x420036,
                        ttlv_item(0x420094, 7, b"wrapping-key"),
                    ),
                ),
            ),
        )
        import_prefix = (
            ttlv_item(0x420094, 7, b"import-omitted")
            + ttlv_enumeration(0x420057, 2)
        )
        import_common = (
            ttlv_enumeration(0x4200F8, 0xF1234568)
            + ttlv_structure(0x420125, ttlv_item(0x4200FD, 7, b"import-attribute"))
            + wrapped_key
        )
        expected_imports = (
            _batch_item(42, _BATCH_IDS[0], import_prefix + import_common),
            _batch_item(42, _BATCH_IDS[1],
                ttlv_item(0x420094, 7, b"import-false")
                + ttlv_enumeration(0x420057, 2)
                + ttlv_item(0x420124, 6, b"\0\0\0\0\0\0\0\0")
                + import_common,
            ),
            _batch_item(42, _BATCH_IDS[2],
                ttlv_item(0x420094, 7, b"import-true")
                + ttlv_enumeration(0x420057, 2)
                + ttlv_item(0x420124, 6, b"\0\0\0\0\0\0\0\1")
                + import_common,
            ),
        )
        wrapping = ttlv_structure(
            0x420047,
            ttlv_enumeration(0x42009E, 1),
            ttlv_structure(0x420036, ttlv_item(0x420094, 7, b"wrapping-key-id")),
        )
        expected_export = _batch_item(
            43,
            _BATCH_IDS[3],
            ttlv_item(0x420094, 7, b"export-wrapped") + wrapping,
        )
        expected_get = _batch_item(
            10,
            _BATCH_IDS[4],
            ttlv_item(0x420094, 7, b"get-wrapped") + wrapping,
        )
        request = requests[0]
        expected_items = (*expected_imports, expected_export, expected_get)
        assert all(bool(item in request) for item in expected_items), (
            "all five operation payloads match their exact TTLV field types, values, and order"
        )
        assert [request.index(item) for item in expected_items] == sorted(
            request.index(item) for item in expected_items
        ), "the ordered batch preserves the item order"
    finally:
        if batch is not None:
            batch.close()
        if result is not None:
            result.close()
        limits.close()
        peer.join(timeout=4)
        if peer.is_alive():
            raise AssertionError("mTLS peer did not finish")
        if not failures.empty():
            raise failures.get()
