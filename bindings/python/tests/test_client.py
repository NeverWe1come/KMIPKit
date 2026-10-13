"""Contract tests for the shared Python client facade (KMIPKIT-0043 T024).

The client facade is a language adapter over the reviewed T076 C ABI. These
tests intentionally do not create a real network connection; Rust/C tests own
transport-wire verification.
"""

from __future__ import annotations

from pathlib import Path
from typing import get_type_hints

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
    assert client.BatchItemResult.__annotations__["pending_correlation_value"] == (
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
