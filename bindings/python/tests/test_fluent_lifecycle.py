"""Public Python lifecycle facade shape and behavior tests."""

from __future__ import annotations

from pathlib import Path

import pytest
from _test_support import mtls_transport_config
from kmipkit import attribute_operations, client, errors, extensions, fluent, ttlv

from test_fluent import _start_lifecycle_peer


def test_request_namespace_has_all_lifecycle_builders() -> None:
    for name in ("activate", "archive", "locate", "recover", "revoke"):
        assert hasattr(fluent.Request, name), f"Request.{name} is missing"


def test_client_has_named_lifecycle_helpers() -> None:
    for name in ("activate_by_id", "archive_by_id", "locate", "recover_by_id", "revoke_by_id"):
        assert hasattr(client.Client, name), f"Client.{name} is missing"


def test_fluent_lifecycle_builds_typed_batch_items() -> None:
    request = fluent.Request(object())
    identifier = extensions.UniqueIdentifier.text_string("lifecycle-target")
    with ttlv.ttlv_structure_create() as structure:
        with attribute_operations.AttributeSet.try_new(structure) as attributes:
            items = [
                request.activate(identifier).build(),
                request.archive(identifier).build(),
                request.locate(attributes).build(),
                request.recover(identifier).build(),
            ]
            with extensions.RevocationReason.create(1, "scheduled rotation") as reason:
                builder = request.revoke(identifier, reason)
                assert "scheduled rotation" not in repr(builder)
                with pytest.raises(errors.KmipKitError):
                    request.revoke(None, reason)  # type: ignore[arg-type]
                revoke = request.revoke(identifier, reason).build()
                items.append(revoke)
            try:
                assert [item.operation for item in items] == [0x12, 0x15, 0x08, 0x16, 0x13]
            finally:
                for item in items:
                    item.close()


def test_fluent_and_direct_lifecycle_helpers_roundtrip_over_mtls() -> None:
    operations = (0x12, 0x15, 0x08, 0x16, 0x13) * 2
    port, observations, peer_failures, peer = _start_lifecycle_peer(operations)
    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    identifier = extensions.UniqueIdentifier.text_string("lifecycle-target")
    results: list[client.BatchResult] = []
    try:
        fixtures = Path(__file__).parent / "fixtures" / "login-mtls"
        with configuration, client.Client.create(
            configuration, mtls_transport_config(port, fixtures)
        ) as kmip_client, ttlv.codec_limits_defaults() as limits:
            with ttlv.ttlv_structure_create() as structure:
                with attribute_operations.AttributeSet.try_new(structure) as attributes:
                    builders = [
                        kmip_client.request().activate(identifier),
                        kmip_client.request().archive(identifier),
                        kmip_client.request().locate(attributes)
                        .maximum_items(5).offset_items(2),
                        kmip_client.request().recover(identifier),
                    ]
                    with extensions.RevocationReason.create(1, "scheduled rotation") as reason:
                        builders.append(kmip_client.request().revoke(identifier, reason))
                    for builder in builders:
                        results.append(builder.execute(limits))
                    results.extend(
                        [
                            kmip_client.activate_by_id(identifier, limits),
                            kmip_client.archive_by_id(identifier, limits),
                            kmip_client.locate(
                                attributes, limits, maximum_items=4, offset_items=1,
                                options=client.RequestOptions(),
                            ),
                            kmip_client.recover_by_id(identifier, limits),
                        ]
                    )
                    with extensions.RevocationReason.create(
                        2, "scheduled rotation"
                    ) as reason:
                        results.append(
                            kmip_client.revoke_by_id(
                                identifier,
                                reason,
                                limits,
                                compromise_occurrence_date=1_800_000_000,
                                options=client.RequestOptions(),
                            )
                        )
        peer.join(timeout=12)
        assert not peer.is_alive()
        assert peer_failures.empty()
        assert [item.items[0].operation for item in results] == list(operations)
        assert [operation for operation, _ in observations] == list(operations)
    finally:
        for result in results:
            result.close()
        peer.join(timeout=12)
        if not peer_failures.empty():
            raise peer_failures.get()
