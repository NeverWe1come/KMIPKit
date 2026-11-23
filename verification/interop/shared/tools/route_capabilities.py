#!/usr/bin/env python3
"""Command-line entry point for the matrix public API route capability manifest."""

from pathlib import Path
import sys

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[4]))

from verification.interop.shared.tools.matrix.route_capabilities import main


if __name__ == "__main__":
    main()
