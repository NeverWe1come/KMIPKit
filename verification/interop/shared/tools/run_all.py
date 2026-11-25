#!/usr/bin/env python3
"""Unified Multi-Language Interoperability Orchestrator for KMIPKit.

Executes test suites across Rust, C, Java, and Python validators, collects
standardized validation-results.json outputs, and generates consolidated
Markdown and Interactive HTML master reports.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime
import hashlib
import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# Ensure repository root and current tool dir are in sys.path
_current_dir = Path(__file__).resolve().parent
_repo_root = Path(__file__).resolve().parents[4]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))
if str(_current_dir) not in sys.path:
    sys.path.insert(0, str(_current_dir))

try:
    from verification.interop.shared.tools.aggregate_matrix import aggregate_results, generate_html, generate_markdown
    from verification.interop.shared.tools.campaign import read_validated
    from verification.interop.shared.tools.cosmian_minimum import build_selection as build_cosmian_selection
    from verification.interop.shared.tools.operation_catalog import build_catalog, build_normative_catalog, select_cases
except ImportError:
    from aggregate_matrix import aggregate_results, generate_html, generate_markdown  # type: ignore
    from campaign import read_validated  # type: ignore
    from cosmian_minimum import build_selection as build_cosmian_selection
    from operation_catalog import build_catalog, build_normative_catalog, select_cases  # type: ignore

SUPPORTED_LANGUAGES = ["rust", "c", "java", "python"]


@dataclass(frozen=True)
class Listener:
    address: str
    certificate: Path
    key: Path


def check_toolchain_prerequisites() -> dict[str, bool]:
    """Check availability of language compilers and runtimes."""
    return {
        "rust": shutil.which("cargo") is not None,
        "c": shutil.which("cc") is not None or shutil.which("gcc") is not None or shutil.which("clang") is not None,
        "java": shutil.which("java") is not None,
        "python": shutil.which("python3") is not None or shutil.which("python") is not None,
    }


def build_command_for_language(
    language: str,
    base_dir: Path,
    profile_path: Path,
    connection_path: Path | None,
    catalog_path: Path,
    vectors_path: Path,
    output_path: Path,
    *,
    project_dir: Path | None = None,
    python: Path | None = None,
    enable_mutations: bool = False,
    c_catalog_override: bool = False,
    listener: Listener | None = None,
) -> list[str]:
    """Use the installed consumer's run command and actual report location."""
    del vectors_path  # The installed consumers load their own canonical vectors.
    project = (project_dir or base_dir / f"verification/interop/{language}-kmip-validator").resolve()
    out_file = output_path.resolve()
    command = ["run"]
    if listener is not None:
        command.extend(("--listen", listener.address, "--listener-cert", str(listener.certificate.resolve()),
                        "--listener-key", str(listener.key.resolve()),
                        "--listener-ready-file", str(out_file.parent / "listener-address.txt")))
    if connection_path is not None:
        command.extend(("--connection", str(connection_path.resolve())))
    if language != "c" or c_catalog_override:
        command.extend(("--catalog", str(catalog_path.resolve())))
    if language in ("rust", "c", "python"):
        command.extend(("--capabilities", str(profile_path.resolve())))
    if language == "rust":
        command.extend(("--output", str(out_file)))
        return [str(project / "target/debug/rust-kmip-validator"), *command]
    if language == "c":
        command.extend(("--output", str(out_file)))
        if enable_mutations:
            command.append("--allow-mutations")
        return [str(project / "build/kmip_validator_c"), *command]
    if language in ("java", "python"):
        command.extend(("--output-dir", str(out_file.parent)))
        if enable_mutations:
            command.append("--enable-mutations")
        if language == "java":
            command.extend(("--artifact-manifest", str(project / "build/artifact-manifest.json")))
            return [str(project / "build/install/java-kmip-validator/bin/java-kmip-validator"), *command]
        return [str(python or sys.executable), "-m", "kmipkit_validator", *command]
    raise ValueError(f"Unsupported language: {language}")


def run_validator(
    language: str,
    cmd: list[str],
    output_path: Path,
    dry_run: bool = False,
    *,
    project_dir: Path | None = None,
) -> dict[str, Any]:
    """Execute single validator process and return status summary."""
    start_time = datetime.datetime.now(datetime.timezone.utc)
    if dry_run:
        return {
            "language": language,
            "status": "PLANNED",
            "command": " ".join(cmd),
            "output_file": str(output_path),
            "exit_code": 0,
            "duration_seconds": 0.0,
        }

    try:
        env = None
        if language == "python" and project_dir is not None:
            env = dict(os.environ, PYTHONPATH=str(project_dir / "src"))
        cwd = output_path.parent if language == "python" else None
        proc = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False, env=env, cwd=cwd)
        duration = (datetime.datetime.now(datetime.timezone.utc) - start_time).total_seconds()
        status = "FAILED"
        if output_path.is_file():
            run_summary = read_validated(output_path, "validation-results.schema.json")["summary"]
            if not run_summary["failed"]:
                status = "INCOMPLETE" if run_summary["not_run"] or run_summary["policy_restricted"] else "COMPLETED"
        return {
            "language": language,
            "status": status,
            "command": " ".join(cmd),
            "output_file": str(output_path),
            "exit_code": proc.returncode,
            "duration_seconds": duration,
        }
    except (OSError, ValueError) as e:
        duration = (datetime.datetime.now(datetime.timezone.utc) - start_time).total_seconds()
        return {
            "language": language,
            "status": "ERROR",
            "command": " ".join(cmd),
            "output_file": str(output_path),
            "exit_code": -1,
            "duration_seconds": duration,
            "error": str(e),
        }


def orchestrate_interop_run(
    languages: list[str],
    base_dir: Path,
    profile_path: Path,
    catalog_path: Path,
    vectors_path: Path,
    output_dir: Path,
    connection_path: Path | None = None,
    parallel: bool = False,
    dry_run: bool = False,
    *,
    projects: dict[str, Path] | None = None,
    python: Path | None = None,
    enable_mutations: bool = False,
    c_catalog_override: bool = False,
    listener: Listener | None = None,
) -> dict[str, Any]:
    """Launch one installed consumer per language and collect fresh reports."""
    if not dry_run and (connection_path is None or not connection_path.is_file()):
        raise ValueError("a readable --connection is required for live execution")
    if not languages or len(languages) != len(set(languages)):
        raise ValueError("select each language at most once")
    if not dry_run:
        connection = read_validated(connection_path, "server-connection.schema.json")
        if "c" in languages and not connection["execution_policy"]["probe_unsupported"]:
            raise ValueError("the C validator requires probe_unsupported=true in the connection")
        if connection["execution_policy"]["allow_mutations"] and not enable_mutations:
            raise ValueError("the connection permits mutations; pass --enable-mutations to run its recipes")
        if enable_mutations and parallel:
            raise ValueError("run mutation recipes sequentially on a shared test server")
    output_dir = output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    toolchains = check_toolchain_prerequisites()
    commands = {}
    result_files = {}
    project_dirs = {}
    for lang in languages:
        project_dirs[lang] = (projects or {}).get(lang, base_dir / f"verification/interop/{lang}-kmip-validator").resolve()
        out_file = output_dir / lang / "results.json"
        result_files[lang] = out_file
        commands[lang] = build_command_for_language(
            lang, base_dir, profile_path, connection_path, catalog_path, vectors_path, out_file,
            project_dir=project_dirs[lang], python=python, enable_mutations=enable_mutations,
            c_catalog_override=c_catalog_override, listener=listener,
        )
    if not dry_run:
        # Reserve isolated, fresh paths before sending any requests.
        for out_file in result_files.values():
            out_file.parent.mkdir()

    def run(lang):
        return run_validator(
            lang, commands[lang], result_files[lang], dry_run, project_dir=project_dirs[lang],
        )

    if parallel and not dry_run:
        with concurrent.futures.ThreadPoolExecutor(max_workers=len(languages)) as executor:
            runs = list(executor.map(run, languages))
    else:
        runs = [run(lang) for lang in languages]

    aggregated = None if dry_run else write_master_reports(output_dir, report_files(runs))

    return {
        "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "dry_run": dry_run,
        "toolchains": toolchains,
        "languages": languages,
        "runs": runs,
        "aggregated": aggregated,
        "output_dir": str(output_dir),
    }


def report_files(runs):
    return [Path(run["output_file"]) for run in runs
            if run["status"] != "ERROR" and Path(run["output_file"]).is_file()]


def write_master_reports(output_dir, paths):
    if not paths:
        return None
    report = aggregate_results(paths)
    (output_dir / "master-matrix.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    (output_dir / "master-report.md").write_text(generate_markdown(report), encoding="utf-8")
    (output_dir / "master-report.html").write_text(generate_html(report), encoding="utf-8")
    return report


def incomplete_case_ids(catalog, results):
    """Return case IDs without the type of evidence required by their recipe."""
    expected = {case["case_id"]: case for case in catalog["test_cases"]}
    observed = {}
    for case in results["cases"]:
        case_id = case["case_id"]
        if case_id not in expected or case_id in observed:
            raise ValueError("result contains an unexpected or duplicate case ID")
        observed[case_id] = case

    return sorted(
        case_id for case_id, recipe in expected.items()
        if recipe["execution"]["status"] != "not_applicable"
        and not complete_observation(recipe, observed.get(case_id, {}))
    )


def complete_observation(recipe, observation):
    if observation.get("library_correctness") != "passed":
        return False
    status = observation.get("status")
    result_code = observation.get("result_status_code")
    requires = recipe["requires"]
    if recipe["execution"]["mode"] == "local_model" or requires.get("expected_status") == "NotSent":
        return status == "passed" and result_code is None
    if requires.get("expected_status") == "OperationFailed":
        reasons = requires.get("expected_result_reasons", [])
        return (status == "passed" and result_code == 1
                and (not reasons or observation.get("result_reason_code") in reasons))
    if status == "server_unsupported":
        return result_code == 1
    if status != "passed":
        return False
    if result_code is not None:
        return result_code == 0
    return (requires.get("workflow_type") == "server_to_client"
            and observation.get("server_compatibility") == "supported")


def validate_completion_report(evidence, language, connection, catalog_sha256):
    expected = {
        "language": language, "protocol_version": connection["protocol_version"],
        "catalog_sha256": catalog_sha256,
    }
    if (any(evidence[key] != value for key, value in expected.items())
            or evidence["server"]["id"] != connection["server_id"]
            or any(evidence["execution"][axis] != connection[axis]
                   for axis in ("transport", "encoding", "api_level"))):
        raise ValueError("result identity does not match the requested catalog, server, or execution route")


def main() -> None:
    parser = argparse.ArgumentParser(description="KMIPKit Cross-Language Orchestrator")
    parser.add_argument("--languages", type=str, default="rust,c,java,python", help="Comma-separated languages (default: rust,c,java,python)")
    parser.add_argument("--profile", type=Path, default=Path("verification/interop/shared/profiles/generic-kmip-2.1.json"), help="KMS profile JSON")
    parser.add_argument("--connection", type=Path, action="append", default=[],
                        help="Connection configuration JSON. Repeat for other versions or execution settings.")
    parser.add_argument("--all-operations", action="store_true",
                        help="Generate the complete operation catalog for each connection version")
    parser.add_argument("--require-complete", action="store_true",
                        help="Fail if any applicable selected case has missing or incomplete evidence")
    scope = parser.add_mutually_exclusive_group()
    scope.add_argument("--normative", action="store_true",
                       help="With --all-operations, select all KMIP 2.x normative cases and existing variants")
    scope.add_argument("--cosmian-minimum", action="store_true",
                        help="With --all-operations, select the published Cosmian/Eviden KMIP 2.x minimum")
    scope.add_argument("--with-variants", action="store_true",
                        help="With --all-operations, also run existing version-specific conformance cases")
    parser.add_argument("--operation", action="append", default=[], help="With --all-operations, select an exact operation name")
    parser.add_argument("--case", action="append", default=[], help="With --all-operations, select an exact case ID")
    parser.add_argument("--catalog", type=Path, default=Path("verification/interop/shared/catalog/kmipkit-1.0-test-catalog.json"),
                        help="Test catalog for Rust, Java and Python. C uses its build-time catalog unless --all-operations is set.")
    parser.add_argument("--vectors", type=Path, default=Path("verification/interop/shared/vectors/canonical-vectors.json"), help="Test vectors JSON")
    parser.add_argument("--output-dir", type=Path, help="Directory to store outputs (default: verification/interop/results/<timestamp>)")
    parser.add_argument("--parallel", action="store_true", help="Execute validators concurrently")
    parser.add_argument("--dry-run", action="store_true", help="Print execution plan without executing")
    parser.add_argument("--project", nargs=2, metavar=("LANGUAGE", "PATH"), action="append", default=[],
                        help="Use an installed consumer from a separate checkout")
    parser.add_argument("--python", type=Path, help="Python interpreter with the installed KMIPKit wheel")
    parser.add_argument("--enable-mutations", action="store_true", help="Opt in to mutating recipes explicitly")
    parser.add_argument("--listen", help="Address and port for server-initiated operations")
    parser.add_argument("--listener-cert", type=Path, help="PEM certificate for incoming mutual TLS")
    parser.add_argument("--listener-key", type=Path, help="PEM private key for incoming mutual TLS")
    parser.add_argument("--campaign", action="store_true", help="Include the planned obligations and catalog evidence in a campaign report")

    args = parser.parse_args()

    base_dir = Path(__file__).resolve().parents[4]
    langs = [name.strip().lower() for name in args.languages.split(",")]
    projects = {name: Path(path) for name, path in args.project}
    if not set(langs).issubset(SUPPORTED_LANGUAGES):
        parser.error(f"select languages from {', '.join(SUPPORTED_LANGUAGES)}")
    if len(projects) != len(args.project) or not set(projects).issubset(langs):
        parser.error("give one project path per selected language")
    if args.all_operations and not args.connection:
        parser.error("--all-operations requires a connection configuration")
    if (args.with_variants or args.cosmian_minimum or args.normative or args.operation or args.case) and not args.all_operations:
        parser.error("--with-variants, --cosmian-minimum, --normative, --operation and --case require --all-operations")
    if args.require_complete and (args.dry_run or not args.all_operations):
        parser.error("--require-complete requires a live --all-operations selection")
    if args.campaign and args.dry_run:
        parser.error("--campaign requires executing the validators")

    listener = None
    if args.listen or args.listener_cert or args.listener_key:
        if not (args.listen and args.listener_cert and args.listener_key and args.connection):
            parser.error("--listen requires --listener-cert, --listener-key and --connection")
        if args.parallel:
            parser.error("use a sequential run when sharing one incoming listener address")
        listener = Listener(args.listen, args.listener_cert, args.listener_key)

    if args.output_dir:
        out_dir = args.output_dir.resolve()
    else:
        ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        out_dir = base_dir / f"verification/interop/results/{ts}"

    connections = args.connection or [None]
    results = []
    versions = []
    operation_catalogs = {}
    gaps = False
    completion = []
    try:
        if args.connection and (not args.dry_run or args.all_operations):
            versions = list(dict.fromkeys(
                read_validated(path, "server-connection.schema.json")["protocol_version"]
                for path in args.connection
            ))
        if args.all_operations:
            catalog_dir = out_dir / "catalogs"
            selections = {}
            catalog_dir.mkdir(parents=True, exist_ok=True)
            for version in versions:
                if args.cosmian_minimum:
                    catalog, scope = build_cosmian_selection(base_dir, version)
                    with (catalog_dir / f"kmip-{version}-cosmian-scope.json").open("x", encoding="utf-8") as stream:
                        json.dump(scope, stream, indent=2, ensure_ascii=False)
                    if args.require_complete and not (args.operation or args.case):
                        unmapped = [row for row in scope["rows"] if row["disposition"] not in {
                            "out_of_scope", "server_cases_selected",
                        }]
                        if unmapped:
                            gaps = True
                            print(f"KMIP {version}: {len(unmapped)} Cosmian rows have no mapped server case",
                                  file=sys.stderr)
                elif args.normative:
                    catalog, _ = build_normative_catalog(base_dir, version)
                else:
                    catalog = build_catalog(base_dir, version, include_variants=args.with_variants)
                selections[version] = select_cases(catalog, operations=args.operation, case_ids=args.case)
                catalog = catalog_dir / f"kmip-{version}-operations.json"
                with catalog.open("x", encoding="utf-8") as stream:
                    json.dump(selections[version], stream, indent=2, ensure_ascii=False)
                operation_catalogs[version] = catalog
        for index, connection in enumerate(connections, 1):
            catalog_path = args.catalog
            active_listener = None
            if args.all_operations or listener:
                configuration = read_validated(connection, "server-connection.schema.json")
                version = configuration["protocol_version"]
                if args.all_operations:
                    catalog_path = operation_catalogs[version]
                if (listener and version in {"2.0", "2.1"} and configuration["api_level"] == "generic-ttlv"
                        and configuration["transport"] == "raw_tls" and configuration["encoding"] == "TTLV"):
                    active_listener = listener
            destination = out_dir if len(connections) == 1 else out_dir / f"connection-{index}"
            result = orchestrate_interop_run(
                languages=langs,
                base_dir=base_dir,
                profile_path=args.profile,
                catalog_path=catalog_path,
                vectors_path=args.vectors,
                output_dir=destination,
                connection_path=connection,
                parallel=args.parallel,
                dry_run=args.dry_run,
                projects=projects,
                python=args.python.absolute() if args.python else None,
                enable_mutations=args.enable_mutations,
                c_catalog_override=args.all_operations, listener=active_listener,
            )
            results.extend(result["runs"])
            if args.require_complete:
                catalog_digest = hashlib.sha256(catalog_path.read_bytes()).hexdigest()
                for run in result["runs"]:
                    entry = {
                        "language": run["language"],
                        "protocol_version": version,
                        "api_level": configuration["api_level"],
                        "connection_index": index,
                        "status": "completed",
                    }
                    completion.append(entry)
                    evidence_path = Path(run["output_file"])
                    if not evidence_path.is_file() or run["status"] == "ERROR":
                        entry["status"] = "missing_report"
                        gaps = True
                        continue
                    try:
                        evidence = read_validated(evidence_path, "validation-results.schema.json")
                        validate_completion_report(evidence, run["language"], configuration, catalog_digest)
                        missing = incomplete_case_ids(selections[version], evidence)
                    except ValueError:
                        entry["status"] = "invalid_report"
                        gaps = True
                        continue
                    entry["missing_case_ids"] = missing
                    if missing:
                        entry["status"] = "incomplete"
                        gaps = True
                        print(f"[{run['language'].upper()}] KMIP {version} {configuration['api_level']}: "
                              f"{len(missing)} applicable cases without complete evidence", file=sys.stderr)
        if args.require_complete:
            (out_dir / "completion.json").write_text(json.dumps(completion, indent=2) + "\n", encoding="utf-8")

        paths = [] if args.dry_run else report_files(results)
        if len(connections) > 1:
            write_master_reports(out_dir, paths)
    except (OSError, ValueError, KeyError) as error:
        parser.error(str(error))

    for run in results:
        print(f"[{run['language'].upper()}] {run['status']}  {run['output_file']}")
    if paths:
        print(f"Combined report  {out_dir / 'master-report.html'}")
    campaign_status = 0
    if args.campaign and paths:
        command = [sys.executable, str(_current_dir / "campaign.py"),
                   "--output-dir", str(out_dir / "campaign"), "--profile", str(args.profile.resolve())]
        for version in versions:
            command.extend(("--version", version))
        for lang in langs:
            command.extend(("--language", lang))
        for path in operation_catalogs.values():
            command.extend(("--catalog-source", str(path)))
        for path in paths:
            command.extend(("--live-report", str(path)))
        campaign_status = subprocess.run(command, check=False).returncode
    if not args.dry_run and (len(paths) != len(results) or any(run["exit_code"] for run in results)):
        sys.exit(1)
    sys.exit(1 if gaps else campaign_status)


if __name__ == "__main__":
    main()
