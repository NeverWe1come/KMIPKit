"""Unit and integration tests for KMIP 2.1 Multi-Profile Mock KMS Server."""

from __future__ import annotations

import json
import socket
import unittest
from pathlib import Path

# Add shared dir to path
import sys
_shared_dir = Path(__file__).resolve().parents[1]
if str(_shared_dir) not in sys.path:
    sys.path.insert(0, str(_shared_dir))

from importlib import import_module
mock_server_mod = import_module("mock-kms-server")
KmsState = mock_server_mod.KmsState
NameCollisionError = mock_server_mod.state.NameCollisionError
IllegalLifecycleTransitionError = mock_server_mod.state.IllegalLifecycleTransitionError
MockKmsDispatcher = mock_server_mod.MockKmsDispatcher
MockKmsServer = mock_server_mod.MockKmsServer

PROFILES_DIR = _shared_dir / "profiles"


class MockKmsServerTests(unittest.TestCase):
    def test_state_name_collision_and_isolation(self) -> None:
        state = KmsState()

        # 1. First key creation succeeds
        k1 = state.create_object(name="kmipkit-isolated-key-01", object_type="SymmetricKey")
        self.assertIsNotNone(k1.unique_identifier)
        self.assertEqual(k1.state, "Active")

        # 2. Duplicate key with exact same name fails with collision error
        with self.assertRaises(NameCollisionError):
            state.create_object(name="kmipkit-isolated-key-01", object_type="SymmetricKey")

        # 3. Key with different isolation suffix succeeds
        k2 = state.create_object(name="kmipkit-isolated-key-02", object_type="SymmetricKey")
        self.assertNotEqual(k1.unique_identifier, k2.unique_identifier)

    def test_state_lifecycle_transitions(self) -> None:
        state = KmsState()
        k1 = state.create_object(name="lifecycle-key", object_type="SymmetricKey")

        # Destroying active object directly without revoke must fail
        with self.assertRaises(IllegalLifecycleTransitionError):
            state.destroy_object(k1.unique_identifier, allow_active=False)

        # Transition Active -> Revoked -> Destroyed
        state.revoke_object(k1.unique_identifier, reason="CessationOfOperation")
        k_rev = state.get_object(k1.unique_identifier)
        self.assertEqual(k_rev.state, "Revoked")

        k_dest = state.destroy_object(k1.unique_identifier)
        self.assertEqual(k_dest.state, "Destroyed")

        # Once destroyed, name can be reused without collision
        k_new = state.create_object(name="lifecycle-key", object_type="SymmetricKey")
        self.assertEqual(k_new.name, "lifecycle-key")
        self.assertNotEqual(k_new.unique_identifier, k1.unique_identifier)

    def test_dispatcher_cosmian_profile_enforcement(self) -> None:
        cosmian_prof = PROFILES_DIR / "cosmian-kms.json"
        dispatcher = MockKmsDispatcher.from_profile_file(cosmian_prof)

        # 1. DiscoverVersions
        res_disc = dispatcher.dispatch({"operation": "DiscoverVersions"})
        self.assertEqual(res_disc["result_status"], "Success")
        self.assertIn("2.1", res_disc["supported_versions"])

        # 2. Query
        res_query = dispatcher.dispatch({"operation": "Query"})
        self.assertEqual(res_query["result_status"], "Success")
        self.assertEqual(res_query["server_information"]["vendor"], "Cosmian")
        self.assertIn("Create", res_query["operations"])
        self.assertNotIn("Archive", res_query["operations"])

        # 3. Create Key
        res_create = dispatcher.dispatch({
            "operation": "Create",
            "name": "cosmian-test-key-1",
            "object_type": "SymmetricKey",
            "algorithm": "AES",
            "length": 256,
        })
        self.assertEqual(res_create["result_status"], "Success")
        uid = res_create["unique_identifier"]

        # 4. Name Collision via Dispatcher
        res_collide = dispatcher.dispatch({
            "operation": "Create",
            "name": "cosmian-test-key-1",
            "object_type": "SymmetricKey",
        })
        self.assertEqual(res_collide["result_status"], "OperationFailed")
        self.assertEqual(res_collide["result_reason"], "IndexAlreadyExists")

        # 5. Unsupported Operation: Cosmian rejects Archive
        res_archive = dispatcher.dispatch({
            "operation": "Archive",
            "unique_identifier": uid,
        })
        self.assertEqual(res_archive["result_status"], "OperationFailed")
        self.assertEqual(res_archive["result_reason"], "OperationNotSupported")

    def test_server_tcp_socket_interaction(self) -> None:
        dispatcher = MockKmsDispatcher()
        with MockKmsServer(dispatcher=dispatcher, host="127.0.0.1", port=0) as server:
            self.assertGreater(server.port, 0)
            conn_config = server.get_connection_config()
            self.assertEqual(conn_config["host"], "127.0.0.1")

            # Connect via raw TCP socket and send DiscoverVersions request
            with socket.create_connection((server.host, server.port), timeout=2.0) as sock:
                req_bytes = json.dumps({"operation": "DiscoverVersions"}).encode("utf-8") + b"\n"
                sock.sendall(req_bytes)

                resp_line = sock.makefile().readline()
                resp = json.loads(resp_line)

                self.assertEqual(resp["result_status"], "Success")
                self.assertIn("2.1", resp["supported_versions"])


if __name__ == "__main__":
    unittest.main()
