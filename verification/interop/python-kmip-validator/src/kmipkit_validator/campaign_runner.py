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
    if sys.argv[1:] in (["--inspect-oasis-ttlv"], ["--construct-oasis-ttlv"]):
        return inspect_ttlv(construct=sys.argv[1] == "--construct-oasis-ttlv")
    if sys.argv[1:] == ["--inspect-table-version"]:
        return inspect_version()
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


def inspect_version():
    from kmipkit.protocol import VersionGate
    from kmipkit.protocol_values import KeyFormatType, ObjectType

    if not _is_installed_package():
        raise ValueError("An installed KMIPKit wheel is required")
    artifact = validate_artifact()
    print(json.dumps({key: artifact[key] for key in ("library_commit", "artifact_sha256")}))
    lookup = {"key-format": KeyFormatType.name, "object-type": ObjectType.name}
    for line in sys.stdin:
        kind, raw = line.split()
        code = int(raw)
        for minor in (0, 1):
            assigned = VersionGate.validate_tag(2, minor, code) if kind == "tag" else lookup[kind](code) is not None
            print(json.dumps(assigned))
    return 0


def inspect_ttlv(*, construct=False):
    from kmipkit import errors, ttlv
    from .generic import _encode

    if not _is_installed_package():
        raise ValueError("An installed KMIPKit wheel is required")
    artifact = validate_artifact()
    print(json.dumps({key: artifact[key] for key in ("library_commit", "artifact_sha256")}))
    for line in sys.stdin:
        wire = bytes.fromhex(line.strip())
        try:
            with ttlv.codec_limits_defaults() as limits:
                item = build_ttlv_item(wire, limits) if construct else ttlv.ttlv_decode(wire)
                with item, ttlv.ttlv_item_view_from_item(item) as view:
                    if construct and _encode(view) != wire:
                        raise ValueError("constructed fixture changed on re-encoding")
                    nodes = []
                    append_ttlv_node(view, nodes)
        except (errors.KmipKitError, ValueError):
            nodes = None
        print(json.dumps(nodes, separators=(",", ":")))
    return 0


def build_ttlv_item(wire, limits, depth=0):
    from kmipkit import ttlv

    if len(wire) < 8 or depth >= 32:
        raise ValueError("invalid fixture header")
    size = int.from_bytes(wire[4:8], "big")
    end = 8 + size
    if end + (-size % 8) != len(wire) or any(wire[end:]):
        raise ValueError("invalid fixture length or padding")
    code, kind, body = int.from_bytes(wire[:3], "big"), wire[3], wire[8:end]
    scalars = {
        2: (4, True, ttlv.ttlv_value_integer),
        3: (8, True, ttlv.ttlv_value_long_integer),
        5: (4, False, ttlv.ttlv_value_enumeration),
        9: (8, True, ttlv.ttlv_value_date_time),
        10: (4, False, ttlv.ttlv_value_interval),
        11: (8, True, ttlv.ttlv_value_date_time_extended),
    }
    with ttlv.raw_tag_from_raw(code) as raw, ttlv.raw_tag_try_checked(raw) as tag:
        if kind == 1:
            structure = ttlv.ttlv_structure_create()
            try:
                offset = 0
                while offset < len(body):
                    if len(body) - offset < 8:
                        raise ValueError("truncated structure child")
                    length = int.from_bytes(body[offset + 4:offset + 8], "big")
                    next_offset = offset + 8 + length + (-length % 8)
                    with build_ttlv_item(body[offset:next_offset], limits, depth + 1) as child:
                        structure = ttlv.ttlv_structure_with_item(structure, child, limits)
                    offset = next_offset
                value = ttlv.ttlv_value_structure(structure, limits)
            finally:
                structure.close()
        elif kind in scalars:
            width, signed, constructor = scalars[kind]
            if len(body) != width:
                raise ValueError("invalid scalar length")
            value = constructor(int.from_bytes(body, "big", signed=signed))
        elif kind == 6 and len(body) == 8 and int.from_bytes(body, "big") in (0, 1):
            value = ttlv.ttlv_value_boolean(bool(body[-1]))
        elif kind == 4 and body and len(body) % 8 == 0:
            value = ttlv.ttlv_value_big_integer(body, limits)
        elif kind == 7:
            value = ttlv.ttlv_value_text_string(body, limits)
        elif kind == 8:
            value = ttlv.ttlv_value_byte_string(body, limits)
        else:
            raise ValueError("invalid fixture type")
        with value:
            return ttlv.ttlv_item_create(tag, value, limits)


def append_ttlv_node(view, nodes):
    from kmipkit import ttlv

    with ttlv.ttlv_item_view_tag(view) as tag:
        raw = ttlv.tag_value(tag)
    kind = ttlv.ttlv_item_view_type(view)
    with ttlv.ttlv_item_view_value(view) as value:
        if kind != ttlv.ItemType.Structure:
            scalar_readers = {
                ttlv.ItemType.Integer: ttlv.ttlv_value_view_integer,
                ttlv.ItemType.LongInteger: ttlv.ttlv_value_view_long_integer,
                ttlv.ItemType.Enumeration: ttlv.ttlv_value_view_enumeration,
                ttlv.ItemType.Boolean: ttlv.ttlv_value_view_boolean,
                ttlv.ItemType.DateTime: ttlv.ttlv_value_view_date_time,
                ttlv.ItemType.Interval: ttlv.ttlv_value_view_interval,
                ttlv.ItemType.DateTimeExtended: ttlv.ttlv_value_view_date_time_extended,
            }
            reader = scalar_readers.get(kind)
            scalar = reader(value) if reader else ttlv.ttlv_value_view_bytes(value).hex()
            nodes.append((raw, kind, 0, scalar))
            return
        with ttlv.ttlv_value_view_structure(value) as structure:
            count = ttlv.ttlv_structure_view_item_count(structure)
            nodes.append((raw, kind, count))
            for index in range(count):
                with ttlv.ttlv_structure_view_item_at(structure, index) as child:
                    append_ttlv_node(child, nodes)


if __name__ == "__main__":
    raise SystemExit(main())
