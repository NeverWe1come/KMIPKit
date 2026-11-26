"""Controlled TLS 1.3 mTLS exchange tests; never evidence of vendor support."""

import hashlib
import json
import os
import struct
import unittest
from unittest.mock import call, MagicMock, Mock, patch

from kmipkit_validator.contract import DEFAULT_CATALOG
from kmipkit_validator.driver import PythonKmipDriver
from kmipkit_validator.controlled_peer import (
    ControlledTlsPeer, PKI, find_value, normalized_operation_name,
    discover_versions_response, enumeration, item, parse_children, ping_response, structure, text_response,
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
            self.assertEqual(len(cases), {"2.0": 128, "2.1": 135}[protocol_version])
            for case in cases:
                for encoding, transport in network_combinations:
                    with self.subTest(
                        protocol_version=protocol_version,
                        case_id=case["case_id"],
                        encoding=encoding,
                        transport=transport,
                    ):
                        is_version_boundary = case["requires"].get("expected_status") == "NotSent"
                        peer = ControlledTlsPeer(
                            expect_no_connection=is_version_boundary,
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
                            if is_version_boundary:
                                observed = driver.execute_case(case)
                                self.assertEqual("passed", observed["status"])
                                self.assertEqual("unknown", observed["server_compatibility"])
                                self.assertEqual([], peer.requests)
                                continue
                            if case["operation"] in async_operations:
                                correlation = (
                                    None
                                    if case["operation"] == "Query Asynchronous Requests"
                                    else b"fixture-correlation"
                                )
                                observed = driver._async_exchange(case["operation"], correlation)
                            elif case["case_id"] == "test-multipart-hash-sha256":
                                observed = driver.execute_case(case)
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
                            wire_operation = case["operation"]
                            if wire_operation == "Create" and case.get("requires", {}).get("object_type") in {
                                "Private Key", "Public Key"
                            }:
                                wire_operation = "Create Key Pair"
                            elif wire_operation == "Create" and case.get("requires", {}).get("object_type") == "Split Key":
                                wire_operation = "Create Split Key"
                            self.assertEqual(
                                normalized_operation_name(wire_operation),
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
        protocol_version: str = "2.1",
        fluent: bool = False,
        draft_operation: str | None = None,
    ) -> dict:
        from kmipkit import extensions

        connection = {
            "server_id": "loopback",
            "protocol_version": protocol_version,
            "transport": transport,
            "encoding": encoding,
            "api_level": "fluent" if fluent else "typed-protocol",
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
        case = (next(entry for entry in catalog["test_cases"] if entry["case_id"] == case_id)
                if draft_operation is None else {"case_id": case_id, "operation": draft_operation})
        driver = PythonKmipDriver(connection)
        identifier = (extensions.UniqueIdentifier.identifier("same-request-fixture")
                      if draft_operation is not None else extensions.UniqueIdentifier.text_string("same-request-fixture"))
        item = None
        try:
            if draft_operation is not None:
                item = driver._build_draft_item(draft_operation, identifier, "same-request", fluent=fluent,
                                                ciphertext=b"draft-fixture-ciphertext")
            elif fluent:
                item = driver._build_fluent_item(case, identifier, name="same-request")
            elif high_level:
                item = driver._build_high_level_item(case, identifier, name="same-request")
            else:
                item = driver._build_item(case, identifier, name="same-request")
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

    def test_all_draft_operations_reach_a_kmip30_peer_by_both_builders(self):
        for operation in (
            "Create Group", "Create User", "Create Credential", "Deactivate",
            "Obliterate", "Encapsulate", "Decapsulate",
        ):
            with self.subTest(operation=operation):
                observations = []
                for fluent in (False, True):
                    peer = ControlledTlsPeer(response_factory=lambda request: ping_response(
                        request, status=1, reason=5, protocol_minor=0, protocol_major=3,
                    ))
                    with patch("kmipkit_validator.driver.uuid.uuid4", return_value="disposable-fixture"):
                        result = self._execute_attribute_route(
                            peer, high_level=False, case_id="draft-operation-probe", fluent=fluent,
                            encoding="TTLV", transport="raw_tls", protocol_version="3.0",
                            draft_operation=operation,
                        )
                    self.assertEqual("server_unsupported", result["status"])
                    observations.append((peer.requests[0], peer.responses[0]))
                self.assertEqual(*observations)

    def test_fluent_split_key_register_and_recertify_match_typed_requests(self):
        from kmipkit.protocol_values import Operation

        cases = {
            "test-op-create-split-key": Operation.CREATE_SPLIT_KEY,
            "test-obj-split-key": Operation.CREATE_SPLIT_KEY,
            "test-op-register": Operation.REGISTER,
            "test-obj-certificate": Operation.REGISTER,
            "test-obj-certificate-request": Operation.REGISTER,
            "test-obj-opaque-object": Operation.REGISTER,
            "test-obj-pgp-key": Operation.REGISTER,
            "test-obj-secret-data": Operation.REGISTER,
            "test-op-re-certify": Operation.RE_CERTIFY,
        }
        for version in ("2.0", "2.1"):
            for case_id, operation in cases.items():
                with self.subTest(version=version, case_id=case_id):
                    requests = []
                    for fluent in (False, True):
                        peer = ControlledTlsPeer(protocol_version=version, status=1, reason=5)
                        result = self._execute_attribute_route(
                            peer, high_level=False, case_id=case_id, encoding="TTLV",
                            transport="raw_tls", protocol_version=version, fluent=fluent,
                        )
                        self.assertEqual("server_unsupported", result["status"])
                        request, = peer.requests
                        self.assertEqual(int(operation), int.from_bytes(
                            find_value(parse_children(request[8:]), 0x42005C), "big",
                        ))
                        requests.append(request)
                    self.assertEqual(*requests)

    def test_generic_tags_reach_a_kmip2_peer_through_the_normal_runner(self):
        from kmipkit_validator.cli import _plan, _run

        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        selected = {"test-op-locate", "test-op-query", "test-op-discover-versions", "test-op-ping", "test-crypto-hash-sha_256-0", "test-crypto-rng-retrieve-rng-32", "test-crypto-rng-seed-rng-32"}
        catalog["test_cases"] = [case for case in catalog["test_cases"] if case["case_id"] in selected]
        for version in ("2.0", "2.1"):
            minor = int(version[-1])
            for case in catalog["test_cases"]:
                if version not in case["execution"]["protocol_versions"]:
                    continue
                case_id = case["case_id"]
                for expected in ("passed", "server_unsupported", "failed"):
                    with self.subTest(version=version, case=case_id, expected=expected):
                        query = case_id == "test-op-query"
                        ping = case_id == "test-op-ping"
                        hashing = case_id == "test-crypto-hash-sha_256-0"
                        rng_seed = case_id == "test-crypto-rng-seed-rng-32"
                        locate = case_id == "test-op-locate"
                        rng = case_id == "test-crypto-rng-retrieve-rng-32"
                        operation = {"test-op-locate": 0x08, "test-op-query": 0x18, "test-op-discover-versions": 0x1E,
                                     "test-crypto-hash-sha_256-0": 0x27, "test-crypto-rng-retrieve-rng-32": 0x25, "test-crypto-rng-seed-rng-32": 0x26, "test-op-ping": 0x3B}[case_id]

                        def reply(request):
                            tree = parse_children(request[8:])
                            payload = parse_children(find_value(tree, 0x420079))
                            if ping:
                                self.assertEqual([], payload)
                            elif rng:
                                self.assertEqual(payload, [(0x4200C4, 2, struct.pack(">i", 32))])
                            elif rng_seed:
                                self.assertEqual(payload, [(0x4200C2, 8, b"A" * 32)])
                            elif locate:
                                self.assertEqual(payload, [
                                    (0x42004F, 2, struct.pack(">i", 1)),
                                    (0x4200D4, 2, struct.pack(">i", 0)),
                                ])
                            elif hashing:
                                self.assertEqual(find_value(payload, 0x420038), struct.pack(">I", 6))
                                self.assertEqual(find_value(payload, 0x4200C2), b"abc")
                            elif query:
                                self.assertEqual(find_value(payload, 0x420074), struct.pack(">I", 1))
                            else:
                                self.assertEqual(find_value(payload, 0x42006B), struct.pack(">i", minor))
                            if expected == "server_unsupported":
                                return ping_response(request, 1, 5, protocol_minor=minor)
                            reply_minor = 1 - minor if expected == "failed" and not (hashing or locate or rng or rng_seed) else minor
                            if ping:
                                return ping_response(request, 0, None, protocol_minor=reply_minor)
                            if not (query or hashing or locate or rng or rng_seed):
                                return discover_versions_response(request, 0, None, reply_minor, ((2, minor),))
                            protocol = structure(0x420069, item(0x42006A, 2, struct.pack(">i", 2)),
                                                 item(0x42006B, 2, struct.pack(">i", reply_minor)))
                            header = structure(0x42007A, protocol, item(0x420092, 9, struct.pack(">q", 1)),
                                               item(0x42000D, 2, struct.pack(">i", 1)))
                            returned = (item(0x4200C2, 8, hashlib.sha256(
                                b"wrong" if expected == "failed" else b"abc").digest())
                                if hashing else enumeration(0x42005C, 1))
                            if rng:
                                returned = item(0x4200C2, 8, bytes(31 if expected == "failed" else 32))
                            if rng_seed:
                                returned = item(0x4200C4, 2, struct.pack(">i", 33 if expected == "failed" else 0))
                            if locate:
                                returned = item(0x4200D5, 2, struct.pack(">i", 2)) + item(0x420094, 7, b"known-id")
                                if expected == "failed":
                                    returned += item(0x420094, 7, b"second-id")
                            batch = structure(0x42000F, enumeration(0x42005C, operation),
                                              enumeration(0x42007F, 0), structure(0x42007C, returned))
                            return structure(0x42007B, header, batch)

                        peer = ControlledTlsPeer(response_factory=reply, protocol_version=version)
                        try:
                            connection = {
                                "server_id": "generic-loopback", "protocol_version": version,
                                "transport": "raw_tls", "encoding": "TTLV", "api_level": "generic-ttlv",
                                "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": peer.port}},
                                "tls": {
                                    "ca_cert_path": str(PKI / "server-cert.pem"),
                                    "client_cert_path": str(PKI / "client-cert.pem"),
                                    "client_key_path": str(PKI / "client-key.pem"),
                                    "server_name_override": "localhost",
                                },
                                "execution_policy": {"allow_mutations": rng or rng_seed, "probe_unsupported": True},
                            }
                            selection = {**catalog, "test_cases": [case]}
                            plan = _plan(connection, selection, b"{}")
                            self.assertTrue(plan["cases"][0]["route_available"])
                            result, = _run(connection, b"{}", selection, b"{}", {}, None, rng or rng_seed)
                            self.assertEqual(expected, result["status"], result)
                        finally:
                            peer.close()
                        self.assertEqual(1, len(peer.requests))

    def test_owned_key_operations_use_only_the_created_identifier(self):
        from kmipkit_validator.cli import _run

        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        routes = {"create": [1, 20], "destroy": [1, 20], "get": [1, 10, 20], "get-attributes": [1, 11, 20],
                  "check": [1, 18, 9, 19, 20], "activate": [1, 18, 19, 20], "revoke": [1, 18, 19, 20],
                  "get-attribute-list": [1, 12, 20], "archive": [1, 21, 22, 20], "recover": [1, 21, 22, 20]}
        mutations = {"add-attribute": 0x0D, "modify-attribute": 0x0E,
                     "delete-attribute": 0x0F, "set-attribute": 0x31}
        routes.update({name: [1, 11, code, 11, 20] for name, code in mutations.items()})
        combinations = [("generic-ttlv", target, operations) for target, operations in routes.items()]
        combinations += [(api, target, routes[target]) for api in ("typed-protocol", "fluent")
                         for target in ("activate", "revoke")]
        for version in ("2.0", "2.1"):
            for api, target, operations in combinations:
                for valid_get in ((True, False) if target in {"get", "get-attributes", "get-attribute-list", "activate", "revoke", "check", *mutations}
                                  and (target != "revoke" or api == "generic-ttlv") else (True,)):
                    with self.subTest(version=version, api=api, target=target, valid_get=valid_get):
                        minor = int(version[-1])
                        owned_id = b"owned-key-id"
                        seen = []
                        description = None if target == "add-attribute" else b"kmipkit-attribute-fixture"
                        updated = b"kmipkit-changed-description"

                        def answer(request):
                            nonlocal description
                            tree = parse_children(request[8:])
                            operation = int.from_bytes(find_value(tree, 0x42005C), "big")
                            seen.append(operation)
                            payload = parse_children(find_value(tree, 0x420079))
                            fields = item(0x420094, 7, owned_id)
                            if operation == 1:
                                self.assertEqual(find_value(payload, 0x420028), struct.pack(">I", 3))
                                self.assertEqual(find_value(payload, 0x42002A), struct.pack(">i", 256))
                                if target in mutations:
                                    self.assertEqual(description, find_value(payload, 0x4200FC))
                                fields = enumeration(0x420057, 2) + fields
                            else:
                                self.assertEqual(find_value(payload, 0x420094), owned_id)
                            if operation == 10:
                                key = structure(0x42008F, structure(0x420040,
                                    enumeration(0x420042, 1),
                                    structure(0x420045, item(0x420043, 8, bytes(32))),
                                    enumeration(0x420028, 3), item(0x42002A, 2, struct.pack(">i", 256))))
                                fields = (enumeration(0x420057, 2)
                                          + item(0x420094, 7, owned_id if valid_get else b"foreign-key-id") + key)
                            if target in mutations and operation == 11:
                                self.assertEqual(find_value(payload, 0x42013B), struct.pack(">I", 0x4200FC))
                                if description is not None:
                                    fields += structure(0x420125, item(0x4200FC, 7, description))
                            if operation == mutations.get(target):
                                current = find_value(payload, 0x42013C)
                                new = find_value(payload, 0x42013D)
                                if target == "modify-attribute":
                                    self.assertEqual([(0x4200FC, 7, description)], parse_children(current))
                                else:
                                    self.assertIsNone(current)
                                if target == "delete-attribute":
                                    self.assertEqual(find_value(payload, 0x42013B), struct.pack(">I", 0x4200FC))
                                    self.assertIsNone(new)
                                else:
                                    self.assertEqual([(0x4200FC, 7, updated)], parse_children(new))
                                if valid_get:
                                    description = None if target == "delete-attribute" else updated
                            if operation == 11 and target not in mutations:
                                self.assertEqual(find_value(payload, 0x42013B), struct.pack(">I", 0x42002A))
                                fields += structure(0x420125, item(0x42002A, 2, struct.pack(">i", 256 if valid_get else 128)))
                            if operation == 12:
                                fields += enumeration(0x42013B, 0x420028)
                                if valid_get:
                                    fields += enumeration(0x42013B, 0x42002A)
                            if operation == 9:
                                self.assertEqual(find_value(payload, 0x42002C), struct.pack(">i", 4))
                                fields += item(0x42002C, 2, struct.pack(">i", 4 if valid_get else 8))
                            if operation == 19:
                                self.assertEqual(parse_children(find_value(payload, 0x420081)),
                                                 [(0x420082, 5, struct.pack(">I", 6 if api == "generic-ttlv" else 1))])
                            if not valid_get and ((target == "activate" and operation == 18)
                                                  or (target == "revoke" and operation == 19)):
                                fields = item(0x420094, 7, b"foreign-key-id")
                            protocol = structure(0x420069, item(0x42006A, 2, struct.pack(">i", 2)),
                                                 item(0x42006B, 2, struct.pack(">i", minor)))
                            header = structure(0x42007A, protocol, item(0x420092, 9, struct.pack(">q", 1)),
                                               item(0x42000D, 2, struct.pack(">i", 1)))
                            batch = structure(0x42000F, enumeration(0x42005C, operation),
                                              enumeration(0x42007F, 0), structure(0x42007C, fields))
                            return structure(0x42007B, header, batch)

                        peer = ControlledTlsPeer(response_factory=answer, protocol_version=version,
                                                 connection_count=len(operations))
                        try:
                            connection = {
                                "server_id": "owned-key-loopback", "protocol_version": version,
                                "transport": "raw_tls", "encoding": "TTLV", "api_level": api,
                                "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": peer.port}},
                                "tls": {"ca_cert_path": str(PKI / "server-cert.pem"),
                                        "client_cert_path": str(PKI / "client-cert.pem"),
                                        "client_key_path": str(PKI / "client-key.pem"),
                                        "server_name_override": "localhost",
                                        "connect_timeout_seconds": 2, "read_timeout_seconds": 2},
                                "execution_policy": {"allow_mutations": True, "probe_unsupported": True},
                            }
                            case = next(row for row in catalog["test_cases"] if row["case_id"] == f"test-op-{target}")
                            selection = {**catalog, "test_cases": [case]}
                            result, = _run(connection, b"{}", selection, b"{}", {}, None, True)
                            self.assertEqual("passed" if valid_get else "failed", result["status"], result)
                            self.assertEqual(operations, seen)
                        finally:
                            peer.close()

    def test_rejected_ml_kem_setup_is_not_reported_as_a_target_exchange(self):
        for operation in ("Encapsulate", "Decapsulate"):
            for api in ("typed-protocol", "fluent"):
                with self.subTest(operation=operation, api=api):
                    driver = PythonKmipDriver.__new__(PythonKmipDriver)
                    driver._api_level = api
                    driver._extensions = MagicMock()
                    driver._fixture_attributes = MagicMock()
                    driver._build_draft_item = Mock()
                    driver._exchange = Mock(return_value={
                        "status": "server_unsupported", "result_status_code": 1, "result_reason_code": 5,
                    })
                    case = {"case_id": f"test-op-{operation.lower()}", "operation": operation}
                    outcome = driver.execute_case(case)
                    self.assertEqual("not_run", outcome["status"])
                    self.assertEqual("unknown", outcome["server_compatibility"])
                    self.assertNotIn("result_reason_code", outcome)
                    self.assertIn("ML-KEM fixture key pair", outcome["diagnostic"])
                    driver._extensions.client_batch_item_create_key_pair.assert_called_once()
                    driver._exchange.assert_called_once()
                    driver._build_draft_item.assert_not_called()

    def test_locate_cleanup_destroys_only_the_owned_fixture(self):
        from kmipkit import extensions

        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        case = next(case for case in catalog["test_cases"] if case["case_id"] == "test-op-locate")
        owned = extensions.UniqueIdentifier.text_string("owned-fixture")
        unrelated = extensions.UniqueIdentifier.text_string("preexisting-server-object")
        for api in ("typed-protocol", "fluent"):
            with self.subTest(api=api):
                driver = PythonKmipDriver.__new__(PythonKmipDriver)
                driver._api_level = api
                driver._extensions = Mock()
                driver._fixture_attributes = Mock()
                driver._build_item = Mock()
                driver._build_fluent_item = Mock()
                driver._exchange = Mock(side_effect=[
                    {"status": "passed", "_identifiers": [owned]},
                    {"status": "passed", "_identifiers": [owned, unrelated]},
                    {"status": "passed"},
                ])
                self.assertEqual("passed", driver.execute_case(case)["status"])
                self.assertEqual([call(owned)], driver._extensions.client_batch_item_destroy.call_args_list)
                if api == "fluent":
                    driver._build_fluent_item.assert_called_once()
                else:
                    driver._build_item.assert_called_once()

    def test_fluent_and_typed_requests_match_for_catalogued_attributes(self):
        from kmipkit import protocol_values

        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
        for version in ("2.0", "2.1"):
            cases = [
                case["case_id"] for case in catalog["test_cases"]
                if version in case["execution"]["protocol_versions"]
                and PythonKmipDriver.supports(case, "fluent")
                and case["requires"].get("expected_status") != "NotSent"
                and (
                    case["requires"].get("attribute") in (None, "Certificate Attributes", "Vendor Attribute")
                    or PythonKmipDriver._attribute_tag(case["requires"]["attribute"]) is not None
                )
            ]
            self.assertGreater(len(cases), 20)
            for case_id in cases:
                with self.subTest(case_id=case_id, version=version):
                    observations = []
                    for fluent in (False, True):
                        peer = ControlledTlsPeer(
                            status=protocol_values.ResultStatus.OPERATION_FAILED,
                            reason=protocol_values.ResultReason.OPERATION_NOT_SUPPORTED,
                            protocol_version=version,
                        )
                        result = self._execute_attribute_route(
                            peer, high_level=False, fluent=fluent, case_id=case_id,
                            protocol_version=version, encoding="TTLV", transport="raw_tls",
                        )
                        observations.append((peer.requests[0], peer.responses[0], result["status"]))
                    self.assertEqual(observations[0], observations[1])
                    self.assertEqual(observations[0][2], "server_unsupported")

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
