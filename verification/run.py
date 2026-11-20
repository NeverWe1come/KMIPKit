#!/usr/bin/env python3
"""Discover and launch KMIPKit verification workflows."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

COMMANDS = {
    "security-smoke": ([sys.executable, "verification/security/run_campaign.py", "--smoke"], ROOT),
    "interop-plan": ([sys.executable, "verification/interop/shared/tools/run_all.py", "--dry-run"], ROOT),
    "python-tests": ([sys.executable, "-m", "pytest", "-q", "bindings/python/tests"], ROOT),
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("check", nargs="?", choices=sorted(COMMANDS))
    parser.add_argument("args", nargs=argparse.REMAINDER)
    arguments = parser.parse_args()
    if arguments.check is None:
        print("Available verification checks:")
        for name in sorted(COMMANDS):
            print(f"  {name}")
        print("Run python verification/run.py <check> --help for tool options.")
        return 0
    command, cwd = COMMANDS[arguments.check]
    environment = os.environ.copy()
    return subprocess.run([*command, *arguments.args], cwd=cwd, env=environment, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
