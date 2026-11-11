"""KMIPKIT-0096 FR-001–004; KMIP v2.1 §§4–5, §4.60 Table 150, §§7.1–8."""

from contextlib import contextmanager
import socket

import pytest

from kmipkit import client, errors, extensions, fluent, ttlv
from kmipkit.typed_attributes import StandardAttribute
from _test_support import mtls_transport_config, read_ttlv_frame, start_tls13_mtls_peer
from test_fluent import _FIXTURES, _children, _create_response, _item, _parse_item, _structure

_PRIVATE = b"position-private-value"


@contextmanager
def _expert_item(limits, tag_value=0x540001):
    with ttlv.raw_tag_from_raw(tag_value) as raw:
        with ttlv.raw_tag_try_checked(raw) as tag:
            with ttlv.ttlv_value_text_string(_PRIVATE, limits) as value:
                with ttlv.ttlv_item_create(tag, value, limits) as item:
                    yield item


def _assert_redacted(value):
    assert _PRIVATE.decode() not in str(value)
    assert _PRIVATE.decode() not in repr(value)


@pytest.mark.parametrize(
    "positions, expected_tags",
    [
        ((0,), [0x540001, 0x420028, 0x42002A, 0x42002C, 0x420053, 0x4200FC]),
        ((2,), [0x420028, 0x42002A, 0x540001, 0x42002C, 0x420053, 0x4200FC]),
        ((5,), [0x420028, 0x42002A, 0x42002C, 0x420053, 0x4200FC, 0x540001]),
        ((7, 2, 0), [0x540001, 0x420028, 0x540003, 0x42002A, 0x42002C, 0x420053, 0x4200FC, 0x540008]),
    ],
)
def test_positioned_items_preserve_final_wire_order_and_survive_source_close(positions, expected_tags):
    """Catch append-only or call-order insertion and borrowed native Item storage."""
    observed = []

    def serve(stream):
        request = read_ttlv_frame(stream)
        root = _parse_item(request, 0, len(request))
        batch = next(item for item in _children(request, root) if item[0] == 0x42000F)
        payload = next(item for item in _children(request, batch) if item[0] == 0x420079)
        attributes = next(item for item in _children(request, payload) if item[0] == 0x420125)
        children = _children(request, attributes)
        observed.extend(item[0] for item in children)
        for tag, kind, start, _end in children:
            if 0x540000 <= tag <= 0x54FFFF:
                length = int.from_bytes(request[start - 4:start], "big")
                assert kind == 7
                assert request[start:start + length] == _PRIVATE
        stream.sendall(_create_response())

    port, failures, peer = start_tls13_mtls_peer(_FIXTURES, serve)
    try:
        with extensions.create_client_extension_registry((), extensions.default_extension_registry_limits()) as registry:
            with extensions.create_client_configuration(registry) as configuration:
                with client.Client.create(configuration, mtls_transport_config(port, _FIXTURES)) as kmip_client:
                    with ttlv.codec_limits_defaults() as limits, kmip_client.request().create_symmetric_key() as builder:
                        builder.aes_256().usage_mask(12).name("ordered-key")
                        with ttlv.ttlv_value_text_string(b"description", limits) as description:
                            builder.add_attribute(StandardAttribute.DESCRIPTION, description)
                        for index in positions:
                            tag = 0x540001 + index if len(positions) > 1 else 0x540001
                            with _expert_item(limits, tag) as item:
                                assert builder.attribute_item_at(index, item) is builder
                        _assert_redacted(builder)
                        with builder.build() as typed:
                            assert isinstance(typed, extensions.ClientBatchItem)
                            assert typed.operation == 1
                        with builder.execute(limits) as result:
                            assert result.items[0].outcome is client.BatchOutcome.COMPLETED
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert observed == expected_tags
    finally:
        peer.join(timeout=12)
        if not failures.empty():
            raise failures.get()


@pytest.mark.parametrize("index", [True, False, -1, 1 << 64, 1.0, _PRIVATE.decode(), None, object()])
def test_position_rejects_invalid_python_indices_without_retaining_payload(index):
    with ttlv.codec_limits_defaults() as limits, _expert_item(limits) as item:
        with fluent.SymmetricKeyBuilder() as builder:
            builder.aes_256().usage_mask(12)
            with pytest.raises(errors.InvalidInputError) as rejected:
                builder.attribute_item_at(index, item)
            _assert_redacted(rejected.value)
            assert rejected.value.__cause__ is None
            with builder.build() as typed:
                assert typed.operation == 1


@pytest.mark.parametrize("value", [None, True, object(), _PRIVATE])
def test_position_requires_an_explicit_ttlv_item(value):
    with fluent.SymmetricKeyBuilder() as builder:
        with pytest.raises(errors.InvalidInputError) as rejected:
            builder.attribute_item_at(0, value)
        _assert_redacted(rejected.value)


@pytest.mark.parametrize("positions", [(4,), (0, 0), ((1 << 64) - 1,)])
def test_bad_final_positions_fail_build_and_execute_before_connecting(positions):
    """An actual listening socket must receive no connection for invalid requests."""
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        listener.settimeout(0.05)
        with extensions.create_client_extension_registry((), extensions.default_extension_registry_limits()) as registry:
            with extensions.create_client_configuration(registry) as configuration:
                with client.Client.create(configuration, mtls_transport_config(listener.getsockname()[1], _FIXTURES)) as kmip_client:
                    with ttlv.codec_limits_defaults() as limits, _expert_item(limits) as item:
                        with kmip_client.request().create_symmetric_key() as builder:
                            builder.aes_256().usage_mask(12)
                            for index in positions:
                                builder.attribute_item_at(index, item)
                            for action in (builder.build, lambda: builder.execute(limits)):
                                with pytest.raises(errors.ClientExecutionError) as rejected:
                                    action()
                                _assert_redacted(rejected.value)
                                assert rejected.value.__cause__ is None
                                assert rejected.value.client_category is errors.ClientErrorCategory.VALIDATION
                                assert rejected.value.delivery is errors.DeliveryState.NOT_SENT
                            with pytest.raises(TimeoutError):
                                listener.accept()


def test_positioned_catalogued_item_keeps_native_type_validation():
    with ttlv.codec_limits_defaults() as limits, _expert_item(limits, 0x420028) as item:
        with fluent.SymmetricKeyBuilder() as builder:
            builder.aes_256().usage_mask(12)
            with pytest.raises(errors.ClientExecutionError) as rejected:
                builder.attribute_item_at(0, item)
            _assert_redacted(rejected.value)
            assert rejected.value.delivery is errors.DeliveryState.NOT_SENT
            with builder.build() as typed:
                assert typed.operation == 1


def test_positioned_item_does_not_bypass_required_key_properties():
    with ttlv.codec_limits_defaults() as limits, _expert_item(limits) as item:
        with fluent.SymmetricKeyBuilder() as builder:
            builder.attribute_item_at(0, item)
            with pytest.raises(errors.InvalidInputError):
                builder.build()


@pytest.mark.parametrize("tag", [0x4200FC, 0x540001])
def test_position_accepts_catalogued_and_explicit_unknown_items(tag):
    with ttlv.codec_limits_defaults() as limits, _expert_item(limits, tag) as item:
        with fluent.SymmetricKeyBuilder() as builder:
            builder.aes_256().usage_mask(12).attribute_item_at(1, item)
            with builder.to_batch_item() as typed:
                assert typed.operation == 1


@pytest.mark.parametrize("field_order, valid", [((0, 1, 2), True), ((1, 0, 2), False), ((0, 2), False)])
def test_position_preserves_table_150_vendor_field_validation(field_order, valid):
    fields = (
        _item(0x42009D, 7, b"example.invalid"),
        _item(0x42000A, 7, b"private-label"),
        _item(0x42000B, 7, _PRIVATE),
    )
    with ttlv.ttlv_decode(_structure(0x420008, *(fields[index] for index in field_order))) as item:
        with fluent.SymmetricKeyBuilder() as builder:
            builder.aes_256().usage_mask(12)
            if valid:
                builder.attribute_item_at(0, item)
                with builder.build() as typed:
                    assert typed.operation == 1
            else:
                with pytest.raises(errors.ClientExecutionError) as rejected:
                    builder.attribute_item_at(0, item)
                _assert_redacted(rejected.value)
                assert rejected.value.delivery is errors.DeliveryState.NOT_SENT


def test_positioned_item_rejects_closed_builder_and_closed_item():
    with ttlv.codec_limits_defaults() as limits, _expert_item(limits) as item:
        builder = fluent.SymmetricKeyBuilder()
        builder.close()
        builder.close()
        with pytest.raises(RuntimeError, match="closed"):
            builder.attribute_item_at(0, item)
        with fluent.SymmetricKeyBuilder() as builder:
            item.close()
            with pytest.raises(errors.InvalidInputError):
                builder.attribute_item_at(0, item)


def test_non_builder_native_argument_failure_keeps_generic_delivery_metadata():
    # KMIPKIT-0096-FR-003: scoped builder metadata must not change other ABI calls.
    with pytest.raises(errors.ClientExecutionError) as rejected:
        client._raise_client_status(client.lib.kmipkit_client_batch_create(client.ffi.NULL))
    assert rejected.value.delivery is errors.DeliveryState.NONE
    assert rejected.value.client_category is errors.ClientErrorCategory.VALIDATION
    _assert_redacted(rejected.value)
