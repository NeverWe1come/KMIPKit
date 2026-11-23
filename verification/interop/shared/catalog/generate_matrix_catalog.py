#!/usr/bin/env python3
"""Command-line entry point for the KMIP 2.0 and 2.1 normative matrix catalog."""

from pathlib import Path
import sys

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[4]))

from verification.interop.shared.tools.matrix.kmip2x_catalog import main


if __name__ == "__main__":
    main()
