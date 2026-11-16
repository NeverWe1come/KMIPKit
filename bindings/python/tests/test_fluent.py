from __future__ import annotations

import queue
import socket
import ssl
import struct
import threading
from pathlib import Path

import pytest
from _test_support import (
    mtls_transport_config,
    read_ttlv_frame,
    require_tls13_mtls_peer,
    start_tls13_mtls_peer,
)
from kmipkit import attribute_operations, client, errors, extensions, fluent, protocol_values, ttlv
from kmipkit.typed_attributes import StandardAttribute

_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"


def test_fluent_request_exposes_a_delete_attribute_builder():
    assert callable(getattr(fluent.Request, "delete_attribute", None))
    assert hasattr(fluent, "DeleteAttributeBuilder")
    assert "DeleteAttributeBuilder" in fluent.__all__


def test_fluent_request_exposes_locate_and_revoke_builders():
    assert callable(getattr(fluent.Request, "locate", None))
    assert callable(getattr(fluent.Request, "revoke", None))
    assert hasattr(fluent, "LocateBuilder")
    assert hasattr(fluent, "RevokeBuilder")
    assert "LocateBuilder" in fluent.__all__
    assert "RevokeBuilder" in fluent.__all__


def test_locate_and_revoke_build_typed_items_with_explicit_optional_fields():
    with ttlv.ttlv_structure_create() as empty_items:
        with attribute_operations.AttributeSet.try_new(empty_items) as attributes:
            with fluent.Request(None).locate(attributes) as locate:
                item = (
                    locate.maximum_items(12)
                    .offset_items(3)
                    .storage_status_mask(2)
                    .object_group_member(1)
                    .asynchronous_indicator(5)
                    .to_batch_item()
                )
                assert item.operation == 8
                item.close()

    revoke = (
        fluent.Request(None)
        .revoke(2, "key material compromised")
        .unique_identifier(extensions.UniqueIdentifier.text_string("lifecycle-key"))
        .compromise_occurrence_date(1_700_000_000)
        .asynchronous_indicator(7)
    )
    item = revoke.to_batch_item()
    assert item.operation == 19
    item.close()

    invalid = fluent.Request(None).revoke(1).compromise_occurrence_date(1_700_000_000)
    with pytest.raises(errors.InvalidInputError):
        invalid.build()


def test_locate_and_revoke_execute_forward_options_and_async_indicator(monkeypatch):
    calls = []

    class FakeBatch:
        @classmethod
        def create(cls):
            return cls()

        def __enter__(self):
            return self

        def __exit__(self, *_arguments):
            return None

        def add_item(self, item):
            self.operation = item.operation

        def set_asynchronous_indicator(self, value):
            self.asynchronous_indicator = value

    class FakeClient:
        def execute(self, batch, limits):
            calls.append(("default", batch.operation, batch.asynchronous_indicator))
            return "default-result"

        def execute_with_options(self, batch, limits, options):
            calls.append(("options", batch.operation, batch.asynchronous_indicator, options))
            return "options-result"

    monkeypatch.setattr(fluent, "Batch", FakeBatch)
    monkeypatch.setattr(fluent, "Client", FakeClient)
    fake_client = FakeClient()
    configured_options = client.RequestOptions()
    override_options = client.RequestOptions()

    with ttlv.codec_limits_defaults() as limits, ttlv.ttlv_structure_create() as empty_items:
        with attribute_operations.AttributeSet.try_new(empty_items) as attributes:
            with fluent.Request(fake_client).locate(attributes) as locate:
                assert locate.with_options(configured_options) is locate
                assert locate.asynchronous_indicator(3) is locate
                assert locate.execute(limits) == "options-result"
                assert locate.execute_with_options(limits, override_options) == "options-result"

        revoke = fluent.Request(fake_client).revoke(2, "rekey after compromise")
        assert revoke.with_options(configured_options) is revoke
        assert revoke.asynchronous_indicator(4) is revoke
        assert revoke.execute(limits) == "options-result"
        assert revoke.execute_with_options(limits, override_options) == "options-result"

    assert [call[:3] for call in calls] == [
        ("options", 8, 3),
        ("options", 8, 3),
        ("options", 19, 4),
        ("options", 19, 4),
    ]

    with ttlv.ttlv_structure_create() as empty_items:
        with attribute_operations.AttributeSet.try_new(empty_items) as attributes:
            with pytest.raises(errors.InvalidInputError):
                fluent.Request(fake_client).locate(attributes).with_options(object())
    with pytest.raises(errors.InvalidInputError):
        fluent.Request(fake_client).revoke(2).with_options(object())


def test_delete_attribute_builder_builds_an_independent_typed_request():
    identifier = extensions.UniqueIdentifier.text_string("delete-attribute-key")
    reference = extensions.AttributeReference.tag(StandardAttribute.DESCRIPTION.tag)
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.ttlv_value_text_string(b"retired description", limits) as value:
            with fluent.DeleteAttributeBuilder() as builder:
                builder.unique_identifier(identifier.text)
                builder.catalogued_attribute(StandardAttribute.DESCRIPTION, value)
                builder.attribute_reference(reference)
                builder.validate()

                with builder.build_request() as request:
                    assert isinstance(request, attribute_operations.DeleteAttributeRequest)
                    assert request.unique_identifier == identifier
                    assert request.attribute_reference == reference
                    with request.current_attribute as current:
                        assert current is not None
                    with request.to_ttlv_payload() as payload:
                        with ttlv.ttlv_structure_view(payload) as view:
                            assert ttlv.ttlv_structure_view_item_count(view) == 3


def test_delete_attribute_builder_supports_explicit_vendor_options_and_validation():
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.raw_tag_from_raw(0x004200FC) as raw_tag:
            with ttlv.raw_tag_try_checked(raw_tag) as tag:
                with ttlv.ttlv_value_text_string(b"expert value", limits) as value:
                    with ttlv.ttlv_item_create(tag, value, limits) as item:
                        reference = extensions.AttributeReference.name("KMIPKit", "Region")
                        with fluent.DeleteAttributeBuilder() as builder:
                            assert builder.unique_identifier("key-42") is builder
                            assert builder.current_attribute_item(item) is builder
                            assert builder.attribute_reference(reference) is builder
                            assert builder.with_options(client.RequestOptions()) is builder
                            assert builder.asynchronous_indicator(3) is builder
                            builder.validate()
                            with builder.build_request() as request:
                                assert request.attribute_reference == reference
                                with request.current_attribute as current:
                                    assert current is not None
                            with builder.build() as batch_item:
                                assert batch_item.operation == protocol_values.Operation.DELETE_ATTRIBUTE
                            with pytest.raises(errors.KmipKitError):
                                builder.execute(limits)

                            with pytest.raises(errors.KmipKitError):
                                builder.unique_identifier(42)
                            with pytest.raises(errors.KmipKitError):
                                builder.current_attribute_item(object())
                            with pytest.raises(errors.KmipKitError):
                                builder.catalogued_attribute(object(), value)
                            with pytest.raises(errors.KmipKitError):
                                builder.vendor_attribute("Vendor", "region", object())
                            with pytest.raises(errors.KmipKitError):
                                builder.attribute_reference(object())
                            with pytest.raises(errors.KmipKitError):
                                builder.with_options(object())
                            with pytest.raises(errors.KmipKitError):
                                builder.asynchronous_indicator(-1)
                            with pytest.raises(errors.KmipKitError):
                                builder.asynchronous_indicator(0x1_0000_0000)

                        with fluent.DeleteAttributeBuilder() as vendor_builder:
                            assert vendor_builder.current_vendor_attribute(
                                "Vendor_Name.1", "region", item
                            ) is vendor_builder
                            vendor_builder.validate()
                            with vendor_builder.build_request() as request:
                                with request.current_attribute as current:
                                    assert current is not None
                            vendor_builder.close()
                            with pytest.raises(RuntimeError, match="closed"):
                                vendor_builder.validate()


def _item(tag: int, item_type: int, value: bytes) -> bytes:
    return tag.to_bytes(3, "big") + bytes((item_type,)) + len(value).to_bytes(4, "big") + value + bytes((-len(value)) % 8)


def _structure(tag: int, *children: bytes) -> bytes:
    return _item(tag, 1, b"".join(children))


def _parse_item(data: bytes, offset: int, parent_end: int) -> tuple[int, int, int, int]:
    tag = int.from_bytes(data[offset : offset + 3], "big")
    item_type = data[offset + 3]
    length = int.from_bytes(data[offset + 4 : offset + 8], "big")
    value_start = offset + 8
    end = value_start + length + (-length % 8)
    assert end <= parent_end
    return tag, item_type, value_start, end


def _children(data: bytes, parent: tuple[int, int, int, int]) -> list[tuple[int, int, int, int]]:
    _, item_type, value_start, padded_end = parent
    assert item_type == 1
    value_end = value_start + int.from_bytes(data[value_start - 4 : value_start], "big")
    children = []
    offset = value_start
    while offset < value_end:
        child = _parse_item(data, offset, value_end)
        children.append(child)
        offset = child[3]
    assert offset == value_end and padded_end <= len(data)
    return children


def _create_response() -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 1)),
    )
    item = _structure(
        0x42000F,
        _item(0x42005C, 5, struct.pack(">I", 1)),
        _item(0x42007F, 5, struct.pack(">I", 0)),
        _structure(
            0x42007C,
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, b"fluent-key"),
        ),
    )
    return _structure(0x42007B, header, item)


def _request_header_indicator(request: bytes) -> int:
    message = _parse_item(request, 0, len(request))
    header = next(child for child in _children(request, message) if child[0] == 0x420077)
    indicators = [child for child in _children(request, header) if child[0] == 0x420007]
    assert len(indicators) == 1
    indicator = indicators[0]
    encoded_length = int.from_bytes(request[indicator[2] - 4 : indicator[2]], "big")
    assert indicator[1] == 5 and encoded_length == 4
    return int.from_bytes(request[indicator[2] : indicator[2] + 4], "big")


def _request_operation_and_identifier(request: bytes) -> tuple[int, bytes | None]:
    root = _parse_item(request, 0, len(request))
    batch_item = next(child for child in _children(request, root) if child[0] == 0x42000F)
    fields = _children(request, batch_item)
    operation = next(child for child in fields if child[0] == 0x42005C)
    operation_raw = int.from_bytes(request[operation[2] : operation[2] + 4], "big")
    payload = next((child for child in fields if child[0] == 0x420079), None)
    if payload is None:
        return operation_raw, None
    identifier = next((child for child in _children(request, payload) if child[0] == 0x420094), None)
    if identifier is None:
        return operation_raw, None
    assert identifier[1] == 7
    length = int.from_bytes(request[identifier[2] - 4 : identifier[2]], "big")
    return operation_raw, request[identifier[2] : identifier[2] + length]


def _lifecycle_response(operation: int, identifier: bytes) -> bytes:
    version = _structure(
        0x420069,
        _item(0x42006A, 2, struct.pack(">i", 2)),
        _item(0x42006B, 2, struct.pack(">i", 1)),
    )
    header = _structure(
        0x42007A,
        version,
        _item(0x420092, 9, struct.pack(">q", 1)),
        _item(0x42000D, 2, struct.pack(">i", 1)),
    )
    if operation == 1:
        payload_fields = [
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, identifier),
        ]
    elif operation == 10:
        key_block = _structure(
            0x420040,
            _item(0x420042, 5, struct.pack(">I", 1)),
            _structure(
                0x420045,
                _item(0x420043, 8, bytes(range(32))),
            ),
        )
        payload_fields = [
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420094, 7, identifier),
            _structure(0x42008F, key_block),
        ]
    elif operation == 20:
        payload_fields = [_item(0x420094, 7, identifier)]
    elif operation == 11:
        payload_fields = [
            _item(0x420094, 7, identifier),
            _structure(0x420125, _item(0x42002A, 2, struct.pack(">i", 256))),
        ]
    elif operation == 12:
        payload_fields = [
            _item(0x420094, 7, identifier),
            _item(0x42013B, 5, struct.pack(">I", 0x42002A)),
            _structure(
                0x42013B,
                _item(0x42009D, 7, b"KMIPKit"),
                _item(0x42000A, 7, b"Comment"),
            ),
        ]
    else:
        raise AssertionError(f"unexpected fluent operation {operation}")
    response_item = _structure(
        0x42000F,
        _item(0x42005C, 5, struct.pack(">I", operation)),
        _item(0x42007F, 5, struct.pack(">I", 0)),
        _structure(0x42007C, *payload_fields),
    )
    return _structure(0x42007B, header, response_item)


def _start_lifecycle_peer(
    expected_operations: tuple[int, ...],
    *,
    expected_asynchronous_indicator: int | None = None,
    expected_indicators_by_operation: dict[int, int] | None = None,
) -> tuple[int, list[tuple[int, bytes | None, bytes]], queue.Queue[BaseException], threading.Thread]:
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(3)
    listener.settimeout(12)
    port = listener.getsockname()[1]
    observations: list[tuple[int, bytes | None, bytes]] = []
    failures: queue.Queue[BaseException] = queue.Queue()

    def serve() -> None:
        try:
            context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
            context.minimum_version = ssl.TLSVersion.TLSv1_3
            context.maximum_version = ssl.TLSVersion.TLSv1_3
            context.load_cert_chain(_FIXTURES / "server-cert.pem", _FIXTURES / "server-key.pem")
            context.load_verify_locations(cafile=_FIXTURES / "client-cert.pem")
            context.verify_mode = ssl.CERT_REQUIRED
            for expected_operation in expected_operations:
                raw, _ = listener.accept()
                with raw, context.wrap_socket(raw, server_side=True) as stream:
                    require_tls13_mtls_peer(stream)
                    request = read_ttlv_frame(stream)
                    expected_indicator = (expected_indicators_by_operation or {}).get(
                        expected_operation, expected_asynchronous_indicator
                    )
                    if expected_indicator is not None:
                        actual_indicator = _request_header_indicator(request)
                        if actual_indicator != expected_indicator:
                            raise AssertionError(
                                "expected asynchronous indicator "
                                f"{expected_indicator}, got {actual_indicator}"
                            )
                    operation, request_identifier = _request_operation_and_identifier(request)
                    if operation != expected_operation:
                        raise AssertionError(f"expected operation {expected_operation}, got {operation}")
                    observations.append((operation, request_identifier, request))
                    response_identifier = request_identifier or b"fluent-created-id"
                    stream.sendall(_lifecycle_response(operation, response_identifier))
        except BaseException as error:  # noqa: BLE001 - make peer failures visible in the test.
            failures.put(error)
        finally:
            listener.close()

    peer = threading.Thread(target=serve, daemon=True)
    peer.start()
    return port, observations, failures, peer


def test_fluent_symmetric_builder_requires_explicit_key_properties() -> None:
    builder = fluent.SymmetricKeyBuilder()
    try:
        builder.aes_256()
        with pytest.raises(errors.KmipKitError):
            builder.build()
    finally:
        builder.close()


def test_client_request_namespace_and_named_identifier_validation() -> None:
    assert hasattr(client.Client, "request")
    assert hasattr(client.Client, "create_symmetric_key")
    assert hasattr(client.Client, "get_by_id")
    assert hasattr(client.Client, "destroy_by_id")
    with pytest.raises(errors.KmipKitError):
        fluent._identifier("bad")


def test_operation_builds_typed_batch_item() -> None:
    operation = fluent.Operation(
        object(), "get", extensions.UniqueIdentifier.text_string("key-1")
    )
    with operation.build() as item:
        assert item.operation == 10


def test_fluent_operation_and_symmetric_builder_execute_with_and_without_options() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer((10, 20, 1, 1))
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    identifier = extensions.UniqueIdentifier.text_string("fluent-execute-id")
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                results.append(kmip_client.request().get(identifier).execute(limits))
                results.append(kmip_client.request().destroy(identifier).with_options(
                    client.RequestOptions()
                ).execute(limits))
                results.append(kmip_client.request().create_symmetric_key()
                               .aes_256().usage_mask(12).execute(limits))
                results.append(kmip_client.request().create_symmetric_key()
                               .aes_256().usage_mask(12)
                               .with_options(client.RequestOptions()).execute(limits))
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [result.items[0].operation for result in results] == [10, 20, 1, 1]
        assert [operation for operation, _, _ in observations] == [10, 20, 1, 1]
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_destroy_operation_builds_typed_batch_item() -> None:
    operation = fluent.Operation(
        object(), "destroy", extensions.UniqueIdentifier.text_string("key-1")
    )
    with operation.to_batch_item() as item:
        assert item.operation == 20


def test_request_exposes_named_attribute_read_builders() -> None:
    request = fluent.Request(object())
    unique_identifier = extensions.UniqueIdentifier.text_string("key-1")
    references = [extensions.AttributeReference.tag(0x0042_002F)]

    get_attributes = request.get_attributes(
        unique_identifier=unique_identifier,
        attribute_references=references,
    )
    get_attribute_list = request.get_attribute_list(unique_identifier=None)

    with get_attributes.build() as attributes_item:
        assert attributes_item.operation == 11
    with get_attribute_list.build() as list_item:
        assert list_item.operation == 12


def test_attribute_read_builders_execute_single_mtls_exchanges_and_preserve_order() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer(
        (11, 12), expected_indicators_by_operation={12: 2}
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    identifier = extensions.UniqueIdentifier.text_string("attribute-read-key")
    references = (
        extensions.AttributeReference.tag(0x42002A),
        extensions.AttributeReference.name("KMIPKit", "Comment"),
    )
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client, ttlv.codec_limits_defaults() as limits:
            read_attributes = kmip_client.request().get_attributes(
                unique_identifier=identifier, attribute_references=references
            ).with_options(client.RequestOptions())
            list_attributes = kmip_client.request().get_attribute_list(
                unique_identifier=identifier
            ).asynchronous_indicator(2)
            results.append(read_attributes.execute(limits))
            results.append(list_attributes.execute(limits))
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [result.items[0].operation for result in results] == [11, 12]
        assert [operation for operation, _, _ in observations] == [11, 12]
        assert [observed_id for _, observed_id, _ in observations] == [
            b"attribute-read-key", b"attribute-read-key"
        ]
        requests = [request for _, _, request in observations]
        tag_reference = _item(0x42013B, 5, struct.pack(">I", 0x42002A))
        name_reference = _structure(
            0x42013B,
            _item(0x42009D, 7, b"KMIPKit"),
            _item(0x42000A, 7, b"Comment"),
        )
        assert requests[0].index(tag_reference) < requests[0].index(name_reference)
        assert _request_header_indicator(requests[1]) == 2
        assert results[0].items[0].get_attributes_response is not None
        assert results[0].items[0].get_attributes_response.unique_identifier == "attribute-read-key"
        assert results[1].items[0].get_attribute_list_response is not None
        assert results[1].items[0].get_attribute_list_response.attribute_references == references
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_python_direct_attribute_read_methods_execute_single_mtls_exchanges() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer((11, 12))
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    identifier = extensions.UniqueIdentifier.text_string("direct-attribute-read-key")
    references = (
        extensions.AttributeReference.tag(0x42002A),
        extensions.AttributeReference.name("KMIPKit", "Comment"),
    )
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client, ttlv.codec_limits_defaults() as limits:
            with pytest.raises(errors.KmipKitError):
                kmip_client.get_attributes(
                    unique_identifier=None,
                    attribute_references=references,
                    limits=limits,
                )
            with pytest.raises(errors.KmipKitError):
                kmip_client.get_attribute_list(
                    unique_identifier=None, limits=limits
                )
            results.append(
                kmip_client.get_attributes(
                    unique_identifier=identifier,
                    attribute_references=references,
                    limits=limits,
                    options=client.RequestOptions(),
                )
            )
            results.append(
                kmip_client.get_attribute_list(
                    unique_identifier=identifier, limits=limits
                )
            )
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [result.items[0].operation for result in results] == [11, 12]
        assert [operation for operation, _, _ in observations] == [11, 12]
        assert [observed_id for _, observed_id, _ in observations] == [
            b"direct-attribute-read-key", b"direct-attribute-read-key"
        ]
        requests = [request for _, _, request in observations]
        tag_reference = _item(0x42013B, 5, struct.pack(">I", 0x42002A))
        name_reference = _structure(
            0x42013B,
            _item(0x42009D, 7, b"KMIPKit"),
            _item(0x42000A, 7, b"Comment"),
        )
        assert requests[0].index(tag_reference) < requests[0].index(name_reference)
        assert results[0].items[0].get_attributes_response is not None
        assert results[1].items[0].get_attribute_list_response is not None
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_attribute_read_builders_preserve_omitted_identifier_and_reject_duplicates() -> None:
    request = fluent.Request(object())
    list_builder = request.get_attribute_list()
    with list_builder.build() as item:
        assert item.operation == 12
    attributes_builder = request.get_attributes(
        attribute_references=[extensions.AttributeReference.tag(0x42002A)]
    )
    with attributes_builder.build() as item:
        assert item.operation == 11
    duplicate = extensions.AttributeReference.tag(0x42002A)
    with pytest.raises(errors.KmipKitError):
        request.get_attributes(attribute_references=[duplicate, duplicate]).build()


def test_attribute_read_builders_validate_and_expose_typed_batch_items() -> None:
    request = fluent.Request(object())
    identifier = extensions.UniqueIdentifier.text_string("attribute-read-key")
    reference = extensions.AttributeReference.tag(0x42002A)
    builder = request.get_attributes(
        unique_identifier=identifier, attribute_references=[reference]
    )
    builder.validate()
    with builder.to_batch_item() as item:
        assert item.operation == 11

    list_builder = request.get_attribute_list()
    list_builder.validate()
    with list_builder.to_batch_item() as item:
        assert item.operation == 12

    with pytest.raises(errors.KmipKitError):
        builder.with_options(object())  # type: ignore[arg-type]
    with pytest.raises(errors.KmipKitError):
        builder.execute(object())  # type: ignore[arg-type]
    with pytest.raises(errors.KmipKitError):
        request.get_attributes(
            unique_identifier=extensions.UniqueIdentifier.integer(-1),
            attribute_references=[reference],
        )
    with pytest.raises(errors.KmipKitError):
        request.get_attribute_list(
            unique_identifier=extensions.UniqueIdentifier.integer(-1)
        )

    with pytest.raises(errors.KmipKitError):
        request.get_attributes(attribute_references=object())  # type: ignore[arg-type]
    with pytest.raises(errors.KmipKitError):
        request.get_attributes(attribute_references=[object()])  # type: ignore[list-item]
    with pytest.raises(errors.KmipKitError):
        request.get_attributes(attribute_references=[reference] * 100_001)


def test_direct_attributes_maps_native_call_failures_and_null_outputs(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class FakeClient:
        def _pointer(self) -> object:
            return fluent.ffi.NULL

    class FakeLimits:
        def _pointer(self) -> object:
            return fluent.ffi.NULL

    class FakeLibrary:
        def __init__(self, function: object) -> None:
            self.kmipkit_client_get_attributes_v1 = function

    monkeypatch.setattr(fluent, "Client", FakeClient)
    monkeypatch.setattr(fluent.ttlv, "CodecLimits", FakeLimits)
    kmip_client = FakeClient()
    limits = FakeLimits()
    identifier = extensions.UniqueIdentifier.text_string("attribute-read-key")
    references = (extensions.AttributeReference.tag(0x42002A),)

    def raise_type_error(*_arguments: object) -> int:
        raise TypeError("invalid native argument")

    monkeypatch.setattr(fluent, "lib", FakeLibrary(raise_type_error))
    with pytest.raises(errors.KmipKitError):
        fluent._execute_direct_attributes(
            kmip_client, identifier, references, limits, None
        )

    def return_without_response(*_arguments: object) -> int:
        return 0

    monkeypatch.setattr(fluent, "lib", FakeLibrary(return_without_response))
    with pytest.raises(errors.KmipKitError):
        fluent._execute_direct_attributes(
            kmip_client, identifier, references, limits, None
        )


def test_direct_attributes_rejects_invalid_arguments_before_native_call(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class FakeClient:
        def _pointer(self) -> object:
            return fluent.ffi.NULL

    class FakeLimits:
        def _pointer(self) -> object:
            return fluent.ffi.NULL

    monkeypatch.setattr(fluent, "Client", FakeClient)
    monkeypatch.setattr(fluent.ttlv, "CodecLimits", FakeLimits)
    identifier = extensions.UniqueIdentifier.text_string("attribute-read-key")
    reference = extensions.AttributeReference.tag(0x42002A)

    invalid_inputs = (
        (object(), FakeLimits(), identifier, (reference,), None),
        (FakeClient(), object(), identifier, (reference,), None),
        (FakeClient(), FakeLimits(), identifier, (reference,), object()),
        (FakeClient(), FakeLimits(), extensions.UniqueIdentifier.integer(-1), (reference,), None),
        (FakeClient(), FakeLimits(), identifier, object(), None),
        (FakeClient(), FakeLimits(), identifier, (object(),), None),
    )
    for kmip_client, limits, unique_identifier, references, options in invalid_inputs:
        with pytest.raises(errors.KmipKitError):
            fluent._execute_direct_attributes(
                kmip_client, unique_identifier, references, limits, options
            )


def test_operation_sets_batch_level_asynchronous_indicator() -> None:
    operation = fluent.Operation(
        object(), "get", extensions.UniqueIdentifier.text_string("key-1")
    )
    assert operation.asynchronous_indicator(2) is operation
    assert operation._asynchronous_indicator == 2
    with pytest.raises(errors.KmipKitError):
        operation.asynchronous_indicator(True)


def test_operation_async_indicator_is_encoded_in_request_header() -> None:
    port, _, peer_failures, peer = _start_lifecycle_peer(
        (10,), expected_asynchronous_indicator=2
    )
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result: client.BatchResult | None = None
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                result = (
                    kmip_client.request()
                    .get(extensions.UniqueIdentifier.text_string("key-1"))
                    .asynchronous_indicator(2)
                    .execute(limits)
                )
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert result is not None and result.items[0].operation == 10
        assert result.items[0].outcome is client.BatchOutcome.COMPLETED
    finally:
        if result is not None:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_builder_builds_existing_typed_batch_item() -> None:
    with fluent.SymmetricKeyBuilder() as builder:
        item = builder.aes_256().usage_mask(12).name("test-key").build()
    batch = client.Batch.create()
    try:
        batch.add_item(item)
        assert batch.item_count == 1
    finally:
        batch.close()


def test_builder_context_manager_releases_native_handle() -> None:
    builder = fluent.SymmetricKeyBuilder()
    with builder:
        assert builder._handle != fluent.ffi.NULL
    assert builder._handle == fluent.ffi.NULL
    with pytest.raises(RuntimeError, match="closed"):
        builder.name("too-late")


def test_asynchronous_indicator_is_batch_level_and_not_a_request_option() -> None:
    with fluent.SymmetricKeyBuilder() as builder:
        builder.asynchronous_indicator(1)
        assert builder._asynchronous_indicator == 1
        assert builder._options is None
        with pytest.raises(errors.KmipKitError):
            builder.asynchronous_indicator(True)


def test_fluent_async_indicator_is_encoded_in_request_header() -> None:
    observed: list[bytes] = []

    def handler(stream: object) -> None:
        request = read_ttlv_frame(stream)  # type: ignore[arg-type]
        observed.append(request)
        root = _parse_item(request, 0, len(request))
        batch_item = next(child for child in _children(request, root) if child[0] == 0x42000F)
        assert batch_item[0] == 0x42000F
        stream.sendall(_create_response())  # type: ignore[attr-defined]

    port, peer_failures, peer = start_tls13_mtls_peer(_FIXTURES, handler)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    result: client.BatchResult | None = None
    try:
        with configuration, client.Client.create(configuration, mtls_transport_config(port, _FIXTURES)) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                result = (
                    kmip_client.request()
                    .create_symmetric_key()
                    .aes_256()
                    .usage_mask(12)
                    .asynchronous_indicator(2)
                    .execute(limits)
                )
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert len(observed) == 1
        assert _request_header_indicator(observed[0]) == 2
        assert len(result.items) == 1
        assert result.items[0].outcome is client.BatchOutcome.COMPLETED
    finally:
        if result is not None:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_fluent_create_get_destroy_uses_server_created_identifier() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer((1, 10, 20))
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    try:
        with configuration, client.Client.create(configuration, mtls_transport_config(port, _FIXTURES)) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                create_result = (
                    kmip_client.request()
                    .create_symmetric_key()
                    .aes_256()
                    .usage_mask(12)
                    .execute(limits)
                )
                results.append(create_result)
                created_item = create_result.items[0]
                assert created_item.operation == 1
                assert created_item.outcome is client.BatchOutcome.COMPLETED
                assert created_item.result_status == 0
                assert created_item.create_object_type == 2
                assert created_item.unique_identifier == extensions.UniqueIdentifier.text_string(
                    "fluent-created-id"
                )

                created_identifier = created_item.unique_identifier
                assert created_identifier is not None
                get_result = kmip_client.request().get(created_identifier).execute(limits)
                results.append(get_result)
                get_item = get_result.items[0]
                assert get_item.operation == 10
                assert get_item.outcome is client.BatchOutcome.COMPLETED
                assert get_item.result_status == 0
                get_response = get_item.get_response()
                assert get_response is not None
                assert get_response.object_type_raw == 2
                assert get_response.unique_identifier == created_identifier
                assert get_response.object is not None

                destroy_result = kmip_client.request().destroy(created_identifier).execute(limits)
                results.append(destroy_result)
                destroy_item = destroy_result.items[0]
                assert destroy_item.operation == 20
                assert destroy_item.outcome is client.BatchOutcome.COMPLETED
                assert destroy_item.result_status == 0
                assert destroy_item.destroy_response is not None
                assert destroy_item.destroy_response.unique_identifier == created_identifier

        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [observation[:2] for observation in observations] == [
            (1, None),
            (10, b"fluent-created-id"),
            (20, b"fluent-created-id"),
        ]
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_direct_client_helpers_create_get_and_destroy_with_exchange_options() -> None:
    port, observations, peer_failures, peer = _start_lifecycle_peer((1, 10, 20))
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    results: list[client.BatchResult] = []
    identifier = extensions.UniqueIdentifier.text_string("fluent-created-id")
    try:
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, _FIXTURES)
        ) as kmip_client:
            with ttlv.codec_limits_defaults() as limits:
                results.append(kmip_client.create_symmetric_key(
                    limits,
                    algorithm=3,
                    length=256,
                    usage_mask=12,
                    name="direct-key",
                    options=client.RequestOptions(),
                ))
                results.append(kmip_client.get_by_id(
                    identifier, limits, options=client.RequestOptions()
                ))
                results.append(kmip_client.destroy_by_id(
                    identifier, limits, options=client.RequestOptions()
                ))
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [result.items[0].operation for result in results] == [1, 10, 20]
        assert [operation for operation, _, _ in observations] == [1, 10, 20]
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()


def test_symmetric_builder_validates_catalogued_and_explicit_attributes() -> None:
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.ttlv_value_text_string(b"fluently managed", limits) as description:
            with fluent.SymmetricKeyBuilder() as builder:
                builder.aes_256().usage_mask(12).name("expert-key")
                builder.add_attribute(StandardAttribute.DESCRIPTION, description)
                for tag_value, text in ((0x4200FD, b"expert"), (0x540001, b"vendor-value")):
                    raw = ttlv.raw_tag_from_raw(tag_value)
                    try:
                        tag = ttlv.raw_tag_try_checked(raw)
                        try:
                            value = ttlv.ttlv_value_text_string(text, limits)
                            item = ttlv.ttlv_item_create(tag, value, limits)
                            try:
                                builder.add_attribute_item(item)
                                if tag_value == 0x540001:
                                    builder.add_vendor_attribute("Vendor_Name.1", "region", item)
                            finally:
                                item.close()
                        finally:
                            tag.close()
                    finally:
                        raw.close()
                with builder.build() as item:
                    assert item.operation == 1


def test_fluent_builder_and_operation_reject_invalid_options_and_closed_handles() -> None:
    builder = fluent.SymmetricKeyBuilder()
    with builder:
        with pytest.raises(errors.KmipKitError):
            builder.name(None)  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.with_options(object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.add_attribute("description", object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.add_attribute_item(object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.add_vendor_attribute("vendor", "name", object())  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            builder.asynchronous_indicator(0x1_0000_0000)
        with pytest.raises(errors.KmipKitError):
            builder.execute(object())  # type: ignore[arg-type]
    with pytest.raises(RuntimeError, match="closed"):
        builder.build()

    operation = fluent.Operation(
        object(), "destroy", extensions.UniqueIdentifier.integer(-17)
    )
    assert operation.with_options(client.RequestOptions()) is operation
    with pytest.raises(errors.KmipKitError):
        operation.with_options(object())  # type: ignore[arg-type]
    record, owner = fluent._identifier(extensions.UniqueIdentifier.enumeration(9))
    assert owner is None
    assert record.enumeration_value == 9
    assert record.kind == int(extensions.UniqueIdentifierKind.ENUMERATION)


def test_fluent_abi_pointer_bridge_and_direct_entrypoint_argument_checks() -> None:
    foreign_pointer = fluent._base_ffi.cast("kmipkit_client_t *", 0x1234)
    bridged = fluent._client_abi_pointer("kmipkit_client_t *", foreign_pointer)
    assert int(fluent.ffi.cast("uintptr_t", bridged)) == 0x1234

    with ttlv.codec_limits_defaults() as limits:
        with pytest.raises(errors.KmipKitError):
            fluent._execute_direct(object(), object(), None, limits, None)  # type: ignore[arg-type]
        with pytest.raises(errors.KmipKitError):
            fluent._execute_direct_create(object(), object(), limits, None)  # type: ignore[arg-type]


def test_fluent_finalizer_swallows_shutdown_cleanup_errors(monkeypatch: pytest.MonkeyPatch) -> None:
    builder = fluent.SymmetricKeyBuilder()

    def fail_close() -> None:
        raise RuntimeError("native runtime already shut down")

    monkeypatch.setattr(builder, "close", fail_close)
    builder.__del__()


def test_legacy_extensions_remain_available() -> None:
    assert extensions.UniqueIdentifier.text_string("key-1").text == "key-1"


@pytest.mark.parametrize(
    ("builder_factory", "builder_type", "request_type", "operation"),
    [
        (
            "set_attribute",
            fluent.SetAttributeBuilder,
            attribute_operations.SetAttributeRequest,
            protocol_values.Operation.SET_ATTRIBUTE,
        ),
        (
            "add_attribute",
            fluent.AddAttributeBuilder,
            attribute_operations.AddAttributeRequest,
            protocol_values.Operation.ADD_ATTRIBUTE,
        ),
    ],
)
def test_request_namespace_exposes_typed_attribute_mutation_builder(
    builder_factory: str,
    builder_type: type[fluent.SetAttributeBuilder] | type[fluent.AddAttributeBuilder],
    request_type: type[
        attribute_operations.SetAttributeRequest | attribute_operations.AddAttributeRequest
    ],
    operation: protocol_values.Operation,
) -> None:
    request_builder = getattr(fluent.Request(None), builder_factory)
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.ttlv_value_text_string(b"managed by the fluent API", limits) as description:
            with request_builder() as builder:
                assert isinstance(builder, builder_type)
                with pytest.raises(errors.KmipKitError):
                    builder.validate()
                assert builder.unique_identifier("key-42") is builder
                assert builder.catalogued_attribute(
                    StandardAttribute.DESCRIPTION, description
                ) is builder
                assert builder.with_options(client.RequestOptions()) is builder
                assert builder.asynchronous_indicator(3) is builder
                builder.validate()
                item = builder.build()
                try:
                    assert item.operation == operation
                finally:
                    item.close()

                request = builder.build_request()
                try:
                    assert isinstance(request, request_type)
                    with request.to_ttlv_payload() as payload:
                        assert payload is not None
                    with request.to_batch_item() as request_item:
                        assert request_item.operation == operation
                finally:
                    request.close()

                with pytest.raises(errors.KmipKitError):
                    builder.unique_identifier(None)  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    builder.catalogued_attribute(None, description)  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    builder.catalogued_attribute(StandardAttribute.DESCRIPTION, object())  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    builder.attribute_item(object())  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    builder.vendor_attribute("vendor", "region", object())  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    builder.with_options(object())  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    builder.asynchronous_indicator(-1)
                with pytest.raises(errors.KmipKitError):
                    builder.execute(limits)

        with ttlv.raw_tag_from_raw(0x4200FC) as raw_tag:
            with ttlv.raw_tag_try_checked(raw_tag) as tag:
                with ttlv.ttlv_value_text_string(b"expert attribute", limits) as value:
                    with ttlv.ttlv_item_create(tag, value, limits) as item:
                        with request_builder() as builder:
                            builder.unique_identifier("key-42").attribute_item(item)
                            builder.validate()
                        with request_builder() as builder:
                            builder.unique_identifier("key-42").vendor_attribute(
                                "Vendor_Name.1", "region", item
                            )
                            builder.validate()
    with pytest.raises(RuntimeError, match="closed"):
        builder.validate()


@pytest.mark.parametrize("builder_factory", ["set_attribute", "add_attribute"])
def test_attribute_mutation_builder_executes_default_and_override_options(
    builder_factory: str, monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, int | None]] = []

    class FakeClient:
        def execute(self, batch: object, limits: object) -> str:
            calls.append(("default", getattr(batch, "asynchronous_indicator", None)))
            return "default result"

        def execute_with_options(self, batch: object, limits: object, options: object) -> str:
            calls.append(("options", getattr(batch, "asynchronous_indicator", None)))
            return "options result"

    class FakeBatch:
        asynchronous_indicator: int | None = None

        @classmethod
        def create(cls) -> FakeBatch:
            return cls()

        def add_item(self, item: object) -> None:
            item.close()  # type: ignore[attr-defined]

        def set_asynchronous_indicator(self, value: int) -> None:
            self.asynchronous_indicator = value

        def close(self) -> None:
            pass

    monkeypatch.setattr(fluent, "Client", FakeClient)
    monkeypatch.setattr(fluent, "Batch", FakeBatch)
    request = fluent.Request(FakeClient())
    request_builder = getattr(request, builder_factory)
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.ttlv_value_text_string(b"default exchange", limits) as value:
            with request_builder() as builder:
                builder.unique_identifier("key-42").catalogued_attribute(
                    StandardAttribute.DESCRIPTION, value,
                )
                assert builder.execute(limits) == "default result"
        with ttlv.ttlv_value_text_string(b"overridden exchange", limits) as value:
            with request_builder() as builder:
                builder.unique_identifier("key-42").catalogued_attribute(
                    StandardAttribute.DESCRIPTION, value,
                ).with_options(client.RequestOptions()).asynchronous_indicator(2)
                assert builder.execute(limits) == "options result"
    assert calls == [("default", None), ("options", 2)]


def test_single_item_exchange_helper_forwards_async_and_selected_options(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    calls: list[tuple[str, int | None, object | None]] = []

    class FakeClient:
        def execute(self, batch: object, limits: object) -> str:
            calls.append(("default", getattr(batch, "asynchronous_indicator", None), None))
            return "default result"

        def execute_with_options(
            self, batch: object, limits: object, options: object,
        ) -> str:
            calls.append(("options", getattr(batch, "asynchronous_indicator", None), options))
            return "options result"

    class FakeItem:
        def __enter__(self) -> FakeItem:
            return self

        def __exit__(self, *_arguments: object) -> None:
            self.close()

        def close(self) -> None:
            pass

    class FakeBatch:
        asynchronous_indicator: int | None = None

        @classmethod
        def create(cls) -> FakeBatch:
            return cls()

        def __enter__(self) -> FakeBatch:
            return self

        def __exit__(self, *_arguments: object) -> None:
            self.close()

        def add_item(self, _item: object) -> None:
            pass

        def set_asynchronous_indicator(self, value: int) -> None:
            self.asynchronous_indicator = value

        def close(self) -> None:
            pass

    monkeypatch.setattr(fluent, "Client", FakeClient)
    monkeypatch.setattr(fluent, "Batch", FakeBatch)
    configured_options = client.RequestOptions()
    override_options = client.RequestOptions()
    with ttlv.codec_limits_defaults() as limits:
        assert fluent._execute_single_item(
            FakeClient(), FakeItem, limits, configured_options, 2,
        ) == "options result"
        assert fluent._execute_single_item(
            FakeClient(), FakeItem, limits, configured_options, None, override_options,
        ) == "options result"

    assert calls == [
        ("options", 2, configured_options),
        ("options", None, override_options),
    ]
def test_add_attribute_builder_rejects_success_without_native_output(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native_library = fluent.lib

    class NativeLibraryStub:
        def __getattr__(self, name: str) -> object:
            return getattr(native_library, name)

        def kmipkit_add_attribute_builder_create(self, output: object) -> int:
            return 0

        def kmipkit_add_attribute_builder_build_request(
            self, builder: object, output: object
        ) -> int:
            return 0

    stub = NativeLibraryStub()
    monkeypatch.setattr(fluent, "lib", stub)
    with pytest.raises(errors.KmipKitError):
        fluent.AddAttributeBuilder()

    monkeypatch.setattr(fluent, "lib", native_library)
    with fluent.AddAttributeBuilder() as builder:
        monkeypatch.setattr(fluent, "lib", stub)
        with pytest.raises(errors.KmipKitError):
            builder.build_request()


def test_add_attribute_builder_finalizer_swallows_close_errors(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def fail_close(self: fluent.AddAttributeBuilder) -> None:
        raise RuntimeError("close failed")

    monkeypatch.setattr(fluent.AddAttributeBuilder, "close", fail_close)
    builder = object.__new__(fluent.AddAttributeBuilder)
    builder.__del__()
    del builder
