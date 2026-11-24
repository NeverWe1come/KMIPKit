"""Collect scoped interoperability evidence from the installed wheel."""

import argparse
import json
from pathlib import Path
import sys
from uuid import uuid4

from .cli import _is_installed_package
from .contract import load_connection, validate_artifact, validate_local_inputs
from .driver import PythonKmipDriver, _catalog_model_data
from .oasis_query import oasis_query_evidence

OPERATION_CODES = {
    "client_batch_item_create_group": 0x3C,
    "client_batch_item_obliterate": 0x3D,
    "client_batch_item_create_user": 0x3E,
    "client_batch_item_create_credential": 0x3F,
    "client_batch_item_deactivate": 0x40,
    "client_batch_item_encapsulate": 0x41,
    "client_batch_item_decapsulate": 0x42,
}


def draft_events(lines):
    from kmipkit import extensions, ttlv

    for line in lines:
        row = json.loads(line)
        if (row.get("language"), row.get("protocol_version"), row.get("kind"),
            row.get("action"), row.get("execution_status")) != ("python", "3.0", "public_route", "construct", "required"):
            continue
        symbol = row["public_api_symbol"]
        if not symbol.startswith("kmipkit.extensions."):
            continue
        name = symbol.removeprefix("kmipkit.extensions.")
        expected = OPERATION_CODES.get(name)
        if expected is None:
            continue
        try:
            with ttlv.ttlv_structure_create() as payload, getattr(extensions, name)(payload) as item:
                passed = item.operation == expected
        except Exception:
            # Native exception messages can hold unredacted data.
            passed = False
        yield {
            "obligation_id": row["obligation_id"], "public_api_symbol": symbol,
            "status": "passed" if passed else "failed", "assertion": "draft_operation_code",
        }


def local_events(lines):
    """Check exact generic TTLV symbols and concrete normative tag/type targets."""
    targets, _, _ = _catalog_model_data()
    for line in lines:
        row = json.loads(line)
        if row.get("protocol_version") == "3.0":
            yield from draft_events((line,))
            continue
        if (row.get("language"), row.get("mode"), row.get("execution_status")) != ("python", "local", "required"):
            continue
        target = targets.get(row.get("target_id"))
        if target is None:
            continue
        symbol = row["public_api_symbol"]
        api = symbol.removeprefix("kmipkit.ttlv.")
        if symbol == api:
            continue
        try:
            assertion = _check_local_target(api, row["action"], target)
            passed = True
        except Exception:
            # Keep native details and fixture values out of campaign evidence.
            assertion = "generic_ttlv_check_failed"
            passed = False
        if assertion is not None:
            yield {
                "obligation_id": row["obligation_id"], "public_api_symbol": symbol,
                "status": "passed" if passed else "failed", "assertion": assertion,
            }


def _check_local_target(api, action, target):
    if target["kind"] in {"tag", "tag_range"} and action == "construct" and api in {
        "raw_tag_from_raw", "raw_tag_try_checked",
    }:
        PythonKmipDriver._execute_catalog_tag({"requires": {
            "wire_value": target["wire_value"], "target_kind": target["kind"], "allocation": target["allocation"],
        }}, check_allocation=api == "raw_tag_try_checked")
        return "generic_ttlv_tag_allocation" if api == "raw_tag_try_checked" else "generic_ttlv_raw_tag_value"
    if target["kind"] != "data_type":
        return None
    if (api, action) == ("ttlv_item_create", "construct"):
        PythonKmipDriver._exercise_generic_type(target["name"])
        return "generic_ttlv_construct_boundary_values"
    if action == "decode" and api in {"ttlv_decode", "ttlv_item_view_from_item"}:
        PythonKmipDriver._execute_data_type_case(target["name"], bounded=api == "ttlv_item_view_from_item")
        return "generic_ttlv_decode_boundary_vectors"
    return None


def query_evidence(lines, connection):
    """Use the exact typed Query route for the selected live wire configuration."""
    constructor = "kmipkit.extensions.client_batch_item_query"
    executor = "kmipkit.client.Client.execute"
    eligible = {
        "language": "python", "protocol_version": connection["protocol_version"],
        "encoding": connection["encoding"], "transport": connection["transport"],
        "case_id": "test-op-query", "target_id": "KMIPKIT-ELEM-OP-C2S-QUERY",
        "public_api_symbol": constructor, "execution_api_symbol": executor, "execution_status": "required",
    }
    rows = []
    for line in lines:
        row = json.loads(line)
        if row.get("action") == "send" and all(row.get(key) == value for key, value in eligible.items()):
            rows.append(row)
    if len(rows) != 1:
        raise ValueError("The plan must contain one matching typed Query send obligation")
    from kmipkit.errors import ClientExecutionError
    from kmipkit.protocol_values import ResultReason

    driver = PythonKmipDriver(connection)
    try:
        response = driver.query_operations_and_objects()
    except (ClientExecutionError, ValueError):
        response = {"status": "failed", "server_compatibility": "unknown",
                    "result_status_code": None, "result_reason_code": None}
    finally:
        driver.close()
    compatibility = response["server_compatibility"]
    status = "passed" if response["status"] == "passed" else "failed"
    if response["result_reason_code"] in (ResultReason.PERMISSION_DENIED, ResultReason.AUTHENTICATION_NOT_SUCCESSFUL):
        status = "blocked"
        compatibility = "policy_restricted"
    event = {
        "obligation_id": rows[0]["obligation_id"], "public_api_symbol": constructor,
        "execution_api_symbol": executor, "status": status,
        "encoding": connection["encoding"], "transport": connection["transport"],
        "server_compatibility": compatibility, "assertion": "query_operations_and_objects",
    }
    for field in ("result_status_code", "result_reason_code"):
        if response[field] is not None:
            event[field] = response[field]
    return [event]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--matrix-sha256", required=True)
    parser.add_argument("--scope", choices=("local", "controlled", "live"), required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--profile", type=Path)
    parser.add_argument("--connection", type=Path)
    args = parser.parse_args()
    try:
        if not _is_installed_package():
            raise ValueError("An installed KMIPKit wheel is required")
        plan_path = args.plan.resolve(strict=True)
        if plan_path.name != "plan.jsonl" or args.evidence.parent.resolve(strict=True) != plan_path.parent:
            raise ValueError("Campaign evidence must be written beside the shared plan.jsonl")
        match args.scope:
            case "local":
                evidence = plan_path.with_name("evidence-python-local.json")
            case "controlled":
                evidence = plan_path.with_name("evidence-python-controlled.json")
            case "live":
                evidence = plan_path.with_name("evidence-python-live.json")
            case _:
                raise ValueError("Invalid campaign scope")
        if args.evidence.name != evidence.name:
            raise ValueError("Evidence filename does not match the campaign scope")
        provenance = validate_artifact()
        payload = {
            "schema_version": "1.0", "run_id": str(uuid4()), "scope": args.scope, "language": "python",
            "matrix_sha256": args.matrix_sha256, "library_commit": provenance["library_commit"],
            "artifact_sha256": provenance["artifact_sha256"],
        }
        with plan_path.open(encoding="utf-8") as source:
            if args.scope == "local":
                if args.connection is not None:
                    raise ValueError("Local campaigns do not take a connection")
                events = list(local_events(source))
            elif args.scope == "controlled":
                if args.connection is not None:
                    raise ValueError("The controlled OASIS scenario supplies its own loopback peer")
                events = oasis_query_evidence(source)
            else:
                if args.connection is None:
                    raise ValueError("Live campaigns require a connection")
                connection, _ = load_connection(args.connection)
                if connection["protocol_version"] not in {"2.0", "2.1"} or connection["api_level"] != "typed-protocol":
                    raise ValueError("The live campaign requires KMIP 2.0/2.1 and the typed protocol route")
                if validate_local_inputs(connection):
                    raise ValueError("Live connection inputs are incomplete")
                payload["server"] = {
                    "id": connection["server_id"], "product": connection["server_product"],
                    "version": connection["server_version"],
                    "configuration_fingerprint": connection["server_configuration_fingerprint"],
                }
                events = query_evidence(source, connection)
        if not events:
            raise ValueError("The plan contains no supported Python obligations")
        payload["events"] = events
        with evidence.open("x", encoding="utf-8") as output:
            json.dump(payload, output)
            output.write("\n")
        return int(any(event["status"] == "failed" for event in events))
    except (OSError, ValueError, KeyError, ImportError):
        print("Python campaign failed. Check the plan, connection and installed artifact.", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
