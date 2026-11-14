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
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

# Ensure repository root and current tool dir are in sys.path
_current_dir = Path(__file__).resolve().parent
_repo_root = Path(__file__).resolve().parents[3]
if str(_repo_root) not in sys.path:
    sys.path.insert(0, str(_repo_root))
if str(_current_dir) not in sys.path:
    sys.path.insert(0, str(_current_dir))

try:
    from interop.shared.tools.aggregate_matrix import aggregate_results, generate_html, generate_markdown
except ImportError:
    from aggregate_matrix import aggregate_results, generate_html, generate_markdown  # type: ignore

SUPPORTED_LANGUAGES = ["rust", "c", "java", "python"]


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
    allow_mutations: bool = False,
) -> list[str]:
    """Construct command line arguments for the specified language validator."""
    prof_arg = str(profile_path.resolve())
    cat_arg = str(catalog_path.resolve())
    vec_arg = str(vectors_path.resolve())
    out_arg = str(output_path.resolve())
    conn_arg = str(connection_path.resolve()) if connection_path and connection_path.exists() else ""

    if language == "rust":
        cmd = [
            "cargo", "run", "--manifest-path", str(base_dir / "interop/rust-kmip-validator/Cargo.toml"),
            "--", "--catalog", cat_arg, "--vectors", vec_arg, "--profile", prof_arg, "--output", out_arg,
        ]
        if conn_arg:
            cmd.extend(["--connection", conn_arg])
        return cmd

    if language == "c":
        c_bin = base_dir / "interop/c-kmip-validator/build/kmip_validator_c"
        if os.name == "nt":
            c_bin = c_bin.with_suffix(".exe")
        if not c_bin.exists():
            c_bin = base_dir / "interop/c-kmip-validator/kmip_validator_c"
            if os.name == "nt":
                c_bin = c_bin.with_suffix(".exe")
        cmd = [str(c_bin), "run", "--catalog", cat_arg, "--profile", prof_arg, "--output", out_arg]
        if conn_arg:
            cmd.extend(["--connection", conn_arg])
        if allow_mutations:
            cmd.append("--allow-mutations")
        return cmd

    if language == "java":
        jar_path = base_dir / "interop/java-kmip-validator/build/libs/kmipkit-java-validator.jar"
        if jar_path.exists():
            cmd = ["java", "-jar", str(jar_path), "--catalog", cat_arg, "--vectors", vec_arg, "--profile", prof_arg, "--output", out_arg]
        else:
            cmd = [
                "gradle", "-p", str(base_dir / "interop/java-kmip-validator"),
                "run", f"--args=--catalog {cat_arg} --vectors {vec_arg} --profile {prof_arg} --output {out_arg}"
            ]
        if conn_arg:
            cmd.extend(["--connection", conn_arg])
        return cmd

    if language == "python":
        py_cli = base_dir / "interop/python-kmip-validator/src/kmipkit_validator/cli.py"
        cmd = [
            sys.executable, str(py_cli),
            "--catalog", cat_arg, "--vectors", vec_arg, "--profile", prof_arg, "--output", out_arg,
        ]
        if conn_arg:
            cmd.extend(["--connection", conn_arg])
        return cmd

    raise ValueError(f"Unsupported language: {language}")


def run_validator(
    language: str,
    cmd: list[str],
    output_path: Path,
    dry_run: bool = False,
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
        proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
        duration = (datetime.datetime.now(datetime.timezone.utc) - start_time).total_seconds()
        return {
            "language": language,
            "status": "COMPLETED" if proc.returncode == 0 else "FAILED",
            "command": " ".join(cmd),
            "output_file": str(output_path),
            "exit_code": proc.returncode,
            "duration_seconds": duration,
            "stdout": proc.stdout[-500:] if proc.stdout else "",
            "stderr": proc.stderr[-500:] if proc.stderr else "",
        }
    except Exception as e:
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
    allow_mutations: bool = False,
) -> dict[str, Any]:
    """Orchestrate multi-language validator execution and aggregation."""
    output_dir.mkdir(parents=True, exist_ok=True)
    toolchains = check_toolchain_prerequisites()

    commands = {}
    result_files = []
    execution_plans = []

    for lang in languages:
        out_file = output_dir / f"results-{lang}.json"
        result_files.append(out_file)
        cmd = build_command_for_language(
            language=lang,
            base_dir=base_dir,
            profile_path=profile_path,
            connection_path=connection_path,
            catalog_path=catalog_path,
            vectors_path=vectors_path,
            output_path=out_file,
            allow_mutations=allow_mutations,
        )
        commands[lang] = cmd

    runs = []
    if parallel and not dry_run:
        with concurrent.futures.ThreadPoolExecutor(max_workers=len(languages)) as executor:
            future_to_lang = {
                executor.submit(run_validator, lang, commands[lang], output_dir / f"results-{lang}.json", dry_run): lang
                for lang in languages
            }
            for future in concurrent.futures.as_completed(future_to_lang):
                runs.append(future.result())
    else:
        for lang in languages:
            res = run_validator(lang, commands[lang], output_dir / f"results-{lang}.json", dry_run)
            runs.append(res)

    # If dry-run, we generate sample aggregated output or skip
    aggregated = None
    if not dry_run:
        valid_files = [f for f in result_files if f.exists()]
        if valid_files:
            aggregated = aggregate_results(valid_files)
            # Write master-matrix.json
            with open(output_dir / "master-matrix.json", "w", encoding="utf-8") as f:  # NOSONAR
                json.dump(aggregated, f, indent=2)
            # Write master-report.md
            with open(output_dir / "master-report.md", "w", encoding="utf-8") as f:  # NOSONAR
                f.write(generate_markdown(aggregated))
            # Write master-report.html
            with open(output_dir / "master-report.html", "w", encoding="utf-8") as f:  # NOSONAR
                f.write(generate_html(aggregated))

    return {
        "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "dry_run": dry_run,
        "toolchains": toolchains,
        "languages": languages,
        "runs": runs,
        "aggregated": aggregated,
        "output_dir": str(output_dir),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="KMIPKit Cross-Language Orchestrator")
    parser.add_argument("--languages", type=str, default="rust,c,java,python", help="Comma-separated languages (default: rust,c,java,python)")
    parser.add_argument("--profile", type=Path, default=Path("interop/shared/profiles/generic-kmip-2.1.json"), help="KMS profile JSON")
    parser.add_argument("--connection", type=Path, help="Connection configuration JSON")
    parser.add_argument("--catalog", type=Path, default=Path("interop/shared/catalog/kmipkit-1.0-test-catalog.json"), help="Test catalog JSON")
    parser.add_argument("--vectors", type=Path, default=Path("interop/shared/vectors/canonical-vectors.json"), help="Test vectors JSON")
    parser.add_argument("--output-dir", type=Path, help="Directory to store outputs (default: interop/results/<timestamp>)")
    parser.add_argument("--parallel", action="store_true", help="Execute validators concurrently")
    parser.add_argument("--dry-run", action="store_true", help="Print execution plan without executing")
    parser.add_argument("--allow-mutations", action="store_true", help="Pass explicit mutating-operation opt-in to validators that support it")

    args = parser.parse_args()

    if not args.dry_run and (not args.connection or not args.connection.is_file()):
        parser.error("--connection must name a configured dedicated test server when executing validators")

    base_dir = Path(__file__).resolve().parents[3]
    langs = [l.strip().lower() for l in args.languages.split(",") if l.strip().lower() in SUPPORTED_LANGUAGES]
    if not langs:
        print(f"Error: No valid languages specified. Choose from: {SUPPORTED_LANGUAGES}", file=sys.stderr)
        sys.exit(1)

    if args.output_dir:
        out_dir = args.output_dir.resolve()
    else:
        ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        out_dir = base_dir / f"interop/results/{ts}"

    summary = orchestrate_interop_run(
        languages=langs,
        base_dir=base_dir,
        profile_path=args.profile,
        catalog_path=args.catalog,
        vectors_path=args.vectors,
        output_dir=out_dir,
        connection_path=args.connection,
        parallel=args.parallel,
        dry_run=args.dry_run,
        allow_mutations=args.allow_mutations,
    )

    print("\n=======================================================")
    print("KMIPKit Cross-Language Orchestrator Summary")
    print("=======================================================")
    print(f"Dry Run:    {summary['dry_run']}")
    print(f"Output Dir: {summary['output_dir']}")
    print("Languages planned/executed:")
    for r in summary["runs"]:
        print(f" - [{r['language'].upper()}] {r['status']} -> {r.get('output_file')}")

    if summary.get("aggregated"):
        print("\nAggregated Master Reports generated:")
        print(f" - Markdown: {out_dir / 'master-report.md'}")
        print(f" - HTML:     {out_dir / 'master-report.html'}")
        print(f" - JSON:     {out_dir / 'master-matrix.json'}")
    print("=======================================================\n")


if __name__ == "__main__":
    main()
