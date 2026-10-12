"""Reconcile compiler-observed Rust paths as blocked manifest records."""

from __future__ import annotations

from typing import Any


def blocked_record(observed: dict[str, Any], manifest: dict[str, Any]) -> dict[str, Any]:
    """Build a blocked record for an observed Rust path without inventing mappings."""
    return {}


def reconcile(manifest: dict[str, Any], report: dict[str, Any]) -> list[dict[str, Any]]:
    """Add exact blocked records for report paths not yet represented in the manifest."""
    return []
