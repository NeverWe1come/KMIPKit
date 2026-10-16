"""Contract tests for the shared Python client facade (KMIPKIT-0043 T024).

The client facade is a language adapter over the reviewed T076 C ABI. These
tests intentionally do not create a real network connection; Rust/C tests own
transport-wire verification.
"""

from __future__ import annotations

import gc
import importlib
import json
import weakref
from dataclasses import replace
from pathlib import Path
from types import SimpleNamespace
from typing import get_type_hints

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._handles import _wrap_owned_handle


def test_python_package_advertises_inline_typing_support() -> None:
    package = Path(client.__file__).parent
    assert (package / "py.typed").is_file()


def test_errors_module_does_not_export_its_intenum_implementation_base() -> None:
    assert not hasattr(errors, "IntEnum")


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


def test_transport_config_rejects_file_backed_revocation_lists_without_echoing_path() -> None:
    path = Path("revocation-list-not-read.der")

    with pytest.raises(errors.InvalidInputError) as caught:
        _transport_configuration(
            revocation_lists=(
                client.TransportInput.file(path, client.CredentialEncoding.DER),
            )
        )

    assert str(path) not in str(caught.value)


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


def test_public_native_handle_wrappers_reject_direct_cffi_handle_adoption() -> None:
    pointer = client.ffi.cast("void *", 1)
    wrappers = (
        ("client.Batch", lambda: client.Batch(pointer)),
        ("client.BatchResult", lambda: client.BatchResult(pointer, ())),
        ("client.Client", lambda: client.Client(pointer)),
        ("ttlv.RawTag", lambda: ttlv.RawTag(pointer)),
        ("ttlv.Tag", lambda: ttlv.Tag(pointer)),
        ("ttlv.CodecLimits", lambda: ttlv.CodecLimits(pointer)),
        ("ttlv.Value", lambda: ttlv.Value(pointer)),
        ("ttlv.Structure", lambda: ttlv.Structure(pointer)),
        ("ttlv.TtlvItem", lambda: ttlv.TtlvItem(pointer)),
        ("ttlv.TtlvStructureView", lambda: ttlv.TtlvStructureView(pointer)),
        ("ttlv.TtlvItemView", lambda: ttlv.TtlvItemView(pointer)),
        ("ttlv.TtlvValueView", lambda: ttlv.TtlvValueView(pointer)),
        ("extensions.ExtensionIdentity", lambda: extensions.ExtensionIdentity(pointer, None, None, None)),
        ("extensions.Compatibility", lambda: extensions.Compatibility(pointer)),
        ("extensions.TtlvPath", lambda: extensions.TtlvPath(pointer, ())),
        (
            "extensions.Discriminator",
            lambda: extensions.Discriminator(
                pointer, SimpleNamespace(tags=()), ttlv.ItemType.Integer, 0
            ),
        ),
        ("extensions.ExtensionSchema", lambda: extensions.ExtensionSchema(pointer)),
        ("extensions.ExtensionChildRule", lambda: extensions.ExtensionChildRule(pointer)),
        (
            "extensions.ExtensionOrderConstraint",
            lambda: extensions.ExtensionOrderConstraint(pointer),
        ),
        ("extensions.ExtensionInformation", lambda: extensions.ExtensionInformation(pointer, None)),
        (
            "extensions.ExtensionDefinition",
            lambda: extensions.ExtensionDefinition(pointer, object()),
        ),
        (
            "extensions.ClientExtensionRegistry",
            lambda: extensions.ClientExtensionRegistry(pointer, (), object()),
        ),
        (
            "extensions.ClientConfiguration",
            lambda: extensions.ClientConfiguration(pointer, (), object()),
        ),
        (
            "extensions.ValidatedExtensionValue",
            lambda: extensions.ValidatedExtensionValue(pointer),
        ),
        (
            "extensions.RegisteredExtensionValue",
            lambda: extensions.RegisteredExtensionValue(pointer, object()),
        ),
        (
            "extensions.ClientRequestMessageExtension",
            lambda: extensions.ClientRequestMessageExtension(pointer),
        ),
        ("extensions.ClientBatchItem", lambda: extensions.ClientBatchItem(pointer)),
        ("extensions.RegisterRequest", lambda: extensions.RegisterRequest(pointer)),
        (
            "extensions.ExtensionRecognition",
            lambda: extensions.ExtensionRecognition(pointer, object(), "vendor", object()),
        ),
    )

    for path, construct in wrappers:
        candidate = None
        try:
            with pytest.raises(errors.InvalidInputError):
                candidate = construct()
        finally:
            if candidate is not None:
                object.__setattr__(candidate, "_NativeHandle__handle", client.ffi.NULL)

    with pytest.raises(errors.InvalidInputError):
        _wrap_owned_handle(client.Batch, client.ffi.NULL)
    with pytest.raises(errors.InvalidInputError):
        client.Batch(pointer, _token=object())


def test_live_batch_handle_cannot_be_adopted_by_a_second_wrapper() -> None:
    original = client.Batch.create()
    duplicate = None
    pointer = original._pointer()
    try:
        with pytest.raises(errors.InvalidInputError):
            duplicate = client.Batch(pointer)
        assert original._pointer() == pointer
    finally:
        if duplicate is not None:
            object.__setattr__(duplicate, "_NativeHandle__handle", client.ffi.NULL)
        original.close()


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
    item = _wrap_owned_handle(client._ClientBatchItemResponse, handle)

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


def _no_supported_versions(
    _item: object, present: object, _versions: object, _capacity: int, count: object
) -> int:
    present[0] = 0  # type: ignore[index]
    count[0] = 0  # type: ignore[index]
    return 0


def _transport_configuration(**overrides: object) -> client.TransportConfig:
    values: dict[str, object] = {
        "endpoint": client.Endpoint.raw_tls("kms.example.test", 5696),
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
            client.TransportInput.memory(b"revocation-list", client.CredentialEncoding.DER),
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
    assert record.flags == 0
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


@pytest.mark.parametrize(
    ("source_kind", "encoding"),
    [
        ("memory", client.CredentialEncoding.PEM),
        ("memory", client.CredentialEncoding.DER),
        ("file", client.CredentialEncoding.PEM),
        ("file", client.CredentialEncoding.DER),
    ],
)
def test_transport_record_forwards_identity_and_trust_source_modes(
    tmp_path: Path, source_kind: str, encoding: client.CredentialEncoding
) -> None:
    def value(name: str) -> client.TransportInput:
        if source_kind == "memory":
            return client.TransportInput.memory(name.encode(), encoding)
        return client.TransportInput.file(tmp_path / name, encoding)

    certificate = value("client-cert")
    private_key = value("client-key")
    trust_certificate = value("trust-cert")
    crl = client.TransportInput.memory(b"crl-bytes", encoding)
    configuration = _transport_configuration(
        client_certificates=(certificate,),
        client_private_key=private_key,
        trust=client.TrustSource.certificate_authorities((trust_certificate,)),
        revocation_lists=(crl,),
    )

    record, owners = client._transport_record(configuration)
    expected_source = 1 if source_kind == "memory" else 2
    for input_record in (
        record.client_certificates[0],
        record.client_private_key,
        record.trust_certificates[0],
    ):
        assert input_record.source_kind == expected_source
        assert input_record.encoding == int(encoding)
    assert record.revocation_lists[0].source_kind == 1
    assert record.revocation_lists[0].encoding == int(encoding)
    if source_kind == "file":
        assert client.ffi.string(
            record.trust_certificates[0].data.data,
            record.trust_certificates[0].data.length,
        ).endswith(b"trust-cert")
    assert owners


def test_https_tls_server_name_is_forwarded_separately_from_authority() -> None:
    configuration = _transport_configuration(
        endpoint=client.Endpoint.https(
            "https://192.0.2.10:8443",
            tls_server_name="kms.example.test",
            target_uri="/kmip",
        ),
    )
    record, _owners = client._transport_record(configuration)

    assert client.ffi.string(record.endpoint.data, record.endpoint.length) == (
        b"https://192.0.2.10:8443"
    )
    assert client.ffi.string(
        record.tls_server_name.data, record.tls_server_name.length
    ) == b"kms.example.test"
    assert client.ffi.string(record.target_uri.data, record.target_uri.length) == b"/kmip"


def test_https_tls_server_name_override_rejects_malformed_authority() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint.https(
            "https://[invalid", tls_server_name="kms.example.test"
        )


def test_raw_tls_ip_accepts_an_explicit_tls_server_name() -> None:
    endpoint = client.Endpoint.raw_tls(
        "192.0.2.10", 5696, tls_server_name="kms.example.test"
    )

    assert endpoint.address == "192.0.2.10"
    assert endpoint.tls_server_name == "kms.example.test"


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


def test_ttlv_fixed_width_integer_inputs_reject_boolean_values() -> None:
    with pytest.raises(errors.InvalidInputError):
        ttlv.ttlv_value_integer(True)

    information = extensions.create_extension_information("typed-extension")
    try:
        with pytest.raises(errors.InvalidInputError):
            extensions.with_tag(information, True)
        assert not information.closed
    finally:
        information.close()


def test_ttlv_fixed_width_integer_inputs_reject_out_of_range_values() -> None:
    for constructor, value in (
        (ttlv.ttlv_value_integer, 1 << 31),
        (ttlv.ttlv_value_long_integer, 1 << 63),
        (ttlv.ttlv_value_enumeration, 1 << 32),
        (ttlv.ttlv_value_date_time, -(1 << 63) - 1),
        (ttlv.ttlv_value_interval, 1 << 32),
        (ttlv.ttlv_value_date_time_extended, 1 << 63),
    ):
        with pytest.raises(errors.InvalidInputError):
            constructor(value)

    information = extensions.create_extension_information("typed-extension")
    try:
        with pytest.raises(errors.InvalidInputError):
            extensions.with_tag(information, 1 << 32)
        assert not information.closed
    finally:
        information.close()


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
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
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
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    batch.add_item(batch_item)
    assert batch_item.closed
    batch.close()


def test_typed_batch_item_unique_id_builder_preserves_bytes_and_transfers_ownership(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    original_storage = extensions.ffi.new("uint8_t[]", 1)
    original = extensions.ffi.cast("kmipkit_client_batch_item_t *", original_storage)
    result_storage = extensions.ffi.new("uint8_t[]", 1)
    result = extensions.ffi.cast("kmipkit_client_batch_item_t *", result_storage)
    observed: list[bytes] = []

    def create(output: object) -> int:
        output[0] = original  # type: ignore[index]
        return 0

    def with_unique_id(
        _item: object, data: object, length: int, output: object
    ) -> int:
        observed.append(bytes(extensions.ffi.buffer(data, length)))  # type: ignore[arg-type]
        output[0] = result  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        extensions,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_discover_versions=create,
            kmipkit_client_batch_item_with_unique_batch_item_id=with_unique_id,
            kmipkit_client_batch_item_release=lambda _pointer: None,
        ),
    )
    item = extensions.client_batch_item_discover_versions()
    identified = extensions.with_unique_batch_item_id(item, b"\x00\xffid")
    assert item.closed
    assert identified._pointer() == result
    assert observed == [b"\x00\xffid"]
    identified.close()


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


@pytest.mark.parametrize(
    ("source_kind", "encoding"),
    [
        ("memory", client.CredentialEncoding.PEM),
        ("memory", client.CredentialEncoding.DER),
        ("file", client.CredentialEncoding.PEM),
        ("file", client.CredentialEncoding.DER),
    ],
)
def test_client_create_forwards_identity_and_trust_modes_to_native_constructor(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    source_kind: str,
    encoding: client.CredentialEncoding,
) -> None:
    extension_limits = extensions.default_extension_registry_limits()
    registry = extensions.create_client_extension_registry((), extension_limits)
    extension_configuration = extensions.create_client_configuration(registry)
    transport_pointer, _transport_storage = _opaque("kmipkit_transport_config_t")
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    observed: list[object] = []

    def value(name: str) -> client.TransportInput:
        if source_kind == "memory":
            return client.TransportInput.memory(name.encode(), encoding)
        return client.TransportInput.file(tmp_path / name, encoding)

    certificate = value("client-cert")
    private_key = value("client-key")
    trust_certificate = value("trust-cert")
    revocation_list = client.TransportInput.memory(b"crl-bytes", encoding)
    transport_configuration = _transport_configuration(
        client_certificates=(certificate,),
        client_private_key=private_key,
        trust=client.TrustSource.certificate_authorities((trust_certificate,)),
        revocation_lists=(revocation_list,),
    )

    def input_mode(value: object) -> tuple[int, int, bytes]:
        source = value.data  # type: ignore[attr-defined]
        data = client.ffi.string(source.data, int(source.length))
        return (
            int(value.source_kind),  # type: ignore[attr-defined]
            int(value.encoding),  # type: ignore[attr-defined]
            data,
        )

    def create_transport(record: object, output: object) -> int:
        observed.append(
            {
                "abi_version": int(record.abi_version),  # type: ignore[attr-defined]
                "client_certificates": tuple(
                    input_mode(record.client_certificates[index])  # type: ignore[attr-defined]
                    for index in range(int(record.client_certificate_count))  # type: ignore[attr-defined]
                ),
                "client_private_key": input_mode(record.client_private_key),  # type: ignore[attr-defined]
                "trust_certificates": tuple(
                    input_mode(record.trust_certificates[index])  # type: ignore[attr-defined]
                    for index in range(int(record.trust_certificate_count))  # type: ignore[attr-defined]
                ),
                "revocation_lists": tuple(
                    input_mode(record.revocation_lists[index])  # type: ignore[attr-defined]
                    for index in range(int(record.revocation_list_count))  # type: ignore[attr-defined]
                ),
            }
        )
        output[0] = transport_pointer  # type: ignore[index]
        return 0

    def create_client(_configuration: object, slot: object, output: object) -> int:
        slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = client_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_transport_config_create_v1=create_transport,
            kmipkit_transport_config_release=lambda _pointer: None,
            kmipkit_client_create=create_client,
            kmipkit_client_release=lambda _pointer: None,
        ),
    )

    def expected_input(value: client.TransportInput) -> tuple[int, int, bytes]:
        source = value.source
        data = source if isinstance(source, bytes) else str(source).encode("utf-8")
        return (1 if isinstance(source, bytes) else 2, int(value.encoding), data)

    try:
        created = client.Client.create(extension_configuration, transport_configuration)
        assert observed == [
            {
                "abi_version": 1,
                "client_certificates": (expected_input(certificate),),
                "client_private_key": expected_input(private_key),
                "trust_certificates": (expected_input(trust_certificate),),
                "revocation_lists": ((1, int(encoding), b"crl-bytes"),),
            }
        ]
        created.close()
    finally:
        extension_configuration.close()


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
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
    )
    monkeypatch.setattr(client, "lib", fake)
    result = client._batch_result(response_pointer)
    assert result.items == (
        client.BatchItemResult(91, 2, 7, b"uid-1", b"pending"),
    )
    result.close()
    assert released == ["item", "response"]


def test_batch_execute_preserves_ordered_mixed_outcomes_and_invokes_native_execute_once(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    item_storage = [_opaque("kmipkit_client_batch_item_response_t") for _ in range(3)]
    item_pointers = [pointer for pointer, _storage in item_storage]
    identifiers = [client.ffi.new("uint8_t[]", f"item-{index}".encode()) for index in range(3)]
    correlation = client.ffi.new("uint8_t[]", b"\x00\xa1\x7f")
    released: list[str] = []
    execution_calls: list[bool] = []

    def execute(_client: object, batch_slot: object, _limits: object, output: object) -> int:
        execution_calls.append(True)
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        return 0

    def item_index(item: object) -> int:
        return item_pointers.index(item)

    def metadata(item: object, out: object) -> int:
        index = item_index(item)
        out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v1_t")  # type: ignore[attr-defined]
        out.abi_version = 1  # type: ignore[attr-defined]
        out.outcome_kind = (1, 2, 1)[index]  # type: ignore[attr-defined]
        out.result_status = (0, 2, 1)[index]  # type: ignore[attr-defined]
        out.has_result_reason = int(index == 2)  # type: ignore[attr-defined]
        if index == 2:
            out.result_reason = 1  # type: ignore[attr-defined]
        return 0

    def item_at(_response: object, index: int, out: object) -> int:
        out[0] = item_pointers[int(index)]  # type: ignore[index]
        return 0

    def unique_id(item: object, present: object, value: object) -> int:
        index = item_index(item)
        present[0] = 1  # type: ignore[index]
        value.data = identifiers[index]  # type: ignore[attr-defined]
        value.length = 6  # type: ignore[attr-defined]
        return 0

    def pending_correlation(item: object, present: object, value: object) -> int:
        if item_index(item) == 1:
            present[0] = 1  # type: ignore[index]
            value.data = correlation  # type: ignore[attr-defined]
            value.length = 3  # type: ignore[attr-defined]
        else:
            present[0] = 0  # type: ignore[index]
            value.data = client.ffi.NULL  # type: ignore[attr-defined]
            value.length = 0  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_execute=execute,
        kmipkit_client_release=lambda _ptr: None,
        kmipkit_client_batch_release=lambda _ptr: None,
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 3) or 0,
        kmipkit_client_batch_response_item_at=item_at,
        kmipkit_client_batch_item_response_metadata_v1=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=unique_id,
        kmipkit_client_batch_item_response_pending_correlation_value=pending_correlation,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
    )
    monkeypatch.setattr(client, "lib", fake)

    result = None
    try:
        result = handle.execute(batch, limits)
        assert result.items == (
            client.BatchItemResult(client.BatchOutcome.COMPLETED, 0, None, b"item-0", None),
            client.BatchItemResult(client.BatchOutcome.PENDING, 2, None, b"item-1", b"\x00\xa1\x7f"),
            client.BatchItemResult(client.BatchOutcome.COMPLETED, 1, 1, b"item-2", None),
        )
        assert execution_calls == [True]
        assert batch.closed
    finally:
        if result is not None:
            result.close()
        limits.close()
        batch.close()
        handle.close()

    assert released == ["item", "item", "item", "response"]


def test_discover_versions_response_mapping_matches_shared_fixture(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    fixture_path = Path(__file__).resolve().parents[3] / "tests/fixtures/api/shared-capability-parity.json"
    cases = json.loads(fixture_path.read_text(encoding="utf-8"))["cases"]
    case = next(item for item in cases if item["id"] == "discover-versions.supported-versions.v1")
    expected = case["expectedOutput"]
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_storage = [_opaque("kmipkit_client_batch_item_response_t") for _ in range(4)]
    item_pointers = [pointer for pointer, _storage in item_storage]
    released: list[str] = []

    def metadata(item: object, out: object) -> int:
        index = item_pointers.index(item)
        out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v1_t")  # type: ignore[attr-defined]
        out.abi_version = 1  # type: ignore[attr-defined]
        out.outcome_kind = (1, 2, 1, 1)[index]  # type: ignore[attr-defined]
        out.result_status = (0, 2, 1, 0)[index]  # type: ignore[attr-defined]
        out.has_result_reason = int(index == 2)  # type: ignore[attr-defined]
        out.result_reason = 1 if index == 2 else 0  # type: ignore[attr-defined]
        return 0

    def supported_versions(
        item: object, present: object, output: object, capacity: int, count: object
    ) -> int:
        index = item_pointers.index(item)
        present[0] = int(index in (0, 3))  # type: ignore[index]
        versions = expected["supportedVersions"] if index == 0 else []
        count[0] = len(versions) if index in (0, 3) else 0  # type: ignore[index]
        if capacity:
            for offset, version in enumerate(versions):
                output[offset].major = version["major"]  # type: ignore[index,attr-defined]
                output[offset].minor = version["minor"]  # type: ignore[index,attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 4) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, index, out: out.__setitem__(0, item_pointers[int(index)]) or 0,
        kmipkit_client_batch_item_response_metadata_v1=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda _item, present, value: present.__setitem__(0, 0) or setattr(value, "data", client.ffi.NULL) or setattr(value, "length", 0) or 0,
        kmipkit_client_batch_item_response_pending_correlation_value=lambda _item, present, value: present.__setitem__(0, 0) or setattr(value, "data", client.ffi.NULL) or setattr(value, "length", 0) or 0,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=supported_versions,
    )
    monkeypatch.setattr(client, "lib", fake)

    result = client._batch_result(response_pointer)
    try:
        assert result.items[0].supported_versions == tuple(
            client.ProtocolVersion(item["major"], item["minor"])
            for item in expected["supportedVersions"]
        )
        assert result.items[1].supported_versions is None
        assert result.items[2].supported_versions is None
        assert result.items[3].supported_versions == ()
    finally:
        result.close()

    assert released == ["item", "item", "item", "item", "response"]


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
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
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


def test_client_execute_with_options_preserves_timeout_kinds_and_consumes_batch(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    options = client.RequestOptions(
        connect_timeout=client.TimeoutLimit.default(),
        write_timeout=client.TimeoutLimit.bounded(0),
        read_timeout=client.TimeoutLimit.unbounded(),
        total_timeout=client.TimeoutLimit.bounded(2, 125_000_000),
    )
    observed: list[tuple[int, int, int]] = []

    def execute(
        _client: object,
        batch_slot: object,
        _limits: object,
        request_options: object,
        output: object,
    ) -> int:
        observed.extend(
            (
                int(getattr(request_options, name).kind),
                int(getattr(request_options, name).seconds),
                int(getattr(request_options, name).nanoseconds),
            )
            for name in (
                "connect_timeout",
                "write_timeout",
                "read_timeout",
                "total_timeout",
            )
        )
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_with_options_v1=execute,
            kmipkit_client_batch_response_item_count=lambda _response, count: count.__setitem__(0, 0) or 0,
            kmipkit_client_batch_response_release=lambda _ptr: None,
            kmipkit_client_release=lambda _ptr: None,
            kmipkit_client_batch_release=lambda _ptr: None,
        ),
    )
    try:
        result = handle.execute_with_options(batch, limits, options)
        assert observed == [(0, 0, 0), (1, 0, 0), (2, 0, 0), (1, 2, 125_000_000)]
        assert batch.closed
        assert result.items == ()
        result.close()
    finally:
        limits.close()
        batch.close()
        handle.close()


def test_request_options_and_execution_validate_types_before_native_calls(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    timeout = client.TimeoutLimit.default()
    with pytest.raises(errors.InvalidInputError):
        client.RequestOptions(None, timeout, timeout, timeout)  # type: ignore[arg-type]

    with pytest.raises(errors.InvalidInputError):
        client._validated_request_options(object())  # type: ignore[arg-type]

    forged = object.__new__(client.RequestOptions)
    for name in ("connect_timeout", "write_timeout", "read_timeout", "total_timeout"):
        object.__setattr__(forged, name, timeout)
    object.__setattr__(forged, "read_timeout", object())
    with pytest.raises(errors.InvalidInputError):
        client._validated_request_options(forged)

    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    options = client.RequestOptions(timeout, timeout, timeout, timeout)
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_release=lambda _ptr: None,
            kmipkit_client_batch_release=lambda _ptr: None,
        ),
    )
    try:
        for arguments in (
            (object(), limits, options),
            (batch, object(), options),
            (batch, limits, object()),
        ):
            with pytest.raises(errors.InvalidInputError):
                handle.execute_with_options(*arguments)  # type: ignore[arg-type]
    finally:
        limits.close()
        batch.close()
        handle.close()


@pytest.mark.parametrize("failure", [TypeError, ValueError, OverflowError])
def test_client_execute_with_options_restores_batch_on_ffi_conversion_error(
    monkeypatch: pytest.MonkeyPatch, failure: type[Exception]
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()

    def execute(*_arguments: object) -> int:
        raise failure("secret native conversion diagnostic")

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_with_options_v1=execute,
            kmipkit_client_release=lambda _ptr: None,
            kmipkit_client_batch_release=lambda _ptr: None,
        ),
    )
    options = client.RequestOptions(*(client.TimeoutLimit.default() for _ in range(4)))
    try:
        with pytest.raises(errors.InvalidInputError) as caught:
            handle.execute_with_options(batch, limits, options)
        assert "secret native conversion diagnostic" not in str(caught.value)
        assert not batch.closed
        assert batch._pointer() == batch_pointer
    finally:
        limits.close()
        batch.close()
        handle.close()


def test_client_execute_with_options_restores_batch_if_native_success_omits_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_with_options_v1=lambda *_arguments: 0,
            kmipkit_client_release=lambda _ptr: None,
            kmipkit_client_batch_release=lambda _ptr: None,
        ),
    )
    options = client.RequestOptions(
        *(client.TimeoutLimit.default() for _ in range(4))
    )
    try:
        with pytest.raises(errors.KmipKitError):
            handle.execute_with_options(batch, limits, options)
        assert not batch.closed
        assert batch._pointer() == batch_pointer
    finally:
        limits.close()
        batch.close()
        handle.close()


def test_client_execute_restores_batch_on_ffi_conversion_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
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


@pytest.mark.parametrize(
    ("status", "category", "cause", "delivery"),
    [
        (
            8,
            errors.ClientErrorCategory.PROTOCOL,
            errors.ClientErrorCause.PROTOCOL_OTHER,
            errors.DeliveryState.RESPONSE_STARTED,
        ),
        (
            9,
            errors.ClientErrorCategory.TRANSPORT,
            errors.ClientErrorCause.TIMEOUT,
            errors.DeliveryState.POSSIBLY_SENT,
        ),
        (
            10,
            errors.ClientErrorCategory.SERVER_RESULT,
            errors.ClientErrorCause.NONE,
            errors.DeliveryState.NONE,
        ),
    ],
)
def test_client_execute_maps_failure_delivery_and_restores_unconsumed_batch(
    monkeypatch: pytest.MonkeyPatch,
    status: int,
    category: errors.ClientErrorCategory,
    cause: errors.ClientErrorCause,
    delivery: errors.DeliveryState,
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()

    def last_error(status: object, category: object, cause: object, delivery: object) -> int:
        status[0] = status_code  # type: ignore[index]
        category[0] = int(client_category)  # type: ignore[index]
        cause[0] = int(cause_code)  # type: ignore[index]
        delivery[0] = int(delivery_state)  # type: ignore[index]
        return 0

    status_code = status
    client_category = category
    cause_code = cause
    delivery_state = delivery

    fake = SimpleNamespace(
        kmipkit_client_execute=lambda _client, _batch, _limits, _output: status_code,
        kmipkit_last_error_get=last_error,
        kmipkit_client_release=lambda _ptr: None,
        kmipkit_client_batch_release=lambda _ptr: None,
    )
    monkeypatch.setattr(client, "lib", fake)
    try:
        with pytest.raises(errors.ClientExecutionError) as caught:
            handle.execute(batch, limits)
        assert caught.value.status == status_code
        assert caught.value.client_category is client_category
        assert caught.value.cause is cause_code
        assert caught.value.delivery is delivery_state
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
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
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
    item = _wrap_owned_handle(client._ClientBatchItemResponse, pointer)

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
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    try:
        with pytest.raises(errors.InvalidInputError) as caught:
            batch.add_item(batch_item)
        assert "do not echo" not in str(caught.value)
        assert batch_item._pointer() == original_pointer
    finally:
        batch.close()
        batch_item.close()


def test_batch_add_item_rejects_untyped_item() -> None:
    pointer, _storage = _opaque("kmipkit_client_batch_t")
    batch = _wrap_owned_handle(client.Batch, pointer)
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
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    limits = ttlv.codec_limits_defaults()
    try:
        with pytest.raises(errors.InvalidInputError):
            handle.execute(object(), limits)  # type: ignore[arg-type]
        with pytest.raises(errors.InvalidInputError):
            handle.execute(_wrap_owned_handle(client.Batch, batch_pointer), object())  # type: ignore[arg-type]
        batch_pointer_2, _batch_storage_2 = _opaque("kmipkit_client_batch_t")
        batch = _wrap_owned_handle(client.Batch, batch_pointer_2)
        monkeypatch.setattr(
            client,
            "lib",
            SimpleNamespace(
                kmipkit_client_execute=lambda _client, _slot, _limits, _output: 0,
                kmipkit_client_release=lambda _ptr: None,
                kmipkit_client_batch_release=lambda _ptr: None,
            ),
        )
        with pytest.raises(errors.KmipKitError):
            handle.execute(batch, limits)
        assert not batch.closed
        batch.close()
    finally:
        limits.close()
        handle.close()


@pytest.mark.parametrize("operation", ["execute", "execute_with_options"])
@pytest.mark.parametrize("closed_input", ["client", "limits"])
def test_client_execute_rejects_closed_borrowed_handles_before_consuming_batch(
    monkeypatch: pytest.MonkeyPatch, operation: str, closed_input: str
) -> None:
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    client_handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    released: list[tuple[str, object]] = []
    fake = SimpleNamespace(
        kmipkit_client_release=lambda pointer: released.append(("client", pointer)),
        kmipkit_client_batch_release=lambda pointer: released.append(("batch", pointer)),
        kmipkit_client_execute=lambda *_arguments: pytest.fail(
            "closed client inputs must fail before native execution"
        ),
        kmipkit_client_execute_with_options_v1=lambda *_arguments: pytest.fail(
            "closed client inputs must fail before native execution"
        ),
    )
    monkeypatch.setattr(client, "lib", fake)

    try:
        if closed_input == "client":
            client_handle.close()
        else:
            limits.close()

        with pytest.raises(errors.InvalidInputError):
            if operation == "execute":
                client_handle.execute(batch, limits)
            else:
                client_handle.execute_with_options(
                    batch, limits, client.RequestOptions()
                )

        assert not batch.closed
    finally:
        batch.close()
        limits.close()
        client_handle.close()

    assert ("batch", batch_pointer) in released


def test_batch_add_item_rejects_closed_batch_before_consuming_item(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    item = extensions.client_batch_item_discover_versions()
    released: list[tuple[str, object]] = []
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_release=lambda pointer: released.append(
                ("batch", pointer)
            ),
            kmipkit_client_batch_item_release=lambda pointer: released.append(
                ("item", pointer)
            ),
            kmipkit_client_batch_add_item=lambda *_arguments: pytest.fail(
                "closed batch inputs must fail before native execution"
            ),
        ),
    )

    try:
        batch.close()
        with pytest.raises(errors.InvalidInputError):
            batch.add_item(item)
        assert not item.closed
    finally:
        item.close()

    assert released == [("batch", batch_pointer)]


@pytest.mark.parametrize(
    ("wrapper", "accessor"),
    [
        (ttlv.TtlvItemView, "kmipkit_ttlv_item_view_type"),
        (ttlv.TtlvValueView, "kmipkit_ttlv_value_view_type"),
    ],
)
def test_native_unknown_ttlv_item_type_maps_to_invalid_input(
    monkeypatch: pytest.MonkeyPatch, wrapper: type, accessor: str
) -> None:
    handle_type = (
        "kmipkit_ttlv_item_view_t"
        if wrapper is ttlv.TtlvItemView
        else "kmipkit_ttlv_value_view_t"
    )
    storage = ttlv.ffi.new("uint8_t[]", 1)
    pointer = ttlv.ffi.cast(f"{handle_type} *", storage)
    view = _wrap_owned_handle(wrapper, pointer)

    def unknown_type(_view: object, output: object) -> int:
        output[0] = 0xFF  # type: ignore[index]
        return 0

    monkeypatch.setattr(ttlv, "lib", SimpleNamespace(**{accessor: unknown_type}))
    try:
        with pytest.raises(errors.InvalidInputError):
            if wrapper is ttlv.TtlvItemView:
                ttlv.ttlv_item_view_type(view)
            else:
                ttlv.ttlv_value_view_type(view)
    finally:
        view.close()


def test_zero_client_status_does_not_read_last_error() -> None:
    client._raise_client_status(0)


def test_transport_record_handles_empty_optional_spans_from_absent_file_inputs() -> None:
    configuration = _transport_configuration(
        revocation_lists=(),
        trust=client.TrustSource.platform(),
    )
    record, _owners = client._transport_record(configuration)
    assert record.tls_server_name.length == 0
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
    trust = client.TrustSource.certificate_authorities((path,))
    configuration = _transport_configuration(
        client_certificates=(certificate,),
        client_private_key=key,
        trust=trust,
        revocation_lists=(),
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
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
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
        (99, "https://kms.example.test", 0, None, None),
    ],
)
def test_endpoint_rejects_incompatible_kind_and_endpoint_fields(
    arguments: tuple[object, ...],
) -> None:
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(*arguments)  # type: ignore[arg-type]


@pytest.mark.parametrize(
    "kind,address,port",
    [
        (client.EndpointKind.RAW_TLS, "kms.example.test", 5696),
        (client.EndpointKind.HTTPS, "https://kms.example.test", 0),
    ],
)
def test_endpoint_rejects_tls_name_override_for_dns_host(
    kind: client.EndpointKind, address: str, port: int
) -> None:
    with pytest.raises(errors.InvalidInputError):
        client.Endpoint(kind, address, port, "server.example.test")


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


def test_transport_input_memory_uses_wipeable_owned_storage() -> None:
    caller_buffer = bytearray(b"private-key-sentinel")
    item = client.TransportInput.memory(caller_buffer, client.CredentialEncoding.PEM)
    owner = client._transport_source(item)
    assert isinstance(owner, client._SecretBytes)
    owned_buffer = owner.buffer
    owner_reference = weakref.ref(owner)

    assert type(item.source) is bytes
    assert item.source == b"private-key-sentinel"
    assert owned_buffer == b"private-key-sentinel"
    caller_buffer[:] = b"caller-buffer-remains"
    assert item.source == b"private-key-sentinel"

    del item, owner
    gc.collect()

    assert owner_reference() is None
    assert owned_buffer == bytes(len(owned_buffer))
    assert caller_buffer == b"caller-buffer-remains"


@pytest.mark.parametrize("transport_status", [0, 4, None])
def test_client_create_zeroizes_temporary_key_copies_and_keeps_config_reusable(
    monkeypatch: pytest.MonkeyPatch, transport_status: int | None
) -> None:
    caller_buffer = bytearray(b"private-key-sentinel")
    key = client.TransportInput.memory(caller_buffer, client.CredentialEncoding.PEM)
    original_owner = client._transport_source(key)
    transport_configuration = _transport_configuration(client_private_key=key)
    extension_configuration = _extension_configuration()
    temporary_configs: list[client.TransportConfig] = []
    original_validator = client._validated_transport_config

    def capture_validated(value: client.TransportConfig) -> client.TransportConfig:
        validated = original_validator(value)
        temporary_configs.append(validated)
        return validated

    monkeypatch.setattr(client, "_validated_transport_config", capture_validated)
    captured_secret_owners: list[client._SecretBytes] = []
    original_wipe = client._wipe_transport_secrets

    def capture_wiped_owners(owners: tuple[object, ...] | list[object]) -> None:
        captured_secret_owners.extend(
            owner for owner in owners if isinstance(owner, client._SecretBytes)
        )
        original_wipe(owners)

    monkeypatch.setattr(client, "_wipe_transport_secrets", capture_wiped_owners)
    transport_pointer, _transport_storage = _opaque("kmipkit_transport_config_t")
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    key_spans: list[tuple[object, int]] = []
    current_status = [transport_status]

    def create_transport(record: object, output: object) -> int:
        key_record = record.client_private_key  # type: ignore[attr-defined]
        key_spans.append((key_record.data.data, int(key_record.data.length)))
        status = current_status[0]
        if status is None:
            raise RuntimeError("simulated native failure")
        if status == 0:
            output[0] = transport_pointer  # type: ignore[index]
        return status

    def create_client(_configuration: object, slot: object, output: object) -> int:
        slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = client_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_transport_config_create_v1=create_transport,
            kmipkit_transport_config_release=lambda _pointer: None,
            kmipkit_client_create=create_client,
            kmipkit_client_release=lambda _pointer: None,
        ),
    )

    try:
        if transport_status == 0:
            created = client.Client.create(extension_configuration, transport_configuration)
            created.close()
        elif transport_status is None:
            with pytest.raises(RuntimeError, match="simulated native failure"):
                client.Client.create(extension_configuration, transport_configuration)
        else:
            with pytest.raises(errors.InvalidInputError):
                client.Client.create(extension_configuration, transport_configuration)
        if transport_status != 0:
            current_status[0] = 0
            created = client.Client.create(extension_configuration, transport_configuration)
            created.close()

        assert key_spans
        assert captured_secret_owners
        for pointer, length in key_spans:
            assert bytes(client.ffi.buffer(pointer, length)) == bytes(length)
        assert all(
            owner.buffer == bytes(len(owner.buffer)) for owner in captured_secret_owners
        )
        assert temporary_configs
        for temporary in temporary_configs:
            temporary_key = client._transport_source(temporary.client_private_key)
            assert isinstance(temporary_key, client._SecretBytes)
            assert temporary_key.buffer == bytes(len(temporary_key.buffer))
        assert original_owner.buffer == b"private-key-sentinel"
        assert key.source == b"private-key-sentinel"
        assert caller_buffer == b"private-key-sentinel"
    finally:
        extension_configuration.close()


def test_transport_record_wipes_key_span_when_later_record_validation_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class UnvalidatedTrust(client.TrustSource):
        def __post_init__(self) -> None:
            pass

    key = client.TransportInput.memory(b"private-key-sentinel", client.CredentialEncoding.PEM)
    certificate = client.TransportInput.memory(
        b"client-certificate", client.CredentialEncoding.PEM
    )
    invalid_trust = UnvalidatedTrust(client.TrustKind.CERTIFICATES, (object(),))
    transport = _unvalidated_transport_configuration(
        client_private_key=key,
        client_certificates=(certificate,),
        trust=invalid_trust,
    )
    captured_secret_owners: list[client._SecretBytes] = []
    original_wipe = client._wipe_transport_secrets

    def capture_wiped_owners(owners: tuple[object, ...] | list[object]) -> None:
        captured_secret_owners.extend(
            owner for owner in owners if isinstance(owner, client._SecretBytes)
        )
        original_wipe(owners)

    monkeypatch.setattr(client, "_wipe_transport_secrets", capture_wiped_owners)

    with pytest.raises(errors.InvalidInputError):
        client._transport_record(transport)

    assert captured_secret_owners
    assert all(
        owner.buffer == bytes(len(owner.buffer)) for owner in captured_secret_owners
    )
    assert client._transport_source(key).buffer == b"private-key-sentinel"
    assert client._transport_source(certificate).buffer == b"client-certificate"


def test_transport_input_preserves_dataclass_replace_and_hash_behavior() -> None:
    original = client.TransportInput.memory(
        b"private-key-sentinel", client.CredentialEncoding.PEM
    )

    copied = replace(original)

    assert copied == original
    assert hash(copied) == hash(original)
    assert {original: "credential"}[copied] == "credential"

    replaced_source = replace(original, source=b"replacement-key")
    assert replaced_source.source == b"replacement-key"
    assert original.source == b"private-key-sentinel"
