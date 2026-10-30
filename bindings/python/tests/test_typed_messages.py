"""Four-language typed message binding contract for Python consumers."""

from __future__ import annotations

from collections.abc import Callable, Iterable

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
