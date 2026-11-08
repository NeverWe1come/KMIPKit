"""Shared command-line workflow for reviewed API-manifest reconcilers."""

from __future__ import annotations

import argparse
import json
import sys
from collections.abc import Callable, Sequence
from pathlib import Path
from typing import Any

Manifest = dict[str, Any]
Reconciler = Callable[[Manifest, Manifest], Manifest]
Summary = Callable[[int, int, bool], str]


def run_reconciliation_cli(
    reconcile: Reconciler,
    *,
    spec_id: str,
    description: str | None,
    summarize: Summary,
    argv: Sequence[str] | None = None,
) -> int:
    """Load reviewed inputs, optionally write a reconciled manifest, and report status."""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument(
        "--write",
        action="store_true",
        help="update the reviewed manifest input",
    )
    args = parser.parse_args(argv)
    manifest_path = args.repo_root / "specification/api/public-api.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        report = json.loads(args.report.read_text(encoding="utf-8"))
        reconciled = reconcile(manifest, report)
    except (OSError, ValueError) as error:
        print(f"{spec_id} manifest reconciliation failed: {error}", file=sys.stderr)
        return 2

    declarations_added = len(reconciled["publicDeclarations"]) - len(
        manifest["publicDeclarations"]
    )
    exclusions_added = len(reconciled["approvedExclusions"]) - len(
        manifest.get("approvedExclusions", [])
    )
    print(summarize(declarations_added, exclusions_added, args.write))
    if args.write:
        manifest_path.write_text(
            json.dumps(reconciled, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
    return 0
