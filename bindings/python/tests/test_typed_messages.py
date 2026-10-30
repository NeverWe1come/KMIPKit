"""Four-language typed message binding contract for Python consumers."""

from __future__ import annotations

import json
from collections.abc import Callable, Iterable
from pathlib import Path

import pytest
from kmipkit import messages, ttlv

_REQUEST_MESSAGE = 0x0042_0078
_RESPONSE_MESSAGE = 0x0042_007B
_REQUEST_HEADER = 0x0042_0077
_RESPONSE_HEADER = 0x0042_007A
_PROTOCOL_VERSION = 0x0042_0069
_PROTOCOL_VERSION_MAJOR = 0x0042_006A
_PROTOCOL_VERSION_MINOR = 0x0042_006B
_BATCH_COUNT = 0x0042_000D
_BATCH_ITEM = 0x0042_000F
_OPERATION = 0x0042_005C
_REQUEST_PAYLOAD = 0x0042_0079
_RESPONSE_PAYLOAD = 0x0042_007C
_RESULT_STATUS = 0x0042_007F
_UNIQUE_BATCH_ITEM_ID = 0x0042_0093
_TIME_STAMP = 0x0042_0092
_ASYNC_INDICATOR = 0x0042_0007
_ATTESTATION_CAPABLE = 0x0042_00D3
_BATCH_ERROR_CONTINUATION = 0x0042_000E
_BATCH_ORDER = 0x0042_0010
_MESSAGE_EXTENSION = 0x0042_0051
_VENDOR_IDENTIFICATION = 0x0042_009D
_CRITICALITY_INDICATOR = 0x0042_0026
_VENDOR_EXTENSION = 0x0042_009C
_SERVER_HASHED_PASSWORD = 0x0042_0155
_ROOT = Path(__file__).resolve().parents[3]
_SHARED_PARITY_FIXTURE = _ROOT / "tests" / "fixtures" / "api" / "shared-capability-parity.json"
_TYPED_MESSAGE_CASE_ID = "typed-message.ordered-open-values-and-lifetime.v1"


def test_typed_message_model_is_declared_as_a_shared_four_language_fixture() -> None:
    fixture = json.loads(_SHARED_PARITY_FIXTURE.read_text(encoding="utf-8"))
    case = next(
        (item for item in fixture["cases"] if item["id"] == _TYPED_MESSAGE_CASE_ID),
        None,
    )

    assert case is not None
    assert set(case["consumerTests"]) == {"rust", "c", "java", "python"}
    assert set(case["input"]) == {"request", "response", "invalidRequest"}
    assert set(case["expectedOutput"]) == {"request", "response", "validation", "lifetime"}
    assert case["consumerTests"]["python"]["status"] == "available"


def _item(raw_tag: int, value: ttlv.Value, limits: ttlv.CodecLimits) -> ttlv.TtlvItem:
    raw = ttlv.raw_tag_from_raw(raw_tag)
    try:
        checked = ttlv.raw_tag_try_checked(raw)
    finally:
        raw.close()
    try:
        return ttlv.ttlv_item_create(checked, value, limits)
    finally:
        checked.close()


def _structure(
    fields: Iterable[tuple[int, Callable[[], ttlv.Value]]],
    limits: ttlv.CodecLimits,
) -> ttlv.Structure:
    result = ttlv.ttlv_structure_create()
    for raw_tag, make_value in fields:
        result = ttlv.ttlv_structure_with_item(
            result, _item(raw_tag, make_value(), limits), limits
        )
    return result


def _integer(value: int) -> Callable[[], ttlv.Value]:
    return lambda: ttlv.ttlv_value_integer(value)


def _enumeration(value: int) -> Callable[[], ttlv.Value]:
    return lambda: ttlv.ttlv_value_enumeration(value)


def _boolean(value: bool) -> Callable[[], ttlv.Value]:
    return lambda: ttlv.ttlv_value_boolean(value)


def _text(value: bytes, limits: ttlv.CodecLimits) -> Callable[[], ttlv.Value]:
    return lambda: ttlv.ttlv_value_text_string(value, limits)


def _nested(
    fields: Iterable[tuple[int, Callable[[], ttlv.Value]]],
    limits: ttlv.CodecLimits,
) -> Callable[[], ttlv.Value]:
    def make_value() -> ttlv.Value:
        return ttlv.ttlv_value_structure(_structure(fields, limits), limits)

    return make_value


def _limits() -> ttlv.CodecLimits:
    return ttlv.codec_limits_defaults()


def _request_tree(
    *,
    operation: int = 0xF123_4567,
    malformed_operation: bytes | None = None,
    include_extension: bool = False,
    header_options: tuple[tuple[int, Callable[[], ttlv.Value]], ...] = (),
) -> ttlv.Structure:
    limits = _limits()
    header_fields: list[tuple[int, Callable[[], ttlv.Value]]] = [
        (
            _PROTOCOL_VERSION,
            _nested(
                [
                    (_PROTOCOL_VERSION_MAJOR, _integer(2)),
                    (_PROTOCOL_VERSION_MINOR, _integer(1)),
                ],
                limits,
            ),
        ),
        *header_options,
        (_BATCH_COUNT, _integer(1)),
    ]
    batch_fields: list[tuple[int, Callable[[], ttlv.Value]]] = [
        (
            _OPERATION,
            _text(malformed_operation, limits)
            if malformed_operation is not None
            else _enumeration(operation),
        ),
        (_REQUEST_PAYLOAD, _nested([], limits)),
    ]
    if include_extension:
        batch_fields.append(
            (
                _MESSAGE_EXTENSION,
                _nested(
                    [
                        (_VENDOR_IDENTIFICATION, _text(b"Acme", limits)),
                        (_CRITICALITY_INDICATOR, _boolean(False)),
                        (
                            _VENDOR_EXTENSION,
                            _nested(
                                [(0x0054_0001, _text(b"opaque vendor value", limits))],
                                limits,
                            ),
                        ),
                    ],
                    limits,
                ),
            )
        )
    result = _structure(
        [
            (_REQUEST_HEADER, _nested(header_fields, limits)),
            (_BATCH_ITEM, _nested(batch_fields, limits)),
        ],
        limits,
    )
    limits.close()
    return result


def _response_tree(
    *,
    status: int = 0x8000_0001,
    header_options: tuple[tuple[int, Callable[[], ttlv.Value]], ...] = (),
) -> ttlv.Structure:
    limits = _limits()
    result = _structure(
        [
            (
                _RESPONSE_HEADER,
                _nested(
                    [
                        (
                            _PROTOCOL_VERSION,
                            _nested(
                                [
                                    (_PROTOCOL_VERSION_MAJOR, _integer(2)),
                                    (_PROTOCOL_VERSION_MINOR, _integer(1)),
                                ],
                                limits,
                            ),
                        ),
                        (_TIME_STAMP, lambda: ttlv.ttlv_value_date_time(1_234_567)),
                        *header_options,
                        (_BATCH_COUNT, _integer(1)),
                    ],
                    limits,
                ),
            ),
            (
                _BATCH_ITEM,
                _nested(
                    [
                        (_OPERATION, _enumeration(0x0000_000A)),
                        (_RESULT_STATUS, _enumeration(status)),
                        (_RESPONSE_PAYLOAD, _nested([], limits)),
                    ],
                    limits,
                ),
            ),
        ],
        limits,
    )
    limits.close()
    return result


def _field_tags(structure: ttlv.Structure) -> list[int]:
    tags: list[int] = []
    with ttlv.ttlv_structure_view(structure) as structure_view:
        for index in range(ttlv.ttlv_structure_view_item_count(structure_view)):
            with (
                ttlv.ttlv_structure_view_item_at(structure_view, index) as item_view,
                ttlv.ttlv_item_view_tag(item_view) as tag,
            ):
                tags.append(ttlv.tag_value(tag))
    return tags


def _fixture_tag(value: str) -> int:
    return int(value, 16)


def _fixture_integer_from_raw_hex(value: str) -> int:
    raw = int(value, 16)
    return raw if raw < 0x8000_0000 else raw - 0x1_0000_0000


def _fixture_extension(
    extension: dict[str, object], limits: ttlv.CodecLimits
) -> Callable[[], ttlv.Value]:
    vendor_tag = _fixture_tag(str(extension["vendorTag"]))
    vendor_value = bytes.fromhex(str(extension["vendorValueHex"]))
    return _nested(
        [
            (
                _VENDOR_IDENTIFICATION,
                _text(str(extension["vendorIdentification"]).encode(), limits),
            ),
            (
                _CRITICALITY_INDICATOR,
                _boolean(bool(extension["criticalityIndicator"])),
            ),
            (
                _VENDOR_EXTENSION,
                _nested([(vendor_tag, lambda: ttlv.ttlv_value_byte_string(vendor_value, limits))], limits),
            ),
        ],
        limits,
    )


def _typed_request_tree(
    request_input: dict[str, object], *, invalid: dict[str, object] | None = None
) -> ttlv.Structure:
    limits = _limits()
    try:
        batch_items = request_input["batchItems"]
        assert isinstance(batch_items, list)
        header_fields: list[tuple[int, Callable[[], ttlv.Value]]] = []
        for raw_tag in request_input["headerFieldTags"]:  # type: ignore[union-attr]
            tag = _fixture_tag(str(raw_tag))
            if tag == _PROTOCOL_VERSION:
                header_fields.append(
                    (
                        tag,
                        _nested(
                            [
                                (_PROTOCOL_VERSION_MAJOR, _integer(2)),
                                (_PROTOCOL_VERSION_MINOR, _integer(1)),
                            ],
                            limits,
                        ),
                    )
                )
            elif tag == _BATCH_COUNT:
                header_fields.append((tag, _integer(len(batch_items))))
            else:
                raise AssertionError(f"unexpected fixture request header tag {tag:#x}")

        root_fields: list[tuple[int, Callable[[], ttlv.Value]]] = [
            (_REQUEST_HEADER, _nested(header_fields, limits))
        ]
        extension = request_input["extension"]
        assert isinstance(extension, dict)
        for index, raw_item in enumerate(batch_items):
            assert isinstance(raw_item, dict)
            fields: list[tuple[int, Callable[[], ttlv.Value]]] = [
                (_OPERATION, _enumeration(int(raw_item["operationRaw"])))
            ]
            missing_id_index = (
                int(invalid["missingBatchItemIdIndex"]) if invalid is not None else -1
            )
            if index != missing_id_index:
                fields.append(
                    (
                        _UNIQUE_BATCH_ITEM_ID,
                        lambda item=raw_item: ttlv.ttlv_value_byte_string(
                            bytes.fromhex(str(item["uniqueBatchItemIdHex"])), limits
                        ),
                    )
                )
            payload_fields: list[tuple[int, Callable[[], ttlv.Value]]] = [
                (
                    _fixture_tag(str(raw_item["bitMaskTag"])),
                    _integer(_fixture_integer_from_raw_hex(str(raw_item["bitMaskRawHex"]))),
                )
            ]
            if invalid is not None and index == missing_id_index:
                payload_fields.append(
                    (
                        0x0054_0002,
                        _text(str(invalid["payloadMarker"]).encode(), limits),
                    )
                )
            fields.append((_REQUEST_PAYLOAD, _nested(payload_fields, limits)))
            if bool(raw_item["messageExtension"]):
                fields.append((_MESSAGE_EXTENSION, _fixture_extension(extension, limits)))
            root_fields.append((_BATCH_ITEM, _nested(fields, limits)))
        return _structure(root_fields, limits)
    finally:
        limits.close()


def _typed_response_tree(response_input: dict[str, object]) -> ttlv.Structure:
    limits = _limits()
    try:
        header_fields: list[tuple[int, Callable[[], ttlv.Value]]] = []
        for raw_tag in response_input["headerFieldTags"]:  # type: ignore[union-attr]
            tag = _fixture_tag(str(raw_tag))
            if tag == _PROTOCOL_VERSION:
                header_fields.append(
                    (
                        tag,
                        _nested(
                            [
                                (_PROTOCOL_VERSION_MAJOR, _integer(2)),
                                (_PROTOCOL_VERSION_MINOR, _integer(1)),
                            ],
                            limits,
                        ),
                    )
                )
            elif tag == _TIME_STAMP:
                header_fields.append((tag, lambda: ttlv.ttlv_value_date_time(int(response_input["timeStamp"]))))
            elif tag == _BATCH_COUNT:
                header_fields.append((tag, _integer(1)))
            else:
                raise AssertionError(f"unexpected fixture response header tag {tag:#x}")

        fields: list[tuple[int, Callable[[], ttlv.Value]]] = [
            (_OPERATION, _enumeration(int(response_input["operationRaw"]))),
            (_RESULT_STATUS, _enumeration(int(response_input["resultStatusRaw"]))),
            (
                _RESPONSE_PAYLOAD,
                _nested(
                    [
                        (
                            _fixture_tag(str(response_input["responsePayloadBitMaskTag"])),
                            _integer(
                                _fixture_integer_from_raw_hex(
                                    str(response_input["responsePayloadBitMaskRawHex"])
                                )
                            ),
                        )
                    ],
                    limits,
                ),
            ),
        ]
        extension = response_input["extension"]
        assert isinstance(extension, dict)
        if bool(response_input["messageExtension"]):
            fields.append((_MESSAGE_EXTENSION, _fixture_extension(extension, limits)))
        return _structure(
            [
                (_RESPONSE_HEADER, _nested(header_fields, limits)),
                (_BATCH_ITEM, _nested(fields, limits)),
            ],
            limits,
        )
    finally:
        limits.close()


def _first_integer(structure_view: ttlv.TtlvStructureView) -> tuple[int, int]:
    with ttlv.ttlv_structure_view_item_at(structure_view, 0) as item_view:
        with ttlv.ttlv_item_view_tag(item_view) as tag:
            raw_tag = ttlv.tag_value(tag)
        with ttlv.ttlv_item_view_value(item_view) as value_view:
            return raw_tag, ttlv.ttlv_value_view_integer(value_view)


def _assert_extension_matches_fixture(
    extension: messages.MessageExtensionView, expected: dict[str, object]
) -> None:
    assert extension.vendor_identification == expected["extensionVendorIdentification"]
    assert extension.criticality_indicator is expected["extensionCriticalityIndicator"]
    copied = extension.to_ttlv()
    assert _field_tags(copied) == [
        _VENDOR_IDENTIFICATION,
        _CRITICALITY_INDICATOR,
        _VENDOR_EXTENSION,
    ]
    copied.close()
    with extension.vendor_extension as view:
        with ttlv.ttlv_structure_view_item_at(view, 0) as item_view:
            with ttlv.ttlv_item_view_tag(item_view) as tag:
                assert ttlv.tag_value(tag) == _fixture_tag(str(expected["extensionVendorTag"]))
            with ttlv.ttlv_item_view_value(item_view) as value_view:
                raw = bytes(
                    ttlv.ttlv_value_view_byte_at(value_view, index)
                    for index in range(ttlv.ttlv_value_view_byte_length(value_view))
                )
    assert raw.hex() == expected["extensionVendorValueHex"]


def test_typed_message_model_matches_shared_fixture() -> None:
    fixture = json.loads(_SHARED_PARITY_FIXTURE.read_text(encoding="utf-8"))
    case = next(
        item for item in fixture["cases"] if item["id"] == _TYPED_MESSAGE_CASE_ID
    )
    input_data = case["input"]
    expected = case["expectedOutput"]
    request_input = input_data["request"]
    request_expected = expected["request"]
    request_tree = _typed_request_tree(request_input)
    request = messages.RequestMessage.from_ttlv(request_tree)
    assert request_tree.closed
    assert request.batch_item_count == request_expected["batchCount"]
    header = request.header()
    header_ttlv = header.to_ttlv()
    assert [f"0x{tag:08x}" for tag in _field_tags(header_ttlv)] == request_expected[
        "headerFieldTags"
    ]
    header_ttlv.close()
    defaults = request_expected["effectiveDefaults"]
    assert header.asynchronous_indicator is None
    assert header.effective_asynchronous_indicator == defaults["asynchronousIndicator"]
    assert header.attestation_capable_indicator is None
    assert header.effective_attestation_capable_indicator is defaults[
        "attestationCapableIndicator"
    ]
    assert header.batch_error_continuation_option is None
    assert header.effective_batch_error_continuation_option == defaults[
        "batchErrorContinuationOption"
    ]
    assert header.batch_order_option is None
    assert header.effective_batch_order_option is defaults["batchOrderOption"]
    for index, item_expected in enumerate(request_expected["batchItems"]):
        with request.batch_item_at(index) as item:
            assert item.operation == item_expected["operationRaw"]
            assert item.unique_batch_item_id == bytes.fromhex(
                item_expected["uniqueBatchItemIdHex"]
            )
            copied_item = item.to_ttlv()
            assert [f"0x{tag:08x}" for tag in _field_tags(copied_item)] == item_expected[
                "fieldTags"
            ]
            copied_item.close()
            with item.request_payload as payload:
                raw_tag, raw_mask = _first_integer(payload)
            assert f"0x{raw_tag:08x}" == request_input["batchItems"][index]["bitMaskTag"]
            assert f"0x{raw_mask & 0xFFFF_FFFF:08x}" == item_expected["bitMaskRawHex"]
            if index == request_expected["extensionBatchItemIndex"]:
                assert item.extension_count == 1
                extension = item.extension_at(0)
                _assert_extension_matches_fixture(extension, request_expected)
                extension.close()
    request.close()
    header.close()

    response_input = input_data["response"]
    response_expected = expected["response"]
    response_tree = _typed_response_tree(response_input)
    response = messages.ResponseMessage.from_ttlv(response_tree)
    assert response_tree.closed
    assert response.batch_item_count == response_expected["batchCount"]
    with response.header() as response_header:
        response_header_ttlv = response_header.to_ttlv()
        assert [f"0x{tag:08x}" for tag in _field_tags(response_header_ttlv)] == response_expected[
            "headerFieldTags"
        ]
        response_header_ttlv.close()
        assert response_header.time_stamp == response_input["timeStamp"]
    with response.batch_item_at(0) as item:
        assert item.operation == response_expected["operationRaw"]
        assert item.result_status == response_expected["resultStatusRaw"]
        item_ttlv = item.to_ttlv()
        assert [f"0x{tag:08x}" for tag in _field_tags(item_ttlv)] == response_expected[
            "batchItemFieldTags"
        ]
        item_ttlv.close()
        extension = item.extension_at(0)
    response.close()
    _assert_extension_matches_fixture(extension, response_expected)
    extension.close()

    invalid_input = input_data["invalidRequest"]
    invalid_tree = _typed_request_tree(request_input, invalid=invalid_input)
    validation_expected = expected["validation"]
    with pytest.raises(messages.MessageValidationError) as raised:
        messages.RequestMessage.from_ttlv(invalid_tree)
    assert invalid_tree.closed
    assert raised.value.kind == validation_expected["kindCode"]
    assert raised.value.structure_index == validation_expected["structureIndex"]
    assert raised.value.field_index == validation_expected["fieldIndex"]
    marker = validation_expected["payloadMarker"]
    assert marker not in str(raised.value)
    assert marker not in repr(raised.value)


def test_request_message_preserves_unknown_operation_order_and_effective_defaults() -> (
    None
):
    original = _request_tree()

    with messages.RequestMessage.from_ttlv(original) as request:
        assert original.closed
        assert request.batch_item_count == 1
        with request.header() as header:
            assert (header.protocol_version.major, header.protocol_version.minor) == (
                2,
                1,
            )
            assert header.batch_count == 1
            assert header.asynchronous_indicator is None
            assert header.effective_asynchronous_indicator == 3
            assert header.attestation_capable_indicator is None
            assert header.effective_attestation_capable_indicator is False
            assert header.batch_error_continuation_option is None
            assert header.effective_batch_error_continuation_option == 2
            assert header.batch_order_option is None
            assert header.effective_batch_order_option is True

        with request.batch_item_at(0) as item:
            assert item.operation == 0xF123_4567
            copied_item = item.to_ttlv()
        assert _field_tags(copied_item) == [_OPERATION, _REQUEST_PAYLOAD]
        copied_item.close()

        copied_message = request.to_ttlv()
    assert _field_tags(copied_message) == [_REQUEST_HEADER, _BATCH_ITEM]
    copied_message.close()


def test_request_message_preserves_explicit_raw_header_option_values() -> None:
    original = _request_tree(
        header_options=((_ASYNC_INDICATOR, _enumeration(0xF123_4567)),)
    )

    with (
        messages.RequestMessage.from_ttlv(original) as request,
        request.header() as header,
    ):
        assert header.asynchronous_indicator == 0xF123_4567
        assert header.effective_asynchronous_indicator == 0xF123_4567


def test_response_message_preserves_raw_status_and_response_timestamp() -> None:
    original = _response_tree()

    with messages.ResponseMessage.from_ttlv(original) as response:
        assert original.closed
        with response.header() as header:
            assert (header.protocol_version.major, header.protocol_version.minor) == (
                2,
                1,
            )
            assert header.time_stamp == 1_234_567
            assert header.batch_count == 1
        with response.batch_item_at(0) as item:
            assert item.operation == 0x0000_000A
            assert item.result_status == 0x8000_0001
            copied_item = item.to_ttlv()
    assert _field_tags(copied_item) == [_OPERATION, _RESULT_STATUS, _RESPONSE_PAYLOAD]
    copied_item.close()


def test_response_header_preserves_server_hashed_password_bytes() -> None:
    limits = _limits()
    try:
        original = _response_tree(
            header_options=(
                (
                    _SERVER_HASHED_PASSWORD,
                    lambda: ttlv.ttlv_value_byte_string(b"\x00\x80hash", limits),
                ),
            )
        )
    finally:
        limits.close()

    with (
        messages.ResponseMessage.from_ttlv(original) as response,
        response.header() as header,
    ):
        assert header.server_hashed_password == b"\x00\x80hash"


def test_extension_view_survives_parent_close_and_returns_lossless_ttlv() -> None:
    original = _request_tree(operation=0x0000_001E, include_extension=True)
    request = messages.RequestMessage.from_ttlv(original)
    item = request.batch_item_at(0)
    assert item.extension_count == 1
    extension = item.extension_at(0)
    request.close()
    item.close()

    with extension:
        copied_extension = extension.to_ttlv()
    assert _field_tags(copied_extension) == [
        _VENDOR_IDENTIFICATION,
        _CRITICALITY_INDICATOR,
        _VENDOR_EXTENSION,
    ]
    copied_extension.close()


def test_invalid_message_error_is_payload_free_and_consumes_input() -> None:
    original = _request_tree(malformed_operation=b"DO-NOT-LEAK-MESSAGE-CONTENT")

    with pytest.raises(messages.MessageValidationError) as raised:
        messages.RequestMessage.from_ttlv(original)

    assert original.closed
    assert raised.value.kind > 0
    assert raised.value.structure_index is not None
    assert "DO-NOT-LEAK-MESSAGE-CONTENT" not in str(raised.value)
    assert "DO-NOT-LEAK-MESSAGE-CONTENT" not in repr(raised.value)
