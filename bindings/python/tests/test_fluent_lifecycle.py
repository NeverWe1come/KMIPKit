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


def test_lifecycle_builders_validate_and_reject_invalid_arguments() -> None:
    request = fluent.Request(None)
    identifier = extensions.UniqueIdentifier.text_string("lifecycle-target")
    with pytest.raises(errors.KmipKitError):
        request.locate(None)

    with ttlv.ttlv_structure_create() as structure:
        with attribute_operations.AttributeSet.try_new(structure) as attributes:
            locate = request.locate(attributes)
            locate.storage_status_mask(1).object_group_member(1)
            locate.with_options(client.RequestOptions()).asynchronous_indicator(0)
            locate.validate()
            item = locate.to_batch_item()
            item.close()
            with pytest.raises(errors.KmipKitError):
                locate.with_options(object())  # type: ignore[arg-type]
            with pytest.raises(errors.KmipKitError):
                locate.asynchronous_indicator(-1)
            with pytest.raises(errors.KmipKitError):
                locate.asynchronous_indicator(1 << 32)
            with ttlv.codec_limits_defaults() as limits:
                with pytest.raises(errors.KmipKitError):
                    locate.execute(limits)

            with extensions.RevocationReason.create(1, "scheduled rotation") as reason:
                revoke = request.revoke(identifier, reason)
                revoke.with_options(client.RequestOptions()).asynchronous_indicator(0)
                revoke.validate()
                item = revoke.to_batch_item()
                item.close()
                with pytest.raises(errors.KmipKitError):
                    request.revoke(identifier, None)  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    revoke.with_options(object())  # type: ignore[arg-type]
                with pytest.raises(errors.KmipKitError):
                    revoke.asynchronous_indicator(-1)
                with pytest.raises(errors.KmipKitError):
                    revoke.asynchronous_indicator(1 << 32)
                with ttlv.codec_limits_defaults() as limits:
                    with pytest.raises(errors.KmipKitError):
                        revoke.execute(limits)

    operation = fluent.Operation(None, "unassigned", identifier)
    with pytest.raises(errors.KmipKitError):
        operation.build()
    operation = fluent.Operation(None, "activate", identifier)
    operation.with_options(client.RequestOptions()).asynchronous_indicator(0)
    operation.validate()
    item = operation.to_batch_item()
    item.close()


def test_direct_lifecycle_helpers_forward_optional_arguments(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    class Builder:
        def __init__(self) -> None:
            self.calls: list[tuple[str, object]] = []

        def with_options(self, value: object) -> Builder:
            self.calls.append(("options", value))
            return self

        def maximum_items(self, value: int) -> Builder:
            self.calls.append(("maximum_items", value))
            return self

        def offset_items(self, value: int) -> Builder:
            self.calls.append(("offset_items", value))
            return self

        def storage_status_mask(self, value: int) -> Builder:
            self.calls.append(("storage_status_mask", value))
            return self

        def object_group_member(self, value: int) -> Builder:
            self.calls.append(("object_group_member", value))
            return self

        def compromise_occurrence_date(self, value: int) -> Builder:
            self.calls.append(("compromise_occurrence_date", value))
            return self

        def execute(self, limits: object) -> object:
            self.calls.append(("execute", limits))
            return self

    class Request:
        def __init__(self) -> None:
            self.builders: list[Builder] = []

        def _builder(self, *_args: object) -> Builder:
            builder = Builder()
            self.builders.append(builder)
            return builder

        activate = _builder
        archive = _builder
        locate = _builder
        recover = _builder
        revoke = _builder

    request = Request()
    monkeypatch.setattr(client.Client, "request", lambda _self: request)
    fake_client = object.__new__(client.Client)
    identifier = extensions.UniqueIdentifier.text_string("lifecycle-target")
    options = client.RequestOptions()
    with ttlv.codec_limits_defaults() as limits:
        assert (
            client.Client.activate_by_id(fake_client, identifier, limits, options=options)
            is request.builders[-1]
        )
        assert (
            client.Client.archive_by_id(fake_client, identifier, limits, options=options)
            is request.builders[-1]
        )
        assert (
            client.Client.recover_by_id(fake_client, identifier, limits, options=options)
            is request.builders[-1]
        )
        with ttlv.ttlv_structure_create() as structure:
            with attribute_operations.AttributeSet.try_new(structure) as attributes:
                assert client.Client.locate(
                    fake_client, attributes, limits, maximum_items=4, offset_items=1,
                    storage_status_mask=1, object_group_member=1, options=options,
                ) is request.builders[-1]
        with extensions.RevocationReason.create(1, "scheduled rotation") as reason:
            assert client.Client.revoke_by_id(
                fake_client, identifier, reason, limits,
                compromise_occurrence_date=1_800_000_000, options=options,
            ) is request.builders[-1]
    assert len(request.builders) == 5
    assert all(builder.calls[-1][0] == "execute" for builder in request.builders)
    assert all(
        any(name == "options" for name, _ in builder.calls)
        for builder in request.builders
    )
    assert any(name == "storage_status_mask" for name, _ in request.builders[3].calls)
    assert any(name == "object_group_member" for name, _ in request.builders[3].calls)
    assert any(name == "compromise_occurrence_date" for name, _ in request.builders[4].calls)


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
                                attributes, limits, maximum_items=4, offset_items=1
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
