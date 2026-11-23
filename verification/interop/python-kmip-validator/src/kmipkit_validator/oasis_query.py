"""Execute the pinned KMIP 3.0 Query profile case against a controlled TLS peer."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET

from .contract import ROOT
from .controlled_peer import ControlledTlsPeer, PKI, enumeration, find_value, item, parse_children, structure
from .driver import PythonKmipDriver

CASE_ID = "QS-M-1-30"
FIXTURE = Path(f"verification/interop/shared/catalog/oasis/kmip-3.0/mandatory/{CASE_ID}.xml")


def _wire_values(enum, names):
    values = {name.replace("_", "").casefold(): value for name, value in vars(enum).items()
              if name.isupper() and type(value) is int}
    return tuple(values[name.casefold()] for name in names)


def _query_fixture(expected_digest):
    source = (ROOT / FIXTURE).read_bytes()
    if hashlib.sha256(source).hexdigest() != expected_digest:
        raise ValueError("OASIS Query fixture SHA-256 differs from the plan")
    pair = ET.fromstring(source)
    if [child.tag for child in pair] != ["RequestMessage", "ResponseMessage"]:
        raise ValueError("The Query profile fixture must contain exactly one ordered exchange")
    request, response = pair
    client_correlation = request.find("RequestHeader/ClientCorrelationValue").attrib["value"]
    functions = [field.attrib["value"] for field in request.findall("BatchItem/RequestPayload/QueryFunction")]
    result_status = response.find("BatchItem/ResultStatus").attrib["value"]
    operations = [field.attrib["value"] for field in response.findall("BatchItem/ResponsePayload/Operation")]
    if (request.find("BatchItem/Operation").attrib["value"],
        response.find("BatchItem/Operation").attrib["value"]) != ("Query", "Query"):
        raise ValueError("The profile case requires the Query operation")
    if not functions or not operations or result_status != "Success":
        raise ValueError("The profile case requires a successful Query Operations response")
    return client_correlation, functions, operations


def _respond(request, correlation, expected_functions, operations):
    fields = parse_children(request[8:])
    header = parse_children(find_value(fields, 0x420077))
    version = parse_children(find_value(header, 0x420069))
    if int.from_bytes(find_value(version, 0x42006A), "big") != 3 or int.from_bytes(find_value(version, 0x42006B), "big") != 0:
        raise AssertionError("Query request does not use the profile protocol version")
    if find_value(header, 0x420105) != correlation.encode("utf-8"):
        raise AssertionError("Query request changed the profile correlation field")
    batches = [parse_children(value) for tag, kind, value in fields if tag == 0x42000F and kind == 1]
    if len(batches) != 1 or int.from_bytes(find_value(batches[0], 0x42005C), "big") != 0x18:
        raise AssertionError("Query fixture expects a single Query batch item")
    payload = parse_children(find_value(batches[0], 0x420079))
    functions = tuple(int.from_bytes(value, "big") for tag, kind, value in payload if tag == 0x420074 and kind == 5)
    if functions != expected_functions:
        raise AssertionError("Query request changed the ordered query functions")
    return structure(
        0x42007B,
        structure(0x42007A, structure(0x420069, item(0x42006A, 2, (3).to_bytes(4, "big")),
                                    item(0x42006B, 2, bytes(4))),
                  item(0x420092, 9, (1_794_268_800).to_bytes(8, "big"))),
        structure(0x42000F, enumeration(0x42005C, 0x18), enumeration(0x42007F, 0),
                  structure(0x42007C, *(enumeration(0x42005C, code) for code in operations))),
    )


def _exercise_query(functions, operations, correlation):
    from kmipkit import client, errors, extensions, ttlv

    peer = ControlledTlsPeer(protocol_version="3.0", response_factory=lambda request: _respond(
        request, correlation, functions, operations,
    ))
    try:
        driver = PythonKmipDriver({
            "server_id": "controlled-query", "protocol_version": "3.0", "transport": "raw_tls",
            "encoding": "TTLV", "api_level": "typed-protocol",
            "endpoints": {"raw_tls": {"host": "127.0.0.1", "port": peer.port}},
            "tls": {
                "ca_cert_path": str(PKI / "server-cert.pem"), "client_cert_path": str(PKI / "client-cert.pem"),
                "client_key_path": str(PKI / "client-key.pem"), "server_name_override": "localhost",
                "connect_timeout_seconds": 5, "read_timeout_seconds": 5,
            },
        })
        try:
            with (client.Batch.create() as batch,
                  extensions.client_batch_item_query(functions) as request,
                  ttlv.codec_limits_defaults() as limits):
                batch.add_item(request).set_client_correlation_value(correlation)
                with driver._client.execute(batch, limits) as response:
                    observed = response.item_at(0)
                    query = observed.typed_query_response()
                    if observed.operation != 0x18 or observed.result_status != 0 or query is None:
                        return False
                    with query:
                        return query.operations == operations and len(peer.requests) == 1
        except errors.ClientExecutionError as error:
            if error.delivery is errors.DeliveryState.RESPONSE_STARTED and len(peer.responses) == 1:
                return False
            raise
        finally:
            driver.close()
    finally:
        peer.close()


def oasis_query_evidence(lines):
    from kmipkit.protocol_values import Operation, QueryFunction

    rows = []
    for line in lines:
        row = json.loads(line)
        if row.get("kind") == "scenario_item" and row.get("case_id") == CASE_ID:
            rows.append(row)
    if (len(rows) != 1 or rows[0]["fixture"]["path"] != FIXTURE.as_posix()
            or rows[0]["protocol_version"] != "3.0" or rows[0]["step_index"] != 0 or rows[0]["item_index"] != 0):
        raise ValueError("The plan must contain the pinned KMIP 3.0 Query scenario")
    row = rows[0]
    correlation, functions, operations = _query_fixture(row["fixture"]["sha256"])
    passed = _exercise_query(_wire_values(QueryFunction, functions), _wire_values(Operation, operations), correlation)
    event = {
        "obligation_id": row["obligation_id"], "public_api_symbol": "kmipkit.extensions.client_batch_item_query",
        "execution_api_symbol": "kmipkit.client.Client.execute", "status": "passed" if passed else "failed",
        "encoding": "TTLV", "transport": "raw_tls",
        "assertion": "oasis_query_operations_in_order" if passed else "oasis_query_response_rejected",
    }
    if passed:
        event["result_status_code"] = 0
    return [event]
