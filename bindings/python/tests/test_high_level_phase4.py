"""Fase 4: batch builder, workflows, multipart streams, pending op, locate_iter, S2C."""

import types
import unittest
from unittest.mock import patch

from kmipkit import (
    CipherSpec,
    FluentBatchBuilder,
    HashAlgorithm,
    KeySpec,
    LocatePaginator,
    PendingOperation,
    ServerHandlerBuilder,
    StreamsClient,
    WorkflowsClient,
    client,
    extensions,
    fluent,
)


class _FakeItem:
    def __init__(self, uid: str = "k-1", corr: bytes = b"c1", data: bytes = b"out", located=()):
        self.unique_identifier = types.SimpleNamespace(text=uid)
        self.correlation_value = corr
        self.data = data
        self.located_unique_identifiers = list(located)


class _FakeResp:
    def __init__(self, item: _FakeItem):
        self._item = item

    def item_at(self, _idx: int) -> _FakeItem:
        return self._item


class _FakeBuilder:
    def __init__(self, item: _FakeItem):
        self._item = item
        self.calls: list[tuple[str, object]] = []

    def _rec(self, name: str, val: object = None):
        self.calls.append((name, val))
        return self

    def with_key_spec(self, v):
        return self._rec("with_key_spec", v)

    def usage_mask(self, v):
        return self._rec("usage_mask", v)

    def name(self, v):
        return self._rec("name", v)

    def unique_identifier(self, v):
        return self._rec("unique_identifier", v)

    def with_hash_algorithm(self, v):
        return self._rec("with_hash_algorithm", v)

    def with_cipher_spec(self, v):
        return self._rec("with_cipher_spec", v)

    def data(self, v):
        return self._rec("data", v)

    def initial_indicator(self, v):
        return self._rec("initial_indicator", v)

    def final_indicator(self, v):
        return self._rec("final_indicator", v)

    def correlation_value(self, v):
        return self._rec("correlation_value", v)

    def maximum_items(self, v):
        return self._rec("maximum_items", v)

    def offset_items(self, v):
        return self._rec("offset_items", v)

    def execute(self):
        return _FakeResp(self._item)


class _FakeRequestFactory:
    def __init__(self, item: _FakeItem | None = None):
        self.item = item or _FakeItem()
        self.builders: list[_FakeBuilder] = []

    def _new(self):
        b = _FakeBuilder(self.item)
        self.builders.append(b)
        return b

    def create_symmetric_key(self):
        return self._new()

    def activate(self):
        return self._new()

    def revoke(self, _reason):
        return self._new()

    def destroy(self, _uid):
        return self._new()

    def hash(self):
        return self._new()

    def encrypt(self):
        return self._new()

    def poll(self, _corr):
        return self._new()

    def process(self, _corr):
        return self._new()

    def cancel(self, _corr):
        return self._new()

    def locate(self, _attrs):
        return self._new()


class _FakeClient:
    def __init__(self):
        self.codec_limits = object()
        self.executed = []

    def execute(self, batch, limits):
        self.executed.append(("execute", batch, limits, None))
        return "resp"

    def execute_with_options(self, batch, limits, options):
        self.executed.append(("execute_with_options", batch, limits, options))
        return "resp-opts"


class HighLevelPhase4Test(unittest.TestCase):
    def test_fluent_batch_builder_assembles_ordered_batch(self) -> None:
        fake_client = _FakeClient()
        with FluentBatchBuilder(fake_client).ordered().asynchronous_indicator(1) as batch:
            ping = fluent.PingBuilder(None)
            batch.add(ping.build())
            self.assertEqual(batch.build().item_count, 1)
            self.assertEqual(batch.execute(), "resp")
            batch.with_options({"timeout": 5})
            self.assertEqual(batch.execute(), "resp-opts")

    def test_pending_operation_and_server_handler_builder(self) -> None:
        pending = PendingOperation(b"\x01\x02\x03\x04")
        self.assertEqual(pending.correlation_value, b"\x01\x02\x03\x04")
        factory = _FakeRequestFactory()
        with patch("kmipkit.flows.fluent.Request", return_value=factory):
            pending.poll(None)
            pending.process(None)
            pending.cancel(None)
        self.assertEqual(len(factory.builders), 3)

        handlers = (
            ServerHandlerBuilder()
            .on_discover_versions(lambda req: {"v": []})
            .on_notify(lambda req: {"ok": True})
            .on_put(lambda req: {"put": True})
            .on_query(lambda req: {"ops": []})
            .build()
        )
        self.assertIsNotNone(handlers["discover_versions"])
        self.assertIsNotNone(handlers["notify"])
        self.assertIsNotNone(handlers["put"])
        self.assertIsNotNone(handlers["query"])

    def test_workflows_streams_and_locate_paginator(self) -> None:
        factory = _FakeRequestFactory(_FakeItem(uid="sym-1", corr=b"corr-1", data=b"digest"))
        with patch("kmipkit.flows.fluent.Request", return_value=factory):
            wf = WorkflowsClient(None)
            uid = wf.create_active_symmetric_key(KeySpec.AES_256, 0x0C, name="k1")
            self.assertEqual(uid, "sym-1")
            wf.revoke_and_destroy("sym-1", extensions.RevocationReason.create(5))

            streams = StreamsClient(None)
            hs = streams.hash(HashAlgorithm.SHA256)
            hs.update(b"abc")
            hs.update(b"def")
            self.assertEqual(hs.finish(), b"digest")

            es = streams.encrypt("sym-1", CipherSpec.AES_GCM)
            self.assertEqual(es.update(b"plain"), b"digest")
            self.assertEqual(es.finish(), b"digest")

            factory.item = _FakeItem(located=["id-1", "id-2"])
            pager = LocatePaginator(None, object(), page_size=2)
            self.assertEqual(pager.next_page(), ["id-1", "id-2"])
            factory.item = _FakeItem(located=[])
            self.assertIsNone(pager.next_page())
            self.assertIsNone(pager.next_page())

    def test_client_phase4_methods_exist(self) -> None:
        for name in ("batch", "workflows", "streams"):
            self.assertTrue(hasattr(client.Client, name), name)
        self.assertTrue(hasattr(LocatePaginator, "next_page"))


if __name__ == "__main__":
    unittest.main()
