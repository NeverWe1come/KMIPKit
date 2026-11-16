from __future__ import annotations

import argparse
from datetime import datetime, timezone
import importlib.util
import json
import sys
import uuid
from pathlib import Path
from typing import Any

from .contract import (
    ARTIFACT_MANIFEST,
    DEFAULT_CATALOG,
    RESULT_SCHEMA,
    _validate,
    applicable_cases,
    load_catalog as load_contract_catalog,
    load_connection,
    load_json,
    sha256,
    validate_artifact,
    validate_local_inputs,
)
from .driver import LocalRecipeUnavailable, PythonKmipDriver
from .reporter import write_interop_reports


def _load_profile(path: Path | None, server_id: str) -> dict[str, Any] | None:
    if path is None:
        return None
    profile, _ = load_json(path, "capability profile is unavailable or invalid")
    if profile.get("server", {}).get("id") != server_id:
        raise ValueError("capability profile server identity does not match connection")
    return profile


def _server_metadata(profile: dict[str, Any] | None) -> tuple[str, str]:
    if profile is None:
        return "unverified server", "unverified"
    server = profile.get("server", {})
    product = server.get("product")
    version = server.get("version")
    return (
        product if isinstance(product, str) and product else "unverified server",
        version if isinstance(version, str) and version else "unverified",
    )


def _is_installed_package() -> bool:
    try:
        from importlib.metadata import distribution

        package = distribution("kmipkit")
        module = importlib.util.find_spec("kmipkit")
        if module is None or module.origin is None:
            return False
        package_root = Path(module.origin).resolve()
        return package_root.is_relative_to(Path(sys.prefix).resolve()) and package.version == "1.0.0"
    except (ImportError, OSError, ModuleNotFoundError, ValueError):
        return False


def _case_record(
    case_id: str,
    status: str,
    library: str,
    server: str,
    diagnostic: str,
    duration_ms: float = 0.0,
    result_status_code: int | None = None,
    result_reason_code: int | None = None,
) -> dict[str, Any]:
    record: dict[str, Any] = {
        "case_id": case_id,
        "status": status,
        "library_correctness": library,
        "server_compatibility": server,
        "parity": "not_compared",
        "duration_ms": duration_ms,
        "diagnostic": diagnostic,
    }
    if result_status_code is not None:
        record["result_status_code"] = result_status_code
    if result_reason_code is not None:
        record["result_reason_code"] = result_reason_code
    return record


def _make_report(
    connection: dict[str, Any],
    connection_bytes: bytes,
    catalog: dict[str, Any],
    catalog_bytes: bytes,
    artifact: dict[str, str],
    profile: dict[str, Any] | None,
    cases: list[dict[str, Any]],
) -> dict[str, Any]:
    product, server_version = _server_metadata(profile)
    statuses = ("passed", "failed", "server_unsupported", "policy_restricted", "not_run")
    report = {
        "schema_version": "2.0",
        "run_id": f"python-{uuid.uuid4()}",
        "tested_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "language": "python",
        "server": {
            "id": connection["server_id"],
            "product": product,
            "version": server_version,
            "configuration_fingerprint": connection["server_configuration_fingerprint"],
        },
        "protocol_version": connection["protocol_version"],
        "catalog_sha256": sha256(catalog_bytes),
        "execution": {
            "transport": connection["transport"],
            "encoding": connection["encoding"],
            "api_level": connection["api_level"],
            "library_commit": artifact["library_commit"],
            "artifact_sha256": artifact["artifact_sha256"],
        },
        "summary": {
            "total": len(cases),
            "passed": sum(case["status"] == "passed" for case in cases),
            "failed": sum(case["status"] == "failed" for case in cases),
            "unsupported": sum(case["status"] == "server_unsupported" for case in cases),
            "policy_restricted": sum(case["status"] == "policy_restricted" for case in cases),
            "not_run": sum(case["status"] == "not_run" for case in cases),
        },
        "cases": cases,
    }
    del connection_bytes, catalog
    _validate(report, RESULT_SCHEMA, "generated results violate the shared schema")
    if any(case["status"] not in statuses for case in cases):
        raise ValueError("generated results contain an unknown status")
    return report


def _plan(connection: dict[str, Any], catalog: dict[str, Any], catalog_bytes: bytes) -> dict[str, Any]:
    cases = applicable_cases(catalog, connection["protocol_version"])
    planned_cases = []
    for case in cases:
        local = case["execution"]["mode"] == "local_model"
        limitation = (
            PythonKmipDriver.local_recipe_limitation(case)
            if local
            else PythonKmipDriver.route_limitation(case, connection["api_level"])
        )
        available = limitation is None and (
            local or PythonKmipDriver.supports(case, connection["api_level"])
        )
        planned_case = {
            "case_id": case["case_id"],
            "operation": case["operation"],
            "mode": case["execution"]["mode"],
            "route_available": available,
            "status": "planned" if available else "not_run_documented_api_gap",
        }
        if limitation is not None:
            planned_case["limitation"] = limitation
        planned_cases.append(planned_case)
    return {
        "language": "python",
        "protocol_version": connection["protocol_version"],
        "catalog_sha256": sha256(catalog_bytes),
        "network_attempted": False,
        "capability_policy": "profiles_are_hypotheses_probe_applicable_cases",
        "cases": planned_cases,
    }


def _run(
    connection: dict[str, Any],
    connection_bytes: bytes,
    catalog: dict[str, Any],
    catalog_bytes: bytes,
    artifact: dict[str, str],
    profile: dict[str, Any] | None,
    enable_mutations: bool,
) -> list[dict[str, Any]]:
    applicable = applicable_cases(catalog, connection["protocol_version"])
    policy = connection["execution_policy"]
    executable = [
        case
        for case in applicable
        if PythonKmipDriver.supports(case, connection["api_level"])
        and not (
            PythonKmipDriver.requires_mutation(case)
            and not (policy.get("allow_mutations") is True and enable_mutations)
        )
    ]
    driver: PythonKmipDriver | None = None
    driver_error = False
    if executable:
        try:
            driver = PythonKmipDriver(connection)
        except Exception:
            # Native and TLS error text is deliberately excluded from evidence.
            driver_error = True

    results: list[dict[str, Any]] = []
    for case in applicable:
        case_id = case["case_id"]
        if (
            PythonKmipDriver.requires_mutation(case)
            and not (policy.get("allow_mutations") is True and enable_mutations)
        ):
            results.append(
                _case_record(case_id, "policy_restricted", "not_checked", "policy_restricted", "mutation opt-in required")
            )
            continue
        if case["execution"]["mode"] == "local_model":
            try:
                PythonKmipDriver.execute_local_case(case)
                results.append(
                    _case_record(case_id, "passed", "passed", "unknown", "catalog TTLV and API assertions passed")
                )
            except LocalRecipeUnavailable as error:
                results.append(
                    _case_record(case_id, "not_run", "not_checked", "unknown", str(error))
                )
            except Exception:
                results.append(
                    _case_record(case_id, "failed", "failed", "unknown", "local TTLV model assertion failed")
                )
            continue
        if not PythonKmipDriver.supports(case, connection["api_level"]):
            reason = PythonKmipDriver.route_limitation(case, connection["api_level"])
            results.append(_case_record(case_id, "not_run", "not_checked", "unknown", reason))
            continue
        if not policy.get("probe_unsupported", False) and profile is not None:
            operation = profile.get("operations", {}).get(case["operation"], {})
            if isinstance(operation, dict) and operation.get("supported") is False:
                results.append(
                    _case_record(case_id, "policy_restricted", "not_checked", "policy_restricted", "unsupported capability probe is disabled")
                )
                continue
        if driver_error or driver is None:
            results.append(
                _case_record(case_id, "failed", "not_checked", "unknown", "KMIPKit client configuration failed")
            )
            continue
        started = datetime.now(timezone.utc)
        try:
            observation = driver.execute_case(case)
            elapsed = (datetime.now(timezone.utc) - started).total_seconds() * 1000
            expected_reasons = case.get("requires", {}).get("expected_result_reasons", [])
            if (
                expected_reasons
                and observation.get("result_status_code") == 1
                and observation.get("result_reason_code") in expected_reasons
            ):
                observation.update(
                    status="passed",
                    library_correctness="passed",
                    server_compatibility="supported",
                )
            results.append(
                _case_record(
                    case_id,
                    observation["status"],
                    observation["library_correctness"],
                    observation["server_compatibility"],
                    "response verified" if observation["status"] == "passed" else "KMIP response reported non-success status",
                    elapsed,
                    observation.get("result_status_code"),
                    observation.get("result_reason_code"),
                )
            )
        except Exception:
            # Do not serialize exceptions: they may retain credentials or peer text.
            elapsed = (datetime.now(timezone.utc) - started).total_seconds() * 1000
            results.append(
                _case_record(case_id, "failed", "not_checked", "unknown", "request execution failed", elapsed)
            )
    if driver is not None:
        driver.close()
    return results


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Independent KMIPKit Python interoperability consumer")
    commands = parser.add_subparsers(dest="command", required=True)

    def common(command: argparse.ArgumentParser) -> None:
        command.add_argument("--connection", type=Path, required=True)
        command.add_argument("--catalog", type=Path, default=DEFAULT_CATALOG)
        command.add_argument("--capabilities", type=Path)

    matrix = commands.add_parser('matrix', help='Execute the installed-package API-route matrix against controlled fixtures')
    matrix.add_argument('--output-dir', type=Path, default=Path('build/matrix'))

    plan = commands.add_parser("plan", help="Plan applicable probes without opening a socket")
    common(plan)
    preflight = commands.add_parser("preflight", help="Check local inputs and installed artifact without network access")
    common(preflight)
    run = commands.add_parser("run", help="Run enabled requests against the configured KMIP server")
    common(run)
    run.add_argument("--output-dir", type=Path, default=Path("results"))
    run.add_argument("--enable-mutations", action="store_true")
    run.add_argument("--artifact-manifest", type=Path, default=ARTIFACT_MANIFEST)
    args = parser.parse_args(argv)

    try:
        if args.command == 'matrix':
            from .route_matrix import run_command
            return run_command(args.output_dir)
        connection, connection_bytes = load_connection(args.connection)
        catalog, catalog_bytes = load_contract_catalog(args.catalog)
        profile = _load_profile(args.capabilities, connection["server_id"])
        if args.command == "plan":
            print(json.dumps(_plan(connection, catalog, catalog_bytes), indent=2, sort_keys=True))
            return 0
        local_issues = validate_local_inputs(connection)
        if args.command == "preflight":
            try:
                artifact = validate_artifact(ARTIFACT_MANIFEST)
                package_installed = _is_installed_package()
            except (ValueError, OSError):
                artifact = None
                package_installed = False
            ready = not local_issues and artifact is not None and package_installed
            report = {
                "ready": ready,
                "network_attempted": False,
                "artifact_verified": artifact is not None and package_installed,
                "diagnostics": local_issues + ([] if package_installed else ["pinned KMIPKit package is not installed in this environment"]),
                "plan": _plan(connection, catalog, catalog_bytes),
            }
            print(json.dumps(report, indent=2, sort_keys=True))
            return 0 if ready else 1

        try:
            artifact = validate_artifact(args.artifact_manifest)
        except (ValueError, OSError):
            print("run blocked: installed KMIPKit artifact provenance is unavailable", file=sys.stderr)
            return 1
        if not _is_installed_package():
            print("run blocked: pinned KMIPKit wheel is not installed in the isolated environment", file=sys.stderr)
            return 1
        if local_issues:
            print("run blocked: local connection inputs are incomplete", file=sys.stderr)
            return 1
        results = _run(
            connection,
            connection_bytes,
            catalog,
            catalog_bytes,
            artifact,
            profile,
            args.enable_mutations,
        )
        report = _make_report(
            connection, connection_bytes, catalog, catalog_bytes, artifact, profile, results
        )
        write_interop_reports(report, args.output_dir)
        summary = report["summary"]
        return 0 if summary["failed"] == 0 and summary["not_run"] == 0 and summary["policy_restricted"] == 0 else 1
    except (OSError, ValueError, KeyError, TypeError):
        print("validator input or report contract is invalid", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
