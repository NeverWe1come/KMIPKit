"""Fase 4: batch builder, workflows, multipart streams, pending op, locate_iter, S2C."""

import unittest

from kmipkit import (
    FluentBatchBuilder,
    LocatePaginator,
    PendingOperation,
    ServerHandlerBuilder,
    client,
    fluent,
)


class HighLevelPhase4Test(unittest.TestCase):
    def test_fluent_batch_builder_assembles_ordered_batch(self) -> None:
        with FluentBatchBuilder(None).ordered() as batch:
            ping = fluent.PingBuilder(None)
            batch.add(ping.build())
            self.assertEqual(batch.build().item_count, 1)

    def test_pending_operation_and_server_handler_builder(self) -> None:
        pending = PendingOperation(b"\x01\x02\x03\x04")
        self.assertEqual(pending.correlation_value, b"\x01\x02\x03\x04")

        handlers = (
            ServerHandlerBuilder()
            .on_notify(lambda req: {"ok": True})
            .on_query(lambda req: {"ops": []})
            .build()
        )
        self.assertIsNotNone(handlers["notify"])
        self.assertIsNotNone(handlers["query"])
        self.assertIsNone(handlers["put"])

    def test_client_phase4_methods_exist(self) -> None:
        for name in ("batch", "workflows", "streams"):
            self.assertTrue(hasattr(client.Client, name), name)
        self.assertTrue(hasattr(LocatePaginator, "next_page"))


if __name__ == "__main__":
    unittest.main()
