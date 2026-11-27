"""Phase 1 fluent & configuration parity tests for Python bindings."""

from __future__ import annotations

from pathlib import Path
import pytest

from kmipkit import client, errors, extensions, protocol_values, ttlv


def _contains_tag(structure: ttlv.Structure, target_tag: int) -> bool:
    with ttlv.ttlv_structure_view(structure) as structure_view:
        return _contains_tag_structure(structure_view, target_tag)


def _contains_tag_structure(structure_view: ttlv.TtlvStructureView, target_tag: int) -> bool:
    for idx in range(ttlv.ttlv_structure_view_item_count(structure_view)):
        with ttlv.ttlv_structure_view_item_at(structure_view, idx) as child:
            with ttlv.ttlv_item_view_tag(child) as tag:
                if ttlv.tag_value(tag) == target_tag:
                    return True
            if ttlv.ttlv_item_view_type(child) == ttlv.ItemType.Structure:
                with ttlv.ttlv_item_view_value(child) as value_view:
                    with ttlv.ttlv_value_view_structure(value_view) as sub_view:
                        if _contains_tag_structure(sub_view, target_tag):
                            return True
    return False


def test_client_configuration_embeds_and_preserves_codec_limits() -> None:
    with extensions.ClientConfiguration.defaults() as cfg:
        assert ttlv.codec_limits_max_message_bytes(cfg.codec_limits) > 0
        with ttlv.codec_limits_create(1024 * 1024, 16, 2048) as custom:
            with cfg.with_codec_limits(custom) as updated:
                assert ttlv.codec_limits_max_message_bytes(updated.codec_limits) == 1024 * 1024
                with updated.with_protocol_version(client.ProtocolVersion(3, 0)) as v3:
                    assert ttlv.codec_limits_max_message_bytes(v3.codec_limits) == 1024 * 1024
                    assert v3.protocol_version == client.ProtocolVersion(3, 0)


def test_connect_mtls_validates_missing_pem_paths_before_network() -> None:
    with pytest.raises(errors.InvalidInputError):
        client.Client.connect_mtls(
            "127.0.0.1",
            5696,
            Path("/nonexistent/kmipkit/ca.pem"),
            Path("/nonexistent/kmipkit/cert.pem"),
            Path("/nonexistent/kmipkit/key.pem"),
        )


def test_server_result_error_captures_pending_and_rejection_metadata() -> None:
    rejected = errors.ServerResultError(1, 5, "Unsupported")
    assert rejected.is_server_rejected
    assert not rejected.is_pending
    assert rejected.result_status == 1
    assert rejected.result_reason == 5

    pending = errors.ServerResultError(2, None, None, b"corr-123")
    assert pending.is_pending
    assert not pending.is_server_rejected
    assert pending.asynchronous_correlation_value == b"corr-123"


def test_kmip30_builders_emit_alternative_name_attribute() -> None:
    from kmipkit.fluent import CreateKeyPairBuilder, SymmetricKeyBuilder

    with SymmetricKeyBuilder().with_protocol_version(client.ProtocolVersion(3, 0)) as b30:
        b30.aes_256().usage_mask(protocol_values.CryptographicUsageMask.ENCRYPT).name("k-30")
        with b30.build() as item, client.Batch.create() as batch:
            batch.add_item(item)
            with batch.build_request_message(client.ProtocolVersion(3, 0)) as msg:
                with msg.to_ttlv() as structure:
                    assert _contains_tag(structure, 0x0042_00BF)
                    assert not _contains_tag(structure, 0x0042_0053)

    with CreateKeyPairBuilder().with_protocol_version(client.ProtocolVersion(3, 0)) as pair30:
        pair30.algorithm(protocol_values.CryptographicAlgorithm.ECDSA).length(256)
        pair30.private_usage_mask(protocol_values.CryptographicUsageMask.SIGN)
        pair30.public_usage_mask(protocol_values.CryptographicUsageMask.VERIFY)
        pair30.private_name("priv-30").public_name("pub-30")
        with pair30.build() as item, client.Batch.create() as batch:
            batch.add_item(item)
            with batch.build_request_message(client.ProtocolVersion(3, 0)) as msg:
                with msg.to_ttlv() as structure:
                    assert _contains_tag(structure, 0x0042_00BF)
                    assert not _contains_tag(structure, 0x0042_0053)


def test_execute_returns_item_or_raises_and_execute_outcome_preserves_failure(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from kmipkit.fluent import PingBuilder

    fake_client = object.__new__(client.Client)
    fake_client._codec_limits = ttlv.codec_limits_defaults()
    fake_client._protocol_version = client.ProtocolVersion(2, 1)

    ok_item = client.BatchItemResult(
        client.BatchOutcome.COMPLETED, 0, None, None, None, operation=0x3B, ping_response=True
    )
    fail_item = client.BatchItemResult(
        client.BatchOutcome.COMPLETED, 1, 5, None, None, operation=0x3B
    )
    outcomes = [ok_item, fail_item, fail_item]

    class _FakeBatch:
        def __init__(self, item: client.BatchItemResult) -> None:
            self.items = (item,)

        def _take_single_item(self) -> client.BatchItemResult:
            (one,) = self.items
            self.items = ()
            return one

        def __enter__(self) -> _FakeBatch:
            return self

        def __exit__(self, *_: object) -> None:
            return None

    monkeypatch.setattr(fake_client, "execute", lambda _b, _l: _FakeBatch(outcomes.pop(0)))

    res = PingBuilder(fake_client).execute()
    assert res is ok_item

    with pytest.raises(errors.ServerResultError) as exc_info:
        PingBuilder(fake_client).execute()
    assert exc_info.value.is_server_rejected
    assert exc_info.value.result_status == 1
    assert exc_info.value.result_reason == 5

    outcome = PingBuilder(fake_client).execute_outcome()
    assert outcome.result_status == 1
    assert outcome.result_reason == 5
