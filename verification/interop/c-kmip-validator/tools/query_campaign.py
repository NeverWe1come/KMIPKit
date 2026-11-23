"""Join one installed C ABI Query execution to the matching wire obligation."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory

VALIDATOR = Path(__file__).resolve().parents[1]
CATALOG = VALIDATOR.parent / "shared/catalog/kmipkit-2x-matrix-catalog.json"
QUERY = {
    "language": "c", "case_id": "test-op-query", "target_id": "KMIPKIT-ELEM-OP-C2S-QUERY",
    "public_api_symbol": "kmipkit_client_batch_item_query", "execution_api_symbol": "kmipkit_client_execute",
    "action": "send", "execution_status": "required",
}


def live_query(plan, connection_path):
    connection_path = connection_path.resolve(strict=True)
    connection = json.loads(connection_path.read_text(encoding="utf-8"))
    if connection["protocol_version"] not in {"2.0", "2.1"} or connection["api_level"] != "typed-protocol":
        raise ValueError("The C Query probe requires KMIP 2.0/2.1 and the typed API")
    wire = {key: connection[key] for key in ("encoding", "transport")}
    required = {**QUERY, **wire, "protocol_version": connection["protocol_version"]}
    matching = [row for row in plan if all(row.get(key) == value for key, value in required.items())]
    if len(matching) != 1:
        raise ValueError("The plan must contain exactly one matching C Query send obligation")

    # The JSON connection path is passed as a single, absolute option value.
    with TemporaryDirectory(prefix="kmip-c-query-") as work:
        report_file = Path(work) / "query-results.json"
        subprocess.run(
            [str(VALIDATOR / "build/kmip_validator_c"), "run", "--case", "test-op-query",
             "--connection", str(connection_path), "--catalog", str(CATALOG), "--output", str(report_file)],
            check=False, capture_output=True,
        )
        if not report_file.is_file():
            raise ValueError("The C validator did not produce Query evidence")
        report = json.loads(report_file.read_text(encoding="utf-8"))

    if report["protocol_version"] != connection["protocol_version"] or any(
        report["execution"][key] != value for key, value in wire.items()
    ):
        raise ValueError("C report wire settings differ from the connection")
    if len(report["cases"]) != 1 or report["cases"][0]["case_id"] != QUERY["case_id"]:
        raise ValueError("C validator did not isolate the selected Query case")
    result = report["cases"][0]
    status = {
        "passed": "passed", "failed": "failed", "not_run": "blocked",
        "policy_restricted": "blocked", "server_unsupported": "failed",
    }[result["status"]]
    compatibility = result["server_compatibility"]
    if result.get("result_reason_code") in (3, 12):
        status = "blocked"
        compatibility = "policy_restricted"
    event = {
        **wire, "obligation_id": matching[0]["obligation_id"],
        "public_api_symbol": QUERY["public_api_symbol"], "execution_api_symbol": QUERY["execution_api_symbol"],
        "status": status, "server_compatibility": compatibility,
        "assertion": "query_operations_and_typed_response",
    }
    for field in ("result_status_code", "result_reason_code"):
        if field in result:
            event[field] = result[field]
    server = {key: report["server"][key] for key in ("id", "product", "version", "configuration_fingerprint")}
    return server, report["execution"], event
