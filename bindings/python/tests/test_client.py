"""Contract tests for the shared Python client facade (KMIPKIT-0043 T024).

The client facade is a language adapter over the reviewed T076 C ABI. These
tests intentionally do not create a real network connection; Rust/C tests own
transport-wire verification.
"""

from __future__ import annotations

import importlib
from pathlib import Path
from typing import get_type_hints
from types import SimpleNamespace

import pytest

from kmipkit import client, errors, extensions, ttlv


def test_python_package_advertises_inline_typing_support() -> None:
    package = Path(client.__file__).parent
    assert (package / "py.typed").is_file()


def test_transport_contract_types_expose_all_supported_input_sources() -> None:
    pem = client.TransportInput.memory(b"certificate", client.CredentialEncoding.PEM)
    der_file = client.TransportInput.file(Path("trust.der"), client.CredentialEncoding.DER)

    assert pem.source == b"certificate"
    assert pem.encoding is client.CredentialEncoding.PEM
    assert der_file.source == Path("trust.der")
    assert der_file.encoding is client.CredentialEncoding.DER


def test_transport_configuration_keeps_endpoint_trust_and_timeout_choices_explicit() -> None:
    configuration = client.TransportConfig(
        endpoint=client.Endpoint.raw_tls("kmip.example.test", 5696),
        client_certificates=(
            client.TransportInput.memory(b"certificate", client.CredentialEncoding.PEM),
        ),
        client_private_key=client.TransportInput.memory(
            b"private key", client.CredentialEncoding.PEM
        ),
        trust=client.TrustSource.platform(),
        connect_timeout=client.TimeoutLimit.bounded(3, 250_000_000),
        max_request_bytes=1024 * 1024,
    )

    assert configuration.endpoint == client.Endpoint.raw_tls("kmip.example.test", 5696)
    assert configuration.trust.kind is client.TrustKind.PLATFORM
    assert configuration.connect_timeout == client.TimeoutLimit.bounded(3, 250_000_000)


def test_transport_inputs_reject_empty_or_invalid_values_without_echoing_them() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint.raw_tls("", 5696)

    with pytest.raises(errors.InvalidInputError) as caught:
        client.TransportInput.memory(b"private-material", client.CredentialEncoding.PEM)
        client.TransportConfig(
            endpoint=client.Endpoint.https("https://kmip.example.test"),
            client_certificates=(),
            client_private_key=client.TransportInput.memory(
                b"private-material", client.CredentialEncoding.PEM
            ),
            trust=client.TrustSource.platform(),
        )

    assert "private-material" not in str(caught.value)


def test_batch_and_client_are_context_managed_and_retain_ordered_typed_items() -> None:
    assert "return" in get_type_hints(client.Client.execute)
    assert "batch" in get_type_hints(client.Client.execute)
    assert "item" in get_type_hints(client.Batch.add_item)
    assert client.Client.__enter__
    assert client.Batch.__enter__
    assert client.BatchResult.__enter__
    assert get_type_hints(client.BatchItemResult)["pending_correlation_value"] == (
        bytes | None
    )
    assert extensions.client_batch_item_discover_versions
    assert ttlv.CodecLimits


def test_execution_error_exposes_sanitized_delivery_metadata() -> None:
    error = errors.ClientExecutionError(
        status=9,
        category=errors.ClientErrorCategory.TRANSPORT,
        cause=errors.ClientErrorCause.TIMEOUT,
        delivery=errors.DeliveryState.POSSIBLY_SENT,
    )

    assert error.status == 9
    assert error.delivery is errors.DeliveryState.POSSIBLY_SENT
    assert "private" not in str(error)
    assert "timeout" not in str(error).lower()


def test_malformed_presence_flag_is_rejected_as_invalid_input(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    native = client.ffi.new("uint8_t[]", 1)
    handle = client.ffi.cast("kmipkit_client_batch_item_response_t *", native)
    item = client._ClientBatchItemResponse(handle)

    def malformed_presence(
        _item: object, present: object, value: object
    ) -> int:
        present[0] = 2  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = 0  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_unique_batch_item_id=malformed_presence,
            kmipkit_client_batch_item_response_release=lambda _handle: None,
        ),
    )
    try:
        with pytest.raises(errors.InvalidInputError):
            client._response_span(
                item,
                client.lib.kmipkit_client_batch_item_response_unique_batch_item_id,
            )
    finally:
        item.close()


def _opaque(c_type: str) -> tuple[object, object]:
    storage = client.ffi.new("uint8_t[]", 1)
    return client.ffi.cast(f"{c_type} *", storage), storage


def _transport_configuration(**overrides: object) -> client.TransportConfig:
    values: dict[str, object] = {
        "endpoint": client.Endpoint.raw_tls(
            "kms.example.test", 5696, tls_server_name="kms.example.test"
        ),
        "client_certificates": (
            client.TransportInput.memory(b"client-cert", client.CredentialEncoding.PEM),
        ),
        "client_private_key": client.TransportInput.file(
            "client-key.pem", client.CredentialEncoding.PEM
        ),
        "trust": client.TrustSource.certificate_authorities(
            (client.TransportInput.memory(b"root-ca", client.CredentialEncoding.DER),)
        ),
        "revocation_lists": (
            client.TransportInput.file("revocations.der", client.CredentialEncoding.DER),
        ),
        "connect_timeout": client.TimeoutLimit.bounded(4, 123),
        "write_timeout": client.TimeoutLimit.unbounded(),
        "read_timeout": client.TimeoutLimit.default(),
        "total_timeout": client.TimeoutLimit.bounded(9),
        "max_request_bytes": 8192,
    }
    values.update(overrides)
    return client.TransportConfig(**values)  # type: ignore[arg-type]


def test_transport_record_serializes_memory_files_and_timeout_fields() -> None:
    record, owners = client._transport_record(_transport_configuration())
    assert record.struct_size == client.ffi.sizeof("kmipkit_transport_config_v1_t")
    assert record.abi_version == 1
    assert record.endpoint_kind == int(client.EndpointKind.RAW_TLS)
    assert record.trust_kind == int(client.TrustKind.CERTIFICATES)
    assert record.flags == 1
    assert client.ffi.string(record.endpoint.data, record.endpoint.length) == b"kms.example.test"
    assert record.endpoint_port == 5696
    assert record.client_certificate_count == 1
    assert record.client_certificates[0].source_kind == 1
    assert record.client_certificates[0].encoding == int(client.CredentialEncoding.PEM)
    assert client.ffi.string(
        record.client_certificates[0].data.data,
        record.client_certificates[0].data.length,
    ) == b"client-cert"
    assert record.client_private_key.source_kind == 2
    assert client.ffi.string(
        record.client_private_key.data.data, record.client_private_key.data.length
    ).endswith(b"client-key.pem")
    assert record.trust_certificate_count == 1
    assert record.revocation_list_count == 1
    assert record.connect_timeout.kind == int(client.TimeoutKind.BOUNDED)
    assert record.connect_timeout.seconds == 4
    assert record.connect_timeout.nanoseconds == 123
    assert record.write_timeout.kind == int(client.TimeoutKind.UNBOUNDED)
    assert record.read_timeout.kind == int(client.TimeoutKind.DEFAULT)
    assert record.total_timeout.seconds == 9
    assert record.max_request_bytes == 8192
    assert owners


def test_transport_record_supports_https_platform_trust_and_empty_optional_arrays() -> None:
    configuration = _transport_configuration(
        endpoint=client.Endpoint.https(
            "https://kms.example.test:8443", target_uri="/kmip"
        ),
        trust=client.TrustSource.platform(),
        revocation_lists=(),
    )
    record, _owners = client._transport_record(configuration)
    assert record.endpoint_kind == int(client.EndpointKind.HTTPS)
    assert record.trust_kind == int(client.TrustKind.PLATFORM)
    assert record.flags == 2
    assert record.endpoint_port == 0
    assert record.trust_certificates == client.ffi.NULL
    assert record.trust_certificate_count == 0
    assert record.revocation_lists == client.ffi.NULL
    assert record.revocation_list_count == 0
    assert client.ffi.string(record.target_uri.data, record.target_uri.length) == b"/kmip"


@pytest.mark.parametrize(
    ("factory", "arguments"),
    [
        (client.Endpoint.raw_tls, ("host", 0)),
        (client.Endpoint.raw_tls, ("host", 65536)),
        (client.Endpoint.raw_tls, ("host", True)),
        (client.Endpoint.https, ("",)),
        (client.TransportInput.memory, (b"", client.CredentialEncoding.PEM)),
        (client.TransportInput.memory, (b"x", 1)),
        (client.TransportInput.file, ("key", 1)),
        (client.TrustSource.certificate_authorities, ((),)),
        (client.TimeoutLimit.bounded, (-1,)),
        (client.TimeoutLimit.bounded, (1, 1_000_000_000)),
    ],
)
def test_public_configuration_factories_reject_invalid_inputs_without_echoing_secrets(
    factory: object, arguments: tuple[object, ...]
) -> None:
    with pytest.raises(errors.InvalidInputError):
        factory(*arguments)  # type: ignore[operator]


def test_transport_input_copies_mutable_memory_and_transport_config_normalizes_lists() -> None:
    mutable = bytearray(b"secret bytes")
    copied = client.TransportInput.memory(mutable, client.CredentialEncoding.DER)
    mutable[:] = b"changed data"
    assert copied.source == b"secret bytes"
    configuration = _transport_configuration(
        client_certificates=[copied], revocation_lists=[]
    )
    assert configuration.client_certificates == (copied,)
    assert configuration.revocation_lists == ()


def test_timeout_and_unsigned_validation_cover_boundary_values() -> None:
    assert client.TimeoutLimit.bounded(0, 1).nanoseconds == 1
    assert client.TimeoutLimit.unbounded().kind is client.TimeoutKind.UNBOUNDED
    assert client._unsigned(0, 8) == 0
    assert client._unsigned(255, 8) == 255
    for value in (True, -1, 256):
        with pytest.raises(errors.InvalidInputError):
            client._unsigned(value, 8)
    with pytest.raises(errors.InvalidInputError):
        client.TimeoutLimit(client.TimeoutKind.DEFAULT, 1)
    with pytest.raises(errors.InvalidInputError):
        client.TimeoutLimit(client.TimeoutKind.UNBOUNDED, nanoseconds=1)
    for nanoseconds in (1.5, True):
        with pytest.raises(errors.InvalidInputError):
            client.TimeoutLimit.bounded(1, nanoseconds)


def test_fixed_width_integer_validation_ignores_subclass_comparisons() -> None:
    class LyingInt(int):
        def __lt__(self, other: object) -> bool:
            return False

        def __gt__(self, other: object) -> bool:
            return False

    oversized = LyingInt(1 << 100)
    with pytest.raises(errors.InvalidInputError):
        client.TimeoutLimit.bounded(oversized)
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(client.EndpointKind.RAW_TLS, "kms.example.test", oversized)

    bounded = client.TimeoutLimit.bounded(LyingInt(3), LyingInt(5))
    endpoint = client.Endpoint(client.EndpointKind.RAW_TLS, "kms.example.test", LyingInt(5696))
    assert type(bounded.seconds) is int
    assert type(bounded.nanoseconds) is int
    assert type(endpoint.port) is int


def test_transport_config_rejects_oversized_lists_before_iteration() -> None:
    class OversizedList(list[client.TransportInput]):
        iterated = False

        def __len__(self) -> int:
            return 0

        def __iter__(self):  # type: ignore[no-untyped-def]
            self.iterated = True
            return super().__iter__()

    item = client.TransportInput.memory(b"certificate", client.CredentialEncoding.PEM)
    values = OversizedList([item] * 4_097)

    with pytest.raises(errors.ResourceLimitError):
        _transport_configuration(client_certificates=values)
    assert not values.iterated

    values.iterated = False
    with pytest.raises(errors.ResourceLimitError):
        client.TrustSource(client.TrustKind.CERTIFICATES, values)
    assert not values.iterated


def test_transport_input_rejects_unencodable_file_path_without_exposing_it() -> None:
    invalid_path = Path("\ud800")
    with pytest.raises(errors.InvalidInputError) as caught:
        client.TransportInput(invalid_path, client.CredentialEncoding.PEM)
    assert "\ud800" not in str(caught.value)


def test_batch_factory_and_setters_map_arguments_to_native_calls(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    batch_pointer, _storage = _opaque("kmipkit_client_batch_t")
    observed: list[tuple[str, tuple[object, ...]]] = []

    def create(output: object) -> int:
        output[0] = batch_pointer  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_create=create,
        kmipkit_client_batch_release=lambda _pointer: None,
        kmipkit_client_batch_set_request_time_stamp=lambda _batch, value: observed.append(("timestamp", (value,))) or 0,
        kmipkit_client_batch_set_order_option=lambda _batch, value: observed.append(("ordered", (value,))) or 0,
        kmipkit_client_batch_add_error_continuation_value=lambda _batch, value: observed.append(("continuation", (value,))) or 0,
        kmipkit_client_batch_set_asynchronous_indicator=lambda _batch, value: observed.append(("async", (value,))) or 0,
        kmipkit_client_batch_set_client_correlation_value=lambda _batch, value, length: observed.append(("correlation", (bytes(client.ffi.buffer(value, length)) if length else b"", length))) or 0,
    )
    monkeypatch.setattr(client, "lib", fake)
    batch = client.Batch.create()
    try:
        assert batch.set_request_timestamp(-(1 << 63)) is batch
        assert batch.set_ordered(True) is batch
        assert batch.add_error_continuation_value(0xFFFF_FFFF) is batch
        assert batch.set_asynchronous_indicator(3) is batch
        assert batch.set_client_correlation_value("héllo") is batch
        assert observed == [
            ("timestamp", (-(1 << 63),)),
            ("ordered", (1,)),
            ("continuation", (0xFFFF_FFFF,)),
            ("async", (3,)),
            ("correlation", ("héllo".encode(), len("héllo".encode()))),
        ]
        with pytest.raises(errors.InvalidInputError):
            batch.set_ordered(1)  # type: ignore[arg-type]
    finally:
        batch.close()


def test_batch_add_item_restores_ownership_when_native_does_not_consume(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    batch_item = extensions.client_batch_item_discover_versions()
    original_pointer = batch_item._pointer()
    fake = SimpleNamespace(
        kmipkit_client_batch_add_item=lambda _batch, _slot: 4,
        kmipkit_client_batch_release=lambda _pointer: None,
        kmipkit_client_batch_item_release=lambda _pointer: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    batch = client.Batch(batch_pointer)
    try:
        with pytest.raises(errors.InvalidInputError):
            batch.add_item(batch_item)
        assert not batch_item.closed
        assert batch_item._pointer() == original_pointer
    finally:
        batch.close()
        batch_item.close()


def test_batch_add_item_transfers_ownership_when_native_accepts_it(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    batch_item = extensions.client_batch_item_discover_versions()
    fake = SimpleNamespace(
        kmipkit_client_batch_add_item=lambda _batch, slot: slot.__setitem__(0, client.ffi.NULL) or 0,
        kmipkit_client_batch_release=lambda _pointer: None,
        kmipkit_client_batch_item_release=lambda _pointer: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    batch = client.Batch(batch_pointer)
    batch.add_item(batch_item)
    assert batch_item.closed
    batch.close()


def test_client_create_serializes_transport_and_returns_owned_client(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    extension_limits = extensions.default_extension_registry_limits()
    registry = extensions.create_client_extension_registry((), extension_limits)
    configuration = extensions.create_client_configuration(registry)
    transport_pointer, _transport_storage = _opaque("kmipkit_transport_config_t")
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    observed: list[int] = []

    def create_transport(record: object, output: object) -> int:
        observed.append(int(record.abi_version))  # type: ignore[attr-defined]
        output[0] = transport_pointer  # type: ignore[index]
        return 0

    def create_client(_configuration: object, slot: object, output: object) -> int:
        slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = client_pointer  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        kmipkit_transport_config_create_v1=create_transport,
        kmipkit_transport_config_release=lambda _pointer: None,
        kmipkit_client_create=create_client,
        kmipkit_client_release=lambda _pointer: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    created = client.Client.create(configuration, _transport_configuration())
    assert observed == [1]
    assert not created.closed
    created.close()
    configuration.close()


def test_client_create_releases_unconsumed_transport_after_client_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    limits = extensions.default_extension_registry_limits()
    registry = extensions.create_client_extension_registry((), limits)
    configuration = extensions.create_client_configuration(registry)
    transport_pointer, _storage = _opaque("kmipkit_transport_config_t")
    released: list[object] = []
    fake = SimpleNamespace(
        kmipkit_transport_config_create_v1=lambda _record, output: output.__setitem__(0, transport_pointer) or 0,
        kmipkit_transport_config_release=lambda pointer: released.append(pointer),
        kmipkit_client_create=lambda _configuration, _slot, _output: 4,
    )
    monkeypatch.setattr(client, "lib", fake)
    try:
        with pytest.raises(errors.InvalidInputError):
            client.Client.create(configuration, _transport_configuration())
        assert released == [transport_pointer]
    finally:
        configuration.close()


def test_client_create_rejects_null_success_handle_and_maps_transport_failure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    limits = extensions.default_extension_registry_limits()
    registry = extensions.create_client_extension_registry((), limits)
    configuration = extensions.create_client_configuration(registry)
    no_handle = SimpleNamespace(
        kmipkit_transport_config_create_v1=lambda _record, output: output.__setitem__(0, client.ffi.NULL) or 0,
        kmipkit_transport_config_release=lambda _pointer: None,
        kmipkit_client_create=lambda _configuration, _slot, _output: 0,
    )
    monkeypatch.setattr(client, "lib", no_handle)
    with pytest.raises(errors.KmipKitError):
        client.Client.create(configuration, _transport_configuration())
    failing = SimpleNamespace(
        kmipkit_transport_config_create_v1=lambda _record, _output: 6,
    )
    monkeypatch.setattr(client, "lib", failing)
    with pytest.raises(errors.ResourceLimitError):
        client.Client.create(configuration, _transport_configuration())
    configuration.close()


def test_batch_result_copies_metadata_unknown_outcome_and_optional_spans(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    unique_id = client.ffi.new("uint8_t[]", b"uid-1")
    pending = client.ffi.new("uint8_t[]", b"pending")
    released: list[str] = []

    def metadata(_item: object, out: object) -> int:
        out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v1_t")  # type: ignore[attr-defined]
        out.abi_version = 1  # type: ignore[attr-defined]
        out.outcome_kind = 91  # type: ignore[attr-defined]
        out.result_status = 2  # type: ignore[attr-defined]
        out.has_result_reason = 1  # type: ignore[attr-defined]
        out.result_reason = 7  # type: ignore[attr-defined]
        return 0

    def set_span(data: object, value: object, present: object) -> int:
        present[0] = 1  # type: ignore[index]
        value.data = data  # type: ignore[attr-defined]
        value.length = len(client.ffi.string(data))  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v1=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda item, present, value: set_span(unique_id, value, present),
        kmipkit_client_batch_item_response_pending_correlation_value=lambda item, present, value: set_span(pending, value, present),
    )
    monkeypatch.setattr(client, "lib", fake)
    result = client._batch_result(response_pointer)
    assert result.items == (
        client.BatchItemResult(91, 2, 7, b"uid-1", b"pending"),
    )
    result.close()
    assert released == ["item", "response"]


def test_batch_result_accepts_absent_spans_and_no_result_reason(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")

    def metadata(_item: object, out: object) -> int:
        out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v1_t")  # type: ignore[attr-defined]
        out.abi_version = 1  # type: ignore[attr-defined]
        out.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
        out.result_status = 0  # type: ignore[attr-defined]
        out.has_result_reason = 0  # type: ignore[attr-defined]
        return 0

    def absent(_item: object, present: object, value: object) -> int:
        present[0] = 0  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = 0  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: None,
        kmipkit_client_batch_item_response_release=lambda _ptr: None,
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v1=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=absent,
        kmipkit_client_batch_item_response_pending_correlation_value=absent,
    )
    monkeypatch.setattr(client, "lib", fake)
    result = client._batch_result(response_pointer)
    assert result.items[0].outcome is client.BatchOutcome.COMPLETED
    assert result.items[0].result_reason is None
    assert result.items[0].unique_batch_item_id is None
    assert result.items[0].pending_correlation_value is None
    result.close()


def test_batch_result_enforces_count_metadata_and_span_limits_and_releases_handles(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    released: list[str] = []
    common = {
        "kmipkit_client_batch_response_release": lambda _ptr: released.append("response"),
        "kmipkit_client_batch_item_response_release": lambda _ptr: released.append("item"),
    }
    over_limit = SimpleNamespace(
        **common,
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 4097) or 0,
    )
    monkeypatch.setattr(client, "lib", over_limit)
    with pytest.raises(errors.ResourceLimitError):
        client._batch_result(response_pointer)
    assert released == ["response"]

    def wrong_metadata(_item: object, out: object) -> int:
        out.struct_size = 1  # type: ignore[attr-defined]
        out.abi_version = 2  # type: ignore[attr-defined]
        return 0

    wrong_abi = SimpleNamespace(
        **common,
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v1=wrong_metadata,
    )
    monkeypatch.setattr(client, "lib", wrong_abi)
    with pytest.raises(errors.InvalidInputError):
        client._batch_result(response_pointer)
    assert released[-2:] == ["item", "response"]

    def valid_metadata(_item: object, out: object) -> int:
        out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v1_t")  # type: ignore[attr-defined]
        out.abi_version = 1  # type: ignore[attr-defined]
        out.outcome_kind = 1  # type: ignore[attr-defined]
        out.result_status = 0  # type: ignore[attr-defined]
        out.has_result_reason = 0  # type: ignore[attr-defined]
        return 0

    def too_large(_item: object, present: object, value: object) -> int:
        present[0] = 1  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = (16 * 1024 * 1024) + 1  # type: ignore[attr-defined]
        return 0

    oversized = SimpleNamespace(
        **common,
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v1=valid_metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=too_large,
    )
    monkeypatch.setattr(client, "lib", oversized)
    with pytest.raises(errors.ResourceLimitError):
        client._batch_result(response_pointer)
    assert released[-2:] == ["item", "response"]


def test_last_error_metadata_preserves_unknown_discriminators_and_fallback(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    def last_error(status: object, category: object, cause: object, delivery: object) -> int:
        status[0] = 9  # type: ignore[index]
        category[0] = 99  # type: ignore[index]
        cause[0] = 98  # type: ignore[index]
        delivery[0] = 97  # type: ignore[index]
        return 0

    monkeypatch.setattr(client, "lib", SimpleNamespace(kmipkit_last_error_get=last_error))
    assert client._read_last_error() == (9, 99, 98, 97)
    with pytest.raises(errors.ClientExecutionError) as caught:
        client._raise_client_status(11)
    assert caught.value.client_category == 99
    assert caught.value.cause == 98
    assert caught.value.delivery == 97
    assert client._known_enum(errors.DeliveryState, int(errors.DeliveryState.NONE)) is errors.DeliveryState.NONE
    assert client._known_enum(errors.DeliveryState, 88) == 88

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_last_error_get=lambda *_outputs: 5),
    )
    assert client._read_last_error() == (7, 5, 10, 0)


def test_client_execute_restores_batch_on_ffi_conversion_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = client.Client(client_pointer)
    batch = client.Batch(batch_pointer)
    limits = ttlv.codec_limits_defaults()
    fake = SimpleNamespace(
        kmipkit_client_execute=lambda *_args: (_ for _ in ()).throw(TypeError("secret")),
        kmipkit_client_release=lambda _ptr: None,
        kmipkit_client_batch_release=lambda _ptr: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    try:
        with pytest.raises(errors.InvalidInputError) as caught:
            handle.execute(batch, limits)
        assert "secret" not in str(caught.value)
        assert not batch.closed
        assert batch._pointer() == batch_pointer
    finally:
        limits.close()
        batch.close()
        handle.close()


def test_client_execute_maps_failure_delivery_and_restores_unconsumed_batch(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = client.Client(client_pointer)
    batch = client.Batch(batch_pointer)
    limits = ttlv.codec_limits_defaults()

    def last_error(status: object, category: object, cause: object, delivery: object) -> int:
        status[0] = 9  # type: ignore[index]
        category[0] = int(errors.ClientErrorCategory.TRANSPORT)  # type: ignore[index]
        cause[0] = int(errors.ClientErrorCause.TIMEOUT)  # type: ignore[index]
        delivery[0] = int(errors.DeliveryState.RESPONSE_STARTED)  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_execute=lambda _client, _batch, _limits, _output: 9,
        kmipkit_last_error_get=last_error,
        kmipkit_client_release=lambda _ptr: None,
        kmipkit_client_batch_release=lambda _ptr: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    try:
        with pytest.raises(errors.ClientExecutionError) as caught:
            handle.execute(batch, limits)
        assert caught.value.client_category is errors.ClientErrorCategory.TRANSPORT
        assert caught.value.cause is errors.ClientErrorCause.TIMEOUT
        assert caught.value.delivery is errors.DeliveryState.RESPONSE_STARTED
        assert not batch.closed
    finally:
        limits.close()
        batch.close()
        handle.close()


def test_client_execute_consumes_batch_and_returns_owned_empty_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    handle = client.Client(client_pointer)
    batch = client.Batch(batch_pointer)
    limits = ttlv.codec_limits_defaults()

    def execute(_client: object, batch_slot: object, _limits: object, output: object) -> int:
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute=execute,
            kmipkit_client_batch_response_item_count=lambda _response, count: count.__setitem__(0, 0) or 0,
            kmipkit_client_batch_response_release=lambda _ptr: None,
            kmipkit_client_release=lambda _ptr: None,
            kmipkit_client_batch_release=lambda _ptr: None,
        ),
    )
    try:
        result = handle.execute(batch, limits)
        assert batch.closed
        assert result.items == ()
        result.close()
    finally:
        limits.close()
        batch.close()
        handle.close()


def test_response_span_rejects_inconsistent_absence_and_null_nonempty_data(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    pointer, _storage = _opaque("kmipkit_client_batch_item_response_t")
    item = client._ClientBatchItemResponse(pointer)

    def inconsistent(_item: object, present: object, value: object) -> int:
        present[0] = 0  # type: ignore[index]
        value.data = client.ffi.cast("uint8_t *", client.ffi.new("uint8_t[]", b"x"))  # type: ignore[attr-defined]
        value.length = 1  # type: ignore[attr-defined]
        return 0

    def null_data(_item: object, present: object, value: object) -> int:
        present[0] = 1  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = 1  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_item_response_release=lambda _ptr: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    try:
        for function in (inconsistent, null_data):
            with pytest.raises(errors.InvalidInputError):
                client._response_span(item, function)
    finally:
        item.close()


def test_batch_response_item_at_rejects_null_handle_and_releases_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _storage = _opaque("kmipkit_client_batch_response_t")
    released: list[object] = []
    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda ptr: released.append(ptr),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, client.ffi.NULL) or 0,
    )
    monkeypatch.setattr(client, "lib", fake)
    with pytest.raises(errors.KmipKitError):
        client._batch_result(response_pointer)
    assert released == [response_pointer]


def test_memory_input_rejects_unsupported_released_and_oversized_buffers() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput.memory("secret", client.CredentialEncoding.PEM)  # type: ignore[arg-type]
    released = memoryview(b"released")
    released.release()
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput.memory(released, client.CredentialEncoding.PEM)
    with pytest.raises(errors.ResourceLimitError):
        client.TransportInput.memory(
            b"x" * ((16 * 1024 * 1024) + 1), client.CredentialEncoding.PEM
        )


def test_file_input_rejects_bad_encodings_and_oversized_path_text() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput.file("client-key", 1)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput.file(Path("\ud800"), client.CredentialEncoding.PEM)
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput.file(
            "x" * 4_097, client.CredentialEncoding.PEM
        )


def test_timeout_and_transport_config_reject_invalid_nested_types() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.TimeoutLimit(3)  # type: ignore[arg-type]
    valid = _transport_configuration()
    invalid_values = (
        {"endpoint": object()},
        {"client_private_key": object()},
        {"trust": object()},
        {"client_certificates": (object(),)},
        {"revocation_lists": (object(),)},
        {"connect_timeout": object()},
        {"max_request_bytes": 0},
    )
    for override in invalid_values:
        fields = {
            "endpoint": valid.endpoint,
            "client_certificates": valid.client_certificates,
            "client_private_key": valid.client_private_key,
            "trust": valid.trust,
            "revocation_lists": valid.revocation_lists,
            "connect_timeout": valid.connect_timeout,
            "write_timeout": valid.write_timeout,
            "read_timeout": valid.read_timeout,
            "total_timeout": valid.total_timeout,
            "max_request_bytes": valid.max_request_bytes,
        }
        fields.update(override)
        with pytest.raises(errors.InvalidInputError):
            client.TransportConfig(**fields)  # type: ignore[arg-type]


def test_text_and_endpoint_validation_cover_encoding_and_length_limits() -> None:
    assert client._nonempty_text("kms.example") == "kms.example"
    for invalid in (None, "", "a\x00b", "\ud800"):
        with pytest.raises(errors.InvalidInputError):
            client._nonempty_text(invalid)  # type: ignore[arg-type]
    with pytest.raises(errors.ResourceLimitError):
        client.Endpoint.https("x" * 4_097)


def test_batch_add_item_maps_cffi_conversion_error_and_releases_lock(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    batch_item = extensions.client_batch_item_discover_versions()
    original_pointer = batch_item._pointer()

    def reject_before_call(_batch: object, _slot: object) -> int:
        raise TypeError("do not echo this")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_add_item=reject_before_call,
            kmipkit_client_batch_release=lambda _ptr: None,
            kmipkit_client_batch_item_release=lambda _ptr: None,
        ),
    )
    batch = client.Batch(batch_pointer)
    try:
        with pytest.raises(errors.InvalidInputError) as caught:
            batch.add_item(batch_item)
        assert "do not echo" not in str(caught.value)
        assert batch_item._pointer() == original_pointer
    finally:
        batch.close()
        batch_item.close()


def test_batch_add_item_rejects_untyped_item() -> None:
    batch = client.Batch.create()
    try:
        with pytest.raises(errors.InvalidInputError):
            batch.add_item(object())  # type: ignore[arg-type]
    finally:
        batch.close()


def _extension_configuration() -> extensions.ClientConfiguration:
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    return extensions.create_client_configuration(registry)


def test_client_create_cleans_up_after_ffi_conversion_failure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    configuration = _extension_configuration()
    transport_pointer, _storage = _opaque("kmipkit_transport_config_t")
    released: list[object] = []

    def fail_conversion(*_arguments: object) -> int:
        raise TypeError("private input")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_transport_config_create_v1=lambda _record, output: output.__setitem__(0, transport_pointer) or 0,
            kmipkit_transport_config_release=lambda pointer: released.append(pointer),
            kmipkit_client_create=fail_conversion,
        ),
    )
    try:
        with pytest.raises(errors.InvalidInputError) as caught:
            client.Client.create(configuration, _transport_configuration())
        assert "private input" not in str(caught.value)
        assert released == [transport_pointer]
    finally:
        configuration.close()


def test_client_create_rejects_wrong_facade_types_and_null_client_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    with pytest.raises(errors.InvalidInputError):
        client.Client.create(object(), _transport_configuration())  # type: ignore[arg-type]
    configuration = _extension_configuration()
    with pytest.raises(errors.InvalidInputError):
        client.Client.create(configuration, object())  # type: ignore[arg-type]
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_transport_config_create_v1=lambda _record, output: output.__setitem__(0, client.ffi.cast("kmipkit_transport_config_t *", client.ffi.new("uint8_t[]", 1))) or 0,
            kmipkit_transport_config_release=lambda _ptr: None,
            kmipkit_client_create=lambda _configuration, slot, output: slot.__setitem__(0, client.ffi.NULL) or output.__setitem__(0, client.ffi.NULL) or 0,
        ),
    )
    try:
        with pytest.raises(errors.KmipKitError):
            client.Client.create(configuration, _transport_configuration())
    finally:
        configuration.close()


def test_client_execute_rejects_wrong_facade_types_and_null_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    released_client_pointers: list[object] = []
    released_batch_pointers: list[object] = []
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute=lambda _client, _slot, _limits, _output: 0,
            kmipkit_client_release=lambda pointer: released_client_pointers.append(pointer),
            kmipkit_client_batch_release=lambda pointer: released_batch_pointers.append(pointer),
        ),
    )
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = client.Client(client_pointer)
    limits = ttlv.codec_limits_defaults()
    wrong_limits_batch = client.Batch(batch_pointer)
    batch_pointer_2, _batch_storage_2 = _opaque("kmipkit_client_batch_t")
    batch = client.Batch(batch_pointer_2)
    try:
        with pytest.raises(errors.InvalidInputError):
            handle.execute(object(), limits)  # type: ignore[arg-type]
        with pytest.raises(errors.InvalidInputError):
            handle.execute(wrong_limits_batch, object())  # type: ignore[arg-type]
        assert not wrong_limits_batch.closed
        wrong_limits_batch.close()
        with pytest.raises(errors.KmipKitError):
            handle.execute(batch, limits)
        assert not batch.closed
        batch.close()
    finally:
        wrong_limits_batch.close()
        batch.close()
        limits.close()
        handle.close()
    assert released_batch_pointers == [batch_pointer, batch_pointer_2]
    assert released_client_pointers == [client_pointer]


def test_zero_client_status_does_not_read_last_error() -> None:
    client._raise_client_status(0)


def test_transport_record_handles_empty_optional_spans_from_absent_file_inputs() -> None:
    configuration = _transport_configuration(
        revocation_lists=(),
        trust=client.TrustSource.platform(),
    )
    record, _owners = client._transport_record(configuration)
    assert record.tls_server_name.length > 0
    assert record.target_uri.data == client.ffi.NULL
    assert record.target_uri.length == 0


def test_secret_bearing_client_records_redact_repr() -> None:
    key_marker = b"private-key-sentinel"
    certificate_marker = b"certificate-sentinel"
    path_marker = "private-path-sentinel"
    correlation_marker = b"pending-ticket-sentinel"
    key = client.TransportInput.memory(key_marker, client.CredentialEncoding.PEM)
    certificate = client.TransportInput.memory(
        certificate_marker, client.CredentialEncoding.PEM
    )
    path = client.TransportInput.file(path_marker, client.CredentialEncoding.PEM)
    trust = client.TrustSource.certificate_authorities((certificate,))
    configuration = _transport_configuration(
        client_certificates=(certificate,),
        client_private_key=key,
        trust=trust,
        revocation_lists=(path,),
    )
    pending = client.BatchItemResult(
        client.BatchOutcome.PENDING, 0, None, None, correlation_marker
    )

    rendered = " ".join(map(repr, (key, path, trust, configuration, pending)))
    for marker in (
        key_marker.decode(),
        certificate_marker.decode(),
        path_marker,
        correlation_marker.decode(),
    ):
        assert marker not in rendered


def test_transport_input_direct_constructor_enforces_limits_and_encoding() -> None:
    with pytest.raises(errors.ResourceLimitError):
        client.TransportInput(
            b"x" * ((16 * 1024 * 1024) + 1), client.CredentialEncoding.PEM
        )
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput(b"certificate", 1)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput("private-path", client.CredentialEncoding.PEM)  # type: ignore[arg-type]


def test_transport_config_rejects_excessive_item_counts_before_ffi_arrays() -> None:
    item = client.TransportInput.memory(b"certificate", client.CredentialEncoding.PEM)
    excessive = (item,) * 4_097

    with pytest.raises(errors.ResourceLimitError):
        _transport_configuration(client_certificates=excessive)
    with pytest.raises(errors.ResourceLimitError):
        _transport_configuration(
            trust=client.TrustSource.certificate_authorities(excessive)
        )
    with pytest.raises(errors.ResourceLimitError):
        _transport_configuration(revocation_lists=excessive)


def test_transport_config_rejects_aggregate_memory_before_ffi_copy() -> None:
    first = client.TransportInput.memory(
        b"a" * (9 * 1024 * 1024), client.CredentialEncoding.PEM
    )
    second = client.TransportInput.memory(
        b"b" * (8 * 1024 * 1024), client.CredentialEncoding.PEM
    )

    with pytest.raises(errors.ResourceLimitError):
        _transport_configuration(client_certificates=(first, second))


def test_batch_result_rejects_noncanonical_result_reason_presence_flag(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    released: list[str] = []

    def metadata(_item: object, out: object) -> int:
        out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v1_t")  # type: ignore[attr-defined]
        out.abi_version = 1  # type: ignore[attr-defined]
        out.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
        out.result_status = 1  # type: ignore[attr-defined]
        out.has_result_reason = 2  # type: ignore[attr-defined]
        out.result_reason = 7  # type: ignore[attr-defined]
        return 0

    def absent_span(_item: object, present: object, value: object) -> int:
        present[0] = 0  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = 0  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v1=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=absent_span,
        kmipkit_client_batch_item_response_pending_correlation_value=absent_span,
    )
    monkeypatch.setattr(client, "lib", fake)

    with pytest.raises(errors.InvalidInputError):
        client._batch_result(response_pointer)
    assert released == ["item", "response"]


def test_batch_setters_hold_native_call_lock_for_the_full_abi_call(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class TrackingLock:
        held = False

        def __enter__(self) -> TrackingLock:
            assert not self.held
            self.held = True
            return self

        def __exit__(self, *_exc: object) -> None:
            assert self.held
            self.held = False

    lock = TrackingLock()
    monkeypatch.setattr(client, "_NATIVE_CALL_LOCK", lock)
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    batch = client.Batch(batch_pointer)
    observed: list[bool] = []

    def check_lock(*_args: object) -> int:
        observed.append(lock.held)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_set_request_time_stamp=check_lock,
            kmipkit_client_batch_set_order_option=check_lock,
            kmipkit_client_batch_add_error_continuation_value=check_lock,
            kmipkit_client_batch_set_asynchronous_indicator=check_lock,
            kmipkit_client_batch_set_client_correlation_value=check_lock,
        ),
    )

    batch.set_request_timestamp(1)
    batch.set_ordered(True)
    batch.add_error_continuation_value(0)
    batch.set_asynchronous_indicator(1)
    batch.set_client_correlation_value(b"correlation")

    assert observed == [True] * 5


def test_endpoint_direct_constructor_enforces_text_shape_and_repr_redaction() -> None:
    class LyingText(str):
        def encode(self, *_args: object, **_kwargs: object) -> bytes:
            return b"x"

    with pytest.raises(errors.ResourceLimitError):
        client.Endpoint(client.EndpointKind.HTTPS, "x" * 4_097)
    with pytest.raises(errors.ResourceLimitError):
        client.Endpoint(client.EndpointKind.HTTPS, LyingText("x" * 4_097))
    with pytest.raises(errors.ResourceLimitError):
        client.Endpoint(
            client.EndpointKind.RAW_TLS,
            "kms.example.test",
            5696,
            tls_server_name="x" * 4_097,
        )
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(client.EndpointKind.HTTPS, "bad\x00uri")
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(client.EndpointKind.HTTPS, "\ud800")
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(client.EndpointKind.RAW_TLS, "kms.example.test", 0)
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(client.EndpointKind.HTTPS, "https://kms.example.test", False)
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(client.EndpointKind.HTTPS, "https://kms.example.test", 443)
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint("https", "https://kms.example.test")  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(client.EndpointKind.HTTPS, None)  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(
            client.EndpointKind.RAW_TLS,
            "kms.example.test",
            5696,
            tls_server_name=42,  # type: ignore[arg-type]
        )

    secret_uri = "https://user:uri-password-sentinel@kms.example.test/"
    endpoint = client.Endpoint.https(secret_uri)
    assert "uri-password-sentinel" not in repr(endpoint)


def test_endpoint_factories_and_transport_config_reject_validation_overrides() -> None:
    class UnvalidatedEndpoint(client.Endpoint):
        def __post_init__(self) -> None:
            pass

    with pytest.raises(errors.InvalidInputError):
        UnvalidatedEndpoint.raw_tls("", 0)
    with pytest.raises(errors.InvalidInputError):
        UnvalidatedEndpoint.https("")

    bypassed = UnvalidatedEndpoint(client.EndpointKind.RAW_TLS, "", 0)
    with pytest.raises(errors.InvalidInputError):
        _transport_configuration(endpoint=bypassed)


def test_client_facade_revalidates_transport_config_subclasses_before_ffi(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class UnvalidatedEndpoint(client.Endpoint):
        def __post_init__(self) -> None:
            pass

    class UnvalidatedTransportConfig(client.TransportConfig):
        def __post_init__(self) -> None:
            pass

    valid = _transport_configuration()
    invalid_endpoint = UnvalidatedEndpoint(client.EndpointKind.RAW_TLS, "", 0)
    transport = UnvalidatedTransportConfig(
        endpoint=invalid_endpoint,
        client_certificates=valid.client_certificates,
        client_private_key=valid.client_private_key,
        trust=valid.trust,
        revocation_lists=valid.revocation_lists,
        connect_timeout=valid.connect_timeout,
        write_timeout=valid.write_timeout,
        read_timeout=valid.read_timeout,
        total_timeout=valid.total_timeout,
        max_request_bytes=valid.max_request_bytes,
    )
    configuration = _extension_configuration()
    ffi_records: list[tuple[int, int]] = []

    def create_transport(record: object, _output: object) -> int:
        ffi_records.append(
            (int(record.endpoint.length), int(record.endpoint_port))  # type: ignore[attr-defined]
        )
        return 4

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_transport_config_create_v1=create_transport),
    )
    try:
        with pytest.raises(errors.InvalidInputError):
            client.Client.create(configuration, transport)
        assert ffi_records == []
    finally:
        configuration.close()


def test_transport_input_measures_real_buffer_size_for_bytes_subclasses() -> None:
    class LyingBytes(bytes):
        def __len__(self) -> int:
            return 1

    oversized = LyingBytes(b"x" * (16 * 1024 * 1024 + 1))

    with pytest.raises(errors.ResourceLimitError):
        client.TransportInput(oversized, client.CredentialEncoding.PEM)


def test_transport_record_uses_real_bytes_subclass_buffer_length() -> None:
    class LyingBytes(bytes):
        def __len__(self) -> int:
            return 1

    bounded = LyingBytes(b"small")
    configuration = _transport_configuration(
        client_private_key=client.TransportInput(
            bounded, client.CredentialEncoding.PEM
        )
    )
    record, _owners = client._transport_record(configuration)
    assert record.client_private_key.data.length == 5


def test_transport_path_uses_base_utf8_encoding_for_subclasses() -> None:
    class LyingText(str):
        def encode(self, *_args: object, **_kwargs: object) -> bytes:
            return b"x"

    concrete_path = type(Path("ignored"))

    class LyingPath(concrete_path):
        def __str__(self) -> str:
            return LyingText("x" * 4_097)

    path = LyingPath("ignored")
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput(path, client.CredentialEncoding.PEM)


def test_transport_input_snapshots_a_dynamic_bytes_subclass_buffer() -> None:
    class ChangingBuffer(bytes):
        def __new__(cls) -> ChangingBuffer:
            value = super().__new__(cls, b"underlying")
            value.buffer_reads = 0
            return value

        def __buffer__(self, _flags: int) -> memoryview:
            self.buffer_reads += 1
            data = b"trusted" if self.buffer_reads == 1 else b"attacker-expanded"
            return memoryview(data)

    dynamic = ChangingBuffer()
    item = client.TransportInput(dynamic, client.CredentialEncoding.PEM)

    assert type(item.source) is bytes
    assert item.source == b"trusted"
    assert dynamic.buffer_reads == 1

    configuration = _transport_configuration(client_private_key=item)
    record, _owners = client._transport_record(configuration)
    span = record.client_private_key.data
    assert span.length == 7
    assert client.ffi.buffer(span.data, span.length)[:] == b"trusted"


def test_transport_input_snapshots_path_text_before_marshalling() -> None:
    concrete_path = type(Path("ignored"))

    class ChangingPath(concrete_path):
        calls = 0

        def __str__(self) -> str:
            type(self).calls += 1
            return "safe.pem" if type(self).calls == 1 else "bad\x00path"

    ChangingPath.calls = 0
    item = client.TransportInput(ChangingPath("ignored"), client.CredentialEncoding.PEM)

    assert type(item.source) is type(Path("safe.pem"))
    assert str(item.source) == "safe.pem"

    configuration = _transport_configuration(client_private_key=item)
    record, _owners = client._transport_record(configuration)
    span = record.client_private_key.data
    assert client.ffi.buffer(span.data, span.length)[:] == b"safe.pem"


@pytest.mark.parametrize(
    "arguments",
    [
        (client.EndpointKind.RAW_TLS, "kms.example.test", 5696, None, "/not-allowed"),
        (client.EndpointKind.HTTPS, "https://kms.example.test", 0, "server", None),
        (99, "https://kms.example.test", 0, None, None),
    ],
)
def test_endpoint_rejects_incompatible_kind_and_endpoint_fields(
    arguments: tuple[object, ...],
) -> None:
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(*arguments)  # type: ignore[arg-type]


@pytest.mark.parametrize(
    "path",
    [object(), "nul\x00path.pem", "surrogate\ud800.pem", "p" * 4_097],
)
def test_transport_file_factory_rejects_invalid_or_oversized_paths(path: object) -> None:
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput.file(path, client.CredentialEncoding.PEM)  # type: ignore[arg-type]


@pytest.mark.parametrize(
    "path",
    [Path("nul\x00path.pem"), Path("surrogate\ud800.pem"), Path("p" * 4_097)],
)
def test_transport_input_constructor_rejects_nul_utf8_and_oversized_paths(
    path: Path,
) -> None:
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput(path, client.CredentialEncoding.PEM)


@pytest.mark.parametrize("value", [b"", bytearray(b"secret"), memoryview(b"secret")])
def test_transport_input_constructor_rejects_empty_or_noncanonical_buffers(
    value: bytes | bytearray | memoryview,
) -> None:
    with pytest.raises(errors.InvalidInputError):
        client.TransportInput(value, client.CredentialEncoding.PEM)  # type: ignore[arg-type]


def test_trust_and_transport_groups_reject_invalid_members_and_group_shapes() -> None:
    certificate = client.TransportInput.memory(b"ca", client.CredentialEncoding.PEM)
    with pytest.raises(errors.InvalidInputError):
        client.TrustSource(client.TrustKind.PLATFORM, (certificate,))
    with pytest.raises(errors.InvalidInputError):
        client.TrustSource(client.TrustKind.CERTIFICATES, ())
    with pytest.raises(errors.InvalidInputError):
        client.TrustSource(client.TrustKind.PLATFORM, object())  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        client.TrustSource(client.TrustKind.CERTIFICATES, (object(),))  # type: ignore[arg-type]
    with pytest.raises(errors.InvalidInputError):
        _transport_configuration(client_certificates=object())
    with pytest.raises(errors.InvalidInputError):
        _transport_configuration(client_certificates=(object(),))


def test_transport_group_count_uses_the_real_tuple_subclass_length() -> None:
    item = client.TransportInput.memory(b"certificate", client.CredentialEncoding.PEM)

    class LyingTuple(tuple[client.TransportInput, ...]):
        def __len__(self) -> int:
            return 0

    values = LyingTuple((item,) * 4_097)
    with pytest.raises(errors.ResourceLimitError):
        _transport_configuration(client_certificates=values)


def test_transport_config_canonicalizes_a_valid_tuple_subclass() -> None:
    item = client.TransportInput.memory(b"certificate", client.CredentialEncoding.PEM)

    class TupleSubclass(tuple[client.TransportInput, ...]):
        pass

    values = TupleSubclass((item,))
    configuration = _transport_configuration(client_certificates=values)
    assert type(configuration.client_certificates) is tuple
    assert configuration.client_certificates == (item,)


def test_transport_input_rejects_buffer_exporter_failure() -> None:
    class BrokenBuffer(bytes):
        def __new__(cls) -> BrokenBuffer:
            return super().__new__(cls, b"material")

        def __buffer__(self, _flags: int) -> memoryview:
            raise BufferError("buffer unavailable")

    with pytest.raises(errors.InvalidInputError):
        client.TransportInput(BrokenBuffer(), client.CredentialEncoding.PEM)


def _unvalidated_transport_configuration(**overrides: object) -> client.TransportConfig:
    class SkipsPostInit(client.TransportConfig):
        def __post_init__(self) -> None:
            pass

    valid = _transport_configuration()
    values: dict[str, object] = {
        "endpoint": valid.endpoint,
        "client_certificates": valid.client_certificates,
        "client_private_key": valid.client_private_key,
        "trust": valid.trust,
        "revocation_lists": valid.revocation_lists,
        "connect_timeout": valid.connect_timeout,
        "write_timeout": valid.write_timeout,
        "read_timeout": valid.read_timeout,
        "total_timeout": valid.total_timeout,
        "max_request_bytes": valid.max_request_bytes,
    }
    values.update(overrides)
    return SkipsPostInit(**values)  # type: ignore[arg-type]


@pytest.mark.parametrize(
    "field,value",
    [
        ("endpoint", object()),
        ("client_certificates", object()),
        ("client_private_key", object()),
        ("trust", object()),
        ("connect_timeout", object()),
    ],
)
def test_client_facade_rejects_unvalidated_config_subclass_groups_before_ffi(
    monkeypatch: pytest.MonkeyPatch, field: str, value: object
) -> None:
    configuration = _extension_configuration()
    transport = _unvalidated_transport_configuration(**{field: value})
    calls: list[object] = []

    def create_transport(record: object, _output: object) -> int:
        calls.append(record)
        return 4

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(kmipkit_transport_config_create_v1=create_transport),
    )
    try:
        with pytest.raises(errors.InvalidInputError):
            client.Client.create(configuration, transport)
        assert calls == []
    finally:
        configuration.close()


def test_client_facade_reconstructs_timeout_and_trust_subclasses_before_ffi(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class UnvalidatedTrust(client.TrustSource):
        def __post_init__(self) -> None:
            pass

    class UnvalidatedTimeout(client.TimeoutLimit):
        def __post_init__(self) -> None:
            pass

    certificate = client.TransportInput.memory(b"ca", client.CredentialEncoding.PEM)
    invalid_trust = UnvalidatedTrust(client.TrustKind.PLATFORM, (certificate,))
    invalid_timeout = UnvalidatedTimeout(client.TimeoutKind.DEFAULT, 1, 0)
    configuration = _extension_configuration()
    calls: list[object] = []
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_transport_config_create_v1=lambda record, _output: calls.append(record) or 4
        ),
    )

    try:
        for transport in (
            _unvalidated_transport_configuration(trust=invalid_trust),
            _unvalidated_transport_configuration(connect_timeout=invalid_timeout),
        ):
            with pytest.raises(errors.InvalidInputError):
                client.Client.create(configuration, transport)
        assert calls == []
    finally:
        configuration.close()


def test_private_python_client_loader_redacts_native_path_failure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from cffi import FFI
    from kmipkit import _client_ffi

    path_marker = "private-native-path-sentinel"

    def fail_dlopen(_ffi: object, path: str) -> object:
        assert path == path_marker
        raise OSError(path_marker)

    with monkeypatch.context() as patch:
        patch.setenv("KMIPKIT_NATIVE_LIBRARY", path_marker)
        patch.setattr(FFI, "dlopen", fail_dlopen)
        with pytest.raises(ImportError, match="KMIPKit native client ABI is unavailable") as caught:
            importlib.reload(_client_ffi)

    assert path_marker not in str(caught.value)
    importlib.reload(_client_ffi)
