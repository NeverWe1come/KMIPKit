"""Controlled TLS 1.3 mTLS exchange tests; never evidence of vendor support."""

import json
import os
import unittest
from unittest.mock import patch

from kmipkit_validator.contract import DEFAULT_CATALOG
from kmipkit_validator.driver import PythonKmipDriver
from kmipkit_validator.controlled_peer import (
    ControlledTlsPeer, PKI, find_value, normalized_operation_name,
    parse_children, text_response,
)


class ControlledExchangeTests(unittest.TestCase):
    def test_every_applicable_catalog_operation_is_sent_to_a_controlled_kmip_peer(self):
        from kmipkit import extensions, protocol_values

        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        async_operations = {
            "Poll", "Process", "Cancel", "Query Asynchronous Requests"
        }
        network_combinations = tuple(
            (encoding, transport)
            for encoding in ("TTLV", "JSON", "XML")
            for transport in ("raw_tls", "https")
        )
        for protocol_version in ("2.0", "2.1"):
            cases = [
                case
                for case in catalog["test_cases"]
                if case["execution"]["mode"] == "server_exchange"
                and protocol_version in case["execution"]["protocol_versions"]
                and case.get("requires", {}).get("workflow_type") != "server_to_client"
                and PythonKmipDriver.supports(case, "typed-protocol")
            ]
            self.assertGreater(len(cases), 140)
            for case in cases:
                for encoding, transport in network_combinations:
                    with self.subTest(
                        protocol_version=protocol_version,
                        case_id=case["case_id"],
                        encoding=encoding,
                        transport=transport,
                    ):
                        peer = ControlledTlsPeer(
                            status=protocol_values.ResultStatus.OPERATION_FAILED,
                            reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                            encoding=encoding,
                            transport=transport,
                            protocol_version=protocol_version,
                        )
                        connection = {
                            "server_id": "loopback",
                            "protocol_version": protocol_version,
                            "transport": transport,
                            "encoding": encoding,
                            "api_level": "typed-protocol",
                            "endpoints": {
                                "raw_tls": {"host": "127.0.0.1", "port": peer.port},
                                "https": {"url": f"https://127.0.0.1:{peer.port}/kmip"},
                            },
                            "tls": {
                                "ca_cert_path": str(PKI / "server-cert.pem"),
                                "client_cert_path": str(PKI / "client-cert.pem"),
                                "client_key_path": str(PKI / "client-key.pem"),
                                "server_name_override": "localhost",
                                "connect_timeout_seconds": 5,
                                "read_timeout_seconds": 5,
                            },
                        }
                        driver = PythonKmipDriver(connection)
                        item = None
                        try:
                            if case["operation"] in async_operations:
                                correlation = (
                                    None
                                    if case["operation"] == "Query Asynchronous Requests"
                                    else b"fixture-correlation"
                                )
                                observed = driver._async_exchange(case["operation"], correlation)
                            elif case["operation"] == "Logout":
                                ticket_type = driver._item(0x0042_014A, "enumeration", 1)
                                ticket_value = driver._item(0x0042_014B, "bytes", b"fixture-ticket")
                                ticket = driver._structure([ticket_type, ticket_value])
                                try:
                                    item = extensions.client_batch_item_logout(ticket)
                                    observed = driver._exchange(item)
                                finally:
                                    ticket.close()
                                    ticket_type.close()
                                    ticket_value.close()
                            else:
                                item = driver._build_item(
                                    case,
                                    extensions.UniqueIdentifier.text_string("catalog-operation-fixture"),
                                    name="catalog-operation-fixture",
                                )
                                observed = driver._exchange(item)
                            self.assertEqual("server_unsupported", observed["status"])
                            self.assertEqual(
                                normalized_operation_name(case["operation"]),
                                normalized_operation_name(str(peer.request_observation["operation"])),
                            )
                            self.assertEqual(1, len(peer.requests))
                        finally:
                            if item is not None:
                                item.close()
                            driver.close()
                            peer.close()

    def _execute_attribute_route(
        self,
        peer: ControlledTlsPeer,
        *,
        high_level: bool,
        case_id: str,
        encoding: str,
        transport: str,
    ) -> dict:
        from kmipkit import extensions

        connection = {
            "server_id": "loopback",
            "protocol_version": "2.1",
            "transport": transport,
            "encoding": encoding,
            "api_level": "typed-protocol",
            "endpoints": {
                "raw_tls": {"host": "127.0.0.1", "port": peer.port},
                "https": {"url": f"https://127.0.0.1:{peer.port}/kmip"},
            },
            "tls": {
                "ca_cert_path": str(PKI / "server-cert.pem"),
                "client_cert_path": str(PKI / "client-cert.pem"),
                "client_key_path": str(PKI / "client-key.pem"),
                "server_name_override": "localhost",
                "connect_timeout_seconds": 5,
                "read_timeout_seconds": 5,
            },
        }
        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        case = next(entry for entry in catalog["test_cases"] if entry["case_id"] == case_id)
        driver = PythonKmipDriver(connection)
        identifier = extensions.UniqueIdentifier.text_string("same-request-fixture")
        item = None
        try:
            item = (
                driver._build_high_level_item(case, identifier, name="same-request")
                if high_level
                else driver._build_item(case, identifier, name="same-request")
            )
            return driver._exchange(item)
        finally:
            if item is not None:
                item.close()
            driver.close()
            peer.close()

    def test_high_level_and_typed_attribute_routes_emit_identical_requests_and_responses(self):
        from kmipkit import protocol_values

        network_combinations = tuple(
            (encoding, transport)
            for encoding in ("TTLV", "JSON", "XML")
            for transport in ("raw_tls", "https")
        )
        for case_id in (
            "test-op-add-attribute",
            "test-op-adjust-attribute",
            "test-op-delete-attribute",
            "test-op-get-attributes",
            "test-op-get-attribute-list",
            "test-op-modify-attribute",
            "test-attr-comment",
        ):
            for encoding, transport in network_combinations:
                sent = []
                received = []
                outcomes = []
                for high_level in (False, True):
                    peer = ControlledTlsPeer(
                        status=protocol_values.ResultStatus.OPERATION_FAILED,
                        reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                        encoding=encoding,
                        transport=transport,
                    )
                    outcomes.append(
                        self._execute_attribute_route(
                            peer,
                            high_level=high_level,
                            case_id=case_id,
                            encoding=encoding,
                            transport=transport,
                        )
                    )
                    sent.append(peer.requests[0])
                    received.append(peer.responses[0])
                with self.subTest(case_id=case_id, encoding=encoding, transport=transport):
                    self.assertEqual(sent[0], sent[1])
                    self.assertEqual(received[0], received[1])
                    self.assertEqual(
                        ["server_unsupported", "server_unsupported"],
                        [result["status"] for result in outcomes],
                    )

    def _execute_ping(
        self,
        peer: ControlledTlsPeer,
        authentication: dict | None = None,
        case_id: str = "test-op-ping",
        protocol_version: str = "2.1",
        encoding: str = "TTLV",
        transport: str = "raw_tls",
    ) -> dict:
        connection = {
            "server_id": "loopback",
            "protocol_version": protocol_version,
            "transport": transport,
            "encoding": encoding,
            "api_level": "typed-protocol",
            "endpoints": {
                "raw_tls": {"host": "127.0.0.1", "port": peer.port},
                "https": {"url": f"https://127.0.0.1:{peer.port}/kmip"},
            },
            "tls": {
                "ca_cert_path": str(PKI / "server-cert.pem"),
                "client_cert_path": str(PKI / "client-cert.pem"),
                "client_key_path": str(PKI / "client-key.pem"),
                "server_name_override": "localhost",
                "connect_timeout_seconds": 5,
                "read_timeout_seconds": 5,
            },
        }
        if authentication is not None:
            connection["authentication"] = authentication
        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        case = next(entry for entry in catalog["test_cases"] if entry["case_id"] == case_id)
        driver = None
        try:
            driver = PythonKmipDriver(connection)
            return driver.execute_case(case)
        finally:
            if driver is not None:
                driver.close()
            peer.close()

    def test_ping_uses_installed_client_and_verifies_a_controlled_tls_response(self):
        from kmipkit import protocol_values

        peer = ControlledTlsPeer()
        observed = self._execute_ping(peer)
        self.assertEqual("passed", observed["status"])
        self.assertEqual(0, observed["result_status_code"])
        self.assertEqual(1, len(peer.requests))

    def test_ping_round_trips_all_encodings_over_raw_tls_and_https(self):
        for encoding in ("TTLV", "JSON", "XML"):
            for transport in ("raw_tls", "https"):
                with self.subTest(encoding=encoding, transport=transport):
                    peer = ControlledTlsPeer(encoding=encoding, transport=transport)
                    observed = self._execute_ping(peer, encoding=encoding, transport=transport)
                    self.assertEqual("passed", observed["status"])
                    self.assertEqual("passed", observed["library_correctness"])
                    self.assertEqual("Ping", peer.request_observation["operation"])
                    self.assertEqual(1, peer.request_observation["batch_count"])

    def test_text_error_response_uses_encoding_wire_enum_names(self):
        from kmipkit import protocol_values

        for encoding in ("JSON", "XML"):
            with self.subTest(encoding=encoding):
                response = text_response(
                    "Ping",
                    encoding,
                    protocol_values.ResultStatus.OPERATION_FAILED,
                    protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                )
                self.assertIn(b"OperationFailed", response)
                self.assertIn(b"OperationNotSupported", response)

    def test_normative_not_supported_response_is_separate_from_library_failure(self):
        from kmipkit import protocol_values

        peer = ControlledTlsPeer(
            status=protocol_values.ResultStatus.OPERATION_FAILED,
            reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
        )
        observed = self._execute_ping(peer)
        self.assertEqual("server_unsupported", observed["status"])
        self.assertEqual("passed", observed["library_correctness"])
        self.assertEqual("unsupported", observed["server_compatibility"])

    def test_username_password_authentication_is_sent_but_never_reported(self):
        peer = ControlledTlsPeer()
        authentication = {
            "username_password": {
                "username_env": "KMIPKIT_INTEROP_TEST_USER",
                "password_env": "KMIPKIT_INTEROP_TEST_PASSWORD",
            }
        }
        with patch.dict(
            os.environ,
            {
                "KMIPKIT_INTEROP_TEST_USER": "interop-user-sentinel",
                "KMIPKIT_INTEROP_TEST_PASSWORD": "interop-password-sentinel",
            },
        ):
            observed = self._execute_ping(peer, authentication)
        request = peer.requests[0]
        self.assertIn(bytes.fromhex("420023"), request)
        self.assertIn(b"interop-user-sentinel", request)
        self.assertIn(b"interop-password-sentinel", request)
        self.assertNotIn("interop-user-sentinel", json.dumps(observed))
        self.assertNotIn("interop-password-sentinel", json.dumps(observed))

    def test_query_async_requests_uses_installed_explicit_async_api(self):
        from kmipkit import protocol_values

        peer = ControlledTlsPeer(
            status=protocol_values.ResultStatus.OPERATION_FAILED,
            reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
        )
        observed = self._execute_ping(peer, case_id="test-op-query-asynchronous-requests")
        self.assertEqual("server_unsupported", observed["status"])
        self.assertEqual(1, observed["result_status_code"])
        self.assertEqual(1, len(peer.requests))
        self.assertEqual(
            0x39,
            int.from_bytes(find_value(parse_children(peer.requests[0][8:]), 0x42005C), "big"),
        )

    def test_kmip20_process_is_rejected_before_opening_a_connection(self):
        peer = ControlledTlsPeer(expect_no_connection=True)
        observed = self._execute_ping(
            peer,
            case_id="test-v20-reject-process",
            protocol_version="2.0",
        )
        self.assertEqual("passed", observed["status"])
        self.assertEqual("passed", observed["library_correctness"])
        self.assertEqual([], peer.requests)

    def test_kmip20_query_async_requests_is_rejected_before_opening_a_connection(self):
        peer = ControlledTlsPeer(expect_no_connection=True)
        observed = self._execute_ping(
            peer,
            case_id="test-v20-reject-query-asynchronous-requests",
            protocol_version="2.0",
        )
        self.assertEqual("passed", observed["status"])
        self.assertEqual("passed", observed["library_correctness"])
        self.assertEqual([], peer.requests)


if __name__ == "__main__":
    unittest.main()
