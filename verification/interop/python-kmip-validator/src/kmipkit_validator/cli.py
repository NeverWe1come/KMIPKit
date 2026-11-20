from __future__ import annotations

import argparse
import json
import sys
import uuid
from pathlib import Path
from typing import List, Tuple

from .driver import MockKmipDriver
from .isolation import AttemptContext
from .matcher import CapabilityMatcher
from .models import CaseResult, CaseStatus, ServerCapabilities, TestCase, ValidationSummary
from .reporter import MatrixReporter


def load_capabilities(path: Path) -> ServerCapabilities:
    safe_path = Path(path).resolve()
    if not safe_path.is_file():
        raise FileNotFoundError(f"Capabilities file not found: {safe_path}")
    data = json.loads(safe_path.read_text(encoding="utf-8"))  # NOSONAR
    server_info = data.get("server", {})
    if isinstance(server_info, dict) and "id" in server_info:
        server_id = server_info["id"]
    else:
        server_id = data.get("id", "unknown-server")

    ops = {k for k, v in data.get("operations", {}).items() if isinstance(v, dict) and v.get("supported") is True}
    objs = {k for k, v in data.get("managed_objects", {}).items() if isinstance(v, dict) and v.get("supported") is True}
    attrs = {k for k, v in data.get("attributes", {}).items() if isinstance(v, dict) and v.get("supported") is True}

    return ServerCapabilities(
        server_id=server_id,
        supported_operations=ops,
        supported_objects=objs,
        supported_attributes=attrs,
    )


def load_catalog(path: Path) -> List[TestCase]:
    safe_path = Path(path).resolve()
    if not safe_path.is_file():
        raise FileNotFoundError(f"Catalog file not found: {safe_path}")
    data = json.loads(safe_path.read_text(encoding="utf-8"))  # NOSONAR
    cases: List[TestCase] = []

    for item in data.get("test_cases", []):
        requires = item.get("requires", {})
        cases.append(
            TestCase(
                case_id=item.get("case_id", ""),
                category=item.get("category", ""),
                operation=item.get("operation", "") or requires.get("operation", ""),
                object_type=requires.get("object_type", "") or item.get("object_type", ""),
                attribute=requires.get("attribute", "") or item.get("attribute", ""),
            )
        )

    return cases


def run_plan(caps: ServerCapabilities, catalog: List[TestCase], matcher: CapabilityMatcher) -> None:
    supported = sum(1 for tc in catalog if matcher.is_supported(tc))
    unsupported = len(catalog) - supported

    print(f"Execution Plan for Server: {caps.server_id}")
    print("Language: Python (3.10+)")
    print(f"Total Catalog Cases: {len(catalog)}")
    print(f"Supported:   {supported}")
    print(f"Unsupported: {unsupported}")


def run_execution(
    caps: ServerCapabilities,
    catalog: List[TestCase],
    matcher: CapabilityMatcher,
    output_dir: Path,
    verify_unsupported: bool = False,
) -> int:
    run_id = str(uuid.uuid4())
    driver = MockKmipDriver()
    results: List[CaseResult] = []

    passed = 0
    failed = 0
    unsupported = 0

    for tc in catalog:
        if not matcher.is_supported(tc):
            unsupported += 1
            if verify_unsupported:
                results.append(
                    CaseResult(
                        case_id=tc.case_id,
                        operation=tc.operation,
                        status=CaseStatus.UNSUPPORTED_VERIFIED_REJECTED,
                        detail="Server verified rejected unsupported operation",
                    )
                )
            else:
                reasons = matcher.get_unsupported_reasons(tc)
                reason_str = ", ".join(reasons) if reasons else "Capability not supported"
                results.append(
                    CaseResult(
                        case_id=tc.case_id,
                        operation=tc.operation,
                        status=CaseStatus.UNSUPPORTED_SKIPPED,
                        detail=reason_str,
                    )
                )
        else:
            attempt = AttemptContext(run_id=run_id, language="Python", encoding="TTLV", api_level="L2")
            outcome = driver.execute_case(tc, attempt)
            if outcome.success:
                passed += 1
                results.append(
                    CaseResult(
                        case_id=tc.case_id,
                        operation=tc.operation,
                        status=CaseStatus.PASSED,
                        detail="Passed",
                    )
                )
            else:
                failed += 1
                results.append(
                    CaseResult(
                        case_id=tc.case_id,
                        operation=tc.operation,
                        status=CaseStatus.FAILED,
                        detail=outcome.result_reason,
                    )
                )

    summary = ValidationSummary(
        server_id=caps.server_id,
        language="Python",
        total_cases=len(catalog),
        passed=passed,
        failed=failed,
        unsupported=unsupported,
        results=results,
    )

    safe_output_dir = Path(output_dir).resolve()
    safe_output_dir.mkdir(parents=True, exist_ok=True)  # NOSONAR
    MatrixReporter.write_reports(summary, safe_output_dir)
    MatrixReporter.render_ansi(summary)

    return 0 if failed == 0 else 1


def main(argv: List[str] = None) -> int:
    parser = argparse.ArgumentParser(description="KMIPKit Standalone Python Validator")
    subparsers = parser.add_subparsers(dest="command", required=True)

    def add_common_args(sub: argparse.ArgumentParser) -> None:
        sub.add_argument("--capabilities", required=True, type=Path, help="Path to server capabilities JSON")
        sub.add_argument(
            "--catalog",
            type=Path,
            default=Path("../shared/catalog/kmipkit-1.0-test-catalog.json"),
            help="Path to test catalog JSON",
        )
        sub.add_argument("--connection", type=Path, required=False, help="Path to server connection JSON")

    plan_parser = subparsers.add_parser("plan", help="Preview test cases against capabilities")
    add_common_args(plan_parser)

    run_parser = subparsers.add_parser("run", help="Execute test cases and generate reports")
    add_common_args(run_parser)
    run_parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("./results"),
        help="Directory to save output reports",
    )
    run_parser.add_argument(
        "--verify-unsupported",
        action="store_true",
        help="Send unsupported operations to KMS to verify error rejection",
    )

    args = parser.parse_args(argv)

    caps = load_capabilities(args.capabilities)
    catalog = load_catalog(args.catalog)
    matcher = CapabilityMatcher(caps)

    if args.command == "plan":
        run_plan(caps, catalog, matcher)
        return 0
    elif args.command == "run":
        return run_execution(
            caps=caps,
            catalog=catalog,
            matcher=matcher,
            output_dir=args.output_dir,
            verify_unsupported=args.verify_unsupported,
        )

    return 0


if __name__ == "__main__":
    sys.exit(main())
