"""Contract tests for the shared Python client facade (KMIPKIT-0043 T024).

The client facade is a language adapter over the reviewed T076 C ABI. These
tests intentionally do not create a real network connection; Rust/C tests own
transport-wire verification.
"""

from __future__ import annotations

import gc
import importlib
import json
import sys
import weakref
from dataclasses import replace
from pathlib import Path
from types import SimpleNamespace
from typing import get_type_hints

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._handles import _wipe_staging_buffer, _wrap_owned_handle


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


@pytest.mark.parametrize(
    ("major", "minor"),
    [
        (-(2**31) - 1, 1),
        (2**31, 1),
        (2, -(2**31) - 1),
        (2, 2**31),
    ],
)
def test_protocol_version_rejects_components_outside_signed_ttlv_integer_range(
    major: int, minor: int
) -> None:
    with pytest.raises(ValueError, match="signed 32-bit"):
        client.ProtocolVersion(major, minor)


@pytest.mark.parametrize("value", [True, 2.1, "2", None])
def test_protocol_version_rejects_non_integer_components(value: object) -> None:
    with pytest.raises(TypeError, match="must be an integer"):
        client.ProtocolVersion(value, 1)  # type: ignore[arg-type]


def test_protocol_version_preserves_signed_ttlv_integer_boundaries() -> None:
    version = client.ProtocolVersion(-(2**31), 2**31 - 1)

    assert version.major == -(2**31)
    assert version.minor == 2**31 - 1


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


def test_client_execution_error_response_ttlv_annotation_resolves() -> None:
    annotations = get_type_hints(errors.ClientExecutionError.__init__)

    assert annotations["response_ttlv"] == ttlv.Structure | None


def test_version_two_metadata_preserves_unknowns_message_presence_and_redacts_repr() -> None:
    message = "metadata marker: création ✓"
    metadata = client.BatchItemMetadataV2(
        operation=0xFFFF_FFFF,
        outcome=99,
        result_status=0xFFFF_FFFE,
        result_reason=0xFFFF_FFFD,
        result_message=message,
    )
    empty_message = client.BatchItemMetadataV2(40, client.BatchOutcome.COMPLETED, 0xFFFF_FFFE, None, "")
    absent_message = client.BatchItemMetadataV2(40, client.BatchOutcome.COMPLETED, 0xFFFF_FFFE, None, None)

    assert metadata.operation == 0xFFFF_FFFF
    assert metadata.outcome == 99
    assert metadata.result_status == 0xFFFF_FFFE
    assert metadata.result_reason == 0xFFFF_FFFD
    assert metadata.result_message == message
    assert message not in repr(metadata)
    assert empty_message.result_message == ""
    assert absent_message.result_message is None


def test_version_two_metadata_rejects_invalid_integer_ranges_and_non_utf8_text() -> None:
    for values in (
        (-1, 1, 0, None, None),
        (0x1_0000_0000, 1, 0, None, None),
        (1, -1, 0, None, None),
        (1, 1, -1, None, None),
        (1, 1, 0, 0x1_0000_0000, None),
    ):
        with pytest.raises(errors.InvalidInputError):
            client.BatchItemMetadataV2(*values)

    with pytest.raises(errors.InvalidInputError):
        client.BatchItemMetadataV2(40, 1, 0, None, "\ud800")


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


def _initialize_metadata_v2(out: object) -> None:
    out.struct_size = client.ffi.sizeof("kmipkit_client_batch_item_metadata_v2_t")  # type: ignore[attr-defined]
    out.abi_version = 2  # type: ignore[attr-defined]
    out.operation = 0  # type: ignore[attr-defined]
    out.outcome_kind = int(client.BatchOutcome.COMPLETED)  # type: ignore[attr-defined]
    out.result_status = 0  # type: ignore[attr-defined]
    out.has_result_reason = 0  # type: ignore[attr-defined]
    out.result_reason = 0  # type: ignore[attr-defined]
    out.has_result_message = 0  # type: ignore[attr-defined]
    out.reserved = 0  # type: ignore[attr-defined]
    out.result_message_data = client.ffi.NULL  # type: ignore[attr-defined]
    out.result_message_length = 0  # type: ignore[attr-defined]


def _no_creation_response_symbols() -> dict[str, object]:
    def create(_item: object, present: object, object_type: object, _identifier: object) -> int:
        present[0] = 0  # type: ignore[index]
        object_type[0] = 0  # type: ignore[index]
        return 0

    def create_pair(
        _item: object, present: object, _private_identifier: object, _public_identifier: object
    ) -> int:
        present[0] = 0  # type: ignore[index]
        return 0

    def create_split_key(_item: object, present: object, count: object) -> int:
        present[0] = 0  # type: ignore[index]
        count[0] = 0  # type: ignore[index]
        return 0

    return {
        "kmipkit_client_batch_item_response_create": create,
        "kmipkit_client_batch_item_response_create_key_pair": create_pair,
        "kmipkit_client_batch_item_response_create_split_key_count": create_split_key,
    }


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


def test_secret_and_file_transport_inputs_keep_equality_hash_and_redaction_contracts(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    secret = client._SecretBytes(b"private-material")
    equal_secret = client._SecretBytes(b"private-material")
    other_secret = client._SecretBytes(b"different-material")

    assert secret == equal_secret
    assert secret != other_secret
    assert client._SecretBytes.__eq__(secret, object()) is NotImplemented
    assert secret.to_bytes() == b"private-material"

    def fail_wipe(_self: client._SecretBytes) -> None:
        raise RuntimeError("synthetic wipe failure")

    monkeypatch.setattr(client._SecretBytes, "wipe", fail_wipe)
    secret.__del__()

    file_input = client.TransportInput.file(
        Path("trust-material.pem"), client.CredentialEncoding.PEM
    )
    assert client.TransportInput.__eq__(file_input, object()) is NotImplemented
    assert isinstance(hash(file_input), int)


def test_transport_input_rejects_path_stringification_failures() -> None:
    concrete_path = type(Path("ignored"))

    class InvalidStringPath(concrete_path):
        def __str__(self) -> str:
            raise TypeError("synthetic path conversion failure")

    with pytest.raises(errors.InvalidInputError):
        client.TransportInput(InvalidStringPath("ignored"), client.CredentialEncoding.PEM)


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


def test_batch_request_timestamp_preserves_absence_and_signed_boundaries(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    batch_pointer, _storage = _opaque("kmipkit_client_batch_t")
    values = [(0, 0), (1, 0), (1, -(1 << 63)), (1, (1 << 63) - 1)]

    def create(output: object) -> int:
        output[0] = batch_pointer  # type: ignore[index]
        return 0

    def request_timestamp(_batch: object, has_value: object, value: object) -> int:
        present, timestamp = values.pop(0)
        has_value[0] = present  # type: ignore[index]
        value[0] = timestamp  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_create=create,
        kmipkit_client_batch_release=lambda _pointer: None,
        kmipkit_client_batch_request_time_stamp=request_timestamp,
    )
    monkeypatch.setattr(client, "lib", fake)
    batch = client.Batch.create()
    try:
        assert batch.request_timestamp is None
        assert batch.request_timestamp == 0
        assert batch.request_timestamp == -(1 << 63)
        assert batch.request_timestamp == (1 << 63) - 1
        assert values == []
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


def test_request_batch_item_unique_id_accessor_preserves_absence_empty_and_bytes(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_storage = extensions.ffi.new("uint8_t[]", 1)
    item_pointer = extensions.ffi.cast("kmipkit_client_batch_item_t *", item_storage)
    binary_storage = extensions.ffi.new("uint8_t[]", b"\x00\xffid")
    state: list[tuple[int, object, int]] = [
        (0, extensions.ffi.NULL, 0),
        (1, extensions.ffi.NULL, 0),
        (1, binary_storage, 4),
    ]

    def create(output: object) -> int:
        output[0] = item_pointer  # type: ignore[index]
        return 0

    def unique_id(_item: object, out_present: object, out_value: object) -> int:
        present, data, length = state.pop(0)
        out_present[0] = present  # type: ignore[index]
        out_value.data = data  # type: ignore[attr-defined]
        out_value.length = length  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        extensions,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_discover_versions=create,
            kmipkit_client_batch_item_unique_batch_item_id=unique_id,
            kmipkit_client_batch_item_release=lambda _pointer: None,
        ),
    )
    item = extensions.client_batch_item_discover_versions()
    absent = extensions.client_batch_item_unique_batch_item_id(item)
    empty = extensions.client_batch_item_unique_batch_item_id(item)
    binary = extensions.client_batch_item_unique_batch_item_id(item)

    assert absent is None
    assert empty == b""
    assert binary == b"\x00\xffid"
    assert not item.closed
    assert item._pointer() == item_pointer
    item.close()


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
    message_text = "server metadata: création ✓"
    message = client.ffi.new("uint8_t[]", message_text.encode("utf-8"))
    released: list[str] = []

    def metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
        out.operation = 0xFFFF_FFFF  # type: ignore[attr-defined]
        out.outcome_kind = 91  # type: ignore[attr-defined]
        out.result_status = 1  # type: ignore[attr-defined]
        out.has_result_reason = 1  # type: ignore[attr-defined]
        out.result_reason = 7  # type: ignore[attr-defined]
        out.has_result_message = 1  # type: ignore[attr-defined]
        out.result_message_data = message  # type: ignore[attr-defined]
        out.result_message_length = len(message_text.encode("utf-8"))  # type: ignore[attr-defined]
        return 0

    def set_span(data: object, value: object, present: object) -> int:
        present[0] = 1  # type: ignore[index]
        value.data = data  # type: ignore[attr-defined]
        value.length = len(client.ffi.string(data))  # type: ignore[attr-defined]
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
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda item, present, value: set_span(unique_id, value, present),
        kmipkit_client_batch_item_response_pending_correlation_value=absent_span,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
        **_no_creation_response_symbols(),
    )
    monkeypatch.setattr(client, "lib", fake)
    result = client._batch_result(response_pointer)
    assert result.items == (
        client.BatchItemResult(
            91, 1, 7, b"uid-1", None, None, 0xFFFF_FFFF, message_text
        ),
    )
    assert message_text not in repr(result.items[0])
    result.close()
    assert released == ["item", "response"]


def test_batch_result_dispatches_check_operation_to_typed_mapper(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    released: list[str] = []

    def metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
        out.operation = 0x09  # type: ignore[attr-defined]
        out.result_status = 1  # type: ignore[attr-defined]
        return 0

    def absent_span(_item: object, present: object, value: object) -> int:
        present[0] = 0  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = 0  # type: ignore[attr-defined]
        return 0

    def absent_check(
        _item: object, present: object, identifier_present: object,
        identifier: object, count_present: object, count: object,
        mask_present: object, usage_mask: object, lease_present: object, lease: object,
    ) -> int:
        present[0] = 0  # type: ignore[index]
        identifier_present[0] = 0  # type: ignore[index]
        identifier.kind = 0  # type: ignore[attr-defined]
        count_present[0] = 0  # type: ignore[index]
        count[0] = 0  # type: ignore[index]
        mask_present[0] = 0  # type: ignore[index]
        usage_mask[0] = 0  # type: ignore[index]
        lease_present[0] = 0  # type: ignore[index]
        lease[0] = 0  # type: ignore[index]
        return 0

    def absent_check_ttlv(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = client.ffi.NULL  # type: ignore[index]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=absent_span,
        kmipkit_client_batch_item_response_pending_correlation_value=absent_span,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
        kmipkit_client_batch_item_response_check=absent_check,
        kmipkit_client_batch_item_response_check_ttlv=absent_check_ttlv,
        **_no_creation_response_symbols(),
    )
    monkeypatch.setattr(client, "lib", fake)

    result = client._batch_result(response_pointer)

    assert result.items[0].operation == 0x09
    assert result.items[0].check_response is None
    result.close()
    assert released == ["item", "response"]


@pytest.mark.parametrize(
    ("operation", "mapper_name", "response_field"),
    [
        pytest.param(0x2A, "_import_response", "import_response", id="import"),
        pytest.param(0x34, "_interop_response", "interop_response", id="interop"),
    ],
)
def test_batch_result_dispatches_successful_import_and_interop_responses(
    monkeypatch: pytest.MonkeyPatch,
    operation: int,
    mapper_name: str,
    response_field: str,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    released: list[str] = []
    dispatched: list[object] = []
    mapped_response = object()

    def metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
        out.operation = operation  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda *_args: 0,
        kmipkit_client_batch_item_response_pending_correlation_value=lambda *_args: 0,
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client, "_response_span", lambda _item, _function: None)
    monkeypatch.setattr(client, "_discover_versions_supported_versions", lambda _item: None)
    monkeypatch.setattr(client, "_create_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_key_pair_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_split_key_response", lambda _item: None)

    def response_mapper(item: object) -> object:
        dispatched.append(item._pointer())  # type: ignore[attr-defined]
        return mapped_response

    monkeypatch.setattr(client, mapper_name, response_mapper)

    result = client._batch_result(response_pointer)
    try:
        assert result.items[0].operation == operation
        assert getattr(result.items[0], response_field) is mapped_response
        assert dispatched == [item_pointer]
    finally:
        result.close()

    assert released == ["item", "response"]


def test_batch_result_dispatches_export_operation_to_typed_mapper(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    released: list[str] = []
    export_response = object()
    dispatched: list[object] = []

    def metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
        out.operation = 0x2B  # type: ignore[attr-defined]
        out.result_status = 1  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda *_args: 0,
        kmipkit_client_batch_item_response_pending_correlation_value=lambda *_args: 0,
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client, "_response_span", lambda _item, _function: None)
    monkeypatch.setattr(client, "_discover_versions_supported_versions", lambda _item: None)
    monkeypatch.setattr(client, "_create_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_key_pair_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_split_key_response", lambda _item: None)

    def export_mapper(item: object) -> object:
        dispatched.append(item)
        return export_response

    monkeypatch.setattr(client, "_export_response", export_mapper)

    result = client._batch_result(response_pointer)
    try:
        assert result.items[0].operation == 0x2B
        assert result.items[0].result_status == 1
        assert result.items[0].export_response is export_response
        assert len(dispatched) == 1
    finally:
        result.close()

    assert released == ["item", "response"]


@pytest.mark.parametrize(
    ("outcome", "result_status", "expect_mapping"),
    [
        pytest.param(client.BatchOutcome.COMPLETED, 0, True, id="success"),
        pytest.param(client.BatchOutcome.PENDING, 2, False, id="pending"),
        pytest.param(client.BatchOutcome.COMPLETED, 1, False, id="failure"),
    ],
)
def test_batch_result_dispatches_get_constraints_only_for_successful_completion(
    monkeypatch: pytest.MonkeyPatch,
    outcome: client.BatchOutcome,
    result_status: int,
    expect_mapping: bool,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    released: list[str] = []
    get_constraints_response = object()
    dispatched: list[object] = []

    def metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
        out.operation = 0x38  # type: ignore[attr-defined]
        out.outcome_kind = int(outcome)  # type: ignore[attr-defined]
        out.result_status = result_status  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda *_args: 0,
        kmipkit_client_batch_item_response_pending_correlation_value=lambda *_args: 0,
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client, "_response_span", lambda _item, _function: None)
    monkeypatch.setattr(client, "_discover_versions_supported_versions", lambda _item: None)
    monkeypatch.setattr(client, "_create_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_key_pair_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_split_key_response", lambda _item: None)

    def get_constraints_mapper(item: object) -> object:
        dispatched.append(item)
        return get_constraints_response

    monkeypatch.setattr(client, "_get_constraints_response", get_constraints_mapper)

    result = client._batch_result(response_pointer)
    try:
        assert result.items[0].outcome is outcome
        assert result.items[0].operation == 0x38
        assert result.items[0].result_status == result_status
        assert result.items[0].get_constraints_response is (
            get_constraints_response if expect_mapping else None
        )
        assert len(dispatched) == int(expect_mapping)
    finally:
        result.close()

    assert released == ["item", "response"]


@pytest.mark.parametrize(
    ("operation", "mapper_name", "accessor_name", "accessor_is_method"),
    [
        pytest.param(0x0A, "_get_response", "get_response", True, id="get"),
        pytest.param(0x08, "_locate_response", "locate_response", True, id="locate"),
        pytest.param(0x2C, "_log_response", "log_response", False, id="log"),
    ],
)
def test_batch_result_dispatches_get_locate_and_log_mappers(
    monkeypatch: pytest.MonkeyPatch,
    operation: int,
    mapper_name: str,
    accessor_name: str,
    accessor_is_method: bool,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    mapped_response = object()
    released: list[str] = []
    dispatched: list[object] = []

    def metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
        out.operation = operation  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 1) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, _index, out: out.__setitem__(0, item_pointer) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda *_args: 0,
        kmipkit_client_batch_item_response_pending_correlation_value=lambda *_args: 0,
        **_no_creation_response_symbols(),
    )
    monkeypatch.setattr(client, "lib", fake)
    monkeypatch.setattr(client, "_response_span", lambda _item, _function: None)
    monkeypatch.setattr(client, "_discover_versions_supported_versions", lambda _item: None)
    monkeypatch.setattr(client, "_create_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_key_pair_response", lambda _item: (None, None))
    monkeypatch.setattr(client, "_create_split_key_response", lambda _item: None)

    def mapper(item: object) -> object:
        dispatched.append(item._pointer())  # type: ignore[attr-defined]
        return mapped_response

    monkeypatch.setattr(client, mapper_name, mapper)
    result = client._batch_result(response_pointer)
    try:
        item_result = result.items[0]
        assert item_result.operation == operation
        accessor = getattr(item_result, accessor_name)
        assert (accessor() if accessor_is_method else accessor) is mapped_response
        assert dispatched == [item_pointer]
    finally:
        result.close()

    assert released == ["item", "response"]


def test_batch_execute_preserves_ordered_mixed_outcomes_and_invokes_native_execute_once(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    fixture_path = (
        Path(__file__).resolve().parents[3]
        / "tests/fixtures/api/shared-capability-parity.json"
    )
    cases = json.loads(fixture_path.read_text(encoding="utf-8"))["cases"]
    case = next(
        item
        for item in cases
        if item["id"] == "batch.ordered-completed-pending-failed.v1"
    )
    expected = case["expectedOutput"]
    expected_ids = tuple(
        bytes.fromhex(value) for value in expected["uniqueBatchItemIds"]
    )
    status_codes = {"Success": 0, "Operation Pending": 2, "Operation Failed": 1}
    expected_statuses = tuple(
        status_codes[value] for value in expected["resultStatuses"]
    )
    assert len(expected["itemIndexes"]) == len(expected_ids) == len(expected_statuses)

    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    item_storage = [
        _opaque("kmipkit_client_batch_item_response_t") for _ in expected["itemIndexes"]
    ]
    item_pointers = [pointer for pointer, _storage in item_storage]
    identifiers = [client.ffi.new("uint8_t[]", value) for value in expected_ids]
    correlation_value = bytes.fromhex(expected["correlationValueHex"])
    correlation = client.ffi.new("uint8_t[]", correlation_value)
    released: list[str] = []
    execution_calls: list[bool] = []

    def execute(
        _client: object,
        batch_slot: object,
        _limits: object,
        output: object,
        error_output: object,
    ) -> int:
        execution_calls.append(True)
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        assert error_output[0] == client.ffi.NULL  # type: ignore[index]
        return 0

    def item_index(item: object) -> int:
        return item_pointers.index(item)

    def metadata(item: object, out: object) -> int:
        index = item_index(item)
        _initialize_metadata_v2(out)
        status = expected["resultStatuses"][index]
        out.outcome_kind = 2 if status == "Operation Pending" else 1  # type: ignore[attr-defined]
        out.result_status = expected_statuses[index]  # type: ignore[attr-defined]
        failed = status == "Operation Failed"
        out.has_result_reason = int(failed)  # type: ignore[attr-defined]
        if failed:
            out.result_reason = 1  # type: ignore[attr-defined]
        return 0

    def item_at(_response: object, index: int, out: object) -> int:
        out[0] = item_pointers[int(index)]  # type: ignore[index]
        return 0

    def unique_id(item: object, present: object, value: object) -> int:
        index = item_index(item)
        present[0] = 1  # type: ignore[index]
        value.data = identifiers[index]  # type: ignore[attr-defined]
        value.length = len(expected_ids[index])  # type: ignore[attr-defined]
        return 0

    def pending_correlation(item: object, present: object, value: object) -> int:
        if expected["resultStatuses"][item_index(item)] == "Operation Pending":
            present[0] = 1  # type: ignore[index]
            value.data = correlation  # type: ignore[attr-defined]
            value.length = len(correlation_value)  # type: ignore[attr-defined]
        else:
            present[0] = 0  # type: ignore[index]
            value.data = client.ffi.NULL  # type: ignore[attr-defined]
            value.length = 0  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_execute_with_error_ttlv_v1=execute,
        kmipkit_client_release=lambda _ptr: None,
        kmipkit_client_batch_release=lambda _ptr: None,
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: (
            count.__setitem__(0, len(item_pointers)) or 0
        ),
        kmipkit_client_batch_response_item_at=item_at,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=unique_id,
        kmipkit_client_batch_item_response_pending_correlation_value=pending_correlation,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
        **_no_creation_response_symbols(),
    )
    monkeypatch.setattr(client, "lib", fake)

    result = None
    try:
        result = handle.execute(batch, limits)
        expected_results = tuple(
            client.BatchItemResult(
                client.BatchOutcome.PENDING
                if status == "Operation Pending"
                else client.BatchOutcome.COMPLETED,
                expected_statuses[index],
                1 if status == "Operation Failed" else None,
                expected_ids[index],
                correlation_value if status == "Operation Pending" else None,
            )
            for index, status in enumerate(expected["resultStatuses"])
        )
        assert result.items == expected_results
        assert (
            sum(item.outcome is client.BatchOutcome.PENDING for item in result.items)
            == expected["pendingCount"]
        )
        assert expected["automaticFollowUp"] is False
        assert expected["transportExchangeCount"] == 1
        assert len(execution_calls) == expected["transportExchangeCount"]
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
        _initialize_metadata_v2(out)
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
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=lambda _item, present, value: present.__setitem__(0, 0) or setattr(value, "data", client.ffi.NULL) or setattr(value, "length", 0) or 0,
        kmipkit_client_batch_item_response_pending_correlation_value=lambda _item, present, value: present.__setitem__(0, 0) or setattr(value, "data", client.ffi.NULL) or setattr(value, "length", 0) or 0,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=supported_versions,
        **_no_creation_response_symbols(),
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


def test_creation_response_mapping_preserves_wire_identifiers_and_failure_message(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    fixture_path = Path(__file__).resolve().parents[3] / "tests/fixtures/api/shared-capability-parity.json"
    fixture_cases = json.loads(fixture_path.read_text(encoding="utf-8"))["cases"]
    create_case = next(
        case for case in fixture_cases if case["id"] == "creation.create.request-response.v1"
    )
    create_input = create_case["input"]
    assert create_input["attributesPresent"] is True
    assert create_input["protectionStorageMasks"]["present"] is True
    create_attributes = ttlv.ttlv_structure_create()
    create_masks = ttlv.ttlv_structure_create()
    create_request = None
    try:
        create_request = extensions.client_batch_item_create(
            create_input["objectType"], create_attributes, create_masks
        )
        assert create_request is not None
    finally:
        if create_request is not None:
            create_request.close()
        create_attributes.close()
        create_masks.close()

    key_pair_case = next(
        case for case in fixture_cases if case["id"] == "creation.key-pair.request-response.v1"
    )
    key_pair_input = key_pair_case["input"]
    assert key_pair_input["effectiveTable191ValuesEqual"] is True
    key_pair_structures = [
        _fixture_attribute_structure(key_pair_input[group]["attributeTagsInOrder"])
        for group in ("commonAttributes", "privateKeyAttributes", "publicKeyAttributes")
    ]
    key_pair_common_masks = ttlv.ttlv_structure_create()
    key_pair_public_masks = ttlv.ttlv_structure_create()
    key_pair_request = None
    try:
        key_pair_request = extensions.client_batch_item_create_key_pair(
            *key_pair_structures,
            key_pair_common_masks,
            None,
            key_pair_public_masks,
        )
        assert key_pair_request is not None
    finally:
        if key_pair_request is not None:
            key_pair_request.close()
        for structure in key_pair_structures:
            structure.close()
        key_pair_common_masks.close()
        key_pair_public_masks.close()

    fixtures = {
        case["id"]: case["expectedOutput"]
        for case in fixture_cases
        if case["id"].startswith("creation.")
    }
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_storage = [_opaque("kmipkit_client_batch_item_response_t") for _ in range(4)]
    item_pointers = [pointer for pointer, _storage in item_storage]
    message_text = fixtures["creation.result-message-on-failure.v1"]["resultMessage"]
    message_data = client.ffi.new("uint8_t[]", message_text.encode("utf-8"))
    text_storage: list[object] = []
    released: list[str] = []

    def item_index(item: object) -> int:
        return item_pointers.index(item)

    def write_identifier(output: object, kind: int, value: object) -> None:
        output.kind = kind  # type: ignore[attr-defined]
        output.reserved = 0  # type: ignore[attr-defined]
        output.text_data = client.ffi.NULL  # type: ignore[attr-defined]
        output.text_length = 0  # type: ignore[attr-defined]
        output.enumeration_value = 0  # type: ignore[attr-defined]
        output.integer_value = 0  # type: ignore[attr-defined]
        if kind == int(extensions.UniqueIdentifierKind.TEXT_STRING):
            text = str(value).encode("utf-8")
            data = client.ffi.new("uint8_t[]", text)
            text_storage.append(data)
            output.text_data = data  # type: ignore[attr-defined]
            output.text_length = len(text)  # type: ignore[attr-defined]
        elif kind == int(extensions.UniqueIdentifierKind.ENUMERATION):
            output.enumeration_value = int(value)  # type: ignore[attr-defined]
        else:
            output.integer_value = int(value)  # type: ignore[attr-defined]

    def metadata(item: object, out: object) -> int:
        index = item_index(item)
        _initialize_metadata_v2(out)
        out.operation = (1, 1, 2, 40)[index]  # type: ignore[attr-defined]
        out.result_status = (0, 1, 0, 0)[index]  # type: ignore[attr-defined]
        if index == 1:
            out.has_result_reason = 1  # type: ignore[attr-defined]
            out.result_reason = fixtures["creation.result-message-on-failure.v1"]["resultReason"]  # type: ignore[attr-defined]
            out.has_result_message = 1  # type: ignore[attr-defined]
            out.result_message_data = message_data  # type: ignore[attr-defined]
            out.result_message_length = len(message_text.encode("utf-8"))  # type: ignore[attr-defined]
        return 0

    def create(item: object, present: object, object_type: object, identifier: object) -> int:
        index = item_index(item)
        present[0] = int(index == 0)  # type: ignore[index]
        object_type[0] = 7 if index == 0 else 0  # type: ignore[index]
        if index == 0:
            write_identifier(identifier, int(extensions.UniqueIdentifierKind.INTEGER), -8)
        return 0

    def create_key_pair(
        item: object, present: object, private_identifier: object, public_identifier: object
    ) -> int:
        index = item_index(item)
        present[0] = int(index == 2)  # type: ignore[index]
        if index == 2:
            write_identifier(
                private_identifier,
                int(extensions.UniqueIdentifierKind.ENUMERATION),
                0xFFFF_FFFF,
            )
            write_identifier(
                public_identifier,
                int(extensions.UniqueIdentifierKind.TEXT_STRING),
                "public-key-α",
            )
        return 0

    def split_count(item: object, present: object, count: object) -> int:
        index = item_index(item)
        present[0] = int(index == 3)  # type: ignore[index]
        count[0] = 4 if index == 3 else 0  # type: ignore[index]
        return 0

    split_values = (
        (extensions.UniqueIdentifierKind.TEXT_STRING, "part-a"),
        (extensions.UniqueIdentifierKind.ENUMERATION, 0xFFFF_FFFF),
        (extensions.UniqueIdentifierKind.INTEGER, -7),
        (extensions.UniqueIdentifierKind.TEXT_STRING, "part-a"),
    )

    def split_identifier(item: object, index: int, output: object) -> int:
        assert item_index(item) == 3
        kind, value = split_values[int(index)]
        write_identifier(output, int(kind), value)
        return 0

    def absent_span(_item: object, present: object, value: object) -> int:
        present[0] = 0  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = 0  # type: ignore[attr-defined]
        return 0

    fake = SimpleNamespace(
        kmipkit_client_batch_response_release=lambda _ptr: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _ptr: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _ptr, count: count.__setitem__(0, 4) or 0,
        kmipkit_client_batch_response_item_at=lambda _ptr, index, out: out.__setitem__(0, item_pointers[int(index)]) or 0,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=absent_span,
        kmipkit_client_batch_item_response_pending_correlation_value=absent_span,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
        kmipkit_client_batch_item_response_create=create,
        kmipkit_client_batch_item_response_create_key_pair=create_key_pair,
        kmipkit_client_batch_item_response_create_split_key_count=split_count,
        kmipkit_client_batch_item_response_create_split_key_unique_identifier_at=split_identifier,
    )
    monkeypatch.setattr(client, "lib", fake)

    result = client._batch_result(response_pointer)
    try:
        create_expected = fixtures["creation.create.request-response.v1"]
        create_item = result.items[0]
        assert create_item.operation == 1
        assert create_item.create_object_type == create_expected["responseObjectType"]
        assert create_item.unique_identifier == extensions.UniqueIdentifier.integer(
            create_expected["uniqueIdentifier"]["integer"]
        )
        assert create_item.result_message is create_expected["resultMessage"]

        failure_expected = fixtures["creation.result-message-on-failure.v1"]
        failure_item = result.items[1]
        assert failure_item.operation == failure_expected["operation"]
        assert failure_item.outcome == failure_expected["outcomeKind"]
        assert failure_item.result_status == failure_expected["resultStatus"]
        assert failure_item.result_reason == failure_expected["resultReason"]
        assert failure_item.result_message == failure_expected["resultMessage"]
        assert failure_item.unique_identifier is None
        assert message_text not in repr(failure_item)
        assert message_text not in repr(failure_item.metadata_v2)

        key_pair_expected = fixtures["creation.key-pair.request-response.v1"]
        key_pair_item = result.items[2]
        assert key_pair_item.result_status == key_pair_expected["resultStatus"]
        assert key_pair_item.result_message is key_pair_expected["resultMessage"]
        assert key_pair_item.private_key_unique_identifier == extensions.UniqueIdentifier.enumeration(
            key_pair_expected["privateUniqueIdentifier"]["enumeration"]
        )
        assert key_pair_expected["privateUniqueIdentifier"]["wireType"] == "Enumeration"
        assert key_pair_item.public_key_unique_identifier == extensions.UniqueIdentifier.text_string(
            key_pair_expected["publicUniqueIdentifier"]["text"]
        )
        assert key_pair_expected["publicUniqueIdentifier"]["wireType"] == "TextString"

        split_expected = fixtures["creation.split-key.request-response.v1"]
        split_item = result.items[3]
        wire_order = split_expected["responseUniqueIdentifiersInWireOrder"]
        assert split_item.split_key_unique_identifiers == tuple(
            {
                "TextString": extensions.UniqueIdentifier.text_string,
                "Enumeration": extensions.UniqueIdentifier.enumeration,
                "Integer": extensions.UniqueIdentifier.integer,
            }[item["wireType"]](item.get("text", item.get("enumeration", item.get("integer"))))
            for item in wire_order
        )
    finally:
        result.close()


def _fixture_attribute_structure(tags: list[str]) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    structure = ttlv.ttlv_structure_create()
    try:
        for raw_tag in tags:
            unvalidated_tag = ttlv.raw_tag_from_raw(int(raw_tag, 16))
            tag = None
            value = None
            item = None
            try:
                tag = ttlv.raw_tag_try_checked(unvalidated_tag)
                value = ttlv.ttlv_value_enumeration(0xDEADBEEF)
                item = ttlv.ttlv_item_create(tag, value, limits)
                structure = ttlv.ttlv_structure_with_item(structure, item, limits)
            finally:
                if item is not None:
                    item.close()
                if value is not None:
                    value.close()
                if tag is not None:
                    tag.close()
                unvalidated_tag.close()
        result = structure
        structure = None
        return result
    finally:
        if structure is not None:
            structure.close()
        limits.close()

    assert released == ["item", "item", "item", "item", "response"]


def test_create_split_key_fixture_success_and_pending_metadata(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Map success and Pending from OASIS KMIP v2.1 §6.1.10 Tables 193–195 and §5.1 Table 157."""
    fixture_path = Path(__file__).resolve().parents[3] / "tests/fixtures/api/shared-capability-parity.json"
    fixture_cases = json.loads(fixture_path.read_text(encoding="utf-8"))["cases"]
    case = next(
        case for case in fixture_cases if case["id"] == "creation.split-key.request-response.v1"
    )
    request = case["input"]
    expected = case["expectedOutput"]
    pending_expected = expected["pendingMetadata"]
    pending_response_identifiers = expected["pendingResponseUniqueIdentifiersInWireOrder"]
    assert request["attributesPresent"] is True
    assert request["protectionStorageMasks"]["present"] is True
    assert expected["primeFieldSizePresent"] is False
    assert expected["resultMessagePresencePreserved"] is True
    assert expected["automaticFollowUp"] is False
    assert expected["transportExchangeCount"] == 1
    unique_batch_item_ids = expected["uniqueBatchItemIds"]
    assert unique_batch_item_ids == ["split-success", "split-pending"]
    assert pending_response_identifiers == [
        {"wireType": "TextString", "text": "pending-part"}
    ]

    attributes = ttlv.ttlv_structure_create()
    masks = ttlv.ttlv_structure_create()
    split_requests: list[extensions.ClientBatchItem] = []
    native_make_handle = extensions._make_handle
    captured_request_arguments: list[tuple[object, ...]] = []
    captured_batch_item_ids: list[bytes] = []
    construction_complete = False
    split_item: extensions.ClientBatchItem | None = None
    identified_item: extensions.ClientBatchItem | None = None

    def capture_split_key_builder(
        c_type: str,
        function_name: str,
        arguments: tuple[object, ...],
        out_index: int | None = None,
        consumed_indices: tuple[int, ...] = (),
    ) -> object:
        if function_name == "kmipkit_client_batch_item_create_split_key":
            captured_request_arguments.append(arguments)
        elif function_name == "kmipkit_client_batch_item_with_unique_batch_item_id":
            captured_batch_item_ids.append(
                bytes(extensions.ffi.buffer(arguments[1], int(arguments[2])))
            )
        return native_make_handle(c_type, function_name, arguments, out_index, consumed_indices)

    monkeypatch.setattr(extensions, "_make_handle", capture_split_key_builder)
    try:
        for batch_item_id in unique_batch_item_ids:
            split_item = extensions.client_batch_item_create_split_key(
                request["objectType"],
                request["splitKeyParts"],
                request["splitKeyThreshold"],
                request["splitKeyMethod"],
                attributes,
                unique_identifier=extensions.UniqueIdentifier.integer(
                    request["uniqueIdentifier"]["integer"]
                ),
                prime_field_size=None,
                protection_storage_masks=masks,
            )
            identified_item = extensions.with_unique_batch_item_id(
                split_item, batch_item_id.encode("ascii")
            )
            split_item.close()
            split_item = None
            split_requests.append(identified_item)
            identified_item = None
        assert len(captured_request_arguments) == len(unique_batch_item_ids) == 2
        assert captured_batch_item_ids == [
            batch_item_id.encode("ascii") for batch_item_id in unique_batch_item_ids
        ]
        for builder_arguments in captured_request_arguments:
            assert builder_arguments[0] == request["objectType"]
            identifier_argument = builder_arguments[1]
            assert identifier_argument.kind == int(extensions.UniqueIdentifierKind.INTEGER)  # type: ignore[attr-defined]
            assert identifier_argument.integer_value == request["uniqueIdentifier"]["integer"]  # type: ignore[attr-defined]
            assert builder_arguments[2:5] == (
                request["splitKeyParts"],
                request["splitKeyThreshold"],
                request["splitKeyMethod"],
            )
            assert builder_arguments[5] == 0
            assert builder_arguments[6] == client.ffi.NULL
            assert builder_arguments[7] == 0
            assert builder_arguments[8] != client.ffi.NULL
            assert builder_arguments[9] != client.ffi.NULL
        construction_complete = True
    finally:
        attributes.close()
        masks.close()
        if split_item is not None:
            split_item.close()
        if identified_item is not None:
            identified_item.close()
        if not construction_complete:
            for split_request in split_requests:
                split_request.close()

    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_storage = [
        _opaque("kmipkit_client_batch_item_response_t") for _ in range(2)
    ]
    item_pointers = [pointer for pointer, _storage in item_storage]
    correlation_bytes = bytes.fromhex(pending_expected["correlationValueHex"])
    correlation = client.ffi.new("uint8_t[]", correlation_bytes)
    identifier_storage: list[object] = []
    batch_item_id_storage: list[object] = []
    released: list[str] = []

    def item_index(item: object) -> int:
        return item_pointers.index(item)

    def write_identifier(output: object, identifier: dict[str, object]) -> None:
        kind = str(identifier["wireType"])
        output.reserved = 0  # type: ignore[attr-defined]
        output.text_data = client.ffi.NULL  # type: ignore[attr-defined]
        output.text_length = 0  # type: ignore[attr-defined]
        output.enumeration_value = 0  # type: ignore[attr-defined]
        output.integer_value = 0  # type: ignore[attr-defined]
        if kind == "TextString":
            data = client.ffi.new("uint8_t[]", str(identifier["text"]).encode("utf-8"))
            identifier_storage.append(data)
            output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)  # type: ignore[attr-defined]
            output.text_data = data  # type: ignore[attr-defined]
            output.text_length = len(str(identifier["text"]).encode("utf-8"))  # type: ignore[attr-defined]
        elif kind == "Enumeration":
            output.kind = int(extensions.UniqueIdentifierKind.ENUMERATION)  # type: ignore[attr-defined]
            output.enumeration_value = int(identifier["enumeration"])  # type: ignore[attr-defined]
        else:
            assert kind == "Integer"
            output.kind = int(extensions.UniqueIdentifierKind.INTEGER)  # type: ignore[attr-defined]
            output.integer_value = int(identifier["integer"])  # type: ignore[attr-defined]

    def metadata(item: object, out: object) -> int:
        index = item_index(item)
        _initialize_metadata_v2(out)
        out.operation = int(pending_expected["sourceOperation"])  # type: ignore[attr-defined]
        out.outcome_kind = 2 if index == 1 else 1  # type: ignore[attr-defined]
        out.result_status = int(pending_expected["status"] if index == 1 else expected["resultStatus"])  # type: ignore[attr-defined]
        return 0

    def item_at(_response: object, index: int, out: object) -> int:
        out[0] = item_pointers[int(index)]  # type: ignore[index]
        return 0

    def absent_span(_item: object, present: object, value: object) -> int:
        present[0] = 0  # type: ignore[index]
        value.data = client.ffi.NULL  # type: ignore[attr-defined]
        value.length = 0  # type: ignore[attr-defined]
        return 0

    def batch_item_id(item: object, present: object, value: object) -> int:
        data = unique_batch_item_ids[item_index(item)].encode("ascii")
        storage = client.ffi.new("uint8_t[]", data)
        batch_item_id_storage.append(storage)
        present[0] = 1  # type: ignore[index]
        value.data = storage  # type: ignore[attr-defined]
        value.length = len(data)  # type: ignore[attr-defined]
        return 0

    def pending_correlation(item: object, present: object, value: object) -> int:
        if item_index(item) == 1:
            present[0] = 1  # type: ignore[index]
            value.data = correlation  # type: ignore[attr-defined]
            value.length = len(correlation_bytes)  # type: ignore[attr-defined]
            return 0
        return absent_span(item, present, value)

    def split_count(item: object, present: object, count: object) -> int:
        index = item_index(item)
        identifiers = (
            expected["responseUniqueIdentifiersInWireOrder"]
            if index == 0
            else pending_response_identifiers
        )
        present[0] = 1  # type: ignore[index]
        count[0] = len(identifiers)  # type: ignore[index]
        return 0

    def split_identifier(item: object, index: int, output: object) -> int:
        identifiers = (
            expected["responseUniqueIdentifiersInWireOrder"]
            if item_index(item) == 0
            else pending_response_identifiers
        )
        write_identifier(output, identifiers[int(index)])
        return 0

    creation_symbols = _no_creation_response_symbols()
    creation_symbols.update(
        {
            "kmipkit_client_batch_item_response_create_split_key_count": split_count,
            "kmipkit_client_batch_item_response_create_split_key_unique_identifier_at": split_identifier,
        }
    )
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    native_client_library = client.lib
    batch_items: list[object] = []
    owned_batch_items: list[object] = []
    execution_calls: list[bool] = []
    asynchronous_indicators: list[int] = []

    def release_batch_items() -> None:
        while owned_batch_items:
            native_client_library.kmipkit_client_batch_item_release(owned_batch_items.pop())

    def release_batch(_batch: object) -> None:
        release_batch_items()

    def add_item(_batch: object, item_slot: object) -> int:
        batch_items.append(item_slot[0])  # type: ignore[index]
        owned_batch_items.append(item_slot[0])  # type: ignore[index]
        item_slot[0] = client.ffi.NULL  # type: ignore[index]
        return 0

    def set_asynchronous_indicator(_batch: object, value: int) -> int:
        asynchronous_indicators.append(int(value))
        return 0

    def execute(
        _client: object,
        batch_slot: object,
        _limits: object,
        output: object,
        error_output: object,
    ) -> int:
        execution_calls.append(True)
        assert len(batch_items) == len(unique_batch_item_ids) == 2
        assert asynchronous_indicators == [2]
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        assert error_output[0] == client.ffi.NULL  # type: ignore[index]
        release_batch_items()
        return 0

    fake = SimpleNamespace(
        kmipkit_client_release=lambda _client: None,
        kmipkit_client_batch_release=release_batch,
        kmipkit_client_batch_add_item=add_item,
        kmipkit_client_batch_set_asynchronous_indicator=set_asynchronous_indicator,
        kmipkit_client_execute_with_error_ttlv_v1=execute,
        kmipkit_client_batch_response_release=lambda _response: released.append("response"),
        kmipkit_client_batch_item_response_release=lambda _item: released.append("item"),
        kmipkit_client_batch_response_item_count=lambda _response, count: count.__setitem__(0, 2) or 0,
        kmipkit_client_batch_response_item_at=item_at,
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=batch_item_id,
        kmipkit_client_batch_item_response_pending_correlation_value=pending_correlation,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
        **creation_symbols,
    )
    monkeypatch.setattr(client, "lib", fake)

    result = None
    try:
        batch.set_asynchronous_indicator(2)
        for split_request in split_requests:
            batch.add_item(split_request)
        result = handle.execute(batch, limits)
        assert len(result.items) == 2
        completed, pending = result.items
        assert completed.operation == pending_expected["sourceOperation"]
        assert completed.outcome == client.BatchOutcome.COMPLETED
        assert completed.result_status == expected["resultStatus"]
        assert completed.result_reason is None
        assert completed.result_message is expected["resultMessage"]
        assert completed.unique_batch_item_id == unique_batch_item_ids[0].encode("ascii")
        expected_identifiers = tuple(
            {
                "TextString": extensions.UniqueIdentifier.text_string,
                "Enumeration": extensions.UniqueIdentifier.enumeration,
                "Integer": extensions.UniqueIdentifier.integer,
            }[identifier["wireType"]](
                identifier.get("text", identifier.get("enumeration", identifier.get("integer")))
            )
            for identifier in expected["responseUniqueIdentifiersInWireOrder"]
        )
        expected_pending_identifiers = tuple(
            extensions.UniqueIdentifier.text_string(identifier["text"])
            for identifier in pending_response_identifiers
        )
        assert completed.split_key_unique_identifiers == expected_identifiers
        assert pending.operation == pending_expected["sourceOperation"]
        assert pending.outcome == client.BatchOutcome.PENDING
        assert pending.result_status == pending_expected["status"]
        assert pending.result_reason is None
        assert pending.result_message is pending_expected["resultMessage"]
        assert pending.unique_batch_item_id == unique_batch_item_ids[1].encode("ascii")
        assert pending.pending_correlation_value == correlation_bytes
        assert pending.split_key_unique_identifiers == expected_pending_identifiers
        assert pending_expected["correlationValueHex"] == "00ff80"
        assert len(batch_items) == len(unique_batch_item_ids) == 2
        assert asynchronous_indicators == [2]
        assert len(execution_calls) == expected["transportExchangeCount"]
    finally:
        if result is not None:
            result.close()
        for split_request in split_requests:
            split_request.close()
        batch.close()
        handle.close()
        limits.close()

    assert released == ["item", "item", "response"]


def test_create_split_key_response_preserves_more_than_one_hundred_thousand_identifiers(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    identifier_count = 100_001
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)
    released: list[str] = []

    def split_count(_item: object, present: object, count: object) -> int:
        present[0] = 1  # type: ignore[index]
        count[0] = identifier_count  # type: ignore[index]
        return 0

    def split_identifier(_item: object, index: int, output: object) -> int:
        output.struct_size = client.ffi.sizeof("kmipkit_unique_identifier_v1_t")  # type: ignore[attr-defined]
        output.abi_version = 1  # type: ignore[attr-defined]
        output.kind = int(extensions.UniqueIdentifierKind.INTEGER)  # type: ignore[attr-defined]
        output.reserved = 0  # type: ignore[attr-defined]
        output.text_data = client.ffi.NULL  # type: ignore[attr-defined]
        output.text_length = 0  # type: ignore[attr-defined]
        output.enumeration_value = 0  # type: ignore[attr-defined]
        output.integer_value = index  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: released.append("item"),
            kmipkit_client_batch_item_response_create_split_key_count=split_count,
            kmipkit_client_batch_item_response_create_split_key_unique_identifier_at=(
                split_identifier
            ),
        ),
    )
    try:
        identifiers = client._create_split_key_response(item)
        assert identifiers is not None
        assert len(identifiers) == identifier_count
        assert identifiers[0] == extensions.UniqueIdentifier.integer(0)
        assert identifiers[50_000] == extensions.UniqueIdentifier.integer(50_000)
        assert identifiers[-1] == extensions.UniqueIdentifier.integer(identifier_count - 1)
    finally:
        item.close()
    assert released == ["item"]


def test_create_split_key_response_rejects_count_above_python_collection_limit(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)
    accessed_indices: list[int] = []

    def split_count(_item: object, present: object, count: object) -> int:
        present[0] = 1  # type: ignore[index]
        count[0] = sys.maxsize + 1  # type: ignore[index]
        return 0

    def split_identifier(_item: object, index: int, _output: object) -> int:
        accessed_indices.append(index)
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_create_split_key_count=split_count,
            kmipkit_client_batch_item_response_create_split_key_unique_identifier_at=(
                split_identifier
            ),
        ),
    )
    try:
        with pytest.raises(errors.ResourceLimitError):
            client._create_split_key_response(item)
    finally:
        item.close()
    assert accessed_indices == []


def test_batch_result_accepts_absent_spans_and_no_result_reason(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")

    def metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
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
        kmipkit_client_batch_item_response_metadata_v2=metadata,
        kmipkit_client_batch_item_response_unique_batch_item_id=absent,
        kmipkit_client_batch_item_response_pending_correlation_value=absent,
        kmipkit_client_batch_item_response_discover_versions_supported_versions=_no_supported_versions,
        **_no_creation_response_symbols(),
    )
    monkeypatch.setattr(client, "lib", fake)
    result = client._batch_result(response_pointer)
    assert result.items[0].outcome is client.BatchOutcome.COMPLETED
    assert result.items[0].result_reason is None
    assert result.items[0].unique_batch_item_id is None
    assert result.items[0].pending_correlation_value is None
    result.close()


def test_certify_response_rejects_identifier_when_response_is_absent(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)

    def absent_response_with_identifier(_item: object, present: object, identifier: object) -> int:
        present[0] = 0  # type: ignore[index]
        identifier.kind = 2  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_certify=absent_response_with_identifier,
        ),
    )
    try:
        with pytest.raises(errors.InvalidInputError):
            client._certify_response(item)
    finally:
        item.close()


def test_discover_versions_response_accepts_more_than_default_codec_elements(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    supported_count = 100_001

    def supported_versions(
        _item: object, present: object, output: object, capacity: int, count: object
    ) -> int:
        present[0] = 1  # type: ignore[index]
        count[0] = supported_count  # type: ignore[index]
        if capacity:
            for index in range(capacity):
                output[index].major = 2  # type: ignore[index,attr-defined]
                output[index].minor = 1  # type: ignore[index,attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_discover_versions_supported_versions=(
                supported_versions
            )
        ),
    )
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)

    try:
        versions = client._discover_versions_supported_versions(item)
        assert versions is not None
        assert len(versions) == supported_count
        assert versions[0] == client.ProtocolVersion(2, 1)
        assert versions[-1] == client.ProtocolVersion(2, 1)
    finally:
        item.close()


@pytest.mark.parametrize(
    ("present_value", "count_value"),
    [(0, 1), (2, 0), (1, (1 << 64) - 1)],
)
def test_discover_versions_rejects_inconsistent_first_pass_metadata(
    monkeypatch: pytest.MonkeyPatch, present_value: int, count_value: int
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")

    def malformed_versions(
        _item: object, present: object, _output: object, _capacity: int, count: object
    ) -> int:
        present[0] = present_value  # type: ignore[index]
        count[0] = count_value  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_discover_versions_supported_versions=(
                malformed_versions
            ),
        ),
    )
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)
    try:
        with pytest.raises(errors.KmipKitError):
            client._discover_versions_supported_versions(item)
    finally:
        item.close()


def test_discover_versions_rejects_changed_second_pass_metadata(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    calls = 0

    def changing_versions(
        _item: object, present: object, _output: object, capacity: int, count: object
    ) -> int:
        nonlocal calls
        calls += 1
        present[0] = 1 if capacity == 0 else 0  # type: ignore[index]
        count[0] = 1  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_discover_versions_supported_versions=(
                changing_versions
            ),
        ),
    )
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)
    try:
        with pytest.raises(errors.InvalidInputError):
            client._discover_versions_supported_versions(item)
        assert calls == 2
    finally:
        item.close()


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
        kmipkit_client_batch_item_response_metadata_v2=wrong_metadata,
    )
    monkeypatch.setattr(client, "lib", wrong_abi)
    with pytest.raises(errors.InvalidInputError):
        client._batch_result(response_pointer)
    assert released[-2:] == ["item", "response"]

    def valid_metadata(_item: object, out: object) -> int:
        _initialize_metadata_v2(out)
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
        kmipkit_client_batch_item_response_metadata_v2=valid_metadata,
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


def test_client_execution_error_retains_owned_generic_response_without_disclosing_it() -> None:
    response = ttlv.ttlv_structure_create()
    try:
        failure = errors.ClientExecutionError(
            status=8,
            category=errors.ClientErrorCategory.PROTOCOL,
            cause=errors.ClientErrorCause.PROTOCOL_OTHER,
            delivery=errors.DeliveryState.RESPONSE_STARTED,
            response_ttlv=response,
        )

        assert failure.response_ttlv is response
        assert "KMIPKit-response-sentinel" not in str(failure)
        assert "KMIPKit-response-sentinel" not in repr(failure)
        assert failure.args == (str(failure),)
        failure.response_ttlv.close()
        assert failure.response_ttlv.closed
    finally:
        response.close()

    absent = errors.ClientExecutionError(
        status=9,
        category=errors.ClientErrorCategory.TRANSPORT,
        cause=errors.ClientErrorCause.TRANSPORT_IO,
        delivery=errors.DeliveryState.POSSIBLY_SENT,
    )
    assert absent.response_ttlv is None


def _take_client_compatible_structure() -> tuple[ttlv.Structure, object, object, object | None]:
    response = ttlv.ttlv_structure_create()
    ttlv_pointer, owner = response._take()
    address = int(ttlv.ffi.cast("uintptr_t", ttlv_pointer))
    client_pointer = client.ffi.cast("kmipkit_ttlv_structure_t *", address)
    return response, ttlv_pointer, client_pointer, owner


def test_client_status_releases_error_response_on_success_status(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response, ttlv_pointer, pointer, _owner = _take_client_compatible_structure()
    released: list[object] = []

    def release_response(raw_pointer: object) -> None:
        released.append(raw_pointer)

    monkeypatch.setattr(client, "lib", SimpleNamespace(kmipkit_ttlv_structure_release=release_response))

    try:
        with pytest.raises(errors.KmipKitError):
            client._raise_client_status(0, pointer)

        assert response.closed
        assert len(released) == 1
    finally:
        ttlv.lib.kmipkit_ttlv_structure_release(ttlv_pointer)


def test_client_status_releases_non_protocol_error_response(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response, ttlv_pointer, pointer, _owner = _take_client_compatible_structure()
    released: list[object] = []

    monkeypatch.setattr(
        client,
        "_read_last_error",
        lambda: (
            8,
            int(errors.ClientErrorCategory.TRANSPORT),
            int(errors.ClientErrorCause.TRANSPORT_IO),
            int(errors.DeliveryState.RESPONSE_STARTED),
        ),
    )

    def release_response(raw_pointer: object) -> None:
        released.append(raw_pointer)

    monkeypatch.setattr(client, "lib", SimpleNamespace(kmipkit_ttlv_structure_release=release_response))

    try:
        with pytest.raises(errors.ClientExecutionError) as caught:
            client._raise_client_status(8, pointer)

        assert caught.value.response_ttlv is None
        assert response.closed
        assert len(released) == 1
    finally:
        ttlv.lib.kmipkit_ttlv_structure_release(ttlv_pointer)


def test_client_status_does_not_retain_ttlv_for_ordinary_protocol_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setattr(
        client,
        "_read_last_error",
        lambda: (
            8,
            int(errors.ClientErrorCategory.PROTOCOL),
            int(errors.ClientErrorCause.PROTOCOL_OTHER),
            int(errors.DeliveryState.RESPONSE_STARTED),
        ),
    )

    with pytest.raises(errors.ClientExecutionError) as caught:
        client._raise_client_status(8, ttlv.ffi.NULL)

    assert caught.value.response_ttlv is None


def test_client_status_releases_response_when_protocol_adoption_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response, ttlv_pointer, pointer, _owner = _take_client_compatible_structure()
    released: list[object] = []

    monkeypatch.setattr(
        client,
        "_read_last_error",
        lambda: (
            8,
            int(errors.ClientErrorCategory.PROTOCOL),
            int(errors.ClientErrorCause.PROTOCOL_OTHER),
            int(errors.DeliveryState.RESPONSE_STARTED),
        ),
    )

    def fail_adoption(_address: int) -> ttlv.Structure:
        raise ValueError("test adoption failure")

    def release_response(raw_pointer: object) -> None:
        released.append(raw_pointer)

    monkeypatch.setattr(ttlv, "_structure_from_native_address", fail_adoption)
    monkeypatch.setattr(client, "lib", SimpleNamespace(kmipkit_ttlv_structure_release=release_response))

    try:
        with pytest.raises(errors.KmipKitError):
            client._raise_client_status(8, pointer)

        assert response.closed
        assert len(released) == 1
    finally:
        ttlv.lib.kmipkit_ttlv_structure_release(ttlv_pointer)


def test_client_status_closes_adopted_response_if_error_construction_fails(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    response, ttlv_pointer, pointer, _owner = _take_client_compatible_structure()
    adopted: list[ttlv.Structure] = []
    adopt = ttlv._structure_from_native_address

    monkeypatch.setattr(
        client,
        "_read_last_error",
        lambda: (
            8,
            int(errors.ClientErrorCategory.PROTOCOL),
            int(errors.ClientErrorCause.PROTOCOL_OTHER),
            int(errors.DeliveryState.RESPONSE_STARTED),
        ),
    )

    def adopt_response(address: int) -> ttlv.Structure:
        adopted_response = adopt(address)
        adopted.append(adopted_response)
        return adopted_response

    def fail_error_construction(**_values: object) -> errors.ClientExecutionError:
        raise MemoryError("test error-construction failure")

    monkeypatch.setattr(ttlv, "_structure_from_native_address", adopt_response)
    monkeypatch.setattr(errors, "ClientExecutionError", fail_error_construction)

    try:
        with pytest.raises(MemoryError, match="test error-construction failure"):
            client._raise_client_status(8, pointer)

        assert len(adopted) == 1
        assert adopted[0].closed
        assert response.closed
    finally:
        if not adopted or not adopted[0].closed:
            ttlv.lib.kmipkit_ttlv_structure_release(ttlv_pointer)


def test_staging_wipe_accepts_absent_buffers() -> None:
    _wipe_staging_buffer(None)
    _wipe_staging_buffer(ttlv.ffi.NULL)


def test_structure_from_native_address_rejects_null_address() -> None:
    with pytest.raises(errors.InvalidInputError):
        ttlv._structure_from_native_address(0)


@pytest.mark.parametrize("options_enabled", [False, True])
def test_client_execution_adopts_approved_uid_shape_error_response_for_both_facades(
    monkeypatch: pytest.MonkeyPatch, options_enabled: bool
) -> None:
    sentinel = b"KMIPKit-response-sentinel"
    response = _approved_uid_shape_error_response_ttlv(sentinel)
    response_pointer, _response_owner = response._take()
    response_address = int(ttlv.ffi.cast("uintptr_t", response_pointer))
    client_pointer, _client_storage = _opaque("kmipkit_client_t")
    batch_pointer, _batch_storage = _opaque("kmipkit_client_batch_t")
    handle = _wrap_owned_handle(client.Client, client_pointer)
    batch = _wrap_owned_handle(client.Batch, batch_pointer)
    limits = ttlv.codec_limits_defaults()
    timeout = client.TimeoutLimit.default()
    options = client.RequestOptions(timeout, timeout, timeout, timeout)

    def execute(*arguments: object) -> int:
        batch_slot = arguments[1]
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        error_output = arguments[-1]
        error_output[0] = client.ffi.cast(  # type: ignore[index]
            "kmipkit_ttlv_structure_t *", response_address
        )
        return 8

    def last_error(status: object, category: object, cause: object, delivery: object) -> int:
        status[0] = 8  # type: ignore[index]
        category[0] = int(errors.ClientErrorCategory.PROTOCOL)  # type: ignore[index]
        cause[0] = int(errors.ClientErrorCause.PROTOCOL_INVALID_VALUE)  # type: ignore[index]
        delivery[0] = int(errors.DeliveryState.RESPONSE_STARTED)  # type: ignore[index]
        return 0

    function_name = (
        "kmipkit_client_execute_with_options_and_error_ttlv_v1"
        if options_enabled
        else "kmipkit_client_execute_with_error_ttlv_v1"
    )
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            **{
                function_name: execute,
                "kmipkit_last_error_get": last_error,
                "kmipkit_client_release": lambda _ptr: None,
                "kmipkit_client_batch_release": lambda _ptr: None,
            }
        ),
    )
    try:
        with pytest.raises(errors.ClientExecutionError) as caught:
            if options_enabled:
                handle.execute_with_options(batch, limits, options)
            else:
                handle.execute(batch, limits)

        failure = caught.value
        assert batch.closed
        assert failure.response_ttlv is not None
        assert "KMIPKit-response-sentinel" not in str(failure)
        assert "KMIPKit-response-sentinel" not in repr(failure)
        with ttlv.ttlv_structure_view(failure.response_ttlv) as root:
            assert _contains_ttlv_byte_string(root, 0x540001, sentinel)
        failure.response_ttlv.close()
        assert failure.response_ttlv.closed
    finally:
        limits.close()
        batch.close()
        handle.close()


def _approved_uid_shape_error_response_ttlv(sentinel: bytes) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()

    def item(tag_value: int, value: ttlv.Value) -> ttlv.TtlvItem:
        raw_tag = ttlv.raw_tag_from_raw(tag_value)
        tag = ttlv.raw_tag_try_checked(raw_tag)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()

    def integer_item(tag_value: int, value: int) -> ttlv.TtlvItem:
        return item(tag_value, ttlv.ttlv_value_integer(value))

    def enumeration_item(tag_value: int, value: int) -> ttlv.TtlvItem:
        return item(tag_value, ttlv.ttlv_value_enumeration(value))

    def bytes_item(tag_value: int, value: bytes) -> ttlv.TtlvItem:
        return item(tag_value, ttlv.ttlv_value_byte_string(value, limits))

    def text_item(tag_value: int, value: str) -> ttlv.TtlvItem:
        return item(
            tag_value,
            ttlv.ttlv_value_text_string(value.encode("utf-8"), limits),
        )

    def structure_item(tag_value: int, items: tuple[ttlv.TtlvItem, ...]) -> ttlv.TtlvItem:
        structure = ttlv.ttlv_structure_create()
        for child in items:
            structure = ttlv.ttlv_structure_with_item(structure, child, limits)
        return item(tag_value, ttlv.ttlv_value_structure(structure, limits))

    try:
        protocol_version = structure_item(
            0x420069,
            (integer_item(0x42006A, 2), integer_item(0x42006B, 1)),
        )
        header = structure_item(
            0x42007A,
            (
                protocol_version,
                item(0x420092, ttlv.ttlv_value_long_integer(1)),
                integer_item(0x42000D, 1),
            ),
        )
        payload = structure_item(
            0x42007C,
            (
                bytes_item(0x4200C4, b"\x5a"),
                bytes_item(0x540001, sentinel),
            ),
        )
        response_item = structure_item(
            0x42000F,
            (
                enumeration_item(0x42005C, 35),
                text_item(0x420093, "item-0"),
                enumeration_item(0x42007F, 0),
                payload,
            ),
        )
        root = ttlv.ttlv_structure_create()
        root = ttlv.ttlv_structure_with_item(root, header, limits)
        root = ttlv.ttlv_structure_with_item(root, response_item, limits)
        return root
    finally:
        limits.close()


def _contains_ttlv_byte_string(
    structure: ttlv.TtlvStructureView, raw_tag: int, expected: bytes
) -> bool:
    for index in range(ttlv.ttlv_structure_view_item_count(structure)):
        item_view = ttlv.ttlv_structure_view_item_at(structure, index)
        try:
            tag = ttlv.ttlv_item_view_tag(item_view)
            try:
                item_tag = ttlv.tag_value(tag)
            finally:
                tag.close()
            item_type = ttlv.ttlv_item_view_type(item_view)
            value = ttlv.ttlv_item_view_value(item_view)
            try:
                if item_tag == raw_tag and item_type is ttlv.ItemType.ByteString:
                    actual = bytes(
                        ttlv.ttlv_value_view_byte_at(value, byte_index)
                        for byte_index in range(ttlv.ttlv_value_view_byte_length(value))
                    )
                    if actual == expected:
                        return True
                if item_type is ttlv.ItemType.Structure:
                    nested = ttlv.ttlv_value_view_structure(value)
                    try:
                        if _contains_ttlv_byte_string(nested, raw_tag, expected):
                            return True
                    finally:
                        nested.close()
            finally:
                value.close()
        finally:
            item_view.close()
    return False


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
        error_output: object,
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
        assert error_output[0] == client.ffi.NULL  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_with_options_and_error_ttlv_v1=execute,
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
            kmipkit_client_execute_with_options_and_error_ttlv_v1=execute,
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
            kmipkit_client_execute_with_options_and_error_ttlv_v1=lambda *_arguments: 0,
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
        kmipkit_client_execute_with_error_ttlv_v1=lambda *_args: (_ for _ in ()).throw(TypeError("secret")),
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
        kmipkit_client_execute_with_error_ttlv_v1=lambda *_arguments: status_code,
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

    def execute(
        _client: object,
        batch_slot: object,
        _limits: object,
        output: object,
        error_output: object,
    ) -> int:
        batch_slot[0] = client.ffi.NULL  # type: ignore[index]
        output[0] = response_pointer  # type: ignore[index]
        assert error_output[0] == client.ffi.NULL  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_with_error_ttlv_v1=execute,
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


@pytest.mark.parametrize(
    ("indicator", "expected_symbol"),
    (
        (None, "kmipkit_client_execute_guarded_protection_item"),
        (
            0xFEDC_BA98,
            "kmipkit_client_execute_guarded_protection_item_with_asynchronous_indicator",
        ),
    ),
)
def test_guarded_protection_facade_consumes_item_and_preserves_cost_and_indicator(
    monkeypatch: pytest.MonkeyPatch,
    indicator: int | None,
    expected_symbol: str,
) -> None:
    native_client_pointer = client.ffi.cast("kmipkit_client_t *", 31)
    native_context_pointer = client.ffi.cast(
        "kmipkit_usage_allocation_context_t *", 32
    )
    native_response_pointer = client.ffi.cast(
        "kmipkit_client_batch_response_t *", 33
    )
    handle = _wrap_owned_handle(client.Client, native_client_pointer)
    context = _wrap_owned_handle(client.UsageAllocationContext, native_context_pointer)
    item = extensions.client_batch_item_encrypt(
        extensions.UniqueIdentifier.text_string("allocation-object")
    )
    limits = ttlv.codec_limits_defaults()
    observed: list[tuple[object, ...]] = []
    released: list[object] = []

    def execute(*arguments: object) -> int:
        if indicator is None:
            raw_client, raw_context, item_slot, cost, _limits, response_out = arguments
            raw_indicator = None
        else:
            raw_client, raw_context, item_slot, cost, raw_indicator, _limits, response_out = arguments
        native_item = item_slot[0]  # type: ignore[index]
        observed.append((raw_client, raw_context, native_item, cost, raw_indicator))
        extensions._client_lib.kmipkit_client_batch_item_release(native_item)
        item_slot[0] = client.ffi.NULL  # type: ignore[index]
        response_out[0] = native_response_pointer  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            **{
                expected_symbol: execute,
                "kmipkit_client_release": lambda value: released.append(value),
                "kmipkit_usage_allocation_context_release": lambda value: released.append(value),
            }
        ),
    )
    def batch_result(value: object, result_limits: ttlv.CodecLimits) -> object:
        assert value == native_response_pointer
        assert result_limits is limits
        return value

    monkeypatch.setattr(client, "_batch_result", batch_result)

    result = (
        handle.execute_guarded_protection_item(context, item, 9, limits)
        if indicator is None
        else handle.execute_guarded_protection_item_with_asynchronous_indicator(
            context, item, 9, indicator, limits
        )
    )

    assert result == native_response_pointer
    assert len(observed) == 1
    assert observed[0][:2] == (native_client_pointer, native_context_pointer)
    assert observed[0][3:] == (9, indicator)
    assert item.closed
    context.close()
    handle.close()
    limits.close()
    assert released == [native_context_pointer, native_client_pointer]


@pytest.mark.parametrize("cost", (0, -1))
def test_guarded_protection_rejects_nonpositive_cost_before_consuming_item(
    monkeypatch: pytest.MonkeyPatch, cost: int
) -> None:
    native_client_pointer = client.ffi.cast("kmipkit_client_t *", 41)
    native_context_pointer = client.ffi.cast(
        "kmipkit_usage_allocation_context_t *", 42
    )
    handle = _wrap_owned_handle(client.Client, native_client_pointer)
    context = _wrap_owned_handle(client.UsageAllocationContext, native_context_pointer)
    item = extensions.client_batch_item_encrypt(
        extensions.UniqueIdentifier.text_string("allocation-object")
    )
    limits = ttlv.codec_limits_defaults()
    released: list[object] = []
    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_release=lambda value: released.append(value),
            kmipkit_usage_allocation_context_release=lambda value: released.append(value),
            kmipkit_client_execute_guarded_protection_item=lambda *_args: pytest.fail(
                "invalid local cost must be rejected before native execution"
            ),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        handle.execute_guarded_protection_item(context, item, cost, limits)

    assert not item.closed
    item.close()
    context.close()
    handle.close()
    limits.close()
    assert released == [native_context_pointer, native_client_pointer]


@pytest.mark.parametrize(
    ("mode", "expected_error", "item_consumed"),
    (
        pytest.param("wrong-context", errors.InvalidInputError, False, id="wrong-context"),
        pytest.param("conversion-error", errors.InvalidInputError, False, id="conversion-error"),
        pytest.param("not-consumed", errors.KmipKitError, False, id="not-consumed"),
        pytest.param("null-response", errors.KmipKitError, True, id="null-response"),
    ),
)
def test_guarded_protection_failures_preserve_item_ownership(
    monkeypatch: pytest.MonkeyPatch,
    mode: str,
    expected_error: type[Exception],
    item_consumed: bool,
) -> None:
    native_client_pointer = client.ffi.cast("kmipkit_client_t *", 51)
    native_context_pointer = client.ffi.cast(
        "kmipkit_usage_allocation_context_t *", 52
    )
    handle = _wrap_owned_handle(client.Client, native_client_pointer)
    context = _wrap_owned_handle(client.UsageAllocationContext, native_context_pointer)
    item = extensions.client_batch_item_encrypt(
        extensions.UniqueIdentifier.text_string("allocation-object")
    )
    limits = ttlv.codec_limits_defaults()
    released: list[object] = []

    def execute(*arguments: object) -> int:
        _client, _context, item_slot, _cost, _limits, _response = arguments
        if mode == "conversion-error":
            raise TypeError("native argument conversion failed")
        if mode == "not-consumed":
            return 0
        native_item = item_slot[0]  # type: ignore[index]
        extensions._client_lib.kmipkit_client_batch_item_release(native_item)
        item_slot[0] = client.ffi.NULL  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_execute_guarded_protection_item=execute,
            kmipkit_client_release=lambda value: released.append(value),
            kmipkit_usage_allocation_context_release=lambda value: released.append(value),
        ),
    )

    actual_context = object() if mode == "wrong-context" else context
    try:
        with pytest.raises(expected_error):
            handle.execute_guarded_protection_item(actual_context, item, 9, limits)  # type: ignore[arg-type]
        assert item.closed is item_consumed
    finally:
        if not item.closed:
            item.close()
        context.close()
        handle.close()
        limits.close()

    assert released == [native_context_pointer, native_client_pointer]


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
    handle = _wrap_owned_handle(client.Client, client_pointer)
    limits = ttlv.codec_limits_defaults()
    wrong_limits_batch = _wrap_owned_handle(client.Batch, batch_pointer)
    batch_pointer_2, _batch_storage_2 = _opaque("kmipkit_client_batch_t")
    batch = _wrap_owned_handle(client.Batch, batch_pointer_2)
    try:
        with pytest.raises(errors.InvalidInputError):
            handle.execute(object(), limits)  # type: ignore[arg-type]
        with pytest.raises(errors.InvalidInputError):
            handle.execute(wrong_limits_batch, object())  # type: ignore[arg-type]
        assert not wrong_limits_batch.closed
        wrong_limits_batch.close()
        monkeypatch.setattr(
            client,
            "lib",
            SimpleNamespace(
                kmipkit_client_execute_with_error_ttlv_v1=lambda *_arguments: 0,
                kmipkit_client_release=lambda pointer: released_client_pointers.append(pointer),
                kmipkit_client_batch_release=lambda pointer: released_batch_pointers.append(pointer),
            ),
        )
        with pytest.raises(errors.KmipKitError):
            handle.execute(batch, limits)
        assert not batch.closed
    finally:
        wrong_limits_batch.close()
        batch.close()
        limits.close()
        handle.close()
    assert released_batch_pointers == [batch_pointer, batch_pointer_2]
    assert released_client_pointers == [client_pointer]


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
        kmipkit_client_execute_with_error_ttlv_v1=lambda *_arguments: pytest.fail(
            "closed client inputs must fail before native execution"
        ),
        kmipkit_client_execute_with_options_and_error_ttlv_v1=lambda *_arguments: pytest.fail(
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
        _initialize_metadata_v2(out)
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
        kmipkit_client_batch_item_response_metadata_v2=metadata,
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


@pytest.mark.parametrize(
    ("kind", "text", "number"),
    (
        (0, None, None),
        (extensions.UniqueIdentifierKind.TEXT_STRING, None, None),
        (extensions.UniqueIdentifierKind.TEXT_STRING, "id", 1),
        (extensions.UniqueIdentifierKind.ENUMERATION, "id", 1),
        (extensions.UniqueIdentifierKind.ENUMERATION, None, None),
        (extensions.UniqueIdentifierKind.INTEGER, "id", 1),
        (extensions.UniqueIdentifierKind.INTEGER, None, None),
    ),
    ids=(
        "unknown-kind",
        "missing-text",
        "text-and-number",
        "enum-with-text",
        "missing-enum",
        "integer-with-text",
        "missing-integer",
    ),
)
def test_unique_identifier_rejects_inconsistent_python_values(
    kind: object, text: str | None, number: int | None
) -> None:
    with pytest.raises(errors.InvalidInputError):
        extensions.UniqueIdentifier(kind, text, number)  # type: ignore[arg-type]


def test_unique_identifier_repr_redacts_text_value() -> None:
    identifier = extensions.UniqueIdentifier.text_string("secret-identifier-α")

    assert "value=redacted" in repr(identifier)
    assert "secret-identifier-α" not in repr(identifier)


def test_native_unique_identifier_preserves_wire_kind_and_copies_text() -> None:
    null_record, null_owner = extensions._native_unique_identifier(None)
    assert null_record == extensions.ffi.NULL
    assert null_owner is None

    text = "key-α"
    text_record, text_owner = extensions._native_unique_identifier(
        extensions.UniqueIdentifier.text_string(text)
    )
    assert text_owner is not None
    assert text_record.abi_version == 1
    assert text_record.kind == int(extensions.UniqueIdentifierKind.TEXT_STRING)
    assert text_record.text_length == len(b"key-\xce\xb1")
    assert bytes(extensions.ffi.buffer(text_record.text_data, text_record.text_length)) == (
        b"key-\xce\xb1"
    )

    enum_record, enum_owner = extensions._native_unique_identifier(
        extensions.UniqueIdentifier.enumeration(0xFFFF_FFFF)
    )
    assert enum_record.kind == int(extensions.UniqueIdentifierKind.ENUMERATION)
    assert enum_record.text_data == extensions.ffi.NULL
    assert enum_record.text_length == 0
    assert enum_record.enumeration_value == 0xFFFF_FFFF
    assert enum_record.integer_value == 0
    assert enum_owner is None

    integer_record, integer_owner = extensions._native_unique_identifier(
        extensions.UniqueIdentifier.integer(-7)
    )
    assert integer_record.kind == int(extensions.UniqueIdentifierKind.INTEGER)
    assert integer_record.text_data == extensions.ffi.NULL
    assert integer_record.text_length == 0
    assert integer_record.enumeration_value == 0
    assert integer_record.integer_value == -7
    assert integer_owner is None

    with pytest.raises(errors.InvalidInputError):
        extensions._native_unique_identifier("not-an-identifier")  # type: ignore[arg-type]


def test_native_structure_checks_required_and_typed_inputs() -> None:
    assert extensions._native_structure(None) == extensions.ffi.NULL
    with pytest.raises(errors.InvalidInputError):
        extensions._native_structure(None, required=True)
    with pytest.raises(errors.InvalidInputError):
        extensions._native_structure(object())  # type: ignore[arg-type]

    structure = ttlv.ttlv_structure_create()
    try:
        assert extensions._native_structure(structure) == structure._pointer()
    finally:
        structure.close()


@pytest.mark.parametrize(
    ("prime_field_size", "expected_bytes"),
    (
        (b"\x01\x02", b"\x01\x02"),
        (bytearray(b"\x03"), b"\x03"),
        (memoryview(b"\x04"), b"\x04"),
        (b"", b""),
    ),
    ids=("bytes", "bytearray", "memoryview", "empty"),
)
def test_split_key_prime_field_is_copied_to_native_input(
    monkeypatch: pytest.MonkeyPatch,
    prime_field_size: bytes | bytearray | memoryview,
    expected_bytes: bytes,
) -> None:
    attributes = ttlv.ttlv_structure_create()
    captured: list[tuple[object, ...]] = []
    observed_prime_bytes: list[bytes] = []
    sentinel = object()

    def make_handle(
        _c_type: str,
        _function_name: str,
        arguments: tuple[object, ...],
        *_args: object,
        **_kwargs: object,
    ) -> object:
        captured.append(arguments)
        prime_data = arguments[6]
        prime_length = int(arguments[7])
        observed_prime_bytes.append(
            bytes(extensions.ffi.buffer(prime_data, prime_length))
            if prime_length
            else b""
        )
        return sentinel

    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, handle: handle)
    try:
        item = extensions.client_batch_item_create_split_key(
            7,
            2,
            1,
            0,
            attributes,
            prime_field_size=prime_field_size,
        )
        assert item is sentinel
        arguments = captured[0]
        assert arguments[5] == 1
        assert arguments[7] == len(expected_bytes)
        assert observed_prime_bytes == [expected_bytes]
        if not expected_bytes:
            assert arguments[6] == extensions.ffi.NULL
    finally:
        attributes.close()


def test_split_key_prime_field_rejects_non_byte_inputs() -> None:
    attributes = ttlv.ttlv_structure_create()
    try:
        with pytest.raises(errors.InvalidInputError):
            extensions.client_batch_item_create_split_key(
                7,
                2,
                1,
                0,
                attributes,
                prime_field_size="not-bytes",  # type: ignore[arg-type]
            )
    finally:
        attributes.close()


@pytest.mark.parametrize(
    "malformation",
    (
        "struct-size",
        "abi-version",
        "reserved",
        "text-too-large",
        "text-null-data",
        "text-invalid-utf8",
        "enum-extra-text-length",
        "integer-extra-enum-value",
        "unknown-kind",
    ),
)
def test_read_unique_identifier_rejects_malformed_native_records(
    malformation: str,
) -> None:
    output = client._initialize_unique_identifier_output()
    storage: object | None = None
    if malformation == "struct-size":
        output.struct_size = 0
    elif malformation == "abi-version":
        output.abi_version = 2
    elif malformation == "reserved":
        output.reserved = 1
    elif malformation in ("text-too-large", "text-null-data", "text-invalid-utf8"):
        output.kind = int(extensions.UniqueIdentifierKind.TEXT_STRING)
        if malformation == "text-too-large":
            output.text_length = client._MAX_INPUT_BYTES + 1
        elif malformation == "text-null-data":
            output.text_length = 1
        else:
            storage = client.ffi.new("uint8_t[]", b"\xff")
            output.text_data = storage
            output.text_length = 1
    elif malformation == "enum-extra-text-length":
        output.kind = int(extensions.UniqueIdentifierKind.ENUMERATION)
        output.text_length = 1
    elif malformation == "integer-extra-enum-value":
        output.kind = int(extensions.UniqueIdentifierKind.INTEGER)
        output.enumeration_value = 1
    else:
        output.kind = 99

    with pytest.raises(errors.InvalidInputError):
        client._read_unique_identifier(output)
    if storage is not None:
        assert output.text_data == storage


@pytest.mark.parametrize(
    "metadata",
    (
        SimpleNamespace(
            has_result_message=0,
            result_message_data=client.ffi.NULL,
            result_message_length=1,
        ),
        SimpleNamespace(
            has_result_message=0,
            result_message_data=client.ffi.NULL,
            result_message_length=0,
        ),
    ),
    ids=("absent-with-length", "absent-valid"),
)
def test_metadata_result_message_checks_absent_pointer_and_length_consistency(
    metadata: object,
) -> None:
    if metadata.result_message_length == 0:
        assert client._metadata_result_message(metadata) is None
    else:
        with pytest.raises(errors.InvalidInputError):
            client._metadata_result_message(metadata)


def test_metadata_result_message_rejects_oversize_and_null_data() -> None:
    with pytest.raises(errors.ResourceLimitError):
        client._metadata_result_message(
            SimpleNamespace(
                has_result_message=1,
                result_message_data=client.ffi.NULL,
                result_message_length=client._MAX_INPUT_BYTES + 1,
            )
        )
    with pytest.raises(errors.InvalidInputError):
        client._metadata_result_message(
            SimpleNamespace(
                has_result_message=1,
                result_message_data=client.ffi.NULL,
                result_message_length=1,
            )
        )


@pytest.mark.parametrize("malformation", ("reserved", "presence", "invalid-utf8"))
def test_batch_result_rejects_malformed_native_metadata(
    monkeypatch: pytest.MonkeyPatch, malformation: str
) -> None:
    response_pointer, _response_storage = _opaque("kmipkit_client_batch_response_t")
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")
    released: list[str] = []
    message_storage: list[object] = []

    def metadata(_item: object, output: object) -> int:
        _initialize_metadata_v2(output)
        if malformation == "reserved":
            output.reserved = 1  # type: ignore[attr-defined]
        elif malformation == "presence":
            output.has_result_message = 2  # type: ignore[attr-defined]
        else:
            message = client.ffi.new("uint8_t[]", b"\xff")
            message_storage.append(message)
            output.has_result_message = 1  # type: ignore[attr-defined]
            output.result_message_data = message  # type: ignore[attr-defined]
            output.result_message_length = 1  # type: ignore[attr-defined]
        return 0

    def absent_span(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output.data = client.ffi.NULL  # type: ignore[attr-defined]
        output.length = 0  # type: ignore[attr-defined]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_response_release=lambda _response: released.append("response"),
            kmipkit_client_batch_item_response_release=lambda _item: released.append("item"),
            kmipkit_client_batch_response_item_count=lambda _response, count: (
                count.__setitem__(0, 1) or 0
            ),
            kmipkit_client_batch_response_item_at=lambda _response, _index, output: (
                output.__setitem__(0, item_pointer) or 0
            ),
            kmipkit_client_batch_item_response_metadata_v2=metadata,
            kmipkit_client_batch_item_response_unique_batch_item_id=absent_span,
            kmipkit_client_batch_item_response_pending_correlation_value=absent_span,
            kmipkit_client_batch_item_response_discover_versions_supported_versions=(
                _no_supported_versions
            ),
            **_no_creation_response_symbols(),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._batch_result(response_pointer)
    assert released == ["item", "response"]


def test_batch_item_metadata_rejects_non_string_result_message() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.BatchItemMetadataV2(1, 1, 0, None, object())  # type: ignore[arg-type]


def test_creation_response_rejects_invalid_presence_flags(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")

    def create(_item: object, present: object, object_type: object, _identifier: object) -> int:
        present[0] = 2  # type: ignore[index]
        object_type[0] = 0  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_create=create,
        ),
    )
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)
    try:
        with pytest.raises(errors.InvalidInputError):
            client._create_response(item)
    finally:
        item.close()


def test_create_key_pair_response_rejects_invalid_presence_flag(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")

    def create_pair(
        _item: object, present: object, _private: object, _public: object
    ) -> int:
        present[0] = 2  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_create_key_pair=create_pair,
        ),
    )
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)
    try:
        with pytest.raises(errors.InvalidInputError):
            client._create_key_pair_response(item)
    finally:
        item.close()


@pytest.mark.parametrize(
    ("present_value", "count_value"),
    ((0, 1), (2, 1), (1, 0)),
    ids=("absent-with-count", "invalid-presence", "present-with-zero-count"),
)
def test_create_split_key_response_rejects_invalid_presence_and_count(
    monkeypatch: pytest.MonkeyPatch, present_value: int, count_value: int
) -> None:
    item_pointer, _item_storage = _opaque("kmipkit_client_batch_item_response_t")

    def split_count(_item: object, present: object, count: object) -> int:
        present[0] = present_value  # type: ignore[index]
        count[0] = count_value  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_release=lambda _item: None,
            kmipkit_client_batch_item_response_create_split_key_count=split_count,
        ),
    )
    item = _wrap_owned_handle(client._ClientBatchItemResponse, item_pointer)
    try:
        with pytest.raises(errors.InvalidInputError):
            client._create_split_key_response(item)
    finally:
        item.close()
